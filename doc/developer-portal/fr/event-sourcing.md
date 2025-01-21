# Guide Event-Sourcing & CQRS

## Vue d'ensemble

Ce guide décrit l'implémentation de l'Event-Sourcing et du CQRS (Command Query Responsibility Segregation) dans le Framework ACCI. Ces modèles d'architecture permettent une application évolutive, maintenable et auditable.

## Concepts de Base

### Event-Sourcing

- Tous les changements sont stockés sous forme d'événements
- L'état actuel est reconstruit par la relecture des événements
- Historique d'audit complet disponible
- Possibilité de requêtes temporelles

### CQRS

- Séparation des modèles de lecture et d'écriture
- Les commandes modifient l'état
- Les requêtes lisent des projections optimisées
- Mise à jour asynchrone des modèles de lecture

## Event Store

### Structure d'Événement

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

### Stockage d'Événements

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

    // Notification des projections pour l'événement
    notify_projections(&event).await?;
    
    Ok(())
}
```

### Requête Event Stream

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

## Gestion des Commandes

### Structure de Commande

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

### Gestionnaire de Commandes

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
        // Validation
        validate_command(&cmd)?;

        // Charger l'agrégat depuis l'Event Store
        let events = self.event_store
            .get_events(&cmd.data.user_id, None)
            .await?;
        let user = User::from_events(events)?;

        // Exécuter la logique métier
        let new_events = user.process_create_command(cmd)?;

        // Stocker les événements
        for event in &new_events {
            self.event_store.store_event(event.clone()).await?;
        }

        Ok(new_events)
    }
}
```

## Projections

### Définition de Projection

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
                // Logique de mise à jour
            }
            _ => {}
        }
        Ok(())
    }

    async fn rebuild(&mut self) -> Result<(), Error> {
        // Supprimer le modèle de lecture
        sqlx::query!("TRUNCATE TABLE user_view")
            .execute(&self.pool)
            .await?;

        // Rejouer tous les événements
        let events = get_all_events().await?;
        for event in events {
            self.handle_event(&event).await?;
        }
        Ok(())
    }
}
```

### Gestionnaire de Projections

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

## Requêtes

### Gestionnaire de Requêtes

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

## Cohérence & Cohérence Éventuelle

### Contrôle de Concurrence Optimiste

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

### Mécanisme de Snapshot

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

## Tests

### Tests Unitaires

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

### Tests d'Intégration

```rust
#[tokio::test]
async fn test_user_projection() {
    let mut projection = UserProjection::new(test_db_pool().await);
    
    // Générer l'événement
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

    // Traiter l'événement
    projection.handle_event(&event).await.unwrap();

    // Vérifier le modèle de lecture
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

## Optimisation des Performances

### Index Event Store

```sql
-- Index primaire pour les événements
CREATE INDEX idx_events_aggregate ON events (aggregate_id, version);

-- Index pour les types d'événements
CREATE INDEX idx_events_type ON events (event_type);

-- Index pour les timestamps
CREATE INDEX idx_events_timestamp ON events (timestamp);
```

### Optimisation des Projections

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
                    // Logique d'insertion par lots
                }
                "UserUpdated" => {
                    // Logique de mise à jour par lots
                }
                _ => {}
            }
        }

        tx.commit().await?;
        Ok(())
    }
}
```

## Monitoring

### Métriques

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

### Contrôles de Santé

```rust
pub async fn check_event_store_health() -> Result<(), Error> {
    // Vérifier les derniers événements
    let latest_event = sqlx::query!(
        r#"
        SELECT MAX(timestamp) as last_event
        FROM events
        "#
    )
    .fetch_one(&pool)
    .await?;

    // Vérifier si des événements ont été traités récemment
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

## Meilleures Pratiques

### Conception d'Événements
- Événements comme faits du passé
- Immuables et auto-descriptifs
- Versionnement pour l'évolution du schéma
- Contexte suffisant dans les métadonnées

### Conception des Commandes
- Validation au niveau de la commande
- Prise en compte de l'idempotence
- Intention claire dans le nom
- Données minimales mais complètes

### Conception des Projections
- Optimisées pour les modèles de requêtes
- Mises à jour incrémentales
- Reconstructibles
- Index performants

### Gestion des Erreurs
- Tentatives pour les erreurs temporaires
- File d'attente des lettres mortes pour les événements échoués
- Surveillance des incohérences
- Messages d'erreur clairs 