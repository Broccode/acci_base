# Guide du Développeur

## Environnement de Développement

### Prérequis
- Rust 1.75 ou supérieur
- Docker 24.0 ou supérieur
- Git 2.40 ou supérieur
- VS Code ou IntelliJ avec plugin Rust
- Trunk (pour le développement frontend avec Leptos)

### Installation
```bash
# Installation de Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup component add rustfmt clippy
rustup target add wasm32-unknown-unknown

# Outils de Développement
cargo install cargo-watch
cargo install cargo-audit
cargo install cargo-deny
cargo install trunk
```

### Configuration IDE
```json
// VS Code settings.json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.allFeatures": true,
    "editor.formatOnSave": true,
    "rust-analyzer.cargo.target": "wasm32-unknown-unknown"
}
```

## Guide de Style de Code

### Formatage Rust
- Utiliser `rustfmt` avec la configuration standard
- Longueur maximale de ligne : 100 caractères
- Utiliser les commentaires de bloc pour la documentation
- Commentaires en ligne uniquement pour la logique complexe

### Conventions de Nommage
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

### Organisation du Code
```rust
// Structure des modules
src/
├── main.rs
├── lib.rs
├── config/
├── api/
├── domain/
├── infrastructure/
└── utils/
```

## Workflow Git

### Stratégie de Branches
- `main`: Code de production
- `develop`: Branche de développement
- `feature/*`: Nouvelles fonctionnalités
- `bugfix/*`: Corrections de bugs
- `release/*`: Préparation de version

### Conventions de Commit
```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types:
- `feat`: Nouvelle fonctionnalité
- `fix`: Correction de bug
- `docs`: Documentation
- `style`: Formatage
- `refactor`: Restructuration du code
- `test`: Tests
- `chore`: Maintenance

### Processus de Pull Request
1. Créer une branche feature depuis `develop`
2. Implémenter et tester le code
3. Créer une Pull Request
4. Effectuer la Code Review
5. Passer avec succès le pipeline CI/CD
6. Merger dans `develop`

## Stratégies de Test

### Tests Unitaires
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

### Tests d'Intégration
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

### Tests Basés sur les Propriétés
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

### Workflow GitHub Actions
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

### Pipeline de Déploiement
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

## Débogage

### Journalisation
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

### Gestion des Erreurs
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

## Optimisation des Performances

### Profilage
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

### Mise en Cache
```rust
use cached::proc_macro::cached;

#[cached(
    time = 300, // 5 minutes
    result = true,
    key = "String",
    convert = r#"{ format!("{}", user_id) }"#
)]
pub async fn get_user_profile(user_id: i64) -> Result<UserProfile, Error> {
    // Requête base de données
}
```

## Sécurité

### Validation des Entrées
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

### Prévention SQL-Injection
```rust
// Toujours utiliser des requêtes paramétrées
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

## Maintenance

### Mises à Jour des Dépendances
```bash
# Vérifier les mises à jour
cargo outdated

# Mettre à jour les dépendances
cargo update

# Vérifier les vulnérabilités
cargo audit
```

### Surveillance
```rust
use metrics::{counter, gauge};

pub fn track_request() {
    counter!("api.requests.total", 1);
    gauge!("api.requests.active", 1.0);
}
```

## Meilleures Pratiques

### Gestion des Erreurs
- Utiliser `thiserror` pour les erreurs de bibliothèque
- Utiliser `anyhow` pour les erreurs d'application
- Implémenter ses propres types d'erreur
- Logger les erreurs avec contexte

### Async/Await
- Utiliser `tokio` comme Runtime
- Implémenter `Stream` pour les grands ensembles de données
- Utiliser les Connection Pools
- Gérer correctement les Timeouts

### Tests
- Écrire des tests unitaires pour la logique
- Implémenter des tests d'intégration
- Utiliser les tests basés sur les propriétés
- Mocker les services externes

### Documentation
- Documenter les APIs publiques
- Ajouter des exemples
- Maintenir CHANGELOG.md à jour
- Écrire des messages de commit significatifs 