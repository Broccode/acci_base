# ACCI Framework Developer Portal

## Quick Start Guides

### Authentication

#### JavaScript/TypeScript
```typescript
import { AcciClient } from '@acci/client';

const client = new AcciClient({
  baseUrl: 'https://api.acci-framework.com/v1',
  tenantId: 'your-tenant-id'
});

// Login
const { accessToken, refreshToken } = await client.auth.login({
  email: 'user@example.com',
  password: 'your-password'
});

// Use access token for subsequent requests
client.setAccessToken(accessToken);

// Refresh token when needed
const { accessToken: newToken } = await client.auth.refresh(refreshToken);
```

#### Python
```python
from acci_framework import AcciClient

client = AcciClient(
    base_url='https://api.acci-framework.com/v1',
    tenant_id='your-tenant-id'
)

# Login
auth_response = client.auth.login(
    email='user@example.com',
    password='your-password'
)

# Use access token for subsequent requests
client.set_access_token(auth_response.access_token)

# Refresh token when needed
new_token = client.auth.refresh(auth_response.refresh_token)
```

#### Rust
```rust
use acci_framework::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::new(
        "https://api.acci-framework.com/v1",
        "your-tenant-id"
    );

    // Login
    let auth = client.auth().login(
        "user@example.com",
        "your-password"
    ).await?;

    // Use access token for subsequent requests
    client.set_access_token(&auth.access_token);

    // Refresh token when needed
    let new_token = client.auth().refresh(&auth.refresh_token).await?;
    
    Ok(())
}
```

### User Management

#### JavaScript/TypeScript
```typescript
// List users
const { data: users, pagination } = await client.users.list({
  page: 1,
  perPage: 20
});

// Create user
const newUser = await client.users.create({
  email: 'newuser@example.com',
  password: 'secure-password',
  name: 'New User',
  roles: ['user']
});

// Update user
const updatedUser = await client.users.update(userId, {
  name: 'Updated Name',
  roles: ['user', 'admin']
});

// Get user details
const user = await client.users.get(userId);
```

#### Python
```python
# List users
users_response = client.users.list(page=1, per_page=20)
users = users_response.data
pagination = users_response.pagination

# Create user
new_user = client.users.create(
    email='newuser@example.com',
    password='secure-password',
    name='New User',
    roles=['user']
)

# Update user
updated_user = client.users.update(
    user_id,
    name='Updated Name',
    roles=['user', 'admin']
)

# Get user details
user = client.users.get(user_id)
```

#### Rust
```rust
// List users
let users_response = client.users()
    .list(ListUsersParams {
        page: Some(1),
        per_page: Some(20),
    })
    .await?;

// Create user
let new_user = client.users()
    .create(CreateUserParams {
        email: "newuser@example.com",
        password: "secure-password",
        name: "New User",
        roles: vec!["user".to_string()],
    })
    .await?;

// Update user
let updated_user = client.users()
    .update(user_id, UpdateUserParams {
        name: Some("Updated Name".to_string()),
        roles: Some(vec!["user".to_string(), "admin".to_string()]),
        ..Default::default()
    })
    .await?;

// Get user details
let user = client.users().get(user_id).await?;
```

### Tenant Management

#### JavaScript/TypeScript
```typescript
// List tenants
const { data: tenants, pagination } = await client.tenants.list({
  page: 1,
  perPage: 20
});

// Create tenant
const newTenant = await client.tenants.create({
  name: 'New Tenant',
  domain: 'newtenant.example.com',
  settings: {
    theme: 'light',
    features: ['api', 'dashboard']
  }
});

// Update tenant
const updatedTenant = await client.tenants.update(tenantId, {
  name: 'Updated Tenant',
  settings: {
    theme: 'dark',
    features: ['api', 'dashboard', 'analytics']
  }
});

// Get tenant details
const tenant = await client.tenants.get(tenantId);
```

#### Python
```python
# List tenants
tenants_response = client.tenants.list(page=1, per_page=20)
tenants = tenants_response.data
pagination = tenants_response.pagination

# Create tenant
new_tenant = client.tenants.create(
    name='New Tenant',
    domain='newtenant.example.com',
    settings={
        'theme': 'light',
        'features': ['api', 'dashboard']
    }
)

# Update tenant
updated_tenant = client.tenants.update(
    tenant_id,
    name='Updated Tenant',
    settings={
        'theme': 'dark',
        'features': ['api', 'dashboard', 'analytics']
    }
)

# Get tenant details
tenant = client.tenants.get(tenant_id)
```

#### Rust
```rust
// List tenants
let tenants_response = client.tenants()
    .list(ListTenantsParams {
        page: Some(1),
        per_page: Some(20),
    })
    .await?;

// Create tenant
let new_tenant = client.tenants()
    .create(CreateTenantParams {
        name: "New Tenant",
        domain: "newtenant.example.com",
        settings: json!({
            "theme": "light",
            "features": ["api", "dashboard"]
        }),
    })
    .await?;

// Update tenant
let updated_tenant = client.tenants()
    .update(tenant_id, UpdateTenantParams {
        name: Some("Updated Tenant".to_string()),
        settings: Some(json!({
            "theme": "dark",
            "features": ["api", "dashboard", "analytics"]
        })),
        ..Default::default()
    })
    .await?;

// Get tenant details
let tenant = client.tenants().get(tenant_id).await?;
```

## SDK Documentation

### Installation

#### JavaScript/TypeScript
```bash
# npm
npm install @acci/client

# yarn
yarn add @acci/client

# pnpm
pnpm add @acci/client
```

#### Python
```bash
# pip
pip install acci-framework

# poetry
poetry add acci-framework
```

#### Rust
```toml
[dependencies]
acci-framework = "1.0"
```

### Configuration

All SDKs support the following configuration options:

```typescript
interface ClientConfig {
  // Base URL for API requests
  baseUrl: string;
  
  // Tenant ID for multi-tenancy
  tenantId: string;
  
  // Optional configuration
  options?: {
    // Request timeout in milliseconds
    timeout?: number;
    
    // Retry configuration
    retry?: {
      attempts: number;
      backoff: number;
    };
    
    // Custom headers
    headers?: Record<string, string>;
  };
}
```

### Error Handling

All SDKs use consistent error types:

```typescript
interface ApiError {
  // Error code for programmatic handling
  code: string;
  
  // Human-readable error message
  message: string;
  
  // Additional error details
  details?: Record<string, unknown>;
  
  // Request ID for support
  requestId?: string;
}
```

Example error handling:

#### JavaScript/TypeScript
```typescript
try {
  await client.users.create(userData);
} catch (error) {
  if (error instanceof ApiError) {
    console.error(
      `API Error: ${error.code} - ${error.message}`,
      `Request ID: ${error.requestId}`
    );
  }
}
```

#### Python
```python
try:
    client.users.create(user_data)
except ApiError as error:
    print(
        f"API Error: {error.code} - {error.message}",
        f"Request ID: {error.request_id}"
    )
```

#### Rust
```rust
match client.users().create(user_data).await {
    Ok(user) => println!("User created: {}", user.id),
    Err(error) => match error {
        ApiError::Client { code, message, request_id, .. } => {
            println!(
                "API Error: {} - {}",
                code, message,
                "Request ID: {}", request_id
            );
        }
        _ => println!("Unknown error: {}", error),
    }
}
```

## API Reference

For detailed API documentation, visit:
- REST API: https://api.acci-framework.com/docs/rest
- GraphQL API: https://api.acci-framework.com/docs/graphql

## Support

- Documentation: https://docs.acci-framework.com
- GitHub Issues: https://github.com/your-org/acci-framework/issues
- Community Forum: https://community.acci-framework.com
- Email Support: support@acci-framework.com 