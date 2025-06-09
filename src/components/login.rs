// src/components/login.rs
use yew::prelude::*;
use crate::auth::github;

#[function_component(Login)]
pub fn login() -> Html {
    let onclick = Callback::from(|_| {
        github::initiate_login();
    });

    html! {
        <div>
            <button {onclick} class="github-login">
                {"Login with GitHub"}
            </button>
        </div>
    }
}