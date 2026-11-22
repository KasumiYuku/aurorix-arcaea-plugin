use serde::Deserialize;

pub const MAX_ERROR_BODY_BYTES: usize = 512;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BackendError {
    #[error("后端配置有误：{message}")]
    Config { message: String },
    #[error("参数不合法：{message}")]
    InvalidInput { message: String },
    #[error("后端返回 HTTP {status}：{body}")]
    Http {
        status: u16,
        code: Option<String>,
        body: String,
        message: String,
        request_id: Option<String>,
        retry_after_seconds: Option<u64>,
    },
    #[error("未登录或凭证失效（HTTP {status}）：{message}")]
    Credential {
        status: u16,
        message: String,
        request_id: Option<String>,
    },
    #[error("请求后端超时：{message}")]
    Timeout { message: String },
    #[error("连接后端失败：{message}")]
    Transport { message: String },
    #[error("后端响应解析失败：{message}")]
    Decode { message: String },
    #[error("上游业务错误 {code}：{message}")]
    Upstream {
        code: String,
        message: String,
        request_id: Option<String>,
    },
    #[error("SSE 协议错误：{message}")]
    Sse { message: String },
}

impl BackendError {
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            message: message.into(),
        }
    }

    pub fn decode(message: impl Into<String>) -> Self {
        Self::Decode {
            message: message.into(),
        }
    }

    pub fn sse(message: impl Into<String>) -> Self {
        Self::Sse {
            message: message.into(),
        }
    }

    pub fn status(&self) -> Option<u16> {
        match self {
            Self::Http { status, .. } | Self::Credential { status, .. } => Some(*status),
            _ => None,
        }
    }

    pub fn code(&self) -> Option<&str> {
        match self {
            Self::Http { code, .. } => code.as_deref(),
            Self::Upstream { code, .. } => Some(code.as_str()),
            _ => None,
        }
    }

    pub fn is_not_found(&self) -> bool {
        self.status() == Some(404)
    }

    pub fn is_rate_limited(&self) -> bool {
        if self.status() == Some(429) {
            return true;
        }
        matches!(
            self.code(),
            Some("rate_limited") | Some("cooldown") | Some("too_many_requests")
        )
    }

    pub fn is_credential_invalid(&self) -> bool {
        match self {
            Self::Credential { .. } => true,
            _ => matches!(
                self.code(),
                Some("unauthenticated") | Some("unauthorized") | Some("token_invalid")
            ),
        }
    }

    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Timeout { .. } | Self::Transport { .. } => true,
            Self::Http { status, code, .. } => {
                matches!(status, 502..=504)
                    || matches!(code.as_deref(), Some("external") | Some("timeout"))
            }
            Self::Upstream { code, .. } => matches!(code.as_str(), "external" | "timeout"),
            _ => false,
        }
    }

    pub fn retry_after_seconds(&self) -> Option<u64> {
        match self {
            Self::Http {
                retry_after_seconds,
                ..
            } => *retry_after_seconds,
            _ => None,
        }
    }

    pub fn user_message(&self) -> String {
        let (text, request_id, traceable) = self.user_parts();
        match traceable && !request_id.trim().is_empty() {
            true => format!("{text}（请求编号：{request_id}）"),
            false => text,
        }
    }

    fn user_parts(&self) -> (String, String, bool) {
        match self {
            Self::Http {
                status,
                code,
                message,
                request_id,
                ..
            } => (
                match message.trim().is_empty() {
                    true => http_fallback(*status, code.as_deref()),
                    false => message.trim().to_string(),
                },
                request_id.clone().unwrap_or_default(),
                *status >= 500
                    || matches!(
                        code.as_deref(),
                        Some("internal") | Some("external") | Some("timeout") | Some("unavailable")
                    ),
            ),
            Self::Credential {
                status,
                message,
                request_id,
            } => (
                match message.trim().is_empty() {
                    true => String::from("登录凭证已失效，请重新授权"),
                    false => message.trim().to_string(),
                },
                request_id.clone().unwrap_or_default(),
                *status >= 500,
            ),
            Self::Upstream {
                message,
                request_id,
                ..
            } => (
                match message.trim().is_empty() {
                    true => String::from("上游这次没给结果，稍后再试"),
                    false => message.trim().to_string(),
                },
                request_id.clone().unwrap_or_default(),
                true,
            ),
            Self::InvalidInput { message } => (message.clone(), String::new(), false),
            Self::Config { .. } => (
                String::from("查询服务暂时不可用，请联系管理员"),
                String::new(),
                false,
            ),
            Self::Timeout { .. } => (
                String::from("后端这次没在预算内回应，稍后再试"),
                String::new(),
                false,
            ),
            Self::Transport { .. } => (
                String::from("连不上后端，稍后再试"),
                String::new(),
                false,
            ),
            Self::Decode { .. } | Self::Sse { .. } => (
                String::from("后端回了个看不懂的结果，稍后再试"),
                String::new(),
                false,
            ),
        }
    }

    pub fn from_reqwest(err: &reqwest::Error) -> Self {
        if err.is_timeout() {
            return Self::Timeout {
                message: "等待后端响应超时".to_string(),
            };
        }
        if err.is_connect() {
            return Self::Transport {
                message: format!("连接后端失败：{err}"),
            };
        }
        if err.is_decode() || err.is_body() {
            return Self::Decode {
                message: format!("读取后端响应失败：{err}"),
            };
        }
        Self::Transport {
            message: format!("请求后端失败：{err}{}", describe_chain(err)),
        }
    }

    pub fn from_status(status: u16, retry_after_seconds: Option<u64>, raw: &[u8]) -> Self {
        let text = String::from_utf8_lossy(raw);
        let problem = serde_json::from_slice::<ProblemBody>(raw).ok();
        let code = problem
            .as_ref()
            .and_then(|value| value.code.clone())
            .filter(|value| !value.is_empty());
        let request_id = problem
            .as_ref()
            .and_then(|value| value.request_id.clone())
            .filter(|value| !value.is_empty());
        let message = problem
            .as_ref()
            .map(ProblemBody::message)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| truncate_body(text.as_ref()));
        let failure = Self::Http {
            status,
            code: code.clone(),
            body: truncate_body(text.as_ref()),
            message: message.clone(),
            request_id: request_id.clone(),
            retry_after_seconds,
        };
        if failure.is_credential_invalid() || status == 401 {
            return Self::Credential {
                status,
                message,
                request_id,
            };
        }
        failure
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProblemBody {
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(default)]
    pub fields: Vec<ProblemField>,
    #[serde(default)]
    pub request_id: Option<String>,
}
impl ProblemBody {
    pub fn message(&self) -> String {
        let parts: Vec<&str> = self
            .fields
            .iter()
            .map(|field| field.message.trim())
            .filter(|message| !message.is_empty())
            .collect();
        if !parts.is_empty() {
            return parts.join("；");
        }
        self.detail.clone().unwrap_or_default().trim().to_string()
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProblemField {
    #[serde(default)]
    pub field: String,
    #[serde(default)]
    pub message: String,
}

fn describe_chain(err: &reqwest::Error) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(url) = err.url() {
        parts.push(format!("地址 {url}"));
    }
    let mut source = std::error::Error::source(err);
    while let Some(inner) = source {
        let text = inner.to_string();
        if !text.is_empty() && !parts.contains(&text) {
            parts.push(text);
        }
        source = inner.source();
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!("（{}）", parts.join("；"))
    }
}

pub fn truncate_body(body: &str) -> String {
    if body.len() <= MAX_ERROR_BODY_BYTES {
        return body.to_string();
    }
    let mut end = MAX_ERROR_BODY_BYTES;
    while end > 0 && !body.is_char_boundary(end) {
        end -= 1;
    }
    body[..end].to_string()
}

fn http_fallback(status: u16, code: Option<&str>) -> String {
    match code {
        Some("invalid_argument") | Some("validation_failed") => {
            String::from("请求参数不正确，请检查后重试")
        }
        Some("forbidden") => String::from("没有执行该操作的权限"),
        Some("rate_limited") => String::from("操作过于频繁，请稍后再试"),
        Some("unavailable") => String::from("查询服务暂时不可用，请稍后再试"),
        Some("not_found") => String::from("后端还没有这份数据"),
        _ => match status {
            400 => String::from("请求参数不正确，请检查后重试"),
            401 | 403 => String::from("没有执行该操作的权限"),
            404 => String::from("后端还没有这份数据"),
            429 => String::from("操作过于频繁，请稍后再试"),
            500..=599 => String::from("后端连接失败，请稍后重试"),
            _ => String::from("后端这次出错了，稍后再试"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_364() {
        let long = "错".repeat(400);
        let cut = truncate_body(&long);
        assert!(cut.len() <= MAX_ERROR_BODY_BYTES, "截断后不能超过 512 字节");
        assert!(cut.chars().all(|ch| ch == '错'), "不能切出半个字符");
        assert_eq!(truncate_body("短正文"), "短正文", "短正文要原样保留");
    }

    #[test]
    fn case_365() {
        let limited =
            BackendError::from_status(429, Some(12), r#"{"code":"rate_limited"}"#.as_bytes());
        assert!(limited.is_rate_limited(), "429 必须算限流");
        assert_eq!(
            limited.retry_after_seconds(),
            Some(12),
            "Retry-After 要带出来"
        );
        let expired = BackendError::from_status(401, None, r#"{"detail":"令牌失效"}"#.as_bytes());
        assert!(expired.is_credential_invalid(), "401 必须算凭证失效");
        let unavailable = BackendError::from_status(502, None, b"bad gateway");
        assert!(unavailable.is_retryable(), "502 可以重试");
        let internal = BackendError::from_status(500, None, b"boom");
        assert!(!internal.is_retryable(), "500 不是可重试的后端不可达");
    }
}

