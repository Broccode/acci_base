# Monitoring & Observability

## Prometheus Metrics

### System Metrics

```prometheus
# CPU Usage
process_cpu_usage_percent{service="acci-api"} 45.2

# Memory Usage
process_memory_rss_bytes{service="acci-api"} 1024576
process_memory_heap_bytes{service="acci-api"} 512288

# File Descriptors
process_open_fds{service="acci-api"} 124

# Garbage Collection
process_gc_duration_seconds{service="acci-api"} 0.042
```

### HTTP Metrics

```prometheus
# Request Rate
http_requests_total{service="acci-api",method="POST",path="/auth/login"} 1234

# Response Time
http_request_duration_seconds{service="acci-api",method="GET",path="/users",quantile="0.95"} 0.123

# Error Rate
http_errors_total{service="acci-api",method="POST",path="/users",status="400"} 23

# Active Requests
http_active_requests{service="acci-api"} 45
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
acci_tenant_requests_total{service="acci-api",tenant="tenant-123"} 89012
```

### Resource Pool Metrics

```prometheus
# Database Connections
db_connections_active{service="acci-api",pool="main"} 10
db_connections_idle{service="acci-api",pool="main"} 5
db_connections_max{service="acci-api",pool="main"} 20

# Cache Stats
cache_hits_total{service="acci-api",cache="user"} 5678
cache_misses_total{service="acci-api",cache="user"} 123
cache_size_bytes{service="acci-api",cache="user"} 1048576
```

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

## Alerting Rules

### Prometheus Alert Rules

```yaml
groups:
  - name: acci-api
    rules:
      # High Error Rate
      - alert: HighErrorRate
        expr: |
          sum(rate(http_errors_total{service="acci-api"}[5m]))
          /
          sum(rate(http_requests_total{service="acci-api"}[5m]))
          > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: High error rate detected
          description: Error rate is above 5% for 5 minutes

      # High Response Time
      - alert: SlowResponses
        expr: |
          histogram_quantile(0.95,
            rate(http_request_duration_seconds_bucket{service="acci-api"}[5m]))
          > 1.0
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: Slow response times detected
          description: 95th percentile response time is above 1 second

      # Resource Exhaustion
      - alert: HighMemoryUsage
        expr: |
          process_memory_rss_bytes{service="acci-api"}
          /
          process_memory_max_bytes{service="acci-api"}
          > 0.85
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: High memory usage detected
          description: Memory usage is above 85% for 5 minutes

      # Database Connection Pool
      - alert: DatabaseConnectionsSaturated
        expr: |
          db_connections_active{service="acci-api"}
          /
          db_connections_max{service="acci-api"}
          > 0.8
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: Database connection pool near capacity
          description: More than 80% of database connections are in use

      # Failed Authentication
      - alert: HighFailedLogins
        expr: |
          rate(acci_failed_logins_total{service="acci-api"}[5m])
          > 10
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: High rate of failed logins
          description: More than 10 failed logins per minute detected
```

### Alert Thresholds

| Metric | Warning | Critical | Duration |
|--------|---------|----------|----------|
| Error Rate | 2% | 5% | 5m |
| Response Time (p95) | 500ms | 1s | 5m |
| CPU Usage | 70% | 85% | 5m |
| Memory Usage | 80% | 90% | 5m |
| Disk Usage | 75% | 90% | 15m |
| Failed Logins | 10/min | 30/min | 5m |
| DB Connections | 80% | 90% | 5m |
| Cache Miss Rate | 40% | 60% | 15m |

## Dashboards

### System Overview
![System Overview](https://api.acci-framework.com/docs/images/dashboard-system.png)

- CPU & Memory Usage
- Network I/O
- Disk Usage & I/O
- Process Stats

### API Performance
![API Performance](https://api.acci-framework.com/docs/images/dashboard-api.png)

- Request Rate
- Error Rate
- Response Time
- Active Requests

### Business Metrics
![Business Metrics](https://api.acci-framework.com/docs/images/dashboard-business.png)

- Active Users
- API Token Usage
- Tenant Activity
- Failed Logins

### Resource Pools
![Resource Pools](https://api.acci-framework.com/docs/images/dashboard-resources.png)

- Database Connections
- Cache Stats
- Connection Pool Usage
- Queue Depths

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