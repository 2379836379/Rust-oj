use base64::Engine;
use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubmitLanguageOption {
    pub value: String,
    pub label: String,
    pub checked: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubmitPageInfo {
    pub page_url: String,
    pub submit_action_url: Option<String>,
    pub contest_id: Option<String>,
    pub problem_number: Option<String>,
    pub source_encode: Option<String>,
    pub languages: Vec<SubmitLanguageOption>,
}

fn strip_html(html: &str) -> String {
    static TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<[^>]+>").unwrap());
    let no_tags = TAG_RE.replace_all(html, " ");
    no_tags.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extract_hidden_value(html: &str, name: &str) -> Option<String> {
    let pattern = format!(
        r#"(?is)<input\s+type=\"hidden\"\s+name=\"{}\"\s+value=\"([^\"]*)\""#,
        regex::escape(name)
    );
    let re = Regex::new(&pattern).ok()?;
    let caps = re.captures(html)?;
    let v = caps.get(1)?.as_str().trim();
    if v.is_empty() {
        None
    } else {
        Some(v.to_string())
    }
}

pub fn parse_submit_page(html: &str, base_url: &Url) -> SubmitPageInfo {
    static FORM_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"(?is)<form\s+id=\"solution_submit\"\s+action=\"([^\"]+)\"[^>]*>"#)
            .unwrap()
    });

    static LANG_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"(?is)<input\s+type=\"radio\"\s+name=\"language\"\s+value=\"([^\"]+)\"([^>]*)/>\s*([^<]+)"#)
            .unwrap()
    });

    let submit_action_url = FORM_RE
        .captures(html)
        .and_then(|c| c.get(1))
        .and_then(|m| base_url.join(m.as_str()).ok())
        .map(|u| u.to_string());

    let mut languages = Vec::new();
    for caps in LANG_RE.captures_iter(html) {
        let value = caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();
        let flags = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let label = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        if value.is_empty() {
            continue;
        }
        languages.push(SubmitLanguageOption {
            value: value.to_string(),
            checked: flags.to_lowercase().contains("checked"),
            label: strip_html(label),
        });
    }

    SubmitPageInfo {
        page_url: base_url.to_string(),
        submit_action_url,
        contest_id: extract_hidden_value(html, "contestId"),
        problem_number: extract_hidden_value(html, "problemNumber"),
        source_encode: extract_hidden_value(html, "sourceEncode"),
        languages,
    }
}

pub fn default_language(page: &SubmitPageInfo) -> Option<String> {
    if let Some(opt) = page.languages.iter().find(|o| o.checked) {
        return Some(opt.value.clone());
    }
    page.languages.first().map(|o| o.value.clone())
}

pub fn has_language(page: &SubmitPageInfo, language: &str) -> bool {
    page.languages.iter().any(|o| o.value == language)
}

pub fn build_submit_payload(
    page: &SubmitPageInfo,
    language: &str,
    source_text: &str,
) -> Result<String, String> {
    let contest_id = page
        .contest_id
        .as_deref()
        .ok_or_else(|| "missing contestId".to_string())?;
    let problem_number = page
        .problem_number
        .as_deref()
        .ok_or_else(|| "missing problemNumber".to_string())?;
    let source_encode = page
        .source_encode
        .as_deref()
        .ok_or_else(|| "missing sourceEncode".to_string())?;

    let selected_language = if has_language(page, language) {
        language.to_string()
    } else {
        default_language(page).unwrap_or_else(|| language.to_string())
    };

    let encoded_source = base64::engine::general_purpose::STANDARD.encode(source_text.as_bytes());

    Ok(url::form_urlencoded::Serializer::new(String::new())
        .append_pair("contestId", contest_id)
        .append_pair("problemNumber", problem_number)
        .append_pair("sourceEncode", source_encode)
        .append_pair("language", &selected_language)
        .append_pair("source", &encoded_source)
        .finish())
}