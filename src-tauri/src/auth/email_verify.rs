use serde_json::Value;

const DEFAULT_BASE_URL: &str = "http://10.129.240.62:8080";

fn parse_error_message(body: &[u8], fallback: &str) -> String {
    if let Ok(v) = serde_json::from_slice::<Value>(body) {
        if let Some(obj) = v.as_object() {
            if let Some(err) = obj.get("error").and_then(|v| v.as_str()) {
                if !err.trim().is_empty() {
                    return err.trim().to_string();
                }
            }
            if let Some(msg) = obj.get("message").and_then(|v| v.as_str()) {
                if !msg.trim().is_empty() {
                    return msg.trim().to_string();
                }
            }
        }
    }

    let text = String::from_utf8_lossy(body).trim().to_string();
    if !text.is_empty() {
        return text;
    }

    if !fallback.trim().is_empty() {
        return fallback.to_string();
    }

    "Verification request failed.".to_string()
}

pub async fn send_code(email: &str) -> Result<String, String> {
    let email = email.trim();
    if email.is_empty() {
        return Err("Email is required.".to_string());
    }

    let url = format!("{}/auth/send-code", DEFAULT_BASE_URL);
    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .json(&serde_json::json!({ "email": email }))
        .send()
        .await
        .map_err(|e| format!("send-code request: {e}"))?;

    let status = resp.status();
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("read send-code body: {e}"))?;

    if !status.is_success() {
        return Err(parse_error_message(&bytes, &format!("HTTP {status}")));
    }

    if let Ok(v) = serde_json::from_slice::<Value>(&bytes) {
        if let Some(msg) = v.get("message").and_then(|v| v.as_str()) {
            if !msg.trim().is_empty() {
                return Ok(msg.trim().to_string());
            }
        }
    }

    Ok("Verification code sent.".to_string())
}

pub async fn verify_code(email: &str, code: &str) -> Result<(), String> {
    let email = email.trim();
    let code = code.trim();
    if email.is_empty() {
        return Err("Email is required.".to_string());
    }
    if code.is_empty() {
        return Err("Code is required.".to_string());
    }

    let url = format!("{}/auth/login-code", DEFAULT_BASE_URL);
    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .json(&serde_json::json!({ "email": email, "code": code }))
        .send()
        .await
        .map_err(|e| format!("login-code request: {e}"))?;

    let status = resp.status();
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("read login-code body: {e}"))?;

    if !status.is_success() {
        return Err(parse_error_message(&bytes, &format!("HTTP {status}")));
    }

    Ok(())
}