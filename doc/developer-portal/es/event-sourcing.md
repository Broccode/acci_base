# Guía de Event-Sourcing & CQRS

## Visión General

Esta guía describe la implementación de Event-Sourcing y CQRS (Command Query Responsibility Segregation) en el Framework ACCI. Estos patrones arquitectónicos permiten una aplicación escalable, mantenible y auditable.

## Conceptos Básicos

### Event-Sourcing

- Todos los cambios se almacenan como eventos
- El estado actual se reconstruye mediante la reproducción de eventos
- Historial de auditoría completo disponible
- Posibilidad de consultas temporales

### CQRS

- Separación de modelos de lectura y escritura
- Los comandos modifican el estado
- Las consultas leen proyecciones optimizadas
- Actualización asíncrona de los modelos de lectura

## Event Store

### Estructura de Evento

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub event_type: String,
    pub version: i32,
    pub data: Value,
    pub metadata: EventMetadata,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventMetadata {
    pub user_id: Option<String>,
    pub correlation_id: String,
    pub causation_id: Option<String>,
    pub tenant_id: String,
}
```

### Almacenamiento de Eventos

```rust
pub async fn store_event(event: Event) -> Result<(), Error> {
    sqlx::query!(
        r#"
        INSERT INTO events (
            id, aggregate_id, aggregate_type, event_type,
            version, data, metadata, timestamp
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        event.id,
        event.aggregate_id,
        event.aggregate_type,
        event.event_type,
        event.version,
        event.data,
        event.metadata,
        event.timestamp,
    )
    .execute(&pool)
    .await?;

    // Notificación de proyecciones para el evento
    notify_projections(&event).await?;
    
    Ok(())
}
```

### Consulta de Event Stream

```rust
pub async fn get_events(
    aggregate_id: &str,
    from_version: Option<i32>,
) -> Result<Vec<Event>, Error> {
    sqlx::query_as!(
        Event,
        r#"
        SELECT * FROM events
        WHERE aggregate_id = $1
        AND ($2::int IS NULL OR version > $2)
        ORDER BY version ASC
        "#,
        aggregate_id,
        from_version,
    )
    .fetch_all(&pool)
    .await
}
```

## Manejo de Comandos

### Estructura de Comando

```rust
#[derive(Debug)]
pub struct Command<T> {
    pub id: Uuid,
    pub data: T,
    pub metadata: CommandMetadata,
}

#[derive(Debug)]
pub struct CommandMetadata {
    pub user_id: String,
    pub correlation_id: String,
    pub tenant_id: String,
    pub timestamp: DateTime<Utc>,
}
```

### Manejador de Comandos

```rust
#[async_trait]
pub trait CommandHandler<T> {
    async fn handle(&self, command: Command<T>) -> Result<Vec<Event>, Error>;
}

pub struct CreateUserHandler {
    event_store: Arc<dyn EventStore>,
}

#[async_trait]
impl CommandHandler<CreateUserCommand> for CreateUserHandler {
    async fn handle(&self, cmd: Command<CreateUserCommand>) -> Result<Vec<Event>, Error> {
        // Validación
        validate_command(&cmd)?;

        // Cargar agregado desde Event Store
        let events = self.event_store
            .get_events(&cmd.data.user_id, None)
            .await?;
        let user = User::from_events(events)?;

        // Ejecutar lógica de negocio
        let new_events = user.process_create_command(cmd)?;

        // Almacenar eventos
        for event in &new_events {
            self.event_store.store_event(event.clone()).await?;
        }

        Ok(new_events)
    }
}
```

## Proyecciones

### Definición de Proyección

```rust
#[async_trait]
pub trait Projection {
    async fn handle_event(&mut self, event: &Event) -> Result<(), Error>;
    async fn rebuild(&mut self) -> Result<(), Error>;
}

pub struct UserProjection {
    pool: PgPool,
}

#[async_trait]
impl Projection for UserProjection {
    async fn handle_event(&mut self, event: &Event) -> Result<(), Error> {
        match event.event_type.as_str() {
            "UserCreated" => {
                let data: UserCreatedData = serde_json::from_value(event.data.clone())?;
                sqlx::query!(
                    r#"
                    INSERT INTO user_view (
                        id, email, name, status, created_at
                    ) VALUES ($1, $2, $3, $4, $5)
                    "#,
                    data.user_id,
                    data.email,
                    data.name,
                    "active",
                    event.timestamp,
                )
                .execute(&self.pool)
                .await?;
            }
            "UserUpdated" => {
                // Lógica de actualización
            }
            _ => {}
        }
        Ok(())
    }

    async fn rebuild(&mut self) -> Result<(), Error> {
        // Eliminar modelo de lectura
        sqlx::query!("TRUNCATE TABLE user_view")
            .execute(&self.pool)
            .await?;

        // Reproducir todos los eventos
        let events = get_all_events().await?;
        for event in events {
            self.handle_event(&event).await?;
        }
        Ok(())
    }
}
```

### Gestor de Proyecciones

```rust
pub struct ProjectionManager {
    projections: Vec<Box<dyn Projection>>,
}

impl ProjectionManager {
    pub async fn handle_event(&mut self, event: &Event) -> Result<(), Error> {
        for projection in &mut self.projections {
            projection.handle_event(event).await?;
        }
        Ok(())
    }

    pub async fn rebuild_all(&mut self) -> Result<(), Error> {
        for projection in &mut self.projections {
            projection.rebuild().await?;
        }
        Ok(())
    }
}
```

## Consultas

### Manejador de Consultas

```rust
pub struct UserQueryHandler {
    pool: PgPool,
}

impl UserQueryHandler {
    pub async fn get_user(&self, id: &str) -> Result<UserView, Error> {
        sqlx::query_as!(
            UserView,
            r#"
            SELECT * FROM user_view
            WHERE id = $1
            "#,
            id,
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn search_users(
        &self,
        query: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<UserView>, Error> {
        sqlx::query_as!(
            UserView,
            r#"
            SELECT * FROM user_view
            WHERE name ILIKE $1 OR email ILIKE $1
            ORDER BY name
            LIMIT $2 OFFSET $3
            "#,
            format!("%{}%", query),
            limit,
            offset,
        )
        .fetch_all(&self.pool)
        .await
    }
}
```

## Consistencia & Consistencia Eventual

### Control de Concurrencia Optimista

```rust
pub async fn check_version(
    aggregate_id: &str,
    expected_version: i32,
) -> Result<(), Error> {
    let current_version = sqlx::query!(
        r#"
        SELECT MAX(version) as version
        FROM events
        WHERE aggregate_id = $1
        "#,
        aggregate_id,
    )
    .fetch_one(&pool)
    .await?
    .version
    .unwrap_or(0);

    if current_version != expected_version {
        return Err(Error::ConcurrencyError {
            aggregate_id: aggregate_id.to_string(),
            expected_version,
            actual_version: current_version,
        });
    }

    Ok(())
}
```

### Mecanismo de Snapshot

```rust
pub struct Snapshot {
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub version: i32,
    pub state: Value,
    pub timestamp: DateTime<Utc>,
}

impl EventStore {
    pub async fn save_snapshot(&self, snapshot: Snapshot) -> Result<(), Error> {
        sqlx::query!(
            r#"
            INSERT INTO snapshots (
                aggregate_id, aggregate_type, version,
                state, timestamp
            ) VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (aggregate_id)
            DO UPDATE SET
                version = EXCLUDED.version,
                state = EXCLUDED.state,
                timestamp = EXCLUDED.timestamp
            "#,
            snapshot.aggregate_id,
            snapshot.aggregate_type,
            snapshot.version,
            snapshot.state,
            snapshot.timestamp,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn load_snapshot(
        &self,
        aggregate_id: &str,
    ) -> Result<Option<Snapshot>, Error> {
        sqlx::query_as!(
            Snapshot,
            r#"
            SELECT * FROM snapshots
            WHERE aggregate_id = $1
            "#,
            aggregate_id,
        )
        .fetch_optional(&self.pool)
        .await
    }
}
```

## Pruebas

### Pruebas Unitarias

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_user_command() {
        let handler = CreateUserHandler::new(MockEventStore::new());
        
        let cmd = Command {
            id: Uuid::new_v4(),
            data: CreateUserCommand {
                user_id: "user123".to_string(),
                email: "test@example.com".to_string(),
                name: "Test User".to_string(),
            },
            metadata: CommandMetadata {
                user_id: "admin".to_string(),
                correlation_id: "test123".to_string(),
                tenant_id: "tenant1".to_string(),
                timestamp: Utc::now(),
            },
        };

        let events = handler.handle(cmd).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "UserCreated");
    }
}
```

### Pruebas de Integración

```rust
#[tokio::test]
async fn test_user_projection() {
    let mut projection = UserProjection::new(test_db_pool().await);
    
    // Generar evento
    let event = Event {
        id: Uuid::new_v4(),
        aggregate_id: "user123".to_string(),
        aggregate_type: "User".to_string(),
        event_type: "UserCreated".to_string(),
        version: 1,
        data: json!({
            "user_id": "user123",
            "email": "test@example.com",
            "name": "Test User"
        }),
        metadata: EventMetadata::default(),
        timestamp: Utc::now(),
    };

    // Procesar evento
    projection.handle_event(&event).await.unwrap();

    // Verificar modelo de lectura
    let user = sqlx::query_as!(
        UserView,
        "SELECT * FROM user_view WHERE id = $1",
        "user123"
    )
    .fetch_one(&test_db_pool().await)
    .await
    .unwrap();

    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.name, "Test User");
}
```

## Optimización de Rendimiento

### Índices Event Store

```sql
-- Índice primario para eventos
CREATE INDEX idx_events_aggregate ON events (aggregate_id, version);

-- Índice para tipos de eventos
CREATE INDEX idx_events_type ON events (event_type);

-- Índice para timestamps
CREATE INDEX idx_events_timestamp ON events (timestamp);
```

### Optimización de Proyecciones

```rust
impl UserProjection {
    pub async fn handle_batch(
        &mut self,
        events: &[Event],
    ) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;

        for event in events {
            match event.event_type.as_str() {
                "UserCreated" => {
                    // Lógica de inserción por lotes
                }
                "UserUpdated" => {
                    // Lógica de actualización por lotes
                }
                _ => {}
            }
        }

        tx.commit().await?;
        Ok(())
    }
}
```

## Monitoreo

### Métricas

```rust
pub struct EventSourcingMetrics {
    event_counter: Counter,
    projection_lag: Gauge,
    snapshot_size: Histogram,
}

impl EventSourcingMetrics {
    pub fn record_event(&self, event_type: &str) {
        self.event_counter
            .with_label_values(&[event_type])
            .inc();
    }

    pub fn update_projection_lag(&self, lag: Duration) {
        self.projection_lag
            .with_label_values(&["user_projection"])
            .set(lag.as_secs_f64());
    }
}
```

### Controles de Salud

```rust
pub async fn check_event_store_health() -> Result<(), Error> {
    // Verificar últimos eventos
    let latest_event = sqlx::query!(
        r#"
        SELECT MAX(timestamp) as last_event
        FROM events
        "#
    )
    .fetch_one(&pool)
    .await?;

    // Verificar si se han procesado eventos recientemente
    if let Some(last_event) = latest_event.last_event {
        let age = Utc::now() - last_event;
        if age > Duration::minutes(5) {
            return Err(Error::HealthCheck(
                "No recent events processed".to_string()
            ));
        }
    }

    Ok(())
}
```

## Mejores Prácticas

### Diseño de Eventos
- Eventos como hechos del pasado
- Inmutables y auto-descriptivos
- Versionado para evolución del esquema
- Contexto suficiente en metadatos

### Diseño de Comandos
- Validación a nivel de comando
- Consideración de idempotencia
- Intención clara en el nombre
- Datos mínimos pero completos

### Diseño de Proyecciones
- Optimizadas para patrones de consulta
- Actualizaciones incrementales
- Reconstruibles
- Índices eficientes

### Manejo de Errores
- Reintentos para errores temporales
- Cola de mensajes muertos para eventos fallidos
- Monitoreo de inconsistencias
- Mensajes de error claros 