# ACCI Framework Architecture

## Table of Contents
1. [Core Concepts](#core-concepts)
2. [Project Requirements](#project-requirements)
3. [Technical Architecture](#technical-architecture)
4. [CQRS Architecture](#cqrs-architecture)
   - [Core CQRS Components](#core-cqrs-components)
   - [Event Schema Design](#event-schema-design)
   - [Command Structures](#command-structures)
   - [Command Architecture](#command-architecture)
   - [Query Architecture](#query-architecture)
   - [Event Store Configuration](#event-store-configuration)
   - [Message Bus Architecture](#message-bus-architecture)
   - [Projection System](#projection-system)
   - [Data Consistency](#data-consistency)
   - [Performance Optimization](#performance-optimization)
   - [Migration Strategy](#migration-strategy)
   - [Monitoring & Metrics](#monitoring--metrics)
5. [Infrastructure](#infrastructure)
6. [Security](#security)
7. [Development Guidelines](#development-guidelines)
8. [Quality Assurance](#quality-assurance)
9. [Operations](#operations)
10. [Internationalization](#internationalization)

## Core Concepts

### Purpose
```rust
struct ACCIFramework {
    multi_tenancy: bool,    // True - Base requirement
    user_management: bool,  // True - Built-in
    enterprise_ready: bool, // True - Default setting
}
```

### Key Principles
- Multi-tenant first
- API-driven architecture
- Security by design
- Enterprise-grade scalability
- Comprehensive observability

## Project Requirements

### Language Support Matrix
| Language | Code | Comments | Documentation | UI | API Docs |
|----------|------|----------|---------------|----|---------| 
| English  | ✓    | ✓        | ✓             | ✓  | ✓       |
| German   | -    | -        | ✓             | ✓  | ✓       |
| Albanian | -    | -        | ✓             | ✓  | ✓       |
| French   | -    | -        | ✓             | ✓  | ✓       |
| Spanish  | -    | -        | ✓             | ✓  | ✓       |

### Documentation Structure
```
doc/
├── architecture/    # Technical documentation (English only)
├── api/            # API documentation (Multi-language)
│   ├── en/         # English API docs
│   ├── de/         # German API docs
│   ├── sq/         # Albanian API docs
│   ├── fr/         # French API docs
│   └── es/         # Spanish API docs
├── development/    # Development guides (English only)
└── user/           # User documentation (Multi-language)
    ├── en/         # English user docs
    ├── de/         # German user docs
    ├── sq/         # Albanian user docs
    ├── fr/         # French user docs
    └── es/         # Spanish user docs
```

## Technical Architecture

### API Layer
```rust
#[derive(ApiLayer)]
struct ApiArchitecture {
    #[rest_api(
        version = "v1",
        format = "json",
        base_path = "/api/v1"
    )]
    rest: RestApi,

    #[graphql_api(
        schema = "schema.graphql",
        playground = true
    )]
    graphql: GraphQLApi,

    #[websocket_api(
        protocol = "graphql-ws",
        subscriptions = true
    )]
    websocket: WebSocketApi,
}

#[derive(RestEndpoints)]
struct RestApi {
    #[tenant_endpoints]
    struct TenantApi {
    #[endpoint(
            method = "POST",
            path = "/tenants",
            rate_limit = "100/m",
            auth = ["SYSTEM_ADMIN"]
        )]
        async fn create_tenant(
            &self,
            #[validate] payload: CreateTenantRequest,
        ) -> Result<TenantResponse, ApiError> {
            let cmd = CreateTenant {
                name: payload.name,
                subscription_plan: payload.subscription_plan,
                admin_email: payload.admin_email,
                metadata: payload.metadata,
                settings: payload.settings,
            };
            
            let tenant_id = self.command_bus.dispatch(cmd).await?;
            let tenant = self.tenant_queries.get_tenant_details(tenant_id).await?;
            
            Ok(TenantResponse::from(tenant))
        }

        #[endpoint(
            method = "GET",
            path = "/tenants/{tenant_id}",
            cache = "5m",
            auth = ["TENANT_ADMIN", "SYSTEM_ADMIN"]
        )]
        async fn get_tenant(
            &self,
            tenant_id: TenantId,
        ) -> Result<TenantResponse, ApiError> {
            let tenant = self.tenant_queries.get_tenant_details(tenant_id).await?;
            Ok(TenantResponse::from(tenant))
        }

        #[endpoint(
            method = "GET",
            path = "/tenants",
            pagination = true,
            auth = ["SYSTEM_ADMIN"]
        )]
        async fn list_tenants(
            &self,
            #[query] filters: TenantFilters,
            #[pagination] pagination: Pagination,
        ) -> Result<PaginatedResponse<TenantResponse>, ApiError> {
            let tenants = self.tenant_queries
                .list_active_tenants(pagination)
                .await?;
            
            Ok(PaginatedResponse::new(
                tenants.into_iter().map(TenantResponse::from).collect(),
                pagination
            ))
        }
    }

    #[user_endpoints]
    struct UserApi {
        #[endpoint(
            method = "POST",
            path = "/tenants/{tenant_id}/users",
            rate_limit = "100/m",
            auth = ["TENANT_ADMIN"]
        )]
        async fn register_user(
            &self,
            tenant_id: TenantId,
            #[validate] payload: RegisterUserRequest,
        ) -> Result<UserResponse, ApiError> {
            let cmd = RegisterUser {
                tenant_id,
                email: payload.email,
                role: payload.role,
                display_name: payload.display_name,
                metadata: payload.metadata,
                registration_source: RegistrationSource::AdminInvite,
            };
            
            let user_id = self.command_bus.dispatch(cmd).await?;
            let user = self.user_queries.get_user_profile(user_id, tenant_id).await?;
            
            Ok(UserResponse::from(user))
        }

        #[endpoint(
            method = "GET",
            path = "/tenants/{tenant_id}/users/{user_id}",
            cache = "5m",
            auth = ["TENANT_ADMIN", "TENANT_USER"]
        )]
        async fn get_user(
            &self,
            tenant_id: TenantId,
            user_id: UserId,
        ) -> Result<UserResponse, ApiError> {
            let user = self.user_queries.get_user_profile(user_id, tenant_id).await?;
            Ok(UserResponse::from(user))
        }
    }

    #[auth_endpoints]
    struct AuthApi {
        #[endpoint(
            method = "POST",
            path = "/auth/login",
            rate_limit = "10/m",
            auth = ["PUBLIC"]
        )]
        async fn login(
            &self,
            #[validate] payload: LoginRequest,
            #[context] client_info: ClientInfo,
        ) -> Result<LoginResponse, ApiError> {
            let cmd = Login {
                email: payload.email,
                auth_method: payload.auth_method,
                credentials: payload.credentials,
                device_info: client_info.into(),
                ip_address: client_info.ip_address,
            };
            
            let session = self.command_bus.dispatch(cmd).await?;
            Ok(LoginResponse::from(session))
        }

        #[endpoint(
            method = "POST",
            path = "/auth/logout",
            auth = ["AUTHENTICATED"]
        )]
        async fn logout(
            &self,
            #[session] session: SessionInfo,
        ) -> Result<(), ApiError> {
            let cmd = Logout {
                session_id: session.session_id,
                logout_type: LogoutType::UserInitiated,
            };
            
            self.command_bus.dispatch(cmd).await?;
            Ok(())
        }
    }
}

#[derive(GraphQLSchema)]
struct GraphQLApi {
    #[query]
    struct Query {
        #[field(
            name = "tenant",
            auth = ["TENANT_ADMIN", "SYSTEM_ADMIN"]
        )]
        async fn get_tenant(
            &self,
            tenant_id: TenantId,
        ) -> Result<TenantType, ApiError> {
            let tenant = self.tenant_queries.get_tenant_details(tenant_id).await?;
            Ok(TenantType::from(tenant))
        }

        #[field(
            name = "tenants",
            auth = ["SYSTEM_ADMIN"]
        )]
        async fn list_tenants(
            &self,
            filters: Option<TenantFiltersInput>,
            pagination: Option<PaginationInput>,
        ) -> Result<TenantConnection, ApiError> {
            let tenants = self.tenant_queries
                .list_active_tenants(pagination.unwrap_or_default().into())
                .await?;
            
            Ok(TenantConnection::new(tenants))
        }
    }

    #[mutation]
    struct Mutation {
        #[field(
            name = "createTenant",
            auth = ["SYSTEM_ADMIN"]
        )]
        async fn create_tenant(
            &self,
            input: CreateTenantInput,
        ) -> Result<CreateTenantPayload, ApiError> {
            let cmd = CreateTenant {
                name: input.name,
                subscription_plan: input.subscription_plan,
                admin_email: input.admin_email,
                metadata: input.metadata,
                settings: input.settings,
            };
            
            let tenant_id = self.command_bus.dispatch(cmd).await?;
            let tenant = self.tenant_queries.get_tenant_details(tenant_id).await?;
            
            Ok(CreateTenantPayload::new(tenant))
        }
    }

    #[subscription]
    struct Subscription {
        #[field(
            name = "tenantUpdates",
            auth = ["TENANT_ADMIN"]
        )]
        async fn tenant_updates(
            &self,
            tenant_id: TenantId,
        ) -> Result<impl Stream<Item = TenantType>, ApiError> {
            Ok(self.event_bus
                .subscribe(format!("tenant.{}", tenant_id))
                .filter_map(|event| async {
                    match event {
                        TenantEvent::Updated(data) => {
                            let tenant = self.tenant_queries
                                .get_tenant_details(data.tenant_id)
                                .await
                                .ok()?;
                            Some(TenantType::from(tenant))
                        }
                        _ => None,
                    }
                }))
        }
    }
}

#[derive(ApiSecurity)]
struct SecurityConfig {
    #[authentication(
        jwt = true,
        oauth2 = true,
        api_key = true
    )]
    auth: AuthConfig,

    #[authorization(
        rbac = true,
        tenant_isolation = true
    )]
    authz: AuthzConfig,

    #[rate_limiting(
        strategy = "token_bucket",
        scope = ["ip", "user", "tenant"]
    )]
    rate_limits: RateLimitConfig,
}

#[derive(ApiDocumentation)]
struct DocumentationConfig {
    #[openapi(
        version = "3.0.0",
        title = "ACCI Framework API",
        description = "Enterprise-grade multi-tenant API"
    )]
    openapi: OpenApiConfig,

    #[graphql_docs(
        schema = true,
        playground = true
    )]
    graphql_docs: GraphQLDocsConfig,

    #[translations(
        languages = ["en", "de", "sq", "fr", "es"]
    )]
    i18n: I18nConfig,
}

#[derive(ApiMonitoring)]
struct MonitoringConfig {
    #[metrics(
        response_time = true,
        error_rate = true,
        request_rate = true
    )]
    metrics: MetricsConfig,

    #[tracing(
        request_id = true,
        correlation_id = true
    )]
    tracing: TracingConfig,

    #[logging(
        requests = true,
        responses = true,
        errors = true
    )]
    logging: LoggingConfig,
}

### API DTOs
```rust
#[derive(ApiContracts)]
struct ApiDTOs {
    #[tenant_contracts]
    struct TenantDTOs {
        #[request("tenant.create")]
        struct CreateTenantRequest {
            #[validate(length(min = 3, max = 50))]
            name: String,
            
            #[validate(enum_value)]
            subscription_plan: SubscriptionPlan,
            
            #[validate(email)]
            admin_email: EmailAddress,
            
            #[validate(optional)]
            metadata: Option<JsonValue>,
            
            #[validate]
            settings: TenantSettingsDTO,
        }

        #[response("tenant.create")]
        struct TenantResponse {
            tenant_id: TenantId,
            name: String,
            subscription_plan: SubscriptionPlan,
            admin_email: EmailAddress,
            metadata: Option<JsonValue>,
            settings: TenantSettingsDTO,
            status: TenantStatus,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }

        #[request("tenant.update")]
        struct UpdateTenantRequest {
            #[validate(length(min = 3, max = 50))]
            name: Option<String>,
            
            #[validate(enum_value)]
            subscription_plan: Option<SubscriptionPlan>,
            
            #[validate(optional)]
            metadata: Option<JsonValue>,
            
            #[validate]
            settings: Option<TenantSettingsDTO>,
        }

        #[request("tenant.delete")]
        struct DeleteTenantRequest {
            #[validate(enum_value)]
            deletion_reason: DeletionReason,
            
            #[validate(range(min = "30d", max = "365d"))]
            data_retention_period: Option<Duration>,
        }
    }

    #[user_contracts]
    struct UserDTOs {
        #[request("user.register")]
        struct RegisterUserRequest {
            #[validate(email)]
            email: EmailAddress,
            
            #[validate(enum_value)]
            role: UserRole,
            
            #[validate(length(min = 1, max = 100))]
            display_name: Option<String>,
            
            #[validate(optional)]
            metadata: Option<JsonValue>,
        }

        #[response("user.register")]
        struct UserResponse {
            user_id: UserId,
            tenant_id: TenantId,
            email: EmailAddress,
            display_name: Option<String>,
            role: UserRole,
            status: UserStatus,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
            last_login: Option<DateTime<Utc>>,
        }

        #[request("user.update_profile")]
        struct UpdateUserProfileRequest {
            #[validate(email)]
            email: Option<EmailAddress>,
            
            #[validate(length(min = 1, max = 100))]
            display_name: Option<String>,
            
            #[validate(enum_value)]
            role: Option<UserRole>,
            
            #[validate(optional)]
            metadata: Option<JsonValue>,
        }

        #[request("user.deactivate")]
        struct DeactivateUserRequest {
            #[validate(enum_value)]
            reason: DeactivationReason,
            
            #[validate(enum_value)]
            deactivation_type: DeactivationType,
            
            #[validate(future_date)]
            reactivation_date: Option<DateTime<Utc>>,
        }
    }

    #[auth_contracts]
    struct AuthDTOs {
        #[request("auth.login")]
        struct LoginRequest {
            #[validate(email)]
            email: EmailAddress,
            
            #[validate(enum_value)]
            auth_method: AuthenticationMethod,
            
            #[validate]
            credentials: AuthCredentialsDTO,
        }

        #[response("auth.login")]
        struct LoginResponse {
            session_id: SessionId,
            user: UserResponse,
            token: String,
            expires_at: DateTime<Utc>,
        }

        #[nested("auth.credentials")]
        enum AuthCredentialsDTO {
            #[validate(password_policy)]
            Password {
                password: String,
            },
            OAuth {
                provider: String,
                code: String,
            },
            SAML {
                assertion: String,
            },
            TOTP {
                code: String,
            },
            WebAuthn {
                assertion: String,
            },
        }
    }

    #[common_contracts]
    struct CommonDTOs {
        #[nested("tenant.settings")]
        struct TenantSettingsDTO {
            #[validate(array(min = 1))]
            allowed_auth_methods: Vec<AuthenticationMethod>,
            
            #[validate(range(min = 1, max = 1000000))]
            max_users: u32,
            
            #[validate(array)]
            features: HashSet<String>,
            
            #[validate(domain_name)]
            custom_domain: Option<String>,
            
            #[validate]
            security_policy: SecurityPolicyDTO,
        }

        #[nested("tenant.security_policy")]
        struct SecurityPolicyDTO {
            #[validate(range(min = 8, max = 128))]
            password_min_length: u8,
            
            #[validate(bool)]
            require_mfa: bool,
            
            #[validate(range(min = 0, max = 10))]
            max_login_attempts: u8,
            
            #[validate(range(min = "1m", max = "24h"))]
            session_timeout: Duration,
            
            #[validate(array)]
            allowed_ip_ranges: Vec<IpRange>,
        }

        #[nested("pagination")]
        struct PaginationDTO {
            #[validate(range(min = 1, max = 100))]
            page_size: u32,
            
            #[validate(min = 1)]
            page: u32,
            
            #[validate(optional)]
            sort_by: Option<String>,
            
            #[validate(enum_value)]
            sort_order: Option<SortOrder>,
        }

        #[nested("filters")]
        struct FilterDTO {
            field: String,
            operator: FilterOperator,
            value: JsonValue,
        }
    }
}

#[derive(ApiValidation)]
struct ValidationRules {
    #[password_policy]
    struct PasswordPolicy {
        min_length: u8 = 8,
        require_uppercase: bool = true,
        require_lowercase: bool = true,
        require_numbers: bool = true,
        require_special: bool = true,
        max_length: u8 = 128,
    }

    #[email_policy]
    struct EmailPolicy {
        allowed_domains: Option<Vec<String>>,
        block_disposable: bool = true,
        verify_mx: bool = true,
    }

    #[domain_policy]
    struct DomainPolicy {
        allowed_tlds: Vec<String>,
        block_localhost: bool = true,
        verify_ownership: bool = true,
    }
}

#[derive(ApiSerialization)]
struct SerializationConfig {
    #[json(
        rename_all = "camelCase",
        skip_serializing_null = true
    )]
    json_config: JsonConfig,

    #[datetime(
        format = "rfc3339",
        timezone = "utc"
    )]
    datetime_config: DateTimeConfig,

    #[numbers(
        string_integers = false,
        decimal_places = 2
    )]
    number_config: NumberConfig,
}

#[derive(ApiVersioning)]
struct VersioningConfig {
    #[versions]
    supported_versions: Vec<ApiVersion> = [
        ApiVersion {
            version: "v1",
            status: VersionStatus::Current,
            sunset_date: None,
        },
    ],

    #[compatibility(
        breaking_changes = "major",
        deprecation_period = "6m"
    )]
    compatibility: CompatibilityConfig,
}

### API Design Requirements
```rust
#[derive(ApiDesign)]
struct ApiRequirements {
    #[validation(
        request = true,
        response = true,
        schema = true
    )]
    validation: ValidationConfig,

    #[rate_limit(
        requests = 100,
        period = "1m",
        scope = "tenant"
    )]
    rate_limiting: RateLimitConfig,

    #[tenant_quotas(
        api_calls = 10000,
        storage = "100GB",
        users = 1000
    )]
    quota_management: QuotaConfig,
}
```

### Database Layer
```rust
#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "resources")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub tenant_id: i32,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    #[sea_orm(column_type = "JsonBinary")]
    pub metadata: Json,
}
```

### Event System
```rust
#[derive(Event)]
struct ResourceEvent {
    #[event_type("resource.created")]
    created: ResourceCreated,
    #[event_type("resource.updated")]
    updated: ResourceUpdated,
    #[event_type("resource.deleted")]
    deleted: ResourceDeleted,
}
```

### Cache Strategy
```rust
#[derive(Cache)]
struct CacheConfiguration {
    #[cache(type = "memory", ttl = "5m")]
    pub application_cache: ApplicationCache,
    
    #[cache(type = "redis", ttl = "1h")]
    pub distributed_cache: DistributedCache,
    
    #[cache(type = "cdn", ttl = "24h")]
    pub static_assets: StaticAssetCache,
}
```

## CQRS Architecture

### Core CQRS Components
```rust
#[derive(CQRSCore)]
struct CQRSArchitecture {
    #[command_side(
        validation = true,
        idempotency = true,
        event_sourcing = true
    )]
    command_components: CommandArchitecture,

    #[query_side(
        caching = true,
        eventual_consistency = true
    )]
    query_components: QueryArchitecture,

    #[event_store(
        type = "EventStoreDB",
        snapshots = true,
        retention = "infinite"
    )]
    event_storage: EventStore,
}
```

### Event Schema Design
```rust
#[derive(DomainEvent)]
struct EventSchema {
    #[metadata(required = true)]
    metadata: EventMetadata,
    
    #[payload(versioned = true)]
    payload: EventPayload,
    
    #[schema(validation = true)]
    schema: EventSchemaVersion,
}

#[derive(EventMetadata)]
struct EventMetadata {
    #[required(always = true)]
    event_id: EventId,
    
    #[required(always = true)]
    event_type: String,
    
    #[required(always = true)]
    event_version: SemanticVersion,
    
    #[required(always = true)]
    timestamp: DateTime<Utc>,
    
    #[required(always = true)]
    tenant_id: TenantId,
    
    #[required(always = true)]
    correlation_id: CorrelationId,
    
    #[optional]
    causation_id: Option<EventId>,
    
    #[optional]
    user_id: Option<UserId>,
}

#[derive(EventVersioning)]
struct VersioningStrategy {
    #[schema_evolution(
        backwards_compatible = true,
        forwards_compatible = true
    )]
    evolution: SchemaEvolution,

    #[version_mapping(
        strategy = "semantic",
        format = "major.minor.patch"
    )]
    versioning: VersionMapping,

    #[compatibility(
        check = "runtime",
        validation = "strict"
    )]
    compatibility: CompatibilityCheck,
}

#[derive(CoreEvents)]
enum CoreDomainEvents {
    #[tenant(scoped = true)]
    TenantEvents {
        #[event("tenant.created")]
        TenantCreated(TenantCreatedData),
        
        #[event("tenant.updated")]
        TenantUpdated(TenantUpdatedData),
        
        #[event("tenant.deleted")]
        TenantDeleted(TenantDeletedData),
    },

    #[user(scoped = true)]
    UserEvents {
        #[event("user.registered")]
        UserRegistered(UserRegisteredData),
        
        #[event("user.profile_updated")]
        UserProfileUpdated(UserProfileUpdatedData),
        
        #[event("user.deactivated")]
        UserDeactivated(UserDeactivatedData),
    },

    #[authentication(scoped = true)]
    AuthEvents {
        #[event("auth.login_succeeded")]
        LoginSucceeded(LoginSucceededData),
        
        #[event("auth.login_failed")]
        LoginFailed(LoginFailedData),
        
        #[event("auth.logout")]
        LoggedOut(LoggedOutData),
    },
}

#[derive(EventPayloads)]
struct TenantEventPayloads {
    #[event_data("tenant.created")]
    struct TenantCreatedData {
        #[required]
        tenant_id: TenantId,
        #[required]
        name: String,
        #[required]
        subscription_plan: SubscriptionPlan,
        #[required]
        admin_email: EmailAddress,
        #[optional]
        metadata: Option<JsonValue>,
        #[required]
        settings: TenantSettings,
    }

    #[event_data("tenant.updated")]
    struct TenantUpdatedData {
        #[required]
        tenant_id: TenantId,
        #[optional]
        name: Option<String>,
        #[optional]
        subscription_plan: Option<SubscriptionPlan>,
        #[optional]
        metadata: Option<JsonValue>,
        #[optional]
        settings: Option<TenantSettings>,
        #[required]
        updated_fields: Vec<String>,
    }

    #[event_data("tenant.deleted")]
    struct TenantDeletedData {
        #[required]
        tenant_id: TenantId,
        #[required]
        deletion_reason: DeletionReason,
        #[required]
        data_retention_period: Duration,
    }
}

#[derive(EventPayloads)]
struct UserEventPayloads {
    #[event_data("user.registered")]
    struct UserRegisteredData {
        #[required]
        user_id: UserId,
        #[required]
        tenant_id: TenantId,
        #[required]
        email: EmailAddress,
        #[required]
        role: UserRole,
        #[optional]
        display_name: Option<String>,
        #[optional]
        metadata: Option<JsonValue>,
        #[required]
        registration_source: RegistrationSource,
    }

    #[event_data("user.profile_updated")]
    struct UserProfileUpdatedData {
        #[required]
        user_id: UserId,
        #[required]
        tenant_id: TenantId,
        #[optional]
        email: Option<EmailAddress>,
        #[optional]
        display_name: Option<String>,
        #[optional]
        role: Option<UserRole>,
        #[optional]
        metadata: Option<JsonValue>,
        #[required]
        updated_fields: Vec<String>,
    }

    #[event_data("user.deactivated")]
    struct UserDeactivatedData {
        #[required]
        user_id: UserId,
        #[required]
        tenant_id: TenantId,
        #[required]
        reason: DeactivationReason,
        #[required]
        deactivation_type: DeactivationType,
        #[optional]
        reactivation_date: Option<DateTime<Utc>>,
    }
}

#[derive(EventPayloads)]
struct AuthEventPayloads {
    #[event_data("auth.login_succeeded")]
    struct LoginSucceededData {
        #[required]
        user_id: UserId,
        #[required]
        tenant_id: TenantId,
        #[required]
        login_timestamp: DateTime<Utc>,
        #[required]
        auth_method: AuthenticationMethod,
        #[required]
        device_info: DeviceInfo,
        #[required]
        ip_address: IpAddr,
    }

    #[event_data("auth.login_failed")]
    struct LoginFailedData {
        #[optional]
        user_id: Option<UserId>,
        #[optional]
        tenant_id: Option<TenantId>,
        #[required]
        failure_reason: LoginFailureReason,
        #[required]
        auth_method: AuthenticationMethod,
        #[required]
        device_info: DeviceInfo,
        #[required]
        ip_address: IpAddr,
        #[required]
        attempt_timestamp: DateTime<Utc>,
    }

    #[event_data("auth.logout")]
    struct LoggedOutData {
        #[required]
        user_id: UserId,
        #[required]
        tenant_id: TenantId,
        #[required]
        session_id: SessionId,
        #[required]
        logout_type: LogoutType,
        #[required]
        logout_timestamp: DateTime<Utc>,
    }
}

#[derive(SupportTypes)]
enum SupportTypes {
    #[tenant_types]
    struct TenantTypes {
        #[derive(SubscriptionPlan)]
        enum SubscriptionPlan {
            Free,
            Professional,
            Enterprise,
            Custom(String),
        }

        #[derive(TenantSettings)]
        struct TenantSettings {
            allowed_auth_methods: Vec<AuthenticationMethod>,
            max_users: u32,
            features: HashSet<String>,
            custom_domain: Option<String>,
            security_policy: SecurityPolicy,
        }

        #[derive(DeletionReason)]
        enum DeletionReason {
            UserRequested,
            Violation,
            Inactive,
            Other(String),
        }
    }

    #[user_types]
    struct UserTypes {
        #[derive(UserRole)]
        enum UserRole {
            Admin,
            User,
            ReadOnly,
            Custom(String),
        }

        #[derive(RegistrationSource)]
        enum RegistrationSource {
            SelfService,
            AdminInvite,
            OAuth(String),
            SAML(String),
        }

        #[derive(DeactivationType)]
        enum DeactivationType {
            Temporary,
            Permanent,
        }

        #[derive(DeactivationReason)]
        enum DeactivationReason {
            UserRequested,
            AdminAction,
            SecurityViolation,
            Inactive,
            Other(String),
        }
    }

    #[auth_types]
    struct AuthTypes {
        #[derive(AuthenticationMethod)]
        enum AuthenticationMethod {
            Password,
            OAuth(String),
            SAML(String),
            TOTP,
            WebAuthn,
        }

        #[derive(LoginFailureReason)]
        enum LoginFailureReason {
            InvalidCredentials,
            AccountLocked,
            AccountDeactivated,
            TenantDeactivated,
            TooManyAttempts,
            Other(String),
        }

        #[derive(LogoutType)]
        enum LogoutType {
            UserInitiated,
            SessionExpired,
            AdminForced,
            SecurityPolicy,
        }

        #[derive(DeviceInfo)]
        struct DeviceInfo {
            user_agent: String,
            device_id: Option<String>,
            platform: String,
            app_version: Option<String>,
        }
    }
}

#[derive(EventValidation)]
struct ValidationRules {
    #[schema_validation(
        json_schema = true,
        strict_types = true
    )]
    schema_rules: SchemaValidation,

    #[business_validation(
        rules = true,
        tenant_specific = true
    )]
    business_rules: BusinessValidation,

    #[data_validation(
        sanitization = true,
        max_payload_size = "1MB"
    )]
    data_rules: DataValidation,
}

#[derive(EventStorage)]
struct StorageStrategy {
    #[serialization(
        format = "json",
        compression = true
    )]
    serialization: SerializationConfig,

    #[indexing(
        fields = [
            "metadata.tenant_id",
            "metadata.event_type",
            "metadata.timestamp"
        ]
    )]
    indexes: IndexConfig,

    #[partitioning(
        strategy = "by_tenant_and_type",
        shard_key = ["tenant_id", "event_type"]
    )]
    partitioning: PartitionConfig,
}

### Command Structures
```rust
#[derive(CommandStructures)]
struct Commands {
    #[tenant_commands(scope = "tenant")]
    struct TenantCommands {
        #[command("tenant.create")]
        struct CreateTenant {
            #[required]
            name: String,
            #[required]
            subscription_plan: SubscriptionPlan,
            #[required]
            admin_email: EmailAddress,
            #[optional]
            metadata: Option<JsonValue>,
            #[required]
            settings: TenantSettings,
        }

        #[command("tenant.update")]
        struct UpdateTenant {
            #[required]
            tenant_id: TenantId,
            #[optional]
            name: Option<String>,
            #[optional]
            subscription_plan: Option<SubscriptionPlan>,
            #[optional]
            metadata: Option<JsonValue>,
            #[optional]
            settings: Option<TenantSettings>,
        }

        #[command("tenant.delete")]
        struct DeleteTenant {
            #[required]
            tenant_id: TenantId,
            #[required]
            deletion_reason: DeletionReason,
            #[optional]
            data_retention_period: Option<Duration>,
        }
    }

    #[user_commands(scope = "tenant")]
    struct UserCommands {
        #[command("user.register")]
        struct RegisterUser {
            #[required]
            tenant_id: TenantId,
            #[required]
            email: EmailAddress,
            #[required]
            role: UserRole,
            #[optional]
            display_name: Option<String>,
            #[optional]
            metadata: Option<JsonValue>,
            #[required]
            registration_source: RegistrationSource,
        }

        #[command("user.update_profile")]
        struct UpdateUserProfile {
            #[required]
            user_id: UserId,
            #[required]
            tenant_id: TenantId,
            #[optional]
            email: Option<EmailAddress>,
            #[optional]
            display_name: Option<String>,
            #[optional]
            role: Option<UserRole>,
            #[optional]
            metadata: Option<JsonValue>,
        }

        #[command("user.deactivate")]
        struct DeactivateUser {
            #[required]
            user_id: UserId,
            #[required]
            tenant_id: TenantId,
            #[required]
            reason: DeactivationReason,
            #[required]
            deactivation_type: DeactivationType,
            #[optional]
            reactivation_date: Option<DateTime<Utc>>,
        }
    }

    #[auth_commands(scope = "tenant")]
    struct AuthCommands {
        #[command("auth.login")]
        struct Login {
            #[required]
            email: EmailAddress,
            #[required]
            auth_method: AuthenticationMethod,
            #[required]
            credentials: AuthCredentials,
            #[required]
            device_info: DeviceInfo,
            #[required]
            ip_address: IpAddr,
        }

        #[command("auth.logout")]
        struct Logout {
            #[required]
            session_id: SessionId,
            #[required]
            logout_type: LogoutType,
        }
    }
}

#[derive(CommandValidation)]
struct CommandValidation {
    #[validation(
        schema = true,
        business_rules = true
    )]
    validation_rules: ValidationRules,

    #[authorization(
        tenant_check = true,
        role_check = true
    )]
    auth_rules: AuthorizationRules,

    #[rate_limiting(
        tenant_scoped = true,
        user_scoped = true
    )]
    rate_limits: RateLimitRules,
}

#[derive(CommandHandling)]
struct CommandHandling {
    #[middleware(
        order = [
            "logging",
            "validation",
            "authorization",
            "rate_limiting",
            "idempotency"
        ]
    )]
    middleware_chain: MiddlewareChain,

    #[error_handling(
        retry = true,
        compensation = true
    )]
    error_handling: ErrorHandling,

    #[idempotency(
        check = true,
        key_extraction = "command_id"
    )]
    idempotency: IdempotencyConfig,
}

### Aggregate Roots
```rust
#[derive(AggregateRoot)]
struct TenantAggregate {
    #[aggregate_id]
    tenant_id: TenantId,

    #[state]
    state: TenantState,

    #[version]
    version: u64,

    #[commands]
    impl TenantCommands {
        #[command_handler("tenant.create")]
        async fn handle_create(
            &mut self,
            cmd: CreateTenant,
        ) -> Result<Vec<DomainEvent>, DomainError> {
            self.validate_tenant_creation(&cmd)?;
            
            Ok(vec![TenantCreated {
                tenant_id: self.tenant_id,
                name: cmd.name,
                subscription_plan: cmd.subscription_plan,
                admin_email: cmd.admin_email,
                metadata: cmd.metadata,
                settings: cmd.settings,
            }])
        }

        #[command_handler("tenant.update")]
        async fn handle_update(
            &mut self,
            cmd: UpdateTenant,
        ) -> Result<Vec<DomainEvent>, DomainError> {
            self.ensure_active()?;
            self.validate_tenant_update(&cmd)?;
            
            let mut updated_fields = Vec::new();
            let mut event = TenantUpdated {
                tenant_id: self.tenant_id,
                name: None,
                subscription_plan: None,
                metadata: None,
                settings: None,
                updated_fields: Vec::new(),
            };

            if let Some(name) = cmd.name {
                event.name = Some(name);
                updated_fields.push("name".to_string());
            }
            // ... similar for other fields

            event.updated_fields = updated_fields;
            Ok(vec![event])
        }

        #[command_handler("tenant.delete")]
        async fn handle_delete(
            &mut self,
            cmd: DeleteTenant,
        ) -> Result<Vec<DomainEvent>, DomainError> {
            self.ensure_active()?;
            self.validate_tenant_deletion(&cmd)?;
            
            Ok(vec![TenantDeleted {
                tenant_id: self.tenant_id,
                deletion_reason: cmd.deletion_reason,
                data_retention_period: cmd.data_retention_period
                    .unwrap_or_else(|| Duration::days(90)),
            }])
        }
    }

    #[invariants]
    impl TenantInvariants {
        fn ensure_active(&self) -> Result<(), DomainError> {
            if self.state.is_deleted {
                return Err(DomainError::TenantDeleted);
            }
            Ok(())
        }

        fn validate_tenant_creation(&self, cmd: &CreateTenant) -> Result<(), DomainError> {
            // Implementierung der Business Rules
            Ok(())
        }

        fn validate_tenant_update(&self, cmd: &UpdateTenant) -> Result<(), DomainError> {
            // Implementierung der Business Rules
            Ok(())
        }

        fn validate_tenant_deletion(&self, cmd: &DeleteTenant) -> Result<(), DomainError> {
            // Implementierung der Business Rules
            Ok(())
        }
    }

    #[apply_events]
    impl TenantEventApplier {
        fn apply_tenant_created(&mut self, event: &TenantCreated) {
            self.state = TenantState {
                name: event.name.clone(),
                subscription_plan: event.subscription_plan.clone(),
                admin_email: event.admin_email.clone(),
                metadata: event.metadata.clone(),
                settings: event.settings.clone(),
                is_deleted: false,
            };
        }

        fn apply_tenant_updated(&mut self, event: &TenantUpdated) {
            if let Some(name) = &event.name {
                self.state.name = name.clone();
            }
            // ... similar for other fields
        }

        fn apply_tenant_deleted(&mut self, event: &TenantDeleted) {
            self.state.is_deleted = true;
        }
    }
}

#[derive(AggregateRoot)]
struct UserAggregate {
    #[aggregate_id]
    user_id: UserId,

    #[state]
    state: UserState,

    #[version]
    version: u64,

    #[commands]
    impl UserCommands {
        #[command_handler("user.register")]
        async fn handle_register(
            &mut self,
            cmd: RegisterUser,
        ) -> Result<Vec<DomainEvent>, DomainError> {
            self.validate_user_registration(&cmd)?;
            
            Ok(vec![UserRegistered {
                user_id: self.user_id,
                tenant_id: cmd.tenant_id,
                email: cmd.email,
                role: cmd.role,
                display_name: cmd.display_name,
                metadata: cmd.metadata,
                registration_source: cmd.registration_source,
            }])
        }

        #[command_handler("user.update_profile")]
        async fn handle_update_profile(
            &mut self,
            cmd: UpdateUserProfile,
        ) -> Result<Vec<DomainEvent>, DomainError> {
            self.ensure_active()?;
            self.validate_profile_update(&cmd)?;
            
            let mut updated_fields = Vec::new();
            let mut event = UserProfileUpdated {
                user_id: self.user_id,
                tenant_id: cmd.tenant_id,
                email: None,
                display_name: None,
                role: None,
                metadata: None,
                updated_fields: Vec::new(),
            };

            if let Some(email) = cmd.email {
                event.email = Some(email);
                updated_fields.push("email".to_string());
            }
            // ... similar for other fields

            event.updated_fields = updated_fields;
            Ok(vec![event])
        }

        #[command_handler("user.deactivate")]
        async fn handle_deactivate(
            &mut self,
            cmd: DeactivateUser,
        ) -> Result<Vec<DomainEvent>, DomainError> {
            self.ensure_active()?;
            self.validate_deactivation(&cmd)?;
            
            Ok(vec![UserDeactivated {
                user_id: self.user_id,
                tenant_id: cmd.tenant_id,
                reason: cmd.reason,
                deactivation_type: cmd.deactivation_type,
                reactivation_date: cmd.reactivation_date,
            }])
        }
    }

    #[invariants]
    impl UserInvariants {
        fn ensure_active(&self) -> Result<(), DomainError> {
            if self.state.is_deactivated {
                return Err(DomainError::UserDeactivated);
            }
            Ok(())
        }

        fn validate_user_registration(&self, cmd: &RegisterUser) -> Result<(), DomainError> {
            // Implementierung der Business Rules
            Ok(())
        }

        fn validate_profile_update(&self, cmd: &UpdateUserProfile) -> Result<(), DomainError> {
            // Implementierung der Business Rules
            Ok(())
        }

        fn validate_deactivation(&self, cmd: &DeactivateUser) -> Result<(), DomainError> {
            // Implementierung der Business Rules
            Ok(())
        }
    }

    #[apply_events]
    impl UserEventApplier {
        fn apply_user_registered(&mut self, event: &UserRegistered) {
            self.state = UserState {
                tenant_id: event.tenant_id,
                email: event.email.clone(),
                role: event.role.clone(),
                display_name: event.display_name.clone(),
                metadata: event.metadata.clone(),
                is_deactivated: false,
            };
        }

        fn apply_profile_updated(&mut self, event: &UserProfileUpdated) {
            if let Some(email) = &event.email {
                self.state.email = email.clone();
            }
            // ... similar for other fields
        }

        fn apply_user_deactivated(&mut self, event: &UserDeactivated) {
            self.state.is_deactivated = true;
        }
    }
}

#[derive(AggregateStates)]
struct AggregateStates {
    #[state("TenantState")]
    struct TenantState {
        name: String,
        subscription_plan: SubscriptionPlan,
        admin_email: EmailAddress,
        metadata: Option<JsonValue>,
        settings: TenantSettings,
        is_deleted: bool,
    }

    #[state("UserState")]
    struct UserState {
        tenant_id: TenantId,
        email: EmailAddress,
        role: UserRole,
        display_name: Option<String>,
        metadata: Option<JsonValue>,
        is_deactivated: bool,
    }
}

### Business Rules
```rust
#[derive(BusinessRules)]
struct DomainRules {
    #[tenant_rules]
    struct TenantRules {
        #[creation_rules]
        fn validate_tenant_creation(&self, cmd: &CreateTenant) -> Result<(), DomainError> {
            // Name Validierung
            if cmd.name.len() < 3 || cmd.name.len() > 50 {
                return Err(DomainError::InvalidTenantName(
                    "Tenant name must be between 3 and 50 characters".into()
                ));
            }

            // Email Validierung
            if !is_valid_email_domain(&cmd.admin_email) {
                return Err(DomainError::InvalidEmailDomain(
                    "Admin email domain not allowed".into()
                ));
            }

            // Subscription Plan Validierung
            match &cmd.subscription_plan {
                SubscriptionPlan::Custom(plan) => {
                    if !is_valid_custom_plan(plan) {
                        return Err(DomainError::InvalidSubscriptionPlan(
                            "Invalid custom subscription plan".into()
                        ));
                    }
                }
                _ => Ok(()),
            }?;

            // Settings Validierung
            validate_tenant_settings(&cmd.settings)?;

            Ok(())
        }

        #[update_rules]
        fn validate_tenant_update(&self, cmd: &UpdateTenant) -> Result<(), DomainError> {
            // Name Änderung
            if let Some(name) = &cmd.name {
                if name.len() < 3 || name.len() > 50 {
                    return Err(DomainError::InvalidTenantName(
                        "Tenant name must be between 3 and 50 characters".into()
                    ));
                }
            }

            // Plan Downgrade Check
            if let Some(new_plan) = &cmd.subscription_plan {
                if is_plan_downgrade(&self.state.subscription_plan, new_plan) {
                    validate_downgrade_possible(&self.state)?;
                }
            }

            // Settings Änderungen
            if let Some(settings) = &cmd.settings {
                validate_tenant_settings(settings)?;
                validate_settings_transition(&self.state.settings, settings)?;
            }

            Ok(())
        }

        #[deletion_rules]
        fn validate_tenant_deletion(&self, cmd: &DeleteTenant) -> Result<(), DomainError> {
            // Aktive User Check
            if has_active_users(&self.tenant_id) {
                return Err(DomainError::TenantHasActiveUsers);
            }

            // Subscription Check
            if has_active_subscription(&self.tenant_id) {
                return Err(DomainError::ActiveSubscriptionExists);
            }

            // Data Retention Period
            if let Some(period) = cmd.data_retention_period {
                if period < Duration::days(30) || period > Duration::days(365) {
                    return Err(DomainError::InvalidRetentionPeriod);
                }
            }

            Ok(())
        }
    }

    #[user_rules]
    struct UserRules {
        #[registration_rules]
        fn validate_user_registration(&self, cmd: &RegisterUser) -> Result<(), DomainError> {
            // Email Format und Domain
            validate_email_format(&cmd.email)?;
            validate_email_domain(&cmd.email, &cmd.tenant_id)?;

            // Tenant Quota
            check_user_quota(&cmd.tenant_id)?;

            // Rollen-Validierung
            match &cmd.role {
                UserRole::Admin => validate_admin_creation(&cmd.tenant_id)?,
                UserRole::Custom(role) => validate_custom_role(role, &cmd.tenant_id)?,
                _ => Ok(()),
            }?;

            // Registration Source
            match &cmd.registration_source {
                RegistrationSource::OAuth(provider) => {
                    validate_oauth_provider(provider, &cmd.tenant_id)?;
                }
                RegistrationSource::SAML(provider) => {
                    validate_saml_provider(provider, &cmd.tenant_id)?;
                }
                _ => Ok(()),
            }?;

            Ok(())
        }

        #[profile_update_rules]
        fn validate_profile_update(&self, cmd: &UpdateUserProfile) -> Result<(), DomainError> {
            // Email Änderung
            if let Some(email) = &cmd.email {
                validate_email_format(email)?;
                validate_email_domain(email, &cmd.tenant_id)?;
                validate_email_change_cooldown(&self.state)?;
            }

            // Rollen-Änderung
            if let Some(role) = &cmd.role {
                validate_role_change(&self.state.role, role, &cmd.tenant_id)?;
            }

            // Display Name
            if let Some(name) = &cmd.display_name {
                validate_display_name(name)?;
            }

            Ok(())
        }

        #[deactivation_rules]
        fn validate_deactivation(&self, cmd: &DeactivateUser) -> Result<(), DomainError> {
            // Admin Deaktivierung
            if self.state.role == UserRole::Admin {
                validate_admin_deactivation(&cmd.tenant_id)?;
            }

            // Deaktivierungstyp
            match cmd.deactivation_type {
                DeactivationType::Permanent => {
                    validate_permanent_deactivation(&self.state)?;
                }
                DeactivationType::Temporary => {
                    if let Some(reactivation_date) = cmd.reactivation_date {
                        validate_reactivation_date(reactivation_date)?;
                    } else {
                        return Err(DomainError::MissingReactivationDate);
                    }
                }
            }

            Ok(())
        }
    }

    #[cross_aggregate_rules]
    struct CrossAggregateRules {
        #[tenant_user_rules]
        fn validate_cross_tenant_user_operation(
            tenant: &TenantAggregate,
            user: &UserAggregate,
            operation: &CrossTenantOperation
        ) -> Result<(), DomainError> {
            // Tenant Status Check
            if tenant.state.is_deleted {
                return Err(DomainError::TenantDeleted);
            }

            // User Zugehörigkeit
            if user.state.tenant_id != tenant.tenant_id {
                return Err(DomainError::UserTenantMismatch);
            }

            // Operation-spezifische Regeln
            match operation {
                CrossTenantOperation::UserTransfer { new_tenant_id } => {
                    validate_user_transfer(user, tenant, new_tenant_id)?;
                }
                CrossTenantOperation::RoleChange { new_role } => {
                    validate_cross_tenant_role_change(user, tenant, new_role)?;
                }
            }

            Ok(())
        }

        #[quota_rules]
        fn validate_tenant_quotas(
            tenant: &TenantAggregate,
            operation: &QuotaOperation
        ) -> Result<(), DomainError> {
            match operation {
                QuotaOperation::UserCreation => {
                    if get_current_user_count(&tenant.tenant_id) >= tenant.state.settings.max_users {
                        return Err(DomainError::UserQuotaExceeded);
                    }
                }
                QuotaOperation::StorageIncrease(size) => {
                    validate_storage_quota(tenant, *size)?;
                }
                QuotaOperation::ApiCallIncrease(count) => {
                    validate_api_quota(tenant, *count)?;
                }
            }

            Ok(())
        }
    }
}

### Error Types
```rust
#[derive(Error, Debug)]
enum DomainError {
    #[error("Tenant error: {0}")]
    TenantError(#[from] TenantError),

    #[error("User error: {0}")]
    UserError(#[from] UserError),

    #[error("Authentication error: {0}")]
    AuthError(#[from] AuthError),

    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationError),

    #[error("Concurrency error: {0}")]
    ConcurrencyError(#[from] ConcurrencyError),

    #[error("Infrastructure error: {0}")]
    InfrastructureError(#[from] InfrastructureError),
}

#[derive(Error, Debug)]
enum TenantError {
    #[error("Tenant not found: {0}")]
    NotFound(TenantId),

    #[error("Invalid tenant name: {0}")]
    InvalidTenantName(String),

    #[error("Invalid email domain: {0}")]
    InvalidEmailDomain(String),

    #[error("Invalid subscription plan: {0}")]
    InvalidSubscriptionPlan(String),

    #[error("Tenant is deleted")]
    TenantDeleted,

    #[error("Tenant has active users")]
    TenantHasActiveUsers,

    #[error("Active subscription exists")]
    ActiveSubscriptionExists,

    #[error("Invalid retention period")]
    InvalidRetentionPeriod,

    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),

    #[error("Invalid settings: {0}")]
    InvalidSettings(String),

    #[error("Settings transition not allowed: {from:?} -> {to:?}")]
    InvalidSettingsTransition { from: String, to: String },
}

#[derive(Error, Debug)]
enum UserError {
    #[error("User not found: {0}")]
    NotFound(UserId),

    #[error("Invalid email format: {0}")]
    InvalidEmailFormat(String),

    #[error("Email domain not allowed: {0}")]
    EmailDomainNotAllowed(String),

    #[error("User quota exceeded for tenant: {0}")]
    QuotaExceeded(TenantId),

    #[error("Invalid role: {0}")]
    InvalidRole(String),

    #[error("Role change not allowed: {from:?} -> {to:?}")]
    InvalidRoleChange { from: UserRole, to: UserRole },

    #[error("User is deactivated")]
    UserDeactivated,

    #[error("Invalid display name: {0}")]
    InvalidDisplayName(String),

    #[error("Email change not allowed: {reason}")]
    EmailChangeNotAllowed { reason: String },

    #[error("User belongs to different tenant")]
    TenantMismatch,

    #[error("Missing reactivation date for temporary deactivation")]
    MissingReactivationDate,
}

#[derive(Error, Debug)]
enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Account locked: {0}")]
    AccountLocked(String),

    #[error("Too many attempts: retry after {0}")]
    TooManyAttempts(DateTime<Utc>),

    #[error("Invalid authentication method: {0}")]
    InvalidAuthMethod(String),

    #[error("OAuth provider not supported: {0}")]
    UnsupportedOAuthProvider(String),

    #[error("SAML provider not supported: {0}")]
    UnsupportedSAMLProvider(String),

    #[error("Session expired")]
    SessionExpired,

    #[error("Session not found: {0}")]
    SessionNotFound(SessionId),

    #[error("Invalid token")]
    InvalidToken,
}

#[derive(Error, Debug)]
enum ValidationError {
    #[error("Required field missing: {0}")]
    RequiredFieldMissing(String),

    #[error("Field too long: {field} (max: {max}, actual: {actual})")]
    FieldTooLong {
        field: String,
        max: usize,
        actual: usize,
    },

    #[error("Field too short: {field} (min: {min}, actual: {actual})")]
    FieldTooShort {
        field: String,
        min: usize,
        actual: usize,
    },

    #[error("Invalid format: {field} ({reason})")]
    InvalidFormat {
        field: String,
        reason: String,
    },

    #[error("Value out of range: {field} (min: {min}, max: {max}, actual: {actual})")]
    ValueOutOfRange {
        field: String,
        min: String,
        max: String,
        actual: String,
    },
}

#[derive(Error, Debug)]
enum ConcurrencyError {
    #[error("Optimistic lock failure: expected version {expected}, got {actual}")]
    OptimisticLockFailure {
        expected: u64,
        actual: u64,
    },

    #[error("Aggregate already exists: {0}")]
    AlreadyExists(String),

    #[error("Conflict detected: {0}")]
    ConflictDetected(String),
}

#[derive(Error, Debug)]
enum InfrastructureError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("Event store error: {0}")]
    EventStore(#[from] EventStoreError),

    #[error("Cache error: {0}")]
    Cache(#[from] CacheError),

    #[error("Message bus error: {0}")]
    MessageBus(#[from] MessageBusError),

    #[error("External service error: {service} - {message}")]
    ExternalService {
        service: String,
        message: String,
    },

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Rate limit exceeded: retry after {0}")]
    RateLimitExceeded(DateTime<Utc>),
}

#[derive(Error, Debug)]
enum DatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Query failed: {0}")]
    QueryFailed(String),

    #[error("Deadlock detected")]
    DeadlockDetected,

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
}

#[derive(Error, Debug)]
enum EventStoreError {
    #[error("Stream not found: {0}")]
    StreamNotFound(String),

    #[error("Append failed: {0}")]
    AppendFailed(String),

    #[error("Read failed: {0}")]
    ReadFailed(String),

    #[error("Invalid event data: {0}")]
    InvalidEventData(String),
}

#[derive(Error, Debug)]
enum CacheError {
    #[error("Cache miss: {0}")]
    CacheMiss(String),

    #[error("Cache set failed: {0}")]
    SetFailed(String),

    #[error("Cache invalidation failed: {0}")]
    InvalidationFailed(String),
}

#[derive(Error, Debug)]
enum MessageBusError {
    #[error("Publish failed: {0}")]
    PublishFailed(String),

    #[error("Subscribe failed: {0}")]
    SubscribeFailed(String),

    #[error("Message handling failed: {0}")]
    HandlingFailed(String),
}

#[derive(ErrorHandling)]
struct ErrorHandlingConfig {
    #[retry(
        max_attempts = 3,
        backoff = "exponential"
    )]
    retry_policy: RetryPolicy,

    #[compensation(
        enabled = true,
        timeout = "30s"
    )]
    compensation: CompensationPolicy,

    #[logging(
        level = "error",
        include_context = true
    )]
    error_logging: ErrorLoggingConfig,

    #[monitoring(
        metrics = true,
        alerts = true
    )]
    error_monitoring: ErrorMonitoringConfig,
}

### Read Models
```rust
#[derive(ReadModels)]
struct QueryModels {
    #[tenant_views]
    struct TenantReadModels {
        #[view("tenant.details")]
        struct TenantDetailsView {
            #[key]
            tenant_id: TenantId,
            name: String,
            subscription_plan: SubscriptionPlan,
            admin_email: EmailAddress,
            settings: TenantSettings,
            status: TenantStatus,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
            #[index]
            is_active: bool,
        }

        #[view("tenant.metrics")]
        struct TenantMetricsView {
            #[key]
            tenant_id: TenantId,
            total_users: u32,
            active_users: u32,
            storage_used: u64,
            api_calls_current_month: u32,
            last_activity: DateTime<Utc>,
            #[index]
            subscription_status: SubscriptionStatus,
        }

        #[view("tenant.audit")]
        struct TenantAuditView {
            #[key]
            tenant_id: TenantId,
            #[index]
            event_type: String,
            #[index]
            timestamp: DateTime<Utc>,
            actor_id: UserId,
            changes: JsonValue,
            metadata: JsonValue,
        }
    }

    #[user_views]
    struct UserReadModels {
        #[view("user.profile")]
        struct UserProfileView {
            #[key]
            user_id: UserId,
            #[index]
            tenant_id: TenantId,
            email: EmailAddress,
            display_name: Option<String>,
            role: UserRole,
            status: UserStatus,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
            last_login: Option<DateTime<Utc>>,
            #[index]
            is_active: bool,
        }

        #[view("user.security")]
        struct UserSecurityView {
            #[key]
            user_id: UserId,
            #[index]
            tenant_id: TenantId,
            failed_login_attempts: u32,
            last_failed_attempt: Option<DateTime<Utc>>,
            password_changed_at: DateTime<Utc>,
            mfa_enabled: bool,
            allowed_auth_methods: Vec<AuthenticationMethod>,
            security_events: Vec<SecurityEvent>,
        }

        #[view("user.sessions")]
        struct UserSessionsView {
            #[key]
            user_id: UserId,
            #[index]
            tenant_id: TenantId,
            active_sessions: Vec<SessionInfo>,
            device_history: Vec<DeviceInfo>,
            last_active_session: Option<DateTime<Utc>>,
        }
    }

    #[auth_views]
    struct AuthReadModels {
        #[view("auth.sessions")]
        struct SessionView {
            #[key]
            session_id: SessionId,
            #[index]
            user_id: UserId,
            #[index]
            tenant_id: TenantId,
            created_at: DateTime<Utc>,
            expires_at: DateTime<Utc>,
            last_activity: DateTime<Utc>,
            device_info: DeviceInfo,
            ip_address: IpAddr,
            #[index]
            is_active: bool,
        }

        #[view("auth.audit")]
        struct AuthAuditView {
            #[key(composite = true)]
            #[index]
            user_id: UserId,
            #[index]
            tenant_id: TenantId,
            #[index]
            timestamp: DateTime<Utc>,
            event_type: AuthEventType,
            auth_method: AuthenticationMethod,
            device_info: DeviceInfo,
            ip_address: IpAddr,
            metadata: JsonValue,
        }
    }
}

#[derive(ProjectionHandlers)]
struct Projections {
    #[tenant_projections]
    impl TenantProjections {
        #[project("tenant.created")]
        fn project_tenant_created(&mut self, event: &TenantCreated) -> Result<(), ProjectionError> {
            let details = TenantDetailsView {
                tenant_id: event.tenant_id,
                name: event.name.clone(),
                subscription_plan: event.subscription_plan.clone(),
                admin_email: event.admin_email.clone(),
                settings: event.settings.clone(),
                status: TenantStatus::Active,
                created_at: event.metadata.timestamp,
                updated_at: event.metadata.timestamp,
                is_active: true,
            };
            self.tenant_details.insert(event.tenant_id, details)?;

            let metrics = TenantMetricsView {
                tenant_id: event.tenant_id,
                total_users: 1, // Admin User
                active_users: 1,
                storage_used: 0,
                api_calls_current_month: 0,
                last_activity: event.metadata.timestamp,
                subscription_status: SubscriptionStatus::Active,
            };
            self.tenant_metrics.insert(event.tenant_id, metrics)?;

            Ok(())
        }

        #[project("tenant.updated")]
        fn project_tenant_updated(&mut self, event: &TenantUpdated) -> Result<(), ProjectionError> {
            let mut details = self.tenant_details.get(event.tenant_id)?;
            
            if let Some(name) = &event.name {
                details.name = name.clone();
            }
            // ... similar for other fields
            
            details.updated_at = event.metadata.timestamp;
            self.tenant_details.update(event.tenant_id, details)?;

            Ok(())
        }
    }

    #[user_projections]
    impl UserProjections {
        #[project("user.registered")]
        fn project_user_registered(&mut self, event: &UserRegistered) -> Result<(), ProjectionError> {
            let profile = UserProfileView {
                user_id: event.user_id,
                tenant_id: event.tenant_id,
                email: event.email.clone(),
                display_name: event.display_name.clone(),
                role: event.role.clone(),
                status: UserStatus::Active,
                created_at: event.metadata.timestamp,
                updated_at: event.metadata.timestamp,
                last_login: None,
                is_active: true,
            };
            self.user_profiles.insert(event.user_id, profile)?;

            let security = UserSecurityView {
                user_id: event.user_id,
                tenant_id: event.tenant_id,
                failed_login_attempts: 0,
                last_failed_attempt: None,
                password_changed_at: event.metadata.timestamp,
                mfa_enabled: false,
                allowed_auth_methods: vec![AuthenticationMethod::Password],
                security_events: Vec::new(),
            };
            self.user_security.insert(event.user_id, security)?;

            // Update Tenant Metrics
            let mut metrics = self.tenant_metrics.get(event.tenant_id)?;
            metrics.total_users += 1;
            metrics.active_users += 1;
            self.tenant_metrics.update(event.tenant_id, metrics)?;

            Ok(())
        }
    }

    #[auth_projections]
    impl AuthProjections {
        #[project("auth.login_succeeded")]
        fn project_login_succeeded(&mut self, event: &LoginSucceeded) -> Result<(), ProjectionError> {
            let session = SessionView {
                session_id: event.session_id,
                user_id: event.user_id,
                tenant_id: event.tenant_id,
                created_at: event.metadata.timestamp,
                expires_at: event.metadata.timestamp + Duration::hours(24),
                last_activity: event.metadata.timestamp,
                device_info: event.device_info.clone(),
                ip_address: event.ip_address,
                is_active: true,
            };
            self.sessions.insert(event.session_id, session)?;

            let mut profile = self.user_profiles.get(event.user_id)?;
            profile.last_login = Some(event.metadata.timestamp);
            self.user_profiles.update(event.user_id, profile)?;

            Ok(())
        }
    }
}

#[derive(ReadModelQueries)]
struct Queries {
    #[tenant_queries]
    impl TenantQueries {
        #[query("get_tenant_details")]
        async fn get_tenant_details(
            &self,
            tenant_id: TenantId,
        ) -> Result<TenantDetailsView, QueryError> {
            self.tenant_details.get(tenant_id)
        }

        #[query("list_active_tenants")]
        async fn list_active_tenants(
            &self,
            pagination: Pagination,
        ) -> Result<Vec<TenantDetailsView>, QueryError> {
            self.tenant_details
                .find()
                .filter(|t| t.is_active)
                .sort_by(|t| t.created_at)
                .paginate(pagination)
                .execute()
        }
    }

    #[user_queries]
    impl UserQueries {
        #[query("get_user_profile")]
        async fn get_user_profile(
            &self,
            user_id: UserId,
            tenant_id: TenantId,
        ) -> Result<UserProfileView, QueryError> {
            let profile = self.user_profiles.get(user_id)?;
            if profile.tenant_id != tenant_id {
                return Err(QueryError::TenantMismatch);
            }
            Ok(profile)
        }

        #[query("list_tenant_users")]
        async fn list_tenant_users(
            &self,
            tenant_id: TenantId,
            filters: UserFilters,
            pagination: Pagination,
        ) -> Result<Vec<UserProfileView>, QueryError> {
            self.user_profiles
                .find()
                .filter(|u| u.tenant_id == tenant_id)
                .filter(|u| apply_user_filters(u, &filters))
                .sort_by(|u| u.created_at)
                .paginate(pagination)
                .execute()
        }
    }
}

#[derive(ReadModelStorage)]
struct StorageConfig {
    #[storage(
        type = "postgres",
        schema = "read_models"
    )]
    storage: PostgresStorage,

    #[cache(
        type = "redis",
        ttl = "15m"
    )]
    cache: RedisCache,

    #[indices(
        type = "btree",
        concurrent_build = true
    )]
    indexing: IndexConfig,
}

#[derive(ReadModelConsistency)]
struct ConsistencyConfig {
    #[catch_up(
        batch_size = 1000,
        parallel = true
    )]
    catch_up: CatchUpConfig,

    #[monitoring(
        lag_threshold = "5s",
        metrics = true
    )]
    monitoring: MonitoringConfig,

    #[error_handling(
        retry = true,
        max_attempts = 3
    )]
    error_handling: ErrorHandlingConfig,
}

### Command Architecture
```rust
#[derive(CommandSide)]
struct CommandArchitecture {
    #[handlers(validation = true)]
    command_handlers: Vec<Box<dyn CommandHandler>>,

    #[aggregates(
        versioning = true,
        concurrency = "optimistic"
    )]
    domain_aggregates: Vec<Box<dyn AggregateRoot>>,

    #[event_publishing(
        reliable = true,
        ordered = true
    )]
    event_dispatcher: EventDispatcher,
}
```

### Query Architecture
```rust
#[derive(QuerySide)]
struct QueryArchitecture {
    #[read_models(
        denormalized = true,
        cached = true
    )]
    query_models: Vec<Box<dyn ReadModel>>,

    #[projections(
        async = true,
        catchup = true
    )]
    projection_engine: ProjectionEngine,

    #[query_handlers(
        validation = true,
        metrics = true
    )]
    query_handlers: Vec<Box<dyn QueryHandler>>,
}
```

### Event Store Configuration
```rust
#[derive(EventStore)]
struct EventStoreConfig {
    #[storage(
        type = "EventStoreDB",
        clustering = true
    )]
    storage_config: StorageConfig,

    #[streams(
        partitioning = "by_aggregate",
        max_count = 1000
    )]
    stream_config: StreamConfig,

    #[snapshots(
        frequency = 100,
        compression = true
    )]
    snapshot_config: SnapshotConfig,
}
```

### Message Bus Architecture
```rust
#[derive(MessageBus)]
struct MessageBusConfig {
    #[command_bus(
        validation = true,
        middleware = ["logging", "metrics", "validation"]
    )]
    command_bus: CommandBus,

    #[query_bus(
        caching = true,
        middleware = ["logging", "metrics", "cache"]
    )]
    query_bus: QueryBus,

    #[event_bus(
        reliable = true,
        middleware = ["logging", "metrics", "retry"]
    )]
    event_bus: EventBus,
}
```

### Projection System
```rust
#[derive(Projections)]
struct ProjectionSystem {
    #[workers(
        count = 5,
        auto_scale = true
    )]
    projection_workers: ProjectionWorkers,

    #[checkpoints(
        persistent = true,
        interval = "1000"
    )]
    checkpoint_system: CheckpointSystem,

    #[catch_up(
        batch_size = 1000,
        parallel = true
    )]
    catch_up_config: CatchUpConfig,
}
```

### Data Consistency
```rust
#[derive(DataConsistency)]
struct ConsistencyConfig {
    #[eventual_consistency(
        max_lag = "5s",
        monitoring = true
    )]
    consistency_settings: ConsistencySettings,

    #[conflict_resolution(
        strategy = "last_write_wins",
        vector_clocks = true
    )]
    conflict_handling: ConflictResolution,
}
```

### Performance Optimization
```rust
#[derive(Performance)]
struct PerformanceConfig {
    #[read_optimization(
        caching = true,
        materialized_views = true
    )]
    read_performance: ReadOptimization,

    #[write_optimization(
        batch_size = 100,
        async_projections = true
    )]
    write_performance: WriteOptimization,
}
```

### Migration Strategy
```rust
#[derive(Migration)]
struct MigrationPlan {
    #[phases(
        order = ["event_store", "commands", "queries", "projections"]
    )]
    migration_phases: MigrationPhases,

    #[data_migration(
        strategy = "dual_write",
        validation = true
    )]
    data_migration: DataMigration,

    #[rollback(
        enabled = true,
        points = ["phase_end"]
    )]
    rollback_strategy: RollbackStrategy,
}
```

### Monitoring & Metrics
```rust
#[derive(CQRSMonitoring)]
struct CQRSMetrics {
    #[command_metrics(
        latency = true,
        throughput = true,
        error_rate = true
    )]
    command_monitoring: CommandMetrics,

    #[query_metrics(
        response_time = true,
        cache_hits = true,
        read_model_lag = true
    )]
    query_monitoring: QueryMetrics,

    #[event_metrics(
        processing_time = true,
        queue_size = true,
        failed_events = true
    )]
    event_monitoring: EventMetrics,
}
```

## Infrastructure

### Container Architecture
```dockerfile
# Multi-stage build example
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=builder /app/target/release/acci /
ENTRYPOINT ["/acci"]
```

### Identity Provider Architecture
```yaml
services:
  keycloak:
    image: quay.io/keycloak/keycloak:23.0
    environment:
      - KC_DB=postgres
      - KC_HOSTNAME_STRICT=false
      - KC_PROXY=edge
    volumes:
      - keycloak_themes:/opt/keycloak/themes
    deploy:
      replicas: 2
      update_config:
        parallelism: 1
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health/ready"]
      interval: 30s
      timeout: 10s
      retries: 3
```

### Platform Support
```rust
#[derive(PlatformSupport)]
struct SupportedPlatforms {
    #[platform(
        architecture = "amd64",
        os = ["linux", "windows", "macos"],
        tier = "tier1"
    )]
    amd64_support: Amd64Platform,

    #[platform(
        architecture = "ppc64le",
        os = ["linux"],
        tier = "tier2",
        requirements = [
            "libc6-dev-ppc64el-cross",
            "bindgen-cli",
            "aws-lc"
        ]
    )]
    ppc64le_support: Ppc64lePlatform,
}
```

### Service Mesh Requirements
```rust
#[derive(ServiceMesh)]
struct MeshConfig {
    #[discovery(
        automatic = true,
        health_check = true,
        dns_based = true
    )]
    service_discovery: DiscoveryConfig,

    #[traffic(
        circuit_breaker = true,
        retry_logic = true,
        rate_limiting = true
    )]
    traffic_management: TrafficConfig,
}
```

### Monitoring Stack
```yaml
monitoring:
  metrics:
    - type: RED
      implementation: prometheus
    - type: Business
      implementation: influxdb
  tracing:
    implementation: opentelemetry
    sampling_rate: 0.1
  logging:
    implementation: tracing
    format: json
```

### External Service Integration
```rust
#[derive(ExternalServices)]
struct IntegrationConfig {
    #[ldap(
        multi_server = true,
        failover = true,
        connection_timeout = "5s"
    )]
    ldap_config: LDAPConfig,

    #[smtp(
        providers = ["primary", "fallback"],
        retry_policy = "exponential",
        max_retries = 3
    )]
    smtp_config: SMTPConfig,

    #[legacy_node(
        protocol = "grpc",
        timeout = "10s",
        circuit_breaker = true
    )]
    legacy_integration: LegacyConfig,
}
```

## Security

### Authentication Flow
```rust
#[derive(Authentication)]
struct AuthFlow {
    #[auth_method("keycloak")]
    keycloak: KeycloakProvider,
    #[auth_method("oauth2")]
    oauth: OAuth2Provider,
    #[auth_method("ldap")]
    ldap: LDAPProvider,
    #[auth_method("local")]
    local: LocalAuth,

    #[keycloak_config(
        realm = "acci",
        client_id = "acci-backend",
        public_key_cache_ttl = "1h",
        token_validation = true
    )]
    keycloak_settings: KeycloakConfig,
}
```

### Authorization Flow
```rust
#[derive(Authorization)]
struct AuthorizationFlow {
    #[auth_engine("oso")]
    policy_engine: OsoEngine,
    
    #[policy_location("policies/")]
    policy_files: PolicyFiles,
    
    #[policy_reload(
        watch = true,
        interval = "30s"
    )]
    policy_reload: PolicyReload,
}
```

### Multi-Tenancy Security
```rust
#[derive(TenantSecurity)]
struct TenantIsolation {
    #[tenant_boundary]
    data_isolation: DatabaseIsolation,
    #[tenant_boundary]
    api_isolation: APIIsolation,
    #[tenant_boundary]
    storage_isolation: StorageIsolation,
    
    #[keycloak_realm(
        per_tenant = true,
        naming = "tenant_{id}"
    )]
    realm_isolation: RealmIsolation,
    
    #[oso_policies(
        tenant_scoped = true,
        inheritance = true
    )]
    policy_isolation: PolicyIsolation,
}
```

### Policy Management
```rust
#[derive(PolicyManagement)]
struct PolicyConfig {
    #[policy_types(
        rbac = true,
        abac = true,
        resource_based = true
    )]
    policy_types: PolicyTypes,

    #[policy_inheritance(
        global = true,
        tenant = true,
        role = true
    )]
    inheritance: PolicyInheritance,

    #[policy_validation(
        syntax = true,
        conflicts = true,
        coverage = true
    )]
    validation: PolicyValidation,
}
```

### Audit Logging
```rust
#[derive(AuditLog)]
struct AuditConfig {
    #[audit_level(
        security = "all",
        data_access = "write",
        system = "critical"
    )]
    logging_policy: LoggingPolicy,

    #[retention(
        security = "7y",
        data_access = "1y",
        system = "90d"
    )]
    retention_policy: RetentionPolicy,
}
```

## Development Guidelines

### Code Organization
```
src/
├── api/          # API layer (REST & GraphQL)
├── domain/       # Business logic
├── infrastructure/ # External services
├── policies/     # Oso policy files
│   ├── global/   # Global policies
│   ├── rbac/     # Role-based policies
│   └── tenant/   # Tenant-specific policies
└── common/       # Shared utilities
```

### Policy Development
```polar
# Example Oso policy file (policies/rbac/resource_access.polar)
allow(actor: User, action, resource) if
    has_role(actor, "admin") and
    actor.tenant_id = resource.tenant_id;

allow(actor: User, "read", resource: Document) if
    has_role(actor, "reader") and
    actor.tenant_id = resource.tenant_id and
    resource.public = true;

# Tenant isolation
allow(actor: User, _, resource) if
    actor.tenant_id = resource.tenant_id and
    not resource.restricted;
```

### Error Handling
```rust
#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Resource not found: {0}")]
    NotFound(ResourceId),
    #[error("Access denied: {0}")]
    AccessDenied(String),
    #[error("Validation failed: {0}")]
    ValidationError(String),
}
```

### Logging Standards
```rust
#[derive(Logging)]
struct LoggingConfig {
    #[level(
        production = "INFO",
        development = "DEBUG"
    )]
    log_levels: LogLevels,

    #[format(
        production = "json",
        development = "pretty"
    )]
    log_format: LogFormat,

    #[fields(
        always = ["request_id", "tenant_id", "user_id"],
        never = ["password", "token"]
    )]
    context_fields: ContextFields,
}
```

## Quality Assurance

### Testing Strategy
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_multi_tenant_isolation() {
        let tenant1 = TestTenant::new();
        let tenant2 = TestTenant::new();
        
        // Ensure data isolation
        assert!(tenant1.cannot_access_data_of(tenant2));
    }
}
```

### Performance Requirements
```rust
#[benchmark]
async fn api_response_times() {
    // API endpoints: < 100ms for 95th percentile
    // GraphQL queries: < 200ms for 95th percentile
    // Database queries: < 50ms for 95th percentile
}
```

### Security Testing
```rust
#[derive(SecurityTest)]
struct SecurityTestConfig {
    #[scan(type = "dependency")]
    #[threshold(critical = 0, high = 0)]
    dependency_check: DependencyScan,

    #[scan(type = "sast")]
    #[tools("clippy", "cargo-audit")]
    static_analysis: StaticAnalysis,

    #[scan(type = "dast")]
    #[tools("zap", "nuclei")]
    dynamic_analysis: DynamicAnalysis,
}
```

## Operations

### Deployment Process
```yaml
deployment:
  strategy: rolling
  healthcheck:
    path: /health
    interval: 30s
  rollback:
    automatic: true
    threshold: 25%
```

### Monitoring & Alerting
```rust
#[derive(Monitoring)]
struct MonitoringConfig {
    #[alert(threshold = "p95 > 100ms")]
    api_latency: Histogram,
    #[alert(threshold = "rate > 1%")]
    error_rate: Counter,
    #[alert(threshold = "memory > 85%")]
    resource_usage: Gauge,
}
```

### Health Checks
```rust
#[derive(HealthCheck)]
struct HealthConfig {
    #[check(
        path = "/health",
        interval = "30s",
        timeout = "5s"
    )]
    health_endpoint: HealthEndpoint,

    #[dependencies(
        database = true,
        cache = true,
        external = true
    )]
    dependency_checks: DependencyHealth,
}
```

## Internationalization

### Translation System
```rust
#[derive(I18n)]
struct I18nConfig {
    #[primary_language("en")]
    #[supported_languages("de", "sq", "fr", "es")]
    language_config: LanguageConfig,

    #[translation_path("i18n/{lang}/")]
    translation_files: TranslationFiles,

    #[fallback_language("en")]
    fallback_config: FallbackConfig,

    #[thread_safety(
        memoization = true,
        concurrent_access = true
    )]
    safety_config: ThreadSafetyConfig,

    #[middleware(
        query_param = "lang",
        header = "Accept-Language",
        cookie = "preferred_language"
    )]
    detection_config: LanguageDetectionConfig,
}
```

### Message Categories
```rust
#[derive(MessageCategories)]
struct TranslationCategories {
    #[category("errors")]
    error_messages: ErrorMessages,
    
    #[category("system")]
    system_messages: SystemMessages,
    
    #[category("validation")]
    validation_messages: ValidationMessages,
    
    #[category("ui")]
    ui_messages: UIMessages,
    
    #[category("emails")]
    email_templates: EmailTemplates,
}
```

### CI/CD Integration
```rust
#[derive(CIPipeline)]
struct CIConfig {
    #[provider(
        github = true,
        gitlab = true
    )]
    ci_providers: CIProviders,

    #[security(
        sbom = true,
        container_scan = true,
        dependency_audit = true,
        cosign = true
    )]
    security_checks: SecurityConfig,

    #[testing(
        unit = true,
        integration = true,
        coverage = true,
        msrv = "1.75"
    )]
    test_config: TestingConfig,
}
```

### Message Format
```ftl
# Component title
app-title = Application Title

# User count with plural support
user-count = { $count ->
    [one] 1 user online
    *[other] { $count } users online
}

# Button labels
button-save = Save
button-cancel = Cancel
```

### Translation Verification
```rust
#[derive(TranslationVerification)]
struct VerificationConfig {
    #[coverage(
        minimum = 100,
        check_in_ci = true
    )]
    coverage_requirements: CoverageConfig,

    #[review(
        technical = true,
        linguistic = true
    )]
    review_process: ReviewConfig,
}
```

### Infrastructure

```rust
#[derive(Infrastructure)]
struct InfrastructureComponents {
    #[event_store(
        product = "EventStoreDB",
        version = "23.10",
        clustering = true,
        config = {
            projections = true,
            snapshot_interval = 100,
            max_append_size = "1MB",
            tcp_port = 1113,
            http_port = 2113
        }
    )]
    event_store: EventStoreConfig,

    #[message_broker(
        product = "RabbitMQ",
        version = "3.12",
        config = {
            connection_pool = 32,
            channels_per_connection = 10,
            prefetch_count = 100,
            persistent = true,
            dead_letter = true
        }
    )]
    message_broker: MessageBrokerConfig,

    #[cache(
        product = "Redis",
        version = "7.2",
        config = {
            connection_pool = 32,
            max_memory = "2GB",
            eviction_policy = "volatile-lru"
        }
    )]
    cache: CacheConfig,
}

#[derive(Deployment)]
struct DeploymentConfig {
    #[docker_compose]
    struct ComposeServices {
        #[service("eventstoredb")]
        event_store: EventStoreService {
            image: "eventstore/eventstore:23.10",
            environment: {
                EVENTSTORE_CLUSTER_SIZE: "1",
                EVENTSTORE_RUN_PROJECTIONS: "All",
                EVENTSTORE_START_STANDARD_PROJECTIONS: "true",
                EVENTSTORE_EXT_TCP_PORT: "1113",
                EVENTSTORE_HTTP_PORT: "2113",
                EVENTSTORE_INSECURE: "true",
                EVENTSTORE_ENABLE_EXTERNAL_TCP: "true",
                EVENTSTORE_ENABLE_ATOM_PUB_OVER_HTTP: "true"
            },
            volumes: ["eventstore-volume:/var/lib/eventstore"],
            healthcheck: {
                test: ["CMD-SHELL", "curl -f http://localhost:2113/health/live || exit 1"],
                interval: "30s",
                timeout: "10s",
                retries: 3
            }
        },

        #[service("rabbitmq")]
        message_broker: RabbitMQService {
            image: "rabbitmq:3.12-management",
            environment: {
                RABBITMQ_DEFAULT_USER: "acci",
                RABBITMQ_DEFAULT_PASS: "${RABBITMQ_PASSWORD}",
                RABBITMQ_ERLANG_COOKIE: "${RABBITMQ_COOKIE}"
            },
            volumes: [
                "rabbitmq-data:/var/lib/rabbitmq",
                "rabbitmq-config:/etc/rabbitmq"
            ],
            healthcheck: {
                test: ["CMD", "rabbitmq-diagnostics", "check_port_connectivity"],
                interval: "30s",
                timeout: "10s",
                retries: 3
            }
        }
    }
}

#[derive(MessageBusArchitecture)]
struct MessageBusConfig {
    #[exchanges]
    struct Exchanges {
        #[exchange("events")]
        event_exchange: Exchange {
            type: "topic",
            durable: true,
            auto_delete: false
        },

        #[exchange("commands")]
        command_exchange: Exchange {
            type: "direct",
            durable: true,
            auto_delete: false
        },

        #[exchange("dead_letters")]
        dead_letter_exchange: Exchange {
            type: "fanout",
            durable: true,
            auto_delete: false
        }
    }

    #[queues]
    struct Queues {
        #[queue("event.processing")]
        event_processing: Queue {
            durable: true,
            auto_delete: false,
            dead_letter_exchange: "dead_letters",
            message_ttl: "24h",
            max_length: 100000
        },

        #[queue("command.processing")]
        command_processing: Queue {
            durable: true,
            auto_delete: false,
            dead_letter_exchange: "dead_letters",
            message_ttl: "1h",
            max_length: 10000
        }
    }

    #[connection_pool]
    struct ConnectionPool {
        min_size: 5,
        max_size: 32,
        max_lifetime: "1h",
        idle_timeout: "10m",
        connection_timeout: "30s"
    }
}
```
