use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContestSetInfo {
    pub url: String,
    pub title: String,
    pub item_class: String,
    pub problem_number: Option<String>,
    pub end_time: Option<String>,
    pub extra_text: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GroupPageInfo {
    pub group_page_url: String,
    pub contest_sets: Vec<ContestSetInfo>,
}

fn strip_html(html: &str) -> String {
    static TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<[^>]+>").unwrap());
    let no_tags = TAG_RE.replace_all(html, " ");
    no_tags.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn parse_group_page(html: &str, base_url: &Url) -> GroupPageInfo {
    static LIST_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"<ul\s+class=\"current-contest\s+label\"[^>]*>([\s\S]*?)</ul>"#)
            .unwrap()
    });
    static ITEM_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"<li\s+class=\"([^\"]*(?:contest-info|practice-info)[^\"]*)\"[^>]*>([\s\S]*?)</li>"#,
        )
        .unwrap()
    });
    static LINK_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"<h3>[\s\S]*?<a\s+href=\"([^\"]+)\">([^<]+)</a>([\s\S]*?)</h3>"#)
            .unwrap()
    });
    static SPAN_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"<span\s+class=\"([^\"]+)\"[^>]*>([\s\S]*?)</span>"#).unwrap()
    });

    let mut info = GroupPageInfo {
        group_page_url: base_url.to_string(),
        contest_sets: Vec::new(),
    };

    let Some(list_caps) = LIST_RE.captures(html) else {
        return info;
    };
    let list_html = list_caps.get(1).map(|m| m.as_str()).unwrap_or_default();

    for item_caps in ITEM_RE.captures_iter(list_html) {
        let item_class = item_caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();
        let item_html = item_caps.get(2).map(|m| m.as_str()).unwrap_or("");

        let Some(link_caps) = LINK_RE.captures(item_html) else {
            continue;
        };
        let href = link_caps.get(1).map(|m| m.as_str()).unwrap_or_default();
        let title = link_caps.get(2).map(|m| m.as_str()).unwrap_or("").trim();
        let tail_html = link_caps.get(3).map(|m| m.as_str()).unwrap_or("");

        let Ok(url) = base_url.join(href) else {
            continue;
        };

        let problem_number = {
            let t = strip_html(tail_html);
            if t.is_empty() { None } else { Some(t) }
        };

        let mut end_time: Option<String> = None;
        for span_caps in SPAN_RE.captures_iter(item_html) {
            let cls = span_caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            let text = strip_html(span_caps.get(2).map(|m| m.as_str()).unwrap_or(""));
            if (cls == "over-time" || cls == "recently-update") && !text.is_empty() {
                end_time = Some(text);
            }
        }

        info.contest_sets.push(ContestSetInfo {
            url: url.to_string(),
            title: title.to_string(),
            item_class: item_class.to_string(),
            problem_number,
            end_time,
            extra_text: None,
        });
    }

    info
}