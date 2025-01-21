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
    event_store: ComponentHealth,
    message_broker: ComponentHealth,
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
                && details.event_store.status == HealthStatus::Healthy
                && details.message_broker.status == HealthStatus::Healthy
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
    let mut sys = SysInfo::new();
    sys.refresh_all();

    let health_details = check_system_health(&state, &sys).await;
    let (status, status_code, message) = match &health_details {
        Ok(details) => {
            let has_unhealthy = details.tenant_service.status == HealthStatus::Unhealthy
                || details.cache.status == HealthStatus::Unhealthy
                || details.event_store.status == HealthStatus::Unhealthy
                || details.message_broker.status == HealthStatus::Unhealthy;

            let has_degraded = details.tenant_service.status == HealthStatus::Degraded
                || details.cache.status == HealthStatus::Degraded
                || details.event_store.status == HealthStatus::Degraded
                || details.message_broker.status == HealthStatus::Degraded;

            let system_overloaded = details.system.cpu_usage >= 90.0
                || details.system.memory_usage >= 90.0
                || details.system.disk_usage >= 90.0;

            if has_unhealthy {
                (
                    "not_ready".to_string(),
                    StatusCode::SERVICE_UNAVAILABLE,
                    state
                        .i18n
                        .format_message(SupportedLanguage::En, "system-not-ready-message", None)
                        .await
                        .unwrap_or_else(|_| {
                            "System is not ready - critical services unavailable".to_string()
                        }),
                )
            } else if has_degraded || system_overloaded {
                (
                    "partially_ready".to_string(),
                    StatusCode::OK,
                    state
                        .i18n
                        .format_message(SupportedLanguage::En, "system-degraded-message", None)
                        .await
                        .unwrap_or_else(|_| {
                            "System is partially ready - some services degraded".to_string()
                        }),
                )
            } else {
                (
                    "ready".to_string(),
                    StatusCode::OK,
                    state
                        .i18n
                        .format_message(SupportedLanguage::En, "system-ready-message", None)
                        .await
                        .unwrap_or_else(|_| "System is ready".to_string()),
                )
            }
        },
        Err(_) => (
            "not_ready".to_string(),
            StatusCode::SERVICE_UNAVAILABLE,
            state
                .i18n
                .format_message(SupportedLanguage::En, "system-error-message", None)
                .await
                .unwrap_or_else(|_| "System check failed".to_string()),
        ),
    };

    let body = Json(HealthResponse {
        status,
        message,
        timestamp: Utc::now().to_rfc3339(),
        details: health_details.ok(),
    });

    (status_code, body).into_response()
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

    // Check Redis health
    let cache_start = std::time::Instant::now();
    let cache_health = match &state.redis {
        Some(redis) => match redis.ping().await {
            Ok(_) => ComponentHealth {
                status: HealthStatus::Healthy,
                latency_ms: cache_start.elapsed().as_millis() as u64,
                message: None,
            },
            Err(e) => ComponentHealth {
                status: HealthStatus::Unhealthy,
                latency_ms: cache_start.elapsed().as_millis() as u64,
                message: Some(e.to_string()),
            },
        },
        None => ComponentHealth {
            status: HealthStatus::Unhealthy,
            latency_ms: 0,
            message: Some("Redis not configured".to_string()),
        },
    };

    // Check EventStore health
    let es_start = std::time::Instant::now();
    let event_store_health = match &state.event_store {
        Some(es) => match es.check_connection().await {
            Ok(_) => ComponentHealth {
                status: HealthStatus::Healthy,
                latency_ms: es_start.elapsed().as_millis() as u64,
                message: None,
            },
            Err(e) => ComponentHealth {
                status: HealthStatus::Unhealthy,
                latency_ms: es_start.elapsed().as_millis() as u64,
                message: Some(e.to_string()),
            },
        },
        None => ComponentHealth {
            status: HealthStatus::Unhealthy,
            latency_ms: 0,
            message: Some("EventStore not configured".to_string()),
        },
    };

    // Check RabbitMQ health
    let mq_start = std::time::Instant::now();
    let message_broker_health = match &state.message_broker {
        Some(mb) => match mb.check_connection().await {
            Ok(_) => ComponentHealth {
                status: HealthStatus::Healthy,
                latency_ms: mq_start.elapsed().as_millis() as u64,
                message: None,
            },
            Err(e) => ComponentHealth {
                status: HealthStatus::Unhealthy,
                latency_ms: mq_start.elapsed().as_millis() as u64,
                message: Some(e.to_string()),
            },
        },
        None => ComponentHealth {
            status: HealthStatus::Unhealthy,
            latency_ms: 0,
            message: Some("MessageBroker not configured".to_string()),
        },
    };

    // Calculate system metrics
    let total_memory = sys.total_memory() as f64;
    let used_memory = sys.used_memory() as f64;
    let memory_usage = if total_memory > 0.0 {
        (used_memory / total_memory) * 100.0
    } else {
        0.0
    };

    let system_health = SystemHealth {
        cpu_usage: sys.global_cpu_usage() as f64,
        memory_usage,
        disk_usage: calculate_disk_usage(),
    };

    Ok(HealthDetails {
        tenant_service: tenant_health,
        cache: cache_health,
        event_store: event_store_health,
        message_broker: message_broker_health,
        external_services: Vec::new(),
        system: system_health,
    })
}

fn calculate_disk_usage() -> f64 {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    if let Some(disk) = disks.iter().next() {
        let total = disk.total_space() as f64;
        let free = disk.available_space() as f64;
        if total > 0.0 {
            return ((total - free) / total) * 100.0;
        }
    }
    0.0
}
