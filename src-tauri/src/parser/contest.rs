use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContestProblemInfo {
    pub problem_id: String,
    pub title: String,
    pub problem_url: String,
    pub accept_people: u32,
    pub submission_people: u32,
    pub solved: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContestPageInfo {
    pub contest_page_url: String,
    pub problems: Vec<ContestProblemInfo>,
    pub total_problems: u32,
    pub solved_problems: u32,
}

pub fn parse_contest_page(html: &str, base_url: &Url) -> ContestPageInfo {
    static ROW_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"(?is)<tr(?:\s+class=\"alt\")?>([\s\S]*?)</tr>"#).unwrap()
    });
    static SOLVED_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"(?is)<td\s+class=\"solved\"[^>]*>[\s\S]*?accepted\.gif"#).unwrap()
    });
    static ID_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"(?is)<td\s+class=\"problem-id\"[^>]*>\s*<a\s+href=\"([^\"]+)\">([^<]+)</a>\s*</td>"#,
        )
        .unwrap()
    });
    static TITLE_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"(?is)<td\s+class=\"title\"[^>]*>\s*<a\s+href=\"([^\"]+)\">([^<]+)</a>\s*</td>"#,
        )
        .unwrap()
    });
    static ACCEPTED_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"(?is)<td\s+class=\"accepted\"[^>]*>[\s\S]*?>(\d+)</a>\s*</td>"#)
            .unwrap()
    });
    static SUBMISSIONS_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"(?is)<td\s+class=\"submissions\"[^>]*>[\s\S]*?>(\d+)</a>\s*</td>"#,
        )
        .unwrap()
    });

    let mut info = ContestPageInfo {
        contest_page_url: base_url.to_string(),
        problems: Vec::new(),
        total_problems: 0,
        solved_problems: 0,
    };

    for row_caps in ROW_RE.captures_iter(html) {
        let row_html = row_caps.get(1).map(|m| m.as_str()).unwrap_or_default();

        let Some(id_caps) = ID_RE.captures(row_html) else {
            continue;
        };
        let Some(title_caps) = TITLE_RE.captures(row_html) else {
            continue;
        };

        let solved = SOLVED_RE.is_match(row_html);
        let problem_id = id_caps.get(2).map(|m| m.as_str()).unwrap_or("").trim().to_string();
        let title = title_caps.get(2).map(|m| m.as_str()).unwrap_or("").trim().to_string();

        let href = title_caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let Ok(problem_url) = base_url.join(href) else {
            continue;
        };

        let accept_people = ACCEPTED_RE
            .captures(row_html)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<u32>().ok())
            .unwrap_or(0);
        let submission_people = SUBMISSIONS_RE
            .captures(row_html)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<u32>().ok())
            .unwrap_or(0);

        info.problems.push(ContestProblemInfo {
            problem_id,
            title,
            problem_url: problem_url.to_string(),
            accept_people,
            submission_people,
            solved,
        });

        info.total_problems += 1;
        if solved {
            info.solved_problems += 1;
        }
    }

    info
}