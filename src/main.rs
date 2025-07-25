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
mod gemini_chat;
mod prompt_agent;
use components::{home::Home, login::Login, callback::Callback, particle_simulation::ParticleSimulation,navbar::Navbar,particle_system::ParticleSystem};
use boids::BoidsApp;
use gemini_chat::{App as GeminiApp};
use prompt_agent::PromptAgent;


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
    #[at("/particle_system")]
    ParticleSystem,
    #[at("/boids")]
    Boids,
    #[at("/gemini")]
    Gemini,
    #[at("/")]
    PromptAgent,
    #[at("/*path")]
    Misc { path: String },
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
        Route::ParticleSystem => html! { <ParticleSystem /> },
        Route::Gemini => html! { <GeminiApp /> },
        Route::PromptAgent => html! { <PromptAgent /> },
        Route::Misc { path } => html! {<p>{format!("Matched some other path: {}", path)}</p>},
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter basename="/yew-demo">
            <div class="app">
                <Navbar />
                <main class="main-content">
                    <Switch<Route> render={switch} />
                </main>
            </div>
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
