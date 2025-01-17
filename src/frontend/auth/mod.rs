use leptos::*;
use serde::{Deserialize, Serialize};
use web_sys::Storage;

mod components;
pub use components::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeycloakConfig {
    pub url: String,
    pub realm: String,
    pub client_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub expires_in: i32,
    pub refresh_expires_in: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct AuthState {
    pub config: KeycloakConfig,
    pub storage: Storage,
}

impl AuthState {
    pub fn new(config: KeycloakConfig) -> Result<Self, String> {
        let window = web_sys::window().ok_or("No window found")?;
        let storage = window
            .local_storage()
            .map_err(|e| e.to_string())?
            .ok_or("No storage found")?;

        Ok(Self { config, storage })
    }

    pub fn get_token(&self) -> Option<String> {
        self.storage.get_item("token").ok()?
    }

    pub fn set_token(&self, token: &str) -> Result<(), String> {
        self.storage
            .set_item("token", token)
            .map_err(|e| e.to_string())
    }

    pub fn clear_token(&self) -> Result<(), String> {
        self.storage.remove_item("token").map_err(|e| e.to_string())
    }

    pub fn login(&self) {
        let redirect_uri = web_sys::window().unwrap().location().origin().unwrap();

        let auth_url = format!(
            "{}/auth/realms/{}/protocol/openid-connect/auth?client_id={}&redirect_uri={}&response_type=code&scope=openid",
            self.config.url,
            self.config.realm,
            self.config.client_id,
            redirect_uri
        );

        web_sys::window()
            .unwrap()
            .location()
            .set_href(&auth_url)
            .unwrap();
    }

    pub fn logout(&self) -> Result<(), String> {
        self.clear_token()?;

        let redirect_uri = web_sys::window().unwrap().location().origin().unwrap();

        let logout_url = format!(
            "{}/auth/realms/{}/protocol/openid-connect/logout?redirect_uri={}",
            self.config.url, self.config.realm, redirect_uri
        );

        web_sys::window()
            .unwrap()
            .location()
            .set_href(&logout_url)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn handle_callback(&self, code: &str) -> Result<(), String> {
        let redirect_uri = web_sys::window().unwrap().location().origin().unwrap();

        let token_url = format!(
            "{}/auth/realms/{}/protocol/openid-connect/token",
            self.config.url, self.config.realm
        );

        let form_data = web_sys::FormData::new().unwrap();
        form_data
            .append_with_str("grant_type", "authorization_code")
            .unwrap();
        form_data
            .append_with_str("client_id", &self.config.client_id)
            .unwrap();
        form_data.append_with_str("code", code).unwrap();
        form_data
            .append_with_str("redirect_uri", &redirect_uri)
            .unwrap();

        let resp = gloo_net::http::Request::post(&token_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(&form_data)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let token_response: TokenResponse = resp.json().await.map_err(|e| e.to_string())?;
        self.set_token(&token_response.access_token)?;

        Ok(())
    }

    pub fn is_authenticated(&self) -> bool {
        self.get_token().is_some()
    }
}
