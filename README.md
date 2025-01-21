# ACCI Framework

[English](#english) | [Deutsch](#deutsch) | [Shqip](#shqip)

---

[![Rust Tests](https://github.com/Broccode/acci_base/actions/workflows/rust-tests.yml/badge.svg)](https://github.com/Broccode/acci_base/actions/workflows/rust-tests.yml)

## English

### Overview
ACCI Framework is a modern, multi-tenant capable full-stack application framework built with Rust. It provides pre-built user management, authentication, and enterprise-grade deployment capabilities.

### Key Features
- Multi-tenancy support with complete tenant isolation
- Dual API exposure (REST & GraphQL)
- Built-in internationalization (EN, DE, SQ, FR, ES)
- Enterprise-grade security features
- Comprehensive monitoring & observability
- Docker/Kubernetes ready
- Advanced health check system
- Distributed tracing with OpenTelemetry
- Structured logging with correlation IDs
- RED metrics (Rate, Errors, Duration)
- Circuit breaker implementation
- Quota management system
- CDN integration for static assets

### Technical Stack
- Backend: Rust (Axum, async-graphql)
- Frontend: Leptos
- Database: PostgreSQL
- Event Store: EventStoreDB
- Metrics: InfluxDB
- Cache: Redis
- Message Broker: RabbitMQ (with dead-letter support)
- Identity Provider: Keycloak
- Service Mesh: Ready
- Container Registry: Integrated

### Security Features
- Multi-factor authentication
- Role-based access control (RBAC)
- Tenant isolation
- Regular security audits
- Dependency vulnerability scanning
- SBOM generation
- Secret management
- Constant-time comparisons for sensitive data

### Development Features
- Comprehensive CI/CD pipeline
- Pre-commit hooks
- Automated testing
- Code quality checks
- Performance benchmarking
- Documentation generation
- Multi-language API documentation

### Getting Started
```bash
# Clone repository
git clone https://github.com/your-org/acci-framework

# Start development environment
docker compose up -d

# Run tests
cargo test
```

---

## Deutsch

### Überblick
ACCI Framework ist ein modernes Full-Stack-Anwendungsframework mit Multi-Tenant-Fähigkeit, entwickelt in Rust. Es bietet vorgefertigte Benutzerverwaltung, Authentifizierung und Enterprise-Grade-Deployment-Funktionen.

### Hauptfunktionen
- Multi-Tenant-Unterstützung mit vollständiger Tenant-Isolation
- Duale API-Bereitstellung (REST & GraphQL)
- Integrierte Internationalisierung (EN, DE, SQ, FR, ES)
- Enterprise-Grade-Sicherheitsfunktionen
- Umfassendes Monitoring & Observability
- Docker/Kubernetes-ready
- Fortschrittliches Healthcheck-System
- Verteiltes Tracing mit OpenTelemetry
- Strukturiertes Logging mit Korrelations-IDs
- RED-Metriken (Rate, Errors, Duration)
- Circuit-Breaker-Implementierung
- Quota-Management-System
- CDN-Integration für statische Assets

### Technologie-Stack
- Backend: Rust (Axum, async-graphql)
- Frontend: Leptos
- Datenbank: PostgreSQL
- Event Store: EventStoreDB
- Metriken: InfluxDB
- Cache: Redis
- Message Broker: RabbitMQ (mit Dead-Letter-Unterstützung)
- Identity Provider: Keycloak
- Service Mesh: Vorbereitet
- Container Registry: Integriert

### Sicherheitsfunktionen
- Multi-Faktor-Authentifizierung
- Rollenbasierte Zugriffskontrolle (RBAC)
- Tenant-Isolation
- Regelmäßige Sicherheitsaudits
- Dependency-Schwachstellenanalyse
- SBOM-Generierung
- Geheimnisverwaltung
- Zeitkonstante Vergleiche für sensible Daten

### Entwicklungsfunktionen
- Umfassende CI/CD-Pipeline
- Pre-commit Hooks
- Automatisierte Tests
- Code-Qualitätsprüfungen
- Performance-Benchmarking
- Dokumentationsgenerierung
- Mehrsprachige API-Dokumentation

### Erste Schritte
```bash
# Repository klonen
git clone https://github.com/your-org/acci-framework

# Entwicklungsumgebung starten
docker compose up -d

# Tests ausführen
cargo test
```

---

## Shqip

### Përmbledhje
ACCI Framework është një framework modern aplikacionesh full-stack me aftësi multi-tenant, i ndërtuar me Rust. Ai ofron menaxhim të parakonfiguruar të përdoruesve, autentifikim dhe aftësi për deployment në nivel enterprise.

### Karakteristikat Kryesore
- Mbështetje për multi-tenant me izolim të plotë të tenant-ëve
- Ekspozim i dyfishtë i API-ve (REST & GraphQL)
- Internacionalizim i integruar (EN, DE, SQ, FR, ES)
- Karakteristika sigurie të nivelit enterprise
- Monitorim dhe vëzhgim gjithëpërfshirës
- Gati për Docker/Kubernetes
- Sistem i avancuar i kontrollit të shëndetit
- Gjurmim i shpërndarë me OpenTelemetry
- Logging i strukturuar me ID korrelacioni
- Metrika RED (Shkalla, Gabimet, Kohëzgjatja)
- Implementim i Circuit Breaker
- Sistem i menaxhimit të kuotave
- Integrimi CDN për asetet statike

### Stack-u Teknik
- Backend: Rust (Axum, async-graphql)
- Frontend: Leptos
- Databaza: PostgreSQL
- Event Store: EventStoreDB
- Metrikat: InfluxDB
- Cache: Redis
- Message Broker: RabbitMQ (me mbështetje dead-letter)
- Identity Provider: Keycloak
- Service Mesh: Gati
- Container Registry: I integruar

### Karakteristikat e Sigurisë
- Autentifikim multi-faktor
- Kontroll i aksesit bazuar në role (RBAC)
- Izolim i tenant-ëve
- Auditime të rregullta të sigurisë
- Skanim i dobësive të varësive
- Gjenerim SBOM
- Menaxhim i sekreteve
- Krahasime me kohë konstante për të dhëna sensitive

### Karakteristikat e Zhvillimit
- Pipeline gjithëpërfshirës CI/CD
- Hooks para-commit
- Testim i automatizuar
- Kontrolle të cilësisë së kodit
- Benchmark i performancës
- Gjenerim i dokumentacionit
- Dokumentim API shumëgjuhësh

### Fillimi i Përdorimit
```bash
# Klono repository-n
git clone https://github.com/your-org/acci-framework

# Fillo mjedisin e zhvillimit
docker compose up -d

# Ekzekuto testet
cargo test
```

---

## License
Apache 2.0 License - see [LICENSE](LICENSE) for details
