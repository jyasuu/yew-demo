use gloo_timers::callback::Interval;
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

pub enum Msg {
    ToggleTimer,
    ResetTimer,
    Tick,
    SwitchMode,
    WorkTimeChanged(String),
    ShortBreakTimeChanged(String),
    LongBreakTimeChanged(String),
}

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
        let work_time_default = PomodoroMode::Work.default_time_minutes();
        Self {
            header: "🍅 Tomato Clock".to_string(),
            work_time_minutes: work_time_default,
            short_break_time_minutes: PomodoroMode::ShortBreak.default_time_minutes(),
            long_break_time_minutes: PomodoroMode::LongBreak.default_time_minutes(),
            time_left_seconds: work_time_default * 60,
            is_running: false,
            current_mode: PomodoroMode::Work,
            pomodoro_count: 0,
            interval: None,
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
                self.time_left_seconds = self.work_time_minutes * 60;
                self.pomodoro_count = 0;
                true
            }
            Msg::Tick => {
                if self.is_running && self.time_left_seconds > 0 {
                    self.time_left_seconds -= 1;
                    console::log_1(&format!("Tick: time_left_seconds = {}", self.time_left_seconds).into());
                } else if self.is_running && self.time_left_seconds == 0 {
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
        let timer_display_text = format!("{:02}:{:02}", self.time_left_seconds / 60, self.time_left_seconds % 60);
        
        let (status_text, status_color_class, total_time_for_mode_seconds) = match self.current_mode {
            PomodoroMode::Work => (
                PomodoroMode::Work.display_text(),
                PomodoroMode::Work.color_class(),
                self.work_time_minutes * 60
            ),
            PomodoroMode::ShortBreak => (
                PomodoroMode::ShortBreak.display_text(),
                PomodoroMode::ShortBreak.color_class(),
                self.short_break_time_minutes * 60
            ),
            PomodoroMode::LongBreak => (
                PomodoroMode::LongBreak.display_text(),
                PomodoroMode::LongBreak.color_class(),
                self.long_break_time_minutes * 60
            ),
        };

        let progress_offset = if total_time_for_mode_seconds > 0 {
            283.0 - ((self.time_left_seconds as f64 / total_time_for_mode_seconds as f64).min(1.0).max(0.0) * 283.0)
        } else {
            283.0 
        };
        let stroke_dashoffset_style = format!("{}", progress_offset.max(0.0).min(283.0));

        let (button_icon_class, button_text) = if self.is_running {
            ("fas fa-pause mr-2", "Pause")
        } else {
            ("fas fa-play mr-2", "Start")
        };
        let start_pause_button_base_class = if self.is_running {
            "bg-tomato-orange"
        } else {
            "bg-tomato-green"
        };

        let num_filled_tomatoes = if self.pomodoro_count > 0 && self.pomodoro_count % 4 == 0 {
            4
        } else {
            self.pomodoro_count % 4
        };

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

        html! {
            <>
                <div class="max-w-lg w-full p-6">
                    <div class="bg-white rounded-2xl shadow-xl overflow-hidden">
                        <div class="bg-tomato-red text-white p-6 text-center">
                            <h1 class="text-3xl font-bold">{&self.header}</h1>
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
                                    onclick={ctx.link().callback(|_| Msg::ToggleTimer)} >
                                    <i class={button_icon_class}></i>{button_text}
                                </button>
                                <button id="reset" class="bg-gray-300 hover:bg-gray-400 text-gray-700 px-6 py-3 rounded-full font-semibold shadow-md transition duration-300"
                                    onclick={ctx.link().callback(|_| Msg::ResetTimer)} >
                                    <i class="fas fa-redo mr-2"></i>{"Reset"}
                                </button>
                            </div>
                        </div>
                        <div class="p-6 bg-light-tomato">
                            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-1">{"Work Time (min)"}</label>
                                    <input id="work-time" type="number" min="1" max="60" value={self.work_time_minutes.to_string()} 
                                           onchange={on_work_time_change}
                                           class="w-full p-2 border rounded-md focus:ring-2 focus:ring-tomato-red"/>
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-1">{"Short Break (min)"}</label>
                                    <input id="short-break" type="number" min="1" max="60" value={self.short_break_time_minutes.to_string()} 
                                           onchange={on_short_break_change}
                                           class="w-full p-2 border rounded-md focus:ring-2 focus:ring-tomato-red"/>
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-1">{"Long Break (min)"}</label>
                                    <input id="long-break" type="number" min="1" max="60" value={self.long_break_time_minutes.to_string()} 
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
}