// src/components/navbar.rs
use yew::prelude::*;
use yew_router::prelude::*;
use crate::Route;

#[function_component(Navbar)]
pub fn navbar() -> Html {
    let navigator = use_navigator().unwrap();
    let current_route = use_route::<Route>().unwrap_or(Route::PromptAgent);

    // State for dropdown menus
    let ai_dropdown_open = use_state(|| false);
    let demos_dropdown_open = use_state(|| false);
    let tools_dropdown_open = use_state(|| false);
    let mobile_menu_open = use_state(|| false);

    let navigate_to = {
        let navigator = navigator.clone();
        Callback::from(move |route: Route| {
            navigator.push(&route);
        })
    };

    let is_active = |route: &Route| -> &'static str {
        if std::mem::discriminant(&current_route) == std::mem::discriminant(route) {
            "text-blue-600 bg-blue-50"
        } else {
            "text-gray-700 hover:text-blue-600 hover:bg-gray-50"
        }
    };

    let is_group_active = |routes: &[Route]| -> bool {
        routes.iter().any(|route| std::mem::discriminant(&current_route) == std::mem::discriminant(route))
    };

    html! {
        <nav class="bg-white shadow-lg border-b border-gray-200">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <div class="flex justify-between items-center h-16">
                    // Brand/Logo
                    <div class="flex-shrink-0">
                        <a 
                            class="text-2xl font-bold text-blue-600 hover:text-blue-700 cursor-pointer"
                            onclick={
                                let navigate_to = navigate_to.clone();
                                move |_| navigate_to.emit(Route::PromptAgent)
                            }
                        >
                            {"🦀 Yew Demo"}
                        </a>
                    </div>

                    // Desktop Navigation
                    <div class="hidden md:block">
                        <div class="ml-10 flex items-baseline space-x-1">
                            
                            // AI & Chat Group
                            <div class="relative">
                                <button
                                    class={format!("px-3 py-2 rounded-md text-sm font-medium transition-colors duration-200 flex items-center space-x-1 {}",
                                        if is_group_active(&[Route::PromptAgent, Route::Gemini, Route::GeminiMcp, Route::WebRtcChat]) {
                                            "text-blue-600 bg-blue-50"
                                        } else {
                                            "text-gray-700 hover:text-blue-600 hover:bg-gray-50"
                                        }
                                    )}
                                    onclick={
                                        let ai_dropdown_open = ai_dropdown_open.clone();
                                        move |_| ai_dropdown_open.set(!*ai_dropdown_open)
                                    }
                                >
                                    <span>{"🤖 AI & Chat"}</span>
                                    <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                                        <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd" />
                                    </svg>
                                </button>
                                
                                if *ai_dropdown_open {
                                    <div class="absolute right-0 mt-2 w-56 rounded-md shadow-lg bg-white ring-1 ring-black ring-opacity-5 z-50">
                                        <div class="py-1">
                                            <a
                                                class={format!("block px-4 py-2 text-sm cursor-pointer transition-colors duration-200 {}", is_active(&Route::PromptAgent))}
                                                onclick={
                                                    let navigate_to = navigate_to.clone();
                                                    let ai_dropdown_open = ai_dropdown_open.clone();
                                                    move |_| {
                                                        navigate_to.emit(Route::PromptAgent);
                                                        ai_dropdown_open.set(false);
                                                    }
                                                }
                                            >
                                                {"🎯 Prompt Agent"}
                                            </a>
                                            <a
                                                class={format!("block px-4 py-2 text-sm cursor-pointer transition-colors duration-200 {}", is_active(&Route::Gemini))}
                                                onclick={
                                                    let navigate_to = navigate_to.clone();
                                                    let ai_dropdown_open = ai_dropdown_open.clone();
                                                    move |_| {
                                                        navigate_to.emit(Route::Gemini);
                                                        ai_dropdown_open.set(false);
                                                    }
                                                }
                                            >
                                                {"💬 Gemini Chat"}
                                            </a>
                                            <a
                                                class={format!("block px-4 py-2 text-sm cursor-pointer transition-colors duration-200 {}", is_active(&Route::GeminiMcp))}
                                                onclick={
                                                    let navigate_to = navigate_to.clone();
                                                    let ai_dropdown_open = ai_dropdown_open.clone();
                                                    move |_| {
                                                        navigate_to.emit(Route::GeminiMcp);
                                                        ai_dropdown_open.set(false);
                                                    }
                                                }
                                            >
                                                {"🔧 Gemini MCP"}
                                            </a>
                                            <a
                                                class={format!("block px-4 py-2 text-sm cursor-pointer transition-colors duration-200 {}", is_active(&Route::WebRtcChat))}
                                                onclick={
                                                    let navigate_to = navigate_to.clone();
                                                    let ai_dropdown_open = ai_dropdown_open.clone();
                                                    move |_| {
                                                        navigate_to.emit(Route::WebRtcChat);
                                                        ai_dropdown_open.set(false);
                                                    }
                                                }
                                            >
                                                {"📡 WebRTC Chat"}
                                            </a>
                                        </div>
                                    </div>
                                }
                            </div>

                            // Demos & Simulations Group
                            <div class="relative">
                                <button
                                    class={format!("px-3 py-2 rounded-md text-sm font-medium transition-colors duration-200 flex items-center space-x-1 {}",
                                        if is_group_active(&[Route::Boids, Route::ParticleSimulation, Route::ParticleSystem]) {
                                            "text-blue-600 bg-blue-50"
                                        } else {
                                            "text-gray-700 hover:text-blue-600 hover:bg-gray-50"
                                        }
                                    )}
                                    onclick={
                                        let demos_dropdown_open = demos_dropdown_open.clone();
                                        move |_| demos_dropdown_open.set(!*demos_dropdown_open)
                                    }
                                >
                                    <span>{"🎮 Demos"}</span>
                                    <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                                        <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd" />
                                    </svg>
                                </button>
                                
                                if *demos_dropdown_open {
                                    <div class="absolute right-0 mt-2 w-56 rounded-md shadow-lg bg-white ring-1 ring-black ring-opacity-5 z-50">
                                        <div class="py-1">
                                            <a
                                                class={format!("block px-4 py-2 text-sm cursor-pointer transition-colors duration-200 {}", is_active(&Route::Boids))}
                                                onclick={
                                                    let navigate_to = navigate_to.clone();
                                                    let demos_dropdown_open = demos_dropdown_open.clone();
                                                    move |_| {
                                                        navigate_to.emit(Route::Boids);
                                                        demos_dropdown_open.set(false);
                                                    }
                                                }
                                            >
                                                {"🐦 Boids Simulation"}
                                            </a>
                                            <a
                                                class={format!("block px-4 py-2 text-sm cursor-pointer transition-colors duration-200 {}", is_active(&Route::ParticleSimulation))}
                                                onclick={
                                                    let navigate_to = navigate_to.clone();
                                                    let demos_dropdown_open = demos_dropdown_open.clone();
                                                    move |_| {
                                                        navigate_to.emit(Route::ParticleSimulation);
                                                        demos_dropdown_open.set(false);
                                                    }
                                                }
                                            >
                                                {"✨ Particle Simulation"}
                                            </a>
                                            <a
                                                class={format!("block px-4 py-2 text-sm cursor-pointer transition-colors duration-200 {}", is_active(&Route::ParticleSystem))}
                                                onclick={
                                                    let navigate_to = navigate_to.clone();
                                                    let demos_dropdown_open = demos_dropdown_open.clone();
                                                    move |_| {
                                                        navigate_to.emit(Route::ParticleSystem);
                                                        demos_dropdown_open.set(false);
                                                    }
                                                }
                                            >
                                                {"🌟 Particle System"}
                                            </a>
                                        </div>
                                    </div>
                                }
                            </div>

                            // Tools & Utilities Group
                            <div class="relative">
                                <button
                                    class={format!("px-3 py-2 rounded-md text-sm font-medium transition-colors duration-200 flex items-center space-x-1 {}",
                                        if is_group_active(&[Route::TomatoClock, Route::Timer, Route::Tutorial]) {
                                            "text-blue-600 bg-blue-50"
                                        } else {
                                            "text-gray-700 hover:text-blue-600 hover:bg-gray-50"
                                        }
                                    )}
                                    onclick={
                                        let tools_dropdown_open = tools_dropdown_open.clone();
                                        move |_| tools_dropdown_open.set(!*tools_dropdown_open)
                                    }
                                >
                                    <span>{"🛠️ Tools"}</span>
                                    <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                                        <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd" />
                                    </svg>
                                </button>
                                
                                if *tools_dropdown_open {
                                    <div class="absolute right-0 mt-2 w-56 rounded-md shadow-lg bg-white ring-1 ring-black ring-opacity-5 z-50">
                                        <div class="py-1">
                                            <a
                                                class={format!("block px-4 py-2 text-sm cursor-pointer transition-colors duration-200 {}", is_active(&Route::TomatoClock))}
                                                onclick={
                                                    let navigate_to = navigate_to.clone();
                                                    let tools_dropdown_open = tools_dropdown_open.clone();
                                                    move |_| {
                                                        navigate_to.emit(Route::TomatoClock);
                                                        tools_dropdown_open.set(false);
                                                    }
                                                }
                                            >
                                                {"🍅 Tomato Clock"}
                                            </a>
                                            <a
                                                class={format!("block px-4 py-2 text-sm cursor-pointer transition-colors duration-200 {}", is_active(&Route::Timer))}
                                                onclick={
                                                    let navigate_to = navigate_to.clone();
                                                    let tools_dropdown_open = tools_dropdown_open.clone();
                                                    move |_| {
                                                        navigate_to.emit(Route::Timer);
                                                        tools_dropdown_open.set(false);
                                                    }
                                                }
                                            >
                                                {"⏰ Timer"}
                                            </a>
                                            <a
                                                class={format!("block px-4 py-2 text-sm cursor-pointer transition-colors duration-200 {}", is_active(&Route::Tutorial))}
                                                onclick={
                                                    let navigate_to = navigate_to.clone();
                                                    let tools_dropdown_open = tools_dropdown_open.clone();
                                                    move |_| {
                                                        navigate_to.emit(Route::Tutorial);
                                                        tools_dropdown_open.set(false);
                                                    }
                                                }
                                            >
                                                {"📚 Tutorial"}
                                            </a>
                                        </div>
                                    </div>
                                }
                            </div>

                            // Home and Login as direct links
                            <a
                                class={format!("px-3 py-2 rounded-md text-sm font-medium transition-colors duration-200 {}", is_active(&Route::Home))}
                                onclick={
                                    let navigate_to = navigate_to.clone();
                                    move |_| navigate_to.emit(Route::Home)
                                }
                            >
                                {"🏠 Home"}
                            </a>

                            <a
                                class={format!("px-3 py-2 rounded-md text-sm font-medium transition-colors duration-200 {}", is_active(&Route::Login))}
                                onclick={
                                    let navigate_to = navigate_to.clone();
                                    move |_| navigate_to.emit(Route::Login)
                                }
                            >
                                {"🔐 Login"}
                            </a>
                        </div>
                    </div>

                    // Mobile menu button
                    <div class="md:hidden">
                        <button
                            class="inline-flex items-center justify-center p-2 rounded-md text-gray-400 hover:text-blue-600 hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-blue-500"
                            onclick={
                                let mobile_menu_open = mobile_menu_open.clone();
                                move |_| mobile_menu_open.set(!*mobile_menu_open)
                            }
                        >
                            <svg class="block h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
                            </svg>
                        </button>
                    </div>
                </div>

                // Mobile menu
                if *mobile_menu_open {
                    <div class="md:hidden">
                        <div class="px-2 pt-2 pb-3 space-y-1 sm:px-3 bg-gray-50 border-t border-gray-200">
                            // AI & Chat section
                            <div class="space-y-1">
                                <div class="px-3 py-2 text-xs font-semibold text-gray-500 uppercase tracking-wider">{"AI & Chat"}</div>
                                <a
                                    class={format!("block px-3 py-2 rounded-md text-sm font-medium cursor-pointer transition-colors duration-200 {}", is_active(&Route::PromptAgent))}
                                    onclick={
                                        let navigate_to = navigate_to.clone();
                                        let mobile_menu_open = mobile_menu_open.clone();
                                        move |_| {
                                            navigate_to.emit(Route::PromptAgent);
                                            mobile_menu_open.set(false);
                                        }
                                    }
                                >
                                    {"🎯 Prompt Agent"}
                                </a>
                                <a
                                    class={format!("block px-3 py-2 rounded-md text-sm font-medium cursor-pointer transition-colors duration-200 {}", is_active(&Route::Gemini))}
                                    onclick={
                                        let navigate_to = navigate_to.clone();
                                        let mobile_menu_open = mobile_menu_open.clone();
                                        move |_| {
                                            navigate_to.emit(Route::Gemini);
                                            mobile_menu_open.set(false);
                                        }
                                    }
                                >
                                    {"💬 Gemini Chat"}
                                </a>
                                <a
                                    class={format!("block px-3 py-2 rounded-md text-sm font-medium cursor-pointer transition-colors duration-200 {}", is_active(&Route::GeminiMcp))}
                                    onclick={
                                        let navigate_to = navigate_to.clone();
                                        let mobile_menu_open = mobile_menu_open.clone();
                                        move |_| {
                                            navigate_to.emit(Route::GeminiMcp);
                                            mobile_menu_open.set(false);
                                        }
                                    }
                                >
                                    {"🔧 Gemini MCP"}
                                </a>
                                <a
                                    class={format!("block px-3 py-2 rounded-md text-sm font-medium cursor-pointer transition-colors duration-200 {}", is_active(&Route::WebRtcChat))}
                                    onclick={
                                        let navigate_to = navigate_to.clone();
                                        let mobile_menu_open = mobile_menu_open.clone();
                                        move |_| {
                                            navigate_to.emit(Route::WebRtcChat);
                                            mobile_menu_open.set(false);
                                        }
                                    }
                                >
                                    {"📡 WebRTC Chat"}
                                </a>
                            </div>

                            // Demos section
                            <div class="space-y-1 pt-4">
                                <div class="px-3 py-2 text-xs font-semibold text-gray-500 uppercase tracking-wider">{"Demos"}</div>
                                <a
                                    class={format!("block px-3 py-2 rounded-md text-sm font-medium cursor-pointer transition-colors duration-200 {}", is_active(&Route::Boids))}
                                    onclick={
                                        let navigate_to = navigate_to.clone();
                                        let mobile_menu_open = mobile_menu_open.clone();
                                        move |_| {
                                            navigate_to.emit(Route::Boids);
                                            mobile_menu_open.set(false);
                                        }
                                    }
                                >
                                    {"🐦 Boids Simulation"}
                                </a>
                                <a
                                    class={format!("block px-3 py-2 rounded-md text-sm font-medium cursor-pointer transition-colors duration-200 {}", is_active(&Route::ParticleSimulation))}
                                    onclick={
                                        let navigate_to = navigate_to.clone();
                                        let mobile_menu_open = mobile_menu_open.clone();
                                        move |_| {
                                            navigate_to.emit(Route::ParticleSimulation);
                                            mobile_menu_open.set(false);
                                        }
                                    }
                                >
                                    {"✨ Particle Simulation"}
                                </a>
                                <a
                                    class={format!("block px-3 py-2 rounded-md text-sm font-medium cursor-pointer transition-colors duration-200 {}", is_active(&Route::ParticleSystem))}
                                    onclick={
                                        let navigate_to = navigate_to.clone();
                                        let mobile_menu_open = mobile_menu_open.clone();
                                        move |_| {
                                            navigate_to.emit(Route::ParticleSystem);
                                            mobile_menu_open.set(false);
                                        }
                                    }
                                >
                                    {"🌟 Particle System"}
                                </a>
                            </div>

                            // Tools section
                            <div class="space-y-1 pt-4">
                                <div class="px-3 py-2 text-xs font-semibold text-gray-500 uppercase tracking-wider">{"Tools"}</div>
                                <a
                                    class={format!("block px-3 py-2 rounded-md text-sm font-medium cursor-pointer transition-colors duration-200 {}", is_active(&Route::TomatoClock))}
                                    onclick={
                                        let navigate_to = navigate_to.clone();
                                        let mobile_menu_open = mobile_menu_open.clone();
                                        move |_| {
                                            navigate_to.emit(Route::TomatoClock);
                                            mobile_menu_open.set(false);
                                        }
                                    }
                                >
                                    {"🍅 Tomato Clock"}
                                </a>
                                <a
                                    class={format!("block px-3 py-2 rounded-md text-sm font-medium cursor-pointer transition-colors duration-200 {}", is_active(&Route::Timer))}
                                    onclick={
                                        let navigate_to = navigate_to.clone();
                                        let mobile_menu_open = mobile_menu_open.clone();
                                        move |_| {
                                            navigate_to.emit(Route::Timer);
                                            mobile_menu_open.set(false);
                                        }
                                    }
                                >
                                    {"⏰ Timer"}
                                </a>
                                <a
                                    class={format!("block px-3 py-2 rounded-md text-sm font-medium cursor-pointer transition-colors duration-200 {}", is_active(&Route::Tutorial))}
                                    onclick={
                                        let navigate_to = navigate_to.clone();
                                        let mobile_menu_open = mobile_menu_open.clone();
                                        move |_| {
                                            navigate_to.emit(Route::Tutorial);
                                            mobile_menu_open.set(false);
                                        }
                                    }
                                >
                                    {"📚 Tutorial"}
                                </a>
                            </div>

                            // Other links
                            <div class="space-y-1 pt-4">
                                <div class="px-3 py-2 text-xs font-semibold text-gray-500 uppercase tracking-wider">{"Other"}</div>
                                <a
                                    class={format!("block px-3 py-2 rounded-md text-sm font-medium cursor-pointer transition-colors duration-200 {}", is_active(&Route::Home))}
                                    onclick={
                                        let navigate_to = navigate_to.clone();
                                        let mobile_menu_open = mobile_menu_open.clone();
                                        move |_| {
                                            navigate_to.emit(Route::Home);
                                            mobile_menu_open.set(false);
                                        }
                                    }
                                >
                                    {"🏠 Home"}
                                </a>
                                <a
                                    class={format!("block px-3 py-2 rounded-md text-sm font-medium cursor-pointer transition-colors duration-200 {}", is_active(&Route::Login))}
                                    onclick={
                                        let navigate_to = navigate_to.clone();
                                        let mobile_menu_open = mobile_menu_open.clone();
                                        move |_| {
                                            navigate_to.emit(Route::Login);
                                            mobile_menu_open.set(false);
                                        }
                                    }
                                >
                                    {"🔐 Login"}
                                </a>
                            </div>
                        </div>
                    </div>
                }
            </div>
        </nav>
    }
}
