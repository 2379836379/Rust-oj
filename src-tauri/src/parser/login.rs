use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JoinedClassInfo {
    pub name: String,
    pub url: String,
}

fn strip_html_tags(html: &str) -> String {
    static TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<[^>]+>").unwrap());
    let no_tags = TAG_RE.replace_all(html, " ");
    no_tags.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn extract_personal_home_url(html: &str, base_url: &Url) -> Option<String> {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"<a\s+href=\"([^\"]*/user/\d+/?)[^\"]*\"[^>]*>"#).unwrap()
    });

    if let Some(caps) = RE.captures(html) {
        let href = caps.get(1)?.as_str();
        return base_url.join(href).ok().map(|u| u.to_string());
    }

    fallback_personal_home_url(html, base_url)
}

fn fallback_personal_home_url(html: &str, base_url: &Url) -> Option<String> {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"/user/\d+/?").unwrap());
    let m = RE.find(html)?;
    let rel = &html[m.start()..m.end()];
    base_url.join(rel).ok().map(|u| u.to_string())
}

pub fn extract_joined_classes(html: &str, base_url: &Url) -> Vec<JoinedClassInfo> {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"<a\s+[^>]*class=\"[^\"]*\bgroup-name\b[^\"]*\"[^>]*href=\"([^\"]+)\"[^>]*>([\s\S]*?)</a>"#,
        )
        .unwrap()
    });

    let mut out = Vec::new();
    for caps in RE.captures_iter(html) {
        let href = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
        let inner = caps.get(2).map(|m| m.as_str()).unwrap_or_default();
        let name = strip_html_tags(inner);
        if name.is_empty() {
            continue;
        }
        if let Ok(u) = base_url.join(href) {
            out.push(JoinedClassInfo {
                name,
                url: u.to_string(),
            });
        }
    }
    out
}