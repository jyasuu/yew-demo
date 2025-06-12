// src/main.rs
use yew::prelude::*;
use yew_router::prelude::*;
mod tomato_clock;
mod tutorial;
mod timer;
mod auth;
mod components;
mod config;
use components::{home::Home, login::Login, callback::Callback, particle_simulation::ParticleSimulation};

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/login")]
    Login,
    #[at("/callback")]
    Callback,
    #[at("/tomato_clock")]
    TomatoClock,
    #[at("/tutorial")]
    Tutorial,
    #[at("/timer")]
    Timer,
    #[at("/particle_simulation")]
    ParticleSimulation,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::Login => html! { <Login /> },
        Route::Callback => html! { <Callback /> },
        Route::TomatoClock => html! { <tomato_clock::TomatoClockApp /> },
        Route::Tutorial => html! { <tutorial::App /> },
        Route::Timer => html! { <timer::App /> },
        Route::ParticleSimulation => html! { <ParticleSimulation /> },
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
