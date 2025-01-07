# ACCI Framework Development Guidelines

## Project Goals & Requirements

### Core Purpose

- Base framework for various full-stack applications
- Multi-tenancy support for enterprise deployments
- Pre-built user management and authentication
- Support for large-scale provider deployments
- Modular architecture for easy product derivation

### Documentation & Language Requirements

#### User Interface & Documentation Languages

- English: Primary development language
  - All code and code comments
  - All rustdoc documentation
  - Documentation in root folders
  - API documentation
  - Technical specifications
  - Development guidelines
  
- German: Required for
  - API documentation
  - Technical specifications
  - Development guidelines
  - User interface text (via i18n)
  - User documentation (in de/ folders)
  - Marketing materials
  - End-user guides
  
- Albanian: Required for
  - API documentation
  - Technical specifications
  - Development guidelines
  - User interface text (via i18n)
  - User documentation (in sq/ folders)
  - Marketing materials
  - End-user guides

- French: Required for
  - User interface text (via i18n)
  - Marketing materials
  - End-user guides

- Spanish: Required for
  - User interface text (via i18n)
  - Marketing materials
  - End-user guides

#### Translation Requirements

- All user-facing strings must use the i18n system
- No hardcoded non-English strings in code
- Translation files must be organized by language and component
- Translation coverage must be maintained at 100% for supported languages
- Regular translation reviews and updates required

#### Documentation Structure

```
doc/
├── architecture/    # Technical documentation (English only)
├── api/            # API documentation (English only)
├── development/    # Development guides (English only)
└── user/           # User documentation
    ├── en/         # English user docs
    ├── de/         # German user docs
    └── sq/         # Albanian user docs
```

### Technical Requirements

- Containerized web application (Docker/Kubernetes)
- External service integration:
  - LDAP/Active Directory (multi-server capable)
  - SMTP services
  - Legacy Node.js application (gRPC)
  - InfluxDB for performance metrics
- Critical performance requirements for high user loads
- Strict security requirements (banking/insurance sector)
- Audit-ready architecture
- Dual API exposure:

  ```rust
  // Example of unified API definition
  #[derive(ApiEndpoint, GraphQLObject)]
  struct UserEndpoint {
      #[endpoint(
          rest = "GET /users/{id}",
          graphql = "user(id: ID!): User"
      )]
      async fn get_user(&self, id: UserId) -> Result<User, ApiError> {
          // Single implementation for both REST and GraphQL
          self.user_service.get_user(id).await
      }
  }
  
  // Macro generates both REST and GraphQL handlers
  generate_handlers! {
      rest: {
          fn handle_rest(req: HttpRequest) -> Result<Json<User>, RestError> {
              // Auto-generated REST handler
          }
      },
      graphql: {
          fn handle_graphql(ctx: &Context) -> Result<User, GraphQLError> {
              // Auto-generated GraphQL resolver
          }
      }
  }
  ```

### API Design Requirements

- Single source of truth for endpoint definitions
- Automatic generation of:
  - REST endpoints (axum routes)
  - GraphQL schema and resolvers (async-graphql)
  - API documentation (OpenAPI/Swagger)
  - GraphQL schema documentation
  - Type-safe client libraries
- Consistent error handling across both APIs
- Unified validation logic
- Performance monitoring for both API types
- API versioning strategy:

  ```rust
  #[api_version("v1")]
  mod api_v1 {
      #[endpoint(
          rest = "GET /v1/users/{id}",
          graphql = "userV1(id: ID!): UserV1"
      )]
      async fn get_user_v1() { /* ... */ }
  }
  
  #[api_version("v2")]
  mod api_v2 {
      #[endpoint(
          rest = "GET /v2/users/{id}",
          graphql = "user(id: ID!): User"  // v2 ist default in GraphQL
      )]
      async fn get_user() { /* ... */ }
  }
  ```

- API Documentation:
  - Interactive API playground for GraphQL
  - Swagger UI for REST endpoints
  - Versioned documentation matching API versions
  - Automatic changelog generation
- API Deprecation Strategy:

  ```rust
  #[endpoint(
      rest = "GET /v1/users/{id}",
      graphql = "userV1(id: ID!): UserV1",
      deprecated = "Use v2 endpoint instead. Will be removed by 2024-12-31."
  )]
  async fn get_user_v1() { /* ... */ }
  ```

- Rate Limiting:

  ```rust
  #[rate_limit(
      requests = 100,
      period = "1m",
      scope = "tenant"
  )]
  async fn list_users() -> Result<Vec<User>, ApiError> {
      // Implementation
  }
  ```

- Tenant Quotas:
  - API call limits
  - Storage quotas
  - Concurrent user limits
  - Resource usage tracking

### API Testing Requirements

- Unified test cases for both API types:

  ```rust
  #[api_test]
  mod tests {
      #[test_case(ApiType::Rest)]
      #[test_case(ApiType::GraphQL)]
      async fn test_get_user(api_type: ApiType) {
          let client = TestClient::new(api_type);
          
          // Test wird automatisch für beide API-Typen ausgeführt
          let response = client
              .get_user(UserId::new(1))
              .await?;
              
          assert_eq!(response.name, "Test User");
      }
  }
  ```

- Automatic schema validation
- Performance comparison tests
- Load testing for both API types
- Security testing for both exposure methods

### Data Management

- PostgreSQL as primary database
- InfluxDB for performance metrics
- Docker volume-based data persistence
- Snapshot/rollback capability
- External backup system support
- Backup Strategy:
  - Point-in-time recovery
  - Tenant-specific backups
  - Automated backup verification
  - Cross-region backup replication
- Recovery Procedures:

  ```rust
  #[recovery_procedure]
  async fn restore_tenant_data(
      tenant_id: TenantId,
      point_in_time: DateTime<Utc>
  ) -> Result<RecoveryStatus, RecoveryError> {
      // Implementation
  }
  ```

### Deployment & Operations

- Docker Compose as primary deployment method
- Kubernetes compatibility
- Minimal host system requirements
- Self-contained architecture
- Simple operational commands for:
  - Start/stop
  - Snapshot creation
  - Rollback execution

### Documentation & Internationalization

## Language Support

- English: Primary development language, used for all code and comments, documentation in root folders
- German: Required for all documentation (in de/ folders) and user interfaces
- Albanian: Required for all documentation (in sq/ folders) and user interfaces

## Documentation Structure

```
doc/
├── architecture/
│   ├── de/
│   └── sq/
└── contributing/
    ├── de/
    └── sq/
```

## Documentation Requirements

- Every markdown file must have corresponding versions in all three languages
- Code comments must be in English only
- User-facing strings must use fluent (FTL) for i18n
- Documentation must be updated with every code change
- Version-specific documentation required for API changes

## Translation Requirements

- Translation verification required in CI pipeline
- No hardcoded strings allowed in code
- All UI text must use i18n system
- Documentation must maintain consistent terminology across languages
- Translation coverage checks in CI pipeline
- Translation key usage analysis tools

### I18n Requirements

- Every user-facing string MUST be internationalized
- No hardcoded strings in code or templates
- Use `fluent` for all translations:

  ```rust
  use fluent::{FluentBundle, FluentResource};
  
  pub struct I18nContext {
      bundle: FluentBundle<FluentResource>,
      fallback_bundle: FluentBundle<FluentResource>,
  }
  
  impl I18nContext {
      pub fn get_message(&self, key: &str, args: Option<&FluentArgs>) -> String {
          self.bundle
              .get_message(key)
              .or_else(|| self.fallback_bundle.get_message(key))
              .expect(&format!("Missing translation key: {}", key))
      }
  }
  ```

### Fluent (FTL) Syntax Requirements

- Message ID Format:
  - Use hyphens (-) instead of dots (.) for namespacing
  - Example: `app-title` instead of `app.title`
  - Pattern: `{component-name}-{descriptor}`
- Plural Forms:
  - MUST include a default variant marked with *[other]
  - Example:

    ```ftl
    messages-count = { $count ->
        [one] 1 message
        *[other] { $count } messages
    }
    ```

- Message Structure:
  - One message per line
  - No trailing whitespace
  - UTF-8 encoding required
  - Line endings must be LF (\n)
- Required Message Properties:
  - All messages must have a unique ID
  - All plural forms must include at least [one] and *[other]
  - All messages must be present in all supported languages
- File Organization:
  - Group related messages in component-specific files
  - Common messages in common.ftl
  - Error messages in errors.ftl
- Example of correct FTL syntax:

  ```ftl
  # Component title
  app-title = Application Title

  # User count with plural support
  user-count = { $count ->
      [one] 1 user online
      *[other] { $count } users online
  }

  # Button labels
  button-save = Save
  button-cancel = Cancel
  ```

- Translation files structure:

  ```plaintext
  i18n/
  ├── de/
  │   ├── common.ftl
  │   ├── errors.ftl
  │   └── components/
  │       ├── auth.ftl
  │       └── dashboard.ftl
  └── en/
      ├── common.ftl
      ├── errors.ftl
      └── components/
          ├── auth.ftl
          └── dashboard.ftl
  ```

### Testing Strategy

- Comprehensive test coverage:
  - Unit tests
  - Integration tests
  - E2E tests
- Test Requirements
  - Minimum test coverage: 85%
  - Every new feature requires:

    ```rust
    #[cfg(test)]
    mod tests {
        use super::*;
        
        #[tokio::test]
        async fn test_feature_happy_path() {
            // Happy path testing
        }
        
        #[tokio::test]
        async fn test_feature_error_cases() {
            // Error cases testing
        }
        
        #[tokio::test]
        async fn test_feature_edge_cases() {
            // Edge cases testing
        }
    }
    ```

- Integration Testing
  - Docker-based integration tests:

    ```rust
    #[tokio::test]
    async fn test_database_integration() {
        let docker = TestContainers::new();
        let postgres = docker.run(PostgresContainer::default());
        
        // Test database interactions
    }
    ```

- E2E Testing
  - Selenium/Playwright for frontend testing
  - API contract testing with Pact
  - Load testing with k6
  - Security testing with OWASP ZAP
- Test Data Management
  - Fixtures for common test scenarios
  - Factory patterns for test data generation:

    ```rust
    #[derive(Debug)]
    struct TestDataFactory {
        tenant_id: TenantId,
        config: TestConfig,
    }
    
    impl TestDataFactory {
        pub async fn create_user(&self) -> TestUser {
            // Create test user with proper i18n settings
        }
        
        pub async fn create_tenant(&self) -> TestTenant {
            // Create test tenant with i18n configurations
        }
    }
    ```

- Test Documentation
  - Test documentation in both German and English
  - Clear test naming conventions
  - Documentation of test data and scenarios
  - Regular test review process
- Continuous Testing
  - Pre-commit hooks for test execution
  - Automated test execution in CI/CD pipeline
  - Regular test maintenance and updates
  - Performance regression testing
  - i18n compliance testing:

    ```rust
    #[test]
    fn test_i18n_completeness() {
        let de_keys = collect_translation_keys("de");
        let en_keys = collect_translation_keys("en");
        
        assert_eq!(
            de_keys, 
            en_keys, 
            "Translation keys must match across all languages"
        );
    }
    ```

# Modern Rust Async Programming Guidelines

## Core Principles

- Write clear, idiomatic Rust code with precise examples
- Leverage async programming effectively using modern runtimes
- Design for modularity, maintainability, and resource efficiency
- Use expressive, intent-conveying variable names (e.g., `is_connected`, `has_pending_data`)
- Follow Rust naming conventions: snake_case for variables/functions, PascalCase for types/structs
- Minimize code duplication through proper abstraction
- Embrace Rust's type system, ownership model, and safety guarantees

## Async Foundations

### Runtimes and Ecosystem

- Use `tokio` (primary) or `async-std` (alternative) as async runtimes
- Choose runtime based on requirements:
  - `tokio`: Comprehensive ecosystem, production-proven
  - `async-std`: Simpler API, closer to std library design
- Leverage ecosystem crates:
  - `axum` for HTTP servers and web applications
  - `leptos` for the frontend
  - `reqwest` for HTTP clients
  - `tower` for middleware and service abstractions
  - `tonic` for gRPC services
  - `sea-orm` for async databases
  - `serde` for serialization
  - `fluent` for i18n
  - `tracing` for async-aware logging and diagnostics
  - `parking_lot` for thread-safe synchronization
  - `moka` for caching
  - `governor` for rate limiting
  - `rand` for random number generation
  - `base64` for encoding and decoding
  - `nonzero_ext` for non-zero integer types
  - `chrono` for date and time manipulation
  - `num_cpus` for CPU count

### Modern Async Patterns

- Implement async functions using `async fn` syntax
- Use async traits (stable as of Rust 1.75)
- Leverage the `Stream` trait for async iteration
- Utilize `futures` crate utilities and combinators
- Implement proper cancellation handling:

```rust
async fn cancelable_operation() {
    let _guard = tokio::select! {
        _ = async_operation() => {},
        _ = tokio::signal::ctrl_c() => {
            // Cleanup logic
            return;
        }
    };
}
```

## Concurrency and Task Management

### Task Spawning and Lifecycle

- Use `tokio::spawn` for background tasks
- Implement structured concurrency with proper task scoping
- Handle task cleanup with drop guards
- Implement graceful shutdown patterns:

```rust
async fn shutdown_gracefully(tasks: Vec<JoinHandle<()>>) {
    for task in tasks {
        task.abort();
        let _ = task.await;
    }
}
```

### Channels and Communication

- Use appropriate channel types:
  - `mpsc` for multi-producer, single-consumer
  - `broadcast` for multi-consumer scenarios
  - `oneshot` for single-use communication
  - `watch` for state distribution
- Implement backpressure using bounded channels
- Use worker pools for CPU-intensive tasks:

```rust
async fn worker_pool<T>(
    work_queue: mpsc::Receiver<T>,
    worker_count: usize,
) where T: Send + 'static {
    let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
    
    let workers = (0..worker_count)
        .map(|_| tokio::spawn(worker_loop(work_queue.clone(), shutdown_rx.clone())))
        .collect::<Vec<_>>();
}
```

### State Management

- Use async-aware synchronization primitives:
  - `tokio::sync::Mutex` for exclusive access
  - `tokio::sync::RwLock` for shared reads
  - `parking_lot` primitives for sync contexts
- Implement context propagation patterns
- Use `Arc` for shared ownership across tasks

## Error Handling and Safety

### Error Management

- Use `error-stack` for rich error context
- Implement custom error types when needed
- Use the `?` operator for error propagation
- Handle errors at appropriate abstraction levels

```rust
use error_stack::{Report, ResultExt};

#[derive(Debug)]
struct DatabaseError;

async fn db_operation() -> error_stack::Result<(), DatabaseError> {
    perform_operation()
        .await
        .change_context(DatabaseError)
        .attach_printable("Failed during critical operation")?;
    Ok(())
}
```

### Safety Considerations

- Ensure proper `Send` and `Sync` bounds
- Handle panics in async contexts using `catch_unwind`
- Use `unsafe` carefully in async code
- Implement proper pinning when needed:

```rust
use std::pin::Pin;
use std::task::{Context, Poll};

struct AsyncResource<T> {
    inner: Pin<Box<T>>,
}
```

## Observability and Diagnostics

### Logging and Tracing

- Use `tracing` for structured logging
- Implement span hierarchies for request flow
- Add context to log events

```rust
use tracing::{info, instrument};

#[instrument(skip(password))]
async fn authenticate(username: &str, password: &str) {
    info!("Authentication attempt for user: {}", username);
    // Authentication logic
}
```

### Metrics and Monitoring

- Implement metrics collection using `metrics`
- Add health checks and readiness probes
- Monitor async task queues and backlogs
- Track channel capacity and backpressure

## Testing

### Async Testing Patterns

- Use `tokio::test` for async unit tests
- Leverage `tokio-test` utilities
- Implement property-based testing with `proptest`
- Use `async-trait` for mocking

```rust
#[tokio::test]
async fn test_async_operation() {
    let mock = MockService::new();
    let result = perform_operation(&mock).await;
    assert!(result.is_ok());
}
```

### Time Control

- Use `tokio::time::pause` for time-dependent tests
- Implement deterministic async tests
- Test timeout and retry logic

## Performance Optimization

### Async Optimization

- Profile async stack usage
- Optimize `Future` polling patterns
- Use `tokio::task::yield_now` strategically
- Implement efficient cancellation

```rust
async fn optimized_operation() {
    if heavy_computation_needed() {
        tokio::task::spawn_blocking(|| {
            // CPU-intensive work
        }).await;
    }
    tokio::task::yield_now().await;
}
```

### Resource Management

- Implement connection pooling
- Use appropriate buffer sizes
- Monitor and tune task limits
- Implement rate limiting and throttling

## Project Structure

### Organization

- Separate concerns into modules:
  - `api/` - External interfaces
  - `domain/` - Business logic
  - `infrastructure/` - External services
  - `common/` - Shared utilities
- Use feature flags for optional functionality
- Implement clean architecture patterns

### Configuration

- Use `config` crate for configuration management
- Support multiple environments
- Implement secrets management
- Use environment variables with proper validation

## Documentation

- Write comprehensive Rustdoc
- Document async flows and patterns
- Include examples for complex operations
- Maintain architecture decision records (ADRs)

## Additional Project-Specific Patterns

### Multi-Tenancy Implementation

```rust
#[derive(Debug, Clone)]
struct TenantContext {
    id: TenantId,
    config: Arc<TenantConfig>,
}

async fn with_tenant<F, R>(tenant_id: TenantId, f: F) -> Result<R, TenantError>
where
    F: Future<Output = Result<R, TenantError>>,
{
    let tenant_ctx = load_tenant_context(tenant_id).await?;
    set_tenant_context(tenant_ctx).await?;
    f.await
}
```

### Security Patterns

```rust
#[derive(Debug)]
struct SecurityContext {
    tenant: TenantId,
    user: UserId,
    permissions: Arc<Permissions>,
}

async fn verify_access(ctx: &SecurityContext, resource: &Resource) -> Result<(), AccessError> {
    if !ctx.permissions.can_access(resource, ctx.tenant) {
        return Err(AccessError::InsufficientPermissions);
    }
    Ok(())
}
```

### Container Management

```rust
async fn manage_container_lifecycle() {
    let _guard = ContainerLifecycleGuard::new();
    
    tokio::select! {
        _ = handle_container_signals() => {},
        _ = perform_graceful_shutdown() => {},
    }
}
```

## Deployment Guidelines

### Docker Configuration

- Use multi-stage builds
- Implement health checks
- Configure proper shutdown signals
- Manage volume persistence

### Testing Strategy

- Implement container-based integration tests
- Use test containers for database testing
- Implement E2E tests with frontend
- Regular security audit testing

## Additional Resources

- [Multi-Tenancy Best Practices](https://docs.rs/tower-tenant)
- [Container Security Guidelines](https://docs.docker.com/security)
- [Kubernetes Deployment Patterns](https://kubernetes.io/docs/patterns)

### Security Requirements

- Audit Logging:

  ```rust
  #[audit_log(
      action = "user.create",
      tenant_context = true,
      sensitive_fields = ["password", "social_security_number"]
  )]
  async fn create_user(user: NewUser) -> Result<User, UserError> {
      // Implementation
  }
  ```

- Compliance Requirements:
  - GDPR compliance helpers
  - Data retention policies
  - Data export functionality
  - Right to be forgotten implementation

### Technical Requirements

[Nach dem existierenden Monitoring-Teil]

- Metrics Collection:

  ```rust
  #[instrument(
      name = "api.user.create",
      fields(tenant_id, user_type),
      metrics(
          histogram = "api_latency",
          counter = "api_calls_total"
      )
  )]
  async fn create_user() { /* ... */ }
  ```

- Health Checks:
  - Component-level health status
  - Dependency health monitoring
  - Custom health check implementation

### Technical Stack Requirements

#### Backend Framework Requirements

- Async-first architecture
- Proven production reliability
- Strong ecosystem support
- Good documentation and community
- Support for both REST and GraphQL
- Excellent performance characteristics
- Middleware support
- Flexible routing capabilities

#### Database Access Requirements

- Type-safe database interactions
- Migration support
- Connection pooling
- Transaction support
- Tenant isolation capabilities
- Async/non-blocking operations

#### Frontend Framework Requirements

- Component-based architecture
- Server-side rendering support
- Hydration capabilities
- Strong typing support
- i18n integration
- State management
- Form handling
- Testing utilities

#### Required Capabilities (Technology-agnostic)

- HTTP/2 support
- WebSocket support
- gRPC integration
- Authentication/Authorization
- Rate limiting
- Caching
- Metrics collection
- Distributed tracing
- Logging
- Health checking

#### Integration Requirements

- LDAP/Active Directory connectivity
- SMTP capability
- Message Queue support
- External API consumption
- File system operations
- Backup system integration

#### Development Requirements

- Hot reload support
- Debug capabilities
- Profile support
- Test frameworks
- Documentation generation
- API documentation
- Code formatting
- Linting
- Security scanning

### Infrastructure Requirements

#### Monitoring & Alerting

- Real-time system metrics
- Custom alert definitions per tenant
- Performance baseline monitoring
- Resource usage tracking
- SLA monitoring and reporting
- Audit log monitoring
- Security incident detection
- Integration with external monitoring systems

#### High Availability Requirements

- Multi-region deployment capability
- Automatic failover procedures
- Data replication strategies
- Recovery time objectives (RTO)
- Recovery point objectives (RPO)
- Business continuity procedures
- Disaster recovery testing requirements

#### Network Requirements

- Load balancing configuration
- SSL/TLS termination
- VPN support
- Network segmentation
- Firewall rules
- DDoS protection
- Traffic monitoring
- Proxy configuration support

#### Storage Requirements

- Scalable storage solutions
- Backup storage strategies
- Data lifecycle management
- Storage encryption requirements
- Performance requirements for different storage types
- Tenant data isolation on storage level
- Storage quota management

#### CI/CD Requirements

- Automated build environments
- Test environment provisioning
- Deployment pipeline requirements
- Release management
- Environment promotion strategy
- Configuration management
- Secret management in CI/CD
- Artifact storage and versioning

### Documentation Requirements

- All documentation must be provided in three languages:
  - English (primary)
  - German (full translation)
  - Albanian (full translation)
- Documentation path structure:

  ```
  doc/
  ├── architecture/
  │   ├── en/
  │   ├── de/
  │   └── sq/
  └── contributing/
      ├── en/
      ├── de/
      └── sq/
  ```

- Every markdown file must have corresponding versions in all three languages
- Code comments must be in English only
- User-facing strings must use fluent (FTL) for i18n
- Documentation must be updated with every code change
- Version-specific documentation required for API changes

### Language-Specific Requirements

- English: Primary development language, used for all code and comments, documentation in root folders
- German: Required for all documentation (in de/ folders) and user interfaces
- Albanian: Required for all documentation (in sq/ folders) and user interfaces
- Translation verification required in CI pipeline
- No hardcoded strings allowed in code
- All UI text must use i18n system
- Documentation must maintain consistent terminology across languages

### Multi-Language Testing Requirements

- i18n test coverage required
- String format verification across languages
- Character encoding tests
- Translation completeness checks
- UI layout verification for all languages
- Error message translation verification
- Documentation link verification across languages

### Development Workflow Requirements

- Branch Protection Rules:
  - Main and develop branches must be protected
  - Required reviews before merge
  - Required status checks to pass
  - No direct commits to protected branches

### Code Quality Requirements

- Static Analysis:
  - Clippy with strict settings
  - Custom linting rules for i18n compliance
  - Dependency vulnerability scanning
  - Code complexity metrics

### Feature Flag Requirements

- Use centralized feature flag configuration
- Implement strict feature flag naming convention:
  - Format: `{domain}_{feature}_{subfeature}`
  - Example: `auth_mfa_totp`
- Document all feature flags in code and documentation
- Implement proper feature flag cleanup
- Use type-safe feature flag definitions
- Implement proper fallback values
- Cache feature states appropriately
- Log feature flag evaluations
- Implement proper feature flag testing

### SBOM Requirements

- Automated SBOM generation in CI/CD pipeline
- Regular SBOM updates and validation
- SBOM format compliance (CycloneDX)
- Dependency vulnerability tracking
- License compliance monitoring
- SBOM versioning and tracking
- Component origin verification
- Integration points:
  - CI/CD pipeline integration
  - Security audit integration
  - Dependency update workflow
  - Release process integration
  - Audit logging integration

### Performance Requirements

- Response Time Targets:
  - API endpoints: < 100ms for 95th percentile
  - GraphQL queries: < 200ms for 95th percentile
  - Database queries: < 50ms for 95th percentile
- Resource Usage Limits:
  - Memory footprint per service
  - CPU utilization thresholds
  - Connection pool sizes

### EVENT DRIVEN ARCHITECTURE

- Event Patterns:
  - Event Schema Definition und Versionierung
  - Event Validation
  - Dead Letter Queues
  - Event Replay Capabilities

- Message Broker Requirements:
  - At-least-once Delivery
  - Message Persistence
  - Topic/Queue Management
  - Multi-Tenant Event Isolation

### CACHING STRATEGY

- Multi-Level Caching:
  - Application-Level Cache (Memory)
  - Distributed Cache (Redis)

- Cache Policies:
  - TTL Definitionen
  - Cache Invalidation Strategien
  - Cache Warming
  - Tenant-spezifisches Caching

### BACKUP & RECOVERY

- Backup Types:
  - Full System Backups
  - Incremental Backups
  - Point-in-Time Recovery
  - Tenant-specific Backups

- Recovery Procedures:
  - RTO (Recovery Time Objective) < 4 Stunden
  - RPO (Recovery Point Objective) < 15 Minuten
  - Automated Recovery Tests
  - Documented Recovery Procedures

### API GATEWAY REQUIREMENTS

- Gateway Features:
  - Request/Response Transformation
  - API Aggregation
  - Cross-Origin Resource Sharing (CORS)
  - API Versioning

- Security Features:
  - API Key Management
  - OAuth2/OIDC Integration
  - Rate Limiting
  - Request Validation
