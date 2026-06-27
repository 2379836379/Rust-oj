use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProblemPageInfo {
    pub problem_url: String,
    pub title: Option<String>,
    pub submit_url: Option<String>,
    pub time_limit: Option<String>,
    pub memory_limit: Option<String>,
    pub description: Option<String>,
    pub starter_code: Option<String>,
    pub input_spec: Option<String>,
    pub output_spec: Option<String>,
    pub sample_input: Option<String>,
    pub sample_output: Option<String>,
    pub hint: Option<String>,
    pub tried_people: u32,
    pub passed_people: u32,
}

fn strip_html(html: &str) -> String {
    static TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<[^>]+>").unwrap());
    let no_tags = TAG_RE.replace_all(html, " ");
    no_tags.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extract_dl_value(html: &str, key: &str) -> Option<String> {
    let pattern = format!(
        r#"(?is)<dt>\s*{}\s*</dt>\s*<dd>([\s\S]*?)</dd>"#,
        regex::escape(key)
    );
    let re = Regex::new(&pattern).ok()?;
    let caps = re.captures(html)?;
    let v = caps.get(1)?.as_str();
    let v = strip_html(v);
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

fn extract_dl_value_any(html: &str, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(v) = extract_dl_value(html, k) {
            return Some(v);
        }
    }
    None
}

fn extract_section_html(html: &str, key: &str) -> Option<String> {
    // Rust `regex` does not support look-around. Find the <dt>...</dt> marker, then
    // slice until the next <dt> or the end of the <dl>.
    static NEXT_DT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?is)<dt\b"#).unwrap());
    static DL_END_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?is)</dl>"#).unwrap());

    let pattern = format!(r#"(?is)<dt>\s*{}\s*</dt>"#, regex::escape(key));
    let re = Regex::new(&pattern).ok()?;
    let m = re.find(html)?;

    let tail = &html[m.end()..];
    let mut end = tail.len();
    if let Some(m2) = NEXT_DT_RE.find(tail) {
        end = end.min(m2.start());
    }
    if let Some(m2) = DL_END_RE.find(tail) {
        end = end.min(m2.start());
    }

    let v = &tail[..end];
    if v.trim().is_empty() {
        None
    } else {
        Some(v.to_string())
    }
}

fn extract_section_html_any(html: &str, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(v) = extract_section_html(html, k) {
            return Some(v);
        }
    }
    None
}

fn extract_pre_blocks_text(html: &str) -> Option<String> {
    static PRE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?is)<pre[^>]*>([\s\S]*?)</pre>"#).unwrap());

    let mut blocks: Vec<String> = Vec::new();
    for caps in PRE_RE.captures_iter(html) {
        let mut block = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        block = block.replace("\r\n", "\n").replace('\r', "\n");
        block = block
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&");
        let trimmed = block.trim();
        if !trimmed.is_empty() {
            blocks.push(trimmed.to_string());
        }
    }

    if blocks.is_empty() {
        None
    } else {
        Some(blocks.join("\n\n"))
    }
}

pub fn parse_problem_page(html: &str, base_url: &Url) -> ProblemPageInfo {
    let title = {
        static TITLE_RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r#"(?is)<div\s+id=\"pageTitle\"[^>]*>\s*<h2>([^<]+)</h2>"#).unwrap()
        });
        TITLE_RE
            .captures(html)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty())
    };

    let submit_url = {
        static SUBMIT_RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r#"(?is)<a\s+href=\"([^\"]+/submit/)\"[^>]*>"#).unwrap()
        });
        SUBMIT_RE
            .captures(html)
            .and_then(|c| c.get(1))
            .and_then(|m| base_url.join(m.as_str()).ok())
            .map(|u| u.to_string())
    };

    let time_limit = extract_dl_value_any(html, &[
        "总时间限制:",
        "总时间限制：",
        "总时间限制",
        "Time Limit:",
        "Time Limit",
    ]);
    let memory_limit = extract_dl_value_any(html, &[
        "内存限制:",
        "内存限制：",
        "内存限制",
        "Memory Limit:",
        "Memory Limit",
    ]);

    let description_keys = ["描述", "Description"];
    let description = extract_dl_value_any(html, &description_keys);
    let starter_code = extract_section_html_any(html, &description_keys).and_then(|sec| extract_pre_blocks_text(&sec));

    let input_spec = extract_dl_value_any(html, &["输入", "Input"]);
    let output_spec = extract_dl_value_any(html, &["输出", "Output"]);
    let sample_input = extract_dl_value_any(html, &["样例输入", "Sample Input"]);
    let sample_output = extract_dl_value_any(html, &["样例输出", "Sample Output"]);
    let hint = extract_dl_value_any(html, &["提示", "Hint"]);

    let tried_people = extract_dl_value_any(html, &["尝试人数", "Tried"])
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);
    let passed_people = extract_dl_value_any(html, &["通过人数", "Passed"])
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    ProblemPageInfo {
        problem_url: base_url.to_string(),
        title,
        submit_url,
        time_limit,
        memory_limit,
        description,
        starter_code,
        input_spec,
        output_spec,
        sample_input,
        sample_output,
        hint,
        tried_people,
        passed_people,
    }
}