# Authentifizierung

## Überblick

Das ACCI Framework verwendet ein mehrschichtiges Authentifizierungssystem, das JWT (JSON Web Tokens) mit mandantenspezifischer Authentifizierung kombiniert. Alle Anfragen müssen sowohl Authentifizierungs-Tokens als auch Mandanteninformationen enthalten.

## Authentifizierungsablauf

1. Benutzer stellt Anmeldedaten und Mandanten-ID bereit
2. System validiert Anmeldedaten gegen mandantenspezifischen Keycloak-Realm
3. Nach erfolgreicher Authentifizierung stellt das System JWT-Tokens aus
4. Alle nachfolgenden Anfragen müssen enthalten:
   - Bearer-Token im Authorization-Header
   - Mandanten-ID im X-Tenant-ID-Header

## Header

```http
Authorization: Bearer <jwt_token>
X-Tenant-ID: <tenant_uuid>
```

## Endpunkte

### Anmeldung

```http
POST /auth/login
Content-Type: application/json

{
  "email": "benutzer@beispiel.de",
  "password": "sicheres_passwort",
  "tenant_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

Antwort:
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9...",
  "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9...",
  "token_type": "bearer",
  "expires_in": 3600,
  "scope": "api:access"
}
```

### Token-Aktualisierung

```http
POST /auth/refresh
Content-Type: application/json

{
  "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9..."
}
```

## Fehlerbehandlung

### Häufige Fehlercodes

- `401 Unauthorized`: Ungültige Anmeldedaten
- `403 Forbidden`: Gültige Anmeldedaten, aber unzureichende Berechtigungen
- `404 Not Found`: Mandant nicht gefunden
- `429 Too Many Requests`: Anfragelimit überschritten

Beispiel einer Fehlerantwort:
```json
{
  "code": "AUTH_ERROR",
  "message": "Ungültige Anmeldedaten",
  "details": {
    "reason": "password_invalid"
  },
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

## Sicherheitsaspekte

### Token-Speicherung
- Tokens sicher speichern
- Niemals in localStorage speichern
- Wenn möglich httpOnly-Cookies verwenden
- Tokens bei Abmeldung löschen

### CORS
- Strikte CORS-Richtlinien werden durchgesetzt
- Nur vertrauenswürdige Domains whitelisten
- Credentials müssen in Anfragen enthalten sein

### Anfragelimitierung
- Anmeldeversuche sind limitiert
- Mandantenspezifische Limits gelten
- Exponentielles Backoff bei fehlgeschlagenen Versuchen

## Mehrfaktor-Authentifizierung

Wenn MFA aktiviert ist:

1. Initiale Anmeldung liefert MFA-Challenge
2. Client muss MFA-Challenge abschließen
3. Nach erfolgreicher MFA werden Tokens ausgestellt

Beispiel einer MFA-Antwort:
```json
{
  "mfa_required": true,
  "mfa_type": "totp",
  "challenge_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

## Best Practices

1. Token-Aktualisierungslogik korrekt implementieren
2. Token-Ablauf angemessen behandeln
3. Fehlerbehandlung implementieren
4. Sichere Verbindung (HTTPS) verwenden
5. Mandantenisolationsprinzipien befolgen
6. Korrekte Abmeldeprozeduren implementieren
7. Authentifizierungsmetriken überwachen 