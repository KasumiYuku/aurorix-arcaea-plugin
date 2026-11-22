use crate::model::{Difficulty, Region};

use super::client::{decode_json, encode_segment, ArcBackendNext};
use super::dto::{
    ChapterQuery, Play, PlayPayload, PlayerBoard, PlayerChapters, PlayerRank, PlayerStats,
    PlayerSuggest, Profile, ProfileQuery, RatingEvent, RatingEventPayload, ScanEta, ScoreEnvelope,
    SongHistoryRow, StatsQuery, SuggestQuery,
};
use super::error::BackendError;

pub const BOARD_KINDS: [&str; 6] = ["pm50", "fr50", "x50", "nm50", "mx50", "ol50"];

fn local_fallback_worthy(error: &BackendError) -> bool {
    error.is_retryable()
        || error.status() == Some(503)
        || matches!(
            error.code(),
            Some("unavailable") | Some("external") | Some("timeout")
        )
}

impl ArcBackendNext {
    pub async fn profile(
        &self,
        region: Region,
        uid: i64,
        query: ProfileQuery,
    ) -> Result<Profile, BackendError> {
        let path = Self::player_path(region, uid, "/profile", query.params().as_slice())?;
        let body = self
            .get_bytes(region, path.as_str(), self.conn_timeout())
            .await?;
        decode_json(body.as_ref(), "玩家档案响应")
    }

    pub async fn boards(
        &self,
        region: Region,
        uid: i64,
        kind: &str,
        limit: Option<u32>,
    ) -> Result<PlayerBoard, BackendError> {
        let kind = kind.trim();
        if kind.is_empty() {
            return Ok(PlayerBoard::default());
        }
        let segment = encode_segment(kind)?;
        let params: Vec<(&str, String)> = limit
            .filter(|limit| *limit > 0)
            .map(|limit| vec![("limit", limit.to_string())])
            .unwrap_or_default();
        let path = Self::player_path(
            region,
            uid,
            format!("/boards/{segment}").as_str(),
            params.as_slice(),
        )?;
        let body = self
            .get_bytes(region, path.as_str(), self.conn_timeout())
            .await?;
        decode_json(body.as_ref(), "榜面响应")
    }

    pub async fn stats(
        &self,
        region: Region,
        uid: i64,
        query: StatsQuery,
    ) -> Result<PlayerStats, BackendError> {
        let path = Self::player_path(region, uid, "/stats", query.params().as_slice())?;
        let body = self
            .get_bytes(region, path.as_str(), self.conn_timeout())
            .await?;
        decode_json(body.as_ref(), "区间统计响应")
    }

    pub async fn suggest(
        &self,
        region: Region,
        uid: i64,
        query: SuggestQuery,
    ) -> Result<PlayerSuggest, BackendError> {
        let path = Self::player_path(region, uid, "/suggest", query.params().as_slice())?;
        let body = self
            .get_bytes(region, path.as_str(), self.conn_timeout())
            .await?;
        decode_json(body.as_ref(), "推分建议响应")
    }

    pub async fn chapters(
        &self,
        region: Region,
        uid: i64,
        query: ChapterQuery,
    ) -> Result<PlayerChapters, BackendError> {
        let path = Self::player_path(region, uid, "/chapters", query.params().as_slice())?;
        let body = self
            .get_bytes(region, path.as_str(), self.conn_timeout())
            .await?;
        decode_json(body.as_ref(), "曲包完成度响应")
    }

    pub async fn rating_events(
        &self,
        region: Region,
        uid: i64,
        days: Option<u32>,
        limit: Option<u32>,
    ) -> Result<Vec<RatingEvent>, BackendError> {
        let mut params: Vec<(&str, String)> = Vec::new();
        if let Some(days) = days.filter(|days| *days > 0) {
            params.push(("days", days.to_string()));
        }
        if let Some(limit) = limit.filter(|limit| *limit > 0) {
            params.push(("limit", limit.to_string()));
        }
        let path = Self::player_path(region, uid, "/rating-events", params.as_slice())?;
        let body = self
            .get_bytes(region, path.as_str(), self.conn_timeout())
            .await?;
        let payload: Vec<RatingEventPayload> = decode_json(body.as_ref(), "PTT 变化流水响应")?;
        Ok(payload
            .into_iter()
            .filter_map(RatingEvent::from_payload)
            .collect())
    }

    pub async fn rank(&self, region: Region, uid: i64) -> Result<PlayerRank, BackendError> {
        let path = Self::player_path(region, uid, "/rank", &[])?;
        let body = self
            .get_bytes(region, path.as_str(), self.conn_timeout())
            .await?;
        decode_json(body.as_ref(), "全站名次响应")
    }

    pub async fn eta(&self, region: Region, uid: i64) -> Result<ScanEta, BackendError> {
        let path = Self::player_path(region, uid, "/eta", &[])?;
        let body = self
            .get_bytes(region, path.as_str(), self.conn_timeout())
            .await?;
        decode_json(body.as_ref(), "扫描估时响应")
    }

    pub async fn score(
        &self,
        region: Region,
        uid: i64,
        song_id: &str,
        difficulty: Difficulty,
        local: bool,
    ) -> Result<Play, BackendError> {
        match self
            .score_once(region, uid, song_id, difficulty, local)
            .await
        {
            Ok(play) => Ok(play),
            Err(error) if !local && local_fallback_worthy(&error) => {
                self.score_once(region, uid, song_id, difficulty, true)
                    .await
            }
            Err(error) => Err(error),
        }
    }

    async fn score_once(
        &self,
        region: Region,
        uid: i64,
        song_id: &str,
        difficulty: Difficulty,
        local: bool,
    ) -> Result<Play, BackendError> {
        let song = encode_segment(song_id)?;
        let rating_class = difficulty.as_upstream();
        let params: Vec<(&str, String)> = if local {
            vec![("local", "true".to_string())]
        } else {
            Vec::new()
        };
        let path = Self::player_path(
            region,
            uid,
            format!("/scores/{song}/{rating_class}").as_str(),
            params.as_slice(),
        )?;
        let body = self
            .get_bytes(region, path.as_str(), self.conn_timeout())
            .await?;
        let envelope: ScoreEnvelope = serde_json::from_slice(body.as_ref())
            .map_err(|err| BackendError::decode(format!("单曲成绩元信息解析失败：{err}")))?;
        if let Some(error) = envelope.terminal_error() {
            return Err(error);
        }
        let meta = envelope.meta();
        let payload: PlayPayload = envelope
            .payload()
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default();
        Ok(Play {
            found: payload.found,
            record: payload.record,
            identity: payload.identity,
            meta,
        })
    }

    pub async fn history(
        &self,
        region: Region,
        uid: i64,
        song_id: &str,
        difficulty: Difficulty,
        limit: Option<u32>,
    ) -> Result<Vec<SongHistoryRow>, BackendError> {
        let song = encode_segment(song_id)?;
        let rating_class = difficulty.as_upstream();
        let params: Vec<(&str, String)> = limit
            .filter(|limit| *limit > 0)
            .map(|limit| vec![("limit", limit.to_string())])
            .unwrap_or_default();
        let path = Self::player_path(
            region,
            uid,
            format!("/scores/{song}/{rating_class}/history").as_str(),
            params.as_slice(),
        )?;
        let body = self
            .get_bytes(region, path.as_str(), self.conn_timeout())
            .await?;
        decode_json(body.as_ref(), "单曲成绩变迁响应")
    }

    pub async fn diff(
        &self,
        region: Region,
        uid: i64,
        days: u32,
        limit: Option<u32>,
    ) -> Result<Vec<SongHistoryRow>, BackendError> {
        let mut params: Vec<(&str, String)> = Vec::new();
        if days > 0 {
            params.push(("days", days.to_string()));
        }
        if let Some(limit) = limit.filter(|limit| *limit > 0) {
            params.push(("limit", limit.to_string()));
        }
        let path = Self::player_path(region, uid, "/diff", params.as_slice())?;
        let body = self
            .get_bytes(region, path.as_str(), self.conn_timeout())
            .await?;
        decode_json(body.as_ref(), "成绩变化响应")
    }
}
