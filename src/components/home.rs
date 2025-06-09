// src/components/home.rs
use yew::prelude::*;
use crate::auth::github;
use gloo_storage::{LocalStorage, Storage};
use wasm_bindgen_futures::spawn_local;
use yew_router::prelude::*;
use crate::Route;

#[function_component(Home)]
pub fn home() -> Html {
    let user = use_state(|| None);
    let navigator = use_navigator().unwrap();
    
    {
        let user = user.clone();
        use_effect_with(() , move |_| {
            if let Ok(token) = LocalStorage::get::<String>("github_access_token") {
                spawn_local(async move {
                    match github::get_user(&token).await {
                        Ok(user_data) => user.set(Some(user_data)),
                        Err(err) => {
                            gloo_console::error!("Failed to get user:", err);
                            LocalStorage::delete("github_access_token");
                        }
                    }
                });
            }
            
            || {}
        });
    }
    
    let logout = Callback::from(move |_| {
        LocalStorage::delete("github_access_token");
        navigator.push(&Route::Login);
    });
    
    match user.as_ref() {
        Some(user) => html! {
            <div class="user-profile">
                <img src={user.avatar_url.clone()} alt="Avatar" />
                <h2>{&user.name}</h2>
                <p>{"Username: "}{&user.login}</p>
                <button onclick={logout}>{"Logout"}</button>
            </div>
        },
        None => html! {
            <div>
                <p>{"Loading user data..."}</p>
                <button onclick={logout}>{"Logout"}</button>
            </div>
        },
    }
}