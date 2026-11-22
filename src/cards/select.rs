use aurorix_plugin::RenderCard;
use aurorix_render::skia_safe::{BlendMode, ClipOp, Paint, Path, PathBuilder, Rect};
use aurorix_render::{
    CanvasKit, Color, Gradient, GradientStop, LinearGradient, RenderRequest, TextStyle,
};

use super::player;
use super::sketch;
use super::theme;
use super::widgets;
use crate::domain::{Chart, Song};
use crate::model::{Difficulty, Grade};

pub const SELECT_WIDTH: i32 = 2376;

pub const SELECT_HEIGHT: i32 = 1080;

pub const SELECT_DEV_PER_DESIGN: f64 = 1.575;

pub const SELECT_BASE_W: f64 = 1508.5714;

pub const SELECT_BASE_H: f64 = 685.7143;

pub const SELECT_INSET: f64 = 18.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_CONTENT_RIGHT: f64 = SELECT_BASE_W - SELECT_INSET;

pub const SELECT_TOP_BAR_H: f64 = 76.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_BAR_SHIFT: f64 = 132.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_DIFF_TOP: f64 = 86.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_DIFF_H: f64 = 122.7 / SELECT_DEV_PER_DESIGN;

pub const SELECT_DIFF_LEFT: f64 = -19.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_DIFF_LEVEL_CY: f64 = 59.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_DIFF_LEVEL_SIZE: f64 = 42.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_DIFF_WIDTHS: [f64; 5] = [
    290.1 / SELECT_DEV_PER_DESIGN,
    215.8 / SELECT_DEV_PER_DESIGN,
    215.8 / SELECT_DEV_PER_DESIGN,
    206.9 / SELECT_DEV_PER_DESIGN,
    206.9 / SELECT_DEV_PER_DESIGN,
];

pub const SELECT_DIFF_OVERLAP: f64 = 46.0 / SELECT_DEV_PER_DESIGN;
pub const SELECT_DIFF_LEVEL_OFFSETS: [f64; 5] = [
    41.4 / SELECT_DEV_PER_DESIGN,
    5.6 / SELECT_DEV_PER_DESIGN,
    5.0 / SELECT_DEV_PER_DESIGN,
    5.0 / SELECT_DEV_PER_DESIGN,
    5.0 / SELECT_DEV_PER_DESIGN,
];

pub const SELECT_DIFF_ORDER: [Difficulty; 5] = [
    Difficulty::Past,
    Difficulty::Present,
    Difficulty::Future,
    Difficulty::Eternal,
    Difficulty::Beyond,
];

pub const SELECT_SCORE_X: f64 = 0.0;

pub const SELECT_SCORE_Y: f64 = 79.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_SCORE_W: f64 = 596.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_SCORE_H: f64 = 226.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_SCORE_TAIL_PAD: f64 = 40.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_SCORE_TEXT_X: f64 = 160.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_SCORE_TEXT_CY: f64 = 258.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_SCORE_TEXT_SIZE: f64 = 44.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_SCORE_PLACEHOLDER: &str = "00'000'000";

pub const SELECT_PACK_X: f64 = 952.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_PACK_Y: f64 = 120.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_PACK_W: f64 = 439.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_PACK_H: f64 = 173.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_PACK_TOP_ROW_CY: f64 = 22.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_PACK_NAME_CY: f64 = 72.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_PACK_COUNT_CY: f64 = 130.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_PACK_SMALL: f64 = 22.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_PACK_NAME_SIZE: f64 = 38.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_PACK_COUNT_SIZE: f64 = 26.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_INFO_X: f64 = 124.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_INFO_TITLE_Y: f64 = 332.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_INFO_ARTIST_Y: f64 = 395.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_INFO_BPM_Y: f64 = 438.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_INFO_TITLE_SIZE: f64 = 64.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_INFO_ARTIST_SIZE: f64 = 38.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_INFO_BPM_SIZE: f64 = 28.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_INFO_W: f64 = 520.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_ROWS: usize = 6;

pub const SELECT_SEL_SLOT: usize = 3;

pub const SELECT_ROW_PITCH: f64 = 192.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_CELL_OX: f64 = 1539.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_CELL_OY0: f64 = 176.3 / SELECT_DEV_PER_DESIGN;

pub const SELECT_ROW_LEFT: f64 = SELECT_CELL_OX;

pub const SELECT_ROW_BAND_DY: f64 = -119.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_ROW_BAND_H: f64 = 148.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_ROW_CAP_W: f64 = SELECT_ROW_BAND_H * 1.05;

pub const SELECT_ROW_JACKET_W: f64 = SELECT_ROW_BAND_H * 1.72;

pub const SELECT_ROW_SPLIT: f64 = 0.68;

pub const SELECT_ROW_CAP_TOP_SKEW: f64 = 0.36;

pub const SELECT_ROW_CAP_BOT_SKEW: f64 = 0.18;

pub const SELECT_ROW_LEVEL_SIZE: f64 = SELECT_ROW_BAND_H * 0.42;

pub const SELECT_ROW_MARK_SIZE: f64 = SELECT_ROW_BAND_H * 0.34;

pub const SELECT_ROW_TITLE_SIZE: f64 = 30.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_ROW_TITLE_PAD: f64 = 8.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_ROW_ARC_STEP: f64 = 32.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_HERO_CX: f64 = 814.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_HERO_CY: f64 = 690.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_HERO_SIDE: f64 = 690.0 / SELECT_DEV_PER_DESIGN;

pub const SELECT_HERO_ANGLE: f32 = -4.5;

pub const SELECT_HERO_SHEAR: f32 = 0.04;

pub const SELECT_BTN_CX: f64 = 107.05;

pub const SELECT_BTN_W: f64 = 216.1;

pub const SELECT_BTN_H: f64 = 68.0;

pub const SELECT_BTN_LABEL_X: f64 = 97.0;

pub const SELECT_BTN_LABEL_UP: f64 = 42.0;

pub const SELECT_FOOT_LABEL: &str = "主菜单";

pub const SELECT_PAGE_TITLE: &str = "选择一首歌曲";

pub const SELECT_PACK_ALL: &str = "All Songs";

pub const SELECT_NONE: &str = "未收录";

pub const COL_BASE_TOP: Color = Color::new(64, 62, 108, 255);

pub const COL_BASE_MID: Color = Color::new(20, 20, 34, 255);

pub const COL_BASE_BOTTOM: Color = Color::new(62, 44, 90, 255);

pub const COL_BASE_MID_Y: f64 = 450.0;

pub const COL_BADGE: Color = Color::new(58, 56, 78, 235);

pub const COL_PLATE_INK: Color = Color::new(52, 48, 62, 255);

pub const COL_PACK_F: Color = Color::new(126, 92, 190, 255);

pub const COL_PACK_C: Color = Color::new(104, 116, 142, 255);

pub const COL_PANEL: Color = Color::new(246, 244, 252, 238);

pub const COL_PANEL_EDGE: Color = Color::new(255, 255, 255, 235);

pub const COL_STRIP_INK: Color = Color::new(15, 12, 25, 170);

pub const COL_WHITE: Color = Color::new(255, 255, 255, 255);

pub const COL_TITLE_INK: Color = Color::new(10, 8, 18, 255);

pub const COL_LEVEL_INK: Color = Color::new(24, 10, 30, 255);

pub const COL_DIFF_LEVEL_INK: Color = Color::new(26, 10, 32, 200);

pub const COL_INFO_STROKE: Color = Color::new(18, 12, 26, 210);

pub const COL_SOFT_SHADOW: Color = Color::new(8, 6, 16, 175);

pub const COL_FOOT_INK: Color = Color::new(44, 45, 48, 255);

pub const COL_RIM: Color = Color::new(236, 228, 246, 190);

pub const SELECT_CLEAR_YES: char = 'C';

pub const SELECT_CLEAR_NO: char = 'L';

pub fn dv(value: f64) -> f32 {
    (value * SELECT_DEV_PER_DESIGN).round() as f32
}

pub fn dvi(value: f64) -> i32 {
    (value * SELECT_DEV_PER_DESIGN).round() as i32
}

pub fn dsz(value: f64) -> u32 {
    let size = (value * SELECT_DEV_PER_DESIGN).round();
    if size < 1.0 {
        1
    } else {
        size as u32
    }
}

pub fn diff_index(difficulty: Difficulty) -> usize {
    SELECT_DIFF_ORDER
        .iter()
        .position(|candidate| *candidate == difficulty)
        .unwrap_or(0)
}

pub fn diff_tab_w(index: usize) -> f64 {
    SELECT_DIFF_WIDTHS[index.min(SELECT_DIFF_WIDTHS.len() - 1)]
}

pub fn diff_tab_dx(index: usize) -> f64 {
    let last = index.min(SELECT_DIFF_WIDTHS.len() - 1);
    let mut dx = 0.0;
    for i in 1..=last {
        dx += diff_tab_w(i - 1) - SELECT_DIFF_OVERLAP;
    }
    dx
}

pub fn diff_tab_left(index: usize) -> f64 {
    SELECT_DIFF_LEFT + diff_tab_dx(index)
}

pub fn diff_tab_level_cx(index: usize) -> f64 {
    let offset = SELECT_DIFF_LEVEL_OFFSETS[index.min(SELECT_DIFF_LEVEL_OFFSETS.len() - 1)];
    diff_tab_w(index) / 2.0 + offset
}

pub fn diff_selector_asset(index: usize, _slot: Difficulty) -> &'static str {
    match SELECT_DIFF_ORDER.get(index) {
        Some(Difficulty::Beyond) => "difficulty_selector_3",
        Some(Difficulty::Eternal) => "difficulty_selector_4",
        _ => match index {
            0 => "difficulty_selector_0",
            1 => "difficulty_selector_1",
            2 => "difficulty_selector_2",
            _ => "",
        },
    }
}

pub fn cell_suffix(difficulty: Difficulty) -> &'static str {
    if difficulty == Difficulty::Beyond {
        "_beyond"
    } else {
        ""
    }
}

pub fn cell_asset(difficulty: Difficulty, selected: bool) -> String {
    let suffix = cell_suffix(difficulty);
    if selected {
        format!("songselect/cell/song_cell_right{suffix}_selected.png")
    } else {
        format!("songselect/cell/song_cell_right{suffix}.png")
    }
}

pub fn piece_asset(difficulty: Difficulty) -> String {
    format!(
        "songselect/cell/song_cell_selected_piece{}.png",
        cell_suffix(difficulty)
    )
}

pub fn difficulty_asset(difficulty: Difficulty, selected: bool) -> String {
    let index = diff_index(difficulty);
    let stem = diff_selector_asset(index, difficulty);
    if stem.is_empty() {
        return String::new();
    }
    if selected {
        format!("songselect/diff/{stem}_selected.png")
    } else {
        format!("songselect/diff/{stem}.png")
    }
}

pub fn grade_asset(grade: Grade) -> &'static str {
    crate::domain::grade_select_asset(grade)
}

pub fn grade_asset_text(grade: &str) -> &'static str {
    crate::domain::grade_select_asset_for_label(grade)
}

pub fn grade_sprite(grade: &str) -> &'static str {
    crate::domain::grade_asset_for_label(grade).unwrap_or_default()
}

pub fn clear_sprite(clear: char) -> &'static str {
    match clear {
        'L' => "clear_type/clear_fail.png",
        'C' => "clear_type/clear_normal.png",
        'F' => "clear_type/clear_full.png",
        'P' => "clear_type/clear_pure.png",
        _ => "",
    }
}

pub fn diff_tint(difficulty: Difficulty) -> Color {
    match difficulty {
        Difficulty::Past => Color::new(36, 169, 204, 235),
        Difficulty::Present => Color::new(180, 206, 105, 235),
        Difficulty::Future => Color::new(149, 55, 119, 235),
        Difficulty::Eternal => Color::new(136, 115, 163, 235),
        Difficulty::Beyond => Color::new(208, 32, 60, 235),
        Difficulty::Ins => Color::new(32, 48, 112, 235),
    }
}

pub fn darker_tint(color: Color) -> Color {
    let channel =
        |value: u8, scale: i32, add: i32| ((value as i32 * scale / 100 + add).clamp(0, 255)) as u8;
    Color::new(
        channel(color.r, 35, 20),
        channel(color.g, 40, 30),
        channel(color.b, 45, 60),
        235,
    )
}

pub fn mark_ink(grade: &str, clear: char) -> Color {
    match clear {
        'L' => return Color::new(140, 45, 59, 255),
        'C' => return Color::new(45, 122, 120, 255),
        'F' => return Color::new(108, 59, 140, 255),
        'P' => return Color::new(70, 90, 170, 255),
        _ => {}
    }
    match grade.trim().to_uppercase().as_str() {
        "EX+" | "EX" | "AA" | "A" => Color::new(140, 70, 160, 255),
        "B" | "C" | "D" => Color::new(140, 45, 59, 255),
        _ => Color::new(120, 96, 150, 255),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectRow {
    pub title: String,
    pub song_id: String,
    pub artist: String,
    pub difficulty: Difficulty,
    pub level: String,
    pub constant: f64,
    pub has_constant: bool,
    pub selected: bool,
    pub clear: char,
    pub grade: String,
    pub bpm: String,
}

impl Default for SelectRow {
    fn default() -> Self {
        Self {
            title: String::new(),
            song_id: String::new(),
            artist: String::new(),
            difficulty: Difficulty::Future,
            level: String::new(),
            constant: 0.0,
            has_constant: false,
            selected: false,
            clear: '\0',
            grade: String::new(),
            bpm: String::new(),
        }
    }
}

impl SelectRow {
    pub fn new(title: impl Into<String>, difficulty: Difficulty) -> Self {
        Self {
            title: title.into(),
            difficulty,
            ..Self::default()
        }
    }

    pub fn with_song_id(mut self, id: impl Into<String>) -> Self {
        self.song_id = id.into();
        self
    }

    pub fn with_artist(mut self, artist: impl Into<String>) -> Self {
        self.artist = artist.into();
        self
    }

    pub fn with_level(mut self, level: impl Into<String>, constant: f64) -> Self {
        self.level = level.into();
        self.constant = constant;
        self.has_constant = constant > 0.0;
        self
    }

    pub fn with_selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn with_grade(mut self, grade: impl Into<String>, clear: char) -> Self {
        self.grade = grade.into();
        self.clear = clear;
        self
    }

    pub fn with_bpm(mut self, bpm: impl Into<String>) -> Self {
        self.bpm = bpm.into();
        self
    }

    pub fn level_text(&self) -> String {
        if !self.level.trim().is_empty() {
            return self.level.trim().to_string();
        }
        if self.has_constant && self.constant > 0.0 {
            return format!("{:.1}", self.constant);
        }
        String::from(SELECT_NONE)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectPlayer {
    pub name: String,
    pub ptt: f64,
    pub rating_frame: i32,
    pub rating_accent: i32,
    pub global_rank: Option<i32>,
    pub banner: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SelectInput {
    pub song_id: String,
    pub title: String,
    pub artist: String,
    pub pack_id: String,
    pub pack_name: String,
    pub pack_name_zh: String,
    pub jacket: Option<String>,
    pub backdrop: Option<String>,
    pub charts: Vec<Chart>,
    pub selected: Difficulty,
    pub score: Option<i64>,
    pub grade: Option<Grade>,
    pub pack_count: usize,
    pub clear_count: usize,
    pub pure_count: usize,
    pub total_count: usize,
    pub retry_hint: String,
    pub roll: String,
    pub theme: theme::Theme,
    pub footer_note: String,
    pub rows: Option<Vec<SelectRow>>,
    pub levels: Option<Vec<String>>,
    pub page_title: Option<String>,
    pub pack_title: Option<String>,
    pub player: Option<SelectPlayer>,
}

impl SelectInput {
    pub fn new(song: &Song, selected: Difficulty) -> Self {
        Self {
            song_id: song.id.clone(),
            title: song.title.clone(),
            artist: song.artist.clone(),
            pack_id: song.pack_id.clone(),
            pack_name: song.pack_name.clone(),
            pack_name_zh: song.pack_name_zh.clone(),
            jacket: (!song.id.trim().is_empty()).then(|| format!("jackets/{}.jpg", song.id)),
            backdrop: Some(String::from("background/score.png")),
            charts: song.charts.clone(),
            selected,
            score: None,
            grade: None,
            pack_count: 0,
            clear_count: 0,
            pure_count: 0,
            total_count: 0,
            retry_hint: String::from("按 /a rand 再来一次"),
            roll: String::new(),
            theme: theme::Theme::DEFAULT,
            footer_note: String::new(),
            rows: None,
            levels: None,
            page_title: None,
            pack_title: None,
            player: None,
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

    pub fn with_backdrop(mut self, key: impl Into<String>) -> Self {
        let key = key.into();
        self.backdrop = (!key.trim().is_empty()).then_some(key);
        self
    }

    pub fn with_score(mut self, score: Option<i64>) -> Self {
        self.score = score;
        self
    }

    pub fn with_grade(mut self, grade: Option<Grade>) -> Self {
        self.grade = grade;
        self
    }

    pub fn with_pack_stats(
        mut self,
        pack_count: usize,
        clear_count: usize,
        pure_count: usize,
        total_count: usize,
    ) -> Self {
        self.pack_count = pack_count;
        self.clear_count = clear_count;
        self.pure_count = pure_count;
        self.total_count = total_count;
        self
    }

    pub fn with_retry_hint(mut self, hint: impl Into<String>) -> Self {
        self.retry_hint = hint.into();
        self
    }

    pub fn with_roll(mut self, roll: impl Into<String>) -> Self {
        self.roll = roll.into();
        self
    }

    pub fn with_theme(mut self, theme_value: theme::Theme) -> Self {
        self.theme = theme_value;
        self
    }

    pub fn with_footer(mut self, note: impl Into<String>) -> Self {
        self.footer_note = note.into();
        self
    }

    pub fn with_rows(mut self, rows: Vec<SelectRow>) -> Self {
        self.rows = Some(rows);
        self
    }

    pub fn with_levels(mut self, levels: Vec<String>) -> Self {
        self.levels = Some(levels);
        self
    }

    pub fn with_page_title(mut self, title: impl Into<String>) -> Self {
        self.page_title = Some(title.into());
        self
    }

    pub fn with_pack_title(mut self, title: impl Into<String>) -> Self {
        self.pack_title = Some(title.into());
        self
    }

    pub fn with_player(mut self, player: SelectPlayer) -> Self {
        self.player = Some(player);
        self
    }

    pub fn chart(&self, difficulty: Difficulty) -> Option<&Chart> {
        self.charts
            .iter()
            .find(|chart| chart.difficulty == difficulty)
    }

    pub fn selected_chart(&self) -> Option<&Chart> {
        self.chart(self.selected)
    }

    pub fn present(&self) -> Vec<Difficulty> {
        SELECT_DIFF_ORDER
            .into_iter()
            .filter(|difficulty| self.chart(*difficulty).is_some())
            .collect()
    }

    pub fn levels(&self) -> Vec<String> {
        if let Some(levels) = &self.levels {
            let mut out = levels.clone();
            out.resize(5, String::new());
            return out;
        }
        SELECT_DIFF_ORDER
            .into_iter()
            .map(|difficulty| {
                self.chart(difficulty)
                    .map(|chart| chart.level.trim().to_string())
                    .unwrap_or_default()
            })
            .collect()
    }

    pub fn score_text(&self) -> String {
        match self.score {
            Some(score) if score > 0 => sketch::thousands(score),
            _ => String::from(SELECT_SCORE_PLACEHOLDER),
        }
    }

    pub fn page_title(&self) -> String {
        self.page_title
            .clone()
            .unwrap_or_else(|| String::from(SELECT_PAGE_TITLE))
    }

    pub fn pack_title(&self) -> String {
        let title = self
            .pack_title
            .clone()
            .or_else(|| {
                let name = self.pack_name.trim();
                (!name.is_empty()).then(|| name.to_string())
            })
            .unwrap_or_default();
        if title.is_empty() {
            String::from(SELECT_PACK_ALL)
        } else {
            title
        }
    }

    pub fn rows_value(&self) -> Vec<SelectRow> {
        if let Some(rows) = &self.rows {
            return rows.clone();
        }
        let mut out: Vec<SelectRow> = Vec::new();
        for difficulty in SELECT_DIFF_ORDER {
            if let Some(chart) = self.chart(difficulty) {
                out.push(
                    SelectRow::new(self.title.clone(), difficulty)
                        .with_song_id(self.song_id.clone())
                        .with_artist(self.artist.clone())
                        .with_level(chart.level.clone(), chart.constant)
                        .with_selected(difficulty == self.selected),
                );
            }
        }
        if out.is_empty() {
            out.push(
                SelectRow::new(self.title.clone(), self.selected)
                    .with_song_id(self.song_id.clone())
                    .with_artist(self.artist.clone())
                    .with_selected(true),
            );
        }
        out
    }

    pub fn selected_row(&self) -> Option<SelectRow> {
        let rows = self.rows_value();
        rows.iter()
            .find(|row| row.selected)
            .cloned()
            .or_else(|| rows.first().cloned())
    }

    pub fn hero_keys(&self) -> Vec<String> {
        let row = self.selected_row();
        let id = row
            .map(|row| row.song_id)
            .filter(|id| !id.trim().is_empty())
            .unwrap_or_else(|| self.song_id.clone());
        if id.trim().is_empty() {
            return Vec::new();
        }
        let mut keys = vec![
            format!("jacket_thumbs/{id}.jpg"),
            format!("jackets/{id}.jpg"),
        ];
        if let Some(jacket) = &self.jacket {
            let key = jacket.trim();
            if !key.is_empty() && !keys.iter().any(|seen| seen == key) {
                keys.push(key.to_string());
            }
        }
        keys
    }

    pub fn row_keys(row: &SelectRow, jacket: Option<&str>) -> Vec<String> {
        let mut keys: Vec<String> = Vec::new();
        let id = row.song_id.trim();
        if !id.is_empty() {
            keys.push(format!("jacket_thumbs/{id}.jpg"));
            keys.push(format!("jackets/{id}.jpg"));
        }
        if let Some(jacket) = jacket {
            let key = jacket.trim();
            if !key.is_empty() && !keys.iter().any(|seen| seen == key) {
                keys.push(key.to_string());
            }
        }
        keys
    }
}

pub fn row_top(slot: usize) -> f64 {
    SELECT_CELL_OY0 + slot as f64 * SELECT_ROW_PITCH + SELECT_ROW_BAND_DY
}

pub fn row_band_h() -> f64 {
    SELECT_ROW_BAND_H
}

pub fn arc_offset(slot: usize) -> f64 {
    let distance = slot as f64 - SELECT_SEL_SLOT as f64;
    let span = (distance * SELECT_ROW_BAND_H).abs();
    let radius = 7.0 * SELECT_ROW_BAND_H;
    if span < radius {
        radius - (radius * radius - span * span).sqrt()
    } else {
        0.0
    }
}

pub fn row_x0(slot: usize) -> f64 {
    SELECT_ROW_LEFT + arc_offset(slot)
}

pub fn cap_offset(y: f64, top: f64) -> f64 {
    let height = SELECT_ROW_BAND_H;
    let split = top + SELECT_ROW_SPLIT * height;
    let bot = top + height;
    if y <= split {
        SELECT_ROW_CAP_TOP_SKEW * height * (split - y) / (split - top)
    } else {
        SELECT_ROW_CAP_BOT_SKEW * height * (y - split) / (bot - split)
    }
}

pub fn panel_mid(x0: f64) -> f64 {
    x0 + SELECT_ROW_CAP_W + SELECT_ROW_JACKET_W
}

pub fn diff_span(levels: &[String], selected: Difficulty) -> (f64, f64) {
    let sel = diff_index(selected);
    let mut x = SELECT_DIFF_LEFT;
    let mut right = x;
    let mut prev = false;
    for index in 0..SELECT_DIFF_ORDER.len() {
        let has = levels
            .get(index)
            .map(|level| !level.trim().is_empty())
            .unwrap_or(false);
        if !has && index != sel {
            continue;
        }
        if prev {
            x -= SELECT_DIFF_OVERLAP;
        }
        right = x + diff_tab_w(index);
        x = right;
        prev = true;
    }
    (SELECT_DIFF_LEFT, right)
}

pub fn score_tab_width(levels: &[String], selected: Difficulty) -> f64 {
    let (_, right) = diff_span(levels, selected);
    let ext = right + SELECT_SCORE_TAIL_PAD - SELECT_SCORE_X;
    ext.max(SELECT_SCORE_W)
}

pub fn request(input: &SelectInput) -> RenderRequest {
    let _ = input;
    super::harness::card_request(SELECT_WIDTH, SELECT_HEIGHT)
}

pub fn card(input: SelectInput) -> RenderCard {
    let request = request(&input);
    let cache_key = super::memo::fingerprint("rand", &input);
    RenderCard::new(request, "rand.png", move |kit| paint(kit, &input)).with_cache_key(cache_key)
}

pub fn paint(kit: &CanvasKit<'_>, input: &SelectInput) {
    let width = SELECT_WIDTH as f32;
    let height = SELECT_HEIGHT as f32;
    paint_base(kit, width, height);
    paint_backdrop(kit, input, width);
    paint_hero(kit, input);
    paint_score(kit, input);
    paint_rows(kit, input);
    paint_pack(kit, input);
    paint_diff(kit, input);
    paint_info(kit, input);
    paint_foot(kit);
    paint_top_bar(kit, input);
    paint_bands(kit, width, height);
}

const TOP_BAR_BG_LEFT_DEV: f32 = -220.0;

const TOP_BAR_BG_RIGHT_DEV: f32 = 2357.0;

const TOP_BAR_TEXT_ROW: f64 = 28.5;

const TOP_BAND_H: f64 = 50.0;

const NAME_PLATE_LEFT: f64 = 280.0;

const NAME_PLATE_RIGHT: f64 = 691.0;

const STROKE_ORTHO: f32 = 0.9;

const STROKE_DIAG: f32 = 0.65;

const PANEL_SRC_LEFT: f32 = 282.0;

const PANEL_SRC_TOP: f32 = 15.0;

const PANEL_SRC_RIGHT: f32 = 810.0;

const PANEL_SRC_BOT: f32 = 169.0;

const SELECTED_DIM: Color = Color::new(140, 140, 158, 255);

const COL_PTT_EDGE: Color = Color::new(0, 0, 0, 255);

const SCORE_GRADE_INK_H: f32 = 36.0;

const ROW_CLEAR_INK_H: f32 = 22.0;

fn area(x: f64, y: f64, w: f64, h: f64) -> Rect {
    widgets::rect(dv(x), dv(y), dv(w), dv(h))
}

fn device(x: f64, y: f64, w: f64, h: f64) -> Rect {
    widgets::rect(x as f32, y as f32, w as f32, h as f32)
}

fn unit(device_value: f64) -> f64 {
    device_value / SELECT_DEV_PER_DESIGN
}

fn at(x: f64, y: f64) -> (f32, f32) {
    (dv(x), dv(y))
}

fn style_of(family: &str, size: f64, color: Color) -> TextStyle {
    let device = dsz(size);
    let scaled = if family == theme::FONT_DISPLAY {
        (device as f32 / theme::DISPLAY_INK_SCALE).round().max(1.0) as u32
    } else {
        device
    };
    theme::style(family, scaled, color)
}

fn ink_baseline(center_y: f32, size: u32) -> f32 {
    center_y + size as f32 * 0.36
}

fn ink_offset(kit: &CanvasKit<'_>, style: &TextStyle, value: &str) -> f32 {
    if value.chars().any(sketch::is_han) {
        ink_baseline(0.0, style.size)
    } else {
        sketch::ink_center(kit, style, value)
    }
}

fn text_ink_left(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    x: f32,
    center_y: f32,
    value: &str,
    color: Color,
) {
    if value.is_empty() {
        return;
    }
    let baseline = center_y + ink_offset(kit, style, value);
    sketch::text(kit, style, value, x, baseline, color);
}

fn text_ink_center(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    center_x: f32,
    center_y: f32,
    value: &str,
    color: Color,
) {
    if value.is_empty() {
        return;
    }
    let baseline = center_y + ink_offset(kit, style, value);
    if value.chars().any(sketch::is_han) {
        sketch::text_center(kit, style, value, center_x, baseline, color);
    } else {
        let offset = sketch::ink_center_x(kit, style, value);
        sketch::text(kit, style, value, center_x - offset, baseline, color);
    }
}

fn stroke_offsets() -> [(f32, f32); 8] {
    [
        (-STROKE_ORTHO, 0.0),
        (STROKE_ORTHO, 0.0),
        (0.0, -STROKE_ORTHO),
        (0.0, STROKE_ORTHO),
        (-STROKE_DIAG, -STROKE_DIAG),
        (STROKE_DIAG, -STROKE_DIAG),
        (-STROKE_DIAG, STROKE_DIAG),
        (STROKE_DIAG, STROKE_DIAG),
    ]
}

struct StrokeInk {
    fill: Color,
    stroke: Color,
}

fn stroked_center(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    center_x: f32,
    center_y: f32,
    value: &str,
    ink: StrokeInk,
) {
    let StrokeInk { fill, stroke } = ink;
    if value.is_empty() {
        return;
    }
    let width = sketch::text_width(kit, style, value);
    let baseline = center_y + ink_offset(kit, style, value);
    for (dx, dy) in stroke_offsets() {
        sketch::text(
            kit,
            style,
            value,
            center_x - width / 2.0 + dx,
            baseline + dy,
            stroke,
        );
    }
    sketch::text(kit, style, value, center_x - width / 2.0, baseline, fill);
}

fn stroked_left(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    x: f32,
    center_y: f32,
    value: &str,
    ink: StrokeInk,
) {
    let StrokeInk { fill, stroke } = ink;
    if value.is_empty() {
        return;
    }
    let baseline = center_y + ink_offset(kit, style, value);
    for (dx, dy) in stroke_offsets() {
        sketch::text(kit, style, value, x + dx, baseline + dy, stroke);
    }
    sketch::text(kit, style, value, x, baseline, fill);
}

fn soft_shadow_left(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    x: f32,
    center_y: f32,
    value: &str,
) {
    if value.is_empty() {
        return;
    }
    let baseline = center_y + ink_offset(kit, style, value);
    for (index, (dx, dy)) in [(0.8f32, 0.9f32), (1.4, 1.6), (2.0, 2.2)]
        .iter()
        .enumerate()
    {
        let alpha = (COL_SOFT_SHADOW.a as i32 / (1 + index as i32 * 2)) as u8;
        sketch::text(
            kit,
            style,
            value,
            x + dx,
            baseline + dy,
            COL_SOFT_SHADOW.with_alpha(alpha),
        );
    }
}

fn flat_paint(color: Color) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(color.to_skia());
    paint
}

fn poly_path(points: &[(f32, f32)]) -> Option<Path> {
    if points.len() < 3 {
        return None;
    }
    let mut builder = PathBuilder::new();
    builder.move_to(points[0]);
    for point in &points[1..] {
        builder.line_to(*point);
    }
    builder.close();
    Some(builder.detach())
}

fn filled_poly(kit: &CanvasKit<'_>, points: &[(f32, f32)], color: Color) {
    sketch::fill_polygon(kit, points, color);
}

fn fill_poly_gradient(
    kit: &CanvasKit<'_>,
    points: &[(f32, f32)],
    bounds: Rect,
    gradient: &Gradient,
) {
    let Some(path) = poly_path(points) else {
        return;
    };
    let Ok(shader) = gradient.shader(bounds) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_shader(shader);
    kit.canvas().draw_path(&path, &paint);
}

fn multiply_poly(kit: &CanvasKit<'_>, points: &[(f32, f32)], color: Color) {
    let Some(path) = poly_path(points) else {
        return;
    };
    let mut paint = flat_paint(color);
    paint.set_blend_mode(BlendMode::Multiply);
    kit.canvas().draw_path(&path, &paint);
}

fn stretch_asset(kit: &CanvasKit<'_>, key: &str, rect: Rect) -> bool {
    if key.is_empty() || rect.width() <= 0.0 || rect.height() <= 0.0 {
        return false;
    }
    let Ok(texture) = kit.asset(key) else {
        return false;
    };
    let Some(image) = texture.image() else {
        return false;
    };
    kit.image_rect(image, rect);
    true
}

fn cover_first(kit: &CanvasKit<'_>, keys: &[String], rect: Rect) -> bool {
    for key in keys {
        if kit.draw_asset_cover(key, rect, 0.0).is_ok() {
            return true;
        }
    }
    false
}

fn ptt_parts(ptt: f64) -> (String, String) {
    if ptt <= 0.0 {
        return (String::from("--"), String::new());
    }
    let text = format!("{ptt:.3}");
    match text.split_once('.') {
        Some((head, tail)) => (String::from(head), format!(".{tail}")),
        None => (text, String::new()),
    }
}

fn paint_base(kit: &CanvasKit<'_>, width: f32, height: f32) {
    let mid = dv(COL_BASE_MID_Y).min(height);
    let top = widgets::rect(0.0, 0.0, width, mid);
    let _ = kit.fill_gradient(
        top,
        0.0,
        &Gradient::Linear(LinearGradient::vertical([
            GradientStop::new(0.0, COL_BASE_TOP),
            GradientStop::new(1.0, COL_BASE_MID),
        ])),
    );
    let bottom = widgets::rect(0.0, mid, width, height - mid);
    let _ = kit.fill_gradient(
        bottom,
        0.0,
        &Gradient::Linear(LinearGradient::vertical([
            GradientStop::new(0.0, COL_BASE_MID),
            GradientStop::new(1.0, COL_BASE_BOTTOM),
        ])),
    );
}

fn paint_backdrop(kit: &CanvasKit<'_>, input: &SelectInput, width: f32) {
    let Some(key) = input.backdrop.as_deref() else {
        return;
    };
    let Ok(texture) = kit.asset(key) else {
        return;
    };
    let Some(image) = texture.image() else {
        return;
    };
    let image_width = image.width() as f32;
    let image_height = image.height() as f32;
    if image_width <= 0.0 || image_height <= 0.0 || width <= 0.0 {
        return;
    }
    kit.image_rect(
        image,
        widgets::rect(0.0, 0.0, width, width * image_height / image_width),
    );
}

fn paint_hero(kit: &CanvasKit<'_>, input: &SelectInput) {
    let keys = input.hero_keys();
    if keys.is_empty() {
        return;
    }
    let side = dv(SELECT_HERO_SIDE);
    if side <= 0.0 {
        return;
    }
    let canvas = kit.canvas();
    canvas.save();
    canvas.translate((dv(SELECT_HERO_CX), dv(SELECT_HERO_CY)));
    canvas.rotate(SELECT_HERO_ANGLE, None);
    canvas.skew((SELECT_HERO_SHEAR, 0.0));
    let half = side / 2.0;
    let rect = widgets::rect(-half, -half, side, side);
    for key in &keys {
        if kit.draw_asset_cover(key, rect, 0.0).is_ok() {
            break;
        }
    }
    canvas.restore();
}

fn paint_score(kit: &CanvasKit<'_>, input: &SelectInput) {
    let levels = input.levels.clone().unwrap_or_default();
    let width = score_tab_width(&levels, input.selected);
    let rect = area(SELECT_SCORE_X, SELECT_SCORE_Y, width, SELECT_SCORE_H);
    stretch_asset(kit, "songselect/diff/song_score_tab.png", rect);

    let text = input.score_text();
    let style = style_of(theme::FONT_SCORE, SELECT_SCORE_TEXT_SIZE, COL_WHITE);
    text_ink_left(
        kit,
        &style,
        dv(SELECT_SCORE_TEXT_X),
        dv(SELECT_SCORE_TEXT_CY),
        &text,
        COL_WHITE,
    );

    if let Some(row) = input.selected_row() {
        let key = grade_asset_text(&row.grade);
        if !key.is_empty() {
            let cx = dv(SELECT_SCORE_X + width * 0.745) - 10.0;
            sketch::paste_sprite(kit, key, cx, dv(SELECT_SCORE_TEXT_CY), SCORE_GRADE_INK_H);
        }
    }
}

fn paint_rows(kit: &CanvasKit<'_>, input: &SelectInput) {
    let rows = input.rows_value();
    let selected = rows.iter().position(|row| row.selected);
    let Some(sel) = selected else {
        return;
    };
    let mut first = sel as i64 - SELECT_SEL_SLOT as i64;
    let last = rows.len() as i64 - SELECT_ROWS as i64;
    if first > last {
        first = last;
    }
    if first < 0 {
        first = 0;
    }
    for slot in 0..SELECT_ROWS {
        let index = first + slot as i64;
        if index < 0 || index as usize >= rows.len() {
            continue;
        }
        paint_row(kit, input, &rows[index as usize], slot);
    }
}

fn paint_row(kit: &CanvasKit<'_>, input: &SelectInput, row: &SelectRow, slot: usize) {
    let height = row_band_h();
    if height <= 0.0 {
        return;
    }
    let top = row_top(slot);
    let split = top + SELECT_ROW_SPLIT * height;
    let bot = top + height;
    let x0 = row_x0(slot);
    let cap_w = SELECT_ROW_CAP_W;
    let right = SELECT_CONTENT_RIGHT;
    let panel_mid = panel_mid(x0);

    let left_at = |y: f64| x0 + cap_offset(y, top) + cap_w;
    let panel_at = |y: f64| panel_mid + cap_offset(y, top);

    let jacket_poly = [
        at(left_at(top), top),
        at(panel_at(top), top),
        at(panel_at(split), split),
        at(panel_at(bot), bot),
        at(left_at(bot), bot),
        at(left_at(split), split),
    ];
    let keys = SelectInput::row_keys(row, input.jacket.as_deref());
    if !keys.is_empty() {
        let jacket_box = widgets::rect(
            dv(left_at(split)),
            dv(top),
            dv(SELECT_ROW_JACKET_W + SELECT_ROW_CAP_TOP_SKEW * height + unit(1.0)),
            dv(height),
        );
        if let Some(path) = poly_path(&jacket_poly) {
            let canvas = kit.canvas();
            canvas.save();
            canvas.clip_path(&path, ClipOp::Intersect, true);
            let pasted = cover_first(kit, &keys, jacket_box);
            if pasted && row.selected {
                multiply_poly(kit, &jacket_poly, SELECTED_DIM);
            }
            canvas.restore();
        }
    }

    let panel_poly = [
        at(panel_at(top), top),
        at(right, top),
        at(right, bot),
        at(panel_at(bot), bot),
        at(panel_mid, split),
    ];
    filled_poly(kit, &panel_poly, COL_PANEL);

    let panel_area = widgets::rect(
        dv(panel_mid),
        dv(top),
        dv(right) - dv(panel_mid),
        dv(bot) - dv(top),
    );
    let panel_crop = Rect::from_xywh(
        PANEL_SRC_LEFT,
        PANEL_SRC_TOP,
        PANEL_SRC_RIGHT - PANEL_SRC_LEFT,
        PANEL_SRC_BOT - PANEL_SRC_TOP,
    );
    if let Some(path) = poly_path(&panel_poly) {
        let canvas = kit.canvas();
        canvas.save();
        canvas.clip_path(&path, ClipOp::Intersect, true);
        sketch::paste_crop_clip(
            kit,
            &cell_asset(row.difficulty, row.selected),
            panel_crop,
            panel_area,
        );
        canvas.restore();
    }

    let mark_cy = top + height / 2.0 - unit(4.0);
    let panel_w = right - panel_mid;
    let grade_key = grade_sprite(&row.grade);
    if !grade_key.is_empty() {
        sketch::paste_sprite(
            kit,
            grade_key,
            dv(panel_mid + 0.207 * panel_w),
            dv(mark_cy),
            dv(SELECT_ROW_MARK_SIZE),
        );
    }
    let clear_key = clear_sprite(row.clear);
    if !clear_key.is_empty() {
        sketch::paste_sprite(
            kit,
            clear_key,
            dv(panel_mid + 0.427 * panel_w),
            dv(mark_cy),
            ROW_CLEAR_INK_H,
        );
    }

    let strip_x0 = x0 + cap_w + SELECT_ROW_CAP_BOT_SKEW * height;
    let strip_poly = [
        at(strip_x0, split),
        at(right, split),
        at(right, bot),
        at(x0 + SELECT_ROW_CAP_BOT_SKEW * height, bot),
    ];
    let strip_bounds = widgets::rect(
        dv(strip_x0),
        dv(split),
        dv(right) - dv(strip_x0),
        dv(bot) - dv(split),
    );
    fill_poly_gradient(
        kit,
        &strip_poly,
        strip_bounds,
        &Gradient::Linear(LinearGradient::horizontal([
            GradientStop::new(0.0, COL_STRIP_INK),
            GradientStop::new(1.0, COL_STRIP_INK.with_alpha(0)),
        ])),
    );

    let title_x = strip_x0 + SELECT_ROW_TITLE_PAD;
    let title_center = split + (bot - split) / 2.0 + unit(4.0);
    let title_style = style_of(theme::FONT_TEXT, SELECT_ROW_TITLE_SIZE, COL_WHITE);
    stroked_left(
        kit,
        &title_style,
        dv(title_x),
        dv(title_center),
        &row.title,
        StrokeInk {
            fill: COL_WHITE,
            stroke: COL_TITLE_INK,
        },
    );

    let tint = diff_tint(row.difficulty);
    let purple = [
        at(x0 + SELECT_ROW_CAP_TOP_SKEW * height, top),
        at(x0 + SELECT_ROW_CAP_TOP_SKEW * height + cap_w, top),
        at(x0 + cap_w, split),
        at(x0, split),
    ];
    filled_poly(kit, &purple, tint);
    let blue = [
        at(x0, split),
        at(x0 + cap_w, split),
        at(x0 + cap_w + SELECT_ROW_CAP_BOT_SKEW * height, bot),
        at(x0 + SELECT_ROW_CAP_BOT_SKEW * height, bot),
    ];
    filled_poly(kit, &blue, darker_tint(tint));

    if !row.selected {
        let rim = [
            at(x0 + SELECT_ROW_CAP_TOP_SKEW * height, top),
            at(x0 + unit(2.0), split),
            at(x0 + SELECT_ROW_CAP_BOT_SKEW * height, bot),
            at(x0 + SELECT_ROW_CAP_BOT_SKEW * height + unit(3.0), bot),
            at(x0 + unit(5.0), split),
            at(x0 + SELECT_ROW_CAP_TOP_SKEW * height + unit(3.0), top),
        ];
        filled_poly(kit, &rim, COL_RIM);
    }

    let level_cx = x0 + SELECT_ROW_CAP_BOT_SKEW * height + cap_w / 2.0;
    let level_style = style_of(theme::FONT_LATIN, SELECT_ROW_LEVEL_SIZE, COL_WHITE);
    stroked_center(
        kit,
        &level_style,
        dv(level_cx),
        dv(top + 0.34 * height),
        &row.level_text(),
        StrokeInk {
            fill: COL_WHITE,
            stroke: COL_LEVEL_INK,
        },
    );

    if row.selected {
        let start_w = 196.0;
        let start_h = 70.0;
        let start_y = top + height * 0.34;
        let scx = (x0 + cap_w + panel_mid) / 2.0 + cap_offset(start_y, top);
        stretch_asset(
            kit,
            "songselect/cell/start.png",
            widgets::rect(
                dv(scx) - start_w / 2.0,
                dv(start_y) - start_h / 2.0,
                start_w,
                start_h,
            ),
        );

        let k = (dv(height) as f64 + 26.0) / 168.0;
        stretch_asset(
            kit,
            &piece_asset(row.difficulty),
            widgets::rect(
                dv(x0) - 52.0,
                (dv(top) as f64 - 13.0 - 9.0 * k) as f32,
                (127.0 * k) as f32,
                (187.0 * k) as f32,
            ),
        );
    }

    kit.fill_rect(
        widgets::rect(
            dv(panel_at(top)),
            dv(top),
            dv(right) - dv(panel_at(top)),
            1.0,
        ),
        COL_PANEL_EDGE,
    );
    kit.fill_rect(
        widgets::rect(
            dv(panel_at(bot)),
            dv(bot) - 1.0,
            dv(right) - dv(panel_at(bot)),
            1.0,
        ),
        COL_PANEL_EDGE,
    );
    kit.fill_rect(
        widgets::rect(
            dv(right) - 1.0,
            dv(top),
            1.0,
            dv(bot) - dv(top),
        ),
        COL_PANEL_EDGE,
    );
}

fn paint_pack(kit: &CanvasKit<'_>, input: &SelectInput) {
    let rect = area(SELECT_PACK_X, SELECT_PACK_Y, SELECT_PACK_W, SELECT_PACK_H);
    stretch_asset(kit, "songselect/pack/song_currentpack.png", rect);

    let cx = dv(SELECT_PACK_X + SELECT_PACK_W / 2.0);
    let top_row = dv(SELECT_PACK_Y + SELECT_PACK_TOP_ROW_CY);
    let small = style_of(theme::FONT_LATIN, SELECT_PACK_SMALL, COL_WHITE);
    text_ink_left(kit, &small, dv(SELECT_PACK_X) + 50.0, top_row, "PACKS", COL_WHITE);
    text_ink_left(
        kit,
        &small,
        dv(SELECT_PACK_X) + 327.0,
        top_row,
        &input.pack_count.to_string(),
        COL_WHITE,
    );

    let name_style = style_of(theme::FONT_TEXT, SELECT_PACK_NAME_SIZE, COL_WHITE);
    let limit = (dv(SELECT_PACK_W) - 80.0).max(1.0);
    let name = sketch::ellipsize(kit, &name_style, &input.pack_title(), limit);
    text_ink_center(
        kit,
        &name_style,
        cx,
        dv(SELECT_PACK_Y + SELECT_PACK_NAME_CY),
        &name,
        COL_WHITE,
    );

    let count = style_of(theme::FONT_LATIN, SELECT_PACK_COUNT_SIZE, COL_PLATE_INK);
    let count_center = dv(SELECT_PACK_Y + SELECT_PACK_COUNT_CY);
    let cells = [
        (135.0, input.clear_count.to_string()),
        (247.0, input.pure_count.to_string()),
        (293.0, String::from("/")),
        (312.0, input.total_count.to_string()),
    ];
    for (offset, value) in cells {
        text_ink_left(
            kit,
            &count,
            dv(SELECT_PACK_X) + offset,
            count_center,
            &value,
            COL_PLATE_INK,
        );
    }
}

fn paint_diff(kit: &CanvasKit<'_>, input: &SelectInput) {
    let levels = input.levels();
    let sel = diff_index(input.selected);
    let mut x = SELECT_DIFF_LEFT;
    let mut prev = false;
    let level_style = style_of(theme::FONT_TEXT, SELECT_DIFF_LEVEL_SIZE, COL_WHITE);
    for (index, slot) in SELECT_DIFF_ORDER.iter().copied().enumerate() {
        let has = levels
            .get(index)
            .map(|level| !level.trim().is_empty())
            .unwrap_or(false);
        if !has && index != sel {
            continue;
        }
        let stem = diff_selector_asset(index, slot);
        if stem.is_empty() {
            continue;
        }
        let width = diff_tab_w(index);
        if prev {
            x -= SELECT_DIFF_OVERLAP;
        }
        let key = if index == sel {
            format!("songselect/diff/{stem}_selected.png")
        } else {
            format!("songselect/diff/{stem}.png")
        };
        stretch_asset(kit, &key, area(x, SELECT_DIFF_TOP, width, SELECT_DIFF_H));
        if has {
            if let Some(level) = levels.get(index) {
                stroked_center(
                    kit,
                    &level_style,
                    dv(x + diff_tab_level_cx(index)),
                    dv(SELECT_DIFF_TOP + SELECT_DIFF_LEVEL_CY),
                    level.trim(),
                    StrokeInk {
                        fill: COL_WHITE,
                        stroke: COL_DIFF_LEVEL_INK,
                    },
                );
            }
        }
        x += width;
        prev = true;
    }
}

fn paint_info(kit: &CanvasKit<'_>, input: &SelectInput) {
    let Some(row) = input.selected_row() else {
        return;
    };
    let x = dv(SELECT_INFO_X);
    let limit = dv(SELECT_INFO_W);

    let title_style = style_of(theme::FONT_DISPLAY, SELECT_INFO_TITLE_SIZE, COL_WHITE);
    let title = sketch::ellipsize(kit, &title_style, row.title.trim(), limit);
    let title_y = dv(SELECT_INFO_TITLE_Y);
    soft_shadow_left(kit, &title_style, x, title_y, &title);
    stroked_left(
        kit,
        &title_style,
        x,
        title_y,
        &title,
        StrokeInk {
            fill: COL_WHITE,
            stroke: COL_INFO_STROKE,
        },
    );

    if !row.artist.trim().is_empty() {
        let artist_style = style_of(theme::FONT_DISPLAY, SELECT_INFO_ARTIST_SIZE, COL_WHITE);
        let artist = sketch::ellipsize(kit, &artist_style, row.artist.trim(), limit);
        let artist_y = dv(SELECT_INFO_ARTIST_Y);
        soft_shadow_left(kit, &artist_style, x, artist_y, &artist);
        stroked_left(
            kit,
            &artist_style,
            x,
            artist_y,
            &artist,
            StrokeInk {
                fill: COL_WHITE,
                stroke: COL_INFO_STROKE,
            },
        );
    }

    if !row.bpm.trim().is_empty() {
        let text = format!("BPM: {}", row.bpm.trim());
        let bpm_style = style_of(theme::FONT_LATIN, SELECT_INFO_BPM_SIZE, COL_WHITE);
        let bpm_y = dv(SELECT_INFO_BPM_Y);
        soft_shadow_left(kit, &bpm_style, x, bpm_y, &text);
        stroked_left(
            kit,
            &bpm_style,
            x,
            bpm_y,
            &text,
            StrokeInk {
                fill: COL_WHITE,
                stroke: COL_INFO_STROKE,
            },
        );
    }
}

fn paint_foot(kit: &CanvasKit<'_>) {
    let height = SELECT_BASE_H;
    let rect = device(
        dv(SELECT_BTN_CX - SELECT_BTN_W / 2.0) as f64,
        dv(height - SELECT_BTN_H) as f64,
        dv(SELECT_BTN_W) as f64,
        dv(SELECT_BTN_H) as f64,
    );
    stretch_asset(kit, "result/button/back.png", rect);
    let style = style_of(theme::FONT_DISPLAY, 22.0 * 0.93, COL_FOOT_INK);
    sketch::text_center(
        kit,
        &style,
        SELECT_FOOT_LABEL,
        dv(SELECT_BTN_LABEL_X),
        dv(height - SELECT_BTN_LABEL_UP),
        COL_FOOT_INK,
    );
}

fn paint_top_bar(kit: &CanvasKit<'_>, input: &SelectInput) {
    if let Ok(texture) = kit.asset("result/topbar/top_bar_bg.png") {
        if let Some(image) = texture.image() {
            let height = image.height() as f64 * SELECT_DEV_PER_DESIGN;
            if TOP_BAR_BG_RIGHT_DEV > TOP_BAR_BG_LEFT_DEV && height > 0.0 {
                kit.image_rect(
                    image,
                    widgets::rect(
                        TOP_BAR_BG_LEFT_DEV,
                        0.0,
                        TOP_BAR_BG_RIGHT_DEV - TOP_BAR_BG_LEFT_DEV,
                        height as f32,
                    ),
                );
            }
        }
    }

    let shift = SELECT_BAR_SHIFT;
    let title = style_of(theme::FONT_TEXT, 24.0, COL_FOOT_INK);
    text_ink_left(
        kit,
        &title,
        dv(18.0 + shift),
        dv(TOP_BAR_TEXT_ROW),
        &input.page_title(),
        COL_FOOT_INK,
    );
    text_ink_left(
        kit,
        &title,
        dv(875.0 + shift),
        dv(TOP_BAR_TEXT_ROW),
        "Generated by Okamikko",
        COL_FOOT_INK,
    );

    let Some(player) = &input.player else {
        return;
    };

    if !player.name.trim().is_empty() {
        let left = dv(NAME_PLATE_LEFT + shift);
        let plate_right = dv(NAME_PLATE_RIGHT + shift);
        let band = dv(TOP_BAND_H);
        let plates = widgets::rect(left, 0.0, plate_right - left, band);
        let (ink, _) = player::banner_plate(kit, plates, player.banner.as_deref(), false);
        let name = style_of(theme::FONT_LATIN, 26.0, ink);
        text_ink_center(
            kit,
            &name,
            dv((NAME_PLATE_LEFT + NAME_PLATE_RIGHT) / 2.0 + shift),
            dv(TOP_BAND_H / 2.0),
            player.name.trim(),
            ink,
        );
    }

    let avatar_cx = dv(NAME_PLATE_RIGHT + shift - 40.0);
    let avatar_cy = dv(32.0);
    let avatar_r = dv(40.0);
    let avatar = super::avatar::Avatar::new(player.avatar.as_deref())
        .with_global_rank(player.global_rank);
    super::avatar::draw_icon_and_border(kit, avatar_cx, avatar_cy, avatar_r, &avatar);
    let frame = if player.ptt > 0.0 && player.rating_frame >= 0 {
        player.rating_frame
    } else {
        -1
    };
    let badge = if frame >= 0 {
        format!("avatar/rating/rating_{frame}.png")
    } else {
        String::from("avatar/rating/rating_off.png")
    };
    let badge_area = widgets::rect(dv(656.0 + shift), dv(39.0), dv(45.0), dv(43.0));
    if let Some(crop) = sketch::rating_crop(&badge) {
        sketch::paste_crop_clip(kit, &badge, crop, badge_area);
    }
    if let Some(rank) = player.global_rank {
        super::avatar::draw_rank_text_at(kit, avatar_cx, avatar_cy, avatar_r, rank);
    }
    let (head, tail) = ptt_parts(player.ptt);
    let head_style = style_of(theme::FONT_NUMBER, 14.0, COL_WHITE);
    let tail_style = style_of(theme::FONT_NUMBER, 14.0 * 0.875, COL_WHITE);
    let head_width = sketch::text_width(kit, &head_style, &head);
    let ink_left = sketch::ink_box(kit, &head_style, &head)
        .map(|bounds| bounds.left())
        .unwrap_or(0.0);
    let ink_right = if tail.is_empty() {
        head_width
    } else {
        head_width
            + sketch::ink_box(kit, &tail_style, &tail)
                .map(|bounds| bounds.right())
                .unwrap_or(0.0)
    };

    let x = dv(678.5 + shift) - (ink_left + ink_right) / 2.0;
    let baseline = dv(60.5) + ink_offset(kit, &head_style, &head);
    for (dx, dy) in [
        (-1.0, 0.0),
        (1.0, 0.0),
        (0.0, -1.0),
        (0.0, 1.0),
        (-1.0, -1.0),
        (1.0, -1.0),
        (-1.0, 1.0),
        (1.0, 1.0),
    ] {
        sketch::text(kit, &head_style, &head, x + dx, baseline + dy, COL_PTT_EDGE);
        if !tail.is_empty() {
            sketch::text(
                kit,
                &tail_style,
                &tail,
                x + head_width + dx,
                baseline + dy,
                COL_PTT_EDGE,
            );
        }
    }
    sketch::text(kit, &head_style, &head, x, baseline, COL_WHITE);
    if !tail.is_empty() {
        sketch::text(kit, &tail_style, &tail, x + head_width, baseline, COL_WHITE);
    }
}

fn paint_bands(kit: &CanvasKit<'_>, width: f32, height: f32) {
    let left = dv(SELECT_INSET);
    let right = dv(SELECT_CONTENT_RIGHT);
    kit.fill_rect(
        widgets::rect(0.0, 0.0, left, height),
        Color::new(0, 0, 0, 255),
    );
    kit.fill_rect(
        widgets::rect(right, 0.0, width - right, height),
        Color::new(0, 0, 0, 255),
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::harness::{self, Harness};

    const PNG_MAGIC: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

    fn song() -> Song {
        Song::new("fractureray", "Fracture Ray")
            .with_artist("Sakuzyo")
            .with_bpm(160.0)
            .with_chart(Chart::new(Difficulty::Past, "6", 6.5).with_note(720))
            .with_chart(Chart::new(Difficulty::Present, "8", 8.4).with_note(1024))
            .with_chart(Chart::new(Difficulty::Future, "10+", 10.9).with_note(1337))
            .with_chart(Chart::new(Difficulty::Beyond, "11", 11.4).with_note(1520))
    }

    fn sample() -> SelectInput {
        SelectInput::new(&song(), Difficulty::Future)
            .with_score(Some(9_876_543))
            .with_grade(Some(Grade::ExPlus))
            .with_pack_stats(43, 12, 9, 120)
            .with_rows(vec![
                SelectRow::new("Fracture Ray", Difficulty::Past)
                    .with_song_id("fractureray")
                    .with_level("6", 6.5),
                SelectRow::new("Fracture Ray", Difficulty::Present)
                    .with_song_id("fractureray")
                    .with_level("8", 8.4),
                SelectRow::new("Testify", Difficulty::Future)
                    .with_song_id("testify")
                    .with_level("9+", 9.9)
                    .with_grade("EX+", SELECT_CLEAR_YES)
                    .with_bpm("158"),
                SelectRow::new("Fracture Ray", Difficulty::Future)
                    .with_song_id("fractureray")
                    .with_level("10+", 10.9)
                    .with_selected(true)
                    .with_grade("AA", SELECT_CLEAR_NO)
                    .with_bpm("160"),
                SelectRow::new("World Vanquisher", Difficulty::Future)
                    .with_song_id("worldvanquisher")
                    .with_level("9", 9.8),
                SelectRow::new("Acheron", Difficulty::Beyond)
                    .with_song_id("acheron")
                    .with_level("11", 11.2),
            ])
    }

    fn render(input: &SelectInput) -> Vec<u8> {
        let harness = Harness::new();
        let bytes = harness.render(request(input), |kit| paint(kit, input));
        assert!(bytes.starts_with(&PNG_MAGIC), "必须输出 PNG");
        assert!(!bytes.is_empty(), "渲染结果不能为空");
        bytes
    }

    #[test]
    fn case_101() {
        assert_eq!(dvi(SELECT_BASE_W), 2376);
        assert_eq!(dvi(SELECT_BASE_H), 1080);
        assert_eq!(dvi(SELECT_CONTENT_RIGHT), 2358);
        assert_eq!(dvi(SELECT_INSET), 18);
        assert_eq!(dvi(SELECT_TOP_BAR_H), 76);
        assert_eq!(SELECT_WIDTH, 2376);
        assert_eq!(SELECT_HEIGHT, 1080);
        let bytes = render(&sample());
        assert_eq!(harness::image_size(&bytes), Some((2376, 1080)));
    }
    #[test]
    fn case_102() {
        let levels = vec![
            String::from("6"),
            String::from("8"),
            String::from("10+"),
            String::new(),
            String::new(),
        ];
        let (left, right) = diff_span(&levels, Difficulty::Future);
        assert_eq!(left, SELECT_DIFF_LEFT);
        let expect = SELECT_DIFF_LEFT + diff_tab_w(0) + diff_tab_w(1) + diff_tab_w(2)
            - 2.0 * SELECT_DIFF_OVERLAP;
        assert!((right - expect).abs() < 1e-6, "实际 {right}, 期望 {expect}");
        let wide = score_tab_width(&levels, Difficulty::Future);
        assert!((wide - (right + SELECT_SCORE_TAIL_PAD)).abs() < 1e-6);
        let short = vec![
            String::from("6"),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        ];
        assert!((score_tab_width(&short, Difficulty::Past) - SELECT_SCORE_W).abs() < 1e-6);
    }

    #[test]
    fn case_103() {
        let levels = vec![
            String::from("5"),
            String::new(),
            String::new(),
            String::from("9"),
            String::new(),
        ];
        let (left, right) = diff_span(&levels, Difficulty::Eternal);
        assert_eq!(left, SELECT_DIFF_LEFT);
        let expect = SELECT_DIFF_LEFT + diff_tab_w(0) + diff_tab_w(3) - SELECT_DIFF_OVERLAP;
        assert!(
            (right - expect).abs() < 1e-6,
            "缺档不许留空位, 实际 {right}"
        );
        assert_eq!(diff_index(Difficulty::Eternal), 3);
        assert_eq!(diff_index(Difficulty::Beyond), 4);
        assert_eq!(
            diff_selector_asset(3, Difficulty::Eternal),
            "difficulty_selector_4"
        );
        assert_eq!(
            diff_selector_asset(4, Difficulty::Beyond),
            "difficulty_selector_3"
        );
        assert!((diff_tab_level_cx(0) - (diff_tab_w(0) / 2.0 + 41.4 / SELECT_DEV_PER_DESIGN)).abs() < 1e-9);
        assert!(
            (diff_tab_left(2)
                - (SELECT_DIFF_LEFT + diff_tab_w(0) + diff_tab_w(1) - 2.0 * SELECT_DIFF_OVERLAP))
                .abs()
                < 1e-9
        );
    }

    #[test]
    fn case_104() {
        assert_eq!(dv(SELECT_ROW_BAND_H), 148.0);
        assert_eq!(dv(SELECT_ROW_CAP_W), 155.0);
        assert_eq!(dv(SELECT_ROW_JACKET_W), 255.0);
        assert_eq!(dv(SELECT_ROW_LEVEL_SIZE), 62.0);
        assert_eq!(dv(SELECT_ROW_MARK_SIZE), 50.0);
        assert_eq!(dv(SELECT_INFO_TITLE_SIZE), 64.0);
        let top = SELECT_CELL_OY0 - 119.0 / SELECT_DEV_PER_DESIGN;
        assert_eq!(dvi(top), 57);
        let slot3 = SELECT_CELL_OY0 + 3.0 * SELECT_ROW_PITCH - 119.0 / SELECT_DEV_PER_DESIGN;
        assert_eq!(dvi(slot3), 633);
        let height = SELECT_ROW_BAND_H;
        let split = top + height * SELECT_ROW_SPLIT;
        assert!((cap_offset(top, top) - SELECT_ROW_CAP_TOP_SKEW * height).abs() < 1e-9);
        assert!(cap_offset(split, top).abs() < 1e-9);
        assert!((cap_offset(top + height, top) - SELECT_ROW_CAP_BOT_SKEW * height).abs() < 1e-9);
        assert_eq!(arc_offset(SELECT_SEL_SLOT), 0.0);
        assert!(arc_offset(2) > 0.0, "离选中行一格就开始往右退");
        assert!(arc_offset(1) > arc_offset(2), "离得越远退得越多");
        assert!(arc_offset(5) > arc_offset(4));
    }

    #[test]
    fn case_105() {
        assert_eq!(SELECT_ROWS, 6);
        assert_eq!(SELECT_SEL_SLOT, 3);
        let rows = sample().rows_value();
        assert_eq!(rows.len(), 6);
        let sel = rows
            .iter()
            .position(|row| row.selected)
            .expect("必须有选中行");
        assert_eq!(sel, 3);
        assert_eq!(rows[0].title, "Fracture Ray");
        assert_eq!(rows[3].level_text(), "10+");
        assert_eq!(
            SelectRow::new("x", Difficulty::Future).level_text(),
            SELECT_NONE
        );
    }

    #[test]
    fn case_106() {
        assert_eq!(
            difficulty_asset(Difficulty::Future, true),
            "songselect/diff/difficulty_selector_2_selected.png"
        );
        assert_eq!(
            difficulty_asset(Difficulty::Beyond, false),
            "songselect/diff/difficulty_selector_3.png"
        );
        assert_eq!(
            cell_asset(Difficulty::Beyond, true),
            "songselect/cell/song_cell_right_beyond_selected.png"
        );
        assert_eq!(
            cell_asset(Difficulty::Future, false),
            "songselect/cell/song_cell_right.png"
        );
        assert_eq!(
            piece_asset(Difficulty::Beyond),
            "songselect/cell/song_cell_selected_piece_beyond.png"
        );
        assert_eq!(grade_asset(Grade::ExPlus), "songselect/grade/explus.png");
        assert_eq!(grade_asset_text("aa"), "songselect/grade/aa.png");
        assert_eq!(grade_asset_text("S"), "");
    }

    #[test]
    fn case_107() {
        let tint = diff_tint(Difficulty::Future);
        assert_eq!(tint, Color::new(149, 55, 119, 235));
        let dark = darker_tint(tint);
        assert_eq!(dark.r, (149 * 35 / 100 + 20) as u8);
        assert_eq!(dark.g, (55 * 40 / 100 + 30) as u8);
        assert_eq!(dark.b, (119 * 45 / 100 + 60) as u8);
        assert_eq!(dark.a, 235);
        assert_eq!(mark_ink("EX+", '\0'), Color::new(140, 70, 160, 255));
        assert_eq!(mark_ink("", SELECT_CLEAR_NO), Color::new(140, 45, 59, 255));
        assert_eq!(
            mark_ink("", SELECT_CLEAR_YES),
            Color::new(45, 122, 120, 255)
        );
    }

    #[test]
    fn case_108() {
        assert_eq!(sample().score_text(), sketch::thousands(9_876_543));
        let mut input = sample();
        input.score = None;
        assert_eq!(input.score_text(), SELECT_SCORE_PLACEHOLDER);
        input.score = Some(0);
        assert_eq!(input.score_text(), SELECT_SCORE_PLACEHOLDER);
    }
    #[test]
    fn case_109() {
        let input = sample();
        let bytes = render(&input);
        let pixels = harness::decode(&bytes).expect("解码选曲界面");
        for (x, label) in [(4u32, "左"), (2371, "右")] {
            let pixel = pixels.at(x, 540);
            assert!(
                pixel[0] < 12 && pixel[1] < 12 && pixel[2] < 12,
                "{label}缘必须是黑边, 实际 {pixel:?}"
            );
        }
        let hero_x = dv(SELECT_HERO_CX - SELECT_HERO_SIDE / 2.0) as u32;
        let hero_y = dv(SELECT_HERO_CY - SELECT_HERO_SIDE / 2.0) as u32;
        let hero = pixels.ink_ratio(hero_x, hero_y, 300, 300, 9);
        assert!(hero > 0.0, "大曲绘必须画出来");
        let rows = pixels.ink_ratio(1500, 200, 800, 700, 9);
        assert!(rows > 0.01, "右列行卡必须有内容, 实际 {rows}");
        let pack = pixels.ink_ratio(
            dv(SELECT_PACK_X) as u32,
            dv(SELECT_PACK_Y) as u32,
            430,
            160,
            5,
        );
        assert!(pack > 0.04, "曲包牌必须有内容, 实际 {pack}");
        let tab = pixels.ink_ratio(0, dv(SELECT_DIFF_TOP) as u32, 700, 180, 5);
        assert!(tab > 0.10, "难度条必须有内容, 实际 {tab}");
        let info = pixels.ink_ratio(dv(SELECT_INFO_X) as u32, 400, 400, 200, 5);
        assert!(info > 0.0, "左侧曲目信息必须画出来");
    }

    #[test]
    fn case_110() {
        let mut input = SelectInput::new(&Song::new("", ""), Difficulty::Future);
        input.jacket = None;
        input.rows = Some(Vec::new());
        let bytes = render(&input);
        assert_eq!(harness::image_size(&bytes), Some((2376, 1080)));
        let pixels = harness::decode(&bytes).expect("解码空选曲界面");
        assert!(!pixels.blank(), "底色与底栏必须画出来");
        assert!(input.rows_value().iter().all(|row| row.selected));
    }

    #[test]
    fn case_111() {
        let mut input = sample();
        input.title = "A".repeat(120);
        input.rows = None;
        let bytes = render(&input);
        assert_eq!(harness::image_size(&bytes), Some((2376, 1080)));
        let pixels = harness::decode(&bytes).expect("解码长曲名选曲界面");
        assert!(!pixels.blank());
        assert!(input.rows_value().iter().all(|row| !row.title.is_empty()));
    }
}
