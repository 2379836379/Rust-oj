use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResultPageInfo {
    pub page_url: String,
    pub solution_url: Option<String>,
    pub submission_id: Option<String>,
    pub status_text: Option<String>,
    pub status_class: Option<String>,
    pub detail_title: Option<String>,
    pub detail_text: Option<String>,
    pub has_result: bool,
}

fn strip_html(html: &str) -> String {
    static TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<[^>]+>").unwrap());
    let no_tags = TAG_RE.replace_all(html, " ");
    no_tags.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn decode_preformatted_text(html: &str) -> String {
    html.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .trim()
        .to_string()
}

pub fn parse_result_page(html: &str, base_url: &Url) -> ResultPageInfo {
    static TITLE_RE: Lazy<Regex> = Lazy::new(|| {
        // Keep it simple: capture digits in the <h2> title.
        Regex::new(r#"(?is)<div\s+id=\"pageTitle\"[^>]*>\s*<h2>\s*#?(\d+)\s*"#).unwrap()
    });

    static STATUS_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"(?is)<p\s+class=\"compile-status\"[^>]*>[\s\S]*?状态\s*<a\s+href=\"([^\"]+)\"\s+class=\"([^\"]+)\"[^>]*>([\s\S]*?)</a>"#,
        )
        .unwrap()
    });

    static STATUS_RE_EN: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"(?is)<p\s+class=\"compile-status\"[^>]*>[\s\S]*?Status\s*<a\s+href=\"([^\"]+)\"\s+class=\"([^\"]+)\"[^>]*>([\s\S]*?)</a>"#,
        )
        .unwrap()
    });

    static DETAIL_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"(?is)<h3[^>]*class=\"([^\"]*h3-compile-status[^\"]*)\"[^>]*>([\s\S]*?)</h3>[\s\S]*?<pre>([\s\S]*?)</pre>"#,
        )
        .unwrap()
    });

    let submission_id = TITLE_RE
        .captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string());

    let status_caps = STATUS_RE
        .captures(html)
        .or_else(|| STATUS_RE_EN.captures(html));

    let (solution_url, status_class, status_text, has_result) = if let Some(c) = status_caps {
        let href = c.get(1).map(|m| m.as_str()).unwrap_or("");
        let solution_url = base_url.join(href).ok().map(|u| u.to_string());
        let status_class = c.get(2).map(|m| m.as_str().trim().to_string());
        let status_text = c.get(3).map(|m| strip_html(m.as_str()));
        let has_result = status_text.as_deref().map(|s| !s.is_empty()).unwrap_or(false);
        (solution_url, status_class, status_text, has_result)
    } else {
        (None, None, None, false)
    };

    let (detail_title, detail_text) = if let Some(c) = DETAIL_RE.captures(html) {
        let title = strip_html(c.get(2).map(|m| m.as_str()).unwrap_or(""));
        let text = decode_preformatted_text(c.get(3).map(|m| m.as_str()).unwrap_or(""));
        (
            if title.is_empty() { None } else { Some(title) },
            if text.is_empty() { None } else { Some(text) },
        )
    } else {
        (None, None)
    };

    ResultPageInfo {
        page_url: base_url.to_string(),
        solution_url,
        submission_id,
        status_text,
        status_class,
        detail_title,
        detail_text,
        has_result,
    }
}

pub fn is_waiting_status(status_text: Option<&str>) -> bool {
    status_text
        .unwrap_or("")
        .eq_ignore_ascii_case("Waiting")
}