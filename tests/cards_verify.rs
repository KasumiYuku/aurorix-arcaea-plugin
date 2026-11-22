#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;

use aurorix_arcaea_plugin::cards::theme::Theme;
use aurorix_arcaea_plugin::cards::widgets::PlayerBadge;
use aurorix_arcaea_plugin::cards::{
    b50, gallery, guess, harness, lists, score, search, select, song, stats,
};
use aurorix_arcaea_plugin::domain::board::BoardRow;
use aurorix_arcaea_plugin::domain::ptt;
use aurorix_arcaea_plugin::domain::song::Chart;
use aurorix_arcaea_plugin::domain::suggest::{
    NotePlan, SuggestBoard, SuggestGroup, SuggestHint, PICK_TITLE,
};
use aurorix_arcaea_plugin::{Difficulty, Grade, Region};
use chrono::{DateTime, Utc};
use serde::Deserialize;

const 夹具: &str = "/tmp/arcaea-verify/fixtures.json";
const VERIFY_DIR: &str = "/tmp/arcaea-verify/rust";

fn case_516(text: &str) -> Difficulty {
    text.parse().unwrap_or(Difficulty::Future)
}

fn case_517(text: &str) -> Option<Grade> {
    text.parse().ok()
}

fn case_518(text: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(text)
        .ok()
        .map(|value| value.with_timezone(&Utc))
}

fn case_519(text: &str) -> String {
    match case_518(text) {
        Some(value) => value.format("%Y-%m-%d %H:%M").to_string(),
        None => text.to_string(),
    }
}

fn case_520(text: &str) -> String {
    match case_518(text) {
        Some(value) => value.format("%m-%d %H:%M").to_string(),
        None => text.to_string(),
    }
}

fn case_521(text: &str) -> String {
    match case_518(text) {
        Some(value) => value.format("%m-%d").to_string(),
        None => text.to_string(),
    }
}

fn case_522(text: &str) -> String {
    match case_518(text) {
        Some(value) => value.format("%Y-%m-%d").to_string(),
        None => text.to_string(),
    }
}

fn case_523(text: &str) -> Option<i32> {
    match text {
        "PURE" => Some(3),
        "FULL" => Some(2),
        "NORMAL" | "EASY" => Some(1),
        "FAIL" => Some(0),
        _ => None,
    }
}

fn case_524(name: &str) -> String {
    format!("{name}.png")
}

fn case_525(image: &str) -> String {
    image.trim_end_matches(".png").to_string()
}

fn case_526(rows: Vec<BoardRow>) -> Vec<BoardRow> {
    rows
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoIcon {
    id: i32,
    known: bool,
    uncapped: bool,
    pref_char: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoFace {
    icon: GoIcon,
    banner: String,
    pref_banner: String,
    avatar: String,
    character: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoPlayer {
    name: String,
    friend_code: String,
    ptt: Option<f64>,
    rating_frame: Option<i32>,
    rating_accent: Option<i32>,
    date: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoStat {
    value: String,
    label: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoCell {
    label: String,
    value: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoScoreRow {
    rank: usize,
    song_id: String,
    title: String,
    diff: String,
    constant: f64,
    rating: f64,
    score: i64,
    pure: i64,
    shiny_perfect: i64,
    normal_perfect: i64,
    far: i64,
    lost: i64,
    clear: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoBoard {
    player: GoPlayer,
    face: GoFace,
    board: String,
    cn: bool,
    seed: i64,
    best10_avg: f64,
    best50_avg: f64,
    best_count: usize,
    max_potential: f64,
    anchor_upper: usize,
    top10: Vec<GoScoreRow>,
    best50: Vec<GoScoreRow>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoPlay {
    title: String,
    artist: String,
    chart_designer: String,
    song_id: String,
    diff: String,
    constant: Option<f64>,
    score: i64,
    grade: String,
    rating: f64,
    pure: i64,
    shiny_perfect: i64,
    normal_perfect: i64,
    far: i64,
    lost: i64,
    lose_score: f64,
    clear_type: i32,
    has_clear: bool,
    played_at: Option<String>,
    char_id: i32,
    theme: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoScorePage {
    player: GoPlayer,
    face: GoFace,
    page_title: String,
    has_ptt: bool,
    play: GoPlay,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoCover {
    song_id: String,
    diff: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoLevel {
    diff: String,
    level: String,
    constant: f64,
    has_constant: bool,
    note: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoBoardRow {
    rank: i32,
    name: String,
    icon: GoIcon,
    grade: String,
    score: i64,
    rating: f64,
    has_rating: bool,
    rating_frame: i32,
    rating_accent: i32,
    perfect: i32,
    normal_perfect: i32,
    far: i32,
    lost: i32,
    has_judgement: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoSongBoard {
    diff: String,
    rows: Vec<GoBoardRow>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoSongPage {
    player: GoPlayer,
    face: GoFace,
    head: String,
    note: String,
    stats: Vec<GoStat>,
    cover: GoCover,
    cells: Vec<GoCell>,
    alias: String,
    levels: Vec<GoLevel>,
    boards: Vec<GoSongBoard>,
    board_note: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoSelectRow {
    title: String,
    song_id: String,
    artist: String,
    diff: String,
    level: String,
    constant: f64,
    has_constant: bool,
    selected: bool,
    clear: String,
    grade: String,
    bpm: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoBar {
    name: String,
    ptt: f64,
    has_ptt: bool,
    rating_frame: i32,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoSelect {
    diff: String,
    levels: Vec<String>,
    diff_labels: Vec<String>,
    pack: String,
    pack_count: usize,
    clear_count: usize,
    pure_count: usize,
    total: usize,
    score: Option<i64>,
    bar: GoBar,
    rows: Vec<GoSelectRow>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoRandPage {
    face: GoFace,
    select: GoSelect,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoSearchLevel {
    diff: String,
    constant: Option<f64>,
    constant_text: Option<String>,
    active: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoSearchRow {
    song_id: String,
    title: String,
    artist: String,
    group: Option<String>,
    levels: Vec<GoSearchLevel>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoSearchPage {
    head: String,
    note: String,
    foot: String,
    rows: Vec<GoSearchRow>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoGuessPage {
    song_id: String,
    variant: String,
    level: f64,
    zoom: f64,
    interference: f64,
    seed: i64,
    code: String,
    #[serde(rename = "final")]
    revealed: bool,
    frame_zoom: f64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoListRow {
    song_id: String,
    title: String,
    composer: String,
    group: String,
    rank: usize,
    diff: String,
    constant: f64,
    known: bool,
    score: i64,
    grade: String,
    loss: f64,
    need: f64,
    pure: i64,
    normal_perfect: i64,
    far: i64,
    lost: i64,
    has_judgement: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoListPage {
    player: GoPlayer,
    face: GoFace,
    head: String,
    note: String,
    backdrop: String,
    rows: Vec<GoListRow>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoTableCell {
    diff: String,
    constant: f64,
    known: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoTableRow {
    song_id: String,
    title: String,
    artist: String,
    diff: String,
    constant: f64,
    cells: Vec<GoTableCell>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoTableSection {
    label: String,
    rows: Vec<GoTableRow>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoTableMeta {
    library: usize,
    matched: usize,
    charts: usize,
    empty: Option<Vec<String>>,
    cond: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoTablePage {
    player: GoPlayer,
    face: GoFace,
    head: String,
    note: String,
    backdrop: String,
    mode: String,
    meta: GoTableMeta,
    sections: Vec<GoTableSection>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoAhvCard {
    song_id: String,
    title: String,
    diff: String,
    constant: f64,
    played: bool,
    score: i64,
    grade: String,
    perfect: i64,
    shiny_perfect: i64,
    far: i64,
    lost: i64,
    has_judgement: bool,
    clear_type: Option<i32>,
    rating: Option<f64>,
    loss: Option<f64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoAhvSection {
    label: String,
    cards: Vec<GoAhvCard>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoAhvMeta {
    matched: usize,
    played: usize,
    loss: f64,
    cond: Option<Vec<String>>,
    truncated: usize,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoAhvPage {
    player: GoPlayer,
    face: GoFace,
    head: String,
    note: String,
    backdrop: String,
    meta: GoAhvMeta,
    sections: Vec<GoAhvSection>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoConst {
    constant: f64,
    total: i32,
    played: i32,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoGrade {
    name: String,
    count: i32,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoShort {
    song_id: String,
    title: String,
    diff: String,
    constant: f64,
    known: bool,
    score: i64,
    grade: String,
    next: String,
    need: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoStatsPage {
    rendered: Option<bool>,
    render_entry: String,
    placeholder_lines: Vec<String>,
    player: GoPlayer,
    face: GoFace,
    head: String,
    note: String,
    backdrop: String,
    bands: Vec<GoBand>,
    consts: Vec<GoConst>,
    const_note: String,
    grades: Vec<GoGrade>,
    short_title: String,
    short: Vec<GoShort>,
    short_note: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoNotes {
    known: bool,
    far_use: i64,
    lost_use: i64,
    short: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoHint {
    song_id: String,
    title: String,
    artist: String,
    diff: String,
    constant: f64,
    played: bool,
    score: i64,
    target: i64,
    delta: i64,
    grade: String,
    rank: usize,
    target_rank: usize,
    counted: bool,
    pick: bool,
    notes: GoNotes,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoSuggestPage {
    player: GoPlayer,
    face: GoFace,
    head: String,
    note: String,
    backdrop: String,
    section_note: String,
    condition: String,
    ptt_before: f64,
    ptt_after: f64,
    enough: bool,
    known: usize,
    total: usize,
    shown: usize,
    pushable: usize,
    hints: Vec<GoHint>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoChapPack {
    pack_id: String,
    name: String,
    total: usize,
    played: usize,
    cleared: usize,
    full_combo: usize,
    pm: usize,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoChapSection {
    name: String,
    note: String,
    packs: Vec<GoChapPack>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoChapPage {
    player: GoPlayer,
    face: GoFace,
    note: String,
    stamp: String,
    pack_id: String,
    charts_title: String,
    charts_note: String,
    sections: Vec<GoChapSection>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoCurvePoint {
    at: String,
    ptt: f64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoCurve {
    points: Vec<GoCurvePoint>,
    peak: f64,
    peak_at: Option<String>,
    recent7: Option<f64>,
    has_recent7: bool,
    recent30: Option<f64>,
    has_recent30: bool,
    records30: Option<i32>,
    has_records30: bool,
    first: Option<String>,
    last: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoRank {
    rank: usize,
    total: usize,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoTrendPage {
    player: GoPlayer,
    face: GoFace,
    head: String,
    note: String,
    days: usize,
    rank: GoRank,
    frame: i32,
    bounds: Vec<f64>,
    curve: GoCurve,
    stats: Vec<GoStat>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoHisRow {
    at: String,
    score: i64,
    grade: String,
    pure: i64,
    shiny: i64,
    far: i64,
    lost: i64,
    ptt: Option<f64>,
    has_ptt: bool,
    account_ptt: Option<f64>,
    has_account_ptt: bool,
    delta: i64,
    has_delta: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoHisSection {
    diff: String,
    constant: f64,
    has_constant: bool,
    best_score: i64,
    truncated: bool,
    rows: Vec<GoHisRow>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoHisPage {
    player: GoPlayer,
    face: GoFace,
    head: String,
    note: String,
    title: String,
    artist: String,
    song_id: String,
    diff: String,
    limit: usize,
    truncated: bool,
    failed: usize,
    stats: Vec<GoStat>,
    sections: Vec<GoHisSection>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoBand {
    name: String,
    total: i32,
    played: i32,
    cleared: i32,
    full_combo: i32,
    pm: i32,
    avg_score: f64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoBin {
    constant: f64,
    low: f64,
    high: f64,
    avg: f64,
    count: i32,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoScatter {
    max_ptt: f64,
    avg_ptt: Option<f64>,
    has_avg_ptt: bool,
    ability: Option<f64>,
    has_ability: bool,
    bins: Vec<GoBin>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoMeRow {
    rank: i32,
    name: String,
    icon: GoIcon,
    avatar: Option<String>,
    ptt: Option<f64>,
    has_ptt: bool,
    rating_frame: Option<i32>,
    rating_accent: Option<i32>,
    me: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoMeBoard {
    total: i32,
    rows: Vec<GoMeRow>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoChange {
    song_id: String,
    title: String,
    diff: String,
    has_old: bool,
    old_score: i64,
    new_score: i64,
    at: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoMePage {
    player: GoPlayer,
    face: GoFace,
    head: String,
    note: String,
    rank: GoRank,
    curve: GoCurve,
    bands: Vec<GoBand>,
    bands_note: String,
    scatter: GoScatter,
    board: GoMeBoard,
    changes: Vec<GoChange>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoEntry {
    index: usize,
    image: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoGalleryPage {
    kind: String,
    title: String,
    note: String,
    hint: String,
    foot: String,
    grid: String,
    entries: Vec<GoEntry>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GoThemePage {
    hint: String,
    foot: String,
    current: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct Fixtures {
    b50: Option<GoBoard>,
    score: Option<GoScorePage>,
    recent: Option<GoScorePage>,
    song: Option<GoSongPage>,
    rand: Option<GoRandPage>,
    search: Option<GoSearchPage>,
    guess: Option<GoGuessPage>,
    list: Option<GoListPage>,
    table: Option<GoTablePage>,
    ahv: Option<GoAhvPage>,
    stats: Option<GoStatsPage>,
    suggest: Option<GoSuggestPage>,
    chap: Option<GoChapPage>,
    trend: Option<GoTrendPage>,
    his: Option<GoHisPage>,
    me: Option<GoMePage>,
    char: Option<GoGalleryPage>,
    banner: Option<GoGalleryPage>,
    theme: Option<GoThemePage>,
    score_arknights: Option<GoScorePage>,
}

fn case_426(player: &GoPlayer, face: &GoFace) -> PlayerBadge {
    let mut badge = PlayerBadge::new(player.name.clone());
    if let Some(ptt) = player.ptt {
        badge = badge.with_ptt(ptt);
    }
    badge = badge.with_avatar(format!("char_icons/{}", face.avatar));
    badge = badge.with_banner(format!("banners/{}", face.banner));
    badge = badge.with_character(format!("characters/{}", face.character));
    badge
}

fn case_527(icon: &GoIcon, face: &GoFace) -> Option<String> {
    if !icon.known && icon.id <= 0 {
        return None;
    }
    if face.avatar.is_empty() {
        None
    } else {
        Some(format!("char_icons/{}", face.avatar))
    }
}

fn case_427(row: &GoScoreRow) -> BoardRow {
    BoardRow {
        rank: row.rank,
        song_id: row.song_id.clone(),
        title: row.title.clone(),
        difficulty: case_516(&row.diff),
        constant: row.constant,
        has_constant: row.constant > 0.0,
        score: row.score,
        rating: row.rating,
        grade: ptt::grade_of(row.score),
        far: row.far,
        lost: row.lost,
        normal_perfect: 18,
        object_amount: row.pure.max(row.pure + row.far + row.lost),
        clear_type: case_523(&row.clear),
        has_judgement: true,
        band: None,
        span: 0.0,
        pressure: 0.0,
        play_at: None,
        mess: 0.0,
        over_delta: 0.0,
        has_over_level: false,
        over_level: None,
        over_band: None,
        over_mess: None,
        backend_pressure: None,
    }
}

fn case_528(page: &GoScorePage, theme: Theme) -> score::ScoreInput {
    let play = &page.play;
    score::ScoreInput {
        badge: case_426(&page.player, &page.face),
        theme,
        song_id: play.song_id.clone(),
        title: play.title.clone(),
        artist: play.artist.clone(),
        difficulty: case_516(&play.diff),
        constant: play.constant,
        level: String::new(),
        score: play.score,
        grade: case_517(&play.grade).unwrap_or(Grade::ExPlus),
        pure: play.pure,
        far: play.far,
        lost: play.lost,
        shiny_perfect: Some(play.shiny_perfect),
        normal_perfect: Some(play.normal_perfect),
        rating_frame: None,
        rating: play.rating,
        ptt: if page.has_ptt { page.player.ptt } else { None },
        ptt_delta: None,
        previous_rating: None,
        clear_type: play.has_clear.then_some(play.clear_type),
        skill: String::new(),
        jacket: Some(format!("jackets/{}.jpg", play.song_id)),
        play_at: play.played_at.as_deref().and_then(case_518),
        note: String::new(),
        recollection: None,
    }
}

fn case_529(page: &GoSongPage) -> song::SongInput {
    let note_parts: Vec<&str> = page.note.split(" · ").collect();
    let artist = note_parts.first().copied().unwrap_or("").to_string();
    let pack = note_parts.get(1).copied().unwrap_or("");
    let pack_parts: Vec<&str> = pack.split(" / ").collect();
    let charts: Vec<Chart> = page
        .levels
        .iter()
        .map(|level| Chart {
            difficulty: case_516(&level.diff),
            level: level.level.clone(),
            constant: level.constant,
            has_constant: level.has_constant,
            designer: String::new(),
            note: level.note,
            chart_url: String::new(),
            ill_url: String::new(),
            audio_url: String::new(),
        })
        .collect();
    let boards: Vec<song::SongBoard> = page
        .boards
        .iter()
        .map(|board| song::SongBoard {
            difficulty: case_516(&board.diff),
            rows: board
                .rows
                .iter()
                .map(|row| song::SongBoardRow {
                    name: row.name.clone(),
                    avatar: case_527(&row.icon, &page.face),
                    grade: row.grade.clone(),
                    score: row.score,
                    rating: row.rating,
                    has_rating: row.has_rating,
                    rating_frame: row.rating_frame,
                    rating_accent: row.rating_accent,
                    perfect: row.perfect,
                    normal_perfect: row.normal_perfect,
                    far: row.far,
                    lost: row.lost,
                    judgement: row.has_judgement,
                })
                .collect(),
        })
        .collect();
    song::SongInput {
        song_id: page.cover.song_id.clone(),
        title: page.head.clone(),
        title_ja: String::new(),
        aliases: page
            .alias
            .split(" / ")
            .filter(|part| !part.trim().is_empty())
            .map(|part| part.trim().to_string())
            .collect(),
        artist,
        bpm: 0.0,
        bpm_label: None,
        release: String::from("2026-01-01"),
        duration: 0,
        pack_id: String::new(),
        pack_name: pack_parts.first().copied().unwrap_or("").to_string(),
        pack_name_zh: pack_parts.get(1).copied().unwrap_or("").to_string(),
        jacket: Some(format!("jackets/{}.jpg", page.cover.song_id)),
        charts,
        ratings: Vec::new(),
        theme: Theme::DEFAULT,
        footer_note: page.board_note.clone(),
        cells: Some(
            page.cells
                .iter()
                .map(|cell| (cell.label.clone(), cell.value.clone()))
                .collect(),
        ),
        boards: Some(boards),
    }
}

fn case_530(page: &GoRandPage) -> select::SelectInput {
    let select = &page.select;
    let rows: Vec<select::SelectRow> = select
        .rows
        .iter()
        .map(|row| select::SelectRow {
            title: row.title.clone(),
            song_id: row.song_id.clone(),
            artist: row.artist.clone(),
            difficulty: case_516(&row.diff),
            level: row.level.clone(),
            constant: row.constant,
            has_constant: row.has_constant,
            selected: row.selected,
            clear: row.clear.chars().next().unwrap_or('\0'),
            grade: row.grade.clone(),
            bpm: row.bpm.to_string(),
        })
        .collect();
    let chosen = select
        .rows
        .iter()
        .find(|row| row.selected)
        .or_else(|| select.rows.first());
    select::SelectInput {
        song_id: chosen.map(|row| row.song_id.clone()).unwrap_or_default(),
        title: chosen.map(|row| row.title.clone()).unwrap_or_default(),
        artist: chosen.map(|row| row.artist.clone()).unwrap_or_default(),
        pack_id: String::new(),
        pack_name: select.pack.clone(),
        pack_name_zh: String::new(),
        jacket: chosen.map(|row| format!("jackets/{}.jpg", row.song_id)),
        backdrop: Some(String::from("background/score.png")),
        charts: chosen
            .map(|row| {
                vec![Chart {
                    difficulty: case_516(&row.diff),
                    level: row.level.clone(),
                    constant: row.constant,
                    has_constant: row.has_constant,
                    designer: String::new(),
                    note: 0,
                    chart_url: String::new(),
                    ill_url: String::new(),
                    audio_url: String::new(),
                }]
            })
            .unwrap_or_default(),
        selected: case_516(&select.diff),
        score: select.score,
        grade: None,
        pack_count: select.pack_count,
        clear_count: select.clear_count,
        pure_count: select.pure_count,
        total_count: select.total,
        retry_hint: String::new(),
        roll: String::new(),
        theme: Theme::DEFAULT,
        footer_note: String::new(),
        rows: Some(rows),
        levels: Some(select.levels.clone()),
        page_title: None,
        pack_title: Some(select.pack.clone()),
        player: Some(select::SelectPlayer {
            name: select.bar.name.clone(),
            ptt: if select.bar.has_ptt { select.bar.ptt } else { 0.0 },
            rating_frame: select.bar.rating_frame,
            rating_accent: 0,
            global_rank: None,
            banner: Some(format!("banners/{}", page.face.banner)),
            avatar: Some(format!("char_icons/{}", page.face.avatar)),
        }),
    }
}

fn case_531(page: &GoSearchPage) -> search::SearchInput {
    let hits: Vec<search::SearchHit> = page
        .rows
        .iter()
        .map(|row| search::SearchHit {
            song_id: row.song_id.clone(),
            title: row.title.clone(),
            artist: row.artist.clone(),
            jacket: Some(format!("jacket_thumbs/{}.jpg", row.song_id)),
            charts: row
                .levels
                .iter()
                .map(|level| Chart {
                    difficulty: case_516(&level.diff),
                    level: level.constant_text.clone().unwrap_or_default(),
                    constant: level.constant.unwrap_or(0.0),
                    has_constant: level.constant.unwrap_or(0.0) > 0.0,
                    designer: String::new(),
                    note: 0,
                    chart_url: String::new(),
                    ill_url: String::new(),
                    audio_url: String::new(),
                })
                .collect(),
            group: row.group.clone(),
            active: row
                .levels
                .iter()
                .find(|level| level.active)
                .map(|level| case_516(&level.diff)),
        })
        .collect();
    search::SearchInput {
        query: page.head.clone(),
        hits,
        truncated: 0,
        theme: Theme::DEFAULT,
        footer_note: page.foot.clone(),
        head: Some(page.head.clone()),
        note: Some(page.note.clone()),
    }
}

fn case_532(page: &GoGuessPage) -> guess::GuessInput {
    guess::GuessInput {
        stage: if page.revealed {
            guess::GuessStage::Answer
        } else {
            guess::GuessStage::Question
        },
        round: 1,
        song_id: page.song_id.clone(),
        title: String::new(),
        artist: String::new(),
        jacket: Some(format!("jacket_thumbs/{}.jpg", page.song_id)),
        hints: Vec::new(),
        answer: String::new(),
        correct: false,
        cost: 0,
        theme: Theme::DEFAULT,
        footer_note: String::new(),
        seed: Some(page.seed),
        level: Some(page.level),
        zoom: Some(page.zoom),
        interference: Some(page.interference),
        variant: Some(page.variant.clone()),
        code: Some(page.code.clone()),
    }
}

fn case_533(page: &GoListPage) -> lists::ListInput {
    let entries: Vec<lists::ListEntry> = page
        .rows
        .iter()
        .map(|row| lists::ListEntry {
            song_id: row.song_id.clone(),
            title: row.title.clone(),
            artist: row.composer.clone(),
            difficulty: case_516(&row.diff),
            level: String::new(),
            constant: row.constant,
            has_constant: row.known,
            score: row.score,
            has_score: row.score > 0,
            has_judgement: row.has_judgement,
            perfect: row.pure,
            shiny: row.pure - row.normal_perfect,
            far: row.far,
            lost: row.lost,
            loss: row.has_judgement.then_some(row.loss),
            grade: case_517(&row.grade),
            clear_type: row.has_judgement.then_some(1),
            jacket: Some(format!("jacket_thumbs/{}.jpg", row.song_id)),
        })
        .collect();
    lists::ListInput {
        title: page.head.clone(),
        subtitle: page.note.clone(),
        theme: Theme::DEFAULT,
        badge: case_426(&page.player, &page.face),
        entries,
        note: String::new(),
        backdrop: page.backdrop.clone(),
    }
}

fn case_534(page: &GoTablePage) -> lists::TableInput {
    let sheets: Vec<lists::TableSheet> = page
        .sections
        .iter()
        .map(|section| lists::TableSheet {
            band: section.label.clone(),
            charts: section
                .rows
                .iter()
                .map(|row| lists::TableChart {
                    song_id: row.song_id.clone(),
                    title: row.title.clone(),
                    difficulty: case_516(&row.diff),
                    constant: row.constant,
                    has_constant: row.constant > 0.0,
                    score: 0,
                    has_score: false,
                    jacket: Some(format!("jacket_thumbs/{}.jpg", row.song_id)),
                })
                .collect(),
        })
        .collect();
    lists::TableInput {
        title: page.head.clone(),
        subtitle: page.meta.cond.clone().unwrap_or_default().join("\n"),
        theme: Theme::DEFAULT,
        badge: case_426(&page.player, &page.face),
        library: page.meta.library,
        mode: lists::TableMode::Charts,
        sheets,
        songs: Vec::new(),
        note: String::new(),
    }
}

fn case_535(page: &GoAhvPage) -> lists::AhvInput {
    let records: Vec<lists::AhvRecord> = page
        .sections
        .iter()
        .flat_map(|section| section.cards.iter())
        .map(|card| lists::AhvRecord {
            song_id: card.song_id.clone(),
            title: card.title.clone(),
            difficulty: case_516(&card.diff),
            constant: card.constant,
            has_constant: card.constant > 0.0,
            score: card.score,
            score_max: ptt::FULL_SCORE,
            far: card.far,
            lost: card.lost,
            has_judgement: card.has_judgement,
            clear_type: card.clear_type,
            grade: Some(ptt::grade_for(
                card.score,
                card.far,
                card.lost,
                card.has_judgement,
            )),
            jacket: Some(format!("jacket_thumbs/{}.jpg", card.song_id)),
            pure: Some(card.perfect),
            shiny: Some(card.shiny_perfect),
            rating: card.rating,
            loss: card.loss,
        })
        .collect();
    lists::AhvInput {
        title: page.head.clone(),
        subtitle: page.meta.cond.clone().unwrap_or_default().join("\n"),
        theme: Theme::DEFAULT,
        badge: case_426(&page.player, &page.face),
        records,
        note: String::new(),
        generated_at: page.player.date.as_deref().map(case_519),
    }
}

fn case_536(entry: &GoHint) -> SuggestHint {
    SuggestHint {
        song_id: entry.song_id.clone(),
        title: entry.title.clone(),
        difficulty: case_516(&entry.diff),
        constant: entry.constant,
        note: 0,
        score: entry.score,
        has_score: entry.score > 0,
        far: 0,
        lost: 0,
        has_judgement: false,
        rating: 0.0,
        target_rating: 0.0,
        delta: entry.delta,
        target: entry.target,
        rank: entry.rank,
        target_rank: entry.target_rank,
        pick: entry.pick,
        notes: NotePlan {
            known: entry.notes.known,
            far_need: 0,
            lost_need: 0,
            far_room: 0,
            lost_room: 0,
            far_use: entry.notes.far_use,
            lost_use: entry.notes.lost_use,
            short: entry.notes.short,
        },
    }
}

fn case_537(page: &GoSuggestPage) -> stats::SuggestInput {
    let hints: Vec<SuggestHint> = page.hints.iter().map(case_536).collect();
    let picks: Vec<SuggestHint> = hints.iter().filter(|hint| hint.pick).cloned().collect();
    let mut groups: Vec<SuggestGroup> = Vec::new();
    if !picks.is_empty() {
        groups.push(SuggestGroup {
            label: String::new(),
            title: Some(PICK_TITLE),
            hints: picks,
        });
    }
    let mut levels: Vec<i64> = hints
        .iter()
        .filter(|hint| !hint.pick)
        .map(|hint| hint.constant.floor() as i64)
        .collect();
    levels.sort_unstable_by(|left, right| right.cmp(left));
    levels.dedup();
    for level in levels {
        let bucket: Vec<SuggestHint> = hints
            .iter()
            .filter(|hint| !hint.pick && hint.constant.floor() as i64 == level)
            .cloned()
            .collect();
        if bucket.is_empty() {
            continue;
        }
        groups.push(SuggestGroup {
            label: level.to_string(),
            title: None,
            hints: bucket,
        });
    }
    let pick_count = hints.iter().filter(|hint| hint.pick).count();
    stats::SuggestInput {
        title: page.head.clone(),
        subtitle: page.note.clone(),
        theme: Theme::DEFAULT,
        badge: case_426(&page.player, &page.face),
        board: SuggestBoard {
            groups,
            hint_count: hints.len(),
            pick_count,
            pool: page.total,
            pushable: page.pushable,
        },
        note: page.section_note.clone(),
        backdrop: page.backdrop.clone(),
        ptt_before: None,
        ptt_after: None,
    }
}

fn case_538(page: &GoChapPage) -> stats::ChapInput {
    let packs: Vec<stats::ChapPack> = page
        .sections
        .iter()
        .flat_map(|section| {
            section.packs.iter().map(|pack| stats::ChapPack {
                pack_id: pack.pack_id.clone(),
                name: pack.name.clone(),
                song_count: pack.total,
                owned: pack.played,
                cleared: pack.cleared,
                full_combo: pack.full_combo,
                pure_memory: pack.pm,
                jackets: Vec::new(),
                charts: Vec::new(),
                section: section.name.clone(),
                avg_score: 0.0,
                avg_ptt: 0.0,
            })
        })
        .collect();
    stats::ChapInput {
        title: stats::ChapInput::new(case_426(&page.player, &page.face)).title,
        subtitle: page.note.clone(),
        theme: Theme::DEFAULT,
        badge: case_426(&page.player, &page.face),
        packs,
        expand: stats::ChapExpand::All,
        focus: None,
        totals: None,
        note: page.stamp.clone(),
    }
}

fn case_539() -> Vec<String> {
    let path = harness::assets().root().join("jacket_pool.json");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return Vec::new();
    };
    value
        .get("jackets")
        .and_then(|list| list.as_array())
        .map(|list| {
            list.iter()
                .filter_map(|item| item.as_str().map(|id| id.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn case_540(parts: &[String], pool: &[String]) -> String {
    if pool.is_empty() {
        return String::new();
    }
    let mut hash: u64 = 1_469_598_103_934_665_603;
    for part in parts {
        for byte in part.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(1_099_511_628_211);
        }
    }
    pool[(hash % pool.len() as u64) as usize].clone()
}

fn case_541(page: &GoTrendPage) -> stats::TrendInput {
    let mut parts: Vec<String> = vec![page.head.clone(), page.curve.points.len().to_string()];
    if let Some(last) = page.curve.points.last() {
        parts.push(format!("{:.2}", last.ptt));
        parts.push(last.at.chars().take(10).collect());
    }
    let backdrop = case_540(&parts, &case_539());
    stats::TrendInput {
        title: page.head.clone(),
        subtitle: page.note.clone(),
        theme: Theme::DEFAULT,
        badge: case_426(&page.player, &page.face),
        points: page
            .curve
            .points
            .iter()
            .map(|point| {
                let at = case_518(&point.at);
                stats::TrendPoint {
                    label: case_521(&point.at),
                    value: point.ptt,
                    score: 0,
                    date: case_522(&point.at),
                    time: at
                        .map(|value| value.format("%H:%M").to_string())
                        .unwrap_or_default(),
                    minute: at.map(|value| value.timestamp() / 60).unwrap_or(0),
                }
            })
            .collect(),
        note: String::new(),
        backdrop,
        days: None,
        rank: None,
        rank_total: None,
        peak: None,
        recent7: None,
        recent30: None,
        records: None,
        frame: None,
        bounds: Vec::new(),
    }
}

fn case_542(page: &GoHisPage) -> stats::HisInput {
    let section = page.sections.first().cloned().unwrap_or_default();
    let records: Vec<stats::HisRecord> = section
        .rows
        .iter()
        .map(|row| stats::HisRecord {
            normal_perfect: 0,
            label: case_520(&row.at),
            score: row.score,
            delta: if row.has_delta { row.delta } else { 0 },
            ptt: row.ptt.unwrap_or(0.0),
            perfect: row.pure,
            far: row.far,
            lost: row.lost,
            grade: case_517(&row.grade),
            clear_type: None,
        })
        .collect();
    stats::HisInput {
        title: page.head.clone(),
        subtitle: page.note.clone(),
        theme: Theme::DEFAULT,
        badge: case_426(&page.player, &page.face),
        song_id: page.song_id.clone(),
        song_title: page.title.clone(),
        jacket: Some(format!("jackets/{}.jpg", page.song_id)),
        sections: vec![stats::HisSection::new(case_516(&page.diff))
            .with_constant(section.constant)
            .with_records(records)],
        note: String::new(),
        omitted: String::new(),
    }
}

fn case_543(page: &GoCurve) -> gallery::MeCurve {
    gallery::MeCurve {
        points: page
            .curve_points()
            .iter()
            .map(|point| gallery::MeCurvePoint {
                at: case_518(&point.at).map(|value| value.timestamp()).unwrap_or(0),
                ptt: point.ptt,
            })
            .collect(),
        peak: page.peak,
        peak_at: page
            .peak_at
            .as_deref()
            .and_then(case_518)
            .map(|value| value.timestamp()),
        recent7: page.has_recent7.then_some(page.recent7.unwrap_or(0.0)),
        recent30: page.has_recent30.then_some(page.recent30.unwrap_or(0.0)),
        records30: page.has_records30.then(|| page.records30.unwrap_or(0)),
        first: page
            .first
            .as_deref()
            .and_then(case_518)
            .map(|value| value.timestamp()),
        last: page
            .last
            .as_deref()
            .and_then(case_518)
            .map(|value| value.timestamp()),
    }
}

impl GoCurve {
    fn curve_points(&self) -> &Vec<GoCurvePoint> {
        &self.points
    }
}

fn case_544(page: &GoMePage) -> gallery::MeInput {
    gallery::MeInput {
        title: page.head.clone(),
        subtitle: page.note.clone(),
        theme: Theme::DEFAULT,
        badge: case_426(&page.player, &page.face),
        user_id: String::new(),
        region: Region::Cn,
        joined: String::new(),
        updated: page.player.date.as_deref().map(case_522).unwrap_or_default(),
        best_average: 0.0,
        best: Vec::new(),
        recent: Vec::new(),
        note: String::new(),
        curve: Some(case_543(&page.curve)),
        pm_small_p: 0,
        bands: Some(
            page.bands
                .iter()
                .map(|band| gallery::MeBand {
                    kind: case_516(&band.name),
                    name: band.name.clone(),
                    played: band.played,
                    total: band.total,
                    cleared: band.cleared,
                    full_combo: band.full_combo,
                    pm: band.pm,
                    ex_plus: 0,
                    ex: 0,
                    loss_played: 0.0,
                    loss_total: 0.0,
                })
                .collect(),
        ),
        scatter: Some(gallery::MeScatter {
            bins: page
                .scatter
                .bins
                .iter()
                .map(|bin| gallery::MeBin {
                    constant: bin.constant,
                    low: bin.low,
                    high: bin.high,
                    avg: bin.avg,
                    count: bin.count,
                })
                .collect(),
            max_ptt: page.scatter.max_ptt,
            avg_ptt: page.scatter.has_avg_ptt.then_some(page.scatter.avg_ptt.unwrap_or(0.0)),
            ability: page.scatter.has_ability.then_some(page.scatter.ability.unwrap_or(0.0)),
        }),
        board: Some(gallery::MeBoard {
            total: page.board.total,
            rows: page
                .board
                .rows
                .iter()
                .map(|row| gallery::MeBoardRow {
                    rank: row.rank,
                    name: row.name.clone(),
                    avatar: case_527(&row.icon, &page.face),
                    ptt: row.has_ptt.then_some(row.ptt.unwrap_or(0.0)),
                    rating_frame: row.rating_frame,
                    rating_accent: row.rating_accent,
                    me: row.me,
                })
                .collect(),
        }),
        changes: Some(
            page.changes
                .iter()
                .map(|change| gallery::MeChange {
                    title: change.title.clone(),
                    diff: change.diff.clone(),
                    old_score: change.has_old.then_some(change.old_score),
                    new_score: change.new_score,
                    at: change
                        .at
                        .as_deref()
                        .and_then(case_518)
                        .map(|value| value.timestamp()),
                    jacket: Some(format!("jacket_thumbs/{}.jpg", change.song_id)),
                })
                .collect(),
        ),
        rating_frame: page.player.rating_frame,
        rating_accent: page.player.rating_accent,
    }
}

fn case_545(page: &GoGalleryPage) -> gallery::CharInput {
    gallery::CharInput {
        title: page.title.clone(),
        subtitle: page.note.clone(),
        theme: Theme::DEFAULT,
        cells: page
            .entries
            .iter()
            .map(|entry| gallery::CharCell::new(case_525(&entry.image)))
            .collect(),
        group: String::new(),
        note: page.foot.clone(),
    }
}

fn case_546(page: &GoGalleryPage) -> gallery::BannerInput {
    gallery::BannerInput {
        title: page.title.clone(),
        subtitle: page.note.clone(),
        theme: Theme::DEFAULT,
        cells: page
            .entries
            .iter()
            .map(|entry| gallery::BannerCell::new(format!("banners/{}", entry.image)))
            .collect(),
        group: String::new(),
        note: page.foot.clone(),
    }
}

fn case_547(page: &GoThemePage) -> gallery::ThemeInput {
    gallery::ThemeInput {
        title: gallery::ThemeInput::new().title,
        subtitle: page.hint.clone(),
        theme: Theme::DEFAULT,
        themes: vec![
            gallery::ThemeEntry::new(Theme::DEFAULT.name, "默认初始主题")
                .with_preview(Theme::DEFAULT.backdrop)
                .with_active(page.current.is_empty()),
            gallery::ThemeEntry::new(Theme::by_name("arknights").name, "Arknights")
                .with_preview(Theme::by_name("arknights").backdrop)
                .with_active(!page.current.is_empty()),
        ],
        note: page.foot.clone(),
    }
}

fn case_548(page: &GoStatsPage) -> bool {
    page.bands.is_empty()
        && page.consts.is_empty()
        && page.grades.is_empty()
        && page.short.is_empty()
}

fn case_549(page: &GoStatsPage) -> stats::StatsInput {
    stats::StatsInput {
        title: page.head.clone(),
        note: page.note.clone(),
        backdrop: page.backdrop.clone(),
        badge: case_426(&page.player, &page.face),
        bands: page
            .bands
            .iter()
            .map(|band| stats::StatsBand {
                name: band.name.clone(),
                total: band.total,
                played: band.played,
                cleared: band.cleared,
                full_combo: band.full_combo,
                pm: band.pm,
                avg_score: band.avg_score,
            })
            .collect(),
        consts: page
            .consts
            .iter()
            .map(|entry| stats::StatsConst {
                constant: entry.constant,
                total: entry.total,
                played: entry.played,
            })
            .collect(),
        const_note: page.const_note.clone(),
        grades: page
            .grades
            .iter()
            .map(|grade| stats::StatsGrade {
                name: grade.name.clone(),
                count: grade.count,
            })
            .collect(),
        short_title: page.short_title.clone(),
        short: page
            .short
            .iter()
            .map(|row| stats::StatsRow {
                song_id: row.song_id.clone(),
                title: row.title.clone(),
                diff: row.diff.clone(),
                constant: row.constant,
                known: row.known,
                score: row.score,
                grade: row.grade.clone(),
                next: row.next.clone(),
                need: row.need,
            })
            .collect(),
        short_note: page.short_note.clone(),
    }
}

fn case_550(spec: &str, name: &str) -> Option<usize> {
    let mut hit = None;
    for part in spec.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (key, times) = match part.split_once('@') {
            Some((key, times)) => (key.trim(), times.trim().parse().unwrap_or(1)),
            None => (part, 1),
        };
        if name.contains(key) {
            let times = times.max(1);
            hit = Some(hit.unwrap_or(0).max(times));
        }
    }
    hit
}

fn case_551(name: &str, mut draw: impl FnMut() -> Vec<u8>) {
    let times = match std::env::var("卡片") {
        Ok(value) if !value.is_empty() => match case_550(&value, name) {
            Some(times) => times,
            None => {
                println!("{name} 跳过");
                return;
            }
        },
        _ => 1,
    };
    let mut samples: Vec<f64> = Vec::with_capacity(times);
    let mut bytes = Vec::new();
    for _ in 0..times {
        let started = std::time::Instant::now();
        bytes = draw();
        samples.push(started.elapsed().as_secs_f64() * 1000.0);
    }
    let dir = std::path::Path::new(VERIFY_DIR);
    fs::create_dir_all(dir).expect("出图目录必须可创建");
    fs::write(dir.join(name), &bytes).expect("出图必须可写入");
    let (w, h) = harness::image_size(&bytes).unwrap_or((0, 0));
    if samples.len() > 1 {
        let mut warm = samples[1..].to_vec();
        warm.sort_by(|a, b| a.partial_cmp(b).expect("耗时必须是数字"));
        let median = warm[warm.len() / 2];
        println!(
            "{name} {w}x{h} 冷 {:.0}ms 暖中位 {:.0}ms (n={})",
            samples[0],
            median,
            samples.len()
        );
    } else {
        println!("{name} {w}x{h} 冷 {:.0}ms", samples[0]);
    }
}

#[test]
#[ignore]
fn case_552() {
    let raw = fs::read_to_string(夹具).expect("夹具必须可读");
    let root: Fixtures = serde_json::from_str(&raw).expect("夹具必须可解析");
    let dir = PathBuf::from(VERIFY_DIR);
    fs::create_dir_all(&dir).expect("出图目录必须可创建");
    let harness = harness::Harness::new();
    let page = root.b50.as_ref().expect("夹具缺 b50 键");
    let mut rows: Vec<BoardRow> = Vec::new();
    rows.extend(page.top10.iter().map(case_427));
    rows.extend(page.best50.iter().map(case_427));
    let input = b50::B50Input {
        badge: case_426(&page.player, &page.face),
        theme: Theme::DEFAULT,
        board: None,
        best: case_526(rows),
        recent: Vec::new(),
        ptt: page.player.ptt,
        note: String::new(),
        generated_at: page.player.date.as_deref().map(case_519),
        rank_frame: page.player.rating_frame,
        board_stat: None,
        anchor_upper: 0.0,
        over_count: None,
        best10_avg: None,
        best50_avg: None,
        best_count: None,
        max_ptt: None,
        cn: page.cn,
    };
    case_551(
        &case_524("01-b50"),
        || harness.render(b50::request(&input), |kit| b50::paint(kit, &input)),
    );

    let 开始 = std::time::Instant::now();
    let jpeg = harness.render(
        b50::request(&input)
            .with_format(aurorix_render::RenderFormat::Jpeg)
            .with_quality(95),
        |kit| b50::paint(kit, &input),
    );
    let jpeg_ms = 开始.elapsed().as_secs_f64() * 1000.0;
    let jpeg_path = dir.join("01-b50.jpg");
    fs::write(&jpeg_path, &jpeg).expect("JPEG 出图必须可写入");
    let png_bytes = fs::metadata(dir.join("01-b50.png"))
        .map(|meta| meta.len())
        .unwrap_or(0);
    println!(
        "编码对照 01-b50 1820x2134: PNG(fdeflate) {}KB | JPEG(4:4:4 q95) {}KB | JPEG 端到端 {:.0}ms",
        png_bytes / 1024,
        jpeg.len() as u64 / 1024,
        jpeg_ms
    );
}

#[test]
#[ignore]
fn case_553() {
    let raw = fs::read_to_string(夹具).expect("夹具必须可读");
    let root: Fixtures = serde_json::from_str(&raw).expect("夹具必须可解析");
    let dir = PathBuf::from(VERIFY_DIR);
    fs::create_dir_all(&dir).expect("出图目录必须可创建");
    let harness = harness::Harness::new();
    if std::env::var("预热").is_ok() {
        let started = std::time::Instant::now();
        let report = aurorix_arcaea_plugin::warmup(harness.assets(), harness.context());
        println!(
            "预热 {:.0}ms 字体 {} 模板 {} 贴图 {} 缺失 {}",
            started.elapsed().as_secs_f64() * 1000.0,
            report.fonts,
            report.templates,
            report.textures,
            report.missing
        );
    }
    if let Some(page) = root.b50.as_ref() {
        let mut rows: Vec<BoardRow> = Vec::new();
        rows.extend(page.top10.iter().map(case_427));
        rows.extend(page.best50.iter().map(case_427));
        let input = b50::B50Input {
            badge: case_426(&page.player, &page.face),
            theme: Theme::DEFAULT,
            board: None,
            best: case_526(rows),
            recent: Vec::new(),
            ptt: page.player.ptt,
            note: String::new(),
            generated_at: page.player.date.as_deref().map(case_519),
            rank_frame: page.player.rating_frame,
            board_stat: None,
            anchor_upper: 0.0,
            over_count: None,
            best10_avg: None,
            best50_avg: None,
            best_count: None,
            max_ptt: None,
            cn: page.cn,
        };
        case_551(
            &case_524("01-b50"),
            || harness.render(b50::request(&input), |kit| b50::paint(kit, &input)),
        );
    } else {
        println!("01-b50.png 跳过: 夹具缺 b50 键");
    }

    if let Some(page) = root.score.as_ref() {
        let input = case_528(page, Theme::DEFAULT);
        case_551(
            &case_524("02-score"),
            || harness.render(score::score_request(&input), |kit| {
                score::paint_score(kit, &input)
            }),
        );
    } else {
        println!("02-score.png 跳过: 夹具缺 score 键");
    }

    if let Some(page) = root.recent.as_ref() {
        let input = case_528(page, Theme::DEFAULT);
        case_551(
            &case_524("03-recent"),
            || harness.render(score::recent_request(&input), |kit| {
                score::paint_recent(kit, &input)
            }),
        );
    } else {
        println!("03-recent.png 跳过: 夹具缺 recent 键");
    }

    if let Some(page) = root.song.as_ref() {
        let input = case_529(page);
        case_551(
            &case_524("04-song"),
            || harness.render(song::request(&input), |kit| song::paint(kit, &input)),
        );
    } else {
        println!("04-song.png 跳过: 夹具缺 song 键");
    }

    if let Some(page) = root.rand.as_ref() {
        let input = case_530(page);
        case_551(
            &case_524("05-rand"),
            || harness.render(select::request(&input), |kit| {
                select::paint(kit, &input)
            }),
        );
    } else {
        println!("05-rand.png 跳过: 夹具缺 rand 键");
    }

    if let Some(page) = root.search.as_ref() {
        let input = case_531(page);
        case_551(
            &case_524("06-search"),
            || harness.render(search::request(&input), |kit| {
                search::paint(kit, &input)
            }),
        );
    } else {
        println!("06-search.png 跳过: 夹具缺 search 键");
    }

    if let Some(page) = root.guess.as_ref() {
        let input = case_532(page);
        case_551(
            &case_524("07-guess"),
            || harness.render(guess::request(&input), |kit| {
                guess::paint(kit, &input)
            }),
        );
    } else {
        println!("07-guess.png 跳过: 夹具缺 guess 键");
    }

    if let Some(page) = root.list.as_ref() {
        let input = case_533(page);
        case_551(
            &case_524("08-list"),
            || harness.render(lists::request(&input), |kit| lists::paint(kit, &input)),
        );
    } else {
        println!("08-list.png 跳过: 夹具缺 list 键");
    }

    if let Some(page) = root.table.as_ref() {
        let input = case_534(page);
        case_551(
            &case_524("09-table"),
            || harness.render(lists::table_request(&input), |kit| {
                lists::paint_table(kit, &input)
            }),
        );
    } else {
        println!("09-table.png 跳过: 夹具缺 table 键");
    }

    if let Some(page) = root.ahv.as_ref() {
        let input = case_535(page);
        case_551(
            &case_524("10-ahv"),
            || harness.render(lists::ahv_request(&input), |kit| {
                lists::paint_ahv(kit, &input)
            }),
        );
    } else {
        println!("10-ahv.png 跳过: 夹具缺 ahv 键");
    }

    match root.stats.as_ref() {
        Some(page) if page.rendered == Some(false) => println!(
            "11-stats.png 跳过: Go 侧 rendered=false ({}), 占位文案 {:?}",
            page.render_entry, page.placeholder_lines
        ),
        Some(page) if case_548(page) => {
            println!("11-stats.png 跳过: 夹具的 stats 既无 rendered=false, 也没有任何一段正文")
        }
        Some(page) => {
            let input = case_549(page);
            case_551(
                &case_524("11-stats"),
                || harness.render(stats::stats_request(&input), |kit| {
                    stats::paint_stats(kit, &input)
                }),
            );
        }
        None => println!("11-stats.png 跳过: 夹具缺 stats 键"),
    }

    if let Some(page) = root.suggest.as_ref() {
        let input = case_537(page);
        case_551(
            &case_524("12-suggest"),
            || harness.render(stats::suggest_request(&input), |kit| {
                stats::paint_suggest(kit, &input)
            }),
        );
    } else {
        println!("12-suggest.png 跳过: 夹具缺 suggest 键");
    }

    if let Some(page) = root.chap.as_ref() {
        let input = case_538(page);
        case_551(
            &case_524("13-chap"),
            || harness.render(stats::chap_request(&input), |kit| {
                stats::paint_chap(kit, &input)
            }),
        );
    } else {
        println!("13-chap.png 跳过: 夹具缺 chap 键");
    }

    if let Some(page) = root.trend.as_ref() {
        let input = case_541(page);
        case_551(
            &case_524("14-trend"),
            || harness.render(stats::trend_request(&input), |kit| {
                stats::paint_trend(kit, &input)
            }),
        );
    } else {
        println!("14-trend.png 跳过: 夹具缺 trend 键");
    }

    if let Some(page) = root.his.as_ref() {
        let input = case_542(page);
        case_551(
            &case_524("15-his"),
            || harness.render(stats::his_request(&input), |kit| {
                stats::paint_his(kit, &input)
            }),
        );
    } else {
        println!("15-his.png 跳过: 夹具缺 his 键");
    }

    if let Some(page) = root.me.as_ref() {
        let input = case_544(page);
        case_551(
            &case_524("16-me"),
            || harness.render(gallery::request(&input), |kit| {
                gallery::paint(kit, &input)
            }),
        );
    } else {
        println!("16-me.png 跳过: 夹具缺 me 键");
    }

    if let Some(page) = root.char.as_ref() {
        let input = case_545(page);
        case_551(
            &case_524("17-char"),
            || harness.render(gallery::char_request(&input), |kit| {
                gallery::paint_char(kit, &input)
            }),
        );
    } else {
        println!("17-char.png 跳过: 夹具缺 char 键");
    }

    if let Some(page) = root.banner.as_ref() {
        let input = case_546(page);
        case_551(
            &case_524("18-banner"),
            || harness.render(gallery::banner_request(&input), |kit| {
                gallery::paint_banner(kit, &input)
            }),
        );
    } else {
        println!("18-banner.png 跳过: 夹具缺 banner 键");
    }

    if let Some(page) = root.theme.as_ref() {
        let input = case_547(page);
        case_551(
            &case_524("19-theme"),
            || harness.render(gallery::theme_request(&input), |kit| {
                gallery::paint_theme(kit, &input)
            }),
        );
    } else {
        println!("19-theme.png 跳过: 夹具缺 theme 键");
    }

    if let Some(page) = root.score_arknights.as_ref() {
        let input = case_528(page, Theme::by_name("arknights"));
        case_551(
            &case_524("20-score-arknights"),
            || harness.render(score::score_request(&input), |kit| {
                score::paint_score(kit, &input)
            }),
        );
    } else {
        println!("20-score-arknights.png 跳过: 夹具缺 score_arknights 键");
    }
}

#[test]
#[ignore]
fn case_554() {
    let raw = fs::read_to_string(夹具).expect("夹具必须可读");
    let root: Fixtures = serde_json::from_str(&raw).expect("夹具必须可解析");
    let harness = harness::Harness::new();
    aurorix_render::card_cache::clear();
    if let Some(page) = root.b50.as_ref() {
        let mut rows: Vec<BoardRow> = Vec::new();
        rows.extend(page.top10.iter().map(case_427));
        rows.extend(page.best50.iter().map(case_427));
        let input = b50::B50Input {
            badge: case_426(&page.player, &page.face),
            theme: Theme::DEFAULT,
            board: None,
            best: case_526(rows),
            recent: Vec::new(),
            ptt: page.player.ptt,
            note: String::new(),
            generated_at: page.player.date.as_deref().map(case_519),
            rank_frame: page.player.rating_frame,
            board_stat: None,
            anchor_upper: 0.0,
            over_count: None,
            best10_avg: None,
            best50_avg: None,
            best_count: None,
            max_ptt: None,
            cn: page.cn,
        };
        let card = b50::card(input);
        let started = std::time::Instant::now();
        let cold = harness.render_card(&card);
        let cold_ms = started.elapsed().as_secs_f64() * 1000.0;
        let started = std::time::Instant::now();
        let hit = harness.render_card(&card);
        let hit_ms = started.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(cold, hit, "命中缓存的字节必须与首次渲染逐字节相同");
        println!(
            "01-b50 冷 {cold_ms:.0}ms 命中 {hit_ms:.2}ms 字节 {} 条目 {}",
            cold.len(),
            aurorix_render::card_cache::entries()
        );
    }
    if let Some(page) = root.score.as_ref() {
        let input = case_528(page, Theme::DEFAULT);
        let card = score::score_card(input);
        let started = std::time::Instant::now();
        let cold = harness.render_card(&card);
        let cold_ms = started.elapsed().as_secs_f64() * 1000.0;
        let started = std::time::Instant::now();
        let hit = harness.render_card(&card);
        let hit_ms = started.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(cold, hit, "命中缓存的字节必须与首次渲染逐字节相同");
        println!(
            "02-score 冷 {cold_ms:.0}ms 命中 {hit_ms:.2}ms 字节 {} 占用 {}",
            cold.len(),
            aurorix_render::card_cache::used_bytes()
        );
    }
}
