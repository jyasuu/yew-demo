use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PomodoroDefaults {
    pub work_time_minutes: u32,
    pub short_break_time_minutes: u32,
    pub long_break_time_minutes: u32,
}

#[derive(Debug, Deserialize)]
pub struct TimerDefaults {
    pub standalone_interval_ms: u32,
    pub clock_interval_ms: u32,
    pub timeout_ms: u32,
    pub tick_interval_ms: u32,
}

#[derive(Debug, Deserialize)]
pub struct GitHubAuth {
    pub client_id: String,
    pub redirect_uri: String,
    pub token_url: String,
    pub client_secret: String,
    pub user_api_url: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub pomodoro_defaults: PomodoroDefaults,
    pub timer_defaults: TimerDefaults,
    pub tutorial_data_url: String,
    pub alarm_sound_path: String,
    pub github_auth: GitHubAuth,
}
