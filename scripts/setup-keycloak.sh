#!/bin/bash
set -e

# Default values
KEYCLOAK_URL=${KEYCLOAK_URL:-"http://localhost:8080"}
KEYCLOAK_REALM=${KEYCLOAK_REALM:-"acci"}
KEYCLOAK_CLIENT_ID=${KEYCLOAK_CLIENT_ID:-"acci-backend"}
KEYCLOAK_ADMIN=${KEYCLOAK_ADMIN:-"admin"}
KEYCLOAK_ADMIN_PASSWORD=${KEYCLOAK_ADMIN_PASSWORD:-"admin"}

# Get admin token
echo "Getting admin token..."
ADMIN_TOKEN=$(curl -s -X POST \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "username=${KEYCLOAK_ADMIN}" \
    -d "password=${KEYCLOAK_ADMIN_PASSWORD}" \
    -d "grant_type=password" \
    -d "client_id=admin-cli" \
    "${KEYCLOAK_URL}/auth/realms/master/protocol/openid-connect/token" | jq -r '.access_token')

if [ -z "$ADMIN_TOKEN" ] || [ "$ADMIN_TOKEN" = "null" ]; then
    echo "Failed to get admin token"
    exit 1
fi

# Create realm
echo "Creating realm..."
curl -s -X POST \
    -H "Authorization: Bearer ${ADMIN_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{
        "realm": "'${KEYCLOAK_REALM}'",
        "enabled": true,
        "displayName": "ACCI Platform",
        "displayNameHtml": "<div class=\"kc-logo-text\"><span>ACCI Platform</span></div>",
        "bruteForceProtected": true,
        "permanentLockout": false,
        "failureFactor": 3,
        "waitIncrementSeconds": 60,
        "quickLoginCheckMilliSeconds": 1000,
        "minimumQuickLoginWaitSeconds": 60,
        "maxFailureWaitSeconds": 900,
        "maxDeltaTimeSeconds": 43200,
        "accessTokenLifespan": 300,
        "accessTokenLifespanForImplicitFlow": 900,
        "ssoSessionIdleTimeout": 1800,
        "ssoSessionMaxLifespan": 36000,
        "offlineSessionIdleTimeout": 2592000,
        "accessCodeLifespan": 60,
        "accessCodeLifespanUserAction": 300,
        "accessCodeLifespanLogin": 1800,
        "sslRequired": "EXTERNAL",
        "registrationAllowed": false,
        "registrationEmailAsUsername": true,
        "rememberMe": true,
        "verifyEmail": true,
        "loginWithEmailAllowed": true,
        "duplicateEmailsAllowed": false,
        "resetPasswordAllowed": true,
        "editUsernameAllowed": false,
        "revokeRefreshToken": true,
        "refreshTokenMaxReuse": 0,
        "defaultSignatureAlgorithm": "RS256",
        "defaultLocale": "en"
    }' \
    "${KEYCLOAK_URL}/auth/admin/realms" || true

# Wait for realm to be ready
sleep 5

# Create client
echo "Creating client..."
CLIENT_ID=$(curl -s -X POST \
    -H "Authorization: Bearer ${ADMIN_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{
        "clientId": "'${KEYCLOAK_CLIENT_ID}'",
        "enabled": true,
        "clientAuthenticatorType": "client-secret",
        "redirectUris": ["*"],
        "webOrigins": ["*"],
        "standardFlowEnabled": true,
        "implicitFlowEnabled": false,
        "directAccessGrantsEnabled": true,
        "serviceAccountsEnabled": true,
        "publicClient": false,
        "frontchannelLogout": true,
        "protocol": "openid-connect",
        "attributes": {
            "saml.assertion.signature": "false",
            "access.token.lifespan": "300",
            "saml.force.post.binding": "false",
            "saml.multivalued.roles": "false",
            "saml.encrypt": "false",
            "backchannel.logout.revoke.offline.tokens": "false",
            "saml.server.signature": "false",
            "saml.server.signature.keyinfo.ext": "false",
            "exclude.session.state.from.auth.response": "false",
            "backchannel.logout.session.required": "false",
            "client_credentials.use_refresh_token": "false",
            "saml_force_name_id_format": "false",
            "saml.client.signature": "false",
            "tls.client.certificate.bound.access.tokens": "false",
            "saml.authnstatement": "false",
            "display.on.consent.screen": "false",
            "saml.onetimeuse.condition": "false"
        },
        "authenticationFlowBindingOverrides": {},
        "fullScopeAllowed": true,
        "nodeReRegistrationTimeout": -1,
        "protocolMappers": [
            {
                "name": "tenant",
                "protocol": "openid-connect",
                "protocolMapper": "oidc-usermodel-attribute-mapper",
                "consentRequired": false,
                "config": {
                    "userinfo.token.claim": "true",
                    "user.attribute": "tenant",
                    "id.token.claim": "true",
                    "access.token.claim": "true",
                    "claim.name": "tenant",
                    "jsonType.label": "String"
                }
            },
            {
                "name": "roles",
                "protocol": "openid-connect",
                "protocolMapper": "oidc-usermodel-realm-role-mapper",
                "consentRequired": false,
                "config": {
                    "multivalued": "true",
                    "userinfo.token.claim": "true",
                    "id.token.claim": "true",
                    "access.token.claim": "true",
                    "claim.name": "roles",
                    "jsonType.label": "String"
                }
            }
        ]
    }' \
    "${KEYCLOAK_URL}/auth/admin/realms/${KEYCLOAK_REALM}/clients" | jq -r '.id')

# Wait for client to be ready
sleep 5

# Create roles
echo "Creating roles..."
ROLES=("user" "admin" "tenant_admin")
for ROLE in "${ROLES[@]}"; do
    curl -s -X POST \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        -H "Content-Type: application/json" \
        -d '{
            "name": "'${ROLE}'",
            "composite": false,
            "clientRole": false,
            "containerId": "'${KEYCLOAK_REALM}'"
        }' \
        "${KEYCLOAK_URL}/auth/admin/realms/${KEYCLOAK_REALM}/roles" || true
done

# Wait for roles to be ready
sleep 5

# Create default groups
echo "Creating default groups..."
GROUPS=("users" "admins" "tenant_admins")
for GROUP in "${GROUPS[@]}"; do
    # Create group
    GROUP_RESPONSE=$(curl -s -X POST \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        -H "Content-Type: application/json" \
        -d '{
            "name": "'${GROUP}'"
        }' \
        "${KEYCLOAK_URL}/auth/admin/realms/${KEYCLOAK_REALM}/groups")
    
    # Get group ID from location header
    GROUP_ID=$(curl -s -X GET \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        "${KEYCLOAK_URL}/auth/admin/realms/${KEYCLOAK_REALM}/groups" | \
        jq -r '.[] | select(.name=="'${GROUP}'") | .id')
    
    if [ -z "$GROUP_ID" ] || [ "$GROUP_ID" = "null" ]; then
        echo "Failed to get group ID for ${GROUP}"
        continue
    fi
    
    # Get role ID based on group name
    case ${GROUP} in
        "users")
            ROLE="user"
            ;;
        "admins")
            ROLE="admin"
            ;;
        "tenant_admins")
            ROLE="tenant_admin"
            ;;
    esac
    
    ROLE_ID=$(curl -s -X GET \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        "${KEYCLOAK_URL}/auth/admin/realms/${KEYCLOAK_REALM}/roles" | \
        jq -r '.[] | select(.name=="'${ROLE}'") | .id')
    
    if [ -z "$ROLE_ID" ] || [ "$ROLE_ID" = "null" ]; then
        echo "Failed to get role ID for ${ROLE}"
        continue
    fi
    
    # Assign role to group
    curl -s -X POST \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        -H "Content-Type: application/json" \
        -d '[{"id": "'${ROLE_ID}'", "name": "'${ROLE}'"}]' \
        "${KEYCLOAK_URL}/auth/admin/realms/${KEYCLOAK_REALM}/groups/${GROUP_ID}/role-mappings/realm"
done

# Wait for groups to be ready
sleep 5

# Create test users
echo "Creating test users..."
# Admin user
curl -s -X POST \
    -H "Authorization: Bearer ${ADMIN_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{
        "username": "admin",
        "enabled": true,
        "emailVerified": true,
        "firstName": "Admin",
        "lastName": "User",
        "email": "admin@acci.io",
        "credentials": [{
            "type": "password",
            "value": "admin",
            "temporary": false
        }]
    }' \
    "${KEYCLOAK_URL}/auth/admin/realms/${KEYCLOAK_REALM}/users" || true

# Get admin user ID
ADMIN_USER_ID=$(curl -s -X GET \
    -H "Authorization: Bearer ${ADMIN_TOKEN}" \
    "${KEYCLOAK_URL}/auth/admin/realms/${KEYCLOAK_REALM}/users" | \
    jq -r '.[] | select(.username=="admin") | .id')

if [ ! -z "$ADMIN_USER_ID" ] && [ "$ADMIN_USER_ID" != "null" ]; then
    # Get admins group ID
    ADMIN_GROUP_ID=$(curl -s -X GET \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        "${KEYCLOAK_URL}/auth/admin/realms/${KEYCLOAK_REALM}/groups" | \
        jq -r '.[] | select(.name=="admins") | .id')
    
    if [ ! -z "$ADMIN_GROUP_ID" ] && [ "$ADMIN_GROUP_ID" != "null" ]; then
        # Join admin to admins group
        curl -s -X PUT \
            -H "Authorization: Bearer ${ADMIN_TOKEN}" \
            "${KEYCLOAK_URL}/auth/admin/realms/${KEYCLOAK_REALM}/users/${ADMIN_USER_ID}/groups/${ADMIN_GROUP_ID}"
    fi
fi

# Regular user
curl -s -X POST \
    -H "Authorization: Bearer ${ADMIN_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{
        "username": "user",
        "enabled": true,
        "emailVerified": true,
        "firstName": "Regular",
        "lastName": "User",
        "email": "user@acci.io",
        "credentials": [{
            "type": "password",
            "value": "user",
            "temporary": false
        }]
    }' \
    "${KEYCLOAK_URL}/auth/admin/realms/${KEYCLOAK_REALM}/users" || true

# Get user ID
USER_ID=$(curl -s -X GET \
    -H "Authorization: Bearer ${ADMIN_TOKEN}" \
    "${KEYCLOAK_URL}/auth/admin/realms/${KEYCLOAK_REALM}/users" | \
    jq -r '.[] | select(.username=="user") | .id')

if [ ! -z "$USER_ID" ] && [ "$USER_ID" != "null" ]; then
    # Get users group ID
    USERS_GROUP_ID=$(curl -s -X GET \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        "${KEYCLOAK_URL}/auth/admin/realms/${KEYCLOAK_REALM}/groups" | \
        jq -r '.[] | select(.name=="users") | .id')
    
    if [ ! -z "$USERS_GROUP_ID" ] && [ "$USERS_GROUP_ID" != "null" ]; then
        # Join user to users group
        curl -s -X PUT \
            -H "Authorization: Bearer ${ADMIN_TOKEN}" \
            "${KEYCLOAK_URL}/auth/admin/realms/${KEYCLOAK_REALM}/users/${USER_ID}/groups/${USERS_GROUP_ID}"
    fi
fi

# Tenant admin user
curl -s -X POST \
    -H "Authorization: Bearer ${ADMIN_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{
        "username": "tenant_admin",
        "enabled": true,
        "emailVerified": true,
        "firstName": "Tenant",
        "lastName": "Admin",
        "email": "tenant_admin@acci.io",
        "credentials": [{
            "type": "password",
            "value": "tenant_admin",
            "temporary": false
        }],
        "attributes": {
            "tenant": ["default_tenant"]
        }
    }' \
    "${KEYCLOAK_URL}/auth/admin/realms/${KEYCLOAK_REALM}/users" || true

# Get tenant admin user ID
TENANT_ADMIN_USER_ID=$(curl -s -X GET \
    -H "Authorization: Bearer ${ADMIN_TOKEN}" \
    "${KEYCLOAK_URL}/auth/admin/realms/${KEYCLOAK_REALM}/users" | \
    jq -r '.[] | select(.username=="tenant_admin") | .id')

if [ ! -z "$TENANT_ADMIN_USER_ID" ] && [ "$TENANT_ADMIN_USER_ID" != "null" ]; then
    # Get tenant_admins group ID
    TENANT_ADMINS_GROUP_ID=$(curl -s -X GET \
        -H "Authorization: Bearer ${ADMIN_TOKEN}" \
        "${KEYCLOAK_URL}/auth/admin/realms/${KEYCLOAK_REALM}/groups" | \
        jq -r '.[] | select(.name=="tenant_admins") | .id')
    
    if [ ! -z "$TENANT_ADMINS_GROUP_ID" ] && [ "$TENANT_ADMINS_GROUP_ID" != "null" ]; then
        # Join tenant_admin to tenant_admins group
        curl -s -X PUT \
            -H "Authorization: Bearer ${ADMIN_TOKEN}" \
            "${KEYCLOAK_URL}/auth/admin/realms/${KEYCLOAK_REALM}/users/${TENANT_ADMIN_USER_ID}/groups/${TENANT_ADMINS_GROUP_ID}"
    fi
fi

echo "Keycloak setup completed successfully!" 