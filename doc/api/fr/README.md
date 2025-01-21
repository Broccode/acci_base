# Documentation de l'API ACCI Framework

## Structure

```
api/
├── en/                    # Documentation en anglais
├── de/                    # Documentation en allemand
├── sq/                    # Documentation en albanais
├── fr/                    # Documentation en français
└── es/                    # Documentation en espagnol
    ├── rest/             # Documentation de l'API REST
    │   ├── authentication/   # Authentification & Autorisation
    │   ├── endpoints/        # Points de terminaison de l'API
    │   ├── errors/          # Gestion des erreurs
    │   ├── examples/        # Exemples d'utilisation
    │   └── schemas/         # Schémas des requêtes/réponses
    └── graphql/          # Documentation de l'API GraphQL
        ├── authentication/   # Authentification & Autorisation
        ├── endpoints/        # Requêtes & Mutations
        ├── errors/          # Gestion des erreurs
        ├── examples/        # Exemples d'utilisation
        └── schemas/         # Schéma GraphQL
```

## Standards de Documentation

### API REST
- Spécification OpenAPI/Swagger 3.0
- Exemples détaillés de requêtes/réponses
- Réponses d'erreur standardisées
- Informations sur la limitation des requêtes
- Méthodes d'authentification
- Considérations spécifiques aux locataires

### API GraphQL
- Documentation complète du schéma
- Exemples de requêtes/mutations
- Modèles de gestion des erreurs
- Flux d'authentification
- Conseils d'optimisation des performances
- Gestion du contexte des locataires

### Éléments Communs
- Considérations de sécurité
- Limitations & quotas
- Modèles de pagination
- Informations de versionnage
- Meilleures pratiques
- Exemples SDK

## Paramètres URL

### Paramètres de Chemin
Utilisez les accolades pour les paramètres de chemin :
```
/api/v1/tenants/{tenant_id}/users/{user_id}
```

PAS le préfixe avec deux-points :
```
/api/v1/tenants/:tenant_id/users/:user_id  # Déprécié
```

## Réponses d'Erreur

### Format Standard d'Erreur
```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Message lisible par l'humain",
    "details": {
      "field1": "Information supplémentaire",
      "field2": "Contexte additionnel"
    },
    "request_id": "req-123",
    "timestamp": "2024-01-21T10:00:00Z"
  }
}
```

### Types d'Erreurs Courants
```json
// Erreur d'Authentification
{
  "error": {
    "code": "AUTH_ERROR",
    "message": "Échec de l'authentification",
    "details": {
      "reason": "invalid_token",
      "description": "Le jeton a expiré"
    },
    "request_id": "req-123",
    "timestamp": "2024-01-21T10:00:00Z"
  }
}

// Erreur de Validation
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Données invalides",
    "details": {
      "fields": {
        "email": "Format d'email invalide",
        "age": "Doit être supérieur à 0"
      }
    },
    "request_id": "req-124",
    "timestamp": "2024-01-21T10:00:00Z"
  }
}

// Erreur de Limitation de Requêtes
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Trop de requêtes",
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

// Erreur de Locataire
{
  "error": {
    "code": "TENANT_ERROR",
    "message": "Échec de la validation du locataire",
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

## En-têtes de Réponse

### En-têtes Standard
```http
Content-Type: application/json
X-Request-ID: req-123
X-Tenant-ID: tenant-123
```

### En-têtes de Limitation de Requêtes
```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 995
X-RateLimit-Reset: 1705689600
```

### En-têtes de Sécurité
```http
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Content-Security-Policy: default-src 'self'
Referrer-Policy: strict-origin-when-cross-origin
```

## Collection Postman

Une collection Postman complète est disponible dans le répertoire `postman`, incluant :
- Variables d'environnement
- Exemples de requêtes
- Scripts de test
- Scripts pré-requête
- Configurations d'environnement

## Collection Insomnia

Une collection Insomnia est disponible dans le répertoire `insomnia`, comprenant :
- Configuration de l'environnement
- Modèles de requêtes
- Validation des réponses
- Requêtes GraphQL
- Flux d'authentification

## Maintenance

La documentation de l'API est maintenue dans toutes les langues supportées :
- Anglais (en)
- Allemand (de)
- Albanais (sq)
- Français (fr)
- Espagnol (es)

### Processus de Traduction
1. La documentation en anglais est la source de vérité
2. Les modifications sont d'abord effectuées dans la documentation en anglais
3. Les traductions sont mises à jour dans les 24 heures
4. Des vérifications automatisées assurent la cohérence entre les langues

### Contrôle de Version
- Les versions de la documentation correspondent aux versions de l'API
- Les changements majeurs sont clairement marqués
- Les avis de dépréciation incluent des guides de migration
- L'historique des versions est maintenu 