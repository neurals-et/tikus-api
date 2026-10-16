use std::time::Duration;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde_json::Value;

use crate::error::{Result, TikusError};
use crate::models::{AuthResponse, LiveStream, OtpResponse, PaginatedPosts, TikusPost};

pub const DEFAULT_BASE_URL: &str = "https://api.tikus.com/api/v1";
pub const DEFAULT_USER_AGENT: &str = "okhttp/4.12.0";

/// Asynchronous API Client for Tikus Ethiopian live streaming & social video platform.
#[derive(Clone, Debug)]
pub struct TikusClient {
    client: reqwest::Client,
    base_url: String,
    token: Option<String>,
}

impl Default for TikusClient {
    fn default() -> Self {
        Self::new()
    }
}

impl TikusClient {
    /// Create a new TikusClient with default configuration.
    pub fn new() -> Self {
        Self::builder().build().expect("Failed to build default TikusClient")
    }

    /// Obtain a client builder to configure base URL, timeout, proxy, and token.
    pub fn builder() -> TikusClientBuilder {
        TikusClientBuilder::new()
    }

    /// Set or update the bearer access token.
    pub fn set_token(&mut self, token: impl Into<String>) {
        self.token = Some(token.into());
    }

    /// Clear the active token.
    pub fn clear_token(&mut self) {
        self.token = None;
    }

    /// Current active token, if any.
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    // ---------------------------------------------------------------------------
    // Internal HTTP Request Helper
    // ---------------------------------------------------------------------------
    async fn request<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        query: Option<&[(&str, &str)]>,
        body: Option<Value>,
        auth_required: bool,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url.trim_end_matches('/'), path);
        let mut req = self.client.request(method, &url);

        if let Some(params) = query {
            req = req.query(params);
        }

        if let Some(token) = &self.token {
            req = req.header(AUTHORIZATION, format!("Bearer {}", token));
        } else if auth_required {
            return Err(TikusError::Authentication(format!(
                "Authentication required for {}. Set token first.",
                path
            )));
        }

        if let Some(json_body) = body {
            req = req.json(&json_body);
        }

        let resp = req.send().await?;
        let status = resp.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            let body_text = resp.text().await.unwrap_or_default();
            return Err(TikusError::Authentication(format!("Unauthorized: {}", body_text)));
        }

        if status == reqwest::StatusCode::UNPROCESSABLE_ENTITY {
            let body_text = resp.text().await.unwrap_or_default();
            return Err(TikusError::Validation(body_text));
        }

        if status == reqwest::StatusCode::NOT_FOUND {
            let body_text = resp.text().await.unwrap_or_default();
            return Err(TikusError::NotFound(body_text));
        }

        if !status.is_success() {
            let body_text = resp.text().await.unwrap_or_default();
            return Err(TikusError::Api {
                status: status.as_u16(),
                message: body_text,
            });
        }

        let parsed: T = resp.json().await?;
        Ok(parsed)
    }

    // ---------------------------------------------------------------------------
    // Public Feed & Videos
    // ---------------------------------------------------------------------------

    /// Fetch public trending posts with pagination cursor and filters.
    pub async fn get_public_posts(
        &self,
        tab: Option<&str>,
        cursor: Option<&str>,
        tag: Option<&str>,
    ) -> Result<PaginatedPosts> {
        let mut query: Vec<(&str, &str)> = Vec::new();
        if let Some(t) = tab {
            query.push(("tab", t));
        }
        if let Some(c) = cursor {
            query.push(("cursor", c));
        }
        if let Some(tg) = tag {
            query.push(("tag", tg));
        }

        let val: Value = self
            .request(
                reqwest::Method::GET,
                "/posts/public",
                if query.is_empty() { None } else { Some(&query) },
                None,
                false,
            )
            .await?;

        let posts_raw = val.get("data").and_then(|d| d.get("posts")).cloned().unwrap_or(Value::Array(vec![]));
        let cursor_val = val
            .get("data")
            .and_then(|d| d.get("cursor"))
            .and_then(|c| c.as_str())
            .map(|s| s.to_string());
        let has_more = val
            .get("data")
            .and_then(|d| d.get("has_more"))
            .and_then(|h| h.as_bool())
            .unwrap_or(false);

        let posts: Vec<TikusPost> = serde_json::from_value(posts_raw)?;
        Ok(PaginatedPosts {
            posts,
            cursor: cursor_val,
            has_more,
        })
    }

    // ---------------------------------------------------------------------------
    // Authentication & SMS OTP
    // ---------------------------------------------------------------------------

    /// Request SMS verification OTP code.
    pub async fn send_otp(&self, country_code: &str, phone_number: &str) -> Result<OtpResponse> {
        let body = serde_json::json!({
            "country_code": country_code,
            "phone_number": phone_number,
        });
        self.request(reqwest::Method::POST, "/send_otp", None, Some(body), false)
            .await
    }

    /// Verify 6-digit OTP code received via SMS.
    pub async fn verify_otp(
        &self,
        country_code: &str,
        phone_number: &str,
        otp: &str,
    ) -> Result<OtpResponse> {
        let body = serde_json::json!({
            "country_code": country_code,
            "phone_number": phone_number,
            "otp": otp,
        });
        let res: OtpResponse = self
            .request(reqwest::Method::POST, "/verify_otp", None, Some(body), false)
            .await?;

        if res.code.as_deref() == Some("OTP_INVALID") {
            return Err(TikusError::Otp(res.message.unwrap_or_else(|| "Invalid OTP code".to_string())));
        }
        Ok(res)
    }

    /// Register a new user account with verified OTP or registration token.
    pub async fn register(
        &mut self,
        first_name: &str,
        gender: &str,
        country_code: &str,
        phone_number: &str,
        otp: Option<&str>,
        registration_token: Option<&str>,
        last_name: Option<&str>,
    ) -> Result<AuthResponse> {
        let mut body = serde_json::json!({
            "first_name": first_name,
            "gender": gender,
            "country_code": country_code,
            "phone_number": phone_number,
        });

        if let Some(o) = otp {
            body["otp"] = serde_json::Value::String(o.to_string());
        }
        if let Some(rt) = registration_token {
            body["registration_token"] = serde_json::Value::String(rt.to_string());
        }
        if let Some(l) = last_name {
            body["last_name"] = serde_json::Value::String(l.to_string());
        }

        let auth: AuthResponse = self
            .request(reqwest::Method::POST, "/register", None, Some(body), false)
            .await?;

        if let Some(tok) = &auth.token {
            self.set_token(tok);
        } else if let Some(tok) = &auth.access_token {
            self.set_token(tok);
        }

        Ok(auth)
    }

    /// Authenticate using a Google Native Identity OAuth token.
    pub async fn login_with_google(&mut self, id_token: &str) -> Result<AuthResponse> {
        let body = serde_json::json!({ "id_token": id_token });
        let auth: AuthResponse = self
            .request(reqwest::Method::POST, "/auth/google/native", None, Some(body), false)
            .await?;

        if let Some(tok) = &auth.token {
            self.set_token(tok);
        } else if let Some(tok) = &auth.access_token {
            self.set_token(tok);
        }

        Ok(auth)
    }

    // ---------------------------------------------------------------------------
    // Protected Feeds, Streams & Economy (Auth Required)
    // ---------------------------------------------------------------------------

    /// Fetch personalized home feed.
    pub async fn get_home_feed(&self, cursor: Option<&str>) -> Result<PaginatedPosts> {
        let mut query: Vec<(&str, &str)> = Vec::new();
        if let Some(c) = cursor {
            query.push(("cursor", c));
        }

        let val: Value = self
            .request(
                reqwest::Method::GET,
                "/posts/feed",
                if query.is_empty() { None } else { Some(&query) },
                None,
                true,
            )
            .await?;

        let posts_raw = val.get("data").and_then(|d| d.get("posts")).cloned().unwrap_or(Value::Array(vec![]));
        let cursor_val = val
            .get("data")
            .and_then(|d| d.get("cursor"))
            .and_then(|c| c.as_str())
            .map(|s| s.to_string());
        let has_more = val
            .get("data")
            .and_then(|d| d.get("has_more"))
            .and_then(|h| h.as_bool())
            .unwrap_or(false);

        let posts: Vec<TikusPost> = serde_json::from_value(posts_raw)?;
        Ok(PaginatedPosts {
            posts,
            cursor: cursor_val,
            has_more,
        })
    }

    /// Fetch active live streaming channels.
    pub async fn get_live_streams(&self) -> Result<Vec<LiveStream>> {
        let val: Value = self
            .request(reqwest::Method::GET, "/streams/live", None, None, true)
            .await?;
        let streams_raw = val.get("data").cloned().unwrap_or(Value::Array(vec![]));
        let streams: Vec<LiveStream> = serde_json::from_value(streams_raw)?;
        Ok(streams)
    }

    /// Query search suggestions.
    pub async fn search_suggestions(&self, query: &str) -> Result<Value> {
        self.request(
            reqwest::Method::GET,
            "/search/suggest",
            Some(&[("q", query)]),
            None,
            true,
        )
        .await
    }

    /// Fetch current user coin balance and purchase packages.
    pub async fn get_coins(&self) -> Result<Value> {
        self.request(reqwest::Method::GET, "/coins", None, None, true).await
    }
}

/// Builder for TikusClient.
#[derive(Clone, Debug)]
pub struct TikusClientBuilder {
    base_url: String,
    user_agent: String,
    timeout: Duration,
    proxy: Option<String>,
    token: Option<String>,
}

impl Default for TikusClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TikusClientBuilder {
    pub fn new() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            user_agent: DEFAULT_USER_AGENT.to_string(),
            timeout: Duration::from_secs(15),
            proxy: None,
            token: None,
        }
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = ua.into();
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn proxy(mut self, proxy_url: impl Into<String>) -> Self {
        self.proxy = Some(proxy_url.into());
        self
    }

    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    pub fn build(self) -> Result<TikusClient> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&self.user_agent)
                .map_err(|e| TikusError::Api {
                    status: 400,
                    message: format!("Invalid User-Agent: {}", e),
                })?,
        );

        let mut builder = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(self.timeout);

        if let Some(proxy_url) = self.proxy {
            let proxy = reqwest::Proxy::all(&proxy_url)?;
            builder = builder.proxy(proxy);
        }

        let client = builder.build()?;

        Ok(TikusClient {
            client,
            base_url: self.base_url,
            token: self.token,
        })
    }
}
