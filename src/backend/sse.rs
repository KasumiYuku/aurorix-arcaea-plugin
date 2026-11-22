use std::collections::VecDeque;
use std::pin::Pin;
use std::time::Duration;

use bytes::Bytes;
use chrono::{DateTime, Utc};
use futures_util::{stream, Stream, StreamExt};
use serde::Deserialize;
use serde_json::Value;

use super::dto::{
    parse_time, B50Payload, EtaEstimate, QueryMeta, ScoreEnvelope, TaskProgress, B50,
};
use super::error::BackendError;

pub type B50EventStream = Pin<Box<dyn Stream<Item = Result<B50Event, BackendError>> + Send>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProgressSource {
    #[default]
    Unknown,
    Cache,
    Live,
    RetryWait,
}

impl ProgressSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Cache => "cache",
            Self::Live => "live",
            Self::RetryWait => "retry_wait",
        }
    }

    pub fn from_envelope(envelope: &ScoreEnvelope) -> Self {
        if envelope.cached {
            return Self::Cache;
        }
        match envelope.source.to_ascii_lowercase().as_str() {
            "cache" | "cached" | "local" => Self::Cache,
            "arcaea" | "live" | "upstream" | "probe" => Self::Live,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum B50Event {
    Started {
        source: ProgressSource,
        cached: bool,
        cached_at: Option<DateTime<Utc>>,
        total: Option<u32>,
    },
    Progress {
        source: ProgressSource,
        done: u32,
        total: u32,
        percent: Option<f32>,
        eta: EtaEstimate,
        status: String,
    },
    Pending {
        task: TaskProgress,
        eta: EtaEstimate,
    },
    Retry {
        source: ProgressSource,
        delay: Duration,
        remaining: Option<Duration>,
    },
    Degraded {
        reason: String,
        retry_after_seconds: Option<u64>,
    },
    Heartbeat {
        at: Option<String>,
    },
    Completed {
        b50: Box<B50>,
        meta: Box<QueryMeta>,
        source: ProgressSource,
    },
}

impl B50Event {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed { .. })
    }

    pub fn source(&self) -> ProgressSource {
        match self {
            Self::Started { source, .. }
            | Self::Progress { source, .. }
            | Self::Retry { source, .. }
            | Self::Completed { source, .. } => *source,
            _ => ProgressSource::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SseEvent {
    pub name: String,
    pub data: String,
    pub id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SseItem {
    Event(SseEvent),
    Comment(String),
    Retry(u64),
}

#[derive(Debug, Default)]
pub struct SseParser {
    buffer: Vec<u8>,
    pending: SseEvent,
    retry_ms: Option<u64>,
    finished: bool,
}

impl SseParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn feed(&mut self, chunk: &[u8]) -> Vec<SseItem> {
        let mut items = Vec::new();
        if chunk.is_empty() {
            return items;
        }
        self.buffer.extend_from_slice(chunk);
        while let Some(newline) = self.buffer.iter().position(|byte| *byte == b'\n') {
            let mut line: Vec<u8> = self.buffer.drain(..=newline).collect();
            line.pop();
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            self.absorb(&line, &mut items);
        }
        items
    }

    pub fn finish(&mut self) -> Vec<SseItem> {
        let mut items = Vec::new();
        if self.finished {
            return items;
        }
        self.finished = true;
        if !self.buffer.is_empty() {
            let mut leftover = std::mem::take(&mut self.buffer);
            if leftover.last() == Some(&b'\r') {
                leftover.pop();
            }
            self.absorb(&leftover, &mut items);
        }
        self.dispatch(&mut items);
        items
    }

    fn absorb(&mut self, raw: &[u8], items: &mut Vec<SseItem>) {
        let text = String::from_utf8_lossy(raw).to_string();
        if text.is_empty() {
            self.dispatch(items);
            return;
        }
        if let Some(comment) = text.strip_prefix(':') {
            items.push(SseItem::Comment(comment.trim_start().to_string()));
            return;
        }
        let (field, value) = match text.split_once(':') {
            Some((field, value)) => (field, value.strip_prefix(' ').unwrap_or(value)),
            None => (text.as_str(), ""),
        };
        match field {
            "event" => self.pending.name = value.to_string(),
            "data" => {
                if !self.pending.data.is_empty() {
                    self.pending.data.push('\n');
                }
                self.pending.data.push_str(value);
            }
            "id" => {
                if !value.is_empty() {
                    self.pending.id = Some(value.to_string());
                }
            }
            "retry" => {
                if let Ok(milliseconds) = value.trim().parse::<u64>() {
                    self.retry_ms = Some(milliseconds);
                }
            }
            _ => {}
        }
    }

    fn dispatch(&mut self, items: &mut Vec<SseItem>) {
        if let Some(milliseconds) = self.retry_ms.take() {
            items.push(SseItem::Retry(milliseconds));
        }
        if self.pending.data.is_empty() {
            self.pending = SseEvent::default();
            return;
        }
        let event = std::mem::take(&mut self.pending);
        items.push(SseItem::Event(event));
    }
}

pub fn parse_sse(input: &[u8]) -> Vec<SseItem> {
    let mut parser = SseParser::new();
    let mut items = parser.feed(input);
    items.extend(parser.finish());
    items
}

#[derive(Debug, Clone, Default, Deserialize)]
struct ProgressData {
    #[serde(default)]
    done: u32,
    #[serde(default)]
    total: u32,
    #[serde(default)]
    status: String,
    #[serde(default)]
    cached: bool,
    #[serde(default)]
    source: String,
    #[serde(default)]
    eta_seconds: Option<u32>,
    #[serde(default)]
    eta_low_seconds: Option<u32>,
    #[serde(default)]
    eta_high_seconds: Option<u32>,
    #[serde(default)]
    eta_remaining_seconds: Option<u32>,
    #[serde(default)]
    retry_after_seconds: Option<u64>,
    #[serde(default)]
    degraded_reason: Option<String>,
}

impl ProgressData {
    fn eta(&self) -> EtaEstimate {
        EtaEstimate::new(
            self.eta_seconds,
            self.eta_low_seconds,
            self.eta_high_seconds,
            self.eta_remaining_seconds,
        )
    }

    fn source(&self) -> Option<ProgressSource> {
        if self.cached {
            return Some(ProgressSource::Cache);
        }
        match self.source.to_ascii_lowercase().as_str() {
            "cache" | "cached" | "local" => Some(ProgressSource::Cache),
            "arcaea" | "live" | "upstream" | "probe" => Some(ProgressSource::Live),
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
pub struct B50Mapper {
    source: ProgressSource,
    total: u32,
    finished: bool,
}

impl B50Mapper {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn source(&self) -> ProgressSource {
        self.source
    }

    pub fn total(&self) -> u32 {
        self.total
    }

    pub fn map(&mut self, item: SseItem) -> Option<Result<B50Event, BackendError>> {
        if self.finished {
            return None;
        }
        match item {
            SseItem::Comment(_) => None,
            SseItem::Retry(milliseconds) => {
                self.source = ProgressSource::RetryWait;
                Some(Ok(B50Event::Retry {
                    source: ProgressSource::RetryWait,
                    delay: Duration::from_millis(milliseconds),
                    remaining: None,
                }))
            }
            SseItem::Event(event) => self.map_event(event),
        }
    }

    fn map_event(&mut self, event: SseEvent) -> Option<Result<B50Event, BackendError>> {
        let name = event.name.to_ascii_lowercase();
        let value = if event.data.trim().is_empty() {
            None
        } else {
            serde_json::from_str::<Value>(&event.data).ok()
        };
        match name.as_str() {
            "progress" | "b50.progress" | "scan" | "scan.progress" => {
                let data = self.progress_data(value.as_ref());
                self.observe(&data);
                let total = data.total.max(self.total);
                Some(Ok(B50Event::Progress {
                    source: self.source,
                    done: data.done,
                    total,
                    percent: percent(data.done, total),
                    eta: data.eta(),
                    status: data.status.clone(),
                }))
            }
            "cached" | "cache" | "hit" => {
                self.source = ProgressSource::Cache;
                let total = self.total_from(value.as_ref());
                if let Some(total) = total {
                    self.total = total;
                }
                Some(Ok(B50Event::Started {
                    source: ProgressSource::Cache,
                    cached: true,
                    cached_at: self.cached_at(value.as_ref()),
                    total,
                }))
            }
            "live" | "upstream" | "scanning" | "miss" => {
                self.source = ProgressSource::Live;
                let total = self.total_from(value.as_ref());
                if let Some(total) = total {
                    self.total = total;
                }
                Some(Ok(B50Event::Started {
                    source: ProgressSource::Live,
                    cached: false,
                    cached_at: self.cached_at(value.as_ref()),
                    total,
                }))
            }
            "started" | "begin" | "accepted" => {
                let total = self.total_from(value.as_ref());
                if let Some(total) = total {
                    self.total = total;
                }
                Some(Ok(B50Event::Started {
                    source: self.source,
                    cached: self.source == ProgressSource::Cache,
                    cached_at: self.cached_at(value.as_ref()),
                    total,
                }))
            }
            "retry" | "wait" | "cooldown" | "backoff" => {
                self.source = ProgressSource::RetryWait;
                let data = self.progress_data(value.as_ref());
                let delay = data
                    .retry_after_seconds
                    .or_else(|| data.eta_remaining_seconds.map(u64::from))
                    .map(Duration::from_secs)
                    .unwrap_or_else(|| Duration::from_secs(2));
                Some(Ok(B50Event::Retry {
                    source: ProgressSource::RetryWait,
                    delay,
                    remaining: Some(delay),
                }))
            }
            "degraded" | "partial" => {
                let data = self.progress_data(value.as_ref());
                let reason = data
                    .degraded_reason
                    .clone()
                    .or_else(|| {
                        value
                            .as_ref()
                            .and_then(|value| value.get("reason"))
                            .and_then(Value::as_str)
                            .map(str::to_string)
                    })
                    .unwrap_or_else(|| "partial".to_string());
                Some(Ok(B50Event::Degraded {
                    reason,
                    retry_after_seconds: data.retry_after_seconds,
                }))
            }
            "pending" | "queued" | "running" => self.pending_event(value),
            "done" | "completed" | "complete" | "result" | "b50" => Some(self.completed(value)),
            "error" | "failed" => Some(Err(self.failure(value.as_ref(), &event.data))),
            "ping" | "heartbeat" => Some(Ok(B50Event::Heartbeat {
                at: (!event.data.trim().is_empty()).then(|| event.data.clone()),
            })),
            _ => {
                let value = value?;
                if value.get("records").is_some() || value.get("data").is_some() {
                    return Some(self.completed(Some(value)));
                }
                if value.get("status").and_then(Value::as_str).is_some() {
                    return self.pending_event(Some(value));
                }
                None
            }
        }
    }

    fn progress_data(&self, value: Option<&Value>) -> ProgressData {
        value
            .and_then(|value| serde_json::from_value::<ProgressData>(value.clone()).ok())
            .unwrap_or_default()
    }

    fn total_from(&self, value: Option<&Value>) -> Option<u32> {
        let value = value?;
        let total = value.get("total").and_then(Value::as_u64)?;
        u32::try_from(total).ok()
    }

    fn cached_at(&self, value: Option<&Value>) -> Option<DateTime<Utc>> {
        let value = value?;
        value
            .get("cached_at")
            .and_then(Value::as_str)
            .and_then(parse_time)
    }

    fn observe(&mut self, data: &ProgressData) {
        if let Some(source) = data.source() {
            if self.source != ProgressSource::RetryWait {
                self.source = source;
            }
        }
        if data.total > self.total {
            self.total = data.total;
        }
    }

    fn pending_event(&mut self, value: Option<Value>) -> Option<Result<B50Event, BackendError>> {
        let envelope = value
            .and_then(|value| serde_json::from_value::<ScoreEnvelope>(value).ok())
            .unwrap_or_default();
        let task = envelope.task()?;
        let eta = envelope.eta();
        if envelope.status.eq_ignore_ascii_case("failed") {
            return Some(Err(envelope.terminal_error().unwrap_or_else(|| {
                BackendError::sse("后端报告扫描失败却没有给出原因")
            })));
        }
        Some(Ok(B50Event::Pending { task, eta }))
    }

    fn completed(&mut self, value: Option<Value>) -> Result<B50Event, BackendError> {
        let value = value.ok_or_else(|| BackendError::sse("完成事件没有携带数据"))?;
        let envelope = serde_json::from_value::<ScoreEnvelope>(value.clone()).unwrap_or_default();
        let meta = envelope.meta();
        let source = if self.source == ProgressSource::Unknown {
            ProgressSource::from_envelope(&envelope)
        } else {
            self.source
        };
        let payload = match envelope.payload() {
            Some(payload) => serde_json::from_value::<B50Payload>(payload.clone())
                .map_err(|err| BackendError::decode(format!("B50 载荷解析失败：{err}")))?,
            None => {
                let payload = serde_json::from_value::<B50Payload>(value)
                    .map_err(|err| BackendError::decode(format!("B50 载荷解析失败：{err}")))?;
                payload
            }
        };
        self.finished = true;
        Ok(B50Event::Completed {
            b50: Box::new(B50::from_payload(payload, meta.clone())),
            meta: Box::new(meta),
            source,
        })
    }

    fn failure(&self, value: Option<&Value>, raw: &str) -> BackendError {
        let message = value
            .and_then(|value| {
                value
                    .get("error")
                    .and_then(Value::as_str)
                    .or_else(|| value.get("detail").and_then(Value::as_str))
                    .or_else(|| value.get("message").and_then(Value::as_str))
            })
            .map(str::to_string)
            .filter(|message| !message.trim().is_empty())
            .unwrap_or_else(|| {
                if raw.trim().is_empty() {
                    "扫描失败，请稍后重试".to_string()
                } else {
                    raw.to_string()
                }
            });
        BackendError::Upstream {
            code: "scan_failed".to_string(),
            message,
            request_id: None,
        }
    }
}

fn percent(done: u32, total: u32) -> Option<f32> {
    if total == 0 {
        return None;
    }
    Some((done as f32) * 100.0 / (total as f32))
}

pub fn events_from_envelope_body(body: &[u8]) -> Vec<Result<B50Event, BackendError>> {
    let envelope = match serde_json::from_slice::<ScoreEnvelope>(body) {
        Ok(envelope) => envelope,
        Err(err) => {
            return vec![Err(BackendError::decode(format!(
                "B50 响应解析失败：{err}"
            )))]
        }
    };
    let mut events = Vec::new();
    let source = ProgressSource::from_envelope(&envelope);
    let total = envelope.progress.as_ref().map(|progress| progress.total);
    events.push(Ok(B50Event::Started {
        source,
        cached: envelope.cached,
        cached_at: envelope.cached_at,
        total,
    }));
    if let Some(progress) = &envelope.progress {
        events.push(Ok(B50Event::Progress {
            source,
            done: progress.done,
            total: progress.total,
            percent: percent(progress.done, progress.total),
            eta: envelope.eta(),
            status: envelope.status.clone(),
        }));
    }
    if envelope.degraded || envelope.partial {
        let reason = if envelope.degraded_reason.is_empty() {
            "partial".to_string()
        } else {
            envelope.degraded_reason.clone()
        };
        events.push(Ok(B50Event::Degraded {
            reason,
            retry_after_seconds: (envelope.retry_after_seconds > 0)
                .then_some(envelope.retry_after_seconds as u64),
        }));
    }
    if let Some(error) = envelope.terminal_error() {
        events.push(Err(error));
        return events;
    }
    let meta = envelope.meta();
    if let Some(payload) = envelope.payload() {
        match serde_json::from_value::<B50Payload>(payload.clone()) {
            Ok(payload) => {
                events.push(Ok(B50Event::Completed {
                    b50: Box::new(B50::from_payload(payload, meta.clone())),
                    meta: Box::new(meta),
                    source,
                }));
            }
            Err(err) => events.push(Err(BackendError::decode(format!(
                "B50 载荷解析失败：{err}"
            )))),
        }
        return events;
    }
    if let Some(task) = envelope.task() {
        let eta = envelope.eta();
        events.push(Ok(B50Event::Pending { task, eta }));
        return events;
    }
    events.push(Err(BackendError::decode("后端未返回成绩数据")));
    events
}

pub fn b50_event_stream_from_body(body: Bytes) -> B50EventStream {
    Box::pin(stream::iter(events_from_envelope_body(body.as_ref())))
}

struct StreamState<S> {
    chunks: Pin<Box<S>>,
    parser: SseParser,
    mapper: B50Mapper,
    queue: VecDeque<Result<B50Event, BackendError>>,
    done: bool,
}

pub fn b50_event_stream<S>(chunks: S) -> B50EventStream
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
{
    let state = StreamState {
        chunks: Box::pin(chunks),
        parser: SseParser::new(),
        mapper: B50Mapper::new(),
        queue: VecDeque::new(),
        done: false,
    };
    Box::pin(stream::unfold(state, |mut state| async move {
        loop {
            if let Some(item) = state.queue.pop_front() {
                return Some((item, state));
            }
            if state.done {
                return None;
            }
            match state.chunks.next().await {
                Some(Ok(chunk)) => {
                    for item in state.parser.feed(chunk.as_ref()) {
                        if let Some(mapped) = state.mapper.map(item) {
                            state.queue.push_back(mapped);
                        }
                    }
                }
                Some(Err(err)) => {
                    state.done = true;
                    return Some((Err(BackendError::from_reqwest(&err)), state));
                }
                None => {
                    state.done = true;
                    for item in state.parser.finish() {
                        if let Some(mapped) = state.mapper.map(item) {
                            state.queue.push_back(mapped);
                        }
                    }
                }
            }
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn case_366(items: &[SseItem]) -> Vec<&SseEvent> {
        items
            .iter()
            .filter_map(|item| match item {
                SseItem::Event(event) => Some(event),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn case_367() {
        let mut parser = SseParser::new();
        let mut items = parser.feed(b"event: prog");
        items.extend(parser.feed(b"ress\ndata: {\"done\":1"));
        items.extend(parser.feed(b",\"total\":10}\n\n"));
        items.extend(parser.finish());
        let events = case_366(&items);
        assert_eq!(events.len(), 1, "跨包断开也只能派发一个事件");
        assert_eq!(events[0].name, "progress", "事件名要拼回完整");
        assert_eq!(
            events[0].data, "{\"done\":1,\"total\":10}",
            "数据要拼回完整"
        );
    }

    #[test]
    fn case_368() {
        let mut parser = SseParser::new();
        let text = "data: 测试\n\n";
        let bytes = text.as_bytes();
        let mut cut = 0;
        for (index, byte) in bytes.iter().enumerate() {
            if *byte == b'\xe6' {
                cut = index + 1;
                break;
            }
        }
        let mut items = parser.feed(&bytes[..cut]);
        items.extend(parser.feed(&bytes[cut..]));
        let events = case_366(&items);
        assert_eq!(events.len(), 1, "要派发一个事件");
        assert_eq!(events[0].data, "测试", "切断的 UTF-8 要能拼回中文");
    }

    #[test]
    fn case_369() {
        let items = parse_sse(b"event: progress\ndata: first\ndata: second\n\n");
        let events = case_366(&items);
        assert_eq!(events.len(), 1, "多行 data 属于同一个事件");
        assert_eq!(events[0].data, "first\nsecond", "多行 data 用换行拼接");
    }

    #[test]
    fn case_370() {
        let items = parse_sse(b": keepalive\n\n: another\n\n");
        assert!(case_366(&items).is_empty(), "注释行不能派发成事件");
        assert_eq!(items.len(), 2, "两行注释各记一条");
    }

    #[test]
    fn case_371() {
        let items = parse_sse(b"event: progress\r\ndata: 1\r\n\r\nevent: done\ndata: 2\n\n");
        let events = case_366(&items);
        assert_eq!(events.len(), 2, "两种行尾都要认");
        assert_eq!(events[0].name, "progress", "第一个事件名不能带回车");
        assert_eq!(events[0].data, "1", "第一个事件数据不能带回车");
        assert_eq!(events[1].data, "2", "第二个事件也要解析");
    }

    #[test]
    fn case_372() {
        let mut parser = SseParser::new();
        let mut items = parser.feed(b"event: done\ndata: {\"name\":\"a\"}");
        assert!(case_366(&items).is_empty(), "没有空行时先不派发");
        items.extend(parser.finish());
        let events = case_366(&items);
        assert_eq!(events.len(), 1, "收尾时要补派发");
        assert_eq!(events[0].name, "done", "事件名要保留");
        assert!(parser.finish().is_empty(), "重复收尾不能再吐事件");
    }

    #[test]
    fn case_373() {
        let items = parse_sse(b"retry: 3000\n\n");
        assert_eq!(items, vec![SseItem::Retry(3000)], "retry 要换算成毫秒");
        let mut mapper = B50Mapper::new();
        let mapped = mapper.map(SseItem::Retry(3000)).expect("要映射成重试事件");
        let event = mapped.expect("重试事件不该是错误");
        match event {
            B50Event::Retry { delay, source, .. } => {
                assert_eq!(delay, Duration::from_millis(3000), "间隔要按毫秒还原");
                assert_eq!(source.as_str(), "retry_wait", "重试等待要单独区分");
            }
            other => panic!("应当是重试事件, 实际: {other:?}"),
        }
    }

    #[test]
    fn case_374() {
        let items = parse_sse(b"\n\nevent: progress\n\n");
        assert!(case_366(&items).is_empty(), "没有 data 的事件不能派发");
    }

    #[test]
    fn case_375() {
        let items = parse_sse(b"id: 7\nevent: progress\ndata: 1\n\n");
        let events = case_366(&items);
        assert_eq!(events[0].id.as_deref(), Some("7"), "id 要跟随事件带出");
    }

    #[test]
    fn case_376() {
        let mut parser = SseParser::new();
        let mut mapper = B50Mapper::new();
        let mut items = parser.feed(
            b"event: progress\ndata: {\"done\":5,\"total\":10,\"eta_remaining_seconds\":30}\n\n",
        );
        items.extend(parser.finish());
        let mut seen = Vec::new();
        for item in items {
            if let Some(mapped) = mapper.map(item) {
                seen.push(mapped.expect("进度事件不该是错误"));
            }
        }
        assert_eq!(seen.len(), 1, "只应产生一个进度事件");
        match &seen[0] {
            B50Event::Progress {
                done,
                total,
                percent,
                eta,
                ..
            } => {
                assert_eq!(*done, 5, "已完成数要对上");
                assert_eq!(*total, 10, "总量要对上");
                assert_eq!(*percent, Some(50.0), "百分比要算出来");
                assert_eq!(eta.remaining, Some(30), "剩余 ETA 要对上");
            }
            other => panic!("应当是进度事件, 实际: {other:?}"),
        }
        assert_eq!(mapper.total(), 10, "映射器要记住总量");
    }

    #[test]
    fn case_377() {
        let mut mapper = B50Mapper::new();
        let cached = mapper
            .map(SseItem::Event(SseEvent {
                name: "cached".to_string(),
                data: "{\"cached_at\":\"2026-01-02T03:04:05Z\",\"total\":50}".to_string(),
                id: None,
            }))
            .expect("缓存事件要映射")
            .expect("缓存事件不该是错误");
        match cached {
            B50Event::Started {
                source,
                cached,
                cached_at,
                total,
            } => {
                assert_eq!(source.as_str(), "cache", "缓存命中要标成 cache");
                assert!(cached, "缓存命中要带 cached 标记");
                assert!(cached_at.is_some(), "缓存时间要解析出来");
                assert_eq!(total, Some(50), "总量要带出来");
            }
            other => panic!("应当是开始事件, 实际: {other:?}"),
        }
        let live = mapper
            .map(SseItem::Event(SseEvent {
                name: "live".to_string(),
                data: String::new(),
                id: None,
            }))
            .expect("实时事件要映射")
            .expect("实时事件不该是错误");
        assert_eq!(live.source().as_str(), "live", "上游实时要标成 live");
        let wait = mapper
            .map(SseItem::Event(SseEvent {
                name: "retry".to_string(),
                data: "{\"retry_after_seconds\":8}".to_string(),
                id: None,
            }))
            .expect("重试事件要映射")
            .expect("重试事件不该是错误");
        match wait {
            B50Event::Retry { delay, source, .. } => {
                assert_eq!(delay, Duration::from_secs(8), "重试间隔要按秒还原");
                assert_eq!(source.as_str(), "retry_wait", "重试等待要单独区分");
            }
            other => panic!("应当是重试事件, 实际: {other:?}"),
        }
    }

    #[test]
    fn case_378() {
        let mut mapper = B50Mapper::new();
        assert!(
            mapper
                .map(SseItem::Event(SseEvent {
                    name: "whatever".to_string(),
                    data: "not json".to_string(),
                    id: None,
                }))
                .is_none(),
            "看不懂的事件要安静丢弃"
        );
    }

    #[test]
    fn case_379() {
        let body = r#"{"data":{"name":"玩家","current_ptt":12.5,"records":[{"song_id":"x","difficulty":2,"score":9900000,"rank":"EX+"}]},"source":"arcaea","cached":true,"cached_at":"2026-01-02T03:04:05Z","progress":{"done":50,"total":50},"eta_seconds":12}"#;
        let events: Vec<B50Event> = events_from_envelope_body(body.as_bytes())
            .into_iter()
            .map(|item| item.expect("回退解析不该报错"))
            .collect();
        assert!(
            matches!(events[0], B50Event::Started { cached: true, .. }),
            "首个事件要说清来源"
        );
        assert!(
            events
                .iter()
                .any(|event| matches!(event, B50Event::Progress { total: 50, .. })),
            "要给出进度"
        );
        let completed = events
            .iter()
            .find_map(|event| match event {
                B50Event::Completed { b50, .. } => Some(b50),
                _ => None,
            })
            .expect("必须收到完成事件");
        assert_eq!(completed.records.len(), 1, "成绩条数要对上");
        assert!(completed.meta.cached, "元信息要保留缓存标记");
    }

    #[test]
    fn case_380() {
        let events = events_from_envelope_body(
            r#"{"status":"failed","error":"上游没连上","task_id":"t1"}"#.as_bytes(),
        );
        let failure = events
            .into_iter()
            .find_map(Result::err)
            .expect("失败信封必须变成错误");
        assert!(failure.to_string().contains("上游没连上"), "错误正文要透传");
    }
}
