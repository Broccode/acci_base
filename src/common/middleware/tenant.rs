use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::response::Response;
use tower::{Layer, Service};

#[allow(unused_imports)]
use crate::domain::tenant::{Tenant, TenantContext, TenantService};

#[cfg(test)]
use crate::common::error::AppError;

const TENANT_HEADER: &str = "X-Tenant-ID";

#[allow(clippy::disallowed_methods)]
fn create_error_response(status: StatusCode, message: impl Into<String>) -> Response {
    Response::builder()
        .status(status)
        .body(Body::from(message.into()))
        .expect("Failed to create error response")
}

#[derive(Clone)]
pub struct TenantLayer {
    tenant_service: Arc<dyn TenantService>,
}

impl TenantLayer {
    #[allow(dead_code)]
    pub fn new(tenant_service: Arc<dyn TenantService>) -> Self {
        Self { tenant_service }
    }
}

impl<S> Layer<S> for TenantLayer {
    type Service = TenantMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        TenantMiddleware {
            inner: service,
            tenant_service: self.tenant_service.clone(),
        }
    }
}

#[derive(Clone)]
pub struct TenantMiddleware<S> {
    inner: S,
    tenant_service: Arc<dyn TenantService>,
}

impl<S, B> Service<Request<B>> for TenantMiddleware<S>
where
    S: Service<Request<B>, Response = Response> + Send + Clone + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<B>) -> Self::Future {
        let tenant_service = self.tenant_service.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // 1. Versuche zuerst den Tenant über den Header zu ermitteln
            let tenant_result = match request.headers().get(TENANT_HEADER) {
                Some(tenant_id) => match tenant_id.to_str() {
                    Ok(id) => tenant_service.find_by_id(id).await,
                    Err(_) => {
                        return Ok(create_error_response(
                            StatusCode::BAD_REQUEST,
                            "Invalid tenant header value",
                        ));
                    }
                },
                None => {
                    // Wenn kein Header vorhanden, versuche es über die Domain
                    match request
                        .headers()
                        .get(header::HOST)
                        .and_then(|h| h.to_str().ok())
                    {
                        Some(domain) => tenant_service.find_by_domain(domain).await,
                        None => {
                            return Ok(create_error_response(
                                StatusCode::BAD_REQUEST,
                                "Missing host header and tenant ID",
                            ));
                        }
                    }
                }
            };

            // Fehlerbehandlung für Tenant-Ermittlung
            let tenant = match tenant_result {
                Ok(tenant) => tenant,
                Err(e) => {
                    return Ok(create_error_response(
                        StatusCode::NOT_FOUND,
                        format!("Tenant error: {}", e),
                    ));
                }
            };

            // 2. Tenant validieren
            if let Err(e) = tenant.validate_active() {
                return Ok(create_error_response(
                    StatusCode::FORBIDDEN,
                    format!("Tenant error: {}", e),
                ));
            }

            // 3. Tenant-Kontext erstellen
            let request_id = request
                .headers()
                .get("X-Request-ID")
                .map(|h| h.to_str().unwrap_or("unknown"))
                .unwrap_or("unknown")
                .to_string();

            let tenant_context = TenantContext::new(tenant, request_id);

            // 4. Tenant-Kontext in Request Extensions speichern
            request.extensions_mut().insert(tenant_context);

            // Request weiterleiten
            inner.call(request).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::Infallible;
    use tower::ServiceExt;
    use uuid::Uuid;

    #[derive(Clone)]
    struct TestService;

    impl Service<Request<Body>> for TestService {
        type Response = Response;
        type Error = Infallible;
        type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, request: Request<Body>) -> Self::Future {
            let mut response = Response::new(Body::empty());
            response.extensions_mut().clone_from(request.extensions());
            Box::pin(async move { Ok(response) })
        }
    }

    fn create_test_tenant(id: Uuid, domain: &str, is_active: bool) -> Tenant {
        Tenant {
            id,
            name: "Test Tenant".to_string(),
            domain: domain.to_string(),
            is_active,
            settings: Default::default(),
        }
    }

    async fn test_request(
        tenant_service: Arc<dyn TenantService>,
        request: Request<Body>,
    ) -> Response {
        let middleware = TenantLayer::new(tenant_service);
        let mut service = middleware.layer(TestService);
        service.ready().await.unwrap();
        service.call(request).await.unwrap()
    }

    #[tokio::test]
    async fn test_tenant_detection_from_header() {
        let tenant_id = Uuid::new_v4();
        let tenant_service = Arc::new(TestTenantService::new(tenant_id));

        let request = Request::builder()
            .header(TENANT_HEADER, tenant_id.to_string())
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let response = test_request(tenant_service, request).await;
        let tenant_context = response.extensions().get::<TenantContext>().unwrap();
        assert_eq!(tenant_context.tenant.id, tenant_id);
    }

    #[tokio::test]
    async fn test_tenant_detection_from_domain() {
        let tenant_id = Uuid::new_v4();
        let domain = "test.example.com";
        let tenant_service = Arc::new(TestTenantService::new_with_domain(tenant_id, domain));

        let request = Request::builder()
            .header(header::HOST, domain)
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let response = test_request(tenant_service, request).await;
        let tenant_context = response.extensions().get::<TenantContext>().unwrap();
        assert_eq!(tenant_context.tenant.id, tenant_id);
    }

    #[tokio::test]
    async fn test_tenant_not_found() {
        let tenant_service = Arc::new(TestTenantService::new_empty());

        let request = Request::builder()
            .header(header::HOST, "unknown.example.com")
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let response = test_request(tenant_service, request).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_inactive_tenant() {
        let tenant_id = Uuid::new_v4();
        let tenant_service = Arc::new(TestTenantService::new_inactive(tenant_id));

        let request = Request::builder()
            .header(TENANT_HEADER, tenant_id.to_string())
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let response = test_request(tenant_service, request).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    // Test-Implementierung des TenantService
    struct TestTenantService {
        tenant: Option<Tenant>,
        domain: Option<String>,
    }

    impl TestTenantService {
        fn new(id: Uuid) -> Self {
            Self {
                tenant: Some(create_test_tenant(id, "test.example.com", true)),
                domain: None,
            }
        }

        fn new_with_domain(id: Uuid, domain: &str) -> Self {
            Self {
                tenant: Some(create_test_tenant(id, domain, true)),
                domain: Some(domain.to_string()),
            }
        }

        fn new_empty() -> Self {
            Self {
                tenant: None,
                domain: None,
            }
        }

        fn new_inactive(id: Uuid) -> Self {
            Self {
                tenant: Some(create_test_tenant(id, "test.example.com", false)),
                domain: None,
            }
        }
    }

    #[async_trait::async_trait]
    impl TenantService for TestTenantService {
        async fn find_by_id(&self, id: &str) -> Result<Tenant, AppError> {
            match &self.tenant {
                Some(tenant) if tenant.id.to_string() == id => Ok(tenant.clone()),
                _ => Err(AppError::Tenant("Tenant not found".into())),
            }
        }

        async fn find_by_domain(&self, domain: &str) -> Result<Tenant, AppError> {
            match (&self.tenant, &self.domain) {
                (Some(tenant), Some(d)) if d == domain => Ok(tenant.clone()),
                _ => Err(AppError::Tenant("Tenant not found".into())),
            }
        }
    }
}
