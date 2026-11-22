use aurorix_plugin::RenderCard;
use aurorix_render::skia_safe::{ClipOp, PathBuilder, Rect};
use aurorix_render::{CanvasKit, Color, RenderRequest, TextStyle};

use super::brand;
use super::sketch;
use super::theme;
use super::widgets;
use crate::domain::{Chart, Song};
use crate::model::Difficulty;

pub const SONG_SCALE: f64 = 1.0;

pub const SONG_BASE_H: f64 = 770.0;

pub const SONG_MARGIN: f64 = 36.0;

pub const SONG_COL_W: f64 = 332.0;

pub const SONG_COL_GAP: f64 = 16.0;

pub const SONG_ROW_N: usize = 5;

pub const SP_HERO_TOP: f64 = 32.0;

pub const SP_HERO_BOT: f64 = 212.0;

pub const SP_JACKET_L: f64 = 36.0;

pub const SP_JACKET_T1: f64 = 244.0;

pub const SP_JACKET_B1: f64 = 208.0;

pub const SP_JACKET_IMG: f64 = 210.0;

pub const SP_JACKET_MAT: f64 = 4.0;

pub const SP_BAND_T0: f64 = 248.0;

pub const SP_BAND_T1: f64 = 254.0;

pub const SP_BAND_B1: f64 = 218.0;

pub const SP_BAND_B0: f64 = 212.0;

pub const SP_TITLE_T0: f64 = 258.0;

pub const SP_TITLE_B0: f64 = 236.0;

pub const SP_TITLE_BOT: f64 = 140.0;

pub const SP_TITLE_TOP_X: f64 = -194.0;

pub const SP_TITLE_BOT_X: f64 = -216.0;

pub const SP_TITLE_RULE_INSET: f64 = 24.0;

pub const SP_NAME_X: f64 = 280.0;

pub const SP_NAME_BASE: f64 = 72.0;

pub const SP_NAME_SIZE: f64 = 32.0;

pub const SP_ARTIST_MARK_X: f64 = 280.0;

pub const SP_ARTIST_MARK_W: f64 = 4.0;

pub const SP_ARTIST_MARK_Y0: f64 = 86.0;

pub const SP_ARTIST_MARK_Y1: f64 = 100.0;

pub const SP_ARTIST_X: f64 = 292.0;

pub const SP_ARTIST_BASE: f64 = 100.0;

pub const SP_ARTIST_SIZE: f64 = 15.0;

pub const SP_ALIAS_X: f64 = 272.0;

pub const SP_ALIAS_BASE: f64 = 124.0;

pub const SP_ALIAS_SIZE: f64 = 12.0;

pub const SP_LOGO_BOT: f64 = 108.0;

pub const SP_LOGO_TOP_X: f64 = -190.0;

pub const SP_LOGO_BOT_X: f64 = -205.0;

pub const SP_PLATE_BOT_X: f64 = -212.0;

pub const SP_LOGO_CX: f64 = -96.0;

pub const SP_LOGO_CY: f64 = 70.0;

pub const SP_LOGO_W: f64 = 164.0;

pub const SP_KICK_CX: f64 = -104.0;

pub const SP_KICK_BASE: f64 = 128.0;

pub const SP_KICK_SIZE: f64 = 10.0;

pub const SP_KICK_TRACK: f64 = 3.0;

pub const SP_BAR_T0: f64 = 235.0;

pub const SP_BAR_B0: f64 = 221.0;

pub const SP_BAR_Y: f64 = 144.0;

pub const SP_BAR_BOT: f64 = 212.0;

pub const SP_BAR_CELLS: usize = 4;

pub const SP_BAR_SEP_W: f64 = 4.0;

pub const SP_BAR_SEP_LEAN: f64 = 14.0;

pub const SP_BAR_LAB_DX: f64 = 31.0;

pub const SP_BAR_VAL_DX: f64 = 25.0;

pub const SP_BAR_LAB_Y: f64 = 168.0;

pub const SP_BAR_VAL_Y: f64 = 196.0;

pub const SP_BAR_LAB_SZ: f64 = 10.0;

pub const SP_BAR_VAL_SZ: f64 = 17.0;

pub const SP_HEAD_Y: f64 = 232.0;

pub const SP_HEAD_H: f64 = 60.0;

pub const SP_HEAD_BOT: f64 = 292.0;

pub const SP_HEAD_LT1: f64 = 112.0;

pub const SP_HEAD_LB1: f64 = 100.0;

pub const SP_HEAD_RT0: f64 = 116.0;

pub const SP_HEAD_RB0: f64 = 104.0;

pub const SP_HEAD_SEAM_T: f64 = 206.0;

pub const SP_HEAD_SEAM_B: f64 = 198.0;

pub const SP_HEAD_SEAM_Y0: f64 = 242.0;

pub const SP_HEAD_SEAM_Y1: f64 = 282.0;

pub const SP_HEAD_SEAM_W: f64 = 1.5;

pub const SP_HEAD_TAG_X: f64 = 20.0;

pub const SP_HEAD_CONST_X: f64 = 14.0;

pub const SP_HEAD_TAG_BASE: f64 = 252.0;

pub const SP_HEAD_TAG_SIZE: f64 = 14.0;

pub const SP_HEAD_LV_X: f64 = 50.0;

pub const SP_HEAD_LV_SIZE: f64 = 18.0;

pub const SP_HEAD_LV_BASE: f64 =
    SP_HEAD_TAG_BASE - SP_HEAD_TAG_SIZE * 0.36 + SP_HEAD_LV_SIZE * 0.36;

pub const SP_HEAD_CONST_BASE: f64 = 282.0;

pub const SP_HEAD_CONST_SIZE: f64 = 26.0;

pub const SP_HEAD_LAB_BASE: f64 = 254.0;

pub const SP_HEAD_VAL_BASE: f64 = 278.0;

pub const SP_HEAD_NOTE_LAB_DX: f64 = 20.4;

pub const SP_HEAD_NOTE_DX: f64 = 14.4;

pub const SP_HEAD_NOTE_LAB_SZ: f64 = 10.0;

pub const SP_HEAD_NOTE_SZ: f64 = 15.0;

pub const SP_HEAD_CHAR_LAB_DX: f64 = 22.4;

pub const SP_HEAD_CHAR_DX: f64 = 16.4;

pub const SP_HEAD_CHAR_SZ: f64 = 13.0;

pub const SP_ROW_Y0: f64 = 300.0;

pub const SP_ROW_H: f64 = 66.0;

pub const SP_ROW_PITCH: f64 = 82.0;

pub const SP_RULE_Y: f64 = 34.0;

pub const SP_RULE_H: f64 = 2.0;

pub const SP_AXIS_X: f64 = 94.0;

pub const SP_RIGHT_X: f64 = SONG_COL_W - 18.0;

pub const SP_TRI_SIZE: f64 = 7.0;

pub const SP_TRI_PAD: f64 = 3.0;

pub const SP_AV_R: f64 = 25.0;

pub const SP_AV_CX: f64 = 36.0;

pub const SP_AV_CY: f64 = 32.0;

pub const SP_RANK_Y0: f64 = 9.0;

pub const SP_RANK_H: f64 = 16.0;

pub const SP_RANK_W: f64 = 26.0;

pub const SP_RANK_LEAN: f64 = 4.0;

pub const SP_RANK_BASE: f64 = 23.0;

pub const SP_RANK_SIZE: f64 = 10.5;

pub const SP_ROW_NAME_X: f64 = SP_AXIS_X + SP_RANK_W + 6.0;

pub const SP_ROW_NAME_BASE: f64 = 23.0;

pub const SP_ROW_NAME_SIZE: f64 = 14.5;

pub const SP_SCORE_X: f64 = SP_AXIS_X;

pub const SP_SCORE_BASE: f64 = 56.0;

pub const SP_SCORE_SIZE: f64 = 18.0;

pub const SP_JUDGE_BASE: f64 = 54.0;

pub const SP_JUDGE_SIZE: f64 = 11.5;

pub const SP_GRADE_ART_H: f64 = 21.0;

pub const SP_GRADE_ART_RX: f64 = SP_RIGHT_X;

pub const SP_GRADE_ART_CY: f64 = 18.0;

pub const SP_FOOT_Y: f64 = 736.0;

pub const SP_FOOT_H: f64 = 12.0;

pub const SP_FOOT_TEXT_X: f64 = 54.0;

pub const SP_FOOT_BASE: f64 = 746.0;

pub const SP_FOOT_SIZE: f64 = 11.0;

pub const SP_GRID_STEP: f64 = 32.0;

pub const COL_CANVAS: Color = Color::new(0xEC, 0xF1, 0xF6, 0xFF);

pub const COL_WHITE: Color = Color::new(0xFF, 0xFF, 0xFF, 0xFF);

pub const COL_PAPER: Color = Color::new(0xEA, 0xEF, 0xF6, 0xFF);

pub const COL_ACCENT: Color = Color::new(0xE2, 0xE8, 0xF0, 0xFF);

pub const COL_INK: Color = Color::new(0x0F, 0x17, 0x2A, 0xFF);

pub const COL_MUTED: Color = Color::new(0x64, 0x74, 0x8B, 0xFF);

pub const COL_LABEL: Color = Color::new(0x94, 0xA3, 0xB8, 0xFF);

pub const COL_BLUE: Color = Color::new(0x6D, 0xC3, 0xE8, 0xFF);

pub const COL_PLATE_INK: Color = Color::new(0x47, 0x55, 0x69, 0xFF);

pub const COL_RULE: Color = Color::new(0xD2, 0xDB, 0xE7, 0xFF);

pub const COL_TITLE_RULE: Color = Color::new(0xED, 0xF2, 0xF7, 0xFF);

pub const COL_RB1: Color = Color::new(0x0E, 0xA5, 0xE9, 0xFF);

pub const COL_RB2: Color = Color::new(0x10, 0xB9, 0x81, 0xFF);

pub const COL_RB3: Color = Color::new(0x06, 0xB6, 0xD4, 0xFF);

pub const COL_GHOST: Color = Color::new(0xCB, 0xD5, 0xE1, 0xFF);

pub const COL_GRID: Color = Color::new(0xE1, 0xE8, 0xF2, 100);

pub const SP_AVATAR_BORDER: &str = "avatar/char_icon_border.png";

pub const SP_LOGO: &str = "logo/arcaea_logo.png";

pub const SP_KICKER: &str = "SONG PROFILE";

pub const SP_SIGN: &str = "GENERATED BY OKAMIKKO";

pub const SP_BRAND: &str = "ARCAEA // SONG PROFILE";

pub const SP_NONE: &str = "—";

pub const SP_UNKNOWN: &str = "?";
pub const SP_SEAM_W: f64 = 1.2;

pub const SP_AV_RING_W: f64 = 1.4;

pub const SP_LOGO_MARK: [f32; 4] = [279.0, 28.0, 980.0, 224.0];

pub const SP_PTT_FILL: Color = Color::new(246, 249, 255, 255);

pub const SP_PTT_EDGE: Color = Color::new(0, 0, 0, 255);

pub const SP_INK_PAPER: u8 = 211;

pub const SP_INK_WEIGHT: [(f32, u8); 5] = [
    (0.0, 115),
    (0.25, 77),
    (0.5, 46),
    (0.75, 23),
    (1.0, 7),
];

pub const SP_FIT_MIN_SIZE: f64 = 10.5;

pub fn f(v: f64) -> f32 {
    (v * SONG_SCALE) as f32
}

pub fn fi(v: f64) -> i32 {
    (v * SONG_SCALE).round() as i32
}

pub fn fsz(v: f64) -> u32 {
    let size = (v * SONG_SCALE).round();
    if size < 1.0 {
        1
    } else {
        size as u32
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SongTone {
    pub main: Color,
    pub tint: Color,
    pub dark: Color,
}

pub const TONE_PST: SongTone = SongTone {
    main: Color::new(0xBC, 0xE6, 0xFF, 0xFF),
    tint: Color::new(0xE7, 0xF7, 0xFE, 0xFF),
    dark: Color::new(0x1E, 0x5A, 0x8A, 0xFF),
};

pub const TONE_PRS: SongTone = SongTone {
    main: Color::new(0xB8, 0xF0, 0xD4, 0xFF),
    tint: Color::new(0xE7, 0xFA, 0xF3, 0xFF),
    dark: Color::new(0x1B, 0x6B, 0x45, 0xFF),
};

pub const TONE_FTR: SongTone = SongTone {
    main: Color::new(0xDF, 0xC8, 0xFA, 0xFF),
    tint: Color::new(0xF4, 0xF1, 0xFE, 0xFF),
    dark: Color::new(0x4E, 0x2A, 0x7A, 0xFF),
};

pub const TONE_BYD: SongTone = SongTone {
    main: Color::new(0xFD, 0xBF, 0xC8, 0xFF),
    tint: Color::new(0xFF, 0xEE, 0xF0, 0xFF),
    dark: Color::new(0x9C, 0x40, 0x50, 0xFF),
};

pub const TONE_ETR: SongTone = SongTone {
    main: Color::new(0xD2, 0xD8, 0xE6, 0xFF),
    tint: Color::new(0xED, 0xF1, 0xF6, 0xFF),
    dark: Color::new(0x42, 0x4B, 0x5E, 0xFF),
};

pub const TONE_INS: SongTone = SongTone {
    main: Color::new(0xB8, 0xC2, 0xF5, 0xFF),
    tint: Color::new(0xE8, 0xEC, 0xFB, 0xFF),
    dark: Color::new(0x2E, 0x3F, 0x8F, 0xFF),
};

pub fn tone_of(difficulty: Difficulty) -> SongTone {
    match difficulty {
        Difficulty::Past => TONE_PST,
        Difficulty::Present => TONE_PRS,
        Difficulty::Future => TONE_FTR,
        Difficulty::Beyond => TONE_BYD,
        Difficulty::Eternal => TONE_ETR,
        Difficulty::Ins => TONE_INS,
    }
}

pub fn tri_tone(difficulty: Difficulty, index: usize) -> Color {
    if index >= 3 {
        return COL_GHOST;
    }
    match difficulty {
        Difficulty::Past => Color::new(0x7D, 0xD3, 0xFC, 0xFF),
        Difficulty::Present => Color::new(0x6E, 0xE7, 0xB7, 0xFF),
        _ => Color::new(0xC4, 0xB5, 0xFD, 0xFF),
    }
}

pub fn rating_accent(tier: i32) -> Color {
    match tier {
        0 => Color::new(120, 132, 156, 190),
        1 => Color::new(132, 168, 208, 200),
        2 => Color::new(112, 196, 190, 205),
        3 => Color::new(126, 214, 168, 210),
        4 => Color::new(150, 208, 130, 210),
        5 => Color::new(196, 190, 120, 215),
        6 => Color::new(214, 158, 118, 220),
        7 => Color::new(222, 126, 150, 225),
        8 => Color::new(200, 130, 226, 230),
        _ => Color::new(150, 170, 220, 200),
    }
}

pub fn grade_file(grade: &str) -> &'static str {
    crate::domain::grade_file_name_for_label(grade)
}

pub fn grade_ink_box(file: &str) -> Option<(f32, f32, f32, f32)> {
    match file {
        "EX+.png" => Some((52.0, 3.0, 226.0, 108.0)),
        "EX.png" => Some((7.0, 7.0, 142.0, 120.0)),
        "AA.png" => Some((36.0, 7.0, 195.0, 106.0)),
        "A.png" => Some((75.0, 7.0, 154.0, 106.0)),
        "B.png" => Some((86.0, 7.0, 153.0, 106.0)),
        "C.png" => Some((66.0, 6.0, 153.0, 106.0)),
        "D.png" => Some((77.0, 7.0, 157.0, 106.0)),
        _ => None,
    }
}

pub fn label_en(label: &str) -> &str {
    match label.trim() {
        "BPM" => "BPM",
        "上架" | "上架日期" | "RELEASE DATE" => "RELEASE",
        "曲目 ID" | "歌曲 ID" => "SONG ID",
        "物量合计" | "TOTAL NOTES" => "TOTAL NOTES",
        other => other,
    }
}

pub fn split_song_meta(note: &str) -> Option<(String, String)> {
    let trimmed = note.trim();
    let (artist, pack) = trimmed.split_once(" · ")?;
    if artist.is_empty() || pack.is_empty() {
        return None;
    }
    Some((artist.to_string(), pack.to_string()))
}

pub fn columns_for(charts: usize) -> usize {
    match charts {
        0..=2 => 3,
        3 => 3,
        4 => 4,
        _ => 5,
    }
}

pub fn canvas_w(columns: usize) -> f64 {
    let n = columns.max(1) as f64;
    SONG_MARGIN * 2.0 + n * SONG_COL_W + (n - 1.0) * SONG_COL_GAP
}

pub fn width_for(charts: usize) -> i32 {
    fi(canvas_w(columns_for(charts)))
}

pub fn height() -> i32 {
    fi(SONG_BASE_H)
}

pub fn col_x(index: usize) -> f64 {
    SONG_MARGIN + index as f64 * (SONG_COL_W + SONG_COL_GAP)
}

pub fn bar_cell_w(right: f64) -> f64 {
    (right - SP_BAR_T0) / SP_BAR_CELLS as f64
}

pub fn bar_cell_x(k: usize, right: f64) -> f64 {
    SP_BAR_T0 + k as f64 * bar_cell_w(right)
}

pub fn bar_slope() -> f64 {
    SP_BAR_SEP_LEAN / (SP_BAR_BOT - SP_BAR_Y)
}

pub fn bar_text_x(k: usize, right: f64, baseline: f64, dx: f64) -> f64 {
    bar_cell_x(k, right) - (baseline - SP_BAR_Y) * bar_slope() + dx
}

pub fn head_slope() -> f64 {
    (SP_HEAD_LT1 - SP_HEAD_LB1) / (SP_HEAD_BOT - SP_HEAD_Y)
}

pub fn head_text_x(edge_x: f64, edge_y: f64, baseline: f64, dx: f64) -> f64 {
    edge_x - (baseline - edge_y) * head_slope() + dx
}

pub fn row_y(index: usize) -> f64 {
    SP_ROW_Y0 + index as f64 * SP_ROW_PITCH
}

pub fn difficulty_suffix(difficulty: Difficulty) -> &'static str {
    match difficulty {
        Difficulty::Past => "pst",
        Difficulty::Present => "prs",
        Difficulty::Future => "ftr",
        Difficulty::Beyond => "byd",
        Difficulty::Eternal => "etr",
        Difficulty::Ins => "ins",
    }
}

fn split_key(key: &str) -> (&str, &str) {
    match key.rsplit_once('/') {
        Some((dir, name)) => (dir, name),
        None => ("", key),
    }
}

fn stem_of(file: &str) -> (&str, &str) {
    match file.rsplit_once('.') {
        Some((stem, ext)) => (stem, ext),
        None => (file, "jpg"),
    }
}

fn join_key(dir: &str, name: &str) -> String {
    if dir.is_empty() {
        name.to_string()
    } else {
        format!("{dir}/{name}")
    }
}

pub fn jacket_keys_for(jacket: Option<&str>, difficulty: Option<Difficulty>) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    let Some(raw) = jacket else {
        return keys;
    };
    let key = raw.trim();
    if key.is_empty() {
        return keys;
    }
    let (dir, file) = split_key(key);
    let (stem, ext) = stem_of(file);
    if let Some(difficulty) = difficulty {
        let suffix = difficulty_suffix(difficulty);
        keys.push(join_key(dir, &format!("{stem}_{suffix}.{ext}")));
        if dir == "jackets" {
            keys.push(format!("jacket_thumbs/{stem}_{suffix}.{ext}"));
        }
    }
    keys.push(key.to_string());
    if dir == "jackets" {
        keys.push(format!("jacket_thumbs/{file}"));
    }
    keys
}

pub fn jacket_keys(jacket: Option<&str>) -> Vec<String> {
    jacket_keys_for(jacket, None)
}

pub fn duration_text(seconds: i64) -> String {
    if seconds <= 0 {
        return String::from("--:--");
    }
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

pub fn pad2(value: usize) -> String {
    format!("{value:02}")
}

pub fn constant_text(chart: &Chart) -> String {
    if !chart.has_constant || chart.constant <= 0.0 {
        return String::from(SP_UNKNOWN);
    }
    format!("{:.1}", chart.constant)
}

pub fn note_text(note: i64) -> String {
    if note <= 0 {
        return String::from(SP_NONE);
    }
    note.to_string()
}

pub fn live_levels(charts: &[Chart]) -> Vec<Chart> {
    let find = |difficulty: Difficulty| {
        charts
            .iter()
            .find(|chart| chart.difficulty == difficulty)
            .cloned()
    };
    let mut out: Vec<Chart> = Vec::with_capacity(5);
    for difficulty in [Difficulty::Past, Difficulty::Present, Difficulty::Future] {
        out.push(find(difficulty).unwrap_or_else(|| Chart::new(difficulty, "", 0.0)));
    }
    for difficulty in [Difficulty::Eternal, Difficulty::Beyond] {
        if let Some(chart) = find(difficulty) {
            out.push(chart);
        }
    }
    out
}

#[derive(Debug, Clone, PartialEq)]
pub struct SongBoardRow {
    pub name: String,
    pub avatar: Option<String>,
    pub grade: String,
    pub score: i64,
    pub rating: f64,
    pub has_rating: bool,
    pub rating_frame: i32,
    pub rating_accent: i32,
    pub perfect: i32,
    pub normal_perfect: i32,
    pub far: i32,
    pub lost: i32,
    pub judgement: bool,
}

impl Default for SongBoardRow {
    fn default() -> Self {
        Self {
            name: String::new(),
            avatar: None,
            grade: String::new(),
            score: 0,
            rating: 0.0,
            has_rating: false,
            rating_frame: 0,
            rating_accent: 0,
            perfect: 0,
            normal_perfect: 0,
            far: 0,
            lost: 0,
            judgement: false,
        }
    }
}

impl SongBoardRow {
    pub fn new(name: impl Into<String>, score: i64) -> Self {
        Self {
            name: name.into(),
            score,
            ..Self::default()
        }
    }

    pub fn with_avatar(mut self, key: impl Into<String>) -> Self {
        let key = key.into();
        self.avatar = (!key.trim().is_empty()).then_some(key);
        self
    }

    pub fn with_grade(mut self, grade: impl Into<String>) -> Self {
        self.grade = grade.into();
        self
    }

    pub fn with_rating(mut self, rating: f64, frame: i32, accent: i32) -> Self {
        self.rating = rating;
        self.has_rating = rating > 0.0;
        self.rating_frame = frame;
        self.rating_accent = accent;
        self
    }

    pub fn with_judgement(
        mut self,
        perfect: i32,
        normal_perfect: i32,
        far: i32,
        lost: i32,
    ) -> Self {
        self.perfect = perfect;
        self.normal_perfect = normal_perfect;
        self.far = far;
        self.lost = lost;
        self.judgement = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SongBoard {
    pub difficulty: Difficulty,
    pub rows: Vec<SongBoardRow>,
}

impl SongBoard {
    pub fn new(difficulty: Difficulty, rows: Vec<SongBoardRow>) -> Self {
        Self { difficulty, rows }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SongColumn {
    pub difficulty: Difficulty,
    pub has: bool,
    pub x: f64,
    pub tag: String,
    pub level: String,
    pub constant: String,
    pub note: String,
    pub designer: String,
}

impl SongColumn {
    fn placeholder(difficulty: Difficulty, x: f64) -> Self {
        Self {
            difficulty,
            has: false,
            x,
            tag: String::from(SP_UNKNOWN),
            level: String::new(),
            constant: String::from(SP_UNKNOWN),
            note: String::from(SP_NONE),
            designer: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SongPlan {
    pub w: f64,
    pub h: f64,
    pub right: f64,
    pub cell_w: f64,
    pub columns: Vec<SongColumn>,
}

impl SongPlan {
    pub fn n(&self) -> usize {
        self.columns.len().max(1)
    }

    pub fn canvas(&self) -> (i32, i32) {
        (fi(self.w), fi(self.h))
    }

    pub fn rows(&self, index: usize) -> Vec<SongBoardRow> {
        self.columns
            .get(index)
            .map(|_| Vec::new())
            .unwrap_or_default()
    }
}

pub fn plan_for(input: &SongInput) -> SongPlan {
    let live = live_levels(&input.charts);
    let n = live.len().max(1);
    let w = canvas_w(n);
    let right = w - SONG_MARGIN;
    let mut columns = Vec::with_capacity(n);
    for index in 0..n {
        let x = col_x(index);
        match live.get(index) {
            Some(chart) => columns.push(SongColumn {
                difficulty: chart.difficulty,
                has: true,
                x,
                tag: chart.difficulty.as_str().to_string(),
                level: chart.level.trim().to_string(),
                constant: constant_text(chart),
                note: note_text(chart.note),
                designer: chart.designer.trim().to_string(),
            }),
            None => columns.push(SongColumn::placeholder(Difficulty::Future, x)),
        }
    }
    SongPlan {
        w,
        h: SONG_BASE_H,
        right,
        cell_w: bar_cell_w(right),
        columns,
    }
}

#[derive(Debug, Clone)]
pub struct SongInput {
    pub song_id: String,
    pub title: String,
    pub title_ja: String,
    pub aliases: Vec<String>,
    pub artist: String,
    pub bpm: f64,
    pub bpm_label: Option<String>,
    pub release: String,
    pub duration: i64,
    pub pack_id: String,
    pub pack_name: String,
    pub pack_name_zh: String,
    pub jacket: Option<String>,
    pub charts: Vec<Chart>,
    pub ratings: Vec<(Difficulty, crate::model::Grade)>,
    pub theme: theme::Theme,
    pub footer_note: String,
    pub cells: Option<Vec<(String, String)>>,
    pub boards: Option<Vec<SongBoard>>,
}

impl SongInput {
    pub fn new(song: &Song) -> Self {
        Self {
            song_id: song.id.clone(),
            title: song.title.clone(),
            title_ja: String::new(),
            aliases: song.aliases.clone(),
            artist: song.artist.clone(),
            bpm: song.bpm,
            bpm_label: None,
            release: String::new(),
            duration: 0,
            pack_id: song.pack_id.clone(),
            pack_name: song.pack_name.clone(),
            pack_name_zh: song.pack_name_zh.clone(),
            jacket: (!song.id.trim().is_empty()).then(|| format!("jackets/{}.jpg", song.id)),
            charts: song.charts.clone(),
            ratings: Vec::new(),
            theme: theme::Theme::DEFAULT,
            footer_note: String::new(),
            cells: None,
            boards: None,
        }
    }

    pub fn with_jacket(mut self, key: impl Into<String>) -> Self {
        let key = key.into();
        self.jacket = (!key.trim().is_empty()).then_some(key);
        self
    }

    pub fn without_jacket(mut self) -> Self {
        self.jacket = None;
        self
    }

    pub fn with_title_ja(mut self, title: impl Into<String>) -> Self {
        self.title_ja = title.into();
        self
    }

    pub fn with_aliases(mut self, aliases: &[&str]) -> Self {
        self.aliases = aliases.iter().map(|alias| alias.to_string()).collect();
        self
    }

    pub fn with_duration(mut self, seconds: i64) -> Self {
        self.duration = seconds.max(0);
        self
    }

    pub fn with_rating(mut self, difficulty: Difficulty, grade: crate::model::Grade) -> Self {
        self.ratings.retain(|(seen, _)| *seen != difficulty);
        self.ratings.push((difficulty, grade));
        self
    }

    pub fn with_ratings(mut self, ratings: Vec<(Difficulty, crate::model::Grade)>) -> Self {
        self.ratings = ratings;
        self
    }

    pub fn rating(&self, difficulty: Difficulty) -> Option<crate::model::Grade> {
        self.ratings
            .iter()
            .find(|(seen, _)| *seen == difficulty)
            .map(|(_, grade)| *grade)
    }

    pub fn with_theme(mut self, theme_value: theme::Theme) -> Self {
        self.theme = theme_value;
        self
    }

    pub fn with_footer(mut self, note: impl Into<String>) -> Self {
        self.footer_note = note.into();
        self
    }

    pub fn with_cells(mut self, cells: Vec<(String, String)>) -> Self {
        self.cells = Some(cells);
        self
    }

    pub fn with_boards(mut self, boards: Vec<SongBoard>) -> Self {
        self.boards = Some(boards);
        self
    }

    pub fn sorted_charts(&self) -> Vec<Chart> {
        let mut charts = self.charts.clone();
        charts.sort_by_key(|chart| chart.difficulty.rank());
        charts
    }

    pub fn hero_difficulty(&self) -> Option<Difficulty> {
        self.charts
            .iter()
            .map(|chart| chart.difficulty)
            .max_by_key(|d| d.rank())
    }

    pub fn columns(&self) -> usize {
        live_levels(&self.charts).len().max(1)
    }

    pub fn width(&self) -> i32 {
        fi(canvas_w(self.columns()))
    }

    pub fn height(&self) -> i32 {
        height()
    }

    pub fn alias_text(&self) -> String {
        self.aliases
            .iter()
            .map(|alias| alias.trim())
            .filter(|alias| !alias.is_empty())
            .collect::<Vec<&str>>()
            .join(" / ")
    }

    pub fn bpm_text(&self) -> String {
        if let Some(label) = self.bpm_label.as_deref() {
            if !label.trim().is_empty() {
                return label.to_string();
            }
        }
        if self.bpm <= 0.0 {
            String::from("--")
        } else {
            widgets::number(self.bpm, 1)
        }
    }

    pub fn note_total(&self) -> i64 {
        self.charts.iter().map(|chart| chart.note.max(0)).sum()
    }

    pub fn meta_note(&self) -> String {
        let mut parts: Vec<&str> = Vec::with_capacity(2);
        let artist = self.artist.trim();
        if !artist.is_empty() {
            parts.push(artist);
        }
        let pack = self.pack_name.trim();
        if !pack.is_empty() {
            parts.push(pack);
        }
        parts.join(" · ")
    }

    pub fn cells_value(&self) -> Vec<(String, String)> {
        if let Some(cells) = &self.cells {
            return cells.clone();
        }
        let total = self.note_total();
        vec![
            (String::from("BPM"), self.bpm_text()),
            (
                String::from("上架"),
                match self.release.is_empty() {
                    true => String::from(SP_NONE),
                    false => self.release.clone(),
                },
            ),
            (String::from("曲目 ID"), self.song_id.trim().to_string()),
            (
                String::from("物量合计"),
                if total > 0 {
                    sketch::thousands(total)
                } else {
                    String::from(SP_NONE)
                },
            ),
        ]
    }

    pub fn rows_for(&self, difficulty: Difficulty) -> Vec<SongBoardRow> {
        self.boards
            .as_ref()
            .and_then(|boards| {
                boards
                    .iter()
                    .find(|board| board.difficulty == difficulty)
                    .map(|board| board.rows.clone())
            })
            .unwrap_or_default()
    }

    pub fn jacket_candidates(&self) -> Vec<String> {
        jacket_keys_for(self.jacket.as_deref(), self.hero_difficulty())
    }
}

pub fn request(input: &SongInput) -> RenderRequest {
    super::harness::card_request(input.width(), input.height())
        .with_quality(super::harness::SINGLE_CARD_QUALITY)
        .with_scale(1.5)
}

pub fn card(input: SongInput) -> RenderCard {
    let request = request(&input);
    let cache_key = super::memo::fingerprint("song", &input);
    RenderCard::new(request, "song.png", move |kit| paint(kit, &input)).with_cache_key(cache_key)
}

pub fn paint(kit: &CanvasKit<'_>, input: &SongInput) {
    let plan = plan_for(input);
    let (width, height) = plan.canvas();
    paint_backdrop(kit, input, width as f32, height as f32);
    paint_hero(kit, input, &plan);
    paint_columns(kit, input, &plan);
    paint_footer(kit, &plan);
}

fn area(x: f64, y: f64, w: f64, h: f64) -> Rect {
    widgets::rect(f(x), f(y), f(w), f(h))
}

fn style_of(family: &str, size: f64, color: Color) -> TextStyle {
    theme::style(family, fsz(size), color)
}

fn text_w(kit: &CanvasKit<'_>, family: &str, size: f64, value: &str) -> f32 {
    if value.is_empty() {
        return 0.0;
    }
    let style = style_of(family, size, COL_INK);
    sketch::text_width(kit, &style, value)
}

fn text_at(
    kit: &CanvasKit<'_>,
    family: &str,
    size: f64,
    color: Color,
    value: &str,
    x: f32,
    baseline: f32,
) -> f32 {
    if value.is_empty() {
        return 0.0;
    }
    let style = style_of(family, size, color);
    sketch::text(kit, &style, value, x, baseline, color)
}

fn ellipsized(kit: &CanvasKit<'_>, family: &str, size: f64, value: &str, max_w: f32) -> String {
    if value.is_empty() {
        return String::new();
    }
    let style = style_of(family, size, COL_INK);
    sketch::ellipsize(kit, &style, value, max_w)
}

struct Pen<'a> {
    family: &'a str,
    size: f64,
    color: Color,
}

fn tracked_center(
    kit: &CanvasKit<'_>,
    pen: Pen<'_>,
    value: &str,
    center_x: f32,
    baseline: f32,
    track: f64,
) {
    let Pen {
        family,
        size,
        color,
    } = pen;
    if value.is_empty() {
        return;
    }
    let style = style_of(family, size, color);
    let gap = f(track);
    let mut total = 0.0;
    let chars: Vec<char> = value.chars().collect();
    for ch in &chars {
        total += sketch::text_width(kit, &style, &ch.to_string());
    }
    total += gap * (chars.len().saturating_sub(1)) as f32;
    let mut x = center_x - total / 2.0;
    for ch in &chars {
        let glyph = ch.to_string();
        sketch::text(kit, &style, &glyph, x, baseline, color);
        x += sketch::text_width(kit, &style, &glyph) + gap;
    }
}

fn fit_text(kit: &CanvasKit<'_>, pen: Pen<'_>, value: &str, x: f32, baseline: f32, right: f32) {
    let Pen {
        family,
        size,
        color,
    } = pen;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return;
    }
    let mut current = size;
    while current > SP_FIT_MIN_SIZE && x + text_w(kit, family, current, trimmed) > right {
        current = (current - 0.5).max(SP_FIT_MIN_SIZE);
    }
    let avail = right - x;
    let out = if avail > 0.0 && text_w(kit, family, current, trimmed) > avail {
        ellipsized(kit, family, current, trimmed, avail)
    } else {
        trimmed.to_string()
    };
    text_at(kit, family, current, color, &out, x, baseline);
}

fn poly(points: &[(f64, f64)]) -> Vec<(f32, f32)> {
    points.iter().map(|(x, y)| (f(*x), f(*y))).collect()
}

fn clip_poly<R>(kit: &CanvasKit<'_>, points: &[(f64, f64)], body: impl FnOnce() -> R) -> R {
    let scaled = poly(points);
    if scaled.len() < 3 {
        return body();
    }
    let mut builder = PathBuilder::new();
    builder.move_to(scaled[0]);
    for point in &scaled[1..] {
        builder.line_to(*point);
    }
    builder.close();
    let path = builder.detach();
    let canvas = kit.canvas();
    canvas.save();
    canvas.clip_path(&path, ClipOp::Intersect, true);
    let out = body();
    canvas.restore();
    out
}

fn paint_backdrop(kit: &CanvasKit<'_>, input: &SongInput, width: f32, height: f32) {
    let full = widgets::rect(0.0, 0.0, width, height);
    let w = width.max(1.0) as usize;
    let h = height.max(1.0) as usize;
    let mut buf = vec![0u8; w * h * 4];
    ink_fill_paper(&mut buf);
    for key in input.jacket_candidates() {
        let Some((source, sw, sh)) = asset_pixels(kit, &key) else {
            continue;
        };
        let mut layer = ink_stretch(&source, sw, sh, w, h);
        ink_blur(&mut layer, w, h, 2);
        ink_blur(&mut layer, w, h, 2);
        ink_art(&mut buf, &layer, w, h);
        break;
    }
    ink_draw(kit, &buf, w, h, full);
    paint_print_grid(kit, width, height);
}

fn ink_clamp(value: f64) -> u8 {
    if value <= 0.0 {
        0
    } else if value >= 255.0 {
        255
    } else {
        (value + 0.5) as u8
    }
}

fn ink_fill_paper(buf: &mut [u8]) {
    let mut i = 0;
    while i + 3 < buf.len() {
        buf[i] = 0xEA;
        buf[i + 1] = 0xEF;
        buf[i + 2] = 0xF6;
        buf[i + 3] = 255;
        i += 4;
    }
}

fn ink_level(paper: f64, mix: f64, t: f64) -> u8 {
    let ink = (0.76 + mix * 0.17) * 255.0;
    ink_clamp(paper * (1.0 - t) + ink * t)
}

fn asset_pixels(kit: &CanvasKit<'_>, key: &str) -> Option<(Vec<u8>, usize, usize)> {
    let texture = kit.asset(key).ok()?;
    let image = texture.image()?;
    let width = image.width();
    let height = image.height();
    if width <= 0 || height <= 0 {
        return None;
    }
    let info = aurorix_render::skia_safe::ImageInfo::new(
        (width, height),
        aurorix_render::skia_safe::ColorType::RGBA8888,
        aurorix_render::skia_safe::AlphaType::Unpremul,
        None,
    );
    let stride = width as usize * 4;
    let mut data = vec![0u8; stride * height as usize];
    let hint = aurorix_render::skia_safe::image::CachingHint::Allow;
    if !image.read_pixels(&info, &mut data, stride, (0, 0), hint) {
        return None;
    }
    Some((data, width as usize, height as usize))
}

fn ink_stretch(src: &[u8], sw: usize, sh: usize, dw: usize, dh: usize) -> Vec<u8> {
    let mut out = vec![0u8; dw * dh * 4];
    if sw == 0 || sh == 0 || dw == 0 || dh == 0 {
        return out;
    }
    let sx = sw as f64 / dw as f64;
    let sy = sh as f64 / dh as f64;
    for y in 0..dh {
        let fy = ((y as f64 + 0.5) * sy - 0.5).max(0.0);
        let y0 = (fy.floor() as usize).min(sh - 1);
        let y1 = (y0 + 1).min(sh - 1);
        let ty = fy - y0 as f64;
        for x in 0..dw {
            let fx = ((x as f64 + 0.5) * sx - 0.5).max(0.0);
            let x0 = (fx.floor() as usize).min(sw - 1);
            let x1 = (x0 + 1).min(sw - 1);
            let tx = fx - x0 as f64;
            let i00 = (y0 * sw + x0) * 4;
            let i10 = (y0 * sw + x1) * 4;
            let i01 = (y1 * sw + x0) * 4;
            let i11 = (y1 * sw + x1) * 4;
            let o = (y * dw + x) * 4;
            for channel in 0..3 {
                let top = f64::from(src[i00 + channel]) * (1.0 - tx)
                    + f64::from(src[i10 + channel]) * tx;
                let bot = f64::from(src[i01 + channel]) * (1.0 - tx)
                    + f64::from(src[i11 + channel]) * tx;
                out[o + channel] = ink_clamp(top * (1.0 - ty) + bot * ty);
            }
            out[o + 3] = 255;
        }
    }
    out
}

fn ink_blur(px: &mut [u8], w: usize, h: usize, radius: usize) {
    if radius == 0 || w == 0 || h == 0 {
        return;
    }
    for vertical in [true, false] {
        let rad = radius + 1;
        let (outer, inner) = if vertical { (w, h) } else { (h, w) };
        let top = radius.min(inner - 1);
        let mut ring = vec![[0u8; 3]; rad];
        for o in 0..outer {
            let at_of = |i: usize| -> usize {
                if vertical {
                    (i * w + o) * 4
                } else {
                    (o * w + i) * 4
                }
            };
            let mut sums = [0i64; 3];
            let mut n = 0i64;
            for i in 0..=top {
                let at = at_of(i);
                for channel in 0..3 {
                    sums[channel] += i64::from(px[at + channel]);
                }
                n += 1;
            }
            for i in 0..inner {
                let at = at_of(i);
                ring[i % rad] = [px[at], px[at + 1], px[at + 2]];
                for channel in 0..3 {
                    px[at + channel] = (sums[channel] / n) as u8;
                }
                if i >= radius {
                    let back = ring[(i - radius) % rad];
                    for channel in 0..3 {
                        sums[channel] -= i64::from(back[channel]);
                    }
                    n -= 1;
                }
                if i + radius + 1 < inner {
                    let next = at_of(i + radius + 1);
                    for channel in 0..3 {
                        sums[channel] += i64::from(px[next + channel]);
                    }
                    n += 1;
                }
            }
        }
    }
}

fn ink_art(dst: &mut [u8], layer: &[u8], w: usize, h: usize) {
    let pr = f64::from(0xEA_u8);
    let pg = f64::from(0xEF_u8);
    let pb = f64::from(0xF6_u8);
    for y in 0..h {
        let ny = y as f64 / h as f64;
        let vy = 1.0 - ny * 0.85;
        for x in 0..w {
            let nx = x as f64 / w as f64;
            let mut tt = ((0.4 + nx * 0.6) * vy).clamp(0.0, 1.0);
            tt *= 0.45;
            let i = (y * w + x) * 4;
            let r = f64::from(layer[i]) / 255.0;
            let g = f64::from(layer[i + 1]) / 255.0;
            let b = f64::from(layer[i + 2]) / 255.0;
            let lum = 0.299 * r + 0.587 * g + 0.114 * b;
            dst[i] = ink_level(pr, lum * 0.65 + r * 0.35, tt);
            dst[i + 1] = ink_level(pg, lum * 0.65 + g * 0.35, tt);
            dst[i + 2] = ink_level(pb, lum * 0.65 + b * 0.35, tt);
        }
    }
}

fn ink_draw(kit: &CanvasKit<'_>, buf: &[u8], w: usize, h: usize, area: Rect) {
    let info = aurorix_render::skia_safe::ImageInfo::new(
        (w as i32, h as i32),
        aurorix_render::skia_safe::ColorType::RGBA8888,
        aurorix_render::skia_safe::AlphaType::Unpremul,
        None,
    );
    let data = aurorix_render::skia_safe::Data::new_copy(buf);
    let Some(image) = aurorix_render::skia_safe::images::raster_from_data(&info, data, w * 4) else {
        return;
    };
    kit.image_rect(&image, area);
}

fn paint_print_grid(kit: &CanvasKit<'_>, width: f32, height: f32) {
    let step = f(SP_GRID_STEP);
    if step <= 0.0 {
        return;
    }
    let mut x = 0.0;
    while x < width {
        kit.fill_rect(widgets::rect(x, 0.0, 1.0, height), COL_GRID);
        x += step;
    }
    let mut y = 0.0;
    while y < height {
        kit.fill_rect(widgets::rect(0.0, y, width, 1.0), COL_GRID);
        y += step;
    }
}

fn paint_hero(kit: &CanvasKit<'_>, input: &SongInput, plan: &SongPlan) {
    paint_jacket(kit, input);
    paint_band(kit);
    paint_title_cabin(kit, input, plan);
    paint_brand(kit, plan);
    paint_data_bar(kit, input, plan);
}

fn paint_jacket(kit: &CanvasKit<'_>, input: &SongInput) {
    let shape = [
        (SP_JACKET_L, SP_HERO_TOP),
        (SP_JACKET_T1, SP_HERO_TOP),
        (SP_JACKET_B1, SP_HERO_BOT),
        (SP_JACKET_L, SP_HERO_BOT),
    ];
    sketch::fill_polygon(kit, &poly(&shape), COL_WHITE);
    let mat = SP_JACKET_MAT;
    let inset = [
        (SP_JACKET_L + mat, SP_HERO_TOP + mat),
        (SP_JACKET_T1 - mat, SP_HERO_TOP + mat),
        (SP_JACKET_B1 - mat, SP_HERO_BOT - mat),
        (SP_JACKET_L + mat, SP_HERO_BOT - mat),
    ];
    let side = SP_JACKET_IMG;
    let cx = (SP_JACKET_L + SP_JACKET_T1) / 2.0;
    let cy = (SP_HERO_TOP + SP_HERO_BOT) / 2.0;
    let window = area(cx - side / 2.0, cy - side / 2.0, side, side);
    let keys = input.jacket_candidates();
    if keys.is_empty() {
        return;
    }
    clip_poly(kit, &inset, || {
        for key in &keys {
            if kit.draw_asset_cover(key, window, 0.0).is_ok() {
                break;
            }
        }
    });
}

fn paint_band(kit: &CanvasKit<'_>) {
    let band = [
        (SP_BAND_T0, SP_HERO_TOP),
        (SP_BAND_T1, SP_HERO_TOP),
        (SP_BAND_B1, SP_HERO_BOT),
        (SP_BAND_B0, SP_HERO_BOT),
    ];
    sketch::fill_polygon(kit, &poly(&band), COL_BLUE);
}

fn paint_title_cabin(kit: &CanvasKit<'_>, input: &SongInput, plan: &SongPlan) {
    let cabin = [
        (SP_TITLE_T0, SP_HERO_TOP),
        (plan.right + SP_TITLE_TOP_X, SP_HERO_TOP),
        (plan.right + SP_TITLE_BOT_X, SP_TITLE_BOT),
        (SP_TITLE_B0, SP_TITLE_BOT),
    ];
    sketch::fill_polygon(kit, &poly(&cabin), COL_WHITE);

    let limit = f(plan.right + SP_TITLE_TOP_X - 20.0);
    let name_x = f(SP_NAME_X);
    let head = if input.title.trim().is_empty() {
        String::from(SP_NONE)
    } else {
        input.title.trim().to_string()
    };
    let name = ellipsized(
        kit,
        theme::FONT_TITLE,
        SP_NAME_SIZE,
        &head,
        (limit - name_x).max(1.0),
    );
    text_at(
        kit,
        theme::FONT_TITLE,
        SP_NAME_SIZE,
        COL_INK,
        &name,
        name_x,
        f(SP_NAME_BASE),
    );

    let lean = (SP_ARTIST_MARK_Y1 - SP_ARTIST_MARK_Y0) * 0.2;
    let mark = [
        (SP_ARTIST_MARK_X, SP_ARTIST_MARK_Y0),
        (SP_ARTIST_MARK_X + SP_ARTIST_MARK_W, SP_ARTIST_MARK_Y0),
        (
            SP_ARTIST_MARK_X + SP_ARTIST_MARK_W - lean,
            SP_ARTIST_MARK_Y1,
        ),
        (SP_ARTIST_MARK_X - lean, SP_ARTIST_MARK_Y1),
    ];
    sketch::fill_polygon(kit, &poly(&mark), COL_BLUE);

    let note = input.meta_note();
    let mut meta_x = f(SP_ARTIST_X);
    match split_song_meta(&note) {
        Some((artist, pack)) => {
            let drawn = text_at(
                kit,
                theme::FONT_LATIN,
                SP_ARTIST_SIZE,
                COL_MUTED,
                &artist,
                meta_x,
                f(SP_ARTIST_BASE),
            );
            meta_x += drawn + f(8.0);
            let rest = format!("— {pack}");
            text_at(
                kit,
                theme::FONT_LATIN,
                SP_ARTIST_SIZE,
                COL_MUTED,
                &rest,
                meta_x,
                f(SP_ARTIST_BASE),
            );
        }
        None => {
            if !note.trim().is_empty() {
                let shown = ellipsized(
                    kit,
                    theme::FONT_LATIN,
                    SP_ARTIST_SIZE,
                    note.trim(),
                    (limit - meta_x).max(1.0),
                );
                text_at(
                    kit,
                    theme::FONT_LATIN,
                    SP_ARTIST_SIZE,
                    COL_MUTED,
                    &shown,
                    meta_x,
                    f(SP_ARTIST_BASE),
                );
            }
        }
    }

    let rule_x = SP_TITLE_T0 + SP_TITLE_RULE_INSET;
    let rule_w = plan.right + SP_TITLE_BOT_X - SP_TITLE_T0 - SP_TITLE_RULE_INSET * 2.0;
    if rule_w > 0.0 {
        kit.fill_rect(
            area(rule_x, SP_TITLE_BOT - 2.0, rule_w, 1.0),
            COL_TITLE_RULE,
        );
    }

    let alias = input.alias_text();
    if !alias.trim().is_empty() {
        let alias_x = f(SP_ALIAS_X);
        let text = format!("ALIAS: {}", alias.trim());
        let shown = ellipsized(
            kit,
            theme::FONT_LATIN,
            SP_ALIAS_SIZE,
            &text,
            (f(plan.right + SP_TITLE_BOT_X - 20.0) - alias_x).max(1.0),
        );
        text_at(
            kit,
            theme::FONT_LATIN,
            SP_ALIAS_SIZE,
            COL_LABEL,
            &shown,
            alias_x,
            f(SP_ALIAS_BASE),
        );
    }
}

fn paint_brand(kit: &CanvasKit<'_>, plan: &SongPlan) {
    let right = plan.right;
    let logo_cabin = [
        (right + SP_LOGO_TOP_X, SP_HERO_TOP),
        (right, SP_HERO_TOP),
        (right, SP_LOGO_BOT),
        (right + SP_LOGO_BOT_X, SP_LOGO_BOT),
    ];
    sketch::fill_polygon(kit, &poly(&logo_cabin), COL_WHITE);
    let plate = [
        (right + SP_LOGO_BOT_X, SP_LOGO_BOT),
        (right, SP_LOGO_BOT),
        (right, SP_TITLE_BOT),
        (right + SP_PLATE_BOT_X, SP_TITLE_BOT),
    ];
    sketch::fill_polygon(kit, &poly(&plate), COL_ACCENT);

    paint_logo(kit, right + SP_LOGO_CX, SP_LOGO_CY, SP_LOGO_W);
    tracked_center(
        kit,
        Pen {
            family: theme::FONT_TITLE,
            size: SP_KICK_SIZE,
            color: COL_PLATE_INK,
        },
        SP_KICKER,
        f(right + SP_KICK_CX),
        f(SP_KICK_BASE),
        SP_KICK_TRACK,
    );
}

fn paint_logo(kit: &CanvasKit<'_>, cx: f64, cy: f64, w: f64) {
    brand::draw_logo(kit, f(cx), f(cy), f(w));
}

fn paint_data_bar(kit: &CanvasKit<'_>, input: &SongInput, plan: &SongPlan) {
    let bar = [
        (SP_BAR_T0, SP_BAR_Y),
        (plan.right, SP_BAR_Y),
        (plan.right, SP_BAR_BOT),
        (SP_BAR_B0, SP_BAR_BOT),
    ];
    sketch::fill_polygon(kit, &poly(&bar), COL_WHITE);

    let inset = 12.0;
    let slope = bar_slope();
    for k in 1..SP_BAR_CELLS {
        let x = bar_cell_x(k, plan.right) + SP_BAR_SEP_W / 2.0;
        let y0 = SP_BAR_Y + inset;
        let y1 = SP_BAR_BOT - inset;
        let xt = x + inset * slope;
        let lean = slope * (y1 - y0);
        let seam = [
            (xt, y0),
            (xt + SP_HEAD_SEAM_W, y0),
            (xt + SP_HEAD_SEAM_W - lean, y1),
            (xt - lean, y1),
        ];
        sketch::fill_polygon(kit, &poly(&seam), COL_ACCENT);
    }

    let cells = input.cells_value();
    let value_limit = f(plan.cell_w - SP_BAR_SEP_LEAN - 16.0);
    for k in 0..SP_BAR_CELLS {
        let (label, value) = cells
            .get(k)
            .map(|(label, value)| (label_en(label).to_string(), value.clone()))
            .unwrap_or_else(|| (String::new(), String::from(SP_NONE)));
        let label = if label.trim().is_empty() {
            String::new()
        } else {
            label
        };
        let label_x = f(bar_text_x(k, plan.right, SP_BAR_LAB_Y, SP_BAR_LAB_DX));
        text_at(
            kit,
            theme::FONT_TITLE,
            SP_BAR_LAB_SZ,
            COL_BLUE,
            &label,
            label_x,
            f(SP_BAR_LAB_Y),
        );
        let value_x = f(bar_text_x(k, plan.right, SP_BAR_VAL_Y, SP_BAR_VAL_DX));
        let shown = if value.trim().is_empty() {
            String::from(SP_NONE)
        } else {
            ellipsized(
                kit,
                theme::FONT_TITLE,
                SP_BAR_VAL_SZ,
                value.trim(),
                value_limit.max(1.0),
            )
        };
        text_at(
            kit,
            theme::FONT_TITLE,
            SP_BAR_VAL_SZ,
            COL_INK,
            &shown,
            value_x,
            f(SP_BAR_VAL_Y),
        );
    }
}

fn paint_columns(kit: &CanvasKit<'_>, input: &SongInput, plan: &SongPlan) {
    for column in &plan.columns {
        if !column.has {
            continue;
        }
        paint_col_blade(kit, column);
        let rows = input.rows_for(column.difficulty);
        for index in 0..SONG_ROW_N {
            let row = rows.get(index);
            paint_row(kit, column, index, row);
        }
    }
}

fn paint_col_blade(kit: &CanvasKit<'_>, column: &SongColumn) {
    let x = column.x;
    let tone = tone_of(column.difficulty);
    let left = [
        (x, SP_HEAD_Y),
        (x + SP_HEAD_LT1, SP_HEAD_Y),
        (x + SP_HEAD_LB1, SP_HEAD_BOT),
        (x, SP_HEAD_BOT),
    ];
    sketch::fill_polygon(kit, &poly(&left), tone.main);
    let white = [
        (x + SP_HEAD_RT0, SP_HEAD_Y),
        (x + SONG_COL_W, SP_HEAD_Y),
        (x + SONG_COL_W, SP_HEAD_BOT),
        (x + SP_HEAD_RB0, SP_HEAD_BOT),
    ];
    sketch::fill_polygon(kit, &poly(&white), COL_WHITE);

    let inset = 10.0;
    let y0 = SP_HEAD_SEAM_Y0 + inset;
    let y1 = SP_HEAD_SEAM_Y1 - inset;
    let slope = (SP_HEAD_SEAM_B - SP_HEAD_SEAM_T) / (SP_HEAD_SEAM_Y1 - SP_HEAD_SEAM_Y0);
    let xt = x + SP_HEAD_SEAM_T + inset * slope;
    let lean = slope * (y1 - y0);
    let seam = [
        (xt, y0),
        (xt + SP_HEAD_SEAM_W, y0),
        (xt + SP_HEAD_SEAM_W + lean, y1),
        (xt + lean, y1),
    ];
    sketch::fill_polygon(kit, &poly(&seam), COL_ACCENT);

    text_at(
        kit,
        theme::FONT_TITLE,
        SP_HEAD_TAG_SIZE,
        tone.dark,
        &column.tag,
        f(x + SP_HEAD_TAG_X),
        f(SP_HEAD_TAG_BASE),
    );
    if !column.level.is_empty() {
        text_at(
            kit,
            theme::FONT_TITLE,
            SP_HEAD_LV_SIZE,
            tone.dark,
            &column.level,
            f(x + SP_HEAD_LV_X),
            f(SP_HEAD_LV_BASE),
        );
    }
    text_at(
        kit,
        theme::FONT_TITLE,
        SP_HEAD_CONST_SIZE,
        tone.dark,
        &column.constant,
        f(x + SP_HEAD_CONST_X),
        f(SP_HEAD_CONST_BASE),
    );

    text_at(
        kit,
        theme::FONT_TITLE,
        SP_HEAD_NOTE_LAB_SZ,
        tone.dark,
        "NOTES",
        f(x + head_text_x(
            SP_HEAD_RT0,
            SP_HEAD_Y,
            SP_HEAD_LAB_BASE,
            SP_HEAD_NOTE_LAB_DX,
        )),
        f(SP_HEAD_LAB_BASE),
    );
    text_at(
        kit,
        theme::FONT_TITLE,
        SP_HEAD_NOTE_SZ,
        COL_INK,
        &column.note,
        f(x + head_text_x(SP_HEAD_RT0, SP_HEAD_Y, SP_HEAD_VAL_BASE, SP_HEAD_NOTE_DX)),
        f(SP_HEAD_VAL_BASE),
    );

    text_at(
        kit,
        theme::FONT_TITLE,
        SP_HEAD_NOTE_LAB_SZ,
        COL_LABEL,
        "CHARTER",
        f(x + head_text_x(
            SP_HEAD_SEAM_T,
            SP_HEAD_SEAM_Y0,
            SP_HEAD_LAB_BASE,
            SP_HEAD_CHAR_LAB_DX,
        )),
        f(SP_HEAD_LAB_BASE),
    );
    fit_text(
        kit,
        Pen {
            family: theme::FONT_TITLE,
            size: SP_HEAD_CHAR_SZ,
            color: COL_INK,
        },
        &column.designer,
        f(x + head_text_x(
            SP_HEAD_SEAM_T,
            SP_HEAD_SEAM_Y0,
            SP_HEAD_VAL_BASE,
            SP_HEAD_CHAR_DX,
        )),
        f(SP_HEAD_VAL_BASE),
        f(x + SONG_COL_W - 8.0),
    );
}

fn paint_row(kit: &CanvasKit<'_>, column: &SongColumn, index: usize, row: Option<&SongBoardRow>) {
    let x = column.x;
    let y = row_y(index);
    kit.fill_rect(area(x, y, SONG_COL_W, SP_ROW_H), COL_WHITE);

    let x1 = x + SONG_COL_W - SP_TRI_PAD;
    let y1 = y + SP_ROW_H - SP_TRI_PAD;
    let tri = [(x1 - SP_TRI_SIZE, y1), (x1, y1), (x1, y1 - SP_TRI_SIZE)];
    sketch::fill_polygon(kit, &poly(&tri), tri_tone(column.difficulty, index));

    let Some(row) = row else {
        return;
    };

    kit.fill_rect(
        area(
            x + SP_AXIS_X,
            y + SP_RULE_Y,
            SP_RIGHT_X - SP_AXIS_X,
            SP_RULE_H,
        ),
        COL_RULE,
    );

    paint_avatar(kit, x + SP_AV_CX, y + SP_AV_CY, row);

    let tone = tone_of(column.difficulty);
    let (rank_bg, rank_ink) = if index < 3 {
        (tone.main, tone.dark)
    } else {
        (COL_ACCENT, COL_MUTED)
    };
    let rank = [
        (x + SP_AXIS_X, y + SP_RANK_Y0),
        (x + SP_AXIS_X + SP_RANK_W, y + SP_RANK_Y0),
        (
            x + SP_AXIS_X + SP_RANK_W - SP_RANK_LEAN,
            y + SP_RANK_Y0 + SP_RANK_H,
        ),
        (x + SP_AXIS_X - SP_RANK_LEAN, y + SP_RANK_Y0 + SP_RANK_H),
    ];
    sketch::fill_polygon(kit, &poly(&rank), rank_bg);
    let rank_text = pad2(index + 1);
    let rank_style = style_of(theme::FONT_TITLE, SP_RANK_SIZE, rank_ink);
    let rank_w = sketch::text_width(kit, &rank_style, &rank_text);
    text_at(
        kit,
        theme::FONT_TITLE,
        SP_RANK_SIZE,
        rank_ink,
        &rank_text,
        f(x + SP_AXIS_X + SP_RANK_W / 2.0 - SP_RANK_LEAN / 2.0) - rank_w / 2.0,
        f(y + SP_RANK_BASE),
    );

    let name_x = f(x + SP_ROW_NAME_X);
    let name_limit = f(x + SP_RIGHT_X - 70.0);
    let name = if row.name.trim().is_empty() {
        String::from(SP_NONE)
    } else {
        row.name.trim().to_string()
    };
    let shown = ellipsized(
        kit,
        theme::FONT_TITLE,
        SP_ROW_NAME_SIZE,
        &name,
        (name_limit - name_x).max(1.0),
    );
    text_at(
        kit,
        theme::FONT_TITLE,
        SP_ROW_NAME_SIZE,
        COL_INK,
        &shown,
        name_x,
        f(y + SP_ROW_NAME_BASE),
    );

    if !row.grade.trim().is_empty() {
        paint_grade_art(kit, x, y, row.grade.trim());
    }

    let score = sketch::thousands(row.score);
    let score_right = x + SP_RIGHT_X - 150.0;
    let mut size = SP_SCORE_SIZE;
    while size > 14.0
        && f(x + SP_SCORE_X) + text_w(kit, theme::FONT_TITLE, size, &score) > f(score_right)
    {
        size -= 1.0;
    }
    text_at(
        kit,
        theme::FONT_TITLE,
        size,
        COL_INK,
        &score,
        f(x + SP_SCORE_X),
        f(y + SP_SCORE_BASE),
    );

    paint_judge_line(kit, x, y, row);
}

fn paint_judge_line(kit: &CanvasKit<'_>, x: f64, y: f64, row: &SongBoardRow) {
    let (text, ink) = if row.judgement {
        (
            format!(
                "P {}(-{}) · F {} · L {}",
                row.perfect, row.normal_perfect, row.far, row.lost
            ),
            COL_MUTED,
        )
    } else {
        (String::from("P -- · F -- · L --"), COL_GHOST)
    };
    let style = style_of(theme::FONT_TITLE, SP_JUDGE_SIZE, ink);
    let width = sketch::text_width(kit, &style, &text);
    text_at(
        kit,
        theme::FONT_TITLE,
        SP_JUDGE_SIZE,
        ink,
        &text,
        f(x + SP_RIGHT_X) - width,
        f(y + SP_JUDGE_BASE),
    );
}

fn paint_avatar(kit: &CanvasKit<'_>, cx: f64, cy: f64, row: &SongBoardRow) {
    let ptt = row.has_rating.then_some(row.rating);
    let avatar = super::avatar::Avatar::new(row.avatar.as_deref())
        .with_ptt(ptt)
        .with_frame(Some(row.rating_frame));
    super::avatar::draw(kit, f(cx), f(cy), f(SP_AV_R), &avatar);
}

fn paint_grade_art(kit: &CanvasKit<'_>, x: f64, y: f64, grade: &str) {
    let file = grade_file(grade);
    if file.is_empty() {
        return;
    }
    let key = format!("grade/{file}");
    let Ok(texture) = kit.asset(&key) else {
        return;
    };
    let Some((img_w, img_h)) = texture.dimensions() else {
        return;
    };
    if img_w <= 0 || img_h <= 0 {
        return;
    }
    let h = f(SP_GRADE_ART_H);
    let w = h * img_w as f32 / img_h as f32;
    let mut rx = f(x + SP_GRADE_ART_RX);
    let mut cy = f(y + SP_GRADE_ART_CY);
    if let Some((_, min_y, max_x, max_y)) = grade_ink_box(file) {
        let k = h / img_h as f32;
        rx -= (img_w as f32 - max_x) * k;
        cy -= ((min_y + max_y) / 2.0 - img_h as f32 / 2.0) * k;
    }
    let rect = Rect::from_ltrb(rx - w, cy - h / 2.0, rx, cy + h / 2.0);
    let _ = kit.draw_asset(&key, rect, 0.0);
}

fn paint_footer(kit: &CanvasKit<'_>, plan: &SongPlan) {
    let mark = [
        (SONG_MARGIN, SP_FOOT_Y),
        (SONG_MARGIN + 10.0, SP_FOOT_Y),
        (SONG_MARGIN + 7.0, SP_FOOT_Y + SP_FOOT_H),
        (SONG_MARGIN, SP_FOOT_Y + SP_FOOT_H),
    ];
    sketch::fill_polygon(kit, &poly(&mark), COL_BLUE);
    text_at(
        kit,
        theme::FONT_TITLE,
        SP_FOOT_SIZE,
        COL_MUTED,
        SP_BRAND,
        f(SP_FOOT_TEXT_X),
        f(SP_FOOT_BASE),
    );
    let style = style_of(theme::FONT_TITLE, SP_FOOT_SIZE, COL_MUTED);
    let width = sketch::text_width(kit, &style, SP_SIGN);
    text_at(
        kit,
        theme::FONT_TITLE,
        SP_FOOT_SIZE,
        COL_MUTED,
        SP_SIGN,
        f(plan.right) - width,
        f(SP_FOOT_BASE),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::harness;

    const PNG_MAGIC: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

    fn chart(difficulty: Difficulty, level: &str, constant: f64, note: i64) -> Chart {
        Chart::new(difficulty, level, constant)
            .with_designer("Toaster")
            .with_note(note)
    }

    fn sample() -> SongInput {
        let song = Song::new("acheron", "Acheron").with_artist("sasakure.UK");
        let mut input = SongInput::new(&song)
            .with_aliases(&["黑河", "阿刻戎"])
            .with_jacket("jackets/acheron.jpg");
        input.bpm = 197.0;
        input.pack_name = String::from("Memory Archive");
        input.charts = vec![
            chart(Difficulty::Past, "5", 5.0, 700),
            chart(Difficulty::Present, "8", 8.3, 1000),
            chart(Difficulty::Future, "10", 10.6, 1500),
        ];
        input
    }

    fn render(input: &SongInput) -> Vec<u8> {
        let harness = harness::Harness::new();
        let bytes = harness.render(request(input), |kit| paint(kit, input));
        assert!(bytes.starts_with(&PNG_MAGIC), "必须输出 PNG");
        assert!(!bytes.is_empty(), "渲染结果不能为空");
        bytes
    }

    fn device_size(input: &SongInput) -> Option<(u32, u32)> {
        let (width, height) = request(input).physical_size();
        Some((width as u32, height as u32))
    }

    #[test]
    fn case_195() {
        assert_eq!(columns_for(0), 3);
        assert_eq!(columns_for(3), 3);
        assert_eq!(columns_for(4), 4);
        assert_eq!(columns_for(5), 5);
        assert_eq!(columns_for(9), 5);
        assert_eq!(canvas_w(3), 1100.0);
        assert_eq!(canvas_w(4), 1448.0);
        assert_eq!(canvas_w(5), 1796.0);
        assert_eq!(width_for(3), 1100);
        assert_eq!(width_for(4), 1448);
        assert_eq!(width_for(5), 1796);
        assert_eq!(height(), 770);
        assert_eq!(fi(SONG_BASE_H), 770);
    }

    #[test]
    fn case_196() {
        let charts = vec![
            chart(Difficulty::Future, "9", 9.6, 1200),
            chart(Difficulty::Beyond, "11", 11.2, 1600),
            chart(Difficulty::Past, "4", 4.0, 600),
        ];
        let live = live_levels(&charts);
        assert_eq!(live.len(), 4);
        assert_eq!(live[0].difficulty, Difficulty::Past);
        assert_eq!(live[1].difficulty, Difficulty::Present);
        assert_eq!(live[2].difficulty, Difficulty::Future);
        assert_eq!(live[3].difficulty, Difficulty::Beyond);
        assert!(!live[1].has_constant, "缺档的列不写定数");
        let mut with_etr = charts.clone();
        with_etr.push(chart(Difficulty::Eternal, "10", 10.0, 1400));
        let live = live_levels(&with_etr);
        assert_eq!(live.len(), 5);
        assert_eq!(live[3].difficulty, Difficulty::Eternal);
        assert_eq!(live[4].difficulty, Difficulty::Beyond);
    }

    #[test]
    fn case_197() {
        let input = SongInput::new(&Song::new("empty", ""));
        let plan = plan_for(&input);
        assert_eq!(plan.columns.len(), 3);
        assert_eq!(plan.canvas(), (1100, 770));
        assert_eq!(plan.right, 1064.0);
        for column in &plan.columns {
            assert!(column.has, "基础三档的列必须画出来");
            assert_eq!(column.constant, SP_UNKNOWN);
            assert_eq!(column.note, SP_NONE);
            assert_eq!(column.level, "");
        }
        assert_eq!(plan.columns[0].tag, "PST");
        assert_eq!(plan.columns[2].tag, "FTR");
    }

    #[test]
    fn case_198() {
        let right = 1064.0;
        assert_eq!(bar_cell_w(right), (right - SP_BAR_T0) / 4.0);
        assert_eq!(bar_cell_x(0, right), SP_BAR_T0);
        assert_eq!(bar_cell_x(2, right), SP_BAR_T0 + 2.0 * bar_cell_w(right));
        let slope = bar_slope();
        assert!((slope - 14.0 / 68.0).abs() < 1e-9);
        let lab = bar_text_x(0, right, SP_BAR_LAB_Y, SP_BAR_LAB_DX);
        assert!((lab - (235.0 - 24.0 * slope + 31.0)).abs() < 1e-6);
        let val = bar_text_x(0, right, SP_BAR_VAL_Y, SP_BAR_VAL_DX);
        assert!((val - (235.0 - 52.0 * slope + 25.0)).abs() < 1e-6);
        assert!(lab > val, "基线越往下, X 越要往左收");
    }

    #[test]
    fn case_199() {
        assert!((head_slope() - 0.2).abs() < 1e-9);
        let note_lab = head_text_x(
            SP_HEAD_RT0,
            SP_HEAD_Y,
            SP_HEAD_LAB_BASE,
            SP_HEAD_NOTE_LAB_DX,
        );
        assert!((note_lab - 132.0).abs() < 1e-6);
        let note_val = head_text_x(SP_HEAD_RT0, SP_HEAD_Y, SP_HEAD_VAL_BASE, SP_HEAD_NOTE_DX);
        assert!((note_val - 121.2).abs() < 1e-6);
        let char_lab = head_text_x(
            SP_HEAD_SEAM_T,
            SP_HEAD_SEAM_Y0,
            SP_HEAD_LAB_BASE,
            SP_HEAD_CHAR_LAB_DX,
        );
        assert!((char_lab - 226.0).abs() < 1e-6);
        let char_val = head_text_x(
            SP_HEAD_SEAM_T,
            SP_HEAD_SEAM_Y0,
            SP_HEAD_VAL_BASE,
            SP_HEAD_CHAR_DX,
        );
        assert!((char_val - 215.2).abs() < 1e-6);
        assert!((SP_HEAD_LV_BASE - 253.44).abs() < 1e-6);
    }

    #[test]
    fn case_200() {
        assert_eq!(row_y(0), 300.0);
        assert_eq!(row_y(1), 382.0);
        assert_eq!(row_y(4), 628.0);
        assert_eq!(SONG_ROW_N, 5);
        assert_eq!(SP_RIGHT_X, 314.0);
        assert_eq!(SP_ROW_NAME_X, 126.0);
        assert_eq!(col_x(0), 36.0);
        assert_eq!(col_x(2), 732.0);
        assert_eq!(col_x(4), 1428.0);
    }

    #[test]
    fn case_201() {
        assert_eq!(grade_file("EX+"), "EX+.png");
        assert_eq!(grade_file(" pm "), "EX+.png");
        assert_eq!(grade_file("fr"), "EX+.png");
        assert_eq!(grade_file("EX"), "EX.png");
        assert_eq!(grade_file("aa"), "AA.png");
        assert_eq!(grade_file("A"), "A.png");
        assert_eq!(grade_file("S"), "");
    }

    #[test]
    fn case_202() {
        assert_eq!(label_en("BPM"), "BPM");
        assert_eq!(label_en("上架"), "RELEASE");
        assert_eq!(label_en("曲目 ID"), "SONG ID");
        assert_eq!(label_en("物量合计"), "TOTAL NOTES");
        assert_eq!(label_en("未知字段"), "未知字段");
    }

    #[test]
    fn case_203() {
        assert_eq!(
            split_song_meta("sasakure.UK · Memory Archive"),
            Some((String::from("sasakure.UK"), String::from("Memory Archive")))
        );
        assert_eq!(split_song_meta("只有曲师"), None);
        assert_eq!(split_song_meta(" · 空曲师"), None);
        let input = sample();
        assert_eq!(input.meta_note(), "sasakure.UK · Memory Archive");
        assert_eq!(input.alias_text(), "黑河 / 阿刻戎");
    }

    #[test]
    fn case_204() {
        let input = sample();
        let cells = input.cells_value();
        assert_eq!(cells.len(), 4);
        assert_eq!(cells[0], (String::from("BPM"), String::from("197.0")));
        assert_eq!(cells[1].1, SP_NONE);
        assert_eq!(cells[2], (String::from("曲目 ID"), String::from("acheron")));
        assert_eq!(cells[3].1, sketch::thousands(3200));
    }

    #[test]
    fn case_205() {
        let input = sample();
        let rows = vec![
            SongBoardRow::new("Alice", 10_012_345)
                .with_grade("EX+")
                .with_rating(12.8, 7, 7)
                .with_judgement(1400, 12, 3, 1),
            SongBoardRow::new("Bob", 9_000_000),
        ];
        let input = input.with_boards(vec![SongBoard::new(Difficulty::Future, rows)]);
        assert_eq!(input.rows_for(Difficulty::Future).len(), 2);
        assert!(input.rows_for(Difficulty::Past).is_empty());
    }

    #[test]
    fn case_110() {
        let mut input = SongInput::new(&Song::new("", ""));
        input.jacket = None;
        let bytes = render(&input);
        assert_eq!(
            harness::image_size(&bytes),
            device_size(&input)
        );
        let pixels = harness::decode(&bytes).expect("解码空曲目页");
        assert!(!pixels.blank(), "纸底与三列占位必须画出来");
        assert!(pixels.distinct_colors(7) > 3);
    }

    #[test]
    fn case_206() {
        let mut input = sample();
        input.title = "A".repeat(120);
        input.charts[2].designer = String::from("Nitro & Toaster feat. someone very long");
        let bytes = render(&input);
        assert_eq!(
            harness::image_size(&bytes),
            device_size(&input)
        );
        let pixels = harness::decode(&bytes).expect("解码长名曲目页");
        assert!(!pixels.blank());
        let head = pixels.ink_ratio(
            fi(col_x(0)) as u32,
            fi(SP_HEAD_Y) as u32,
            fi(SONG_COL_W) as u32,
            fi(SP_HEAD_H) as u32,
            5,
        );
        assert!(head > 0.02, "定数头卡必须看得见, 实际 {head}");
    }

    #[test]
    fn case_207() {
        let input = sample().with_boards(vec![SongBoard::new(
            Difficulty::Future,
            vec![
                SongBoardRow::new("Alice", 10_012_345)
                    .with_grade("EX+")
                    .with_rating(12.8, 7, 7)
                    .with_judgement(1400, 12, 3, 1),
                SongBoardRow::new("Bob", 9_000_000).with_grade("EX"),
            ],
        )]);
        let bytes = render(&input);
        let width = input.width() as u32;
        assert_eq!(harness::image_size(&bytes), device_size(&input));
        let pixels = harness::decode(&bytes).expect("解码带榜曲目页");
        assert!(!pixels.blank());
        let hero = pixels.ink_ratio(0, 0, width, fi(SP_HERO_BOT) as u32, 9);
        assert!(hero > 0.05, "顶部横幅必须有内容, 实际 {hero}");
        let foot = pixels.ink_ratio(
            0,
            (fi(SP_FOOT_Y) - 4) as u32,
            width,
            (fi(SP_FOOT_H) + 8) as u32,
            3,
        );
        assert!(foot > 0.0, "页脚落款必须画出来");
    }
}
