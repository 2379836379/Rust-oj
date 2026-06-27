use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClassPageInfo {
    pub class_page_url: String,
    pub group_entry_url: Option<String>,
    pub course_name: Option<String>,
}

pub fn parse_class_page(html: &str, base_url: &Url) -> ClassPageInfo {
    // Qt regex requires a "前往小组" link, but that link may disappear/shift on refresh.
    // Keep it robust: capture the <a href> that wraps the group logo image.
    static GROUP_LOGO_A_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"(?is)<a\s+href=['\"]([^'\"]+)['\"][^>]*>\s*(<img[^>]*>)\s*</a>"#,
        )
        .unwrap()
    });

    static ALT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?is)\balt=['\"]([^'\"]*)['\"]"#).unwrap());

    let mut info = ClassPageInfo {
        class_page_url: base_url.to_string(),
        group_entry_url: None,
        course_name: None,
    };

    for caps in GROUP_LOGO_A_RE.captures_iter(html) {
        let href = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
        let img_tag = caps.get(2).map(|m| m.as_str()).unwrap_or_default();

        if !img_tag.contains("group-logo") {
            continue;
        }

        if let Ok(u) = base_url.join(href) {
            info.group_entry_url = Some(u.to_string());
        }

        let name = ALT_RE
            .captures(img_tag)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty());
        if name.is_some() {
            info.course_name = name;
        }

        break;
    }

    info
}