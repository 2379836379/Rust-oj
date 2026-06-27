use std::sync::Arc;

use cookie_store::CookieStore;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE, REFERER, USER_AGENT};
use reqwest::{Client, Response};
use reqwest_cookie_store::CookieStoreMutex;
use url::Url;

const DEFAULT_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";

#[derive(Clone)]
pub struct OpenJudgeSession {
    base_url: Url,
    client: Client,
    cookies: Arc<CookieStoreMutex>,
}

impl OpenJudgeSession {
    pub fn new(base_url: Url, cookies: CookieStore) -> Result<Self, String> {
        let cookies = Arc::new(CookieStoreMutex::new(cookies));

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(DEFAULT_UA));
        headers.insert(
            ACCEPT,
            HeaderValue::from_static(
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            ),
        );
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .cookie_provider(cookies.clone())
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .map_err(|e| format!("reqwest client build: {e}"))?;

        Ok(Self {
            base_url,
            client,
            cookies,
        })
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub fn cookie_store(&self) -> Arc<CookieStoreMutex> {
        self.cookies.clone()
    }

    pub async fn post_login(&self, email: &str, password: &str) -> Result<Response, String> {
        let url = self
            .base_url
            .join("/api/auth/login/")
            .map_err(|e| format!("join login url: {e}"))?;

        let redirect_url = self.base_url.to_string();
        let body = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("email", email)
            .append_pair("password", password)
            .append_pair("redirectUrl", &redirect_url)
            .finish();

        self.client
            .post(url)
            .body(body)
            .send()
            .await
            .map_err(|e| format!("login request: {e}"))
    }

    pub async fn get_html(&self, url: Url, referer: Option<&Url>) -> Result<(Url, String), String> {
                let mut req = self.client.get(url.clone());
        let ref_value = referer
            .map(|u| u.to_string())
            .unwrap_or_else(|| self.base_url.to_string());
        req = req.header(REFERER, ref_value);

        let resp = req
            .send()
            .await
            .map_err(|e| format!("get request: {e}"))?;
        let final_url = resp.url().clone();
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| format!("read body: {e}"))?;
        let html = String::from_utf8_lossy(&bytes).to_string();
        Ok((final_url, html))
    }

    pub async fn post_form(
        &self,
        url: Url,
        body: String,
        referer: Option<&Url>,
        ajax: bool,
    ) -> Result<(u16, Url, String), String> {
        let mut req = self.client.post(url).body(body);
        let ref_value = referer
            .map(|u| u.to_string())
            .unwrap_or_else(|| self.base_url.to_string());
        req = req.header(REFERER, ref_value);
        if ajax {
            req = req.header("X-Requested-With", "XMLHttpRequest");
        }

        let resp = req
            .send()
            .await
            .map_err(|e| format!("post request: {e}"))?;
        let status = resp.status().as_u16();
        let final_url = resp.url().clone();
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| format!("read body: {e}"))?;
        let text = String::from_utf8_lossy(&bytes).to_string();
        Ok((status, final_url, text))
    }
}
