# ACCI Framework API Documentation

## Structure

```
api/
├── en/                    # English documentation
├── de/                    # German documentation
├── sq/                    # Albanian documentation
├── fr/                    # French documentation
└── es/                    # Spanish documentation
    ├── rest/             # REST API documentation
    │   ├── authentication/   # Authentication & Authorization
    │   ├── endpoints/        # API Endpoints
    │   ├── errors/          # Error Handling
    │   ├── examples/        # Usage Examples
    │   └── schemas/         # Request/Response Schemas
    └── graphql/          # GraphQL API documentation
        ├── authentication/   # Authentication & Authorization
        ├── endpoints/        # Queries & Mutations
        ├── errors/          # Error Handling
        ├── examples/        # Usage Examples
        └── schemas/         # GraphQL Schema
```

## Documentation Standards

### REST API
- OpenAPI/Swagger Specification 3.0
- Detailed request/response examples
- Standardized error responses
- Rate limiting information
- Authentication methods
- Tenant-specific considerations

### GraphQL API
- Full schema documentation
- Query/Mutation examples
- Error handling patterns
- Authentication flow
- Performance optimization tips
- Tenant context management

### Common Elements
- Security considerations
- Rate limiting & quotas
- Pagination patterns
- Versioning information
- Best practices
- SDK examples

## URL Parameters

### Path Parameters
Use curly braces for path parameters:
```
/api/v1/tenants/{tenant_id}/users/{user_id}
```

NOT colon prefix:
```
/api/v1/tenants/:tenant_id/users/:user_id  # Deprecated
```

## Error Responses

### Standard Error Format
```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable message",
    "details": {
      "field1": "Additional information",
      "field2": "More context"
    },
    "request_id": "req-123",
    "timestamp": "2024-01-21T10:00:00Z"
  }
}
```

### Common Error Types
```json
// Authentication Error
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

// Validation Error
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid input data",
    "details": {
      "fields": {
        "email": "Invalid email format",
        "age": "Must be greater than 0"
      }
    },
    "request_id": "req-124",
    "timestamp": "2024-01-21T10:00:00Z"
  }
}

// Rate Limit Error
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
    "request_id": "req-125",
    "timestamp": "2024-01-21T09:59:00Z"
  }
}

// Tenant Error
{
  "error": {
    "code": "TENANT_ERROR",
    "message": "Tenant validation failed",
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

## Response Headers

### Standard Headers
```http
Content-Type: application/json
X-Request-ID: req-123
X-Tenant-ID: tenant-123
```

### Rate Limit Headers
```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 995
X-RateLimit-Reset: 1705689600
```

### Security Headers
```http
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Content-Security-Policy: default-src 'self'
Referrer-Policy: strict-origin-when-cross-origin
```

## Postman Collection

A complete Postman collection is available in the `postman` directory, including:
- Environment variables
- Request examples
- Test scripts
- Pre-request scripts
- Environment configs

## Insomnia Collection

An Insomnia collection is available in the `insomnia` directory, featuring:
- Environment setup
- Request templates
- Response validation
- GraphQL queries
- Authentication flows

## Maintenance

The API documentation is maintained in all supported languages:
- English (en)
- German (de)
- Albanian (sq)
- French (fr)
- Spanish (es)

### Translation Process
1. English documentation is the source of truth
2. Changes are first made to English documentation
3. Translations are updated within 24 hours
4. Automated checks ensure consistency across languages

### Version Control
- Documentation versions match API versions
- Breaking changes are clearly marked
- Deprecation notices include migration guides
- Version history is maintained 