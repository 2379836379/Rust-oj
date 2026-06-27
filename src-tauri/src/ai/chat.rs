use crate::config;
use reqwest::header;
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Option<ChoiceMessage>,
}

#[derive(Debug, Deserialize)]
struct ChoiceMessage {
    content: Option<String>,
}

pub async fn ai_chat(mut messages: Vec<ChatMessage>) -> Result<String, String> {
    let cfg = config::get_openai_config()?;

    let base = Url::parse(&cfg.base_url).map_err(|e| format!("invalid base_url: {e}"))?;
    let url = base
        .join("chat/completions")
        .map_err(|e| format!("join chat/completions: {e}"))?;

    let system_prompt = cfg.system_prompt.trim().to_string();
    if !system_prompt.is_empty() {
        let has_system = messages
            .first()
            .map(|m| m.role.trim().eq_ignore_ascii_case("system"))
            .unwrap_or(false);
        if !has_system {
            messages.insert(
                0,
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt,
                },
            );
        }
    }

    let payload = json!({
        "model": cfg.model,
        "messages": messages,
    });

    let mut headers = header::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));
    if !cfg.api_key.trim().is_empty() {
        let v = format!("Bearer {}", cfg.api_key.trim());
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&v).map_err(|_| "invalid api_key".to_string())?,
        );
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("reqwest build: {e}"))?;

    let resp = client
        .post(url)
        .headers(headers)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("ai request: {e}"))?;

    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| format!("read ai response: {e}"))?;

    if !status.is_success() {
        return Err(text);
    }

    let parsed: ChatResponse = serde_json::from_str(&text)
        .map_err(|e| format!("parse ai response: {e}

{text}"))?;

    let content = parsed
        .choices
        .get(0)
        .and_then(|c| c.message.as_ref())
        .and_then(|m| m.content.clone())
        .unwrap_or_default();

    if content.trim().is_empty() {
        return Err("AI returned empty content.".to_string());
    }

    Ok(content)
}
