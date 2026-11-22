use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use reqwest::header::{HeaderName, HeaderValue, ACCEPT, CONTENT_TYPE, RETRY_AFTER};
use reqwest::{Client, Method, Response};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use crate::model::{Difficulty, Region};

use super::dto::{
    AllScores, ArcDataSync, B50Payload, Binding, Play, PlayPayload, Player, PrefsPayload, RecentPlay,
    ScoreEnvelope, ScoreRecord, ScoreSource, Song, SongBoard, SongQuery, SongSearch, UpstreamState, B50,
};
use super::error::BackendError;
use super::sse::{self, B50EventStream};

pub const DEFAULT_SEARCH_LIMIT: u32 = 6;
pub const DEFAULT_USER_AGENT: &str = "aurorix-arcaea-plugin/0.1";

#[derive(Debug, Clone)]
pub struct BackendConfig {
    pub base_url: String,
    pub cn_base_url: Option<String>,
    pub token: Option<String>,
    pub admin_token: Option<String>,
    pub timeout: Duration,
    pub long_poll_timeout: Duration,
    pub cn_poll_timeout: Duration,
    pub stream_timeout: Duration,
    pub connect_timeout: Duration,
    pub user_agent: String,
    pub retries: u32,
    pub retry_backoff: Duration,
    pub max_retry_backoff: Duration,
    pub idempotency_header: Option<String>,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            cn_base_url: None,
            token: None,
            admin_token: None,
            timeout: Duration::from_secs(15),
            long_poll_timeout: Duration::from_secs(30),
            cn_poll_timeout: Duration::from_secs(40),
            stream_timeout: Duration::from_secs(300),
            connect_timeout: Duration::from_secs(5),
            user_agent: DEFAULT_USER_AGENT.to_string(),
            retries: 2,
            retry_backoff: Duration::from_millis(500),
            max_retry_backoff: Duration::from_secs(5),
            idempotency_header: None,
        }
    }
}

impl BackendConfig {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            ..Self::default()
        }
    }

    pub fn cn_base(mut self, base_url: impl Into<String>) -> Self {
        self.cn_base_url = Some(base_url.into());
        self
    }

    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    pub fn admin_token(mut self, token: impl Into<String>) -> Self {
        self.admin_token = Some(token.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn long_poll_timeout(mut self, timeout: Duration) -> Self {
        self.long_poll_timeout = timeout;
        self
    }

    pub fn cn_poll_timeout(mut self, timeout: Duration) -> Self {
        self.cn_poll_timeout = timeout;
        self
    }

    pub fn stream_timeout(mut self, timeout: Duration) -> Self {
        self.stream_timeout = timeout;
        self
    }

    pub fn retries(mut self, retries: u32, backoff: Duration) -> Self {
        self.retries = retries;
        self.retry_backoff = backoff;
        self
    }

    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    pub fn idempotency_header(mut self, header: impl Into<String>) -> Self {
        self.idempotency_header = Some(header.into());
        self
    }

    pub fn token_of(&self) -> Option<&str> {
        self.token.as_deref().filter(|token| !token.is_empty())
    }
}

pub fn normalize_base_url(raw: &str) -> Result<String, BackendError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(BackendError::config("后端地址不能为空"));
    }
    if trimmed.chars().any(char::is_whitespace) {
        return Err(BackendError::config("后端地址里不能有空白字符"));
    }
    if trimmed.contains('?') || trimmed.contains('#') {
        return Err(BackendError::config(
            "后端地址只能是站点根地址，不能带查询串或片段",
        ));
    }
    let rest = if let Some(rest) = trimmed.strip_prefix("https:") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("http:") {
        rest
    } else {
        return Err(BackendError::config("后端地址必须以 http 或 https 开头"));
    };
    let rest = rest
        .strip_prefix('/')
        .and_then(|rest| rest.strip_prefix('/'))
        .ok_or_else(|| BackendError::config("后端地址缺少协议分隔符"))?;
    if rest.trim_matches('/').is_empty() {
        return Err(BackendError::config("后端地址缺少主机名"));
    }
    let mut normalized = trimmed.trim_end_matches('/').to_string();
    normalized.push('/');
    Ok(normalized)
}

pub fn region_prefix(region: Region) -> &'static str {
    if region.is_cn() {
        "/cn"
    } else {
        ""
    }
}

pub fn encode_segment(raw: &str) -> Result<String, BackendError> {
    encode(raw, true)
}

pub fn encode_query_value(raw: &str) -> String {
    encode(raw, false).unwrap_or_default()
}

fn encode(raw: &str, trim: bool) -> Result<String, BackendError> {
    let text = if trim { raw.trim() } else { raw };
    if trim && text.is_empty() {
        return Err(BackendError::invalid_input("路径片段不能为空"));
    }
    let mut out = String::with_capacity(text.len());
    for byte in text.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    Ok(out)
}

fn validate_uid(uid: i64) -> Result<i64, BackendError> {
    if uid <= 0 {
        return Err(BackendError::invalid_input(format!(
            "Arcaea UID 必须是正整数，实际收到 {uid}"
        )));
    }
    Ok(uid)
}

pub(crate) fn with_query(path: String, params: &[(&str, String)]) -> String {
    let pairs: Vec<String> = params
        .iter()
        .filter(|(_, value)| !value.is_empty())
        .map(|(key, value)| format!("{key}={}", encode_query_value(value)))
        .collect();
    if pairs.is_empty() {
        path
    } else {
        format!("{path}?{}", pairs.join("&"))
    }
}

pub(crate) fn decode_json<T: DeserializeOwned>(body: &[u8], what: &str) -> Result<T, BackendError> {
    serde_json::from_slice(body)
        .map_err(|err| BackendError::decode(format!("{what}解析失败：{err}")))
}

pub(crate) struct Req<'a> {
    base: &'a str,
    method: Method,
    path: &'a str,
    body: Option<Bytes>,
    timeout: Duration,
    retryable: bool,
    auth: Option<String>,
    key: Option<&'a str>,
}

impl Default for Req<'_> {
    fn default() -> Self {
        Self {
            base: "",
            method: Method::GET,
            path: "",
            body: None,
            timeout: Duration::ZERO,
            retryable: false,
            auth: None,
            key: None,
        }
    }
}

impl<'a> Req<'a> {
    pub(crate) fn new(method: Method, path: &'a str) -> Self {
        Self {
            method,
            path,
            ..Self::default()
        }
    }

    pub(crate) fn base(mut self, base: &'a str) -> Self {
        self.base = base;
        self
    }

    pub(crate) fn body(mut self, body: Bytes) -> Self {
        self.body = Some(body);
        self
    }

    pub(crate) fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub(crate) fn retry(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }

    pub(crate) fn auth(mut self, auth: impl Into<String>) -> Self {
        self.auth = Some(auth.into());
        self
    }

    pub(crate) fn key(mut self, key: &'a str) -> Self {
        self.key = Some(key);
        self
    }
}

#[derive(Debug)]
struct Inner {
    config: BackendConfig,
    client: Client,
}

#[derive(Clone, Debug)]
pub struct ArcBackendNext {
    inner: Arc<Inner>,
}

impl ArcBackendNext {
    pub fn new(config: BackendConfig) -> Result<Self, BackendError> {
        let mut config = config;
        config.base_url = normalize_base_url(&config.base_url)?;
        if let Some(cn_base) = config.cn_base_url.clone() {
            config.cn_base_url = Some(normalize_base_url(&cn_base)?);
        }
        if config.user_agent.trim().is_empty() {
            return Err(BackendError::config("User-Agent 不能为空"));
        }
        if config.timeout.is_zero() {
            config.timeout = Duration::from_secs(15);
        }
        let client = Client::builder()
            .user_agent(config.user_agent.clone())
            .connect_timeout(config.connect_timeout)
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .map_err(|err| BackendError::config(format!("构建 HTTP 客户端失败：{err}")))?;
        Ok(Self {
            inner: Arc::new(Inner { config, client }),
        })
    }

    pub fn config(&self) -> &BackendConfig {
        &self.inner.config
    }

    pub(crate) fn base_for(&self, region: Region) -> &str {
        match (region.is_cn(), self.inner.config.cn_base_url.as_deref()) {
            (true, Some(cn_base)) => cn_base,
            _ => &self.inner.config.base_url,
        }
    }

    pub(crate) fn base_global(&self) -> &str {
        &self.inner.config.base_url
    }

    pub(crate) fn url_for(&self, base: &str, path: &str) -> String {
        let path = path.trim_start_matches('/');
        format!("{base}{path}")
    }

    pub(crate) fn conn_timeout(&self) -> Duration {
        self.inner.config.timeout
    }

    pub(crate) fn cn_timeout(&self) -> Duration {
        self.inner.config.cn_poll_timeout
    }

    fn backoff(&self, attempt: u32) -> Duration {
        let factor = 2u32.saturating_pow(attempt.min(6));
        self.inner
            .config
            .retry_backoff
            .saturating_mul(factor)
            .min(self.inner.config.max_retry_backoff)
    }

    async fn attempt(
        &self,
        url: &str,
        method: &Method,
        body: Option<&Bytes>,
        timeout: Duration,
        auth: Option<&str>,
        key: Option<&str>,
    ) -> Result<Response, BackendError> {
        let mut request = self
            .inner
            .client
            .request(method.clone(), url)
            .timeout(timeout)
            .header(ACCEPT, "application/json");
        let token = auth
            .map(str::to_string)
            .or_else(|| self.inner.config.token.clone());
        if let Some(token) = token.filter(|token| !token.is_empty()) {
            request = request.bearer_auth(token);
        }
        if let (Some(header), Some(key)) = (self.inner.config.idempotency_header.as_deref(), key) {
            if let (Ok(name), Ok(value)) = (
                HeaderName::from_bytes(header.as_bytes()),
                HeaderValue::from_str(key),
            ) {
                request = request.header(name, value);
            }
        }
        if let Some(payload) = body {
            request = request
                .header(CONTENT_TYPE, "application/json")
                .body(payload.clone());
        }
        let response = request
            .send()
            .await
            .map_err(|err| BackendError::from_reqwest(&err))?;
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }
        let retry_after = response
            .headers()
            .get(RETRY_AFTER)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.trim().parse::<u64>().ok());
        let body = response
            .bytes()
            .await
            .map(|bytes| bytes.to_vec())
            .unwrap_or_default();
        Err(BackendError::from_status(
            status.as_u16(),
            retry_after,
            body.as_slice(),
        ))
    }

    pub(crate) async fn send(&self, req: Req<'_>) -> Result<Response, BackendError> {
        let url = self.url_for(req.base, req.path);
        let attempts = if req.retryable {
            self.inner.config.retries.saturating_add(1)
        } else {
            1
        };
        let mut last_error: Option<BackendError> = None;
        for attempt in 0..attempts {
            match self
                .attempt(
                    url.as_str(),
                    &req.method,
                    req.body.as_ref(),
                    req.timeout,
                    req.auth.as_deref(),
                    req.key,
                )
                .await
            {
                Ok(response) => return Ok(response),
                Err(error) => {
                    let can_retry = req.retryable && error.is_retryable() && attempt + 1 < attempts;
                    last_error = Some(error);
                    if !can_retry {
                        break;
                    }
                    tokio::time::sleep(self.backoff(attempt)).await;
                }
            }
        }
        Err(last_error.unwrap_or_else(|| BackendError::Transport {
            message: "请求后端失败".to_string(),
        }))
    }

    pub(crate) async fn call(&self, req: Req<'_>) -> Result<Bytes, BackendError> {
        let response = self.send(req).await?;
        response
            .bytes()
            .await
            .map_err(|err| BackendError::from_reqwest(&err))
    }

    pub(crate) async fn get_bytes(
        &self,
        region: Region,
        path: &str,
        timeout: Duration,
    ) -> Result<Bytes, BackendError> {
        self.call(
            Req::new(Method::GET, path)
                .base(self.base_for(region))
                .timeout(timeout)
                .retry(true),
        )
        .await
    }

    async fn get_envelope(
        &self,
        region: Region,
        path: &str,
        timeout: Duration,
    ) -> Result<ScoreEnvelope, BackendError> {
        let body = self.get_bytes(region, path, timeout).await?;
        decode_json(body.as_ref(), "成绩响应")
    }

    pub(crate) fn player_path(
        region: Region,
        uid: i64,
        suffix: &str,
        params: &[(&str, String)],
    ) -> Result<String, BackendError> {
        let uid = validate_uid(uid)?;
        let path = format!("/v1{}/players/{uid}{suffix}", region_prefix(region));
        Ok(with_query(path, params))
    }

    pub async fn bind(&self, friend_code: &str) -> Result<Binding, BackendError> {
        let code = friend_code.trim();
        if code.is_empty() {
            return Err(BackendError::invalid_input("好友码不能为空"));
        }
        let body = json!({ "friend_code": code }).to_string();
        let bytes = self
            .call(
                Req::new(Method::POST, "/v1/players")
                    .base(self.base_global())
                    .body(Bytes::from(body))
                    .timeout(self.conn_timeout())
                    .retry(false),
            )
            .await?;
        decode_json(bytes.as_ref(), "绑定响应")
    }

    pub async fn player(&self, region: Region, uid: i64) -> Result<Option<Player>, BackendError> {
        let path = Self::player_path(region, uid, "", &[])?;
        let response = self
            .send(
                Req::new(Method::GET, path.as_str())
                    .base(self.base_for(region))
                    .timeout(self.conn_timeout())
                    .retry(true),
            )
            .await;
        let response = match response {
            Ok(response) => response,
            Err(error) if error.is_not_found() => return Ok(None),
            Err(error) => return Err(error),
        };
        let body = response
            .bytes()
            .await
            .map_err(|err| BackendError::from_reqwest(&err))?;
        let player: Player = decode_json(body.as_ref(), "玩家档案")?;
        if player.arcaea_uid == 0 && player.identity.name.is_empty() {
            return Ok(None);
        }
        Ok(Some(player))
    }

    pub async fn b50(&self, region: Region, uid: i64, refresh: bool) -> Result<B50, BackendError> {
        let params: Vec<(&str, String)> = if refresh {
            vec![("refresh", "true".to_string())]
        } else {
            Vec::new()
        };
        let path = Self::player_path(region, uid, "/b50", &params)?;
        let envelope = self
            .get_envelope(region, path.as_str(), self.conn_timeout())
            .await?;
        b50_from_envelope(envelope)
    }

    pub async fn b50_stream(
        &self,
        region: Region,
        uid: i64,
        refresh: bool,
    ) -> Result<B50EventStream, BackendError> {
        let params: Vec<(&str, String)> = if refresh {
            vec![("refresh", "true".to_string())]
        } else {
            Vec::new()
        };
        let path = Self::player_path(region, uid, "/b50", &params)?;
        let response = self
            .send(
                Req::new(Method::GET, path.as_str())
                    .base(self.base_for(region))
                    .timeout(self.inner.config.stream_timeout)
                    .retry(true),
            )
            .await?;
        let streaming = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_ascii_lowercase().contains("event-stream"))
            .unwrap_or(false);
        if streaming {
            return Ok(sse::b50_event_stream(response.bytes_stream()));
        }
        let body = response
            .bytes()
            .await
            .map_err(|err| BackendError::from_reqwest(&err))?;
        Ok(sse::b50_event_stream_from_body(body))
    }

    pub async fn b50_collected(
        &self,
        region: Region,
        uid: i64,
        refresh: bool,
    ) -> Result<B50, BackendError> {
        use futures_util::StreamExt;
        let mut stream = self.b50_stream(region, uid, refresh).await?;
        while let Some(item) = stream.next().await {
            if let sse::B50Event::Completed { b50, .. } = item? {
                return Ok(*b50);
            }
        }
        Err(BackendError::decode("流式响应结束了但没有给出成绩"))
    }

    pub async fn all_scores(
        &self,
        region: Region,
        uid: i64,
        source: ScoreSource,
    ) -> Result<AllScores, BackendError> {
        let params = vec![("source", source.as_str().to_string())];
        let path = Self::player_path(region, uid, "/scores", &params)?;
        let envelope = self
            .get_envelope(region, path.as_str(), self.conn_timeout())
            .await?;
        let meta = envelope.meta();
        if let Some(payload) = envelope.payload() {
            let records: Vec<ScoreRecord> = serde_json::from_value(payload.clone())
                .map_err(|err| BackendError::decode(format!("全量成绩解析失败：{err}")))?;
            return Ok(AllScores { records, meta });
        }
        if let Some(error) = envelope.terminal_error() {
            return Err(error);
        }
        if envelope.task().is_some() {
            return Ok(AllScores {
                records: Vec::new(),
                meta,
            });
        }
        Err(BackendError::decode("后端未返回成绩数据"))
    }

    pub async fn recent(&self, region: Region, uid: i64) -> Result<RecentPlay, BackendError> {
        let path = Self::player_path(region, uid, "/recent", &[])?;
        let body = self
            .get_bytes(region, path.as_str(), self.conn_timeout())
            .await?;
        let envelope: ScoreEnvelope = serde_json::from_slice(body.as_ref())
            .map_err(|err| BackendError::decode(format!("最近成绩元信息解析失败：{err}")))?;
        if let Some(error) = envelope.terminal_error() {
            return Err(error);
        }
        let meta = envelope.meta();
        let payload: PlayPayload = serde_json::from_slice(body.as_ref()).unwrap_or_default();
        Ok(RecentPlay {
            found: payload.found,
            record: payload.record,
            identity: payload.identity,
            meta,
        })
    }

    pub async fn set_prefs(
        &self,
        region: Region,
        uid: i64,
        char: Option<&str>,
        banner: Option<&str>,
    ) -> Result<PrefsPayload, BackendError> {
        if char.is_none() && banner.is_none() {
            return Err(BackendError::invalid_input(
                "至少要改一件自选素材，否则这次请求没有意义",
            ));
        }
        let path = Self::player_path(region, uid, "/prefs", &[])?;
        let mut body = serde_json::Map::new();
        if let Some(char) = char {
            body.insert("char".to_string(), Value::String(char.to_string()));
        }
        if let Some(banner) = banner {
            body.insert("banner".to_string(), Value::String(banner.to_string()));
        }
        let payload = Value::Object(body).to_string();
        let key = super::dedupe_key(region, uid, "prefs");
        let bytes = self
            .call(
                Req::new(Method::PUT, path.as_str())
                    .base(self.base_for(region))
                    .body(Bytes::from(payload))
                    .timeout(self.conn_timeout())
                    .retry(false)
                    .key(key.as_str()),
            )
            .await?;
        decode_json(bytes.as_ref(), "自选素材回执")
    }

    pub async fn songs(
        &self,
        region: Region,
        query: SongQuery,
    ) -> Result<SongSearch, BackendError> {
        let limit = query.limit.unwrap_or(DEFAULT_SEARCH_LIMIT);
        if limit == 0 {
            return Err(BackendError::invalid_input("曲库检索的 limit 必须大于 0"));
        }
        let mut params = vec![
            ("q", query.text.clone()),
            ("filter", query.filter.clone()),
            ("sort", query.sort.clone()),
            ("limit", limit.to_string()),
        ];
        if query.offset > 0 {
            params.push(("offset", query.offset.to_string()));
        }
        let path = with_query("/v1/songs".to_string(), params.as_slice());
        let body = self
            .get_bytes(region, path.as_str(), self.conn_timeout())
            .await?;
        decode_json(body.as_ref(), "曲库响应")
    }

    pub async fn song(&self, region: Region, song_id: &str) -> Result<Song, BackendError> {
        let song = encode_segment(song_id)?;
        let path = format!("/v1{}/songs/{song}", region_prefix(region));
        let body = self
            .get_bytes(region, path.as_str(), self.conn_timeout())
            .await?;
        decode_json(body.as_ref(), "曲目详情")
    }

    pub async fn song_boards(
        &self,
        region: Region,
        song_id: &str,
        difficulty: Difficulty,
        limit: u32,
    ) -> Result<SongBoard, BackendError> {
        let song = encode_segment(song_id)?;
        if limit == 0 {
            return Err(BackendError::invalid_input("单曲榜的 limit 必须大于 0"));
        }
        let rating_class = difficulty.as_upstream();
        let path = with_query(
            format!(
                "/v1{}/songs/{song}/boards/{rating_class}",
                region_prefix(region)
            ),
            &[("limit", limit.to_string())],
        );
        let body = self
            .get_bytes(region, path.as_str(), self.conn_timeout())
            .await?;
        decode_json(body.as_ref(), "单曲榜响应")
    }

    pub async fn task(
        &self,
        region: Region,
        task_id: &str,
        wait_seconds: Option<u32>,
    ) -> Result<ScoreEnvelope, BackendError> {
        let task = encode_segment(task_id)?;
        let params: Vec<(&str, String)> = wait_seconds
            .filter(|seconds| *seconds > 0)
            .map(|seconds| vec![("wait", seconds.to_string())])
            .unwrap_or_default();
        let path = with_query(
            format!("/v1{}/tasks/{task}", region_prefix(region)),
            params.as_slice(),
        );
        self.get_envelope(region, path.as_str(), self.inner.config.long_poll_timeout)
            .await
    }

    pub async fn b50_task(
        &self,
        region: Region,
        task_id: &str,
        wait_seconds: Option<u32>,
    ) -> Result<B50, BackendError> {
        let envelope = self.task(region, task_id, wait_seconds).await?;
        b50_from_envelope(envelope)
    }

    pub async fn recent_task(
        &self,
        region: Region,
        task_id: &str,
        wait_seconds: Option<u32>,
    ) -> Result<RecentPlay, BackendError> {
        let envelope = self.task(region, task_id, wait_seconds).await?;
        let meta = envelope.meta();
        let payload: PlayPayload = envelope
            .payload()
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default();
        Ok(RecentPlay {
            found: payload.found,
            record: payload.record,
            identity: payload.identity,
            meta,
        })
    }

    pub async fn play_task(
        &self,
        region: Region,
        task_id: &str,
        wait_seconds: Option<u32>,
    ) -> Result<Play, BackendError> {
        let envelope = self.task(region, task_id, wait_seconds).await?;
        Ok(play_from_envelope(envelope))
    }

    pub async fn sync_catalog(&self, dry_run: bool) -> Result<ArcDataSync, BackendError> {
        let admin = self
            .inner
            .config
            .admin_token
            .clone()
            .filter(|token| !token.is_empty())
            .ok_or_else(|| BackendError::config("未配置管理令牌，无法同步曲库"))?;
        let mut path = String::from("/v1/admin/arcdata/sync");
        if dry_run {
            path.push_str("?dry=1");
        }
        let response = self
            .call(
                Req::new(Method::POST, path.as_str())
                    .base(self.base_global())
                    .timeout(self.inner.config.stream_timeout)
                    .retry(false)
                    .auth(admin),
            )
            .await;
        let bytes = match response {
            Ok(bytes) => bytes,
            Err(error) => {
                let message = error.to_string();
                return match error.status() {
                    Some(412) => Err(BackendError::Upstream {
                        code: "arcdata_source_missing".to_string(),
                        message: "未配置曲库同步源，请让运维先填好后端 data.sync".to_string(),
                        request_id: None,
                    }),
                    Some(409) => Err(BackendError::Upstream {
                        code: "sync_running".to_string(),
                        message: "正在同步中，请稍后再试".to_string(),
                        request_id: None,
                    }),
                    Some(422) => Err(BackendError::Upstream {
                        code: "arcdata_invalid".to_string(),
                        message: format!("工件校验失败：{message}"),
                        request_id: None,
                    }),
                    _ => Err(error),
                };
            }
        };
        decode_json(bytes.as_ref(), "曲库同步响应")
    }

    pub async fn upstream_state(&self) -> Result<UpstreamState, BackendError> {
        let bytes = self
            .call(
                Req::new(Method::GET, "/v1/admin/upstream")
                    .base(self.base_global())
                    .timeout(self.conn_timeout())
                    .retry(true),
            )
            .await?;
        decode_json(bytes.as_ref(), "上游闸态响应")
    }

    pub async fn set_upstream(
        &self,
        closed: bool,
        reason: &str,
    ) -> Result<UpstreamState, BackendError> {
        let body = json!({ "closed": closed, "reason": reason.trim() }).to_string();
        let bytes = self
            .call(
                Req::new(Method::POST, "/v1/admin/upstream")
                    .base(self.base_global())
                    .body(Bytes::from(body))
                    .timeout(self.conn_timeout())
                    .retry(false),
            )
            .await?;
        decode_json(bytes.as_ref(), "上游闸态响应")
    }
}

fn play_from_envelope(envelope: ScoreEnvelope) -> Play {
    let meta = envelope.meta();
    let payload: PlayPayload = envelope
        .payload()
        .and_then(|value| serde_json::from_value(value.clone()).ok())
        .unwrap_or_default();
    Play {
        found: payload.found,
        record: payload.record,
        identity: payload.identity,
        meta,
    }
}

fn b50_from_envelope(envelope: ScoreEnvelope) -> Result<B50, BackendError> {
    let meta = envelope.meta();
    if let Some(payload) = envelope.payload() {
        let parsed: B50Payload = serde_json::from_value(payload.clone())
            .map_err(|err| BackendError::decode(format!("B50 载荷解析失败：{err}")))?;
        return Ok(B50::from_payload(parsed, meta));
    }
    if let Some(error) = envelope.terminal_error() {
        return Err(error);
    }
    if envelope.task().is_some() {
        return Ok(B50::empty(meta));
    }
    Err(BackendError::decode("后端未返回成绩数据"))
}
