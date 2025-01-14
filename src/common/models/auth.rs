#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub auth_time: usize,
    pub jti: String,
    pub iss: String,
    pub aud: String,
    pub typ: String,
    pub azp: String,
    pub session_state: String,
    pub acr: String,
    pub realm_access: Option<RealmAccess>,
    pub resource_access: Option<HashMap<String, ResourceAccess>>,
    pub scope: String,
    pub sid: String,
    pub email_verified: Option<bool>,
    pub preferred_username: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RealmAccess {
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceAccess {
    pub roles: Vec<String>,
}
