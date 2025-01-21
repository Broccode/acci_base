# Documentación de la API ACCI Framework

## Estructura

```
api/
├── en/                    # Documentación en inglés
├── de/                    # Documentación en alemán
├── sq/                    # Documentación en albanés
├── fr/                    # Documentación en francés
└── es/                    # Documentación en español
    ├── rest/             # Documentación de la API REST
    │   ├── authentication/   # Autenticación & Autorización
    │   ├── endpoints/        # Puntos finales de la API
    │   ├── errors/          # Gestión de errores
    │   ├── examples/        # Ejemplos de uso
    │   └── schemas/         # Esquemas de solicitud/respuesta
    └── graphql/          # Documentación de la API GraphQL
        ├── authentication/   # Autenticación & Autorización
        ├── endpoints/        # Consultas & Mutaciones
        ├── errors/          # Gestión de errores
        ├── examples/        # Ejemplos de uso
        └── schemas/         # Esquema GraphQL
```

## Estándares de Documentación

### API REST
- Especificación OpenAPI/Swagger 3.0
- Ejemplos detallados de solicitud/respuesta
- Respuestas de error estandarizadas
- Información de límites de tasa
- Métodos de autenticación
- Consideraciones específicas de inquilinos

### API GraphQL
- Documentación completa del esquema
- Ejemplos de consultas/mutaciones
- Patrones de manejo de errores
- Flujo de autenticación
- Consejos de optimización de rendimiento
- Gestión del contexto de inquilinos

### Elementos Comunes
- Consideraciones de seguridad
- Límites & cuotas
- Patrones de paginación
- Información de versionado
- Mejores prácticas
- Ejemplos de SDK

## Parámetros URL

### Parámetros de Ruta
Use llaves para los parámetros de ruta:
```
/api/v1/tenants/{tenant_id}/users/{user_id}
```

NO use el prefijo de dos puntos:
```
/api/v1/tenants/:tenant_id/users/:user_id  # Obsoleto
```

## Respuestas de Error

### Formato Estándar de Error
```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Mensaje legible por humanos",
    "details": {
      "field1": "Información adicional",
      "field2": "Contexto adicional"
    },
    "request_id": "req-123",
    "timestamp": "2024-01-21T10:00:00Z"
  }
}
```

### Tipos Comunes de Error
```json
// Error de Autenticación
{
  "error": {
    "code": "AUTH_ERROR",
    "message": "Fallo en la autenticación",
    "details": {
      "reason": "invalid_token",
      "description": "El token ha expirado"
    },
    "request_id": "req-123",
    "timestamp": "2024-01-21T10:00:00Z"
  }
}

// Error de Validación
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Datos inválidos",
    "details": {
      "fields": {
        "email": "Formato de email inválido",
        "age": "Debe ser mayor que 0"
      }
    },
    "request_id": "req-124",
    "timestamp": "2024-01-21T10:00:00Z"
  }
}

// Error de Límite de Tasa
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Demasiadas solicitudes",
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

// Error de Inquilino
{
  "error": {
    "code": "TENANT_ERROR",
    "message": "Fallo en la validación del inquilino",
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

## Encabezados de Respuesta

### Encabezados Estándar
```http
Content-Type: application/json
X-Request-ID: req-123
X-Tenant-ID: tenant-123
```

### Encabezados de Límite de Tasa
```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 995
X-RateLimit-Reset: 1705689600
```

### Encabezados de Seguridad
```http
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Content-Security-Policy: default-src 'self'
Referrer-Policy: strict-origin-when-cross-origin
```

## Colección Postman

Una colección completa de Postman está disponible en el directorio `postman`, incluyendo:
- Variables de entorno
- Ejemplos de solicitudes
- Scripts de prueba
- Scripts de pre-solicitud
- Configuraciones de entorno

## Colección Insomnia

Una colección de Insomnia está disponible en el directorio `insomnia`, que incluye:
- Configuración del entorno
- Plantillas de solicitudes
- Validación de respuestas
- Consultas GraphQL
- Flujos de autenticación

## Mantenimiento

La documentación de la API se mantiene en todos los idiomas soportados:
- Inglés (en)
- Alemán (de)
- Albanés (sq)
- Francés (fr)
- Español (es)

### Proceso de Traducción
1. La documentación en inglés es la fuente de verdad
2. Los cambios se realizan primero en la documentación en inglés
3. Las traducciones se actualizan dentro de las 24 horas
4. Las verificaciones automatizadas aseguran la consistencia entre idiomas

### Control de Versiones
- Las versiones de la documentación coinciden con las versiones de la API
- Los cambios importantes están claramente marcados
- Los avisos de obsolescencia incluyen guías de migración
- Se mantiene el historial de versiones 