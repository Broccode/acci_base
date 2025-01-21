# Udhëzuesi i Zhvilluesit

## Mjedisi i Zhvillimit

### Kërkesat
- Rust 1.75 ose më i lartë
- Docker 24.0 ose më i lartë
- Git 2.40 ose më i lartë
- VS Code ose IntelliJ me plugin Rust
- Trunk (për zhvillimin e frontend-it me Leptos)

### Instalimi
```bash
# Instalimi i Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup component add rustfmt clippy
rustup target add wasm32-unknown-unknown

# Mjetet e Zhvillimit
cargo install cargo-watch
cargo install cargo-audit
cargo install cargo-deny
cargo install trunk
```

### Konfigurimi i IDE
```json
// VS Code settings.json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.allFeatures": true,
    "editor.formatOnSave": true,
    "rust-analyzer.cargo.target": "wasm32-unknown-unknown"
}
```

## Udhëzuesi i Stilit të Kodit

### Formatimi i Rust
- Përdor `rustfmt` me konfigurimin standard
- Gjatësia maksimale e rreshtit: 100 karaktere
- Përdor komentet e bllokut për dokumentim
- Komentet inline vetëm për logjikë komplekse

### Konventat e Emërtimit
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

### Organizimi i Kodit
```rust
// Struktura e moduleve
src/
├── main.rs
├── lib.rs
├── config/
├── api/
├── domain/
├── infrastructure/
└── utils/
```

## Workflow i Git

### Strategjia e Degëve
- `main`: Kodi i prodhimit
- `develop`: Dega e zhvillimit
- `feature/*`: Veçori të reja
- `bugfix/*`: Rregullime të gabimeve
- `release/*`: Përgatitja e versionit

### Konventat e Commit
```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Llojet:
- `feat`: Veçori e re
- `fix`: Rregullim i gabimit
- `docs`: Dokumentim
- `style`: Formatim
- `refactor`: Ristrukturim i kodit
- `test`: Teste
- `chore`: Mirëmbajtje

### Procesi i Pull Request
1. Krijo degë feature nga `develop`
2. Implemento dhe testo kodin
3. Krijo Pull Request
4. Kryej Code Review
5. Kalo me sukses pipeline-in CI/CD
6. Merge në `develop`

## Strategjitë e Testimit

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

## Pipeline CI/CD

### Workflow i GitHub Actions
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

### Pipeline i Deployment
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

### Trajtimi i Gabimeve
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

## Optimizimi i Performancës

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
    time = 300, // 5 minuta
    result = true,
    key = "String",
    convert = r#"{ format!("{}", user_id) }"#
)]
pub async fn get_user_profile(user_id: i64) -> Result<UserProfile, Error> {
    // Kërkesa në bazën e të dhënave
}
```

## Siguria

### Validimi i Input
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

### Parandalimi i SQL-Injection
```rust
// Përdor gjithmonë queries të parametrizuara
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

## Mirëmbajtja

### Përditësimet e Varësive
```bash
# Kontrollo për përditësime
cargo outdated

# Përditëso varësitë
cargo update

# Kontrollo për dobësi sigurie
cargo audit
```

### Monitorimi
```rust
use metrics::{counter, gauge};

pub fn track_request() {
    counter!("api.requests.total", 1);
    gauge!("api.requests.active", 1.0);
}
```

## Praktikat më të Mira

### Trajtimi i Gabimeve
- Përdor `thiserror` për gabimet e bibliotekës
- Përdor `anyhow` për gabimet e aplikacionit
- Implemento llojet e tua të gabimeve
- Logo gabimet me kontekst

### Async/Await
- Përdor `tokio` si Runtime
- Implemento `Stream` për dataset të mëdha
- Përdor Connection Pools
- Trajto Timeouts në mënyrë korrekte

### Testimi
- Shkruaj Unit Tests për logjikën
- Implemento Integration Tests
- Përdor Property-Based Testing
- Mock shërbimet e jashtme

### Dokumentimi
- Dokumento APIs publike
- Shto shembuj
- Mbaj CHANGELOG.md të përditësuar
- Shkruaj mesazhe commit kuptimplota 