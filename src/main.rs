// src/main.rs
use yew::prelude::*;
use yew_router::prelude::*;
mod tomato_clock;
mod tutorial;
mod timer;
mod auth;
mod components;
mod config;
mod boids;
use components::{home::Home, login::Login, callback::Callback, particle_simulation::ParticleSimulation};
use boids::BoidsApp;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/home")]
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
    #[at("/*path")]
    Misc { path: String },
    #[at("/")]
    Boids,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::Login => html! { <Login /> },
        Route::Callback => html! { <Callback /> },
        Route::TomatoClock => html! { <tomato_clock::TomatoClockApp /> },
        Route::Tutorial => html! { <tutorial::App /> },
        Route::Timer => html! { <timer::App /> },
        Route::Boids => html! { <BoidsApp /> },
        Route::ParticleSimulation => html! { <ParticleSimulation /> },
        Route::Misc { path } => html! {<p>{format!("Matched some other path: {}", path)}</p>},
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter basename="/yew-demo">
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
