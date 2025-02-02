# CQRS and Event Sourcing Architecture

## Overview

The ACCI Framework implements Command Query Responsibility Segregation (CQRS) and Event Sourcing patterns to achieve:

- Clear separation of write and read operations
- Scalable and maintainable event-driven architecture
- Complete audit trail of all system changes
- Eventual consistency with optimized read models
- Event-driven integration capabilities

## Core Components

### Command Side

```rust
pub struct CommandSide {
    command_bus: CommandBus,
    command_handlers: Vec<Box<dyn CommandHandler>>,
    aggregate_repository: AggregateRepository,
    event_store: EventStore,
}
```

#### Command Flow

1. Command validation
2. Command handling
3. Aggregate state changes
4. Event generation
5. Event persistence
6. Event publication

### Query Side

```rust
pub struct QuerySide {
    query_bus: QueryBus,
    read_models: Vec<Box<dyn ReadModel>>,
    projections: Vec<Box<dyn Projection>>,
    query_handlers: Vec<Box<dyn QueryHandler>>,
}
```

#### Query Flow

1. Query validation
2. Read model access
3. Data transformation
4. Response generation

## Event Store

### Event Structure

```rust
pub struct Event {
    event_id: EventId,
    aggregate_id: AggregateId,
    aggregate_type: String,
    event_type: String,
    event_version: u32,
    payload: EventPayload,
    metadata: EventMetadata,
    timestamp: DateTime<Utc>,
}
```

### Event Storage

- Uses PostgreSQL with JSONB for event persistence
- Implements event streams per aggregate with optimized indices
- Supports snapshots for performance optimization
- Provides event versioning and schema evolution
- Leverages PostgreSQL partitioning for tenant isolation
- Implements optimistic concurrency control

### Database Schema

```sql
CREATE SCHEMA event_store;
CREATE SCHEMA snapshots;

CREATE TABLE event_store.events (
    event_id UUID PRIMARY KEY,
    aggregate_id TEXT NOT NULL,
    aggregate_type TEXT NOT NULL,
    sequence_number BIGINT NOT NULL,
    event_type TEXT NOT NULL,
    event_version INTEGER NOT NULL,
    data JSONB NOT NULL,
    metadata JSONB NOT NULL,
    tenant_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    version INTEGER NOT NULL,
    CONSTRAINT unique_aggregate_sequence UNIQUE (aggregate_id, sequence_number)
);

CREATE TABLE snapshots.aggregate_snapshots (
    snapshot_id UUID PRIMARY KEY,
    aggregate_id TEXT NOT NULL,
    aggregate_type TEXT NOT NULL,
    sequence_number BIGINT NOT NULL,
    state JSONB NOT NULL,
    metadata JSONB NOT NULL,
    tenant_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT unique_aggregate_snapshot UNIQUE (aggregate_id, sequence_number)
);
```

## Projections

### Types of Projections

1. Real-time projections
2. Catch-up projections
3. Custom projections for specific use cases

### Implementation

```rust
pub trait Projection {
    fn handle_event(&mut self, event: &Event) -> Result<(), ProjectionError>;
    fn get_position(&self) -> Position;
    fn set_position(&mut self, position: Position);
}
```

## Read Models

### Characteristics

- Denormalized for query optimization
- Eventually consistent
- Purpose-built for specific query requirements
- Cached where appropriate

### Example

```rust
pub struct TenantReadModel {
    tenant_id: TenantId,
    name: String,
    status: TenantStatus,
    user_count: u32,
    last_updated: DateTime<Utc>,
    settings: TenantSettings,
}
```

## Event Sourcing

### Benefits

1. Complete audit trail
2. Temporal queries
3. Event replay capability
4. System reconstruction
5. Debug capabilities

### Implementation Details

```rust
pub trait EventSourced {
    fn apply_event(&mut self, event: &Event) -> Result<(), DomainError>;
    fn get_uncommitted_events(&self) -> Vec<Event>;
    fn clear_uncommitted_events(&mut self);
}
```

## Consistency Considerations

### Command Side

- Strong consistency within aggregate boundaries
- Optimistic concurrency control
- Event versioning

### Query Side

- Eventually consistent read models
- Configurable consistency delays
- Monitoring of projection lag

## Performance Optimization

### Command Side

- Aggregate caching
- Snapshot generation
- Batch event persistence

### Query Side

- Read model caching
- Materialized views
- Query optimization

## Integration Patterns

### Event Publishing

```rust
pub trait EventPublisher {
    fn publish_event(&self, event: &Event) -> Result<(), PublishError>;
    fn publish_events(&self, events: &[Event]) -> Result<(), PublishError>;
}
```

### Subscription Handling

```rust
pub trait EventSubscriber {
    fn subscribe(&self, event_types: &[String]) -> Result<Subscription, SubscribeError>;
    fn unsubscribe(&self, subscription: Subscription) -> Result<(), UnsubscribeError>;
}
```

## Testing Strategy

### Command Side Tests

1. Command validation tests
2. Event generation tests
3. State transition tests
4. Concurrency tests

### Query Side Tests

1. Projection tests
2. Read model tests
3. Query handler tests
4. Integration tests

## Monitoring and Metrics

### Command Metrics

- Command processing time
- Event persistence latency
- Failed commands
- Concurrent modifications

### Query Metrics

- Query response time
- Projection lag
- Cache hit ratio
- Read model consistency

## Error Handling

### Command Errors

```rust
pub enum CommandError {
    ValidationError(String),
    ConcurrencyError(String),
    PersistenceError(String),
    DomainError(String),
}
```

### Query Errors

```rust
pub enum QueryError {
    ValidationError(String),
    NotFoundError(String),
    ProjectionError(String),
    ReadModelError(String),
}
```

## Best Practices

1. Keep aggregates small and focused
2. Design events for future use
3. Version all events
4. Implement idempotency
5. Monitor projection lag
6. Cache appropriately
7. Plan for event schema evolution
8. Implement proper error handling
9. Use meaningful event names
10. Document event schemas

## Migration Strategies

### Event Schema Migration

1. Upcasting
2. Versioning
3. Transformation
4. Validation

### Read Model Migration

1. Rebuild from events
2. Incremental updates
3. Parallel running
4. Validation checks

## Security Considerations

1. Event data encryption
2. Access control for commands
3. Read model authorization
4. Audit logging
5. Event store security

## Deployment Considerations

1. Event store clustering
2. Read model scaling
3. Projection worker deployment
4. Monitoring setup
5. Backup strategies

## Observability

### Metrics (Prometheus/OpenMetrics)

- Event Store performance metrics
- Command and Query processing metrics
- Projection lag monitoring
- Cache effectiveness metrics
- Business metrics per tenant

### Tracing (OpenTelemetry)

- Distributed tracing across command and query flows
- Detailed operation spans
- Cross-service correlation
- Performance bottleneck identification

### Logging

- Structured JSON logging
- Correlation IDs across operations
- Tenant-aware logging
- Security audit logging

### Health Checks

- Database connectivity
- Event Store write capability
- Read model consistency
- Projection health
- Overall system health

## Backup and Recovery

### Backup Strategy

- Regular full backups of event store
- Snapshot store backups
- Read model backups (optional)
- Point-in-Time recovery capability
- Tenant-aware backup scheduling

### Recovery Procedures

- Full system recovery
- Single tenant recovery
- Point-in-Time reconstruction
- Read model rebuilding
- Consistency verification

### Monitoring

- Backup success/failure metrics
- Backup size and duration tracking
- Recovery time objectives (RTO)
- Recovery point objectives (RPO)
- Compliance verification

## Tools and Infrastructure

1. PostgreSQL for event storage and read models
2. Redis for caching
3. RabbitMQ for event distribution
4. Prometheus for metrics
5. Grafana for monitoring
6. OpenTelemetry for tracing
7. Jaeger for trace visualization

## Example Implementations

### Command Handler

```rust
impl CommandHandler for CreateTenantHandler {
    fn handle(&self, cmd: CreateTenant) -> Result<Vec<Event>, CommandError> {
        // Validate command
        self.validate(&cmd)?;

        // Create new aggregate
        let mut tenant = TenantAggregate::new();

        // Generate events
        let events = tenant.create(cmd)?;

        // Persist events
        self.event_store.append_events(&events)?;

        Ok(events)
    }
}
```

### Projection Handler

```rust
impl ProjectionHandler for TenantProjection {
    fn handle_event(&mut self, event: &Event) -> Result<(), ProjectionError> {
        match event.event_type.as_str() {
            "TenantCreated" => self.handle_tenant_created(event),
            "TenantUpdated" => self.handle_tenant_updated(event),
            "TenantDeleted" => self.handle_tenant_deleted(event),
            _ => Ok(()),
        }
    }
}
```

### Query Handler

```rust
impl QueryHandler for GetTenantDetailsHandler {
    fn handle(&self, query: GetTenantDetails) -> Result<TenantReadModel, QueryError> {
        // Validate query
        self.validate(&query)?;

        // Get from cache or read model
        self.read_model
            .get_tenant(query.tenant_id)
            .ok_or(QueryError::NotFound)
    }
} 