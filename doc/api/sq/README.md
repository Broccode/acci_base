# Dokumentimi i API-së së ACCI Framework

## Struktura

```
api/
├── en/                    # Dokumentimi në anglisht
├── de/                    # Dokumentimi në gjermanisht
├── sq/                    # Dokumentimi në shqip
├── fr/                    # Dokumentimi në frëngjisht
└── es/                    # Dokumentimi në spanjisht
    ├── rest/             # Dokumentimi i REST API
    │   ├── authentication/   # Autentifikimi & Autorizimi
    │   ├── endpoints/        # Pikat fundore të API
    │   ├── errors/          # Trajtimi i gabimeve
    │   ├── examples/        # Shembuj përdorimi
    │   └── schemas/         # Skemat e kërkesave/përgjigjeve
    └── graphql/          # Dokumentimi i GraphQL API
        ├── authentication/   # Autentifikimi & Autorizimi
        ├── endpoints/        # Queries & Mutations
        ├── errors/          # Trajtimi i gabimeve
        ├── examples/        # Shembuj përdorimi
        └── schemas/         # Skema GraphQL
```

## Standardet e Dokumentimit

### REST API
- Specifikimi OpenAPI/Swagger 3.0
- Shembuj të detajuar të kërkesave/përgjigjeve
- Përgjigje të standardizuara të gabimeve
- Informacion për kufizimin e kërkesave
- Metodat e autentifikimit
- Konsiderata specifike për qiramarrësit

### GraphQL API
- Dokumentim i plotë i skemës
- Shembuj të Query/Mutation
- Modele të trajtimit të gabimeve
- Rrjedha e autentifikimit
- Këshilla për optimizimin e performancës
- Menaxhimi i kontekstit të qiramarrësit

### Elementet e Përbashkëta
- Konsiderata të sigurisë
- Kufizimet & kuotat
- Modelet e faqosjes
- Informacioni i versionimit
- Praktikat më të mira
- Shembuj SDK

## Parametrat URL

### Parametrat e Shtegut
Përdorni kllapa gjarpërore për parametrat e shtegut:
```
/api/v1/tenants/{tenant_id}/users/{user_id}
```

JO prefiksin me dy pika:
```
/api/v1/tenants/:tenant_id/users/:user_id  # I vjetruar
```

## Përgjigjet e Gabimeve

### Formati Standard i Gabimit
```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Mesazh i lexueshëm nga njeriu",
    "details": {
      "field1": "Informacion shtesë",
      "field2": "Kontekst i mëtejshëm"
    },
    "request_id": "req-123",
    "timestamp": "2024-01-21T10:00:00Z"
  }
}
```

### Llojet e Zakonshme të Gabimeve
```json
// Gabim Autentifikimi
{
  "error": {
    "code": "AUTH_ERROR",
    "message": "Autentifikimi dështoi",
    "details": {
      "reason": "invalid_token",
      "description": "Tokeni ka skaduar"
    },
    "request_id": "req-123",
    "timestamp": "2024-01-21T10:00:00Z"
  }
}

// Gabim Validimi
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Të dhëna të pavlefshme",
    "details": {
      "fields": {
        "email": "Format i pavlefshëm email-i",
        "age": "Duhet të jetë më i madh se 0"
      }
    },
    "request_id": "req-124",
    "timestamp": "2024-01-21T10:00:00Z"
  }
}

// Gabim i Kufizimit të Kërkesave
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Shumë kërkesa",
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

// Gabim Qiramarrësi
{
  "error": {
    "code": "TENANT_ERROR",
    "message": "Validimi i qiramarrësit dështoi",
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

## Headers të Përgjigjes

### Headers Standard
```http
Content-Type: application/json
X-Request-ID: req-123
X-Tenant-ID: tenant-123
```

### Headers të Kufizimit të Kërkesave
```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 995
X-RateLimit-Reset: 1705689600
```

### Headers të Sigurisë
```http
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Content-Security-Policy: default-src 'self'
Referrer-Policy: strict-origin-when-cross-origin
```

## Koleksioni Postman

Një koleksion i plotë Postman është i disponueshëm në direktorinë `postman`, përfshirë:
- Variablat e mjedisit
- Shembuj kërkesash
- Skripte testimi
- Skripte para-kërkesash
- Konfigurime mjedisi

## Koleksioni Insomnia

Një koleksion Insomnia është i disponueshëm në direktorinë `insomnia`, që përfshin:
- Konfigurimin e mjedisit
- Modele kërkesash
- Validimin e përgjigjeve
- Queries GraphQL
- Rrjedhat e autentifikimit

## Mirëmbajtja

Dokumentimi i API-së mirëmbahet në të gjitha gjuhët e mbështetura:
- Anglisht (en)
- Gjermanisht (de)
- Shqip (sq)
- Frëngjisht (fr)
- Spanjisht (es)

### Procesi i Përkthimit
1. Dokumentimi në anglisht është burimi i së vërtetës
2. Ndryshimet bëhen fillimisht në dokumentimin në anglisht
3. Përkthimet përditësohen brenda 24 orëve
4. Kontrollet e automatizuara sigurojnë konsistencën në të gjitha gjuhët

### Kontrolli i Versioneve
- Versionet e dokumentimit përputhen me versionet e API-së
- Ndryshimet thelbësore janë qartësisht të shënuara
- Njoftimet e vjetrimit përfshijnë udhëzime migrimi
- Historiku i versioneve mirëmbahet 