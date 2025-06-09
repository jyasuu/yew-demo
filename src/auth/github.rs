// src/auth/github.rs
use serde::Deserialize;
use gloo_storage::{SessionStorage, Storage};

const CLIENT_ID: &str = "Ov23liACWRscsUseORai";
const REDIRECT_URI: &str = "https://8080-jyasuu-yewdemo-k57lkcdb36j.ws-us120.gitpod.io/callback";


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
    
    let auth_url = format!(
        "https://github.com/login/oauth/authorize?\
         client_id={}&\
         redirect_uri={}&\
         scope=user&\
         state=STATE&\
         code_challenge={}&\
         code_challenge_method=S256",
        CLIENT_ID, REDIRECT_URI, code_challenge
    );
    
    gloo_utils::window().location().set_href(&auth_url).unwrap();
}

pub async fn exchange_code(code: String) -> Result<TokenResponse, String> {
    let code_verifier: String = SessionStorage::get("pkce_code_verifier")
        .map_err(|_| "Missing code verifier".to_string())?;
    
    let token_url = "https://github.com/login/oauth/access_token";
    let params = [
        ("client_id", CLIENT_ID),
        ("client_secret", ""),
        ("code", &code),
        ("redirect_uri", REDIRECT_URI),
        ("code_verifier", &code_verifier),
    ];
    
    let client = reqwest::Client::new();
    let response = client.post(token_url)
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
    let client = reqwest::Client::new();
    let response = client.get("https://api.github.com/user")
        .header("User-Agent", "reqwest")
        .header("Authorization", format!("token {}", access_token))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    response.json::<GitHubUser>()
        .await
        .map_err(|e| e.to_string())
}