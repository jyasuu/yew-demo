// src/auth/pkce.rs
use rand::Rng;
use sha2::{Sha256, Digest};
use base64::engine::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

pub fn generate_pkce_pair() -> (String, String) {
    let code_verifier = generate_code_verifier();
    let code_challenge = generate_code_challenge(&code_verifier);
    (code_verifier, code_challenge)
}

fn generate_code_verifier() -> String {
    let random_bytes: Vec<u8> = (0..32).map(|_| rand::rng().random()).collect();
    URL_SAFE_NO_PAD.encode(random_bytes)
}

fn generate_code_challenge(code_verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(hasher.finalize())
}