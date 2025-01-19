use std::time::Duration;

use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};

use crate::common::error::AppError;

/// Initialize the metrics system with Prometheus exporter
pub fn init_metrics() -> Result<PrometheusHandle, AppError> {
    const EXPONENTIAL_SECONDS: &[f64] = &[
        0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
    ];

    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_request_duration_seconds".to_string()),
            EXPONENTIAL_SECONDS,
        )
        .map_err(|e| AppError::configuration(format!("Failed to set metric buckets: {}", e)))?
        .install_recorder()
        .map_err(|e| AppError::configuration(format!("Failed to install metrics recorder: {}", e)))
}

/// Record an HTTP request
#[allow(dead_code)]
pub fn record_request(path: &str, method: &str, status: u16, duration: Duration) {
    let duration_secs = duration.as_secs_f64();

    // Record request count
    counter!(
        "http_requests_total",
        "path" => path.to_string(),
        "method" => method.to_string(),
        "status" => status.to_string()
    )
    .increment(1);

    // Record request duration
    histogram!(
        "http_request_duration_seconds",
        "path" => path.to_string(),
        "method" => method.to_string()
    )
    .record(duration_secs);
}

/// Record system metrics
#[allow(dead_code)]
pub fn record_system_metrics(cpu_usage: f64, memory_usage: f64, disk_usage: f64) {
    gauge!("system_cpu_usage_percent").set(cpu_usage);
    gauge!("system_memory_usage_percent").set(memory_usage);
    gauge!("system_disk_usage_percent").set(disk_usage);
}

/// Record database metrics
#[allow(dead_code)]
pub fn record_db_metrics(pool_size: u32, active_connections: u32, idle_connections: u32) {
    gauge!("db_pool_size").set(pool_size as f64);
    gauge!("db_active_connections").set(active_connections as f64);
    gauge!("db_idle_connections").set(idle_connections as f64);
}

/// Record tenant metrics
#[allow(dead_code)]
pub fn record_tenant_metrics(tenant_id: &str, active_users: u32, storage_used_bytes: u64) {
    gauge!(
        "tenant_active_users",
        "tenant_id" => tenant_id.to_string()
    )
    .set(active_users as f64);

    gauge!(
        "tenant_storage_used_bytes",
        "tenant_id" => tenant_id.to_string()
    )
    .set(storage_used_bytes as f64);
}

/// Record cache metrics
#[allow(dead_code)]
pub fn record_cache_metrics(hits: u64, misses: u64, size_bytes: u64) {
    counter!("cache_hits_total").increment(hits);
    counter!("cache_misses_total").increment(misses);
    gauge!("cache_size_bytes").set(size_bytes as f64);
}

/// Record rate limiting metrics
#[allow(dead_code)]
pub fn record_rate_limit_metrics(tenant_id: &str, allowed: u64, blocked: u64) {
    counter!(
        "rate_limit_allowed_total",
        "tenant_id" => tenant_id.to_string()
    )
    .increment(allowed);

    counter!(
        "rate_limit_blocked_total",
        "tenant_id" => tenant_id.to_string()
    )
    .increment(blocked);
}
