# Entwickler-Guide

## Entwicklungsumgebung

### Voraussetzungen
- Rust 1.75 oder höher
- Docker 24.0 oder höher
- Git 2.40 oder höher
- VS Code oder IntelliJ mit Rust-Plugin
- Trunk (für Leptos Frontend-Entwicklung)

### Installation
```bash
# Rust Installation
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup component add rustfmt clippy
rustup target add wasm32-unknown-unknown

# Development Tools
cargo install cargo-watch
cargo install cargo-audit
cargo install cargo-deny
cargo install trunk
```

### IDE-Konfiguration
```json
// VS Code settings.json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.allFeatures": true,
    "editor.formatOnSave": true,
    "rust-analyzer.cargo.target": "wasm32-unknown-unknown"
}
```

## Code-Style-Guide

### Rust-Formatierung
- Verwende `rustfmt` mit der Standard-Konfiguration
- Maximale Zeilenlänge: 100 Zeichen
- Verwende Blockkommentare für Dokumentation
- Inline-Kommentare nur für komplexe Logik

### Namenskonventionen
```rust
// Structs: PascalCase
pub struct UserProfile {
    // Fields: snake_case
    first_name: String,
    last_name: String,
}

// Functions: snake_case
pub fn validate_user_input(input: &str) -> Result<(), Error> {
    // ...
}

// Constants: SCREAMING_SNAKE_CASE
const MAX_CONNECTION_RETRIES: u32 = 3;
```

### Code-Organisation
```rust
// Modulstruktur
src/
├── main.rs
├── lib.rs
├── config/
├── api/
├── domain/
├── infrastructure/
└── utils/
```

## Git-Workflow

### Branch-Strategie
- `main`: Produktionscode
- `develop`: Entwicklungszweig
- `feature/*`: Neue Features
- `bugfix/*`: Fehlerbehebungen
- `release/*`: Release-Vorbereitung

### Commit-Konventionen
```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Typen:
- `feat`: Neues Feature
- `fix`: Fehlerbehebung
- `docs`: Dokumentation
- `style`: Formatierung
- `refactor`: Code-Umstrukturierung
- `test`: Tests
- `chore`: Wartungsarbeiten

### Pull Request Prozess
1. Feature-Branch von `develop` erstellen
2. Code implementieren und testen
3. Pull Request erstellen
4. Code Review durchführen
5. CI/CD-Pipeline erfolgreich durchlaufen
6. Nach `develop` mergen

## Testing-Strategien

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_validation() {
        let input = "test@example.com";
        assert!(validate_email(input).is_ok());
    }

    #[test]
    fn test_invalid_email() {
        let input = "invalid-email";
        assert!(validate_email(input).is_err());
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_user_creation_flow() {
    let app = test_setup().await;
    let client = reqwest::Client::new();

    let response = client
        .post(&format!("{}/users", app.address))
        .json(&test_user())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 201);
}
```

### Property-Based Testing
```rust
#[test]
fn test_user_properties() {
    proptest!(|(name in "[a-zA-Z]{2,50}")| {
        let user = User::new(&name);
        prop_assert!(user.name.len() >= 2);
        prop_assert!(user.name.len() <= 50);
    });
}
```

## CI/CD-Pipeline

### GitHub Actions Workflow
```yaml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Run tests
        run: cargo test --all-features
        
      - name: Run clippy
        run: cargo clippy -- -D warnings
        
      - name: Check formatting
        run: cargo fmt -- --check

  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Security audit
        run: cargo audit
        
      - name: Check dependencies
        run: cargo deny check

  build:
    needs: [test, security]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Build
        run: cargo build --release
```

### Deployment-Pipeline
```yaml
name: CD

on:
  push:
    tags:
      - 'v*'

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Build Docker image
        run: docker build -t app:${GITHUB_REF#refs/tags/} .
        
      - name: Push to registry
        run: |
          echo ${{ secrets.DOCKER_TOKEN }} | docker login -u ${{ secrets.DOCKER_USER }} --password-stdin
          docker push app:${GITHUB_REF#refs/tags/}
```

## Debugging

### Logging
```rust
use tracing::{info, error, warn, debug};

pub fn process_request(req: Request) {
    debug!("Processing request: {:?}", req);
    
    if let Err(e) = validate_request(&req) {
        error!("Validation failed: {}", e);
        return;
    }
    
    info!("Request processed successfully");
}
```

### Fehlerbehandlung
```rust
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Validation failed: {0}")]
    ValidationError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Not found: {0}")]
    NotFound(String),
}

impl From<ApiError> for Response {
    fn from(error: ApiError) -> Self {
        match error {
            ApiError::ValidationError(_) => Response::new(400),
            ApiError::NotFound(_) => Response::new(404),
            _ => Response::new(500),
        }
    }
}
```

## Performance-Optimierung

### Profiling
```rust
use tracing::instrument;

#[instrument(skip(password))]
pub async fn hash_password(password: &str) -> Result<String, Error> {
    let start = std::time::Instant::now();
    let hash = tokio::task::spawn_blocking(move || {
        bcrypt::hash(password, 12)
    }).await??;
    
    info!("Password hashing took {:?}", start.elapsed());
    Ok(hash)
}
```

### Caching
```rust
use cached::proc_macro::cached;

#[cached(
    time = 300, // 5 minutes
    result = true,
    key = "String",
    convert = r#"{ format!("{}", user_id) }"#
)]
pub async fn get_user_profile(user_id: i64) -> Result<UserProfile, Error> {
    // Datenbankabfrage
}
```

## Sicherheit

### Input-Validierung
```rust
use validator::Validate;

#[derive(Debug, Validate)]
pub struct UserInput {
    #[validate(email)]
    email: String,
    
    #[validate(length(min = 8, max = 72))]
    password: String,
    
    #[validate(length(min = 2, max = 50))]
    name: String,
}
```

### SQL-Injection-Prävention
```rust
// Verwende immer parametrisierte Queries
pub async fn get_user(id: i64) -> Result<User, Error> {
    sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id = $1",
        id
    )
    .fetch_one(&pool)
    .await
}
```

## Wartung

### Dependency Updates
```bash
# Überprüfe auf Updates
cargo outdated

# Aktualisiere Dependencies
cargo update

# Überprüfe auf Sicherheitslücken
cargo audit
```

### Monitoring
```rust
use metrics::{counter, gauge};

pub fn track_request() {
    counter!("api.requests.total", 1);
    gauge!("api.requests.active", 1.0);
}
```

## Best Practices

### Error Handling
- Verwende `thiserror` für Bibliotheksfehler
- Verwende `anyhow` für Anwendungsfehler
- Implementiere eigene Fehlertypen
- Logge Fehler mit Kontext

### Async/Await
- Verwende `tokio` als Runtime
- Implementiere `Stream` für große Datensätze
- Nutze Connection Pools
- Behandle Timeouts korrekt

### Testing
- Schreibe Unit-Tests für Logik
- Implementiere Integrationstests
- Verwende Property-Based Testing
- Mocke externe Dienste

### Dokumentation
- Dokumentiere öffentliche APIs
- Füge Beispiele hinzu
- Halte CHANGELOG.md aktuell
- Schreibe aussagekräftige Commit-Messages 