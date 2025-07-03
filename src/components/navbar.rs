// src/components/navbar.rs
use yew::prelude::*;
use yew_router::prelude::*;
use crate::Route;

#[function_component(Navbar)]
pub fn navbar() -> Html {
    let navigator = use_navigator().unwrap();
    let current_route = use_route::<Route>().unwrap_or(Route::Boids);

    let navigate_to = {
        let navigator = navigator.clone();
        Callback::from(move |route: Route| {
            navigator.push(&route);
        })
    };

    let is_active = |route: &Route| -> &'static str {
        if std::mem::discriminant(&current_route) == std::mem::discriminant(route) {
            "nav-link active"
        } else {
            "nav-link"
        }
    };

    html! {
        <nav class="navbar">
            <div class="nav-container">
                <div class="nav-brand">
                    <span class="brand-text">{"Yew App"}</span>
                </div>
                <ul class="nav-menu">
                    <li class="nav-item">
                        <a 
                            class={is_active(&Route::Boids)}
                            onclick={
                                let navigate_to = navigate_to.clone();
                                move |_| navigate_to.emit(Route::Boids)
                            }
                        >
                            {"Boids"}
                        </a>
                    </li>
                    <li class="nav-item">
                        <a 
                            class={is_active(&Route::Home)}
                            onclick={
                                let navigate_to = navigate_to.clone();
                                move |_| navigate_to.emit(Route::Home)
                            }
                        >
                            {"Home"}
                        </a>
                    </li>
                    <li class="nav-item">
                        <a 
                            class={is_active(&Route::TomatoClock)}
                            onclick={
                                let navigate_to = navigate_to.clone();
                                move |_| navigate_to.emit(Route::TomatoClock)
                            }
                        >
                            {"Tomato Clock"}
                        </a>
                    </li>
                    <li class="nav-item">
                        <a 
                            class={is_active(&Route::Tutorial)}
                            onclick={
                                let navigate_to = navigate_to.clone();
                                move |_| navigate_to.emit(Route::Tutorial)
                            }
                        >
                            {"Tutorial"}
                        </a>
                    </li>
                    <li class="nav-item">
                        <a 
                            class={is_active(&Route::Timer)}
                            onclick={
                                let navigate_to = navigate_to.clone();
                                move |_| navigate_to.emit(Route::Timer)
                            }
                        >
                            {"Timer"}
                        </a>
                    </li>
                    <li class="nav-item">
                        <a 
                            class={is_active(&Route::ParticleSimulation)}
                            onclick={
                                let navigate_to = navigate_to.clone();
                                move |_| navigate_to.emit(Route::ParticleSimulation)
                            }
                        >
                            {"Particle Sim"}
                        </a>
                    </li>
                    <li class="nav-item">
                        <a 
                            class={is_active(&Route::ParticleSystem)}
                            onclick={
                                let navigate_to = navigate_to.clone();
                                move |_| navigate_to.emit(Route::ParticleSystem)
                            }
                        >
                            {"Particle Sys"}
                        </a>
                    </li>
                    <li class="nav-item">
                        <a 
                            class={is_active(&Route::Login)}
                            onclick={
                                let navigate_to = navigate_to.clone();
                                move |_| navigate_to.emit(Route::Login)
                            }
                        >
                            {"Login"}
                        </a>
                    </li>
                </ul>
            </div>
        </nav>
    }
}