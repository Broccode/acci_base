use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use serde::Serialize;
use sysinfo::System as SysInfo;

use crate::common::{error::AppResult, i18n::SupportedLanguage};
use crate::infrastructure::state::AppState;

pub fn health_routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/health", axum::routing::get(health_check))
        .route("/ready", axum::routing::get(readiness_check))
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    status: String,
    message: String,
    timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<HealthDetails>,
}

#[derive(Debug, Serialize)]
pub struct HealthDetails {
    tenant_service: ComponentHealth,
    cache: ComponentHealth,
    external_services: Vec<ServiceHealth>,
    system: SystemHealth,
}

#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    status: HealthStatus,
    latency_ms: u64,
    message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ServiceHealth {
    name: String,
    status: HealthStatus,
    latency_ms: u64,
    message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SystemHealth {
    cpu_usage: f64,
    memory_usage: f64,
    disk_usage: f64,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let mut sys = SysInfo::new();
    sys.refresh_all();

    let health_details = check_system_health(&state, &sys).await;
    let (status, status_code) = match &health_details {
        Ok(details) => {
            let is_healthy = details.tenant_service.status == HealthStatus::Healthy
                && details.cache.status == HealthStatus::Healthy
                && details
                    .external_services
                    .iter()
                    .all(|s| s.status == HealthStatus::Healthy)
                && details.system.cpu_usage < 90.0
                && details.system.memory_usage < 90.0
                && details.system.disk_usage < 90.0;

            if is_healthy {
                ("healthy".to_string(), StatusCode::OK)
            } else {
                ("degraded".to_string(), StatusCode::OK)
            }
        },
        Err(_) => ("unhealthy".to_string(), StatusCode::SERVICE_UNAVAILABLE),
    };

    let body = Json(HealthResponse {
        status,
        message: "Health check completed".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        details: health_details.ok(),
    });

    (status_code, body).into_response()
}

async fn readiness_check(State(state): State<AppState>) -> impl IntoResponse {
    let i18n_msg = state
        .i18n
        .format_message(SupportedLanguage::En, "system-ready-message", None)
        .await
        .unwrap_or_else(|_| "System is ready".to_string());

    let body = Json(HealthResponse {
        status: "ready".to_string(),
        message: i18n_msg,
        timestamp: Utc::now().to_rfc3339(),
        details: None,
    });

    (StatusCode::OK, body).into_response()
}

async fn check_system_health(state: &AppState, sys: &SysInfo) -> AppResult<HealthDetails> {
    // Check tenant service (which includes database health)
    let tenant_start = std::time::Instant::now();
    let tenant_health = match state.tenant_service.list().await {
        Ok(_) => ComponentHealth {
            status: HealthStatus::Healthy,
            latency_ms: tenant_start.elapsed().as_millis() as u64,
            message: None,
        },
        Err(e) => ComponentHealth {
            status: HealthStatus::Unhealthy,
            latency_ms: tenant_start.elapsed().as_millis() as u64,
            message: Some(e.to_string()),
        },
    };

    // TODO: Implement cache health check once cache is integrated
    let cache_health = ComponentHealth {
        status: HealthStatus::Healthy,
        latency_ms: 0,
        message: None,
    };

    // TODO: Add external service checks as they are integrated
    let external_services = Vec::new();

    // Calculate system metrics
    let total_memory = sys.total_memory() as f64;
    let used_memory = sys.used_memory() as f64;
    let memory_usage = if total_memory > 0.0 {
        (used_memory / total_memory) * 100.0
    } else {
        0.0
    };

    let system_health = SystemHealth {
        cpu_usage: 0.0, // CPU usage is not available without the cpu feature
        memory_usage,
        disk_usage: 0.0, // Disk usage is not available without the disk feature
    };

    Ok(HealthDetails {
        tenant_service: tenant_health,
        cache: cache_health,
        external_services,
        system: system_health,
    })
}
