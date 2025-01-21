# ACCI Framework API-Dokumentation

## Struktur

```
api/
├── en/                    # Englische Dokumentation
├── de/                    # Deutsche Dokumentation
├── sq/                    # Albanische Dokumentation
├── fr/                    # Französische Dokumentation
└── es/                    # Spanische Dokumentation
    ├── rest/             # REST API-Dokumentation
    │   ├── authentication/   # Authentifizierung & Autorisierung
    │   ├── endpoints/        # API-Endpunkte
    │   ├── errors/          # Fehlerbehandlung
    │   ├── examples/        # Nutzungsbeispiele
    │   └── schemas/         # Request/Response-Schemas
    └── graphql/          # GraphQL API-Dokumentation
        ├── authentication/   # Authentifizierung & Autorisierung
        ├── endpoints/        # Queries & Mutations
        ├── errors/          # Fehlerbehandlung
        ├── examples/        # Nutzungsbeispiele
        └── schemas/         # GraphQL-Schema
```

## Dokumentationsstandards

### REST API
- OpenAPI/Swagger Spezifikation 3.0
- Detaillierte Request/Response-Beispiele
- Standardisierte Fehlerantworten
- Rate-Limiting-Informationen
- Authentifizierungsmethoden
- Mandantenspezifische Überlegungen

### GraphQL API
- Vollständige Schema-Dokumentation
- Query/Mutation-Beispiele
- Fehlerbehandlungsmuster
- Authentifizierungsablauf
- Performance-Optimierungstipps
- Mandantenkontext-Verwaltung

### Gemeinsame Elemente
- Sicherheitsüberlegungen
- Rate-Limiting & Kontingente
- Paginierungsmuster
- Versionierungsinformationen
- Best Practices
- SDK-Beispiele

## URL-Parameter

### Pfadparameter
Verwenden Sie geschweifte Klammern für Pfadparameter:
```
/api/v1/tenants/{tenant_id}/users/{user_id}
```

NICHT Doppelpunkt-Präfix:
```
/api/v1/tenants/:tenant_id/users/:user_id  # Veraltet
```

## Fehlerantworten

### Standard-Fehlerformat
```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Menschenlesbare Nachricht",
    "details": {
      "field1": "Zusätzliche Information",
      "field2": "Weiterer Kontext"
    },
    "request_id": "req-123",
    "timestamp": "2024-01-21T10:00:00Z"
  }
}
```

### Häufige Fehlertypen
```json
// Authentifizierungsfehler
{
  "error": {
    "code": "AUTH_ERROR",
    "message": "Authentifizierung fehlgeschlagen",
    "details": {
      "reason": "invalid_token",
      "description": "Token ist abgelaufen"
    },
    "request_id": "req-123",
    "timestamp": "2024-01-21T10:00:00Z"
  }
}

// Validierungsfehler
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Ungültige Eingabedaten",
    "details": {
      "fields": {
        "email": "Ungültiges E-Mail-Format",
        "age": "Muss größer als 0 sein"
      }
    },
    "request_id": "req-124",
    "timestamp": "2024-01-21T10:00:00Z"
  }
}

// Rate-Limit-Fehler
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Zu viele Anfragen",
    "details": {
      "limit": 1000,
      "remaining": 0,
      "reset_at": "2024-01-21T10:00:00Z",
      "retry_after": 60
    },
    "request_id": "req-125",
    "timestamp": "2024-01-21T09:59:00Z"
  }
}

// Mandantenfehler
{
  "error": {
    "code": "TENANT_ERROR",
    "message": "Mandantenvalidierung fehlgeschlagen",
    "details": {
      "tenant_id": "tenant-123",
      "reason": "inactive_tenant",
      "status": "suspended"
    },
    "request_id": "req-126",
    "timestamp": "2024-01-21T10:00:00Z"
  }
}
```

## Antwort-Header

### Standard-Header
```http
Content-Type: application/json
X-Request-ID: req-123
X-Tenant-ID: tenant-123
```

### Rate-Limit-Header
```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 995
X-RateLimit-Reset: 1705689600
```

### Sicherheits-Header
```http
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Content-Security-Policy: default-src 'self'
Referrer-Policy: strict-origin-when-cross-origin
```

## Postman-Collection

Eine vollständige Postman-Collection ist im `postman`-Verzeichnis verfügbar, einschließlich:
- Umgebungsvariablen
- Anfrage-Beispiele
- Test-Skripte
- Pre-Request-Skripte
- Umgebungskonfigurationen

## Insomnia-Collection

Eine Insomnia-Collection ist im `insomnia`-Verzeichnis verfügbar, mit:
- Umgebungseinrichtung
- Anfrage-Vorlagen
- Antwort-Validierung
- GraphQL-Abfragen
- Authentifizierungsabläufe

## Wartung

Die API-Dokumentation wird in allen unterstützten Sprachen gepflegt:
- Englisch (en)
- Deutsch (de)
- Albanisch (sq)
- Französisch (fr)
- Spanisch (es)

### Übersetzungsprozess
1. Englische Dokumentation ist die Quelle der Wahrheit
2. Änderungen werden zuerst an der englischen Dokumentation vorgenommen
3. Übersetzungen werden innerhalb von 24 Stunden aktualisiert
4. Automatisierte Prüfungen stellen die Konsistenz über alle Sprachen sicher

### Versionskontrolle
- Dokumentationsversionen entsprechen API-Versionen
- Breaking Changes sind deutlich gekennzeichnet
- Deprecation-Hinweise enthalten Migrationsanleitungen
- Versionshistorie wird gepflegt 