// src/components/callback.rs
use yew::prelude::*;
use yew_router::prelude::*;
use crate::auth::github;
use crate::Route;
use gloo_storage::{LocalStorage, Storage};
use wasm_bindgen_futures::spawn_local;

#[function_component(Callback)]
pub fn callback() -> Html {
    let navigator = use_navigator().unwrap();
    let location = gloo_utils::window().location();
    let query = location.search().unwrap();
    
    use_effect_with((),move |_| {
        spawn_local(async move {
            let params = web_sys::UrlSearchParams::new_with_str(&query).unwrap();
            let code = params.get("code").unwrap_or_default();
            
            if code.is_empty() {
                navigator.push(&Route::Login);
                return;
            }
            
            match github::exchange_code(code).await {
                Ok(token) => {
                    // Store token in local storage
                    LocalStorage::set("github_access_token", token.access_token.clone())
                        .expect("Failed to store access token");
                    
                    // Redirect to home
                    navigator.push(&Route::Home);
                }
                Err(err) => {
                    gloo_console::error!("Token exchange failed:", err);
                    navigator.push(&Route::Login);
                }
            }
        });
        
        || {}
    });
    
    html! {
        <div>{"Processing login..."}</div>
    }
}