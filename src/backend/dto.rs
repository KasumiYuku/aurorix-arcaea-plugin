use std::time::Duration;

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, Offset, TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::model::{Difficulty, Grade};

use super::error::BackendError;

pub const CN_OFFSET_SECONDS: i32 = 8 * 3600;

fn cn_offset() -> FixedOffset {
    FixedOffset::east_opt(CN_OFFSET_SECONDS).unwrap_or_else(|| Utc.fix())
}

fn from_cn_local(naive: NaiveDateTime) -> DateTime<Utc> {
    let offset = cn_offset();
    match offset.from_local_datetime(&naive) {
        chrono::LocalResult::Single(local) => local.with_timezone(&Utc),
        chrono::LocalResult::Ambiguous(first, _) => first.with_timezone(&Utc),
        chrono::LocalResult::None => DateTime::from_naive_utc_and_offset(naive, Utc),
    }
}

pub fn parse_time(text: &str) -> Option<DateTime<Utc>> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(parsed) = DateTime::parse_from_rfc3339(trimmed) {
        return Some(parsed.with_timezone(&Utc));
    }
    for format in [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S%.f",
    ] {
        if let Ok(naive) = NaiveDateTime::parse_from_str(trimmed, format) {
            return Some(from_cn_local(naive));
        }
    }
    for format in ["%Y-%m-%d", "%Y/%m/%d"] {
        if let Ok(date) = NaiveDate::parse_from_str(trimmed, format) {
            return date.and_hms_opt(0, 0, 0).map(from_cn_local);
        }
    }
    None
}

pub fn opt_time<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = Option::<String>::deserialize(deserializer)?;
    Ok(raw.as_deref().filter(|s| !s.is_empty()).and_then(parse_time))
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Binding {
    #[serde(default)]
    pub arcaea_uid: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub friend_code: String,
    #[serde(default)]
    pub rating: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Player {
    #[serde(default)]
    pub arcaea_uid: i64,
    #[serde(default)]
    pub friend_code: String,
    #[serde(flatten)]
    pub identity: PlayerIdentity,
}

impl Player {
    pub fn has_rating(&self) -> bool {
        self.identity.ptt().is_some()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct SkinFields {
    #[serde(default)]
    pub character: Option<i32>,
    #[serde(default)]
    pub char_uncapped: bool,
    #[serde(default)]
    pub char_override: bool,
    #[serde(default)]
    pub banner: String,
    #[serde(default)]
    pub char_pref: String,
    #[serde(default)]
    pub banner_pref: String,
    #[serde(default)]
    pub pfp_pref: String,
}

impl SkinFields {
    pub fn known(&self) -> bool {
        self.character.is_some_and(|id| id >= 0)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct PlayerIdentity {
    #[serde(default)]
    pub name: String,
    #[serde(default, alias = "rating")]
    pub current_ptt: Option<f64>,
    #[serde(default)]
    pub rating_frame: Option<i32>,
    #[serde(default)]
    pub rating_accent: Option<i32>,
    #[serde(default)]
    pub global_rank: Option<i32>,
    #[serde(flatten)]
    pub skin: SkinFields,
}

impl PlayerIdentity {
    pub fn ptt(&self) -> Option<f64> {
        self.current_ptt.filter(|value| *value > 0.0)
    }

    pub fn frame(&self) -> Option<i32> {
        self.rating_frame.filter(|value| *value >= 0)
    }

    pub fn accent(&self) -> Option<i32> {
        self.rating_accent.filter(|value| *value >= 0)
    }

    pub fn rank(&self) -> Option<i32> {
        self.global_rank.filter(|value| *value >= 1)
    }

    pub fn named(&self, fallback: &str) -> Self {
        let mut out = self.clone();
        if out.name.is_empty() {
            out.name = fallback.to_string();
        }
        out
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct ScoreRecord {
    #[serde(default, alias = "SongID")]
    pub song_id: String,
    #[serde(default, alias = "Title")]
    pub title: String,
    #[serde(default, alias = "Difficulty")]
    pub difficulty: i32,
    #[serde(default)]
    pub difficulty_label: String,
    #[serde(default, alias = "Score")]
    pub score: i32,
    #[serde(default, alias = "Rating")]
    pub rating: f64,
    #[serde(default, alias = "Constant")]
    pub constant: f64,
    #[serde(default, alias = "Grade")]
    pub rank: String,
    #[serde(default, alias = "NextRank")]
    pub next_rank: String,
    #[serde(default, alias = "NextNeed")]
    pub next_need: i32,
    #[serde(default, alias = "Perfect")]
    pub perfect: i32,
    #[serde(default, alias = "ShinyPerfect")]
    pub shiny_perfect: i32,
    #[serde(default, alias = "NormalPerfect")]
    pub normal_perfect: i32,
    #[serde(default, alias = "Far")]
    pub far: i32,
    #[serde(default, alias = "Lost")]
    pub lost: i32,
    #[serde(default, alias = "ObjectAmount")]
    pub object_amount: i32,
    #[serde(default, alias = "LoseScore")]
    pub lose_score: Option<f64>,
    #[serde(default, alias = "EquivalentFar")]
    pub equivalent_far: i32,
    #[serde(default, alias = "HasJudgement")]
    pub has_judgement: Option<bool>,
    #[serde(default, alias = "ClearType")]
    pub clear_type: Option<i32>,
    #[serde(default, alias = "TimePlayed")]
    pub time_played: i64,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub chart_designer: String,
}

impl ScoreRecord {
    pub fn difficulty_kind(&self) -> Option<Difficulty> {
        difficulty_of(self.difficulty, &self.difficulty_label)
    }

    pub fn grade(&self) -> Option<Grade> {
        Grade::parse(self.rank.as_str())
    }

    pub fn judged(&self) -> bool {
        self.has_judgement
            .unwrap_or(self.perfect + self.far + self.lost > 0)
    }
}

pub fn difficulty_of(rating_class: i32, label: &str) -> Option<Difficulty> {
    match Difficulty::parse(label) {
        Some(difficulty) if i32::from(difficulty.as_upstream()) == rating_class => Some(difficulty),
        _ => Difficulty::from_upstream(i64::from(rating_class)),
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Progress {
    #[serde(default)]
    pub done: u32,
    #[serde(default)]
    pub total: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct EtaPayload {
    #[serde(default)]
    pub eta_seconds: Option<u32>,
    #[serde(default)]
    pub eta_low_seconds: Option<u32>,
    #[serde(default)]
    pub eta_high_seconds: Option<u32>,
    #[serde(default)]
    pub eta_remaining_seconds: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EtaEstimate {
    pub seconds: Option<u32>,
    pub low: Option<u32>,
    pub high: Option<u32>,
    pub remaining: Option<u32>,
}

impl EtaEstimate {
    pub fn new(
        seconds: Option<u32>,
        low: Option<u32>,
        high: Option<u32>,
        remaining: Option<u32>,
    ) -> Self {
        Self {
            seconds,
            low,
            high,
            remaining,
        }
    }

    pub fn is_known(&self) -> bool {
        self.seconds.is_some()
            || self.low.is_some()
            || self.high.is_some()
            || self.remaining.is_some()
    }

    pub fn finish(&self) -> Duration {
        let seconds = self.seconds.or(self.remaining).unwrap_or(0);
        Duration::from_secs(u64::from(seconds))
    }

    pub fn display(&self) -> String {
        if let Some(remaining) = self.remaining {
            return format!("还剩 {remaining} 秒");
        }
        match (self.low, self.high) {
            (Some(low), Some(high)) => format!("约 {low}~{high} 秒"),
            _ => match self.seconds {
                Some(seconds) => format!("预计 {seconds} 秒"),
                None => "预计时间未知".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TaskProgress {
    pub id: String,
    pub status: String,
    pub done: u32,
    pub total: u32,
    pub elapsed_ms: i64,
    pub eta: EtaEstimate,
}

impl TaskProgress {
    pub fn running(&self) -> bool {
        matches!(
            self.status.to_ascii_lowercase().as_str(),
            "pending" | "running" | "queued"
        )
    }

    pub fn failed(&self) -> bool {
        self.status.eq_ignore_ascii_case("failed")
    }

    pub fn completed(&self) -> bool {
        self.status.eq_ignore_ascii_case("completed")
    }

    pub fn percent(&self) -> Option<f32> {
        if self.total == 0 {
            return None;
        }
        Some((self.done as f32) * 100.0 / (self.total as f32))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct QueryMeta {
    pub source: String,
    pub cached: bool,
    pub cached_at: Option<DateTime<Utc>>,
    pub cooldown_seconds: i64,
    pub degraded: bool,
    pub degraded_reason: String,
    pub partial: bool,
    pub retry_after_seconds: Option<u64>,
    pub msg: String,
    pub task_id: String,
    pub task: Option<TaskProgress>,
    pub progress: Option<TaskProgress>,
    pub status: String,
    pub eta: EtaEstimate,
    pub elapsed_ms: i64,
    pub last_updated: Option<DateTime<Utc>>,
    pub kind: String,
    pub reason: String,
    pub upstream_attempted: bool,
}

impl QueryMeta {
    pub fn awaiting_task(&self) -> bool {
        fn active(status: &str) -> bool {
            matches!(
                status.to_ascii_lowercase().as_str(),
                "pending" | "running" | "queued"
            )
        }
        if active(self.status.as_str()) || active(self.kind.as_str()) {
            return true;
        }
        self.task.as_ref().is_some_and(TaskProgress::running)
            || self.progress.as_ref().is_some_and(TaskProgress::running)
    }

    pub fn live_task(&self) -> Option<&TaskProgress> {
        self.task
            .as_ref()
            .or(self.progress.as_ref())
            .filter(|task| task.running() || task.total > 0)
    }

    pub fn is_cached(&self) -> bool {
        self.cached
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct AnswerBlock {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub upstream_attempted: bool,
    #[serde(default)]
    pub elapsed_ms: i64,
    #[serde(default, deserialize_with = "opt_time")]
    pub cached_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub retry_after_seconds: i64,
}
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ScoreEnvelope {
    #[serde(default)]
    pub data: Option<serde_json::Value>,
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub cached: bool,
    #[serde(default, deserialize_with = "opt_time")]
    pub cached_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub cooldown_remaining_seconds: i64,
    #[serde(default)]
    pub degraded: bool,
    #[serde(default)]
    pub degraded_reason: String,
    #[serde(default)]
    pub partial: bool,
    #[serde(default)]
    pub retry_after_seconds: i64,
    #[serde(default)]
    pub msg: String,
    #[serde(default)]
    pub task_id: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default, deserialize_with = "de_string_or_number")]
    pub status: String,
    #[serde(default)]
    pub elapsed_ms: i64,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub progress: Option<Progress>,
    #[serde(default, deserialize_with = "opt_time")]
    pub last_updated: Option<DateTime<Utc>>,
    #[serde(default)]
    pub eta_seconds: Option<u32>,
    #[serde(default)]
    pub eta_low_seconds: Option<u32>,
    #[serde(default)]
    pub eta_high_seconds: Option<u32>,
    #[serde(default)]
    pub eta_remaining_seconds: Option<u32>,
    #[serde(default)]
    pub answer: Option<AnswerBlock>,
}

impl ScoreEnvelope {
    pub fn payload(&self) -> Option<&serde_json::Value> {
        self.data.as_ref().or(self.result.as_ref())
    }

    pub fn eta(&self) -> EtaEstimate {
        EtaEstimate::new(
            self.eta_seconds,
            self.eta_low_seconds,
            self.eta_high_seconds,
            self.eta_remaining_seconds,
        )
    }

    pub fn task(&self) -> Option<TaskProgress> {
        if self.task_id.is_empty() {
            return None;
        }
        let progress = self.progress.clone().unwrap_or_default();
        Some(TaskProgress {
            id: self.task_id.clone(),
            status: self.status.clone(),
            done: progress.done,
            total: progress.total,
            elapsed_ms: self.elapsed_ms,
            eta: self.eta(),
        })
    }

    pub fn meta(&self) -> QueryMeta {
        let mut degraded_reason = self.degraded_reason.clone();
        if self.degraded && degraded_reason.is_empty() {
            degraded_reason = "partial".to_string();
        }
        let mut kind = String::new();
        let mut reason = String::new();
        let mut upstream_attempted = false;
        let mut cached_at = self.cached_at.or(self.last_updated);
        let mut elapsed_ms = self.elapsed_ms;
        let mut retry_after_seconds = (self.retry_after_seconds > 0)
            .then_some(self.retry_after_seconds as u64);
        if let Some(answer) = self.answer.as_ref() {
            kind = answer.kind.clone();
            reason = answer.reason.clone();
            upstream_attempted = answer.upstream_attempted;
            if answer.elapsed_ms > 0 {
                elapsed_ms = answer.elapsed_ms;
            }
            if cached_at.is_none() {
                cached_at = answer.cached_at;
            }
            if retry_after_seconds.is_none() && answer.retry_after_seconds > 0 {
                retry_after_seconds = Some(answer.retry_after_seconds as u64);
            }
        }
        if self.degraded || self.partial {
            kind = "degraded".to_string();
            reason = if !degraded_reason.is_empty() {
                degraded_reason.clone()
            } else {
                "partial".to_string()
            };
        }
        let task = self.task();
        QueryMeta {
            source: self.source.clone(),
            cached: self.cached,
            cached_at,
            cooldown_seconds: self.cooldown_remaining_seconds,
            degraded: self.degraded,
            degraded_reason,
            partial: self.partial,
            retry_after_seconds,
            msg: self.msg.clone(),
            task_id: self.task_id.clone(),
            task: task.clone(),
            progress: task,
            status: self.status.clone(),
            eta: self.eta(),
            elapsed_ms,
            last_updated: self.last_updated,
            kind,
            reason,
            upstream_attempted,
        }
    }

    pub fn terminal_error(&self) -> Option<BackendError> {
        if !self.status.eq_ignore_ascii_case("failed") && self.error.is_empty() {
            return None;
        }
        let message = if self.error.is_empty() {
            "扫描失败，请稍后重试".to_string()
        } else {
            self.error.clone()
        };
        Some(BackendError::Upstream {
            code: "scan_failed".to_string(),
            message,
            request_id: None,
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct B50Payload {
    #[serde(flatten)]
    pub identity: PlayerIdentity,
    #[serde(default)]
    pub best50: f64,
    #[serde(default)]
    pub best10: f64,
    #[serde(default)]
    pub max_ptt: f64,
    #[serde(default)]
    pub count: i32,
    #[serde(default)]
    pub records: Vec<ScoreRecord>,
    #[serde(default)]
    pub meta: Option<QueryMeta>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct B50 {
    pub identity: PlayerIdentity,
    pub best50: f64,
    pub best10: f64,
    pub max_ptt: f64,
    pub count: i32,
    pub records: Vec<ScoreRecord>,
    pub meta: QueryMeta,
}

impl B50 {
    pub fn from_payload(payload: B50Payload, meta: QueryMeta) -> Self {
        Self {
            identity: payload.identity,
            best50: payload.best50,
            best10: payload.best10,
            max_ptt: payload.max_ptt,
            count: payload.count,
            records: payload.records,
            meta,
        }
    }

    pub fn empty(meta: QueryMeta) -> Self {
        Self {
            meta,
            ..Self::default()
        }
    }

    pub fn has_records(&self) -> bool {
        !self.records.is_empty()
    }

    pub fn top(&self, count: usize) -> &[ScoreRecord] {
        let end = count.min(self.records.len());
        &self.records[..end]
    }

    pub fn payload(&self) -> B50Payload {
        B50Payload {
            identity: self.identity.clone(),
            best50: self.best50,
            best10: self.best10,
            max_ptt: self.max_ptt,
            count: self.count,
            records: self.records.clone(),
            meta: Some(self.meta.clone()),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct RecentPayload {
    #[serde(default)]
    pub found: bool,
    #[serde(default)]
    pub record: Option<ScoreRecord>,
    #[serde(flatten)]
    pub identity: PlayerIdentity,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RecentPlay {
    pub found: bool,
    pub record: Option<ScoreRecord>,
    pub identity: PlayerIdentity,
    pub meta: QueryMeta,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AllScores {
    pub records: Vec<ScoreRecord>,
    pub meta: QueryMeta,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ScoreSource {
    #[default]
    Local,
    Probe,
}

impl ScoreSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Probe => "probe",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct PrefsPayload {
    #[serde(default)]
    pub char: String,
    #[serde(default)]
    pub banner: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SongQuery {
    pub text: String,
    pub filter: String,
    pub sort: String,
    pub offset: u32,
    pub limit: Option<u32>,
}

impl SongQuery {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Self::default()
        }
    }

    pub fn with_filter(mut self, filter: impl Into<String>) -> Self {
        self.filter = filter.into();
        self
    }

    pub fn with_sort(mut self, sort: impl Into<String>) -> Self {
        self.sort = sort.into();
        self
    }

    pub fn with_offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct SongSearch {
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub items: Vec<SongHit>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct SongHit {
    #[serde(default)]
    pub song: Song,
    #[serde(default)]
    pub score: f64,
    #[serde(default, rename = "match")]
    pub matched: String,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Song {
    #[serde(default)]
    pub song_id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub bpm: String,
    #[serde(default)]
    pub set: String,
    #[serde(default)]
    pub pack_name: String,
    #[serde(default)]
    pub pack_name_zh: String,
    #[serde(default)]
    pub pack_aliases: Vec<String>,
    #[serde(default)]
    pub release_ts: i64,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub audio_url: String,
    #[serde(default)]
    pub charts: Vec<Chart>,
}

impl Song {
    pub fn chart(&self, difficulty: Difficulty) -> Option<&Chart> {
        let wanted = i32::from(difficulty.as_upstream());
        self.charts
            .iter()
            .find(|chart| chart.rating_class == wanted)
    }

    pub fn artist_display(&self, difficulty: Difficulty) -> String {
        merge_display(
            &self.artist,
            self.chart(difficulty).map(|c| c.artist.as_str()).unwrap_or(""),
            difficulty,
        )
    }

    pub fn bpm_display(&self, difficulty: Difficulty) -> String {
        merge_display(
            &self.bpm,
            self.chart(difficulty).map(|c| c.bpm.as_str()).unwrap_or(""),
            difficulty,
        )
    }

    pub fn has_alias(&self, needle: &str) -> bool {
        let needle = needle.trim().to_lowercase();
        self.aliases
            .iter()
            .any(|alias| alias.trim().to_lowercase() == needle)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Chart {
    #[serde(default)]
    pub rating_class: i32,
    #[serde(default)]
    pub difficulty: String,
    #[serde(default)]
    pub rating: i32,
    #[serde(default)]
    pub rating_display: String,
    #[serde(default)]
    pub constant: Option<f64>,
    #[serde(default)]
    pub note: Option<i32>,
    #[serde(default, alias = "MaxLose")]
    pub max_lose: Option<f64>,
    #[serde(default)]
    pub chart_designer: String,
    #[serde(default)]
    pub chart_url: String,
    #[serde(default)]
    pub ill_url: String,
    #[serde(default)]
    pub audio_url: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub bpm: String,
}
impl Chart {
    pub fn difficulty_kind(&self) -> Option<Difficulty> {
        difficulty_of(self.rating_class, &self.difficulty)
    }
}

fn merge_display(base: &str, own: &str, difficulty: Difficulty) -> String {
    let base = base.trim();
    let own = own.trim();
    if own.is_empty() || own == base {
        return base.to_string();
    }
    if base.is_empty() {
        return format!("{} [{}]", own, difficulty.as_str());
    }
    format!("{} / {} [{}]", base, own, difficulty.as_str())
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChartPreview {
    pub song_id: String,
    pub title: String,
    pub artist: String,
    pub difficulty: Option<Difficulty>,
    pub rating_class: i32,
    pub constant: Option<f64>,
    pub note: Option<i32>,
    pub chart_designer: String,
    pub chart_url: String,
    pub ill_url: String,
    pub audio_url: String,
}

impl ChartPreview {
    pub fn from_song(song: &Song, chart: &Chart) -> Self {
        Self {
            song_id: song.song_id.clone(),
            title: song.title.clone(),
            artist: song.artist.clone(),
            difficulty: chart.difficulty_kind(),
            rating_class: chart.rating_class,
            constant: chart.constant,
            note: chart.note,
            chart_designer: chart.chart_designer.clone(),
            chart_url: chart.chart_url.clone(),
            ill_url: chart.ill_url.clone(),
            audio_url: chart.audio_url.clone(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct SongBoard {
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub rows: Vec<SongBoardRow>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct SongBoardRow {
    #[serde(default)]
    pub rank: i32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub score: i32,
    #[serde(default)]
    pub rating: Option<f64>,
    #[serde(default)]
    pub rating_frame: i32,
    #[serde(default)]
    pub rating_accent: i32,
    #[serde(default)]
    pub grade: String,
    #[serde(default)]
    pub perfect: i32,
    #[serde(default)]
    pub shiny_perfect: i32,
    #[serde(default)]
    pub normal_perfect: i32,
    #[serde(default)]
    pub far: i32,
    #[serde(default)]
    pub lost: i32,
    #[serde(default)]
    pub has_judgement: Option<bool>,
    #[serde(default)]
    pub character: Option<i32>,
    #[serde(default)]
    pub char_uncapped: Option<bool>,
    #[serde(flatten)]
    pub skin: SkinFields,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct ArcDataSync {
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default)]
    pub changed: bool,
    #[serde(default)]
    pub before: ArcDataCounts,
    #[serde(default)]
    pub after: ArcDataCounts,
    #[serde(default)]
    pub added: ArcDataChange,
    #[serde(default)]
    pub removed: ArcDataChange,
}

impl ArcDataSync {
    pub fn song_delta(&self) -> i64 {
        self.after.songs - self.before.songs
    }

    pub fn chart_delta(&self) -> i64 {
        self.after.charts - self.before.charts
    }

    pub fn alias_delta(&self) -> i64 {
        self.after.aliases - self.before.aliases
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct ArcDataCounts {
    #[serde(default)]
    pub songs: i64,
    #[serde(default)]
    pub charts: i64,
    #[serde(default)]
    pub aliases: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct ArcDataChange {
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub ids: Vec<String>,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct UpstreamState {
    #[serde(default)]
    pub closed: bool,
    #[serde(default)]
    pub manual: bool,
    #[serde(default)]
    pub reason: String,
    #[serde(default, deserialize_with = "opt_time")]
    pub since: Option<DateTime<Utc>>,
    #[serde(default, deserialize_with = "opt_time")]
    pub auto_until: Option<DateTime<Utc>>,
    #[serde(default)]
    pub desc: String,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct CnLoginStart {
    #[serde(default)]
    pub ticket_id: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub user_code: String,
    #[serde(default, deserialize_with = "opt_time")]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub interval_seconds: u32,
    #[serde(default)]
    pub reused: bool,
    #[serde(default)]
    pub device_id: String,
}

impl CnLoginStart {
    pub const DEFAULT_INTERVAL_SECONDS: u32 = 5;

    pub fn interval(&self) -> Duration {
        let seconds = if self.interval_seconds == 0 {
            Self::DEFAULT_INTERVAL_SECONDS
        } else {
            self.interval_seconds
        };
        Duration::from_secs(u64::from(seconds))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CnLoginStatus {
    #[default]
    Pending,
    Ready,
    Expired,
    Failed,
    #[serde(other)]
    Unknown,
}

impl CnLoginStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Ready => "ready",
            Self::Expired => "expired",
            Self::Failed => "failed",
            Self::Unknown => "unknown",
        }
    }

    pub fn waiting(self) -> bool {
        matches!(self, Self::Pending | Self::Unknown)
    }

    pub fn terminal(self) -> bool {
        matches!(self, Self::Ready | Self::Expired | Self::Failed)
    }

    pub fn ready(self) -> bool {
        matches!(self, Self::Ready)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct CnPlayer {
    #[serde(default)]
    pub arcaea_uid: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub friend_code: String,
    #[serde(default)]
    pub device_id: String,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct CnLoginPoll {
    #[serde(default)]
    pub status: CnLoginStatus,
    #[serde(default)]
    pub player: Option<CnPlayer>,
    #[serde(default)]
    pub detail: String,
    #[serde(default)]
    pub retryable: bool,
}

impl CnLoginPoll {
    pub fn waiting(&self) -> bool {
        self.status.waiting()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CnLoginWait {
    pub polls: u32,
    pub elapsed: Duration,
    pub timed_out: bool,
    pub poll: CnLoginPoll,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct CnRenew {
    #[serde(default)]
    pub renewed: bool,
    #[serde(default)]
    pub need_scan: bool,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub cooldown_seconds: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct CnRefresh {
    #[serde(default, rename = "score_count")]
    pub score_count: i64,
    #[serde(default, rename = "known_count")]
    pub known_count: i64,
    #[serde(default)]
    pub cooldown_seconds: i64,
    #[serde(default, deserialize_with = "opt_time")]
    pub next_allowed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub need_relogin: bool,
    #[serde(default, deserialize_with = "opt_time")]
    pub last_login_at: Option<DateTime<Utc>>,
    #[serde(default, rename = "last_failed")]
    pub last_failed: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct FriendCodeRequest {
    pub friend_code: String,
}

impl FriendCodeRequest {
    pub fn new(friend_code: impl Into<String>) -> Self {
        Self {
            friend_code: friend_code.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ProfileQuery {
    #[serde(default)]
    pub days: u32,
    #[serde(default)]
    pub changes: u32,
    #[serde(default)]
    pub board: u32,
    #[serde(default)]
    pub around: u32,
}

impl Default for ProfileQuery {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileQuery {
    pub const DEFAULT_DAYS: u32 = 365;
    pub const DEFAULT_CHANGES: u32 = 8;
    pub const DEFAULT_BOARD: u32 = 12;
    pub const DEFAULT_AROUND: u32 = 2;

    pub fn new() -> Self {
        Self {
            days: Self::DEFAULT_DAYS,
            changes: Self::DEFAULT_CHANGES,
            board: Self::DEFAULT_BOARD,
            around: Self::DEFAULT_AROUND,
        }
    }

    pub fn trend(days: u32) -> Self {
        Self {
            days,
            changes: 1,
            board: 1,
            around: 0,
        }
    }

    pub fn params(&self) -> Vec<(&'static str, String)> {
        vec![
            ("days", self.days.to_string()),
            ("changes", self.changes.to_string()),
            ("board", self.board.to_string()),
            ("around", self.around.to_string()),
        ]
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct ProfilePlayer {
    #[serde(default)]
    pub arcaea_uid: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub friend_code: String,
    #[serde(default)]
    pub rating: Option<f64>,
    #[serde(default)]
    pub rating_frame: Option<i32>,
    #[serde(default)]
    pub rating_accent: Option<i32>,
    #[serde(default)]
    pub global_rank: Option<i32>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct PlayerRank {
    #[serde(default)]
    pub rank: i64,
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub rating: Option<f64>,
}

impl PlayerRank {
    pub fn listed(&self) -> bool {
        self.rank > 0
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct ProfileBoardRow {
    #[serde(default)]
    pub rank: i64,
    #[serde(flatten)]
    pub identity: PlayerIdentity,
    #[serde(default)]
    pub uid: String,
    #[serde(default, rename = "me")]
    pub is_me: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct ProfileBoard {
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub rows: Vec<ProfileBoardRow>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct ProfilePoint {
    #[serde(default)]
    pub day: String,
    #[serde(default)]
    pub rating: Option<f64>,
    #[serde(default)]
    pub records: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct ProfileChange {
    #[serde(default)]
    pub song_id: String,
    #[serde(default)]
    pub difficulty: i32,
    #[serde(default)]
    pub has_old: bool,
    #[serde(default)]
    pub old_score: i32,
    #[serde(default)]
    pub new_score: i32,
    #[serde(default, deserialize_with = "opt_time")]
    pub at: Option<DateTime<Utc>>,
}

impl ProfileChange {
    pub fn difficulty_kind(&self) -> Option<Difficulty> {
        Difficulty::from_upstream(i64::from(self.difficulty))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub bound: bool,
    #[serde(default)]
    pub player: Option<ProfilePlayer>,
    #[serde(default)]
    pub rank: Option<PlayerRank>,
    #[serde(default)]
    pub board: Option<ProfileBoard>,
    #[serde(default)]
    pub curve: Vec<ProfilePoint>,
    #[serde(default)]
    pub changes: Vec<ProfileChange>,
}

impl Profile {
    pub fn name(&self) -> &str {
        self.player
            .as_ref()
            .map(|player| player.name.as_str())
            .unwrap_or_default()
    }

    pub fn points(&self) -> Vec<(Option<DateTime<Utc>>, Option<f64>)> {
        self.curve
            .iter()
            .map(|point| (parse_time(point.day.as_str()), point.rating))
            .collect()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct PlayerBoardRow {
    #[serde(default)]
    pub difficulty_label: String,
    #[serde(default)]
    pub song_id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub difficulty: i32,
    #[serde(default)]
    pub score: i32,
    #[serde(default)]
    pub constant: f64,
    #[serde(default)]
    pub rating: f64,
    #[serde(default)]
    pub rating_frame: i32,
    #[serde(default)]
    pub rank: String,
    #[serde(default)]
    pub perfect: i32,
    #[serde(default)]
    pub shiny_perfect: i32,
    #[serde(default)]
    pub normal_perfect: i32,
    #[serde(default)]
    pub far: i32,
    #[serde(default)]
    pub lost: i32,
    #[serde(default)]
    pub object_amount: i32,
    #[serde(default)]
    pub has_judgement: Option<bool>,
    #[serde(default)]
    pub clear_type: Option<i32>,
    #[serde(default)]
    pub time_played: i64,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub chart_designer: String,
    #[serde(default)]
    pub next_rank: String,
    #[serde(default)]
    pub next_need: i32,
    #[serde(default)]
    pub lose_score: f64,
    #[serde(default)]
    pub equivalent_far: i32,
    #[serde(default)]
    pub board_rank: i64,
    #[serde(default)]
    pub over_level: Option<f64>,
    #[serde(default)]
    pub over_quantum: Option<f64>,
    #[serde(default)]
    pub over_delta: Option<f64>,
    #[serde(default)]
    pub over_mess: Option<f64>,
    #[serde(default)]
    pub over_band: String,
    #[serde(default)]
    pub over_far_share: Option<f64>,
    #[serde(default)]
    pub over_lost_share: Option<f64>,
    #[serde(default)]
    pub has_over_level: Option<bool>,
    #[serde(default)]
    pub pressure: Option<f64>,
}

impl PlayerBoardRow {
    pub fn difficulty_kind(&self) -> Option<Difficulty> {
        difficulty_of(self.difficulty, &self.difficulty_label)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct PlayerBoard {
    #[serde(flatten)]
    pub identity: PlayerIdentity,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub matched: i64,
    #[serde(default)]
    pub known: i64,
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub has_curve: bool,
    #[serde(default)]
    pub anchor: Option<f64>,
    #[serde(default)]
    pub anchor_sd: f64,
    #[serde(default)]
    pub anchor_plays: i64,
    #[serde(default)]
    pub anchor_upper: Option<f64>,
    #[serde(default)]
    pub anchor_source: String,
    #[serde(default)]
    pub band_counts: std::collections::HashMap<String, i64>,
    #[serde(default)]
    pub avg_best10: Option<f64>,
    #[serde(default)]
    pub avg_best50: Option<f64>,
    #[serde(default)]
    pub best_count: i64,
    #[serde(default)]
    pub over_count: i64,
    #[serde(default)]
    pub calibrated: bool,
    #[serde(default)]
    pub mean_gap: f64,
    #[serde(default)]
    pub mean_pressure: f64,
    #[serde(default)]
    pub rows: Vec<PlayerBoardRow>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct StatsQuery {
    #[serde(default)]
    pub from: String,
    #[serde(default)]
    pub to: String,
    #[serde(default)]
    pub diff: Vec<String>,
    #[serde(default)]
    pub view: String,
    #[serde(default)]
    pub short: Option<u32>,
}

impl StatsQuery {
    pub const OPEN_END: &'static str = "+Inf";

    pub fn new(from: impl Into<String>, view: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: Self::OPEN_END.to_string(),
            diff: Vec::new(),
            view: view.into(),
            short: None,
        }
    }

    pub fn with_to(mut self, to: impl Into<String>) -> Self {
        self.to = to.into();
        self
    }

    pub fn with_diff(mut self, diff: Vec<String>) -> Self {
        self.diff = diff;
        self
    }

    pub fn with_short(mut self, short: u32) -> Self {
        self.short = Some(short);
        self
    }

    fn diff_text(&self) -> String {
        self.diff
            .iter()
            .map(|item| item.trim())
            .filter(|item| !item.is_empty())
            .collect::<Vec<&str>>()
            .join(",")
    }

    fn to_text(&self) -> String {
        let to = self.to.trim();
        if to.is_empty() || to.eq_ignore_ascii_case(Self::OPEN_END) {
            String::new()
        } else {
            to.to_string()
        }
    }

    pub fn params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![
            ("from", self.from.trim().to_string()),
            ("to", self.to_text()),
            ("diff", self.diff_text()),
            ("view", self.view.trim().to_string()),
        ];
        if let Some(short) = self.short.filter(|short| *short > 0) {
            params.push(("short", short.to_string()));
        }
        params
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct StatsBand {
    #[serde(default)]
    pub difficulty: i32,
    #[serde(default)]
    pub diff_label: String,
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub played: i64,
    #[serde(default)]
    pub cleared: i64,
    #[serde(default)]
    pub full_combo: i64,
    #[serde(default)]
    pub pm: i64,
    #[serde(default)]
    pub ex_plus: i64,
    #[serde(default)]
    pub ex: i64,
    #[serde(default)]
    pub loss_played: f64,
    #[serde(default)]
    pub loss_total: f64,
    #[serde(default)]
    pub avg_score: f64,
}

impl StatsBand {
    pub fn difficulty_kind(&self) -> Option<Difficulty> {
        difficulty_of(self.difficulty, &self.diff_label)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct StatsConst {
    #[serde(default)]
    pub constant: f64,
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub played: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct StatsGrade {
    #[serde(default)]
    pub grade: String,
    #[serde(default)]
    pub count: i64,
}

impl StatsGrade {
    pub fn grade_kind(&self) -> Option<Grade> {
        Grade::parse(self.grade.as_str())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct StatsShort {
    #[serde(default)]
    pub clear_type: Option<i32>,
    #[serde(default)]
    pub song_id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub difficulty: i32,
    #[serde(default)]
    pub constant: Option<f64>,
    #[serde(default)]
    pub has_constant: bool,
    #[serde(default)]
    pub score: i32,
    #[serde(default)]
    pub grade: String,
    #[serde(default)]
    pub next_rank: String,
    #[serde(default)]
    pub next_need: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct PlayerStats {
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub played: i64,
    #[serde(default)]
    pub cleared: i64,
    #[serde(default)]
    pub full_combo: i64,
    #[serde(default)]
    pub pm: i64,
    #[serde(default)]
    pub pm_small_p: i64,
    #[serde(default)]
    pub avg_score: f64,
    #[serde(default)]
    pub avg_rating: f64,
    #[serde(default)]
    pub bands: Vec<StatsBand>,
    #[serde(default)]
    pub consts: Vec<StatsConst>,
    #[serde(default)]
    pub grades: Vec<StatsGrade>,
    #[serde(default)]
    pub short: Option<Vec<StatsShort>>,
    #[serde(default)]
    pub known: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct SuggestQuery {
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub filter: String,
    #[serde(default)]
    pub q: String,
}

impl SuggestQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_filter(mut self, filter: impl Into<String>) -> Self {
        self.filter = filter.into();
        self
    }

    pub fn with_text(mut self, q: impl Into<String>) -> Self {
        self.q = q.into();
        self
    }

    pub fn params(&self) -> Vec<(&'static str, String)> {
        let mut params = Vec::new();
        if let Some(limit) = self.limit.filter(|limit| *limit > 0) {
            params.push(("limit", limit.to_string()));
        }
        params.push(("filter", self.filter.trim().to_string()));
        params.push(("q", self.q.trim().to_string()));
        params
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct SuggestNotes {
    #[serde(default)]
    pub known: bool,
    #[serde(default)]
    pub far_need: i64,
    #[serde(default)]
    pub lost_need: i64,
    #[serde(default)]
    pub far_room: i64,
    #[serde(default)]
    pub lost_room: i64,
    #[serde(default)]
    pub far_use: i64,
    #[serde(default)]
    pub lost_use: i64,
    #[serde(default)]
    pub short: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct SuggestHint {
    #[serde(default)]
    pub song_id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub difficulty: i32,
    #[serde(default)]
    pub constant: f64,
    #[serde(default)]
    pub note: i64,
    #[serde(default)]
    pub score: i32,
    #[serde(default)]
    pub has_score: bool,
    #[serde(default)]
    pub far: i32,
    #[serde(default)]
    pub lost: i32,
    #[serde(default)]
    pub has_judgement: Option<bool>,
    #[serde(default)]
    pub delta: f64,
    #[serde(default)]
    pub target: f64,
    #[serde(default)]
    pub rating: f64,
    #[serde(default)]
    pub target_rating: f64,
    #[serde(default)]
    pub rank: i64,
    #[serde(default)]
    pub target_rank: i64,
    #[serde(default)]
    pub counted: bool,
    #[serde(default)]
    pub gap: f64,
    #[serde(default)]
    pub reach: bool,
    #[serde(default)]
    pub pick: bool,
    #[serde(default)]
    pub expected_score: f64,
    #[serde(default)]
    pub headroom: f64,
    #[serde(default)]
    pub notes: SuggestNotes,
}

impl SuggestHint {
    pub fn difficulty_kind(&self) -> Option<Difficulty> {
        Difficulty::from_upstream(i64::from(self.difficulty))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct PlayerSuggest {
    #[serde(default)]
    pub displayed: f64,
    #[serde(default)]
    pub target: f64,
    #[serde(default)]
    pub enough: bool,
    #[serde(default)]
    pub pool: i64,
    #[serde(default)]
    pub maxed: i64,
    #[serde(default)]
    pub thin: bool,
    #[serde(default)]
    pub pushable: i64,
    #[serde(default)]
    pub has_ability: bool,
    #[serde(default)]
    pub anchor: Option<f64>,
    #[serde(default)]
    pub pick_count: i64,
    #[serde(default)]
    pub known: i64,
    #[serde(default)]
    pub hints: Vec<SuggestHint>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct ChapterQuery {
    #[serde(default)]
    pub q: String,
    #[serde(default)]
    pub rating_class: Vec<String>,
}

impl ChapterQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_text(mut self, q: impl Into<String>) -> Self {
        self.q = q.into();
        self
    }

    pub fn with_rating_class(mut self, rating_class: Vec<String>) -> Self {
        self.rating_class = rating_class;
        self
    }

    fn rating_text(&self) -> String {
        self.rating_class
            .iter()
            .map(|item| item.trim())
            .filter(|item| !item.is_empty())
            .collect::<Vec<&str>>()
            .join(",")
    }

    pub fn params(&self) -> Vec<(&'static str, String)> {
        vec![
            ("q", self.q.trim().to_string()),
            ("rating_class", self.rating_text()),
        ]
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct ChapterTotals {
    #[serde(default)]
    pub packs: i64,
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub played: i64,
    #[serde(default)]
    pub cleared: i64,
    #[serde(default)]
    pub full_combo: i64,
    #[serde(default)]
    pub pm: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct ChapterPack {
    #[serde(default)]
    pub pack_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub section: String,
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub played: i64,
    #[serde(default)]
    pub cleared: i64,
    #[serde(default)]
    pub full_combo: i64,
    #[serde(default)]
    pub pm: i64,
    #[serde(default)]
    pub avg_score: f64,
    #[serde(default)]
    pub avg_ptt: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct ChapterSection {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub packs: Vec<ChapterPack>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct ChapterChart {
    #[serde(default)]
    pub difficulty: String,
    #[serde(default)]
    pub song_id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub rating_class: i32,
    #[serde(default)]
    pub constant: Option<f64>,
    #[serde(default)]
    pub played: bool,
    #[serde(default)]
    pub score: i32,
    #[serde(default)]
    pub grade: String,
    #[serde(default)]
    pub ptt: f64,
    #[serde(default)]
    pub perfect: i32,
    #[serde(default)]
    pub shiny_perfect: i32,
    #[serde(default)]
    pub near: i32,
    #[serde(default)]
    pub miss: i32,
}

impl ChapterChart {
    pub fn difficulty_kind(&self) -> Option<Difficulty> {
        difficulty_of(self.rating_class, &self.difficulty)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct ChapterDetail {
    #[serde(default)]
    pub pack: ChapterPack,
    #[serde(default)]
    pub charts: Vec<ChapterChart>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct ChapterMatch {
    #[serde(default)]
    pub pack_id: String,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct PlayerChapters {
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub empty: bool,
    #[serde(default)]
    pub totals: ChapterTotals,
    #[serde(default)]
    pub sections: Vec<ChapterSection>,
    #[serde(default)]
    pub unfiled: Option<ChapterPack>,
    #[serde(default)]
    pub detail: Option<ChapterDetail>,
    #[serde(default)]
    pub matched: Vec<ChapterMatch>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct RatingEventPayload {
    #[serde(default, deserialize_with = "opt_time")]
    pub at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub rating: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RatingEvent {
    pub at: DateTime<Utc>,
    pub rating: f64,
}

impl RatingEvent {
    pub fn from_payload(payload: RatingEventPayload) -> Option<Self> {
        let at = payload.at?;
        let rating = payload.rating.filter(|rating| *rating > 0.0)?;
        Some(Self { at, rating })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct ScanEta {
    #[serde(default)]
    pub seconds: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct PlayPayload {
    #[serde(default)]
    pub found: bool,
    #[serde(default)]
    pub record: Option<ScoreRecord>,
    #[serde(flatten)]
    pub identity: PlayerIdentity,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Play {
    pub found: bool,
    pub record: Option<ScoreRecord>,
    pub identity: PlayerIdentity,
    pub meta: QueryMeta,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct SongHistoryRow {
    #[serde(default)]
    pub song_id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub constant: Option<f64>,
    #[serde(default)]
    pub difficulty: i32,
    #[serde(default)]
    pub has_old: bool,
    #[serde(default)]
    pub old_score: i32,
    #[serde(default)]
    pub old_perfect: i32,
    #[serde(default)]
    pub old_shiny_perfect: i32,
    #[serde(default)]
    pub old_near: i32,
    #[serde(default)]
    pub old_miss: i32,
    #[serde(default)]
    pub old_clear_type: Option<i32>,
    #[serde(default)]
    pub old_rating: Option<f64>,
    #[serde(default, deserialize_with = "opt_time")]
    pub old_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub new_score: i32,
    #[serde(default)]
    pub new_perfect: i32,
    #[serde(default)]
    pub new_shiny_perfect: i32,
    #[serde(default)]
    pub new_near: i32,
    #[serde(default)]
    pub new_miss: i32,
    #[serde(default)]
    pub new_clear_type: Option<i32>,
    #[serde(default)]
    pub new_rating: Option<f64>,
    #[serde(default)]
    pub lose_score: Option<f64>,
    #[serde(default)]
    pub account_rating: Option<f64>,
    #[serde(default, deserialize_with = "opt_time")]
    pub at: Option<DateTime<Utc>>,
}

impl SongHistoryRow {
    pub fn difficulty_kind(&self) -> Option<Difficulty> {
        Difficulty::from_upstream(i64::from(self.difficulty))
    }

    pub fn improved(&self) -> bool {
        self.has_old && self.new_score > self.old_score
    }
}

#[cfg(test)]
mod world_rank_tests {
    use super::*;

    #[test]
    fn player_global_rank_parses() {
        let ranked: Player = serde_json::from_str(
            r#"{"arcaea_uid":900001010,"name":"Ranker","rating":13.0,"rating_frame":6,"rating_accent":6,"global_rank":22}"#,
        )
        .expect("有名次的档案解析失败");
        assert_eq!(ranked.identity.global_rank, Some(22));

        let unranked: Player = serde_json::from_str(
            r#"{"arcaea_uid":900001011,"name":"Nobody","rating":0.638}"#,
        )
        .expect("没名次的档案解析失败");
        assert_eq!(
            unranked.identity.global_rank, None,
            "没上榜不该被解出任何名次"
        );
    }

    #[test]
    fn profile_player_global_rank_parses() {
        let ranked: ProfilePlayer = serde_json::from_str(
            r#"{"arcaea_uid":900001010,"name":"Ranker","rating":13.0,"rating_frame":6,"rating_accent":6,"global_rank":22}"#,
        )
        .expect("档案页解析失败");
        assert_eq!(ranked.global_rank, Some(22));

        let unranked: ProfilePlayer = serde_json::from_str(
            r#"{"arcaea_uid":900001011,"name":"Nobody","rating":0.638}"#,
        )
        .expect("档案页解析失败");
        assert_eq!(unranked.global_rank, None);
    }
}

#[cfg(test)]
mod diff_row_tests {
    use super::SongHistoryRow;

    #[test]
    fn diff_row_parses_timex_stamps() {
        let rows: Vec<SongHistoryRow> = serde_json::from_str(
            r#"[{"song_id":"rise","difficulty":2,"title":"Rise","artist":"Combatplayer","has_old":true,"old_score":9784945,"old_perfect":758,"old_shiny_perfect":682,"old_near":26,"old_miss":4,"old_clear_type":1,"new_score":9816697,"new_perfect":764,"new_shiny_perfect":708,"new_near":19,"new_miss":5,"new_clear_type":1,"account_rating":11.121,"constant":7.8,"lose_score":296.01156000000003,"at":"2026-11-27 21:23:55","old_at":"2026-10-31 12:15:46"},
{"song_id":"oblivia","difficulty":2,"title":"Oblivia","artist":"Saiph","new_score":9964277,"new_perfect":950,"new_shiny_perfect":900,"new_near":5,"new_miss":1,"new_clear_type":5,"at":"2026-11-27 17:23:29"}]"#,
        )
        .expect("成绩变化行必须能解析");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].old_at.expect("旧时间是解析出来的").to_rfc3339(), "2026-10-31T04:15:46+00:00");
        assert_eq!(rows[0].at.expect("新时间是解析出来的").to_rfc3339(), "2026-11-27T13:23:55+00:00");
        assert!(rows[0].improved());
        assert_eq!(rows[1].old_at, None, "没有旧成绩的行不该被塞时间");
        assert!(!rows[1].improved());
    }
}

fn de_string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Raw {
        Text(String),
        Number(i64),
        Flag(bool),
    }
    Ok(match Raw::deserialize(deserializer)? {
        Raw::Text(text) => text,
        Raw::Number(number) => number.to_string(),
        Raw::Flag(flag) => flag.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn song() -> Song {
        serde_json::from_str(
            r#"{"song_id":"axiumcrisis","title":"Axium Crisis","artist":"ak+q","bpm":"170",
                "charts":[{"rating_class":0,"rating_display":"3"},
                          {"rating_class":3,"rating_display":"9","artist":"ak+q (lowiro)","bpm":"170?"}]}"#,
        )
        .unwrap()
    }

    #[test]
    fn display_merges_difficulty_override() {
        let s = song();
        assert_eq!(
            s.artist_display(Difficulty::Beyond),
            "ak+q / ak+q (lowiro) [BYD]"
        );
        assert_eq!(s.bpm_display(Difficulty::Beyond), "170 / 170? [BYD]");
    }

    #[test]
    fn display_keeps_base_when_not_overridden() {
        let s = song();
        assert_eq!(s.artist_display(Difficulty::Past), "ak+q");
        assert_eq!(s.bpm_display(Difficulty::Past), "170");
    }

    #[test]
    fn display_keeps_base_when_override_is_identical() {
        let s: Song = serde_json::from_str(
            r#"{"artist":"Frums","bpm":"178","charts":[{"rating_class":2,"artist":"Frums","bpm":"178"}]}"#,
        )
        .unwrap();
        assert_eq!(s.artist_display(Difficulty::Future), "Frums");
        assert_eq!(s.bpm_display(Difficulty::Future), "178");
    }
}
