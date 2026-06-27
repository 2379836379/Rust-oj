use once_cell::sync::Lazy;
use regex::Regex;

use crate::cache::ContestCacheRepository;
use crate::parser::{ClassPageInfo, GroupPageInfo};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeadlineReminder {
    pub course_name: String,
    pub contest_title: String,
    pub contest_url: String,
    pub deadline_text: String,
    pub deadline_epoch_ms: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AlarmTrigger {
    pub reminder: DeadlineReminder,
    pub hours_before: i32,
}

pub fn reminder_key(reminder: &DeadlineReminder) -> String {
    let url = reminder.contest_url.trim();
    if !url.is_empty() {
        return url.to_string();
    }
    format!(
        "{}|{}|{}",
        reminder.course_name,
        reminder.contest_title,
        reminder.deadline_epoch_ms
    )
}

pub fn trigger_hours_before(reminder: &DeadlineReminder, now_ms: i64) -> i32 {
    let deadline_ms = reminder.deadline_epoch_ms;
    if deadline_ms <= 0 {
        return 0;
    }

    let diff_ms = deadline_ms - now_ms;
    if diff_ms < 0 {
        return 0;
    }

    let diff_sec = diff_ms / 1000;
    if diff_sec < 60 * 60 {
        return 1;
    }
    if diff_sec < 2 * 60 * 60 {
        return 2;
    }
    if diff_sec < 3 * 60 * 60 {
        return 3;
    }

    0
}
fn is_contest_item(item_class: &str) -> bool {
    item_class.contains("contest-info")
}

pub fn now_epoch_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn parse_deadline_epoch_ms(text: &str) -> Option<i64> {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new("(\\d{4})[-/](\\d{1,2})[-/](\\d{1,2})\\s+(\\d{1,2}):(\\d{2})").unwrap()
    });

    let caps = RE.captures(text.trim())?;
    let y: i32 = caps.get(1)?.as_str().parse().ok()?;
    let m: u32 = caps.get(2)?.as_str().parse().ok()?;
    let d: u32 = caps.get(3)?.as_str().parse().ok()?;
    let hh: u32 = caps.get(4)?.as_str().parse().ok()?;
    let mm: u32 = caps.get(5)?.as_str().parse().ok()?;

    let days_from_civil = |y: i32, m: u32, d: u32| -> i64 {
        let mut y = y as i64;
        let m = m as i64;
        let d = d as i64;
        y -= if m <= 2 { 1 } else { 0 };
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = y - era * 400;
        let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146097 + doe - 719468
    };

    let days = days_from_civil(y, m, d);
    let seconds = days * 86400 + (hh as i64) * 3600 + (mm as i64) * 60;

    // Interpret OpenJudge deadline text as UTC+8 local time.
    let utc_seconds = seconds - 8 * 3600;
    Some(utc_seconds * 1000)
}

pub fn collect_reminders(
    class_info: &ClassPageInfo,
    group_info: &GroupPageInfo,
    now_ms: i64,
    one_week_ms: i64,
) -> Vec<DeadlineReminder> {
    let mut out = Vec::new();

    for cs in &group_info.contest_sets {
        if !is_contest_item(&cs.item_class) {
            continue;
        }

        let Some(deadline_text) = cs.end_time.clone() else {
            continue;
        };
        let Some(deadline_ms) = parse_deadline_epoch_ms(&deadline_text) else {
            continue;
        };
        let diff = deadline_ms - now_ms;
        if diff < 0 || diff >= one_week_ms {
            continue;
        }

        out.push(DeadlineReminder {
            course_name: class_info.course_name.clone().unwrap_or_default(),
            contest_title: cs.title.clone(),
            contest_url: cs.url.clone(),
            deadline_text,
            deadline_epoch_ms: deadline_ms,
        });
    }

    out
}

pub fn post_process(mut reminders_by_class: Vec<(String, Vec<DeadlineReminder>)>) -> Vec<DeadlineReminder> {
    let mut combined: Vec<DeadlineReminder> = reminders_by_class
        .drain(..)
        .flat_map(|(_, v)| v)
        .collect();

    combined.sort_by_key(|r| r.deadline_epoch_ms);

    // dedupe by contest_url
    let mut deduped: Vec<DeadlineReminder> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for r in combined {
        let u = r.contest_url.trim().to_string();
        if !u.is_empty() && seen.iter().any(|s| s == &u) {
            continue;
        }
        if !u.is_empty() {
            seen.push(u);
        }
        deduped.push(r);
    }

    // hide completed contests (Qt uses contest cache)
    let mut filtered: Vec<DeadlineReminder> = Vec::new();
    for r in deduped {
        if let Ok(Some(contest)) = ContestCacheRepository::load_contest(&r.contest_url) {
            if contest.total_problems > 0 && contest.solved_problems >= contest.total_problems {
                continue;
            }
        }
        filtered.push(r);
    }

    filtered
}
