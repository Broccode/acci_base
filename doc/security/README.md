# Security Documentation

## Authentication Mechanisms

### Keycloak Integration
- OpenID Connect/OAuth2 based authentication
- JWT tokens with RS256 signing
- Automatic key rotation and caching
- Multi-tenant realm support
- Role-based access control
- Protocol mapper for tenant information

### Token Structure
```json
{
  "sub": "user-id",
  "tid": "tenant-id",
  "realm_access": {
    "roles": ["user", "admin"]
  },
  "resource_access": {
    "acci-backend": {
      "roles": ["tenant_admin"]
    }
  },
  "exp": 1705689600,
  "iat": 1705688700,
  "iss": "https://auth.acci-framework.com/realms/acci",
  "tenant_context": {
    "id": "tenant-id",
    "status": "active",
    "features": ["feature1", "feature2"]
  }
}
```

### Multi-Factor Authentication (MFA)
- TOTP-based second factor
- WebAuthn/FIDO2 support
- Backup codes for account recovery
- Enforced for admin accounts
- Optional for regular users
- QR code provisioning

### Tenant Validation
- Mandatory tenant context in protected routes
- Active tenant verification
- Tenant-specific role validation
- Domain-based tenant resolution
- Tenant isolation enforcement
- Cross-tenant access prevention

## Error Handling

### Authentication Errors
```json
{
  "error": {
    "code": "AUTH_ERROR",
    "message": "Authentication failed",
    "details": {
      "reason": "invalid_token",
      "description": "Token has expired"
    },
    "request_id": "req-123",
    "timestamp": "2024-01-21T10:00:00Z"
  }
}
```

### Authorization Errors
```json
{
  "error": {
    "code": "FORBIDDEN",
    "message": "Access denied",
    "details": {
      "required_roles": ["admin"],
      "current_roles": ["user"],
      "resource": "users",
      "action": "create"
    },
    "request_id": "req-124",
    "timestamp": "2024-01-21T10:00:00Z"
  }
}
```

### Tenant Errors
```json
{
  "error": {
    "code": "TENANT_ERROR",
    "message": "Tenant validation failed",
    "details": {
      "tenant_id": "tenant-123",
      "reason": "inactive_tenant",
      "status": "suspended"
    },
    "request_id": "req-125",
    "timestamp": "2024-01-21T10:00:00Z"
  }
}
```

## Rate Limiting & Throttling

### Global Rate Limits
- 1000 requests per minute per IP
- 5000 requests per hour per IP
- Burst allowance of 50 requests

### Authentication Endpoints
- 5 failed login attempts per minute
- 20 failed login attempts per hour
- 15-minute lockout after exceeding limits

### API Endpoints
- 100 requests per minute per token
- 1000 requests per hour per token
- Tenant-specific limits
- Separate limits for read and write operations

### Rate Limit Headers
```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 995
X-RateLimit-Reset: 1705689600
```

### Rate Limit Error Response
```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many requests",
    "details": {
      "limit": 1000,
      "remaining": 0,
      "reset_at": "2024-01-21T10:00:00Z",
      "retry_after": 60
    },
    "request_id": "req-126",
    "timestamp": "2024-01-21T09:59:00Z"
  }
}
```

## Security Headers

```http
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Content-Security-Policy: default-src 'self'
Referrer-Policy: strict-origin-when-cross-origin
Permissions-Policy: geolocation=(), microphone=()
```

## Audit Logging

### Authentication Events
```json
{
  "event_type": "authentication",
  "action": "login_success",
  "timestamp": "2024-01-21T10:00:00Z",
  "user_id": "user-123",
  "tenant_id": "tenant-123",
  "ip_address": "192.168.1.1",
  "user_agent": "Mozilla/5.0...",
  "auth_method": "password",
  "request_id": "req-127"
}
```

### Authorization Events
```json
{
  "event_type": "authorization",
  "action": "access_denied",
  "timestamp": "2024-01-21T10:00:00Z",
  "user_id": "user-123",
  "tenant_id": "tenant-123",
  "resource": "users",
  "requested_action": "create",
  "required_roles": ["admin"],
  "current_roles": ["user"],
  "request_id": "req-128"
}
```

## Security Best Practices

### Password Policy
- Minimum length: 12 characters
- Must contain: uppercase, lowercase, numbers, special characters
- Maximum age: 90 days
- Password history: last 12 passwords
- Bcrypt hashing with work factor 12

### Session Management
- 15-minute access token lifetime
- 7-day refresh token lifetime
- Sliding session extension
- Concurrent session limits
- Device tracking and management

### API Security
- TLS 1.3 required
- Certificate pinning
- API key rotation
- Request signing
- Payload encryption for sensitive data

### Tenant Isolation
- Database row-level security
- Cache key prefixing
- Event store stream isolation
- File storage separation
- Network isolation

## CORS Configuration

### Allowed Origins
- Production domain: https://app.acci-framework.com
- Development domain: https://dev.acci-framework.com
- Local development: http://localhost:3000

### CORS Headers
```http
Access-Control-Allow-Origin: https://app.acci-framework.com
Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS
Access-Control-Allow-Headers: Content-Type, Authorization, X-Tenant-ID
Access-Control-Max-Age: 86400
```

## Token Security

### Storage Guidelines
- Store access tokens in memory only
- Refresh tokens in HttpOnly secure cookies
- Never store tokens in localStorage
- Clear tokens on logout/session end

### Token Rotation
- New refresh token issued with each use
- Old refresh tokens invalidated
- Access tokens not reusable after refresh
- Maximum refresh chain of 7 days

### Token Validation
- Signature verification
- Expiration check
- Tenant ID validation
- Role-based access control
- Active session verification

## API Security

### Request Validation
- Input sanitization using Zod schemas
- Content-Type enforcement
- Request size limits
- UTF-8 encoding validation
- JSON schema validation

### Response Security
- No sensitive data in responses
- Consistent error formats
- Rate limit information
- Request IDs for tracking
- Security headers in all responses

### TLS Configuration
- TLS 1.3 required
- Strong cipher suites only
- Perfect forward secrecy
- OCSP stapling enabled
- Certificate transparency

## Audit & Logging

### Security Events
- Authentication attempts
- Token issuance/revocation
- Permission changes
- Security setting updates
- Rate limit violations

### Audit Log Format
```json
{
  "timestamp": "2024-01-19T20:00:00Z",
  "event": "authentication_attempt",
  "status": "success",
  "user_id": "user-123",
  "tenant_id": "tenant-456",
  "ip_address": "192.168.1.1",
  "user_agent": "Mozilla/5.0...",
  "request_id": "req-789"
}
```

### Log Security
- No sensitive data in logs
- Log encryption at rest
- Secure log transmission
- Log retention policies
- Access control for logs

## Security Best Practices

### Password Requirements
- Minimum 12 characters
- Uppercase and lowercase letters
- Numbers and special characters
- No common passwords
- Regular password changes

### Session Management
- Automatic session timeout
- Single session per user
- Device fingerprinting
- Suspicious activity detection
- Session invalidation on security events

### Error Handling
- No sensitive data in errors
- Generic error messages
- Detailed internal logging
- Error aggregation
- Security event correlation

### Data Protection
- Data encryption at rest
- Secure key management
- Regular key rotation
- Data classification
- Access control policies

## Incident Response

### Security Alerts
- Failed authentication spikes
- Unusual traffic patterns
- Token misuse attempts
- Configuration changes
- System health issues

### Response Procedures
1. Incident detection and classification
2. Immediate containment measures
3. Investigation and analysis
4. System restoration
5. Post-incident review

### Contact Information
- Security team: security@acci-framework.com
- Emergency hotline: +1-555-0123
- Bug bounty program: https://bugbounty.acci-framework.com

## Compliance

### Standards
- OWASP Top 10 compliance
- GDPR requirements
- SOC 2 controls
- ISO 27001 alignment
- PCI DSS compliance where applicable

### Regular Assessments
- Quarterly security audits
- Penetration testing
- Vulnerability scanning
- Code security review
- Compliance monitoring 