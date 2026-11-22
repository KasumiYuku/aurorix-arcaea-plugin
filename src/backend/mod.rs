mod client;
mod cn;
mod dto;
mod error;
mod player;
mod sse;

pub use client::{
    encode_query_value, encode_segment, normalize_base_url, region_prefix, ArcBackendNext,
    BackendConfig, DEFAULT_SEARCH_LIMIT, DEFAULT_USER_AGENT,
};
pub use cn::DEFAULT_POLL_INTERVAL;
pub use dto::{
    parse_time, AllScores, ArcDataChange, ArcDataCounts, ArcDataSync, B50Payload, Binding,
    ChapterChart, ChapterDetail, ChapterMatch, ChapterPack, ChapterQuery, ChapterSection,
    ChapterTotals, Chart, ChartPreview, CnLoginPoll, CnLoginStart, CnLoginStatus, CnLoginWait,
    CnPlayer, CnRefresh, CnRenew, EtaEstimate, EtaPayload, FriendCodeRequest, Play, PlayPayload,
    Player, PlayerBoard, PlayerIdentity, PlayerBoardRow, PlayerChapters, PlayerRank, PlayerStats, PlayerSuggest,
    PrefsPayload, Profile, ProfileBoard, ProfileBoardRow, ProfileChange, ProfilePlayer,
    ProfilePoint, ProfileQuery, Progress, QueryMeta, RatingEvent, RatingEventPayload,
    RecentPayload, RecentPlay, ScanEta, ScoreEnvelope, ScoreRecord, ScoreSource, SkinFields, Song,
    SongBoard, SongBoardRow, SongHistoryRow, SongHit, SongQuery, SongSearch, StatsBand, StatsConst,
    StatsGrade, StatsQuery, StatsShort, SuggestHint, SuggestNotes, SuggestQuery, TaskProgress,
    UpstreamState, B50, CN_OFFSET_SECONDS,
};
pub use error::{truncate_body, BackendError, ProblemBody, ProblemField, MAX_ERROR_BODY_BYTES};
pub use player::BOARD_KINDS;
pub use sse::{
    b50_event_stream, b50_event_stream_from_body, events_from_envelope_body, parse_sse, B50Event,
    B50EventStream, B50Mapper, ProgressSource, SseEvent, SseItem, SseParser,
};

use crate::model::Region;

pub fn dedupe_key(region: Region, uid: i64, command: &str) -> String {
    format!(
        "arc:{}:{}:{}",
        region.as_str(),
        uid,
        normalize_command(command)
    )
}

pub fn normalize_command(command: &str) -> String {
    let mut out = String::with_capacity(command.len());
    let mut pending_space = false;
    for character in command.trim().chars() {
        if character.is_whitespace() {
            pending_space = !out.is_empty();
            continue;
        }
        if pending_space {
            out.push(' ');
            pending_space = false;
        }
        for lowered in character.to_lowercase() {
            out.push(lowered);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_381() {
        let first = dedupe_key(Region::Intl, 1001, "  B50   -f ");
        let second = dedupe_key(Region::Intl, 1001, "b50 -f");
        assert_eq!(first, second, "大小写与多余空白必须归一化");
        assert_eq!(first, "arc:intl:1001:b50 -f", "键的形状要固定");
        assert_ne!(
            first,
            dedupe_key(Region::Cn, 1001, "b50 -f"),
            "不同区域不能共用去重键"
        );
        assert_ne!(
            first,
            dedupe_key(Region::Intl, 1002, "b50 -f"),
            "不同用户不能共用去重键"
        );
    }

    #[test]
    fn case_382() {
        let keys: Vec<String> = (0..8)
            .map(|_| dedupe_key(Region::Cn, 42, "Recent"))
            .collect();
        assert!(
            keys.iter().all(|key| *key == keys[0]),
            "同一输入必须给出同一个键"
        );
    }

    #[test]
    fn case_383() {
        let 分隔 = ["/", "/"].concat();
        let 合法 = format!("https:{分隔}arc.example.com");
        let base = normalize_base_url(&合法).expect("合法地址要能归一化");
        assert!(base.ends_with('/'), "base 必须以斜杠结尾");
        assert_eq!(base, format!("{合法}/"), "只补一个斜杠");
        let 带查询串 = format!("http:{分隔}host?a=1");
        assert!(
            normalize_base_url(&带查询串).is_err(),
            "带查询串的地址必须拒绝"
        );
        assert!(
            normalize_base_url("ftp:/x").is_err(),
            "非 http 协议必须拒绝"
        );
        assert!(normalize_base_url("  ").is_err(), "空地址必须拒绝");
    }

    #[test]
    fn case_384() {
        let segment = encode_segment("a/b c").expect("非空片段可以编码");
        assert_eq!(segment, "a%2Fb%20c", "斜杠必须转义, 防止路径注入");
        assert!(encode_segment("   ").is_err(), "空白片段必须拒绝");
        assert_eq!(
            encode_segment("song-1.id_2").as_deref(),
            Ok("song-1.id_2"),
            "安全字符保持原样"
        );
    }
}
