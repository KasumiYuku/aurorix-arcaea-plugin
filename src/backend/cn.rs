use std::collections::HashMap;
use std::time::Duration;

use bytes::Bytes;
use reqwest::Method;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::time::{sleep, Instant};

use crate::model::Region;

use super::client::{ArcBackendNext, Req};
use super::dto::{CnLoginPoll, CnLoginStart, CnLoginWait, CnPlayer, CnRefresh, CnRenew};
use super::error::BackendError;

pub const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(5);

fn decode_cn<T: serde::de::DeserializeOwned>(body: &[u8], what: &str) -> Result<T, BackendError> {
    serde_json::from_slice(body)
        .map_err(|err| BackendError::decode(format!("{what}解析失败：{err}")))
}

impl ArcBackendNext {
    fn cn_base(&self) -> &str {
        self.base_for(Region::Cn)
    }

    pub async fn cn_login_start(
        &self,
        device_id: &str,
        arcaea_uid: Option<i64>,
    ) -> Result<CnLoginStart, BackendError> {
        let device = device_id.trim();
        if device.is_empty() {
            return Err(BackendError::invalid_input("设备码不能为空"));
        }
        let mut body = serde_json::Map::new();
        body.insert("device_id".to_string(), Value::String(device.to_string()));
        if let Some(uid) = arcaea_uid.filter(|uid| *uid > 0) {
            body.insert("arcaea_uid".to_string(), Value::from(uid));
        }
        let payload = Value::Object(body).to_string();
        let key = super::dedupe_key(Region::Cn, arcaea_uid.unwrap_or(0), "cn_login_start");
        let bytes = self
            .call(
                Req::new(Method::POST, "/v1/cn/login/start")
                    .base(self.cn_base())
                    .body(Bytes::from(payload))
                    .timeout(self.conn_timeout())
                    .retry(false)
                    .key(key.as_str()),
            )
            .await?;
        decode_cn(bytes.as_ref(), "国服授权响应")
    }

    pub async fn cn_login_poll(&self, ticket_id: &str) -> Result<CnLoginPoll, BackendError> {
        let ticket = ticket_id.trim();
        if ticket.is_empty() {
            return Err(BackendError::invalid_input("授权票据不能为空"));
        }
        let payload = json!({ "ticket_id": ticket }).to_string();
        let bytes = self
            .call(
                Req::new(Method::POST, "/v1/cn/login/poll")
                    .base(self.cn_base())
                    .body(Bytes::from(payload))
                    .timeout(self.cn_timeout())
                    .retry(false),
            )
            .await?;
        decode_cn(bytes.as_ref(), "国服授权状态")
    }

    pub async fn cn_login_wait(
        &self,
        ticket_id: &str,
        interval: Duration,
        timeout: Duration,
    ) -> Result<CnLoginWait, BackendError> {
        let interval = if interval.is_zero() {
            DEFAULT_POLL_INTERVAL
        } else {
            interval
        };
        if timeout.is_zero() {
            return Err(BackendError::invalid_input("授权轮询必须给出超时上限"));
        }
        let started = Instant::now();
        let mut polls = 0u32;
        loop {
            polls = polls.saturating_add(1);
            let poll = self.cn_login_poll(ticket_id).await?;
            let elapsed = started.elapsed();
            if poll.status.terminal() {
                return Ok(CnLoginWait {
                    polls,
                    elapsed,
                    timed_out: false,
                    poll,
                });
            }
            if elapsed >= timeout {
                return Ok(CnLoginWait {
                    polls,
                    elapsed,
                    timed_out: true,
                    poll,
                });
            }
            let remaining = timeout.saturating_sub(elapsed);
            sleep(interval.min(remaining)).await;
        }
    }

    pub async fn cn_login_resume(
        &self,
        device_ids: &[String],
    ) -> Result<HashMap<String, CnPlayer>, BackendError> {
        if device_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let devices: Vec<String> = device_ids
            .iter()
            .map(|device| device.trim())
            .filter(|device| !device.is_empty())
            .map(str::to_string)
            .collect();
        if devices.is_empty() {
            return Ok(HashMap::new());
        }
        let payload = json!({ "device_ids": devices }).to_string();
        let bytes = self
            .call(
                Req::new(Method::POST, "/v1/cn/login/resume")
                    .base(self.cn_base())
                    .body(Bytes::from(payload))
                    .timeout(self.conn_timeout())
                    .retry(false),
            )
            .await?;
        let payload: ResumePayload = decode_cn(bytes.as_ref(), "补绑响应")?;
        Ok(payload
            .tickets
            .into_iter()
            .map(|(device, ticket)| (device, ticket.player))
            .collect())
    }

    pub async fn cn_renew(&self, uid: i64) -> Result<CnRenew, BackendError> {
        let path = format!("/v1/cn/players/{uid}/renew");
        let bytes = self
            .call(
                Req::new(Method::POST, path.as_str())
                    .base(self.cn_base())
                    .timeout(self.conn_timeout())
                    .retry(false),
            )
            .await?;
        decode_cn(bytes.as_ref(), "国服续期响应")
    }

    pub async fn cn_refresh(&self, uid: i64) -> Result<CnRefresh, BackendError> {
        let path = format!("/v1/cn/players/{uid}/refresh");
        let bytes = self
            .call(
                Req::new(Method::POST, path.as_str())
                    .base(self.cn_base())
                    .timeout(self.conn_timeout())
                    .retry(false),
            )
            .await?;
        decode_cn(bytes.as_ref(), "国服云存档刷新响应")
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
struct ResumePayload {
    #[serde(default)]
    tickets: HashMap<String, ResumeTicket>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct ResumeTicket {
    #[serde(default)]
    player: CnPlayer,
    #[serde(default, rename = "settled_until")]
    _settled_until: String,
}
