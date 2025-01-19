# Authentication

## Overview

The ACCI Framework uses a multi-layered authentication system that combines JWT (JSON Web Tokens) with tenant-specific authentication. All requests must include both authentication tokens and tenant information.

## Authentication Flow

1. User provides credentials and tenant ID
2. System validates credentials against tenant-specific Keycloak realm
3. Upon successful authentication, system issues JWT tokens
4. All subsequent requests must include:
   - Bearer token in Authorization header
   - Tenant ID in X-Tenant-ID header

## Headers

```http
Authorization: Bearer <jwt_token>
X-Tenant-ID: <tenant_uuid>
```

## Endpoints

### Login

```http
POST /auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "secure_password",
  "tenant_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

Response:
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9...",
  "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9...",
  "token_type": "bearer",
  "expires_in": 3600,
  "scope": "api:access"
}
```

### Token Refresh

```http
POST /auth/refresh
Content-Type: application/json

{
  "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9..."
}
```

## Error Handling

### Common Error Codes

- `401 Unauthorized`: Invalid credentials
- `403 Forbidden`: Valid credentials but insufficient permissions
- `404 Not Found`: Tenant not found
- `429 Too Many Requests`: Rate limit exceeded

Example error response:
```json
{
  "code": "AUTH_ERROR",
  "message": "Invalid credentials",
  "details": {
    "reason": "password_invalid"
  },
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

## Security Considerations

### Token Storage
- Store tokens securely
- Never store in localStorage
- Use httpOnly cookies when possible
- Clear tokens on logout

### CORS
- Strict CORS policies are enforced
- Whitelist only trusted domains
- Credentials must be included in requests

### Rate Limiting
- Login attempts are rate-limited
- Tenant-specific rate limits apply
- Exponential backoff on failed attempts

## Multi-Factor Authentication

When MFA is enabled:

1. Initial login returns MFA challenge
2. Client must complete MFA challenge
3. Upon successful MFA, tokens are issued

Example MFA response:
```json
{
  "mfa_required": true,
  "mfa_type": "totp",
  "challenge_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

## Best Practices

1. Implement proper token refresh logic
2. Handle token expiration gracefully
3. Implement proper error handling
4. Use secure connection (HTTPS)
5. Follow tenant isolation principles
6. Implement proper logout procedures
7. Monitor authentication metrics 