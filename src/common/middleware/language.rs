use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use axum::extract::Query;
use axum::http::Request;
use serde::Deserialize;
use tower::{Layer, Service};

use crate::common::{
    config,
    i18n::{I18nManager, SupportedLanguage},
};

const ACCEPT_LANGUAGE_HEADER: &str = "accept-language";

#[derive(Debug, Clone)]
pub struct LanguageLayer {
    i18n_manager: Arc<I18nManager>,
}

impl LanguageLayer {
    pub fn new(i18n_manager: Arc<I18nManager>) -> Self {
        Self { i18n_manager }
    }
}

impl<S> Layer<S> for LanguageLayer {
    type Service = LanguageMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        LanguageMiddleware {
            inner: service,
            i18n_manager: self.i18n_manager.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LanguageMiddleware<S> {
    inner: S,
    i18n_manager: Arc<I18nManager>,
}

#[derive(Debug, Deserialize)]
struct LanguageQuery {
    lang: Option<String>,
}

impl<S, B> Service<Request<B>> for LanguageMiddleware<S>
where
    S: Service<Request<B>> + Send + Clone + 'static,
    S::Response: Default,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<B>) -> Self::Future {
        let i18n_manager = self.i18n_manager.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // 1. Check URL query parameter
            let query = Query::<LanguageQuery>::try_from_uri(request.uri())
                .ok()
                .and_then(|q| q.lang.clone());

            // 2. Check Accept-Language header
            let accept_language = request
                .headers()
                .get(ACCEPT_LANGUAGE_HEADER)
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.split(',').next())
                .unwrap_or(config::get_default_language());

            // Determine the language to use
            let language = query
                .or_else(|| Some(accept_language.to_string()))
                .unwrap_or_else(|| config::get_default_language().to_string());

            // Validate the language
            let valid_language = if SupportedLanguage::iter().any(|l| l.as_str() == language) {
                language
            } else {
                config::get_default_language().to_string()
            };

            // Add language to request extensions
            request.extensions_mut().insert(valid_language.clone());

            // Add i18n manager to request extensions
            request.extensions_mut().insert(i18n_manager);

            inner.call(request).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::i18n::TestResourceProvider;
    use axum::http::header;
    use axum::response::Response;
    use bytes::Bytes;
    use http_body_util::Full;
    use std::convert::Infallible;
    use tower::ServiceExt;

    #[derive(Clone)]
    struct TestService;

    impl<B> Service<Request<B>> for TestService {
        type Response = Response<Full<Bytes>>;
        type Error = Infallible;
        type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, request: Request<B>) -> Self::Future {
            let mut response = Response::new(Full::new(Bytes::new()));
            response.extensions_mut().clone_from(request.extensions());
            Box::pin(async move { Ok(response) })
        }
    }

    async fn setup_i18n() -> Arc<I18nManager> {
        Arc::new(
            I18nManager::new_with_provider(TestResourceProvider::new())
                .await
                .expect("Failed to initialize i18n"),
        )
    }

    #[tokio::test]
    async fn test_language_detection_from_query() {
        let i18n_manager = setup_i18n().await;
        let middleware = LanguageLayer::new(i18n_manager);
        let service = middleware.layer(TestService);

        let request = Request::builder()
            .uri("/?lang=de")
            .body(Full::new(Bytes::new()))
            .unwrap();

        let response = service.oneshot(request).await.unwrap();
        assert_eq!(response.extensions().get::<String>().unwrap(), "de");
    }

    #[tokio::test]
    async fn test_language_detection_from_header() {
        let i18n_manager = setup_i18n().await;
        let middleware = LanguageLayer::new(i18n_manager);
        let service = middleware.layer(TestService);

        let request = Request::builder()
            .header(header::ACCEPT_LANGUAGE, "fr,en;q=0.9,de;q=0.8")
            .uri("/")
            .body(Full::new(Bytes::new()))
            .unwrap();

        let response = service.oneshot(request).await.unwrap();
        assert_eq!(response.extensions().get::<String>().unwrap(), "fr");
    }

    #[tokio::test]
    async fn test_fallback_to_default_language() {
        let i18n_manager = setup_i18n().await;
        let middleware = LanguageLayer::new(i18n_manager);
        let service = middleware.layer(TestService);

        let request = Request::builder()
            .uri("/")
            .body(Full::new(Bytes::new()))
            .unwrap();

        let response = service.oneshot(request).await.unwrap();
        assert_eq!(response.extensions().get::<String>().unwrap(), "en");
    }
}
