use std::collections::BTreeMap;

use chrono::{DateTime, TimeZone, Utc};

use crate::backend as dto;
use crate::cards::b50::B50Input;
use crate::cards::gallery::{MeInput, MeScore};
use crate::cards::lists::{
    table_band_key, table_band_label, table_songs_sorted, AhvInput, AhvRecord, ListEntry,
    ListInput, TableCell, TableChart, TableInput, TableMode, TableSheet, TableSong,
    TABLE_CELL_ORDER,
};
use crate::cards::score::ScoreInput;
use crate::cards::search::{SearchHit, SearchInput};
use crate::cards::select::{SelectInput, SelectPlayer, SelectRow};
use crate::cards::song::{SongBoard, SongBoardRow, SongInput};
use crate::cards::stats::{
    ChapExpand, ChapInput, ChapPack, ChapSong, HisInput, HisRecord, HisSection, StatsBand,
    StatsConst, StatsGrade, StatsInput, StatsRow, SuggestInput, TrendInput, TrendPoint,
};
use crate::cards::theme::Theme;
use crate::cards::widgets::PlayerBadge;
use crate::domain::suggest::{
    NotePlan, SuggestBoard, SuggestGroup, SuggestHint as DomainHint, PICK_TITLE,
};
use crate::domain::{BoardRecord, BoardRow, Chart, ChartProbe, OverBand, Song, SongMatch};
use crate::model::{diff_order_weight, Difficulty, Grade, Prefs, Region};

pub const HIS_MAX_ROWS: usize = 200;

pub fn difficulty_of(record: &dto::ScoreRecord) -> Difficulty {
    record.difficulty_kind().unwrap_or(Difficulty::Future)
}

pub fn grade_of(record: &dto::ScoreRecord) -> Grade {
    Grade::from_score(i64::from(record.score))
}

pub fn achievement_of(record: &dto::ScoreRecord) -> Grade {
    if let Some(grade) = record.grade().or_else(|| Grade::parse(&record.rank)) {
        return grade;
    }
    crate::domain::ptt::grade_for(
        i64::from(record.score),
        i64::from(record.far),
        i64::from(record.lost),
        record.judged(),
    )
}

pub fn play_at(seconds: i64) -> Option<DateTime<Utc>> {
    if seconds <= 0 {
        return None;
    }
    Utc.timestamp_opt(seconds, 0).single()
}

pub fn clear_of(record: &dto::ScoreRecord) -> Option<i32> {
    if let Some(clear_type) = record.clear_type {
        return Some(clear_type);
    }
    if record.score >= 10_000_000 {
        return Some(3);
    }
    if record.judged() && record.lost == 0 && record.far == 0 && record.perfect > 0 {
        return Some(2);
    }
    None
}

pub fn board_record(record: &dto::ScoreRecord) -> BoardRecord {
    let difficulty = difficulty_of(record);
    let mut row = BoardRecord::new(
        record.song_id.clone(),
        record.title.clone(),
        difficulty,
        record.constant,
    )
    .with_score(i64::from(record.score))
    .with_judgement(
        i64::from(record.far),
        i64::from(record.lost),
        i64::from(record.object_amount),
    );
    if let Some(clear_type) = clear_of(record) {
        row = row.with_clear(clear_type);
    }
    row = row.with_grade(grade_of(record));
    if let Some(at) = play_at(record.time_played) {
        row = row.with_play_at(at);
    }
    row.normal_perfect = i64::from(record.normal_perfect);
    row.backend_rating = (record.rating > 0.0).then_some(record.rating);
    row.next_rank = record.next_rank.clone();
    row.next_need = i64::from(record.next_need);
    row.lose_score = record.lose_score.unwrap_or(0.0);
    row.equivalent_far = i64::from(record.equivalent_far);
    row
}

pub fn board_record_of(row: &dto::PlayerBoardRow) -> BoardRecord {
    let difficulty = row.difficulty_kind().unwrap_or(Difficulty::Future);
    let mut record = BoardRecord::new(
        row.song_id.clone(),
        row.title.clone(),
        difficulty,
        row.constant,
    )
    .with_score(i64::from(row.score));
    record.normal_perfect = i64::from(row.normal_perfect);
    let judged = row
        .has_judgement
        .unwrap_or(row.perfect + row.far + row.lost > 0);
    if judged {
        record = record.with_judgement(
            i64::from(row.far),
            i64::from(row.lost),
            i64::from(row.object_amount),
        );
    }
    let inferred = row.clear_type.or_else(|| (row.score >= 10_000_000).then_some(3));
    if let Some(clear_type) = inferred {
        record = record.with_clear(clear_type);
    }
    record = record.with_grade(Grade::from_score(i64::from(row.score)));
    if let Some(at) = play_at(row.time_played) {
        record = record.with_play_at(at);
    }
    record.over_level = row.over_level;
    record.over_band = OverBand::parse(&row.over_band);
    record.over_mess = row.over_mess;
    record.over_delta = row.over_delta;
    record.over_quantum = row.over_quantum;
    record.over_far_share = row.over_far_share;
    record.over_lost_share = row.over_lost_share;
    record.has_over_level = row.has_over_level.unwrap_or(row.over_level.is_some());
    record.backend_pressure = row.pressure;
    record.backend_rank = (row.board_rank > 0).then_some(row.board_rank);
    record.backend_rating = (row.rating > 0.0).then_some(row.rating);
    record.backend_rating_frame = (row.rating_frame >= 0).then_some(row.rating_frame);
    record.next_rank = row.next_rank.clone();
    record.next_need = i64::from(row.next_need);
    record.lose_score = row.lose_score;
    record.equivalent_far = i64::from(row.equivalent_far);
    record
}

pub fn board_rows(records: &[dto::ScoreRecord]) -> Vec<BoardRow> {
    records
        .iter()
        .enumerate()
        .map(|(index, record)| BoardRow::from_record(&board_record(record), index + 1))
        .collect()
}

pub fn badge_of(identity: &dto::PlayerIdentity, prefs: &Prefs) -> PlayerBadge {
    let skin = merged_skin(identity, prefs);
    let mut badge = PlayerBadge::new(identity.name.as_str());
    badge.ptt = identity.ptt();
    badge.subtitle = String::new();
    badge.frame = identity.frame().or_else(|| badge.ptt.map(frame_of));
    badge.star = badge.frame.map(star_of);
    badge.banner = banner_key(skin.banner_pref.as_str())
        .or_else(|| banner_key(skin.banner.as_str()))
        .or_else(|| Some(crate::cards::player::DEFAULT_BANNER.to_string()));
    if !skin.char_pref.is_empty() {
        let stem = char_stem(skin.char_pref.trim_end_matches(".png"), &skin);
        badge.character = Some(format!("characters/{stem}.png"));
        badge.avatar = Some(format!("char_icons/{stem}_icon.png"));
    } else if let Some(id) = skin.character {
        if let Some(stem) = char_stem_by_id(id, skin.char_uncapped && !skin.char_override) {
            badge.character = Some(format!("characters/{stem}.png"));
            badge.avatar = Some(format!("char_icons/{stem}_icon.png"));
        }
    }
    badge.global_rank = identity.rank();
    badge
}

fn frame_of(ptt: f64) -> i32 {
    i32::from(crate::domain::rating_frame_of(ptt))
}

pub fn frame_of_identity(identity: &dto::PlayerIdentity) -> Option<i32> {
    identity.frame().or_else(|| identity.ptt().map(frame_of))
}

fn merged_skin(identity: &dto::PlayerIdentity, prefs: &Prefs) -> dto::SkinFields {
    let mut skin = identity.skin.clone();
    if !prefs.char.trim().is_empty() {
        skin.char_pref = prefs.char.clone();
    }
    if !prefs.banner.trim().is_empty() {
        skin.banner_pref = prefs.banner.clone();
    }
    skin
}

fn banner_key(name: &str) -> Option<String> {
    banner_pick(name, |key| crate::assets_root().join(key).is_file())
}

fn banner_pick(name: &str, exists: impl Fn(&str) -> bool) -> Option<String> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let stem = name.trim_end_matches(".png");
    let direct = format!("banners/{stem}.png");
    if exists(&direct) {
        return Some(direct);
    }
    let (head, tail) = stem.rsplit_once('_')?;
    if tail.len() != 1 || !tail.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    let padded = format!("banners/{head}_0{tail}.png");
    match exists(&padded) {
        true => Some(padded),
        false => None,
    }
}

fn char_stem_by_id(id: i32, uncapped: bool) -> Option<String> {
    use std::collections::HashMap;
    use std::sync::OnceLock;

    static INDEX: OnceLock<HashMap<(i32, bool), String>> = OnceLock::new();
    let index = INDEX.get_or_init(|| {
        let mut map: HashMap<(i32, bool), String> = HashMap::new();
        let Ok(entries) = std::fs::read_dir(crate::assets_root().join("characters")) else {
            return map;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let Some(stem) = name.strip_suffix(".png") else {
                continue;
            };
            let (base, variant) = match stem.strip_suffix('u') {
                Some(base) => (base, true),
                None => (stem, false),
            };
            let Some((_, tail)) = base.rsplit_once('_') else {
                continue;
            };
            let Ok(found) = tail.parse::<i32>() else {
                continue;
            };
            map.entry((found, variant))
                .or_insert_with(|| stem.to_string());
        }
        map
    });
    index
        .get(&(id, uncapped))
        .or_else(|| index.get(&(id, false)))
        .cloned()
}

fn char_stem(stem: &str, skin: &dto::SkinFields) -> String {
    if !skin.char_uncapped || skin.char_override {
        return stem.to_string();
    }
    let uncapped = format!("{stem}u");
    let icon = format!("char_icons/{uncapped}_icon.png");
    if crate::assets_root().join(icon).is_file() {
        uncapped
    } else {
        stem.to_string()
    }
}

pub fn select_player_of(identity: &dto::PlayerIdentity, prefs: &Prefs) -> SelectPlayer {
    let badge = badge_of(identity, prefs);
    SelectPlayer {
        name: badge.name,
        ptt: badge.ptt.unwrap_or(0.0),
        rating_frame: badge.frame.unwrap_or(0),
        rating_accent: identity.accent().or(badge.frame).unwrap_or(0),
        global_rank: badge.global_rank,
        banner: badge.banner,
        avatar: badge.avatar,
    }
}

fn star_of(frame: i32) -> u8 {
    match frame {
        value if value >= 6 => 6,
        value if value >= 5 => 5,
        value if value >= 4 => 4,
        value if value >= 3 => 3,
        value if value >= 2 => 2,
        value if value >= 1 => 1,
        _ => 0,
    }
}

pub fn song_of(song: &dto::Song) -> Song {
    let mut charts: Vec<Chart> = song.charts.iter().filter_map(chart_of).collect();
    charts.sort_by_key(|chart| diff_order_weight(chart.difficulty));
    Song {
        id: song.song_id.clone(),
        title: song.title.clone(),
        artist: song.artist.clone(),
        bpm: song.bpm.parse::<f64>().unwrap_or(0.0),
        pack_id: song.set.clone(),
        pack_name: song.pack_name.clone(),
        pack_name_zh: song.pack_name_zh.clone(),
        pack_aliases: song.pack_aliases.clone(),
        aliases: song.aliases.clone(),
        charts,
    }
}

pub fn chart_of(chart: &dto::Chart) -> Option<Chart> {
    let difficulty = chart
        .difficulty_kind()
        .or_else(|| Difficulty::from_upstream(i64::from(chart.rating_class)))?;
    Some(
        Chart::new(
            difficulty,
            chart.rating_display.clone(),
            chart.constant.unwrap_or(0.0),
        )
        .with_designer(chart.chart_designer.clone())
        .with_note(i64::from(chart.note.unwrap_or(0)))
        .with_assets(
            chart.chart_url.clone(),
            chart.ill_url.clone(),
            chart.audio_url.clone(),
        ),
    )
}

pub fn jacket_key(song_id: &str) -> String {
    format!("jackets/{song_id}.jpg")
}

fn jacket_of(song_id: &str) -> Option<String> {
    (!song_id.trim().is_empty()).then(|| jacket_key(song_id))
}

pub fn jacket_thumb_key(song_id: &str) -> String {
    format!("jacket_thumbs/{song_id}.jpg")
}

pub fn thumb_of_chart(song_id: &str, difficulty: Difficulty) -> Option<String> {
    if song_id.trim().is_empty() {
        return None;
    }
    let suffix = crate::cards::song::difficulty_suffix(difficulty);
    let variant = format!("jacket_thumbs/{song_id}_{suffix}.jpg");
    if crate::assets_root().join(&variant).is_file() {
        return Some(variant);
    }
    Some(format!("jacket_thumbs/{song_id}.jpg"))
}

pub fn jacket_of_chart(song_id: &str, difficulty: Difficulty) -> Option<String> {
    if song_id.trim().is_empty() {
        return None;
    }
    let suffix = crate::cards::song::difficulty_suffix(difficulty);
    let variant = format!("jackets/{song_id}_{suffix}.jpg");
    if crate::assets_root().join(&variant).is_file() {
        return Some(variant);
    }
    Some(format!("jackets/{song_id}.jpg"))
}

fn jacket_thumb_of(song_id: &str) -> Option<String> {
    (!song_id.trim().is_empty()).then(|| jacket_thumb_key(song_id))
}

pub struct ResolvedSong {
    pub matched: SongMatch,
    pub songs: Vec<dto::Song>,
}

impl ResolvedSong {
    pub fn best(&self) -> Option<&dto::Song> {
        self.songs
            .iter()
            .find(|song| song.song_id == self.matched.best.song.id)
    }
}

pub async fn resolve_song(
    backend: &dto::ArcBackendNext,
    region: Region,
    query: &str,
) -> Result<Option<ResolvedSong>, dto::BackendError> {
    find_song(backend, region, query).await
}

pub async fn resolve_song_full(
    backend: &dto::ArcBackendNext,
    region: Region,
    query: &str,
    full: &str,
) -> Result<Option<ResolvedSong>, dto::BackendError> {
    if let Some(found) = find_song(backend, region, query).await? {
        return Ok(Some(found));
    }
    let full = full.trim();
    if full.is_empty() || full.eq_ignore_ascii_case(query.trim()) {
        return Ok(None);
    }
    find_song(backend, region, full).await
}

async fn find_song(
    backend: &dto::ArcBackendNext,
    region: Region,
    query: &str,
) -> Result<Option<ResolvedSong>, dto::BackendError> {
    let search = backend
        .songs(
            region,
            dto::SongQuery::new(query).with_limit(dto::DEFAULT_SEARCH_LIMIT),
        )
        .await?;
    let hits: Vec<crate::domain::SongHit> = search
        .items
        .iter()
        .map(|hit| crate::domain::SongHit {
            song: song_of(&hit.song),
            relevance: hit.score,
            matched_by: crate::domain::MatchKind::parse(hit.matched.as_str()),
        })
        .collect();
    let songs: Vec<dto::Song> = search.items.into_iter().map(|hit| hit.song).collect();
    let Some(matched) = crate::domain::resolve_hits(hits, search.total.max(0) as usize, query)
    else {
        return Ok(None);
    };
    Ok(Some(ResolvedSong { matched, songs }))
}

pub fn theme_of(prefs: &Prefs) -> Theme {
    Theme::by_name(&prefs.theme)
}

pub fn identity_input(
    region: Region,
    cn: bool,
    identity: &dto::PlayerIdentity,
    prefs: &Prefs,
    board: Option<crate::domain::BoardKind>,
) -> B50Input {
    let badge = badge_of(identity, prefs);
    let mut input = B50Input::new(badge, theme_of(prefs)).with_cn(cn || region.is_cn());
    if let Some(ptt) = identity.ptt() {
        input = input.with_ptt(ptt);
    }
    if let Some(frame) = frame_of_identity(identity) {
        input = input.with_rank_frame(frame);
    }
    if let Some(kind) = board {
        input = input.with_board(kind);
    }
    input
}

pub fn b50_input(
    region: Region,
    cn: bool,
    data: &dto::B50,
    prefs: &Prefs,
    board: Option<crate::domain::BoardKind>,
) -> B50Input {
    let mut input = identity_input(region, cn, &data.identity, prefs, board)
        .with_best(board_rows(&data.records))
        .with_averages(Some(data.best10), Some(data.best50), 0)
        .with_max_ptt(data.max_ptt);
    if let Some(at) = data.meta.cached_at {
        input = input.with_generated_at(crate::commands::local_time(at));
    }
    input
}

pub fn score_input(
    record: &dto::ScoreRecord,
    identity: &dto::PlayerIdentity,
    prefs: &Prefs,
    theme: Theme,
    level: &str,
) -> ScoreInput {
    let difficulty = difficulty_of(record);
    let grade = grade_of(record);
    let score = i64::from(record.score);
    let rating = if record.rating > 0.0 {
        record.rating
    } else {
        crate::domain::ptt::single_rating(record.constant, score)
    };
    let badge = badge_of(identity, prefs);
    let frame = badge.frame;
    let mut input = ScoreInput::new(record.title.clone(), score)
        .with_badge(badge)
        .with_rating_frame(frame)
        .with_theme(theme)
        .with_song(record.song_id.clone())
        .with_artist(record.artist.clone())
        .with_difficulty(difficulty, record.constant)
        .with_level(level.trim().to_string())
        .with_judgement(
            i64::from(record.perfect),
            i64::from(record.far),
            i64::from(record.lost),
        )
        .with_grade(grade)
        .with_rating(rating);
    if record.shiny_perfect > 0 || record.normal_perfect > 0 {
        input = input.with_perfect(
            i64::from(record.shiny_perfect),
            i64::from(record.normal_perfect),
        );
    }
    if let Some(clear_type) = clear_of(record) {
        input = input.with_clear(clear_type);
    }
    if let Some(ptt) = identity.ptt() {
        input = input.with_ptt(ptt);
    }
    if let Some(jacket) = thumb_of_chart(&record.song_id, difficulty_of(record)) {
        input = input.with_jacket(jacket);
    }
    if let Some(at) = play_at(record.time_played) {
        input = input.with_play_at(at);
    }
    input
}

pub fn song_input(song: &dto::Song, theme: Theme) -> SongInput {
    let mut input = SongInput::new(&song_of(song)).with_theme(theme);
    let base_artist = song.artist.trim();
    let base_bpm = song.bpm.trim();
    let pick = song
        .charts
        .iter()
        .filter_map(|chart| chart.difficulty_kind())
        .filter(|difficulty| {
            song.artist_display(*difficulty) != base_artist
                || song.bpm_display(*difficulty) != base_bpm
        })
        .max_by_key(|difficulty| diff_order_weight(*difficulty));
    if song.release_ts > 0 {
        if let Some(at) = chrono::DateTime::from_timestamp(song.release_ts, 0) {
            input.release = at.format("%Y-%m-%d").to_string();
        }
    }
    if let Some(difficulty) = pick {
        input.artist = song.artist_display(difficulty);
        if song.bpm_display(difficulty) != base_bpm {
            input.bpm_label = Some(song.bpm_display(difficulty));
        }
    }
    input
}

pub fn song_boards(board: &dto::SongBoard, difficulty: Difficulty) -> Vec<SongBoard> {
    let rows: Vec<SongBoardRow> = board
        .rows
        .iter()
        .map(|row| SongBoardRow {
            name: row.name.clone(),
            avatar: row
                .character
                .and_then(|id| char_stem_by_id(id, row.char_uncapped.unwrap_or(false)))
                .map(|stem| format!("char_icons/{stem}_icon.png")),
            grade: row.grade.clone(),
            score: i64::from(row.score),
            rating: row.rating.unwrap_or_default(),
            has_rating: row.rating.unwrap_or_default() > 0.0,
            rating_frame: row.rating_frame,
            rating_accent: row.rating_accent,
            perfect: row.perfect,
            normal_perfect: row.normal_perfect,
            far: row.far,
            lost: row.lost,
            judgement: row.has_judgement.unwrap_or(false),
        })
        .collect();
    if rows.is_empty() {
        return Vec::new();
    }
    vec![SongBoard::new(difficulty, rows)]
}

pub fn search_input(query: &str, items: &[dto::SongHit], theme: Theme, limit: usize) -> SearchInput {
    let limit = limit.max(1);
    let hits: Vec<SearchHit> = items
        .iter()
        .take(limit)
        .map(|hit| SearchHit::of(&song_of(&hit.song), jacket_of(&hit.song.song_id)))
        .collect();
    let truncated = items.len().saturating_sub(hits.len());
    SearchInput::new(query, hits)
        .with_theme(theme)
        .with_truncated(truncated)
}

pub fn select_input(
    region: Region,
    song: &dto::Song,
    difficulty: Difficulty,
    theme: Theme,
    player: Option<SelectPlayer>,
    rows: Vec<SelectRow>,
    score: Option<i64>,
) -> SelectInput {
    let root = if region.is_cn() { "/ca" } else { "/a" };
    let mut input = SelectInput::new(&song_of(song), difficulty)
        .with_theme(theme)
        .with_score(score)
        .with_retry_hint(format!("再来一次：{root} rand"));
    if !rows.is_empty() {
        input = input.with_rows(rows);
    }
    if let Some(player) = player {
        input = input.with_player(player);
    }
    input
}

pub fn me_score_of(record: &dto::ScoreRecord) -> MeScore {
    let score = i64::from(record.score);
    let mut out = MeScore::new(record.title.clone(), difficulty_of(record), score)
        .with_constant(record.constant);
    let rating = if record.rating > 0.0 {
        record.rating
    } else {
        crate::domain::ptt::single_rating(record.constant, score)
    };
    if rating > 0.0 {
        out = out.with_ptt(rating);
    }
    out = out.with_grade(grade_of(record));
    if let Some(jacket) = thumb_of_chart(&record.song_id, difficulty_of(record)) {
        out = out.with_jacket(jacket);
    }
    out
}

pub struct MeArgs<'a> {
    pub user_id: &'a str,
    pub region: Region,
    pub joined: &'a str,
    pub updated: &'a str,
    pub best_average: f64,
    pub best: Vec<MeScore>,
    pub rating_frame: Option<i32>,
    pub rating_accent: Option<i32>,
}

pub fn me_input(badge: PlayerBadge, theme: Theme, args: MeArgs<'_>) -> MeInput {
    let MeArgs {
        user_id,
        region,
        joined,
        updated,
        best_average,
        best,
        rating_frame,
        rating_accent,
    } = args;
    let mut input = MeInput::new(badge)
        .with_title("玩家档案")
        .with_theme(theme)
        .with_user(user_id, region)
        .with_joined(joined)
        .with_updated(updated)
        .with_best_average(best_average)
        .with_best(best);
    if let (Some(frame), Some(accent)) = (rating_frame, rating_accent) {
        if frame >= 0 {
            input = input.with_rating(frame, accent);
        }
    }
    input
}

pub fn probe_of(song: Option<&dto::Song>, record: &dto::ScoreRecord) -> ChartProbe {
    let difficulty = difficulty_of(record);
    let chart = song.and_then(|song| song.chart(difficulty));
    ChartProbe {
        title: record.title.clone(),
        difficulty,
        level: chart
            .map(|chart| chart.rating_display.clone())
            .unwrap_or_else(|| record.difficulty_label.clone()),
        constant: record.constant,
        has_constant: record.constant > 0.0,
        bpm: song
            .map(|song| song.bpm.parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0),
        note: i64::from(record.object_amount).max(
            chart
                .and_then(|chart| chart.note)
                .map(i64::from)
                .unwrap_or(0),
        ),
        designer: chart
            .map(|chart| chart.chart_designer.clone())
            .unwrap_or_else(|| record.chart_designer.clone()),
        pack_id: song.map(|song| song.set.clone()).unwrap_or_default(),
        pack_name: song.map(|song| song.pack_name.clone()).unwrap_or_default(),
        pack_name_zh: song
            .map(|song| song.pack_name_zh.clone())
            .unwrap_or_default(),
        pack_aliases: song
            .map(|song| song.pack_aliases.clone())
            .unwrap_or_default(),
    }
}

pub fn record_loss(record: &dto::ScoreRecord) -> Option<f64> {
    record.lose_score
}

pub fn probe_of_chart(
    song: &dto::Song,
    chart: &dto::Chart,
    difficulty: Difficulty,
) -> ChartProbe {
    let constant = chart.constant.unwrap_or(0.0);
    ChartProbe {
        title: song.title.clone(),
        difficulty,
        level: chart.rating_display.clone(),
        constant,
        has_constant: constant > 0.0,
        bpm: song.bpm.parse::<f64>().unwrap_or(0.0),
        note: chart.note.map(i64::from).unwrap_or(0),
        designer: chart.chart_designer.clone(),
        pack_id: song.set.clone(),
        pack_name: song.pack_name.clone(),
        pack_name_zh: song.pack_name_zh.clone(),
        pack_aliases: song.pack_aliases.clone(),
    }
}

pub fn list_entry_of(record: &dto::ScoreRecord, level: &str) -> ListEntry {
    let mut entry = ListEntry::new(
        record.song_id.clone(),
        record.title.clone(),
        difficulty_of(record),
        record.constant,
    )
    .with_artist(record.artist.clone())
    .with_score(i64::from(record.score));
    if let Some(loss) = record_loss(record) {
        entry = entry.with_judgement(
            i64::from(record.perfect),
            i64::from(record.shiny_perfect),
            i64::from(record.far),
            i64::from(record.lost),
        );
        entry = entry.with_loss(loss);
    }
    if !level.is_empty() {
        entry = entry.with_level(level);
    }
    entry = entry.with_grade(grade_of(record));
    if let Some(clear_type) = clear_of(record) {
        entry = entry.with_clear(clear_type);
    }
    if let Some(jacket) = thumb_of_chart(&record.song_id, difficulty_of(record)) {
        entry = entry.with_jacket(jacket);
    }
    entry
}

pub fn list_input(
    badge: PlayerBadge,
    theme: Theme,
    title: &str,
    subtitle: &str,
    note: &str,
    entries: Vec<ListEntry>,
) -> ListInput {
    ListInput::new(badge)
        .with_title(title)
        .with_subtitle(subtitle)
        .with_theme(theme)
        .with_note(note)
        .with_entries(entries)
}

pub fn table_song_of(song: &dto::Song) -> TableSong {
    let mut cells: Vec<TableCell> = Vec::with_capacity(TABLE_CELL_ORDER.len());
    for slot in TABLE_CELL_ORDER {
        let found = song
            .charts
            .iter()
            .find(|chart| chart.rating_class == i32::from(slot.as_upstream()));
        let cell = match found {
            Some(chart) => {
                let real = chart.difficulty_kind().unwrap_or(slot);
                let constant = chart.constant.unwrap_or(0.0);
                match constant > 0.0 {
                    true => TableCell::new(slot, constant).with_real(real),
                    false => TableCell::unknown(slot).with_real(real),
                }
            }
            None => TableCell::unknown(slot),
        };
        cells.push(cell);
    }
    let mut out =
        TableSong::new(song.song_id.clone(), song.title.clone(), cells).with_artist(song.artist.clone());
    if let Some(jacket) = jacket_thumb_of(&song.song_id) {
        out = out.with_jacket(jacket);
    }
    out
}

pub struct TableHead<'a> {
    pub title: &'a str,
    pub note: &'a str,
}

pub fn table_input(
    badge: PlayerBadge,
    theme: Theme,
    head: TableHead<'_>,
    songs: &[dto::Song],
    limit: usize,
    mode: TableMode,
    filter: Option<&crate::domain::Filter>,
) -> TableInput {
    let library: usize = songs.iter().map(|song| song.charts.len()).sum();
    if mode == TableMode::Songs {
        let rows: Vec<TableSong> = songs.iter().map(table_song_of).collect();
        let mut sorted = table_songs_sorted(&rows);
        sorted.truncate(limit.max(1));
        return TableInput::new(badge)
            .with_title(head.title)
            .with_theme(theme)
            .with_note(head.note)
            .with_library(library)
            .with_mode(TableMode::Songs)
            .with_songs(sorted);
    }
    let mut rows: Vec<TableChart> = Vec::new();
    for song in songs {
        for chart in &song.charts {
            let Some(difficulty) = chart.difficulty_kind() else {
                continue;
            };
            let constant = chart.constant.unwrap_or(0.0);
            if let Some(filter) = filter {
                if !filter.keeps_chart(difficulty, constant) {
                    continue;
                }
            }
            let mut entry = TableChart::new(
                song.song_id.clone(),
                song.title.clone(),
                difficulty,
                constant,
            );
            if let Some(jacket) = thumb_of_chart(&song.song_id, difficulty) {
                entry = entry.with_jacket(jacket);
            }
            rows.push(entry);
        }
    }
    rows.sort_by(|left, right| {
        table_band_key(right.constant)
            .cmp(&table_band_key(left.constant))
            .then_with(|| {
                diff_order_weight(left.difficulty).cmp(&diff_order_weight(right.difficulty))
            })
            .then_with(|| left.title.to_lowercase().cmp(&right.title.to_lowercase()))
            .then_with(|| left.song_id.cmp(&right.song_id))
    });
    rows.truncate(limit.max(1));
    let mut sheets: Vec<TableSheet> = Vec::new();
    for chart in rows {
        let band = table_band_label(chart.constant);
        match sheets.last_mut() {
            Some(sheet) if sheet.band == band => sheet.charts.push(chart),
            _ => sheets.push(TableSheet::new(band).with_chart(chart)),
        }
    }
    TableInput::new(badge)
        .with_title(head.title)
        .with_theme(theme)
        .with_note(head.note)
        .with_library(library)
        .with_sheets(sheets)
}

pub fn ahv_record_of(record: &dto::ScoreRecord) -> AhvRecord {
    let mut out = AhvRecord::new(
        record.song_id.clone(),
        record.title.clone(),
        difficulty_of(record),
        record.constant,
    )
    .with_score(i64::from(record.score))
    .with_pure(
        i64::from(record.perfect),
        i64::from(record.shiny_perfect),
    );
    if record.judged() {
        out = out.with_judgement(i64::from(record.far), i64::from(record.lost));
    }
    if let Some(clear_type) = clear_of(record) {
        out = out.with_clear(clear_type);
    }
    out = out.with_grade(achievement_of(record));
    if let Some(jacket) = jacket_thumb_of(&record.song_id) {
        out = out.with_jacket(jacket);
    }
    let score = i64::from(record.score);
    let rating = if record.rating > 0.0 {
        record.rating
    } else {
        crate::domain::ptt::single_rating(record.constant, score)
    };
    if rating > 0.0 {
        out = out.with_rating(rating);
    }
    if let Some(loss) = record_loss(record) {
        out = out.with_loss(loss);
    }
    out
}

pub fn ahv_record_of_chart(
    song: &dto::Song,
    chart: &dto::Chart,
    difficulty: Difficulty,
) -> AhvRecord {
    let constant = chart.constant.unwrap_or(0.0);
    let mut out = AhvRecord::new(song.song_id.clone(), song.title.clone(), difficulty, constant);
    if let Some(jacket) = thumb_of_chart(&song.song_id, difficulty) {
        out = out.with_jacket(jacket);
    }
    if let Some(max_lose) = chart.max_lose.filter(|value| *value > 0.0) {
        out = out.with_loss(max_lose);
    }
    out
}

pub fn ahv_input(
    badge: PlayerBadge,
    theme: Theme,
    title: &str,
    note: &str,
    records: Vec<AhvRecord>,
    generated_at: &str,
) -> AhvInput {
    AhvInput::new(badge)
        .with_title(title)
        .with_theme(theme)
        .with_note(note)
        .with_records(records)
        .with_generated_at(generated_at)
}

pub fn stats_band_of(band: &dto::StatsBand) -> StatsBand {
    let name = band
        .difficulty_kind()
        .map(|difficulty| difficulty.as_str().to_string())
        .unwrap_or_else(|| band.difficulty.to_string());
    StatsBand::new(name)
        .with_counts(band.total as i32, band.played as i32)
        .with_clear(
            band.cleared as i32,
            band.full_combo as i32,
            band.pm as i32,
        )
        .with_avg(band.avg_score)
}

pub fn stats_const_of(item: &dto::StatsConst) -> StatsConst {
    StatsConst::new(item.constant, item.total as i32, item.played as i32)
}

pub fn stats_grade_of(item: &dto::StatsGrade) -> StatsGrade {
    StatsGrade::new(item.grade.clone(), item.count as i32)
}

pub fn stats_row_of(item: &dto::StatsShort) -> StatsRow {
    let difficulty = Difficulty::from_upstream(i64::from(item.difficulty))
        .map(|difficulty| difficulty.as_str().to_string())
        .unwrap_or_else(|| item.difficulty.to_string());
    let mut row = StatsRow::new(item.song_id.clone(), item.title.clone(), difficulty)
        .with_score(i64::from(item.score))
        .with_grade(item.grade.clone());
    if let Some(constant) = item.constant.filter(|value| *value > 0.0) {
        row = row.with_constant(constant);
    }
    if !item.next_rank.is_empty() {
        row = row.with_need(item.next_rank.clone(), i64::from(item.next_need));
    }
    row
}

pub struct StatsArgs<'a> {
    pub title: &'a str,
    pub note: &'a str,
    pub data: &'a dto::PlayerStats,
    pub bands: &'a [dto::StatsBand],
    pub consts: &'a [dto::StatsConst],
    pub short: &'a [dto::StatsShort],
    pub short_title: &'a str,
    pub short_note: &'a str,
}

pub fn stats_input(badge: PlayerBadge, args: StatsArgs<'_>) -> StatsInput {
    let StatsArgs {
        title,
        note,
        data,
        bands,
        consts,
        short,
        short_title,
        short_note,
    } = args;
    StatsInput::new(badge)
        .with_title(title)
        .with_note(note)
        .with_bands(bands.iter().map(stats_band_of).collect())
        .with_consts(consts.iter().map(stats_const_of).collect())
        .with_grades(data.grades.iter().map(stats_grade_of).collect())
        .with_short_title(short_title)
        .with_short(short.iter().map(stats_row_of).collect())
        .with_short_note(short_note)
}

fn domain_hint(hint: &dto::SuggestHint) -> Option<DomainHint> {
    let difficulty = hint.difficulty_kind()?;
    Some(DomainHint {
        song_id: hint.song_id.clone(),
        title: hint.title.clone(),
        difficulty,
        constant: hint.constant,
        note: hint.note.max(0),
        score: i64::from(hint.score),
        has_score: hint.has_score,
        far: i64::from(hint.far),
        lost: i64::from(hint.lost),
        has_judgement: hint
            .has_judgement
            .unwrap_or(hint.far > 0 || hint.lost > 0),
        rating: hint.rating,
        target_rating: hint.target_rating,
        delta: hint.delta.round() as i64,
        target: hint.target.round() as i64,
        rank: hint.rank.max(0) as usize,
        target_rank: hint.target_rank.max(0) as usize,
        pick: hint.pick,
        notes: NotePlan {
            known: hint.notes.known,
            far_need: hint.notes.far_need,
            lost_need: hint.notes.lost_need,
            far_room: hint.notes.far_room,
            lost_room: hint.notes.lost_room,
            far_use: hint.notes.far_use,
            lost_use: hint.notes.lost_use,
            short: hint.notes.short,
        },
    })
}

fn level_floor_of(constant: f64) -> Option<i64> {
    if constant <= 0.0 {
        return None;
    }
    Some(constant.floor() as i64)
}

pub fn suggest_board_of(data: &dto::PlayerSuggest) -> SuggestBoard {
    let hints: Vec<DomainHint> = data.hints.iter().filter_map(domain_hint).collect();
    let total = hints.len();
    let mut picks: Vec<DomainHint> = hints.iter().filter(|hint| hint.pick).cloned().collect();
    let rest: Vec<DomainHint> = hints.into_iter().filter(|hint| !hint.pick).collect();
    let pick_count = picks.len();
    picks.sort_by_key(|left| left.target_rank);
    let mut groups: Vec<SuggestGroup> = Vec::new();
    if !picks.is_empty() {
        groups.push(SuggestGroup {
            label: String::new(),
            title: Some(PICK_TITLE),
            hints: picks,
        });
    }
    let mut buckets: BTreeMap<Option<i64>, Vec<DomainHint>> = BTreeMap::new();
    for hint in rest {
        buckets.entry(level_floor_of(hint.constant)).or_default().push(hint);
    }
    let mut keys: Vec<i64> = buckets.keys().filter_map(|key| *key).collect();
    keys.sort_unstable_by(|left, right| right.cmp(left));
    let has_unknown = buckets.contains_key(&None);
    for key in keys {
        if let Some(group) = buckets.remove(&Some(key)) {
            groups.push(SuggestGroup {
                label: key.to_string(),
                title: None,
                hints: group,
            });
        }
    }
    if has_unknown {
        if let Some(group) = buckets.remove(&None) {
            groups.push(SuggestGroup {
                label: String::new(),
                title: None,
                hints: group,
            });
        }
    }
    SuggestBoard {
        groups,
        hint_count: total,
        pick_count,
        pool: data.pool.max(0) as usize,
        pushable: data.pushable.max(0) as usize,
    }
}

pub fn suggest_input(
    badge: PlayerBadge,
    theme: Theme,
    title: &str,
    note: &str,
    data: &dto::PlayerSuggest,
) -> SuggestInput {
    let mut input = SuggestInput::new(badge, suggest_board_of(data))
        .with_title(title)
        .with_theme(theme)
        .with_note(note);
    if data.enough {
        input = input.with_ptt(data.displayed, data.target);
    }
    input
}

fn chap_pack_of(pack: &dto::ChapterPack, section: &str) -> ChapPack {
    ChapPack::new(
        pack.pack_id.clone(),
        pack.name.clone(),
        pack.total.max(0) as usize,
    )
    .with_owned(pack.played.max(0) as usize)
    .with_cleared(pack.cleared.max(0) as usize)
    .with_full_combo(pack.full_combo.max(0) as usize)
    .with_pure_memory(pack.pm.max(0) as usize)
    .with_section(section)
    .with_average(pack.avg_score, pack.avg_ptt)
}

fn chap_song_of(chart: &dto::ChapterChart) -> ChapSong {
    let difficulty = chart.difficulty_kind().unwrap_or(Difficulty::Future);
    let mut song = ChapSong::new(chart.song_id.clone(), chart.title.clone(), difficulty)
        .with_jacket(
            jacket_of_chart(&chart.song_id, difficulty)
                .unwrap_or_else(|| jacket_key(&chart.song_id)),
        )
        .with_score(i64::from(chart.score))
        .with_played(chart.played)
        .with_judgement(
            i64::from(chart.perfect),
            i64::from(chart.shiny_perfect),
            i64::from(chart.near),
            i64::from(chart.miss),
        );
    if let Some(constant) = chart.constant.filter(|value| *value > 0.0) {
        song = song.with_constant(constant);
    }
    if chart.played {
        song = song.with_grade(Grade::from_score(i64::from(chart.score)));
    }
    if chart.ptt > 0.0 {
        song = song.with_ptt(chart.ptt);
    }
    song
}

pub fn chap_input(
    badge: PlayerBadge,
    theme: Theme,
    title: &str,
    subtitle: &str,
    note: &str,
    data: &dto::PlayerChapters,
) -> ChapInput {
    let mut packs: Vec<ChapPack> = Vec::new();
    if let Some(detail) = data.detail.as_ref() {
        let mut pack = chap_pack_of(&detail.pack, "");
        pack = pack.with_charts(detail.charts.iter().map(chap_song_of).collect());
        packs.push(pack);
    } else {
        for section in &data.sections {
            packs.extend(
                section
                    .packs
                    .iter()
                    .map(|pack| chap_pack_of(pack, section.name.as_str())),
            );
        }
    }
    let mut input = ChapInput::new(badge)
        .with_title(title)
        .with_subtitle(subtitle)
        .with_theme(theme)
        .with_note(note)
        .with_expand(ChapExpand::All)
        .with_packs(packs)
        .with_totals(
            data.totals.played.max(0) as usize,
            data.totals.total.max(0) as usize,
        );
    if data.detail.is_some() {
        input = input.with_focus(0);
    }
    input
}

pub fn trend_input(
    badge: PlayerBadge,
    theme: Theme,
    title: &str,
    note: &str,
    points: Vec<TrendPoint>,
) -> TrendInput {
    TrendInput::new(badge)
        .with_title(title)
        .with_subtitle(note)
        .with_theme(theme)
        .with_note(note)
        .with_points(points)
}

fn his_record_of(row: &dto::SongHistoryRow) -> HisRecord {
    let label = match (row.has_old, row.at) {
        (false, Some(at)) => crate::commands::local_time(at),
        (false, None) => String::from("初始记录"),
        (true, Some(at)) => crate::commands::local_time(at),
        (true, None) => String::from("时间未知"),
    };
    let new_score = i64::from(row.new_score);
    let delta = if row.has_old {
        new_score - i64::from(row.old_score)
    } else {
        0
    };
    let normal_perfect = (i64::from(row.new_perfect) - i64::from(row.new_shiny_perfect)).max(0);
    let mut record = HisRecord::new(label, new_score, delta)
        .with_perfect(i64::from(row.new_perfect))
        .with_normal_perfect(normal_perfect)
        .with_judgement(i64::from(row.new_near), i64::from(row.new_miss))
        .with_grade(crate::domain::grade_of(new_score));
    let ptt = match row.new_rating.filter(|value| *value > 0.0) {
        Some(rating) => Some(rating),
        None => row
            .constant
            .filter(|value| *value > 0.0)
            .map(|constant| crate::domain::ptt::single_rating(constant, new_score))
            .filter(|value| *value > 0.0),
    };
    if let Some(ptt) = ptt {
        record = record.with_ptt(ptt);
    }
    if let Some(clear_type) = row.new_clear_type {
        record = record.with_clear(clear_type);
    }
    record
}

fn history_unjudged(row: &dto::SongHistoryRow) -> bool {
    row.new_perfect == 0 && row.new_near == 0 && row.new_miss == 0
}

fn history_baseline(row: &dto::SongHistoryRow) -> dto::SongHistoryRow {
    dto::SongHistoryRow {
        has_old: false,
        old_score: 0,
        old_perfect: 0,
        old_shiny_perfect: 0,
        old_near: 0,
        old_miss: 0,
        old_clear_type: None,
        old_rating: None,
        new_score: row.old_score,
        new_perfect: row.old_perfect,
        new_shiny_perfect: row.old_shiny_perfect,
        new_near: row.old_near,
        new_miss: row.old_miss,
        new_clear_type: row.old_clear_type,
        new_rating: row.old_rating,
        at: row.old_at,
        ..row.clone()
    }
}

fn merge_history(rows: &[dto::SongHistoryRow]) -> Vec<dto::SongHistoryRow> {
    let mut out: Vec<dto::SongHistoryRow> = Vec::with_capacity(rows.len());
    for row in rows {
        if let Some(prev) = out.last_mut() {
            let same_score = prev.new_score == row.new_score;
            let mixed = history_unjudged(prev) != history_unjudged(row);
            if same_score && mixed {
                let donor = match history_unjudged(row) {
                    true => prev.clone(),
                    false => row.clone(),
                };
                prev.new_perfect = donor.new_perfect;
                prev.new_shiny_perfect = donor.new_shiny_perfect;
                prev.new_near = donor.new_near;
                prev.new_miss = donor.new_miss;
                prev.new_clear_type = match (prev.new_clear_type, donor.new_clear_type) {
                    (Some(left), Some(right)) => Some(left.max(right)),
                    (left, right) => left.or(right),
                };
                if prev.new_rating.unwrap_or(0.0) <= 0.0 {
                    prev.new_rating = donor.new_rating;
                }
                if prev.lose_score.is_none() {
                    prev.lose_score = donor.lose_score;
                }
                if prev.at.is_none() {
                    prev.at = donor.at;
                }
                continue;
            }
        }
        out.push(row.clone());
    }
    if let Some(first) = out.first().cloned() {
        if first.has_old && first.old_score > 0 {
            out.insert(0, history_baseline(&first));
        }
    }
    out.reverse();
    out
}

pub fn his_input(
    badge: PlayerBadge,
    theme: Theme,
    song: &Song,
    sections: &[(Difficulty, Vec<dto::SongHistoryRow>)],
) -> HisInput {
    let sections: Vec<(Difficulty, Vec<dto::SongHistoryRow>)> = sections
        .iter()
        .map(|(difficulty, rows)| (*difficulty, merge_history(rows)))
        .collect();
    let total: usize = sections.iter().map(|(_, rows)| rows.len()).sum();
    let mut kept_rows: Vec<usize> = vec![0; sections.len()];
    if total <= HIS_MAX_ROWS {
        for (slot, (_, rows)) in kept_rows.iter_mut().zip(sections.iter()) {
            *slot = rows.len();
        }
    } else {
        let mut budget = HIS_MAX_ROWS;
        for (index, (_, rows)) in sections.iter().enumerate().rev() {
            if rows.len() <= budget {
                kept_rows[index] = rows.len();
                budget -= rows.len();
            }
        }
        if kept_rows.iter().all(|count| *count == 0) {
            if let (Some(count), Some((_, rows))) = (kept_rows.last_mut(), sections.last()) {
                *count = rows.len().min(HIS_MAX_ROWS);
            }
        }
    }
    let mut built: Vec<HisSection> = Vec::with_capacity(sections.len());
    let mut dropped: Vec<String> = Vec::new();
    let mut dropped_rows = 0usize;
    for (index, (difficulty, rows)) in sections.iter().enumerate() {
        let keep = kept_rows[index].min(rows.len());
        if keep == 0 {
            dropped.push(difficulty.as_str().to_string());
            dropped_rows += rows.len();
            continue;
        }
        let kept = &rows[rows.len() - keep..];
        let constant = song
            .charts
            .iter()
            .find(|chart| chart.difficulty == *difficulty)
            .map(|chart| chart.constant)
            .unwrap_or(0.0);
        let section = HisSection::new(*difficulty)
            .with_records(kept.iter().map(his_record_of).collect())
            .with_truncated(kept.len() < rows.len());
        built.push(match constant > 0.0 {
            true => section.with_constant(constant),
            false => section,
        });
    }
    let mut input = HisInput::new(song.id.clone(), song.title.clone())
        .with_badge(badge)
        .with_theme(theme)
        .with_jacket(jacket_key(&song.id))
        .with_sections(built);
    if !dropped.is_empty() {
        input = input.with_omitted(format!(
            "{} 档共 {dropped_rows} 条未画",
            dropped.join(" / ")
        ));
    }
    input
}

type RankCache = std::sync::Mutex<BTreeMap<(Region, i64), (Option<i32>, std::time::Instant)>>;

static RANKS: std::sync::LazyLock<RankCache> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(BTreeMap::new()));

const RANK_TTL: std::time::Duration = std::time::Duration::from_secs(600);

pub async fn world_rank(
    backend: &dto::ArcBackendNext,
    region: Region,
    uid: i64,
) -> Option<i32> {
    if let Ok(cache) = RANKS.lock() {
        if let Some((rank, at)) = cache.get(&(region, uid)) {
            if at.elapsed() < RANK_TTL {
                return *rank;
            }
        }
    }
    let rank = match backend.player(region, uid).await {
        Ok(Some(player)) => player.identity.global_rank,
        _ => None,
    };
    if let Ok(mut cache) = RANKS.lock() {
        cache.insert((region, uid), (rank, std::time::Instant::now()));
    }
    rank
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chapter_grid_skips_the_unfiled_bucket() {
        let mut data = dto::PlayerChapters::default();
        data.totals.packs = 1;
        data.totals.total = 18;
        data.sections = vec![dto::ChapterSection {
            id: String::from("mainstory"),
            name: String::from("主线"),
            packs: vec![dto::ChapterPack {
                pack_id: String::from("rei"),
                name: String::from("Luminous Sky"),
                section: String::from("mainstory"),
                total: 18,
                played: 18,
                ..dto::ChapterPack::default()
            }],
        }];
        data.unfiled = Some(dto::ChapterPack::default());
        let input = chap_input(
            crate::cards::widgets::PlayerBadge::new("测试"),
            crate::cards::theme::Theme::DEFAULT,
            "",
            "命中 1 个曲包",
            "共 1 个曲包 · 18 张谱面",
            &data,
        );
        assert_eq!(input.packs.len(), 1, "未归档的空包不该多占一格");
        assert_eq!(input.packs[0].section, "主线", "分区名跟着后端走");
        assert!(input.title.is_empty(), "网格版大标题留给刊头");
    }

    fn history_row(judged: bool, old_score: i32, at: i64) -> dto::SongHistoryRow {
        dto::SongHistoryRow {
            song_id: String::from("brandnewworld"),
            difficulty: 2,
            has_old: true,
            old_score,
            new_score: 9_759_204,
            new_perfect: match judged {
                true => 753,
                false => 0,
            },
            new_shiny_perfect: match judged {
                true => 628,
                false => 0,
            },
            new_near: match judged {
                true => 30,
                false => 0,
            },
            new_miss: match judged {
                true => 4,
                false => 0,
            },
            old_at: chrono::DateTime::from_timestamp(at.saturating_sub(3_600), 0),
            new_clear_type: Some(1),
            at: chrono::DateTime::from_timestamp(at, 0),
            ..dto::SongHistoryRow::default()
        }
    }

    #[test]
    fn song_card_merges_per_difficulty_credit() {
        let mut song = dto::Song {
            song_id: String::from("dropdead"),
            title: String::from("dropdead"),
            artist: String::from("Frums"),
            bpm: String::from("50"),
            ..dto::Song::default()
        };
        song.charts = vec![
            dto::Chart {
                rating_class: 2,
                difficulty: String::from("FTR"),
                ..dto::Chart::default()
            },
            dto::Chart {
                rating_class: 3,
                difficulty: String::from("BYD"),
                bpm: String::from("500"),
                artist: String::from("Frums (BYD)"),
                ..dto::Chart::default()
            },
        ];
        let input = song_input(&song, Theme::DEFAULT);
        assert_eq!(input.artist, "Frums / Frums (BYD) [BYD]");
        assert_eq!(input.bpm_label.as_deref(), Some("50 / 500 [BYD]"));
        let plain = song_input(
            &dto::Song {
                artist: String::from("Frums"),
                bpm: String::from("178"),
                ..dto::Song::default()
            },
            Theme::DEFAULT,
        );
        assert_eq!(plain.artist, "Frums");
        assert!(plain.bpm_label.is_none());
    }

    #[test]
    fn his_merges_unjudged_with_filled_row() {
        let rows = vec![
            history_row(false, 9_746_503, 1_760_000_000),
            history_row(true, 9_759_204, 1_760_000_600),
        ];
        let merged = merge_history(rows.as_slice());
        assert_eq!(merged.len(), 2);
        let latest = &merged[0];
        assert!(latest.has_old);
        assert_eq!(latest.old_score, 9_746_503);
        assert_eq!(latest.new_perfect, 753);
        assert_eq!(latest.new_shiny_perfect, 628);
        assert_eq!(latest.new_near, 30);
        assert_eq!(latest.new_miss, 4);
        assert_eq!(his_record_of(latest).delta, 12_701);
        let base = &merged[1];
        assert!(!base.has_old);
        assert_eq!(base.new_score, 9_746_503);
        assert_ne!(his_record_of(base).label, "初始记录");
        let mut legacy = history_row(false, 9_746_503, 1_760_000_000);
        legacy.has_old = false;
        legacy.old_at = None;
        legacy.at = None;
        assert_eq!(his_record_of(&legacy).label, "初始记录");
        assert_eq!(his_record_of(base).delta, 0);
        let alone = vec![history_row(true, 9_700_000, 1_760_000_000)];
        assert_eq!(merge_history(alone.as_slice()).len(), 2);
        let mut higher = history_row(true, 9_759_204, 1_760_000_600);
        higher.new_score = 9_800_000;
        let different = vec![history_row(false, 9_700_000, 1_760_000_000), higher];
        let merged = merge_history(different.as_slice());
        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].new_score, 9_800_000);
        assert_eq!(merged[1].new_score, 9_759_204);
        assert!(!merged[2].has_old);
        assert_eq!(merged[2].new_score, 9_700_000);
    }

    fn case_072(rating_class: i32, display: &str, constant: f64) -> dto::Chart {
        dto::Chart {
            rating_class,
            rating_display: display.to_string(),
            constant: Some(constant),
            ..dto::Chart::default()
        }
    }

    #[test]
    fn banner_pick_pads_single_digit_stage() {
        let padded_only = |key: &str| key == "banners/course_banner_06.png";
        assert_eq!(
            banner_pick("course_banner_6", padded_only).as_deref(),
            Some("banners/course_banner_06.png")
        );
        assert_eq!(
            banner_pick("course_banner_06.png", padded_only).as_deref(),
            Some("banners/course_banner_06.png")
        );
        assert_eq!(banner_pick("online_banner_2026_11", |_| false).as_deref(), None);
        assert_eq!(banner_pick("   ", |_| true).as_deref(), None);
        let empty = case_identity(Some(11.8), Some(4), dto::SkinFields::default());
        assert_eq!(
            badge_of(&empty, &crate::model::Prefs::default())
                .banner
                .as_deref(),
            Some(crate::cards::player::DEFAULT_BANNER)
        );
        let cn = case_identity(
            Some(11.8),
            Some(4),
            dto::SkinFields {
                character: Some(92),
                banner: "course_banner_6".to_string(),
                ..dto::SkinFields::default()
            },
        );
        assert_eq!(
            badge_of(&cn, &crate::model::Prefs::default())
                .banner
                .as_deref(),
            Some("banners/course_banner_06.png")
        );
    }

    fn case_identity(
        ptt: Option<f64>,
        frame: Option<i32>,
        skin: dto::SkinFields,
    ) -> dto::PlayerIdentity {
        dto::PlayerIdentity {
            name: String::from("Refrain"),
            current_ptt: ptt,
            rating_frame: frame,
            rating_accent: frame,
            skin,
            ..dto::PlayerIdentity::default()
        }
    }

    #[test]
    fn case_077() {
        let prefs = crate::model::Prefs::default();
        let skin = dto::SkinFields::default();
        //后端给了哪一档就用哪一档(0 也是合法档位)。
        assert_eq!(
            badge_of(&case_identity(Some(11.8), Some(4), skin.clone()), &prefs).frame,
            Some(4)
        );
        assert_eq!(
            badge_of(&case_identity(Some(3.2), Some(0), skin.clone()), &prefs).frame,
            Some(0)
        );
        assert_eq!(
            badge_of(&case_identity(Some(13.6), Some(8), skin.clone()), &prefs).frame,
            Some(8)
        );
        //没给档位但有 PTT: 按 PTT 现算, 绝不能因为缺档位就画成横杠。
        assert_eq!(
            badge_of(&case_identity(Some(13.6), None, skin.clone()), &prefs).frame,
            Some(i32::from(crate::domain::rating_frame_of(13.6)))
        );
        assert_eq!(
            badge_of(&case_identity(Some(11.8), Some(-1), skin.clone()), &prefs).frame,
            Some(i32::from(crate::domain::rating_frame_of(11.8)))
        );
        //PTT 未知: 档位与排名都必须缺省, 不能编一个第 0 档出来。
        let bare = badge_of(&case_identity(None, None, skin.clone()), &prefs);
        assert_eq!(bare.frame, None);
        assert_eq!(bare.ptt, None);
        assert_eq!(bare.global_rank, None);
        assert_eq!(
            badge_of(&case_identity(Some(0.0), None, skin), &prefs).frame,
            None
        );
    }

    fn case_073(id: &str, charts: Vec<dto::Chart>) -> dto::Song {
        dto::Song {
            song_id: id.to_string(),
            title: id.to_string(),
            charts,
            ..dto::Song::default()
        }
    }

    #[test]
    fn case_074() {
        let songs = vec![
            case_073(
                "s1",
                vec![
                    case_072(2, "9+", 9.6),
                    case_072(3, "9+", 9.7),
                    case_072(4, "9+", 9.8),
                    case_072(2, "10+", 10.4),
                ],
            ),
            case_073("s2", vec![case_072(2, "10", 9.7)]),
        ];
        let input = table_input(
            PlayerBadge::new("Afkeru"),
            Theme::DEFAULT,
            TableHead {
                title: "定数表",
                note: "",
            },
            &songs,
            200,
            TableMode::Charts,
            None,
        );
        let bands: Vec<&str> = input.sheets.iter().map(|sheet| sheet.band.as_str()).collect();
        assert_eq!(bands, vec!["10.4", "9.8", "9.7", "9.6"]);
        assert_eq!(
            input.sheets[2].charts.len(),
            2,
            "9.7 档要同时收下 9+ 与 10 两个等级的谱面"
        );
    }

    #[test]
    fn case_075() {
        let mut record = dto::ScoreRecord {
            score: 9_800_000,
            lose_score: Some(123.4),
            difficulty: 1,
            perfect: 900,
            shiny_perfect: 800,
            ..dto::ScoreRecord::default()
        };
        assert_eq!(
            record_loss(&record),
            Some(123.4),
            "后端给了失分就原样透传, 插件不再按难度或判定二次判断"
        );
        record.lose_score = Some(0.0);
        assert_eq!(record_loss(&record), Some(0.0), "0 也是后端给的数");
        record.lose_score = None;
        assert_eq!(record_loss(&record), None, "后端没给才没有");
    }

    #[test]
    fn case_076() {
        let song = case_073("s1", Vec::new());
        let mut chart = dto::Chart {
            constant: Some(12.0),
            ..dto::Chart::default()
        };
        assert_eq!(
            ahv_record_of_chart(&song, &chart, Difficulty::Future).loss,
            None,
            "没有成绩就没有失分, 不在这里按定数编一个上限"
        );
        chart.constant = Some(9.0);
        assert_eq!(
            ahv_record_of_chart(&song, &chart, Difficulty::Present).loss,
            None,
            "没有成绩就没有失分"
        );
    }
}

