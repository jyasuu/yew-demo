use gloo_net::http::Request;
use yew::prelude::*;
use serde::Deserialize;

#[function_component(TomatoClockApp)]
pub fn app() -> Html {
    let header = "🍅 Tomato Clock";
    
    html! {
    <>

        <div class="max-w-lg w-full p-6">
            <div class="bg-white rounded-2xl shadow-xl overflow-hidden">
                
                <div class="bg-tomato-red text-white p-6 text-center">
                    <h1 class="text-3xl font-bold">{header}</h1>
                    <p class="mt-1 opacity-90">{"Focus. Work. Rest. Repeat."}</p>
                </div>
                
                <div class="p-8 flex flex-col items-center">
                    <div class="relative">
                        <div class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2">
                            <i class="fa-solid fa-apple-whole text-tomato-red text-5xl"></i>
                        </div>
                        
                        <svg class="w-64 h-64" viewBox="0 0 100 100">
                            <circle cx="50" cy="50" r="45" fill="none" stroke="#f0f0f0" stroke-width="8" />
                            <circle id="progress-circle" cx="50" cy="50" r="45" fill="none" stroke="#e74c3c" 
                                    stroke-width="8" stroke-dasharray="283" stroke-dashoffset="0" 
                                    stroke-linecap="round" transform="rotate(-90 50 50)" />
                        </svg>
                        
                        <div class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 text-center">
                            <div id="timer" class="text-4xl font-bold text-gray-800">{"25:00"}</div>
                            <div id="status" class="mt-2 font-semibold text-tomato-red">{"Work Time"}</div>
                        </div>
                    </div>
                    
                    <div class="mt-8 flex space-x-4">
                        <button id="start-pause" class="bg-tomato-green hover:bg-tomato-orange text-white px-6 py-3 rounded-full font-semibold shadow-md transition duration-300">
                            <i class="fas fa-play mr-2"></i>{"Start"}
                        </button>
                        <button id="reset" class="bg-gray-300 hover:bg-gray-400 text-gray-700 px-6 py-3 rounded-full font-semibold shadow-md transition duration-300">
                            <i class="fas fa-redo mr-2"></i>{"Reset"}
                        </button>
                    </div>
                </div>
                
                <div class="p-6 bg-light-tomato">
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">{"Work Time (min)"}</label>
                            <input id="work-time" type="number" min="1" max="60" value="25" class="w-full p-2 border rounded-md focus:ring-2 focus:ring-tomato-red"/>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">{"Short Break (min)"}</label>
                            <input id="short-break" type="number" min="1" max="60" value="5" class="w-full p-2 border rounded-md focus:ring-2 focus:ring-tomato-red"/>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">{"Long Break (min)"}</label>
                            <input id="long-break" type="number" min="1" max="60" value="15" class="w-full p-2 border rounded-md focus:ring-2 focus:ring-tomato-red"/>
                        </div>
                    </div>
                    
                    <div class="mt-6 flex items-center justify-center">
                        <span class="text-gray-700 font-medium mr-3">{"Completed Pomodoros:"}</span>
                        <div id="counter" class="flex">
                            <span class="tomato-counter bg-tomato-red"></span>
                            <span class="tomato-counter"></span>
                            <span class="tomato-counter"></span>
                            <span class="tomato-counter"></span>
                        </div>
                    </div>
                </div>
            </div>
            
            <div class="mt-6 text-center text-gray-600 text-sm">
                <p>{"The Pomodoro Technique: 25 minutes of work followed by a 5-minute break. After 4 cycles, take a longer break."}</p>
            </div>
        </div>
       
    </>
    }
}