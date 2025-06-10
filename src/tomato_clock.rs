use gloo_timers::callback::Interval;
use wasm_bindgen::JsCast;
use web_sys::console;
use web_sys::{Event, HtmlAudioElement, HtmlInputElement};
use yew::prelude::*;
use crate::config::Config;

#[derive(Clone, PartialEq, Debug, Copy)]
pub enum PomodoroMode {
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

pub enum Msg {
    ToggleTimer,
    ResetTimer,
    Tick,
    SwitchMode,
    WorkTimeChanged(String),
    ShortBreakTimeChanged(String),
    LongBreakTimeChanged(String),
}

// --- Extracted Components ---

#[derive(Properties, PartialEq)]
pub struct TimerDisplayProps {
    pub time_left_seconds: u32,
    pub current_mode: PomodoroMode,
    pub work_time_minutes: u32,
    pub short_break_time_minutes: u32,
    pub long_break_time_minutes: u32,
}

#[function_component(TimerDisplay)]
fn timer_display(props: &TimerDisplayProps) -> Html {
    let timer_display_text = format!(
        "{:02}:{:02}",
        props.time_left_seconds / 60,
        props.time_left_seconds % 60
    );

    let (status_text, status_color_class, total_time_for_mode_seconds) = match props.current_mode {
        PomodoroMode::Work => (
            PomodoroMode::Work.display_text(),
            PomodoroMode::Work.color_class(),
            props.work_time_minutes * 60,
        ),
        PomodoroMode::ShortBreak => (
            PomodoroMode::ShortBreak.display_text(),
            PomodoroMode::ShortBreak.color_class(),
            props.short_break_time_minutes * 60,
        ),
        PomodoroMode::LongBreak => (
            PomodoroMode::LongBreak.display_text(),
            PomodoroMode::LongBreak.color_class(),
            props.long_break_time_minutes * 60,
        ),
    };

    let progress_offset = if total_time_for_mode_seconds > 0 {
        283.0
            - ((props.time_left_seconds as f64 / total_time_for_mode_seconds as f64)
                .min(1.0)
                .max(0.0)
                * 283.0)
    } else {
        283.0 // Full circle if total time is 0, indicating it's not started or invalid
    };
    let stroke_dashoffset_style = format!("{}", progress_offset.max(0.0).min(283.0));

    html! {
        <div class="relative">
            <div class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2">
                // You can choose an icon that better represents the timer or leave it as is.
                // For example, a tomato icon: <i class="fa-solid fa-apple-whole text-tomato-red text-5xl"></i>
                // Or a clock icon: <i class="fas fa-clock text-tomato-red text-5xl"></i>
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
    }
}

#[derive(Properties, PartialEq)]
pub struct ControlButtonsProps {
    pub is_running: bool,
    pub on_toggle_timer: Callback<MouseEvent>,
    pub on_reset_timer: Callback<MouseEvent>,
}

#[function_component(ControlButtons)]
fn control_buttons(props: &ControlButtonsProps) -> Html {
    let (button_icon_class, button_text) = if props.is_running {
        ("fas fa-pause mr-2", "Pause")
    } else {
        ("fas fa-play mr-2", "Start")
    };
    let start_pause_button_base_class = if props.is_running {
        "bg-tomato-orange hover:bg-opacity-80" // Adjusted hover for pause
    } else {
        "bg-tomato-green hover:bg-opacity-80" // Adjusted hover for start
    };

    html! {
        <div class="mt-8 flex space-x-4">
            <button
                id="start-pause"
                class={classes!(start_pause_button_base_class, "text-white", "px-6", "py-3", "rounded-full", "font-semibold", "shadow-md", "transition", "duration-300")}
                onclick={props.on_toggle_timer.clone()} >
                <i class={button_icon_class}></i>{button_text}
            </button>
            <button id="reset" class="bg-gray-300 hover:bg-gray-400 text-gray-700 px-6 py-3 rounded-full font-semibold shadow-md transition duration-300"
                onclick={props.on_reset_timer.clone()} >
                <i class="fas fa-redo mr-2"></i>{"Reset"}
            </button>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct TimeSettingsProps {
    pub work_time_minutes: u32,
    pub short_break_time_minutes: u32,
    pub long_break_time_minutes: u32,
    pub on_work_time_change: Callback<Event>,
    pub on_short_break_change: Callback<Event>,
    pub on_long_break_change: Callback<Event>,
    pub is_running: bool,
}

#[function_component(TimeSettings)]
fn time_settings(props: &TimeSettingsProps) -> Html {
    html! {
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
                <label class="block text-sm font-medium text-gray-700 mb-1">{"Work Time (min)"}</label>
                <input id="work-time" type="number" min="1" max="60" value={props.work_time_minutes.to_string()}
                       onchange={props.on_work_time_change.clone()}
                       disabled={props.is_running}
                       class="w-full p-2 border rounded-md focus:ring-2 focus:ring-tomato-red disabled:bg-gray-100"/>
            </div>
            <div>
                <label class="block text-sm font-medium text-gray-700 mb-1">{"Short Break (min)"}</label>
                <input id="short-break" type="number" min="1" max="60" value={props.short_break_time_minutes.to_string()}
                       onchange={props.on_short_break_change.clone()}
                       disabled={props.is_running}
                       class="w-full p-2 border rounded-md focus:ring-2 focus:ring-tomato-red disabled:bg-gray-100"/>
            </div>
            <div>
                <label class="block text-sm font-medium text-gray-700 mb-1">{"Long Break (min)"}</label>
                <input id="long-break" type="number" min="1" max="60" value={props.long_break_time_minutes.to_string()}
                       onchange={props.on_long_break_change.clone()}
                       disabled={props.is_running}
                       class="w-full p-2 border rounded-md focus:ring-2 focus:ring-tomato-red disabled:bg-gray-100"/>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct PomodoroCounterProps {
    pub pomodoro_count: u32,
}

#[function_component(PomodoroCounter)]
fn pomodoro_counter(props: &PomodoroCounterProps) -> Html {
    let num_filled_tomatoes = if props.pomodoro_count > 0 && props.pomodoro_count % 4 == 0 {
        4
    } else {
        props.pomodoro_count % 4
    };

    html! {
        <div class="mt-6 flex items-center justify-center">
            <span class="text-gray-700 font-medium mr-3">{"Completed Pomodoros:"}</span>
            <div id="counter" class="flex">
                {(0..4).map(|i| {
                    let span_class = if i < num_filled_tomatoes { "tomato-counter bg-tomato-red" } else { "tomato-counter bg-tomato-white" };
                    html! { <span class={span_class}></span> }
                }).collect::<Html>()}
            </div>
        </div>
    }
}

// --- Main Application Component ---

pub struct TomatoClockApp {
    header: String,
    work_time_minutes: u32,
    short_break_time_minutes: u32,
    long_break_time_minutes: u32,
    time_left_seconds: u32,
    is_running: bool,
    current_mode: PomodoroMode,
    pomodoro_count: u32,
    interval: Option<Interval>,
    alarm_sound_path: String,
}

impl TomatoClockApp {
    fn get_time_for_mode(&self, mode: PomodoroMode) -> u32 {
        match mode {
            PomodoroMode::Work => self.work_time_minutes * 60,
            PomodoroMode::ShortBreak => self.short_break_time_minutes * 60,
            PomodoroMode::LongBreak => self.long_break_time_minutes * 60,
        }
    }

    fn start_interval(&mut self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        self.interval = Some(Interval::new(1000, move || link.send_message(Msg::Tick)));
    }

    fn stop_interval(&mut self) {
        if let Some(interval) = self.interval.take() {
            interval.cancel();
        }
        self.interval = None;
    }

    fn parse_input_time(input_value: String) -> Option<u32> {
        input_value.parse::<u32>().ok().map(|v| v.max(1).min(60))
    }
}

impl Component for TomatoClockApp {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let config_str = include_str!("../config.json");
        let config: Config = serde_json::from_str(config_str).expect("Failed to parse config.json");

        Self {
            header: "🍅 Tomato Clock".to_string(),
            work_time_minutes: config.pomodoro_defaults.work_time_minutes,
            short_break_time_minutes: config.pomodoro_defaults.short_break_time_minutes,
            long_break_time_minutes: config.pomodoro_defaults.long_break_time_minutes,
            time_left_seconds: config.pomodoro_defaults.work_time_minutes * 60,
            is_running: false,
            current_mode: PomodoroMode::Work,
            pomodoro_count: 0,
            interval: None,
            alarm_sound_path: config.alarm_sound_path,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleTimer => {
                self.is_running = !self.is_running;
                if self.is_running {
                    if self.time_left_seconds == 0 { // If starting from a completed timer
                        self.time_left_seconds = self.get_time_for_mode(self.current_mode);
                        if self.time_left_seconds == 0 { // Don't start if duration is 0
                            self.is_running = false; // revert
                            return true;
                        }
                    }
                    self.start_interval(ctx);
                } else {
                    self.stop_interval();
                }
                true
            }
            Msg::ResetTimer => {
                self.stop_interval();
                self.is_running = false;
                self.current_mode = PomodoroMode::Work;
                let config_str = include_str!("../config.json");
                let config: Config = serde_json::from_str(config_str).expect("Failed to parse config.json");
                self.work_time_minutes = config.pomodoro_defaults.work_time_minutes;
                self.short_break_time_minutes = config.pomodoro_defaults.short_break_time_minutes;
                self.long_break_time_minutes = config.pomodoro_defaults.long_break_time_minutes;
                self.time_left_seconds = self.work_time_minutes * 60;
                self.pomodoro_count = 0;
                true
            }
            Msg::Tick => {
                if self.is_running && self.time_left_seconds > 0 {
                    self.time_left_seconds -= 1;
                    // The console log for every tick can be noisy, uncomment if needed for debugging:
                    // console::log_1(&format!("Tick: time_left_seconds = {}", self.time_left_seconds).into());
                } else if self.is_running && self.time_left_seconds == 0 {
                    // Play sound when timer ends
                    if let Some(window) = web_sys::window() {
                        if let Some(document) = window.document() {
                            if let Some(element) = document.get_element_by_id("alarm") {
                                if let Ok(audio_element) = element.dyn_into::<HtmlAudioElement>() {
                                    let play_result = audio_element.play();
                                    if let Err(e) = play_result {
                                        console::error_1(&format!("Error initiating audio play: {:?}", e).into());
                                    }
                                } else {
                                    console::error_1(&"Failed to cast element to HtmlAudioElement for alarm.".into());
                                }
                            } else {
                                console::error_1(&"Audio element with id 'alarm' not found.".into());
                            }
                        }
                    }

                    self.stop_interval(); // Stop current interval before switching mode
                    self.is_running = false; // Mark as not running to allow mode switch logic to restart if needed
                    ctx.link().send_message(Msg::SwitchMode);
                }
                true
            }
            Msg::SwitchMode => {
                if self.current_mode == PomodoroMode::Work {
                    self.pomodoro_count += 1;
                    if self.pomodoro_count % 4 == 0 {
                        self.current_mode = PomodoroMode::LongBreak;
                    } else {
                        self.current_mode = PomodoroMode::ShortBreak;
                    }
                } else {
                    self.current_mode = PomodoroMode::Work;
                }
                self.time_left_seconds = self.get_time_for_mode(self.current_mode);
                // Optionally auto-start next timer, or require user to click start
                // For now, we don't auto-start. User needs to click "Start" again.
                self.is_running = false; // Ensure timer is stopped
                self.stop_interval();    // And interval is cleared
                true
            }
            Msg::WorkTimeChanged(value) => {
                if let Some(new_time) = Self::parse_input_time(value) {
                    self.work_time_minutes = new_time;
                    if self.current_mode == PomodoroMode::Work && !self.is_running {
                        self.time_left_seconds = new_time * 60;
                    }
                }
                true
            }
            Msg::ShortBreakTimeChanged(value) => {
                if let Some(new_time) = Self::parse_input_time(value) {
                    self.short_break_time_minutes = new_time;
                    if self.current_mode == PomodoroMode::ShortBreak && !self.is_running {
                        self.time_left_seconds = new_time * 60;
                    }
                }
                true
            }
            Msg::LongBreakTimeChanged(value) => {
                if let Some(new_time) = Self::parse_input_time(value) {
                    self.long_break_time_minutes = new_time;
                    if self.current_mode == PomodoroMode::LongBreak && !self.is_running {
                        self.time_left_seconds = new_time * 60;
                    }
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // Callbacks for TimeSettings
        let on_work_time_change = ctx.link().callback(|e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            Msg::WorkTimeChanged(input.value())
        });
        let on_short_break_change = ctx.link().callback(|e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            Msg::ShortBreakTimeChanged(input.value())
        });
        let on_long_break_change = ctx.link().callback(|e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            Msg::LongBreakTimeChanged(input.value())
        });

        // Callbacks for ControlButtons
        let on_toggle_timer = ctx.link().callback(|_| Msg::ToggleTimer);
        let on_reset_timer = ctx.link().callback(|_| Msg::ResetTimer);

        html! {
            <>
                // Audio element for the alarm sound
                <audio id="alarm" preload="auto">
                    <source src={self.alarm_sound_path.clone()} type="audio/mpeg" />
                    {"Your browser does not support the audio element."}
                </audio>
                <div class="max-w-lg w-full p-6">
                    <div class="bg-white rounded-2xl shadow-xl overflow-hidden">
                        // Header
                        <div class="bg-tomato-red text-white p-6 text-center">
                            <h1 class="text-3xl font-bold">{&self.header}</h1>
                            <p class="mt-1 opacity-90">{"Focus. Work. Rest. Repeat."}</p>
                        </div>
                        <div class="p-8 flex flex-col items-center">
                            <div class="relative">
                                <div class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2">
                                    // This inner icon is now part of TimerDisplay, 
                                    // but if you want a static one here outside the SVG, you can add it.
                                    // For this refactor, TimerDisplay handles its own central icon.
                                </div>
                                <TimerDisplay
                                    time_left_seconds={self.time_left_seconds}
                                    current_mode={self.current_mode}
                                    work_time_minutes={self.work_time_minutes}
                                    short_break_time_minutes={self.short_break_time_minutes}
                                    long_break_time_minutes={self.long_break_time_minutes}
                                />
                            </div>
                            <ControlButtons
                                is_running={self.is_running}
                                on_toggle_timer={on_toggle_timer}
                                on_reset_timer={on_reset_timer}
                            />
                        </div>
                        <div class="p-6 bg-light-tomato">
                            <TimeSettings
                                work_time_minutes={self.work_time_minutes}
                                short_break_time_minutes={self.short_break_time_minutes}
                                long_break_time_minutes={self.long_break_time_minutes}
                                on_work_time_change={on_work_time_change}
                                on_short_break_change={on_short_break_change}
                                on_long_break_change={on_long_break_change}
                                is_running={self.is_running}
                            />
                            <PomodoroCounter pomodoro_count={self.pomodoro_count} />
                        </div>
                    </div>
                    // Footer
                    <div class="mt-6 text-center text-gray-600 text-sm">
                        <p>{"The Pomodoro Technique: 25 minutes of work followed by a 5-minute break. After 4 cycles, take a longer break."}</p>
                    </div>
                </div>
            </>
        }
    }
}
