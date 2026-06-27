use super::paths;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub ring_path: Option<String>,
    pub alarm_enabled: bool,
    pub source_path: String,
    pub save_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStateInput {
    pub ring_path: Option<String>,
    pub alarm_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    pub base_url: String,
    pub model: String,
    pub system_prompt: String,
    pub api_key: String,
    pub source_path: String,
    pub save_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfigInput {
    pub base_url: String,
    pub model: String,
    pub system_prompt: String,
    pub api_key: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfigText {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct AppStateFile {
    #[serde(default)]
    ring_path: Option<String>,
    #[serde(default)]
    alarm_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAiFile {
    openai: OpenAiSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAiSection {
    #[serde(default = "default_base_url")]
    base_url: String,
    #[serde(default = "default_model")]
    model: String,
    #[serde(default = "default_system_prompt")]
    system_prompt: String,
    #[serde(default)]
    api_key: String,
}

fn default_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_model() -> String {
    "gpt-5-mini".to_string()
}

fn default_system_prompt() -> String {
    "You are an AI programming assistant for algorithm problems.".to_string()
}

fn resolve_paths(file_name: &str) -> Result<(PathBuf, PathBuf), String> {
    let legacy = paths::legacy_candidate_paths(file_name)?;
    let source = paths::find_first_existing(&legacy);
    let config_dir = paths::config_dir()?;
    let config_path = config_dir.join(file_name);

    let source_path = source.clone().unwrap_or_else(|| config_path.clone());
    let save_path = source.unwrap_or(config_path);
    Ok((source_path, save_path))
}

pub fn get_app_state() -> Result<AppState, String> {
    let (source_path, save_path) = resolve_paths("appstate.toml")?;

    let (ring_path, alarm_enabled) = if source_path.is_file() {
        let text = std::fs::read_to_string(&source_path)
            .map_err(|e| format!("read {}: {e}", source_path.display()))?;
        let parsed: AppStateFile = toml::from_str(&text)
            .map_err(|e| format!("parse {}: {e}", source_path.display()))?;
        (
            parsed.ring_path,
            parsed.alarm_enabled.unwrap_or(false),
        )
    } else {
        (None, false)
    };

    Ok(AppState {
        ring_path,
        alarm_enabled,
        source_path: source_path.display().to_string(),
        save_path: save_path.display().to_string(),
    })
}

pub fn set_app_state(input: AppStateInput) -> Result<(), String> {
    let (_source_path, save_path) = resolve_paths("appstate.toml")?;
    paths::ensure_parent_dir(&save_path)?;

    let file = AppStateFile {
        ring_path: input.ring_path,
        alarm_enabled: Some(input.alarm_enabled),
    };
    let content = toml::to_string_pretty(&file).map_err(|e| format!("toml serialize: {e}"))?;
    std::fs::write(&save_path, content)
        .map_err(|e| format!("write {}: {e}", save_path.display()))?;
    Ok(())
}


fn default_openai_toml() -> String {
    toml::to_string_pretty(&OpenAiFile {
        openai: OpenAiSection {
            base_url: default_base_url(),
            model: default_model(),
            system_prompt: default_system_prompt(),
            api_key: String::new(),
        },
    })
    .unwrap_or_else(|_| "[openai]\nbase_url = \"https://api.openai.com/v1\"\nmodel = \"gpt-5-mini\"\nsystem_prompt = \"You are an AI programming assistant for algorithm problems.\"\napi_key = \"\"\n".to_string())
}

pub fn get_openai_config_text() -> Result<OpenAiConfigText, String> {
    let (source_path, save_path) = resolve_paths("config.toml")?;

    let content = if source_path.is_file() {
        std::fs::read_to_string(&source_path)
            .map_err(|e| format!("read {}: {e}", source_path.display()))?
    } else {
        default_openai_toml()
    };

    Ok(OpenAiConfigText {
        path: save_path.display().to_string(),
        content,
    })
}

pub fn set_openai_config_text(content: String) -> Result<String, String> {
    let (_source_path, save_path) = resolve_paths("config.toml")?;
    paths::ensure_parent_dir(&save_path)?;

    let text = if content.trim().is_empty() {
        default_openai_toml()
    } else {
        content
    };

    // Validate before saving.
    let _: OpenAiFile = toml::from_str(&text)
        .map_err(|e| format!("parse config.toml: {e}"))?;

    std::fs::write(&save_path, text)
        .map_err(|e| format!("write {}: {e}", save_path.display()))?;

    Ok(save_path.display().to_string())
}
pub fn get_openai_config() -> Result<OpenAiConfig, String> {
    let (source_path, save_path) = resolve_paths("config.toml")?;

    let section = if source_path.is_file() {
        let text = std::fs::read_to_string(&source_path)
            .map_err(|e| format!("read {}: {e}", source_path.display()))?;
        let parsed: OpenAiFile =
            toml::from_str(&text).map_err(|e| format!("parse {}: {e}", source_path.display()))?;
        parsed.openai
    } else {
        OpenAiSection {
            base_url: default_base_url(),
            model: default_model(),
            system_prompt: default_system_prompt(),
            api_key: String::new(),
        }
    };

    Ok(OpenAiConfig {
        base_url: section.base_url,
        model: section.model,
        system_prompt: section.system_prompt,
        api_key: section.api_key,
        source_path: source_path.display().to_string(),
        save_path: save_path.display().to_string(),
    })
}

pub fn set_openai_config(input: OpenAiConfigInput) -> Result<(), String> {
    let (_source_path, save_path) = resolve_paths("config.toml")?;
    paths::ensure_parent_dir(&save_path)?;

    let file = OpenAiFile {
        openai: OpenAiSection {
            base_url: if input.base_url.trim().is_empty() {
                default_base_url()
            } else {
                input.base_url
            },
            model: if input.model.trim().is_empty() {
                default_model()
            } else {
                input.model
            },
            system_prompt: input.system_prompt,
            api_key: input.api_key,
        },
    };

    let content = toml::to_string_pretty(&file).map_err(|e| format!("toml serialize: {e}"))?;
    std::fs::write(&save_path, content)
        .map_err(|e| format!("write {}: {e}", save_path.display()))?;
    Ok(())
}

