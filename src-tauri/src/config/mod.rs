mod paths;
mod state;

pub use state::{AppState, AppStateInput, OpenAiConfig, OpenAiConfigInput, OpenAiConfigText};

pub fn get_app_state() -> Result<AppState, String> {
    state::get_app_state()
}

pub fn set_app_state(state: state::AppStateInput) -> Result<(), String> {
    state::set_app_state(state)
}

pub fn get_openai_config() -> Result<OpenAiConfig, String> {
    state::get_openai_config()
}

pub fn set_openai_config(config: OpenAiConfigInput) -> Result<(), String> {
    state::set_openai_config(config)
}

pub fn get_openai_config_text() -> Result<OpenAiConfigText, String> {
    state::get_openai_config_text()
}

pub fn set_openai_config_text(content: String) -> Result<String, String> {
    state::set_openai_config_text(content)
}