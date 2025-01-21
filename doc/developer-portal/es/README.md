# Guía del Desarrollador

## Entorno de Desarrollo

### Requisitos
- Rust 1.75 o superior
- Docker 24.0 o superior
- Git 2.40 o superior
- VS Code o IntelliJ con plugin de Rust
- Trunk (para desarrollo frontend con Leptos)

### Instalación
```bash
# Instalación de Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup component add rustfmt clippy
rustup target add wasm32-unknown-unknown

# Herramientas de Desarrollo
cargo install cargo-watch
cargo install cargo-audit
cargo install cargo-deny
cargo install trunk
```

### Configuración del IDE
```json
// VS Code settings.json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.allFeatures": true,
    "editor.formatOnSave": true,
    "rust-analyzer.cargo.target": "wasm32-unknown-unknown"
}
```

## Guía de Estilo de Código

### Formato de Rust
- Usar `rustfmt` con la configuración estándar
- Longitud máxima de línea: 100 caracteres
- Usar comentarios de bloque para documentación
- Comentarios en línea solo para lógica compleja

### Convenciones de Nombres
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

### Organización del Código
```rust
// Estructura de módulos
src/
├── main.rs
├── lib.rs
├── config/
├── api/
├── domain/
├── infrastructure/
└── utils/
```

## Flujo de Git

### Estrategia de Ramas
- `main`: Código de producción
- `develop`: Rama de desarrollo
- `feature/*`: Nuevas características
- `bugfix/*`: Correcciones de errores
- `release/*`: Preparación de versión

### Convenciones de Commit
```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Tipos:
- `feat`: Nueva característica
- `fix`: Corrección de error
- `docs`: Documentación
- `style`: Formato
- `refactor`: Reestructuración de código
- `test`: Pruebas
- `chore`: Mantenimiento

### Proceso de Pull Request
1. Crear rama feature desde `develop`
2. Implementar y probar código
3. Crear Pull Request
4. Realizar Code Review
5. Pasar pipeline CI/CD exitosamente
6. Merge a `develop`

## Estrategias de Pruebas

### Pruebas Unitarias
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

### Pruebas de Integración
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

### Pruebas Basadas en Propiedades
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

### Workflow de GitHub Actions
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

### Pipeline de Despliegue
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

## Depuración

### Registro
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

### Manejo de Errores
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

## Optimización de Rendimiento

### Perfilado
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

### Caché
```rust
use cached::proc_macro::cached;

#[cached(
    time = 300, // 5 minutos
    result = true,
    key = "String",
    convert = r#"{ format!("{}", user_id) }"#
)]
pub async fn get_user_profile(user_id: i64) -> Result<UserProfile, Error> {
    // Consulta a base de datos
}
```

## Seguridad

### Validación de Entrada
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

### Prevención de SQL-Injection
```rust
// Siempre usar consultas parametrizadas
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

## Mantenimiento

### Actualizaciones de Dependencias
```bash
# Verificar actualizaciones
cargo outdated

# Actualizar dependencias
cargo update

# Verificar vulnerabilidades
cargo audit
```

### Monitoreo
```rust
use metrics::{counter, gauge};

pub fn track_request() {
    counter!("api.requests.total", 1);
    gauge!("api.requests.active", 1.0);
}
```

## Mejores Prácticas

### Manejo de Errores
- Usar `thiserror` para errores de biblioteca
- Usar `anyhow` para errores de aplicación
- Implementar tipos de error propios
- Registrar errores con contexto

### Async/Await
- Usar `tokio` como Runtime
- Implementar `Stream` para grandes conjuntos de datos
- Usar Connection Pools
- Manejar Timeouts correctamente

### Pruebas
- Escribir pruebas unitarias para lógica
- Implementar pruebas de integración
- Usar pruebas basadas en propiedades
- Mockear servicios externos

### Documentación
- Documentar APIs públicas
- Agregar ejemplos
- Mantener CHANGELOG.md actualizado
- Escribir mensajes de commit significativos 