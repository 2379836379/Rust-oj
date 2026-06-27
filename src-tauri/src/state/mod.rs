use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Mutex;

use cookie_store::CookieStore;
use directories::ProjectDirs;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::network::OpenJudgeSession;
use crate::parser::{
    build_submit_payload, extract_joined_classes, extract_personal_home_url, is_waiting_status,
    parse_class_page, parse_contest_page, parse_group_page, parse_problem_page, parse_result_page,
    parse_submit_page, ClassPageInfo, ContestPageInfo, GroupPageInfo, JoinedClassInfo,
    ProblemPageInfo, ResultPageInfo, SubmitPageInfo,
};
use crate::cache::{ClassCacheRepository, ContestCacheRepository, ProblemCacheRepository};
use crate::storage::LoginCache;

mod reminder;

pub use reminder::{AlarmTrigger, DeadlineReminder};

const APP_QUALIFIER: &str = "com";
const APP_ORG: &str = "openjudge";
const APP_NAME: &str = "oj-client";

const OPENJUDGE_BASE_URL: &str = "http://openjudge.cn";
const OJ_JUDGER_BASE_URL: &str = "http://10.129.240.62:18080";

pub struct AppCtx {
    openjudge: Mutex<OpenJudgeState>,
    alarm_triggered: Mutex<HashSet<String>>,
    tray_close_notified: Mutex<bool>,
}

impl Default for AppCtx {
    fn default() -> Self {
        Self {
            openjudge: Mutex::new(OpenJudgeState::default()),
            alarm_triggered: Mutex::new(HashSet::new()),
            tray_close_notified: Mutex::new(false),
        }
    }
}
#[derive(Clone)]
struct OpenJudgeState {
    base_url: Url,
    personal_home_url: Option<Url>,
    verified_email: Option<String>,
    session: Option<OpenJudgeSession>,
}

impl Default for OpenJudgeState {
    fn default() -> Self {
        Self {
            base_url: Url::parse(OPENJUDGE_BASE_URL).expect("valid default base url"),
            personal_home_url: None,
            verified_email: None,
            session: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenJudgeLoginResult {
    pub personal_home_url: String,
    pub classes: Vec<JoinedClassInfo>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugJoinedClassesHtml {
    pub requested_url: String,
    pub final_url: String,
    pub html_len: usize,
    pub html_head: String,
    pub group_name_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenClassResult {
    pub class_info: ClassPageInfo,
    pub group_info: GroupPageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitResponse {
    pub ok: bool,
    pub status_code: u16,
    pub final_url: String,
    pub inferred_result_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeResponse {
    pub ok: bool,
    pub status_code: u16,
    pub body: String,
}

fn config_dir() -> Result<PathBuf, String> {
    let dirs = ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME)
        .ok_or_else(|| "Failed to resolve user config directory".to_string())?;
    Ok(dirs.config_dir().to_path_buf())
}

fn cookies_path() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("openjudge_cookies.json"))
}

fn load_cookies(path: &PathBuf) -> Result<CookieStore, String> {
    if !path.is_file() {
        return Ok(CookieStore::default());
    }
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))
}

fn save_cookies(session: &OpenJudgeSession) -> Result<(), String> {
    let path = cookies_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("create_dir_all {}: {e}", parent.display()))?;
    }

    let cookies = session.cookie_store();
    let store = cookies
        .lock()
        .map_err(|_| "cookie store poisoned".to_string())?;
    let text = serde_json::to_string_pretty(&*store).map_err(|e| format!("cookie serialize: {e}"))?;
    std::fs::write(&path, text).map_err(|e| format!("write {}: {e}", path.display()))
}

fn build_session(base_url: Url) -> Result<OpenJudgeSession, String> {
    let cookie_path = cookies_path()?;
    let cookies = load_cookies(&cookie_path)?;
    OpenJudgeSession::new(base_url, cookies)
}

fn json_error_message(body: &[u8]) -> Option<String> {
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(body) else {
        return None;
    };
    v.get("message").and_then(|m| m.as_str()).map(|s| s.to_string())
}

fn infer_result_url(final_url: &Url, body: &str) -> Option<Url> {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"/submission/\d+/?").unwrap());

    if final_url.path().contains("/submission/") {
        return Some(final_url.clone());
    }

    if let Some(m) = RE.find(body) {
        let rel = &body[m.start()..m.end()];
        return final_url.join(rel).ok();
    }

    None
}
fn is_login_html(html: &str) -> bool {
    html.contains("OpenJudge - Login")
        || html.contains("你还没有登录")
        || html.contains("/api/auth/login/")
}

impl AppCtx {
    fn lock_openjudge(&self) -> Result<std::sync::MutexGuard<'_, OpenJudgeState>, String> {
        self.openjudge.lock().map_err(|_| "state poisoned".to_string())
    }

    fn openjudge_session(&self) -> Result<OpenJudgeSession, String> {
        // Prefer the in-memory session so we keep session cookies (Qt uses one cookie jar).
        let base_url = {
            let st = self.lock_openjudge()?;
            if let Some(sess) = st.session.clone() {
                return Ok(sess);
            }
            st.base_url.clone()
        };

        let session = build_session(base_url)?;
        {
            let mut st = self.lock_openjudge()?;
            if st.session.is_none() {
                st.session = Some(session.clone());
            }
        }
        Ok(session)
    }

    async fn relogin_with_session(&self, session: &OpenJudgeSession) -> Result<bool, String> {
        let email = {
            let st = self.lock_openjudge()?;
            st.verified_email.clone()
        };
        let Some(email) = email else {
            return Ok(false);
        };

        let record = LoginCache::load_by_email(&email)
            .map_err(|e| format!("load login cache: {e}"))?;
        let Some(record) = record else {
            return Ok(false);
        };

        let resp = session.post_login(&record.email, &record.password).await?;
        let status = resp.status();
        let body = resp
            .bytes()
            .await
            .map_err(|e| format!("read login body: {e}"))?;

        let success = status.is_success()
            && body
                .windows(b"\"result\":\"SUCCESS\"".len())
                .any(|w| w == b"\"result\":\"SUCCESS\"");
        if !success {
            return Ok(false);
        }

        let (home_final, home_html) = session.get_html(session.base_url().clone(), None).await?;
        if let Some(personal) = extract_personal_home_url(&home_html, &home_final) {
            if let Ok(u) = Url::parse(&personal) {
                let mut st = self.lock_openjudge()?;
                st.personal_home_url = Some(u);
                st.verified_email = Some(record.email);
                st.session = Some(session.clone());
            }
        }

        let _ = save_cookies(session);
        Ok(true)
    }

    async fn get_html_authed(&self, url: Url, referer: Option<&Url>) -> Result<(Url, String), String> {
        let session = self.openjudge_session()?;
        let (final_url, html) = session.get_html(url.clone(), referer).await?;
        if is_login_html(&html) {
            let relogged = self.relogin_with_session(&session).await.unwrap_or(false);
            if relogged {
                return session.get_html(url, referer).await;
            }
        }
        Ok((final_url, html))
    }

    pub fn requires_email_verification(&self, email: &str) -> bool {
        let normalized = email.trim();
        if normalized.is_empty() {
            return false;
        }

        let verified = self
            .openjudge
            .lock()
            .ok()
            .and_then(|s| s.verified_email.clone());
        if verified.as_deref() == Some(normalized) {
            return false;
        }

        match LoginCache::load_by_email(normalized) {
            Ok(Some(_)) => false,
            _ => true,
        }
    }

    pub fn login_cache_last(&self) -> Result<Option<crate::storage::LoginRecord>, String> {
        LoginCache::load_last_login()
    }

    pub fn login_cache_lookup(
        &self,
        email: String,
    ) -> Result<Option<crate::storage::LoginRecord>, String> {
        LoginCache::load_by_email(&email)
    }

    pub async fn email_send_code(&self, email: String) -> Result<String, String> {
        crate::auth::send_code(&email).await
    }

    pub async fn email_verify_code(&self, email: String, code: String) -> Result<(), String> {
        crate::auth::verify_code(&email, &code).await?;
        let mut st = self.lock_openjudge()?;
        st.verified_email = Some(email.trim().to_string());
        Ok(())
    }

    pub fn logout(&self) -> Result<(), String> {
        let mut st = self.lock_openjudge()?;
        st.personal_home_url = None;
        st.session = None;
        Ok(())
    }

    pub async fn login(&self, email: String, password: String) -> Result<OpenJudgeLoginResult, String> {
        let email = email.trim().to_string();
        if email.is_empty() || password.is_empty() {
            return Err("Email and password are required.".to_string());
        }

        let base_url = { self.lock_openjudge()?.base_url.clone() };
        let session = build_session(base_url.clone())?;

        let resp = session.post_login(&email, &password).await?;
        let status = resp.status();
        let body = resp
            .bytes()
            .await
            .map_err(|e| format!("read login body: {e}"))?;

        let success = status.is_success()
            && body
                .windows(b"\"result\":\"SUCCESS\"".len())
                .any(|w| w == b"\"result\":\"SUCCESS\"");
        if !success {
            let msg = json_error_message(&body)
                .or_else(|| String::from_utf8(body.to_vec()).ok())
                .unwrap_or_else(|| format!("login failed ({status})"));
            return Err(msg);
        }

        let (home_final, home_html) = session.get_html(base_url.clone(), None).await?;
        let personal = extract_personal_home_url(&home_html, &home_final)
            .ok_or_else(|| "Login succeeded, but personal home URL was not found.".to_string())?;
        let personal_url = Url::parse(&personal)
            .map_err(|e| format!("invalid personal home url: {e}"))?;

        let (user_final, user_html) = session
            .get_html(personal_url.clone(), Some(&home_final))
            .await?;
        let classes = extract_joined_classes(&user_html, &user_final);

        save_cookies(&session)?;
        LoginCache::save_login(&email, &password)?;

        {
            let mut st = self.lock_openjudge()?;
            st.personal_home_url = Some(personal_url);
            st.verified_email = Some(email);
            st.session = Some(session.clone());
        }

        Ok(OpenJudgeLoginResult {
            personal_home_url: personal,
            classes,
        })
    }

    pub async fn get_joined_classes(&self) -> Result<Vec<JoinedClassInfo>, String> {
        let personal_url = {
            let st = self.lock_openjudge()?;
            let Some(personal_url) = st.personal_home_url.clone() else {
                return Err("Not logged in".to_string());
            };
            personal_url
        };

        let requested_url = personal_url.to_string();
        let (final_url, html) = self.get_html_authed(personal_url, None).await?;

        let classes = extract_joined_classes(&html, &final_url);
        if classes.is_empty() {
            let html_len = html.len();
            let html_head: String = html.chars().take(4000).collect();
            let group_name_count = html.matches("group-name").count();

            println!(
                "[debug] oj_get_joined_classes empty: requested={} final={} len={} group-name={}\n---- html_head ----\n{}\n---- /html_head ----",
                requested_url,
                final_url,
                html_len,
                group_name_count,
                html_head
            );
        }

        Ok(classes)
    }


    pub async fn debug_get_joined_classes_html(&self) -> Result<DebugJoinedClassesHtml, String> {
        let personal_url = {
            let st = self.lock_openjudge()?;
            let Some(personal_url) = st.personal_home_url.clone() else {
                return Err("Not logged in".to_string());
            };
            personal_url
        };

        let requested_url = personal_url.to_string();
        let (final_url, html) = self.get_html_authed(personal_url, None).await?;

        let html_len = html.len();
        let html_head: String = html.chars().take(4000).collect();
        let group_name_count = html.matches("group-name").count();

        println!(
            "[debug] joined classes fetch: requested={} final={} len={} group-name={}",
            requested_url,
            final_url,
            html_len,
            group_name_count
        );

        Ok(DebugJoinedClassesHtml {
            requested_url,
            final_url: final_url.to_string(),
            html_len,
            html_head,
            group_name_count,
        })
    }
    pub async fn open_class(&self, class_page_url: String) -> Result<OpenClassResult, String> {
        let class_url = Url::parse(&class_page_url)
            .map_err(|e| format!("invalid class url: {e}"))?;
        let (class_final, class_html) = self.get_html_authed(class_url, None).await?;
        let class_info = parse_class_page(&class_html, &class_final);
        if class_info.group_entry_url.is_none() {
            let html_len = class_html.len();
            let html_head: String = class_html.chars().take(4000).collect();
            let has_group_text = class_html.contains("前往小组");
            let has_login = class_html.contains("OpenJudge - Login") || class_html.contains("你还没有登录");
            println!(
                "[debug] open_class missing group entry: requested={} final={} len={} has_group_text={} has_login={}\n---- html_head ----\n{}\n---- /html_head ----",
                class_page_url,
                class_final,
                html_len,
                has_group_text,
                has_login,
                html_head
            );
        }
        let group_entry = class_info
            .group_entry_url
            .clone()
            .ok_or_else(|| "group entry url not found".to_string())?;
        let group_url = Url::parse(&group_entry).map_err(|e| format!("invalid group url: {e}"))?;
        let (group_final, group_html) = self.get_html_authed(group_url, Some(&class_final)).await?;
        let group_info = parse_group_page(&group_html, &group_final);

        let _ = ClassCacheRepository::save_class(&class_info, &group_info);

        Ok(OpenClassResult {
            class_info,
            group_info,
        })
    }

    pub async fn open_contest(&self, contest_page_url: String) -> Result<ContestPageInfo, String> {
        let contest_url = Url::parse(&contest_page_url)
            .map_err(|e| format!("invalid contest url: {e}"))?;

        match self.get_html_authed(contest_url, None).await {
            Ok((final_url, html)) => {
                let parsed = parse_contest_page(&html, &final_url);
                let _ = ContestCacheRepository::save_contest(&parsed);
                Ok(parsed)
            }
            Err(err) => match ContestCacheRepository::load_contest(&contest_page_url) {
                Ok(Some(cached)) => Ok(cached),
                _ => Err(err),
            },
        }
    }

    pub async fn open_problem(&self, problem_url: String) -> Result<ProblemPageInfo, String> {
        let url = Url::parse(&problem_url).map_err(|e| format!("invalid problem url: {e}"))?;

        match self.get_html_authed(url, None).await {
            Ok((final_url, html)) => {
                let parsed = parse_problem_page(&html, &final_url);
                let _ = ProblemCacheRepository::save_problem(&parsed);
                Ok(parsed)
            }
            Err(err) => match ProblemCacheRepository::load_problem(&problem_url) {
                Ok(Some(cached)) => Ok(cached),
                _ => Err(err),
            },
        }
    }

    pub async fn open_submit(&self, submit_page_url: String) -> Result<SubmitPageInfo, String> {
        let url = Url::parse(&submit_page_url).map_err(|e| format!("invalid submit url: {e}"))?;
        let (final_url, html) = self.get_html_authed(url, None).await?;
        Ok(parse_submit_page(&html, &final_url))
    }

    pub async fn submit_solution(
        &self,
        submit_page: SubmitPageInfo,
        language: String,
        source_text: String,
    ) -> Result<SubmitResponse, String> {
        let session = self.openjudge_session()?;


        let action = submit_page
            .submit_action_url
            .clone()
            .ok_or_else(|| "submit action url not found".to_string())?;
        let action_url = Url::parse(&action).map_err(|e| format!("invalid submit action url: {e}"))?;

        let referer_url = Url::parse(&submit_page.page_url)
            .map_err(|e| format!("invalid submit page url: {e}"))?;

        let payload = build_submit_payload(&submit_page, &language, &source_text)?;

        let (status_code, final_url, body) = session
            .post_form(action_url, payload, Some(&referer_url), true)
            .await?;

        save_cookies(&session)?;

        let inferred = infer_result_url(&final_url, &body).map(|u| u.to_string());

        Ok(SubmitResponse {
            ok: status_code >= 200 && status_code < 400,
            status_code,
            final_url: final_url.to_string(),
            inferred_result_url: inferred,
        })
    }

    pub async fn open_result(&self, result_page_url: String) -> Result<ResultPageInfo, String> {
        let url = Url::parse(&result_page_url).map_err(|e| format!("invalid result url: {e}"))?;
        let (final_url, html) = self.get_html_authed(url, None).await?;
        Ok(parse_result_page(&html, &final_url))
    }

    pub fn result_is_waiting(info: &ResultPageInfo) -> bool {
        is_waiting_status(info.status_text.as_deref())
    }

    pub async fn due_soon_reminders(
        &self,
        classes: Vec<JoinedClassInfo>,
    ) -> Result<Vec<DeadlineReminder>, String> {
        let now_ms = reminder::now_epoch_ms();
        let one_week_ms: i64 = 7 * 24 * 60 * 60 * 1000;

        let mut reminders_by_class: Vec<(String, Vec<DeadlineReminder>)> = Vec::new();

        for joined in classes {
            if let Ok(Some((cached_class, cached_group))) = ClassCacheRepository::load_class(&joined.url) {
                let r = reminder::collect_reminders(&cached_class, &cached_group, now_ms, one_week_ms);
                reminders_by_class.push((joined.url.clone(), r));
            }

            let class_url = match Url::parse(&joined.url) {
                Ok(u) => u,
                Err(_) => continue,
            };
            let (class_final, class_html) = match self.get_html_authed(class_url, None).await {
                Ok(v) => v,
                Err(_) => continue,
            };
            let class_info = parse_class_page(&class_html, &class_final);
            let Some(group_entry) = class_info.group_entry_url.clone() else {
                continue;
            };
            let group_url = match Url::parse(&group_entry) {
                Ok(u) => u,
                Err(_) => continue,
            };
            let (group_final, group_html) = match self.get_html_authed(group_url, Some(&class_final)).await {
                Ok(v) => v,
                Err(_) => continue,
            };
            let group_info = parse_group_page(&group_html, &group_final);
            let _ = ClassCacheRepository::save_class(&class_info, &group_info);

            let r = reminder::collect_reminders(&class_info, &group_info, now_ms, one_week_ms);
            let mut replaced = false;
            for (url, list) in reminders_by_class.iter_mut() {
                if url == &joined.url {
                    *list = r.clone();
                    replaced = true;
                    break;
                }
            }
            if !replaced {
                reminders_by_class.push((joined.url.clone(), r));
            }
        }

        Ok(reminder::post_process(reminders_by_class))
    }


    pub fn alarm_process_reminders(
        &self,
        reminders: Vec<DeadlineReminder>,
    ) -> Result<Vec<AlarmTrigger>, String> {
        let now_ms = reminder::now_epoch_ms();
        let mut out: Vec<AlarmTrigger> = Vec::new();

        let mut triggered = self
            .alarm_triggered
            .lock()
            .map_err(|_| "alarm triggered set poisoned".to_string())?;

        for r in reminders {
            let hours_before = reminder::trigger_hours_before(&r, now_ms);
            if hours_before <= 0 {
                continue;
            }

            let key = format!("{}|{}h", reminder::reminder_key(&r), hours_before);
            if triggered.contains(&key) {
                continue;
            }

            triggered.insert(key);
            out.push(AlarmTrigger {
                reminder: r,
                hours_before,
            });
        }

        Ok(out)
    }

    pub fn tray_take_first_close_notification(&self) -> bool {
        let mut flagged = match self.tray_close_notified.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };

        if *flagged {
            return false;
        }

        *flagged = true;
        true
    }

    pub async fn judge_source(
        &self,
        language: String,
        file_name: String,
        source_code: String,
        stdin_text: String,
    ) -> Result<JudgeResponse, String> {
        let base = Url::parse(OJ_JUDGER_BASE_URL)
            .map_err(|e| format!("invalid judger base_url: {e}"))?;
        let url = base.join("/judge").map_err(|e| format!("join /judge: {e}"))?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| format!("reqwest build: {e}"))?;

        let form = reqwest::multipart::Form::new()
            .text("language", language)
            .text("stdin", stdin_text)
            .text("time_limit_ms", "2000")
            .text("memory_limit_mb", "256")
            .part(
                "file",
                reqwest::multipart::Part::text(source_code)
                    .file_name(file_name)
                    .mime_str("text/plain")
                    .map_err(|e| format!("mime: {e}"))?,
            );

        let resp = client
            .post(url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("judge request: {e}"))?;

        let status = resp.status().as_u16();
        let text = resp
            .text()
            .await
            .map_err(|e| format!("read judge body: {e}"))?;

        Ok(JudgeResponse {
            ok: status >= 200 && status < 400,
            status_code: status,
            body: text,
        })
    }
}


