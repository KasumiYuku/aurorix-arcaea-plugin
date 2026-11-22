use chrono::{DateTime, Utc};

use crate::domain::ptt;
use crate::model::{Difficulty, Grade};

pub const BOARD_TOP: usize = 50;

pub const OVER_PASS_SCORE: i64 = 9_800_000;
pub const OVER_HELD_SCORE: i64 = 9_900_000;

pub const OVER_FLOOR: i64 = 7_000_000;

pub const OVER_DISASTER_SCORE: i64 = 9_200_000;

pub const OVER_DISASTER_SPAN_SCORE: i64 = 9_350_000;

pub const OVER_HEAVY_SCORE: i64 = 9_500_000;

pub const OVER_SPAN_MIN: f64 = 0.05;

pub const OVER_SPAN_HEAVY: f64 = 0.8;

pub const OVER_SPAN_DISASTER: f64 = 1.0;

pub const OVER_UNDERPERFORM_FLOOR: f64 = -0.5;

pub const SPAN_EPS: f64 = 1e-9;

pub const SO_CLOSE_SUM: i64 = 2;

pub const SO_CLOSE_FLOOR: i64 = 1;

pub const PRESSURE_COLLAPSE_FULL: f64 = 800_000.0;

pub const CLEAR_FULL_RECALL: i32 = 2;

pub const CLEAR_PURE_MEMORY: i32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BoardKind {
    Pm50,
    Fr50,
    X50,
    Nm50,
    Mx50,
    Ol50,
}

impl BoardKind {
    pub const ALL: [BoardKind; 6] = [
        BoardKind::Pm50,
        BoardKind::Fr50,
        BoardKind::X50,
        BoardKind::Nm50,
        BoardKind::Mx50,
        BoardKind::Ol50,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            BoardKind::Pm50 => "pm50",
            BoardKind::Fr50 => "fr50",
            BoardKind::X50 => "x50",
            BoardKind::Nm50 => "nm50",
            BoardKind::Mx50 => "mx50",
            BoardKind::Ol50 => "ol50",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        let trimmed = text.trim();
        BoardKind::ALL
            .into_iter()
            .find(|kind| kind.as_str() == trimmed)
    }
}

impl std::fmt::Display for BoardKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OverBand {
    Disaster,
    Heavy,
    Mild,
    Held,
    Underperform,
    Normal,
}

impl OverBand {
    pub const ALL: [OverBand; 6] = [
        OverBand::Disaster,
        OverBand::Heavy,
        OverBand::Mild,
        OverBand::Held,
        OverBand::Underperform,
        OverBand::Normal,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            OverBand::Disaster => "disaster",
            OverBand::Heavy => "heavy",
            OverBand::Mild => "mild",
            OverBand::Held => "held",
            OverBand::Underperform => "underperform",
            OverBand::Normal => "normal",
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            OverBand::Disaster => "打不动",
            OverBand::Heavy => "大越",
            OverBand::Mild => "小越",
            OverBand::Held => "攻克",
            OverBand::Underperform => "翻车",
            OverBand::Normal => "达标",
        }
    }

    pub fn severity(self) -> i8 {
        match self {
            OverBand::Disaster => 5,
            OverBand::Heavy => 4,
            OverBand::Mild => 3,
            OverBand::Held => 2,
            OverBand::Underperform => 1,
            OverBand::Normal => 0,
        }
    }

    pub fn is_over(self) -> bool {
        self.severity() >= 2
    }

    pub fn parse(text: &str) -> Option<Self> {
        let lowered = text.trim().to_ascii_lowercase();
        let lowered = if lowered == "sustain" {
            "heavy".to_string()
        } else {
            lowered
        };
        OverBand::ALL
            .into_iter()
            .find(|band| band.as_str() == lowered)
    }
}

impl std::fmt::Display for OverBand {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

pub fn band_severity(band: Option<OverBand>) -> i8 {
    band.map(OverBand::severity).unwrap_or(-1)
}

pub fn band_of(span: f64, score: i64) -> Option<OverBand> {
    if span + SPAN_EPS < OVER_SPAN_MIN {
        if score >= OVER_PASS_SCORE {
            return Some(OverBand::Normal);
        }
        if span + SPAN_EPS >= OVER_UNDERPERFORM_FLOOR {
            return Some(OverBand::Underperform);
        }
        return None;
    }
    if score >= OVER_HELD_SCORE {
        return Some(OverBand::Held);
    }
    if score >= OVER_PASS_SCORE {
        return Some(OverBand::Normal);
    }
    if score < OVER_DISASTER_SCORE
        || (span + SPAN_EPS >= OVER_SPAN_DISASTER && score < OVER_DISASTER_SPAN_SCORE)
    {
        return Some(OverBand::Disaster);
    }
    if span + SPAN_EPS >= OVER_SPAN_HEAVY || score < OVER_HEAVY_SCORE {
        return Some(OverBand::Heavy);
    }
    Some(OverBand::Mild)
}

pub fn span_of(constant: f64, anchor: f64) -> f64 {
    ptt::round9(constant + ptt::PASS_BONUS - (anchor + ptt::PASS_BONUS))
}

pub fn pressure_of(score: i64) -> f64 {
    let shortfall = (OVER_PASS_SCORE - score) as f64 / PRESSURE_COLLAPSE_FULL;
    let clamped = shortfall.clamp(0.0, 1.0);
    ptt::round9(100.0 * clamped)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BandCounts {
    pub disaster: usize,
    pub heavy: usize,
    pub mild: usize,
    pub held: usize,
    pub underperform: usize,
    pub normal: usize,
}

impl BandCounts {
    pub fn add(&mut self, band: OverBand) {
        match band {
            OverBand::Disaster => self.disaster += 1,
            OverBand::Heavy => self.heavy += 1,
            OverBand::Mild => self.mild += 1,
            OverBand::Held => self.held += 1,
            OverBand::Underperform => self.underperform += 1,
            OverBand::Normal => self.normal += 1,
        }
    }

    pub fn get(&self, band: OverBand) -> usize {
        match band {
            OverBand::Disaster => self.disaster,
            OverBand::Heavy => self.heavy,
            OverBand::Mild => self.mild,
            OverBand::Held => self.held,
            OverBand::Underperform => self.underperform,
            OverBand::Normal => self.normal,
        }
    }

    pub fn over_count(&self) -> usize {
        self.disaster + self.heavy + self.mild + self.held
    }

    pub fn total(&self) -> usize {
        self.over_count() + self.underperform + self.normal
    }

    pub fn entries(&self) -> Vec<(OverBand, usize)> {
        OverBand::ALL
            .into_iter()
            .map(|band| (band, self.get(band)))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoardRecord {
    pub song_id: String,
    pub title: String,
    pub difficulty: Difficulty,
    pub constant: f64,
    pub has_constant: bool,
    pub score: i64,
    pub grade: Option<Grade>,
    pub clear_type: Option<i32>,
    pub far: i64,
    pub lost: i64,
    pub normal_perfect: i64,
    pub object_amount: i64,
    pub has_judgement: bool,
    pub play_at: Option<DateTime<Utc>>,
    pub over_level: Option<f64>,
    pub over_band: Option<OverBand>,
    pub over_mess: Option<f64>,
    pub over_delta: Option<f64>,
    pub over_quantum: Option<f64>,
    pub over_far_share: Option<f64>,
    pub over_lost_share: Option<f64>,
    pub has_over_level: bool,
    pub backend_pressure: Option<f64>,
    pub backend_rank: Option<i64>,
    pub backend_rating: Option<f64>,
    pub backend_rating_frame: Option<i32>,
    pub next_rank: String,
    pub next_need: i64,
    pub lose_score: f64,
    pub equivalent_far: i64,
}

impl BoardRecord {
    pub fn new(
        song_id: impl Into<String>,
        title: impl Into<String>,
        difficulty: Difficulty,
        constant: f64,
    ) -> Self {
        Self {
            song_id: song_id.into(),
            title: title.into(),
            difficulty,
            constant,
            has_constant: constant > 0.0,
            score: 0,
            grade: None,
            clear_type: None,
            far: 0,
            lost: 0,
            normal_perfect: 0,
            object_amount: 0,
            has_judgement: false,
            play_at: None,
            over_level: None,
            over_band: None,
            over_mess: None,
            over_delta: None,
            over_quantum: None,
            over_far_share: None,
            over_lost_share: None,
            has_over_level: false,
            backend_pressure: None,
            backend_rank: None,
            backend_rating: None,
            backend_rating_frame: None,
            next_rank: String::new(),
            next_need: 0,
            lose_score: 0.0,
            equivalent_far: 0,
        }
    }

    pub fn with_score(mut self, score: i64) -> Self {
        self.score = score;
        self
    }

    pub fn with_judgement(mut self, far: i64, lost: i64, object_amount: i64) -> Self {
        self.far = far;
        self.lost = lost;
        self.object_amount = object_amount;
        self.has_judgement = true;
        self
    }

    pub fn with_clear(mut self, clear_type: i32) -> Self {
        self.clear_type = Some(clear_type);
        self
    }

    pub fn with_grade(mut self, grade: Grade) -> Self {
        self.grade = Some(grade);
        self
    }

    pub fn with_play_at(mut self, play_at: DateTime<Utc>) -> Self {
        self.play_at = Some(play_at);
        self
    }

    pub fn rating(&self) -> f64 {
        if !self.has_constant || self.constant <= 0.0 {
            return 0.0;
        }
        ptt::single_rating(self.constant, self.score)
    }

    pub fn is_pm(&self) -> bool {
        if let Some(clear_type) = self.clear_type {
            return clear_type == CLEAR_PURE_MEMORY;
        }
        self.has_judgement && self.far == 0 && self.lost == 0
    }

    pub fn is_fr(&self) -> bool {
        if let Some(clear_type) = self.clear_type {
            return clear_type == CLEAR_PURE_MEMORY || clear_type == CLEAR_FULL_RECALL;
        }
        self.has_judgement && self.lost == 0
    }

    pub fn is_so_close(&self) -> bool {
        if !self.has_judgement {
            return false;
        }
        let misses = self.far + self.lost;
        (SO_CLOSE_FLOOR..=SO_CLOSE_SUM).contains(&misses)
    }

    pub fn has_usable_constant(&self) -> bool {
        self.has_constant && self.constant > 0.0
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BoardInput {
    pub records: Vec<BoardRecord>,
    pub anchor: Option<f64>,
    pub has_curve: bool,
    pub anchor_upper: f64,
    pub over_count: Option<usize>,
    pub band_counts: Option<BandCounts>,
    pub mean_pressure: f64,
    pub avg_best10: Option<f64>,
    pub avg_best50: Option<f64>,
    pub best_count: Option<usize>,
    pub mean_gap: Option<f64>,
}

impl BoardInput {
    pub fn new(records: Vec<BoardRecord>) -> Self {
        Self {
            records,
            ..Self::default()
        }
    }

    pub fn with_anchor(mut self, anchor: f64) -> Self {
        self.anchor = Some(anchor);
        self
    }

    pub fn with_curve(mut self, has_curve: bool) -> Self {
        self.has_curve = has_curve;
        self
    }

    pub fn with_upper(mut self, anchor_upper: f64) -> Self {
        self.anchor_upper = anchor_upper;
        self
    }

    pub fn with_counts(mut self, band_counts: BandCounts, over_count: usize) -> Self {
        self.band_counts = Some(band_counts);
        self.over_count = Some(over_count);
        self
    }

    pub fn with_pressure(mut self, mean_pressure: f64) -> Self {
        self.mean_pressure = mean_pressure;
        self
    }

    pub fn with_averages(
        mut self,
        avg_best10: Option<f64>,
        avg_best50: Option<f64>,
        best_count: usize,
    ) -> Self {
        self.avg_best10 = avg_best10.filter(|value| *value > 0.0);
        self.avg_best50 = avg_best50.filter(|value| *value > 0.0);
        if best_count > 0 {
            self.best_count = Some(best_count);
        }
        self
    }

    pub fn with_mean_gap(mut self, mean_gap: f64) -> Self {
        if mean_gap > 0.0 {
            self.mean_gap = Some(mean_gap);
        }
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoardRow {
    pub rank: usize,
    pub song_id: String,
    pub title: String,
    pub difficulty: Difficulty,
    pub constant: f64,
    pub has_constant: bool,
    pub score: i64,
    pub rating: f64,
    pub grade: Grade,
    pub far: i64,
    pub lost: i64,
    pub normal_perfect: i64,
    pub object_amount: i64,
    pub clear_type: Option<i32>,
    pub has_judgement: bool,
    pub band: Option<OverBand>,
    pub span: f64,
    pub pressure: f64,
    pub play_at: Option<DateTime<Utc>>,
    pub mess: f64,
    pub over_delta: f64,
    pub has_over_level: bool,
    pub over_level: Option<f64>,
    pub over_band: Option<OverBand>,
    pub over_mess: Option<f64>,
    pub backend_pressure: Option<f64>,
}

impl BoardRow {
    pub fn from_record(record: &BoardRecord, rank: usize) -> Self {
        Self {
            rank,
            song_id: record.song_id.clone(),
            title: record.title.clone(),
            difficulty: record.difficulty,
            constant: record.constant,
            has_constant: record.has_constant,
            score: record.score,
            rating: record
                .backend_rating
                .unwrap_or_else(|| record.rating()),
            grade: record.grade.unwrap_or_else(|| Grade::from_score(record.score)),
            far: record.far,
            lost: record.lost,
            normal_perfect: record.normal_perfect,
            object_amount: record.object_amount,
            clear_type: record.clear_type,
            has_judgement: record.has_judgement,
            band: None,
            span: 0.0,
            pressure: pressure_of(record.score),
            play_at: record.play_at,
            mess: record.over_mess.unwrap_or(0.0),
            over_delta: record.over_delta.unwrap_or(0.0),
            has_over_level: record.has_over_level,
            over_level: record.over_level,
            over_band: record.over_band,
            over_mess: record.over_mess,
            backend_pressure: record.backend_pressure,
        }
    }

    pub fn band_word(&self) -> &'static str {
        self.band.map(OverBand::word).unwrap_or("")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    pub kind: BoardKind,
    pub matched: usize,
    pub known: usize,
    pub anchor: Option<f64>,
    pub rows: Vec<BoardRow>,
    pub band_counts: BandCounts,
    pub over_count: usize,
    pub avg_best10: Option<f64>,
    pub avg_best50: Option<f64>,
    pub best_count: Option<usize>,
    pub mean_gap: Option<f64>,
}

impl Board {
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

pub fn board_match(kind: BoardKind, record: &BoardRecord) -> bool {
    match kind {
        BoardKind::Pm50 => record.is_pm(),
        BoardKind::Fr50 => record.is_fr(),
        BoardKind::X50 => record.is_so_close(),
        BoardKind::Nm50 => record.has_judgement && !record.is_pm(),
        BoardKind::Mx50 => record.score >= ptt::FULL_SCORE,
        BoardKind::Ol50 => record.has_usable_constant() && record.score >= OVER_FLOOR,
    }
}

fn tie_break(left: &BoardRow, right: &BoardRow) -> std::cmp::Ordering {
    left.song_id
        .cmp(&right.song_id)
        .then_with(|| left.difficulty.cmp(&right.difficulty))
}

fn compare_by_rating(left: &BoardRow, right: &BoardRow) -> std::cmp::Ordering {
    right
        .rating
        .total_cmp(&left.rating)
        .then_with(|| tie_break(left, right))
}

pub fn select_board(input: &BoardInput, kind: BoardKind, limit: usize) -> Board {
    let limit = if limit == 0 || limit > BOARD_TOP {
        BOARD_TOP
    } else {
        limit
    };
    let known = input.records.len();
    let matched = input
        .records
        .iter()
        .filter(|record| board_match(kind, record))
        .count();
    let mut board = Board {
        kind,
        matched,
        known,
        anchor: None,
        rows: Vec::new(),
        band_counts: BandCounts::default(),
        over_count: 0,
        avg_best10: input.avg_best10,
        avg_best50: input.avg_best50,
        best_count: input.best_count,
        mean_gap: input.mean_gap,
    };
    if limit == 0 {
        return board;
    }
    let mut candidates: Vec<&BoardRecord> = input
        .records
        .iter()
        .filter(|record| board_match(kind, record))
        .collect();
    let backend_order = !candidates.is_empty()
        && candidates
            .iter()
            .all(|record| record.backend_rank.is_some());
    if backend_order {
        candidates.sort_by_key(|record| record.backend_rank.unwrap_or(i64::MAX));
    }
    let mut picked: Vec<BoardRow> = candidates
        .iter()
        .map(|record| BoardRow::from_record(record, 0))
        .collect();

    match kind {
        BoardKind::Nm50 => {
            picked.sort_by(|left, right| {
                left.lost
                    .cmp(&right.lost)
                    .then_with(|| left.far.cmp(&right.far))
                    .then_with(|| tie_break(left, right))
            });
            picked.truncate(limit);
            picked.sort_by(compare_by_rating);
        }
        BoardKind::Ol50 => {
            if input.has_curve {
                for row in picked.iter_mut() {
                    row.span = row.over_level.unwrap_or(0.0);
                    row.band = row.over_band.or_else(|| band_of(row.span, row.score));
                    row.mess = row.over_mess.unwrap_or(0.0);
                }
                board.anchor = input.anchor;
            } else {
                for row in picked.iter_mut() {
                    row.span = 0.0;
                    row.band = None;
                    row.mess = 0.0;
                    row.has_over_level = false;
                    if let Some(pressure) = row.backend_pressure {
                        row.pressure = pressure;
                    }
                }
            }
        }
        _ => {
            if !backend_order {
                picked.sort_by(compare_by_rating);
            }
        }
    }

    board.band_counts = input.band_counts.unwrap_or_else(|| {
        let mut counts = BandCounts::default();
        for row in picked.iter() {
            if let Some(band) = row.band {
                counts.add(band);
            }
        }
        counts
    });
    board.over_count = input
        .over_count
        .unwrap_or_else(|| board.band_counts.over_count());
    picked.truncate(limit);
    for (index, row) in picked.iter_mut().enumerate() {
        row.rank = index + 1;
    }
    board.rows = picked;
    board
}

pub fn select_pm50(input: &BoardInput) -> Board {
    select_board(input, BoardKind::Pm50, BOARD_TOP)
}

pub fn select_fr50(input: &BoardInput) -> Board {
    select_board(input, BoardKind::Fr50, BOARD_TOP)
}

pub fn select_x50(input: &BoardInput) -> Board {
    select_board(input, BoardKind::X50, BOARD_TOP)
}

pub fn select_nm50(input: &BoardInput) -> Board {
    select_board(input, BoardKind::Nm50, BOARD_TOP)
}

pub fn select_mx50(input: &BoardInput) -> Board {
    select_board(input, BoardKind::Mx50, BOARD_TOP)
}

pub fn select_ol50(input: &BoardInput) -> Board {
    select_board(input, BoardKind::Ol50, BOARD_TOP)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(song: &str, difficulty: Difficulty, constant: f64, score: i64) -> BoardRecord {
        BoardRecord::new(song, song.to_uppercase(), difficulty, constant)
            .with_score(score)
            .with_judgement(0, 0, 1000)
    }

    #[test]
    fn case_310() {
        for kind in BoardKind::ALL {
            assert_eq!(BoardKind::parse(kind.as_str()), Some(kind));
        }
        assert_eq!(BoardKind::parse("pm50"), Some(BoardKind::Pm50));
        assert_eq!(BoardKind::parse("Pm50"), None);
        assert_eq!(BoardKind::parse("b50"), None);
    }

    #[test]
    fn case_311() {
        let pm = record("a", Difficulty::Future, 10.0, ptt::FULL_SCORE);
        assert!(pm.is_pm());
        assert!(pm.is_fr());
        let one_far = BoardRecord::new("b", "B", Difficulty::Future, 10.0)
            .with_score(ptt::FULL_SCORE)
            .with_judgement(1, 0, 1000);
        assert!(!one_far.is_pm(), "满分带一粒 far 不是 PM");
        assert!(one_far.is_fr());
        let lost = BoardRecord::new("c", "C", Difficulty::Future, 10.0)
            .with_score(9_000_000)
            .with_judgement(0, 1, 1000);
        assert!(!lost.is_fr());
        assert!(lost.is_so_close(), "1 Lost 0 Far 是差一口气");
        assert!(one_far.is_so_close(), "0 Lost 1 Far 是差一口气");
        let both = BoardRecord::new("d", "D", Difficulty::Future, 10.0)
            .with_score(9_000_000)
            .with_judgement(1, 1, 1000);
        assert!(both.is_so_close(), "1 Far + 1 Lost = 2, 正好到上限");
        let two_far = BoardRecord::new("i", "I", Difficulty::Future, 10.0)
            .with_score(9_000_000)
            .with_judgement(2, 0, 1000);
        assert!(two_far.is_so_close(), "2 Far 正好到上限");
        let far_heavy = BoardRecord::new("e", "E", Difficulty::Future, 10.0)
            .with_score(9_000_000)
            .with_judgement(20, 1, 1000);
        assert!(!far_heavy.is_so_close(), "1 Lost 带 20 Far, 远超上限");
        let three_lost = BoardRecord::new("f", "F", Difficulty::Future, 10.0)
            .with_score(9_000_000)
            .with_judgement(0, 3, 1000);
        assert!(!three_lost.is_so_close(), "3 Lost 超过上限");
        let four_lost = BoardRecord::new("g", "G", Difficulty::Future, 10.0)
            .with_score(9_000_000)
            .with_judgement(0, 4, 1000);
        assert!(!four_lost.is_so_close(), "4 Lost 超过上限");
        let six_far = BoardRecord::new("h", "H", Difficulty::Future, 10.0)
            .with_score(9_000_000)
            .with_judgement(6, 0, 1000);
        assert!(!six_far.is_so_close(), "6 Far 超过上限");
    }

    #[test]
    fn case_312() {
        let full = BoardRecord::new("a", "A", Difficulty::Future, 10.0)
            .with_score(9_990_000)
            .with_clear(CLEAR_FULL_RECALL);
        assert!(full.is_fr(), "clear_type 说 FR 就算 FR");
        assert!(!full.is_pm(), "FR 不是 PM");
        let pure = BoardRecord::new("a", "A", Difficulty::Future, 10.0)
            .with_score(ptt::FULL_SCORE)
            .with_clear(CLEAR_PURE_MEMORY);
        assert!(pure.is_pm());
    }

    #[test]
    fn case_313() {
        let input = BoardInput::new(vec![
            record("z_song", Difficulty::Future, 10.0, ptt::FULL_SCORE),
            record("a_song", Difficulty::Present, 8.0, ptt::FULL_SCORE),
            record("a_song", Difficulty::Future, 10.0, ptt::FULL_SCORE),
            record("m_song", Difficulty::Past, 5.0, 9_999_999),
        ]);
        let board = select_pm50(&input);
        assert_eq!(board.matched, 4);
        let ids: Vec<(String, Difficulty)> = board
            .rows
            .iter()
            .map(|row| (row.song_id.clone(), row.difficulty))
            .collect();
        assert_eq!(
            ids,
            vec![
                ("a_song".to_string(), Difficulty::Future),
                ("z_song".to_string(), Difficulty::Future),
                ("a_song".to_string(), Difficulty::Present),
                ("m_song".to_string(), Difficulty::Past),
            ],
            "同分时按 (song_id, difficulty) 决胜，结果必须唯一"
        );
        assert_eq!(board.rows[0].rank, 1);
    }

    #[test]
    fn case_314() {
        let build = |order: [usize; 4]| {
            let all = [
                record("d", Difficulty::Future, 10.0, ptt::FULL_SCORE),
                record("c", Difficulty::Future, 10.0, ptt::FULL_SCORE),
                record("b", Difficulty::Future, 10.0, ptt::FULL_SCORE),
                record("a", Difficulty::Future, 10.0, ptt::FULL_SCORE),
            ];
            let reordered: Vec<BoardRecord> =
                order.iter().map(|index| all[*index].clone()).collect();
            select_mx50(&BoardInput::new(reordered))
                .rows
                .into_iter()
                .map(|row| row.song_id)
                .collect::<Vec<String>>()
        };
        assert_eq!(
            build([0, 1, 2, 3]),
            build([3, 2, 1, 0]),
            "同样的分数换一种输入顺序，输出必须完全一样"
        );
        assert_eq!(build([1, 3, 0, 2]), vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn case_315() {
        let input = BoardInput::new(vec![
            BoardRecord::new("one_far", "A", Difficulty::Future, 10.0)
                .with_score(9_990_000)
                .with_judgement(1, 0, 1000),
            BoardRecord::new("one_lost", "B", Difficulty::Future, 10.0)
                .with_score(9_980_000)
                .with_judgement(0, 1, 1000),
            BoardRecord::new("two_far", "C", Difficulty::Future, 10.0)
                .with_score(9_970_000)
                .with_judgement(2, 0, 1000),
            BoardRecord::new("no_judge", "D", Difficulty::Future, 10.0).with_score(9_960_000),
        ]);
        let board = select_x50(&input);
        assert_eq!(board.matched, 3, "两粒 Far 折成 1, 也在上限内");
        assert_eq!(board.rows.len(), 3);
        assert_eq!(board.rows[0].song_id, "one_far");
        assert_eq!(board.rows[2].song_id, "two_far");
    }

    #[test]
    fn case_316() {
        let mut records = Vec::new();
        for index in 0..60 {
            records.push(
                BoardRecord::new(format!("song{index:02}"), "T", Difficulty::Future, 10.0)
                    .with_score(9_900_000 - index as i64 * 1000)
                    .with_judgement(1, index as i64, 1000),
            );
        }
        let board = select_nm50(&BoardInput::new(records));
        assert_eq!(board.matched, 60);
        assert_eq!(board.rows.len(), BOARD_TOP, "最多只留 50 条");
        assert!(
            board.rows.iter().all(|row| row.lost <= 49),
            "只保留离满分最近的 50 条"
        );
        let order: Vec<i64> = board
            .rows
            .iter()
            .map(|row| (row.rating * 1e6).round() as i64)
            .collect();
        let mut sorted = order.clone();
        sorted.sort_by(|left, right| right.cmp(left));
        assert_eq!(order, sorted, "取完前 50 条之后才按单曲 PTT 降序摊开");
    }

    #[test]
    fn case_317() {
        let input = BoardInput::new(vec![
            record("max", Difficulty::Future, 10.0, ptt::FULL_SCORE),
            record("near", Difficulty::Future, 10.0, ptt::FULL_SCORE - 1),
        ]);
        let board = select_mx50(&input);
        assert_eq!(board.rows.len(), 1);
        assert_eq!(board.rows[0].song_id, "max");
    }

    #[test]
    fn case_318() {
        assert_eq!(band_of(0.3, 9_900_000), Some(OverBand::Held));
        assert_eq!(
            band_of(0.3, 9_899_999),
            Some(OverBand::Normal),
            "EX 到 EX+ 之间是达标"
        );
        assert_eq!(
            band_of(0.3, 9_800_000),
            Some(OverBand::Normal),
            "越级上 EX 记达标，不许掉进大越打不动"
        );
        assert_eq!(band_of(0.3, 9_790_000), Some(OverBand::Mild));
        assert_eq!(band_of(0.3, 9_450_000), Some(OverBand::Heavy));
        assert_eq!(band_of(0.3, 9_100_000), Some(OverBand::Disaster));
        assert_eq!(band_of(0.85, 9_600_000), Some(OverBand::Heavy));
        assert_eq!(band_of(1.2, 9_340_000), Some(OverBand::Disaster));
        assert_eq!(band_of(1.2, 9_400_000), Some(OverBand::Heavy));
        assert_eq!(band_of(-0.2, 9_700_000), Some(OverBand::Underperform));
        assert_eq!(band_of(-0.2, 9_800_000), Some(OverBand::Normal));
        assert_eq!(band_of(-0.6, 9_400_000), None, "线下一大截又没 EX 的不进榜");
        assert_eq!(band_of(0.0499, 9_700_000), Some(OverBand::Underperform));
        assert_eq!(
            band_of(0.05, 9_700_000),
            Some(OverBand::Mild),
            "0.05 是越级跨度下限"
        );
        assert_eq!(
            band_of(0.02, 9_950_000),
            Some(OverBand::Normal),
            "跨度没到 0.05 就是未越级: 打 EX+ 也只是达标, 不是攻克"
        );
        assert_eq!(
            band_of(0.0, 9_900_000),
            Some(OverBand::Normal),
            "跨度 0 的 EX+ 同样只是达标 —— 攻克只属于跨过能力线的局"
        );
        assert_eq!(
            band_of(0.02, 9_850_000),
            Some(OverBand::Normal),
            "非越级跨度上只拿 EX 仍是达标"
        );
        assert_eq!(
            band_of(0.02, 9_700_000),
            Some(OverBand::Underperform),
            "非越级跨度上没 EX 仍是翻车"
        );
        assert_eq!(
            band_of(-0.6, 9_950_000),
            Some(OverBand::Normal),
            "线下一大截的 EX+ 仍是达标, 不是攻克"
        );
        assert_eq!(band_of(-0.6, 9_400_000), None);
    }

    #[test]
    fn case_319() {
        assert_eq!(OverBand::Disaster.severity(), 5);
        assert_eq!(OverBand::Normal.severity(), 0);
        assert_eq!(OverBand::Normal.word(), "达标", "上 EX 记达标");
        assert!(OverBand::Held.is_over(), "攻克算真越级");
        assert!(!OverBand::Underperform.is_over(), "翻车不算越级");
        assert_eq!(band_severity(None), -1);
        assert_eq!(
            OverBand::parse("sustain"),
            Some(OverBand::Heavy),
            "sustain 是 heavy 的旧名"
        );
    }

    #[test]
    fn case_320() {
        let mut mild = record("mild", Difficulty::Future, 10.3, 9_790_000);
        mild.over_level = Some(0.3);
        mild.over_band = Some(OverBand::Mild);
        mild.over_mess = Some(0.42);
        mild.over_delta = Some(0.12);
        mild.has_over_level = true;
        let mut disaster = record("disaster", Difficulty::Future, 11.0, 9_000_000);
        disaster.over_level = Some(1.6);
        disaster.over_band = Some(OverBand::Disaster);
        disaster.over_mess = Some(1.9);
        disaster.has_over_level = true;
        let mut held = record("held", Difficulty::Future, 11.0, 9_900_000);
        held.over_level = Some(0.4);
        held.over_band = Some(OverBand::Held);
        held.has_over_level = true;
        let input = BoardInput::new(vec![mild, disaster, held])
            .with_curve(true)
            .with_anchor(10.0);
        let board = select_ol50(&input);
        let ids: Vec<&str> = board.rows.iter().map(|row| row.song_id.as_str()).collect();
        assert_eq!(ids, vec!["mild", "disaster", "held"], "行顺序完全照抄后端");
        assert_eq!(board.rows[0].span, 0.3, "越级跨度用后端下发的 over_level");
        assert_eq!(board.rows[0].band, Some(OverBand::Mild));
        assert_eq!(board.rows[0].mess, 0.42);
        assert_eq!(board.rows[0].over_delta, 0.12);
        assert!(board.rows[0].has_over_level);
        assert_eq!(board.rows[1].band, Some(OverBand::Disaster));
        assert_eq!(board.band_counts.disaster, 1);
        assert_eq!(board.band_counts.held, 1);
        assert_eq!(board.band_counts.over_count(), 3);
        assert_eq!(board.over_count, 3);
        assert_eq!(board.rows[0].rank, 1);
        assert_eq!(board.rows[2].rank, 3);
    }

    #[test]
    fn case_321() {
        let mut mild = record("mild", Difficulty::Future, 10.3, 9_790_000);
        mild.over_level = Some(0.3);
        mild.over_band = Some(OverBand::Mild);
        mild.has_over_level = true;
        let mut counts = BandCounts::default();
        counts.add(OverBand::Mild);
        counts.add(OverBand::Held);
        let input = BoardInput::new(vec![mild])
            .with_curve(true)
            .with_counts(counts, 7);
        let board = select_ol50(&input);
        assert_eq!(board.band_counts.mild, 1);
        assert_eq!(board.band_counts.held, 1);
        assert_eq!(board.over_count, 7, "越级谱面数直接用后端给的");
    }

    #[test]
    fn case_322() {
        let mut no_constant = BoardRecord::new("no_const", "T", Difficulty::Future, 0.0);
        no_constant.constant = 0.0;
        no_constant.has_constant = true;
        let input = BoardInput::new(vec![
            record("abandon", Difficulty::Future, 12.0, 6_999_999),
            no_constant.with_score(9_900_000).with_judgement(0, 0, 1000),
            record("kept", Difficulty::Future, 12.0, 7_000_000),
        ])
        .with_anchor(10.0);
        let board = select_ol50(&input);
        assert_eq!(board.matched, 1, "候选门槛：有定数且分数不低于 700 万");
        assert_eq!(board.rows[0].song_id, "kept");
    }

    #[test]
    fn case_323() {
        let mut broken = record("broken", Difficulty::Future, 11.5, 8_000_000);
        broken.backend_pressure = Some(100.0);
        let input = BoardInput::new(vec![
            record("mild", Difficulty::Future, 10.1, 9_790_000),
            broken,
        ]);
        let board = select_ol50(&input);
        assert_eq!(board.rows.len(), 2);
        assert_eq!(board.rows[0].song_id, "mild", "不再按压力重排");
        assert!(board.rows.iter().all(|row| row.band.is_none()));
        assert!(board.rows.iter().all(|row| row.span == 0.0));
        assert_eq!(board.anchor, None);
        assert_eq!(board.rows[1].pressure, 100.0, "压力取后端下发的值");
        assert_eq!(board.band_counts.total(), 0);
    }

    #[test]
    fn case_324() {
        let mut records = Vec::new();
        for index in 0..80 {
            records.push(record(
                &format!("song{index:03}"),
                Difficulty::Future,
                10.0,
                ptt::FULL_SCORE,
            ));
        }
        let input = BoardInput::new(records);
        assert_eq!(select_pm50(&input).rows.len(), BOARD_TOP);
        assert_eq!(
            select_board(&input, BoardKind::Pm50, 7).rows.len(),
            7,
            "limit 参数要被尊重"
        );
        assert_eq!(
            select_board(&input, BoardKind::Pm50, 999).rows.len(),
            BOARD_TOP,
            "超过 50 的 limit 夹到 50"
        );
    }

    #[test]
    fn case_325() {
        let input = BoardInput::default();
        for kind in BoardKind::ALL {
            let board = select_board(&input, kind, BOARD_TOP);
            assert!(board.is_empty(), "{kind} 在空输入下必须是空表");
            assert_eq!(board.matched, 0);
            assert_eq!(board.known, 0);
        }
    }

    #[test]
    fn case_326() {
        let mut plain = record("plain", Difficulty::Future, 10.0, 9_950_000);
        plain.over_level = Some(0.02);
        plain.has_over_level = true;
        let mut stale = record("stale", Difficulty::Future, 10.0, 9_900_000);
        stale.over_level = Some(0.0);
        stale.over_band = Some(OverBand::Normal);
        stale.has_over_level = true;
        let mut ex = record("ex", Difficulty::Future, 10.0, 9_899_999);
        ex.over_level = Some(0.02);
        ex.over_band = Some(OverBand::Normal);
        ex.has_over_level = true;
        let input = BoardInput::new(vec![plain, stale, ex])
            .with_curve(true)
            .with_anchor(10.0);
        let board = select_ol50(&input);
        assert!(
            board
                .rows
                .iter()
                .all(|row| row.band == Some(OverBand::Normal)),
            "跨度都不到 0.05: 线内的 EX+ 一律达标, 谁都不许被本地复算改成攻克"
        );
        assert_eq!(board.band_counts.held, 0);
        assert_eq!(board.band_counts.normal, 3);
        assert_eq!(board.band_counts.over_count(), 0);
    }
}
