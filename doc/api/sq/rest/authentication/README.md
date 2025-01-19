# Autentifikimi

## Përmbledhje

ACCI Framework përdor një sistem autentifikimi shumë-shtresor që kombinon JWT (JSON Web Tokens) me autentifikim specifik për tenant. Të gjitha kërkesat duhet të përfshijnë si token-at e autentifikimit ashtu edhe informacionin e tenant-it.

## Procesi i Autentifikimit

1. Përdoruesi ofron kredencialet dhe ID-në e tenant-it
2. Sistemi validon kredencialet kundrejt realm-it specifik të Keycloak për tenant-in
3. Pas autentifikimit të suksesshëm, sistemi lëshon token-at JWT
4. Të gjitha kërkesat e mëpasshme duhet të përfshijnë:
   - Token Bearer në header-in Authorization
   - ID-në e tenant-it në header-in X-Tenant-ID

## Header-at

```http
Authorization: Bearer <jwt_token>
X-Tenant-ID: <tenant_uuid>
```

## Endpoints

### Hyrja

```http
POST /auth/login
Content-Type: application/json

{
  "email": "perdorues@shembull.com",
  "password": "fjalekalim_i_sigurt",
  "tenant_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

Përgjigja:
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9...",
  "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9...",
  "token_type": "bearer",
  "expires_in": 3600,
  "scope": "api:access"
}
```

### Rifreskimi i Token-it

```http
POST /auth/refresh
Content-Type: application/json

{
  "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9..."
}
```

## Trajtimi i Gabimeve

### Kodet e Zakonshme të Gabimeve

- `401 Unauthorized`: Kredenciale të pavlefshme
- `403 Forbidden`: Kredenciale të vlefshme por pa leje të mjaftueshme
- `404 Not Found`: Tenant-i nuk u gjet
- `429 Too Many Requests`: Limiti i kërkesave u tejkalua

Shembull i përgjigjes së gabimit:
```json
{
  "code": "AUTH_ERROR",
  "message": "Kredenciale të pavlefshme",
  "details": {
    "reason": "password_invalid"
  },
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

## Konsiderata të Sigurisë

### Ruajtja e Token-ave
- Ruaj token-at në mënyrë të sigurt
- Asnjëherë mos i ruaj në localStorage
- Përdor cookies httpOnly kur është e mundur
- Fshi token-at gjatë daljes

### CORS
- Zbatohen politika strikte CORS
- Whitelist vetëm domain-et e besuara
- Kredencialet duhet të përfshihen në kërkesa

### Kufizimi i Kërkesave
- Përpjekjet e hyrjes janë të kufizuara
- Aplikohen limite specifike për tenant
- Backoff eksponencial në përpjekjet e dështuara

## Autentifikimi Multi-Faktor

Kur MFA është e aktivizuar:

1. Hyrja fillestare kthen sfidën MFA
2. Klienti duhet të përfundojë sfidën MFA
3. Pas MFA-së së suksesshme, lëshohen token-at

Shembull i përgjigjes MFA:
```json
{
  "mfa_required": true,
  "mfa_type": "totp",
  "challenge_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

## Praktikat më të Mira

1. Implemento logjikën e duhur të rifreskimit të token-ave
2. Trajto skadimin e token-ave në mënyrë elegante
3. Implemento trajtimin e duhur të gabimeve
4. Përdor lidhje të sigurt (HTTPS)
5. Ndiq parimet e izolimit të tenant-it
6. Implemento procedurat e duhura të daljes
7. Monitoro metrikat e autentifikimit 