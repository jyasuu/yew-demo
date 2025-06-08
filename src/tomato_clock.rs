use gloo_timers::callback::Interval;
use std::rc::Rc;
use web_sys::console;
use web_sys::{Event, HtmlInputElement};
use yew::prelude::*;

#[derive(Clone, PartialEq, Debug, Copy)]
enum PomodoroMode {
    Work,
    ShortBreak,
    LongBreak,
}

impl PomodoroMode {
    fn default_time_minutes(&self) -> u32 {
        match self {
            PomodoroMode::Work => 25,
            PomodoroMode::ShortBreak => 5,
            PomodoroMode::LongBreak => 15,
        }
    }

    fn display_text(&self) -> &str {
        match self {
            PomodoroMode::Work => "Work Time",
            PomodoroMode::ShortBreak => "Short Break",
            PomodoroMode::LongBreak => "Long Break",
        }
    }

    fn color_class(&self) -> &str {
        match self {
            PomodoroMode::Work => "text-tomato-red",
            PomodoroMode::ShortBreak => "text-tomato-orange",
            PomodoroMode::LongBreak => "text-tomato-green",
        }
    }
}

#[function_component(TomatoClockApp)]
pub fn app() -> Html {
    let header = "🍅 Tomato Clock";

    let work_time_default = PomodoroMode::Work.default_time_minutes();
    let short_break_default = PomodoroMode::ShortBreak.default_time_minutes();
    let long_break_default = PomodoroMode::LongBreak.default_time_minutes();

    let work_time = use_state(|| work_time_default);
    let short_break_time = use_state(|| short_break_default);
    let long_break_time = use_state(|| long_break_default);

    let time_left = use_state(|| *work_time * 60); // in seconds
    let is_running = use_state(|| false);
    let current_mode = use_state(|| PomodoroMode::Work);
    let pomodoro_count = use_state(|| 0_u32);

    let interval_handle: Rc<std::cell::RefCell<Option<Interval>>> = use_mut_ref(|| None);

    let switch_mode_cb = {
        let current_mode = current_mode.clone();
        let time_left = time_left.clone();
        let pomodoro_count = pomodoro_count.clone();
        let work_time = work_time.clone();
        let short_break_time = short_break_time.clone();
        let long_break_time = long_break_time.clone();

        Callback::from(move |_| {
            let current_mode_val = *current_mode;
            if current_mode_val == PomodoroMode::Work {
                let new_pom_count = *pomodoro_count + 1;
                pomodoro_count.set(new_pom_count);
                if new_pom_count % 4 == 0 {
                    current_mode.set(PomodoroMode::LongBreak);
                    time_left.set(*long_break_time * 60);
                } else {
                    current_mode.set(PomodoroMode::ShortBreak);
                    time_left.set(*short_break_time * 60);
                }
            } else {
                current_mode.set(PomodoroMode::Work);
                time_left.set(*work_time * 60);
            }
        })
    };

    {
        let time_left = time_left.clone();
        let captured_is_running_handle = is_running.clone(); // To set is_running from interval
        let switch_mode_cb = switch_mode_cb.clone();
        let interval_handle_clone = interval_handle.clone();

        use_effect_with(*is_running, // Dependency: effect re-runs if *is_running changes
            move |is_running_dep: &bool| {
                // These are the handles captured by the effect's setup closure:
                // time_left (cloned from outer scope)
                // captured_is_running_handle (cloned from outer scope)
                // switch_mode_cb (cloned from outer scope)
                // interval_handle_clone (cloned from outer scope)
                console::log_1(&format!("Effect setup. is_running_dep: {}, interval_handle_is_some: {}", is_running_dep, interval_handle_clone.borrow().is_some()).into());
                if *is_running_dep {
                    console::log_1(&"Effect: is_running_dep is true.".into());
                    let mut interval_ref_borrow = interval_handle_clone.borrow_mut();
                    if interval_ref_borrow.is_none() { // Only create if not already one (e.g. from HMR)
                        console::log_1(&"Effect: No existing interval. Creating new one.".into());
                        
                        // Clone the handles specifically for the interval's closure
                        let time_left_for_interval = time_left.clone();
                        let is_running_for_interval = captured_is_running_handle.clone();
                        let switch_mode_for_interval = switch_mode_cb.clone();

                        let interval = Interval::new(1000, move || {
                            console::log_1(&format!("Interval tick. Current time_left before set: {}", *time_left_for_interval).into());
                            if *time_left_for_interval > 0 {
                                time_left_for_interval.set(*time_left_for_interval - 1);
                            } else {
                                console::log_1(&"Interval: Time is up. Switching mode and stopping.".into());
                                switch_mode_for_interval.emit(());
                                is_running_for_interval.set(false);
                            }
                        });
                        *interval_ref_borrow = Some(interval);
                    } else {
                        console::log_1(&"Effect: Interval already exists. Doing nothing.".into());
                    }
                } else {
                    console::log_1(&"Effect: is_running_dep is false. Clearing interval.".into());
                    if let Some(interval) = interval_handle_clone.borrow_mut().take() {
                        console::log_1(&"Effect: Interval cancelled because is_running_dep is false.".into());
                        interval.cancel();
                    } else {
                        console::log_1(&"Effect: No interval to cancel (is_running_dep is false).".into());
                    }
                }
                move || { // Cleanup
                    console::log_1(&"Effect cleanup. Attempting to clear interval.".into());
                    if let Some(interval) = interval_handle_clone.borrow_mut().take() { // This uses the interval_handle_clone from the outer scope of use_effect_with
                        console::log_1(&"Effect cleanup: Interval cancelled.".into());
                        interval.cancel();
                    } else {
                        console::log_1(&"Effect cleanup: No interval to cancel.".into());
                    }
                }
            },
        );
    }

    let toggle_timer = {
        let is_running = is_running.clone();
        let time_left = time_left.clone();
        let current_mode = current_mode.clone();
        let work_time = work_time.clone();
        let short_break_time = short_break_time.clone();
        let long_break_time = long_break_time.clone();

        Callback::from(move |_| {
            if *is_running {
                is_running.set(false);
            } else {
                if *time_left == 0 { // If starting from a completed timer
                    let new_time = match *current_mode {
                        PomodoroMode::Work => *work_time * 60,
                        PomodoroMode::ShortBreak => *short_break_time * 60,
                        PomodoroMode::LongBreak => *long_break_time * 60,
                    };
                    if new_time == 0 { return; } // Don't start if duration is 0
                    time_left.set(new_time);
                }
                is_running.set(true);
            }
        })
    };

    let reset_timer = {
        let is_running = is_running.clone();
        let time_left = time_left.clone();
        let current_mode = current_mode.clone();
        let pomodoro_count = pomodoro_count.clone();
        let work_time = work_time.clone();

        Callback::from(move |_| {
            is_running.set(false);
            current_mode.set(PomodoroMode::Work);
            time_left.set(*work_time * 60);
            pomodoro_count.set(0);
        })
    };

    let create_time_input_handler = |
        time_state: UseStateHandle<u32>,
        mode_to_check: PomodoroMode,
        current_mode_handle: UseStateHandle<PomodoroMode>,
        time_left_handle: UseStateHandle<u32>,
        is_running_handle: UseStateHandle<bool>| {
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Ok(val) = input.value().parse::<u32>() {
                let new_time_minutes = val.max(1).min(60); // Clamp value
                time_state.set(new_time_minutes);
                if *current_mode_handle == mode_to_check && !*is_running_handle {
                    time_left_handle.set(new_time_minutes * 60);
                }
                input.set_value(&new_time_minutes.to_string()); // Reflect clamped value
            }
        })
    };

    let on_work_time_change = create_time_input_handler(work_time.clone(), PomodoroMode::Work, current_mode.clone(), time_left.clone(), is_running.clone());
    let on_short_break_change = create_time_input_handler(short_break_time.clone(), PomodoroMode::ShortBreak, current_mode.clone(), time_left.clone(), is_running.clone());
    let on_long_break_change = create_time_input_handler(long_break_time.clone(), PomodoroMode::LongBreak, current_mode.clone(), time_left.clone(), is_running.clone());

    let current_mode_value: PomodoroMode = *current_mode; // Explicitly dereference to get a copy

    let timer_display_text = format!("{:02}:{:02}", *time_left / 60, *time_left % 60);
    console::log_1(&format!("Component render. Main time_left state: {}, is_running: {}", *time_left, *is_running).into());

    // Derive status text, color, and total time based on the current_mode_value
    // by calling methods directly on the enum variants.
    let (status_text, status_color_class, total_time_for_mode_seconds) = match current_mode_value {
        PomodoroMode::Work => (
            PomodoroMode::Work.display_text(),
            PomodoroMode::Work.color_class(),
            *work_time * 60
        ),
        PomodoroMode::ShortBreak => (
            PomodoroMode::ShortBreak.display_text(),
            PomodoroMode::ShortBreak.color_class(),
            *short_break_time * 60
        ),
        PomodoroMode::LongBreak => (
            PomodoroMode::LongBreak.display_text(),
            PomodoroMode::LongBreak.color_class(),
            *long_break_time * 60
        ),
    };

    let progress_offset = if total_time_for_mode_seconds > 0 {
        283.0 - ((*time_left as f64 / total_time_for_mode_seconds as f64).min(1.0).max(0.0) * 283.0)
    } else {
        283.0 
    };
    let stroke_dashoffset_style = format!("{}", progress_offset.max(0.0).min(283.0));

    let (button_icon_class, button_text) = if *is_running {
        ("fas fa-pause mr-2", "Pause")
    } else {
        ("fas fa-play mr-2", "Start")
    };
    let start_pause_button_base_class = if *is_running {
        "bg-tomato-orange"
    } else {
        "bg-tomato-green"
    };

    let num_filled_tomatoes = if *pomodoro_count > 0 && *pomodoro_count % 4 == 0 {
        4
    } else {
        *pomodoro_count % 4
    };

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
                                    stroke-width="8" stroke-dasharray="283" stroke-dashoffset={stroke_dashoffset_style}
                                    stroke-linecap="round" transform="rotate(-90 50 50)" />
                        </svg>
                        
                        <div class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 text-center">
                            <div id="timer" class="text-4xl font-bold text-gray-800">{timer_display_text}</div>
                            <div id="status" class={classes!("mt-2", "font-semibold", status_color_class)}>{status_text}</div>
                        </div>
                    </div>
                    
                    <div class="mt-8 flex space-x-4">
                        <button 
                            id="start-pause" 
                            class={classes!(start_pause_button_base_class, "hover:bg-tomato-orange", "text-white", "px-6", "py-3", "rounded-full", "font-semibold", "shadow-md", "transition", "duration-300")}
                            onclick={toggle_timer}
                        >
                            <i class={button_icon_class}></i>{button_text}
                        </button>
                        <button id="reset" class="bg-gray-300 hover:bg-gray-400 text-gray-700 px-6 py-3 rounded-full font-semibold shadow-md transition duration-300"
                            onclick={reset_timer} >
                            <i class="fas fa-redo mr-2"></i>{"Reset"}
                        </button>
                    </div>
                </div>
                
                <div class="p-6 bg-light-tomato">
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">{"Work Time (min)"}</label>
                            <input id="work-time" type="number" min="1" max="60" value={work_time.to_string()} 
                                   onchange={on_work_time_change}
                                   class="w-full p-2 border rounded-md focus:ring-2 focus:ring-tomato-red"/>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">{"Short Break (min)"}</label>
                            <input id="short-break" type="number" min="1" max="60" value={short_break_time.to_string()} 
                                   onchange={on_short_break_change}
                                   class="w-full p-2 border rounded-md focus:ring-2 focus:ring-tomato-red"/>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">{"Long Break (min)"}</label>
                            <input id="long-break" type="number" min="1" max="60" value={long_break_time.to_string()} 
                                   onchange={on_long_break_change}
                                   class="w-full p-2 border rounded-md focus:ring-2 focus:ring-tomato-red"/>
                        </div>
                    </div>
                    
                    <div class="mt-6 flex items-center justify-center">
                        <span class="text-gray-700 font-medium mr-3">{"Completed Pomodoros:"}</span>
                        <div id="counter" class="flex">
                            {(0..4).map(|i| {
                                let span_class = if i < num_filled_tomatoes { "tomato-counter bg-tomato-red" } else { "tomato-counter" };
                                html! { <span class={span_class}></span> }
                            }).collect::<Html>()}
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