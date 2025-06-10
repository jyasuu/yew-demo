// src/auth/github.rs
use serde::Deserialize;
use gloo_storage::{SessionStorage, Storage};
use crate::config::Config;

#[derive(Deserialize, Debug)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

#[derive(Deserialize, Debug)]
pub struct GitHubUser {
    pub login: String,
    pub avatar_url: String,
    pub name: String,
}

pub fn initiate_login() {
    let (code_verifier, code_challenge) = super::pkce::generate_pkce_pair();
    
    SessionStorage::set("pkce_code_verifier", &code_verifier)
        .expect("Failed to store code verifier");
    
    let config_str = include_str!("../../config.json");
    let config: Config = serde_json::from_str(config_str).expect("Failed to parse config.json");

    let auth_url = format!(
        "https://github.com/login/oauth/authorize?\
         client_id={}&\
         redirect_uri={}&\
         scope=user&\
         state=STATE&\
         code_challenge={}&\
         code_challenge_method=S256",
        config.github_auth.client_id, config.github_auth.redirect_uri, code_challenge
    );
    
    gloo_utils::window().location().set_href(&auth_url).unwrap();
}

pub async fn exchange_code(code: String) -> Result<TokenResponse, String> {
    let config_str = include_str!("../../config.json");
    let config: Config = serde_json::from_str(config_str).expect("Failed to parse config.json");

    let code_verifier: String = SessionStorage::get("pkce_code_verifier")
        .map_err(|_| "Missing code verifier".to_string())?;
    
    let params = [
        ("client_id", config.github_auth.client_id.as_str()),
        ("client_secret", config.github_auth.client_secret.as_str()),
        ("code", &code),
        ("redirect_uri", config.github_auth.redirect_uri.as_str()),
        ("code_verifier", &code_verifier),
    ];
    
    let client = reqwest::Client::new();
    let response = client.post(&config.github_auth.token_url)
        .header("Accept", "application/json")
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    response.json::<TokenResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn get_user(access_token: &str) -> Result<GitHubUser, String> {
    let config_str = include_str!("../../config.json");
    let config: Config = serde_json::from_str(config_str).expect("Failed to parse config.json");

    let client = reqwest::Client::new();
    let response = client.get(&config.github_auth.user_api_url)
        .header("User-Agent", "reqwest")
        .header("Authorization", format!("token {}", access_token))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    response.json::<GitHubUser>()
        .await
        .map_err(|e| e.to_string())
}
