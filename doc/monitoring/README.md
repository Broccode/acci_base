# Monitoring & Observability

## Prometheus Metrics

### System Metrics (via sysinfo)

```prometheus
# Memory Usage
system_memory_total_bytes{service="acci-api"} 17179869184
system_memory_used_bytes{service="acci-api"} 8589934592
system_memory_available_bytes{service="acci-api"} 8589934592

# CPU Usage (per core)
system_cpu_usage_percent{service="acci-api",cpu="0"} 45.2
system_cpu_usage_percent{service="acci-api",cpu="1"} 32.1

# Process Metrics
process_memory_rss_bytes{service="acci-api"} 1024576
process_memory_heap_bytes{service="acci-api"} 512288
process_open_fds{service="acci-api"} 124
process_gc_duration_seconds{service="acci-api"} 0.042
```

### Database Connection Pool Metrics

```prometheus
# Pool Size
db_pool_connections_total{service="acci-api",state="idle"} 5
db_pool_connections_total{service="acci-api",state="active"} 3
db_pool_connections_total{service="acci-api",state="max"} 100

# Connection Timing
db_connection_acquire_duration_seconds{service="acci-api",quantile="0.95"} 0.01
db_connection_acquire_timeout_total{service="acci-api"} 2

# Connection Health
db_connection_errors_total{service="acci-api",type="timeout"} 1
db_connection_errors_total{service="acci-api",type="auth"} 0
db_health_check_success{service="acci-api"} 1
```

### Cache Performance Metrics

```prometheus
# Cache Operations
cache_operations_total{service="acci-api",operation="get",status="hit"} 15234
cache_operations_total{service="acci-api",operation="get",status="miss"} 234
cache_operations_total{service="acci-api",operation="set"} 5678

# Cache Memory
cache_memory_used_bytes{service="acci-api"} 1048576
cache_items_total{service="acci-api"} 1234

# Cache Performance
cache_operation_duration_seconds{service="acci-api",operation="get",quantile="0.95"} 0.002
cache_hit_ratio{service="acci-api"} 0.985
```

### HTTP Metrics

```prometheus
# Request Rate
http_requests_total{service="acci-api",method="POST",path="/auth/login"} 1234
http_requests_total{service="acci-api",method="GET",path="/users"} 5678

# Response Time
http_request_duration_seconds{service="acci-api",method="GET",path="/users",quantile="0.95"} 0.123
http_request_duration_seconds{service="acci-api",method="POST",path="/auth/login",quantile="0.95"} 0.234

# Error Rate
http_errors_total{service="acci-api",method="POST",path="/users",status="400"} 23
http_errors_total{service="acci-api",method="GET",path="/users",status="500"} 1

# Active Requests
http_active_requests{service="acci-api"} 45
```

### Rate Limiting Metrics

```prometheus
# Rate Limit Hits
rate_limit_hits_total{service="acci-api",endpoint="/auth/login"} 123
rate_limit_hits_total{service="acci-api",endpoint="/api/v1/users"} 45

# Current Rate Status
rate_limit_remaining{service="acci-api",endpoint="/auth/login"} 877
rate_limit_remaining{service="acci-api",endpoint="/api/v1/users"} 955

# Rate Limit Configurations
rate_limit_max{service="acci-api",endpoint="/auth/login"} 1000
rate_limit_window_seconds{service="acci-api",endpoint="/auth/login"} 3600
```

### Business Metrics

```prometheus
# Active Users
acci_active_users_total{service="acci-api",tenant="tenant-123"} 1500

# Failed Logins
acci_failed_logins_total{service="acci-api",tenant="tenant-123"} 12

# API Token Usage
acci_token_usage_total{service="acci-api",tenant="tenant-123",token_type="access"} 45678

# Tenant Activity
acci_tenant_requests_total{service="acci-api",tenant="tenant-123"} 12345
acci_tenant_active_sessions{service="acci-api",tenant="tenant-123"} 150
```

## Grafana Dashboards

### System Overview Dashboard
- System resource utilization
- Process metrics
- Database connection pool status
- Cache performance

### API Performance Dashboard
- Request rates and latencies
- Error rates
- Rate limiting status
- Endpoint usage patterns

### Business Metrics Dashboard
- Active users per tenant
- Authentication metrics
- Token usage
- Tenant activity

## Alerting Rules

### Critical Alerts
```yaml
- alert: HighErrorRate
  expr: rate(http_errors_total{status=~"5.."}[5m]) > 0.1
  for: 5m
  labels:
    severity: critical
  annotations:
    description: "High error rate detected"

- alert: DatabaseConnectionPoolExhausted
  expr: db_pool_connections_total{state="active"} >= db_pool_connections_total{state="max"} * 0.9
  for: 5m
  labels:
    severity: critical
  annotations:
    description: "Database connection pool near exhaustion"

- alert: CachePerformanceDegraded
  expr: cache_hit_ratio < 0.8
  for: 15m
  labels:
    severity: warning
  annotations:
    description: "Cache hit ratio below threshold"
```

## Health Checks

### Endpoint: `/health`
```json
{
  "status": "healthy",
  "timestamp": "2024-01-21T10:00:00Z",
  "components": {
    "database": {
      "status": "healthy",
      "latency_ms": 5,
      "pool_status": {
        "active": 3,
        "idle": 5,
        "max": 100
      }
    },
    "cache": {
      "status": "healthy",
      "latency_ms": 2,
      "hit_ratio": 0.985
    },
    "system": {
      "status": "healthy",
      "memory": {
        "total": 17179869184,
        "used": 8589934592,
        "available": 8589934592
      },
      "cpu_usage": 38.6
    }
  }
}
```

### Endpoint: `/metrics`
- Prometheus format metrics endpoint
- Includes all system, database, cache, and business metrics
- Rate limited to prevent abuse
- Authentication required in production

## Tracing Configuration

### OpenTelemetry Setup

```rust
use opentelemetry::trace::Tracer;
use tracing_subscriber::{layer::SubscriberExt, Registry};
use tracing_opentelemetry::OpenTelemetryLayer;

pub fn init_tracer() -> Result<(), Box<dyn Error>> {
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("acci-api")
        .with_collector_endpoint("http://jaeger:14268/api/traces")
        .install_batch(opentelemetry::runtime::Tokio)?;

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = Registry::default().with(telemetry);
    
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
```

### Span Attributes

```rust
use tracing::{info, info_span};

async fn handle_request(req: Request) -> Result<Response, Error> {
    let span = info_span!(
        "handle_request",
        tenant_id = %req.tenant_id,
        user_id = %req.user_id,
        request_id = %req.id
    );
    let _guard = span.enter();

    info!("Processing request");
    // Request handling logic
}
```

### Baggage Propagation

```rust
use opentelemetry::propagation::TextMapPropagator;

fn extract_context(headers: &HeaderMap) -> Context {
    let propagator = TraceContextPropagator::new();
    propagator.extract(&HeaderExtractor(headers))
}

fn inject_context(context: Context, headers: &mut HeaderMap) {
    let propagator = TraceContextPropagator::new();
    propagator.inject_context(&context, &mut HeaderInjector(headers));
}
```

## Logging Standards

### Log Format

```json
{
  "timestamp": "2024-01-19T20:00:00.000Z",
  "level": "INFO",
  "target": "acci_api::auth",
  "span": {
    "name": "handle_login",
    "tenant_id": "tenant-123",
    "request_id": "req-456"
  },
  "fields": {
    "event": "user_login",
    "user_id": "user-789",
    "ip_address": "192.168.1.1",
    "success": true
  }
}
```

### Log Levels

```rust
// Error: Unexpected failures that require immediate attention
error!("Database connection failed: {}", err);

// Warn: Potentially harmful situations that should be investigated
warn!("Rate limit exceeded for tenant {}", tenant_id);

// Info: Normal operational events
info!("User {} logged in successfully", user_id);

// Debug: Detailed information for debugging
debug!("Processing request with params: {:?}", params);

// Trace: Very detailed debugging information
trace!("Entering function with context: {:?}", ctx);
```

### Structured Logging

```rust
use tracing::info;

info!(
    target: "acci_api::auth",
    event = "user_login",
    user_id = %user.id,
    tenant_id = %tenant.id,
    ip_address = %request.ip(),
    success = true,
    "User logged in successfully"
);
```

## Integration

### Prometheus Configuration

```yaml
scrape_configs:
  - job_name: 'acci-api'
    scrape_interval: 15s
    static_configs:
      - targets: ['acci-api:9090']
    metrics_path: '/metrics'
    scheme: 'http'
```

### Jaeger Configuration

```yaml
JAEGER_AGENT_HOST: jaeger
JAEGER_AGENT_PORT: 6831
JAEGER_SAMPLER_TYPE: const
JAEGER_SAMPLER_PARAM: 1
```

### Grafana Configuration

```yaml
datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    
  - name: Jaeger
    type: jaeger
    access: proxy
    url: http://jaeger:16686
``` 