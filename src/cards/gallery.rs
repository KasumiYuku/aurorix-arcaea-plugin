use aurorix_plugin::RenderCard;
use aurorix_render::skia_safe::{paint::Style, ClipOp, Paint, PathBuilder, Rect};
use aurorix_render::{CanvasKit, Color, RenderRequest, TextStyle};
use super::player;

use super::sketch;
use super::theme::{self, Theme};
use super::widgets::PlayerBadge;
use crate::cards::CardKind;
use crate::model::{Difficulty, Grade, Region};

pub const ME_WIDTH: i32 = 2800;

pub const ME_HEIGHT: i32 = 1820;

pub const CHAR_WIDTH: i32 = 2016;

pub const BANNER_WIDTH: i32 = 2016;

pub const THEME_WIDTH: i32 = 2016;

pub const MIN_HEIGHT: i32 = 360;

pub const MAX_HEIGHT: i32 = 12000;

const ME_K: f32 = 2.0;

const WALL_K: f32 = 1.4;

const PICK_BASE_W: f32 = 1440.0;

const PICK_MARGIN_X: f32 = 48.0;

const PICK_GAP: f32 = 14.0;

const PICK_GRID_TOP: f32 = 236.0;

const PICK_FOOTER_H: f32 = 120.0;

const THEME_ROW_H: f32 = 152.0;

const THEME_ROW_GAP: f32 = 18.0;

const THEME_PREVIEW_RATIO: f32 = 1.6;

const PROF_PAPER: Color = Color::rgb(0xF4, 0xF1, 0xEA);

const PROF_BG: Color = Color::rgb(0xFA, 0xF8, 0xF3);

const PROF_SUB_CARD_ICE: Color = Color::rgb(0xEF, 0xE9, 0xDC);

const PROF_SELF_ROW: Color = Color::rgb(0xF1, 0xEB, 0xDE);

const PROF_TEXT_PRIMARY: Color = Color::rgb(0x22, 0x21, 0x1F);

const PROF_TEXT_SECOND: Color = Color::rgb(0x7A, 0x75, 0x6C);

const PROF_TEXT_MUTED: Color = Color::rgb(0x9C, 0x96, 0x8A);

const PROF_ACCENT: Color = Color::rgb(0x3D, 0x5A, 0x73);

const PROF_ACCENT_DARK: Color = Color::rgb(0x2C, 0x42, 0x54);

const PROF_WINE: Color = Color::rgb(0x6B, 0x4C, 0x5B);

const PROF_PILL: Color = Color::rgb(0xF1, 0xEC, 0xE0);

const PROF_RULE: Color = Color::rgb(0xD4, 0xCF, 0xC4);

const PROF_HAIRLINE: Color = Color::rgb(0xE2, 0xDD, 0xD3);

const PROF_INK_RULE: Color = Color::rgb(0x2C, 0x2A, 0x27);

const PROF_RULE_SOFT: Color = Color::rgb(0xDC, 0xD6, 0xCC);
const PROF_GROOVE: Color = Color::rgb(0xE6, 0xE1, 0xD6);

const PROF_GAIN: Color = Color::rgb(0x3B, 0x6E, 0x5C);

const PROF_BAR_PM: Color = Color::rgb(0x2C, 0x42, 0x54);

const PROF_BAR_FR: Color = Color::rgb(0x3F, 0x7F, 0xA6);

const PROF_BAR_CLEAR: Color = Color::rgb(0x8A, 0x93, 0x99);

const PROF_BAR_PLAYED: Color = Color::rgb(0xC4, 0xBC, 0xAB);

const PROF_DASH_INK: Color = Color::rgb(0xB5, 0xAD, 0x9D);

const PROF_AREA_INK: Color = Color::new(0x3D, 0x5A, 0x73, 0x12);
const PROF_CAPSULE: Color = Color::new(0x3D, 0x5A, 0x73, 0x30);

const PROF_WHITE: Color = Color::rgb(0xFA, 0xF8, 0xF3);

const PICK_GROUND: Color = Color::rgb(0xF3, 0xEC, 0xE1);

const PICK_PLATE: Color = Color::rgb(0xFF, 0xFD, 0xF9);

const PICK_CHIP: Color = Color::rgb(0xF3, 0xEC, 0xE1);

const PICK_RULE: Color = Color::rgb(0xE6, 0xDC, 0xCF);

const PICK_TEXT_MAIN: Color = Color::rgb(0x2B, 0x21, 0x18);

const PICK_TEXT_DIM: Color = Color::rgb(0x6E, 0x5A, 0x4F);

fn px(v: f32) -> f32 {
    (v * ME_K).round()
}

fn pt(v: f32) -> f32 {
    (v * WALL_K).round()
}

fn pti(v: f32) -> i32 {
    (v * WALL_K).round() as i32
}

fn font_me(size: f32) -> u32 {
    (size * ME_K).round().max(1.0) as u32
}

fn font_me_display(size: f32) -> u32 {
    font_me(size)
}

fn font_wall(size: f32) -> u32 {
    size.round().max(1.0) as u32
}

fn font_wall_sf(size: f32) -> u32 {
    (size * WALL_K).round().max(1.0) as u32
}

fn prof_display(size: f32, color: Color) -> TextStyle {
    TextStyle::new(theme::FONT_NARROW_BOLD, font_me_display(size)).with_color(color)
}

fn prof_text(size: f32, color: Color) -> TextStyle {
    TextStyle::new(theme::FONT_NARROW, font_me(size)).with_color(color)
}

fn prof_number(size: f32, color: Color) -> TextStyle {
    TextStyle::new(theme::FONT_TITLE, font_me(size)).with_color(color)
}

fn prof_mono(size: f32, color: Color) -> TextStyle {
    TextStyle::new(theme::FONT_NUMBER, font_me(size)).with_color(color)
}

fn wall_text(size: f32, color: Color) -> TextStyle {
    TextStyle::new(theme::FONT_NARROW, font_wall_sf(size * WALL_K)).with_color(color)
}

fn wall_number(size: f32, color: Color) -> TextStyle {
    TextStyle::new(theme::FONT_TITLE, font_wall(size)).with_color(color)
}

fn request_for(width: i32, height: i32) -> RenderRequest {
    super::harness::card_request(width, height)
}

#[derive(Clone, Copy)]
struct Area {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Area {
    const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    fn x1(self) -> f32 {
        self.x + self.w
    }

    fn y1(self) -> f32 {
        self.y + self.h
    }

    fn rect_d(self) -> Rect {
        Rect::from_xywh(
            px(self.x),
            px(self.y),
            px(self.x1()) - px(self.x),
            px(self.y1()) - px(self.y),
        )
    }
}

fn band(x: f32, y: f32, x1: f32, y1: f32) -> Rect {
    Rect::from_xywh(px(x), px(y), px(x1) - px(x), px(y1) - px(y))
}

fn hair(kit: &CanvasKit<'_>, area: Area, color: Color) {
    let x0 = px(area.x);
    let y0 = px(area.y);
    let x1 = px(area.x1());
    let y1 = px(area.y1());
    if x1 - x0 < 2.0 || y1 - y0 < 2.0 {
        return;
    }
    let t = px(1.0);
    let rects = [
        Rect::from_xywh(x0, y0, x1 - x0, t),
        Rect::from_xywh(x0, y1 - t, x1 - x0, t),
        Rect::from_xywh(x0, y0, t, y1 - y0),
        Rect::from_xywh(x1 - t, y0, t, y1 - y0),
    ];
    kit.fill_rects_batch(&rects, color);
}

fn stroke_curve(
    kit: &CanvasKit<'_>,
    points: &[(f32, f32)],
    width: f32,
    color: Color,
    clip: Rect,
    smooth: bool,
) {
    if points.len() < 2 {
        return;
    }
    let canvas = kit.canvas();
    canvas.save();
    canvas.clip_rect(clip, ClipOp::Intersect, true);
    let mut builder = PathBuilder::new();
    builder.move_to(points[0]);
    if smooth {
        let last = points.len() - 1;
        for index in 0..last {
            let p0 = if index == 0 {
                points[0]
            } else {
                points[index - 1]
            };
            let p1 = points[index];
            let p2 = points[index + 1];
            let p3 = if index + 2 <= last {
                points[index + 2]
            } else {
                points[last]
            };
            let c1 = (p1.0 + (p2.0 - p0.0) / 6.0, p1.1 + (p2.1 - p0.1) / 6.0);
            let c2 = (p2.0 - (p3.0 - p1.0) / 6.0, p2.1 - (p3.1 - p1.1) / 6.0);
            builder.cubic_to(c1, c2, p2);
        }
    } else {
        for point in &points[1..] {
            builder.line_to(*point);
        }
    }
    let path = builder.detach();
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(color.to_skia());
    paint.set_style(Style::Stroke);
    paint.set_stroke_width(width);
    paint.set_stroke_join(aurorix_render::skia_safe::paint::Join::Round);
    canvas.draw_path(&path, &paint);
    canvas.restore();
}

#[derive(Clone, Copy)]
struct Dash {
    len: f32,
    gap: f32,
}

#[derive(Clone, Copy)]
struct DashStyle {
    dash: Dash,
    thick: f32,
    radius: f32,
    color: Color,
}

fn dash_h(kit: &CanvasKit<'_>, x0: f32, x1: f32, y: f32, style: DashStyle) {
    let mut x = x0;
    while x < x1 {
        let xe = (x + style.dash.len).min(x1);
        let rect = Rect::from_ltrb(px(x), px(y), px(xe), px(y + style.thick));
        if rect.width() > 0.0 {
            sketch::fill_round(kit, rect, style.radius, style.color);
        }
        x += style.dash.len + style.dash.gap;
    }
}

fn dash_v(kit: &CanvasKit<'_>, x: f32, y0: f32, y1: f32, style: DashStyle) {
    let mut y = y0;
    while y < y1 {
        let ye = (y + style.dash.len).min(y1);
        let rect = Rect::from_ltrb(px(x), px(y), px(x + style.thick), px(ye));
        if rect.height() > 0.0 {
            sketch::fill_round(kit, rect, style.radius, style.color);
        }
        y += style.dash.len + style.dash.gap;
    }
}

fn dash_rule_1(kit: &CanvasKit<'_>, x0: f32, x1: f32, y: f32, color: Color) {
    dash_h(
        kit,
        x0,
        x1,
        y,
        DashStyle {
            dash: Dash { len: 3.0, gap: 3.0 },
            thick: 1.0,
            radius: 0.0,
            color,
        },
    );
}

fn thousands(value: i64) -> String {
    let sign = if value < 0 { "-" } else { "" };
    format!("{sign}{}", sketch::thousands(value.abs()))
}

fn ptt3(value: f64) -> String {
    format!("{value:.3}")
}

fn signed_ptt(value: f64) -> String {
    if value > 0.0 {
        format!("+{}", ptt3(value))
    } else {
        ptt3(value)
    }
}

fn signed_ptt4(value: f64) -> String {
    let rounded = (value * 1e4).round() / 1e4;
    let text = format!("{rounded:.4}");
    if rounded > 0.0 {
        format!("+{text}")
    } else {
        text
    }
}

fn diff_ink(name: &str) -> Color {
    let key = name.trim().to_ascii_uppercase();
    match key.as_str() {
        "PST" | "PAST" => Color::rgb(0x4A, 0x6B, 0x85),
        "PRS" | "PRESENT" => Color::rgb(0x4A, 0x6E, 0x5C),
        "FTR" | "FUTURE" => Color::rgb(0x5E, 0x4B, 0x66),
        "ETR" | "ETERNAL" => Color::rgb(0x55, 0x60, 0x6B),
        "BYD" | "BEYOND" => Color::rgb(0x7A, 0x3E, 0x3E),
        "INS" => Color::rgb(0x3E, 0x44, 0x68),
        _ => PROF_TEXT_SECOND,
    }
}

fn cst(seconds: i64) -> chrono::DateTime<chrono::FixedOffset> {
    use chrono::{DateTime, FixedOffset};
    let offset = FixedOffset::east_opt(8 * 3600)
        .unwrap_or_else(|| FixedOffset::east_opt(0).expect("零偏移时区一定可构造"));
    let utc = DateTime::from_timestamp(seconds, 0)
        .unwrap_or_else(|| DateTime::from_timestamp(0, 0).expect("1970 一定可构造"));
    utc.with_timezone(&offset)
}

fn fmt_datetime(seconds: i64) -> String {
    cst(seconds).format("%m-%d %H:%M").to_string()
}

fn fmt_month(seconds: i64) -> String {
    cst(seconds).format("%Y-%m").to_string()
}

fn fmt_day(seconds: i64) -> String {
    cst(seconds).format("%m-%d").to_string()
}

#[derive(Debug, Clone)]
pub struct MeScore {
    pub title: String,
    pub difficulty: Difficulty,
    pub constant: f64,
    pub score: i64,
    pub ptt: f64,
    pub grade: Option<Grade>,
    pub jacket: Option<String>,
}

impl MeScore {
    pub fn new(title: impl Into<String>, difficulty: Difficulty, score: i64) -> Self {
        Self {
            title: title.into(),
            difficulty,
            constant: 0.0,
            score,
            ptt: 0.0,
            grade: None,
            jacket: None,
        }
    }

    pub fn with_constant(mut self, constant: f64) -> Self {
        self.constant = constant;
        self
    }

    pub fn with_ptt(mut self, ptt: f64) -> Self {
        self.ptt = ptt;
        self
    }

    pub fn with_grade(mut self, grade: Grade) -> Self {
        self.grade = Some(grade);
        self
    }

    pub fn with_jacket(mut self, key: impl Into<String>) -> Self {
        let key = key.into();
        if !key.is_empty() {
            self.jacket = Some(key);
        }
        self
    }

    pub fn grade_or_computed(&self) -> Grade {
        self.grade
            .unwrap_or_else(|| Grade::from_score(self.score))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MeCurvePoint {
    pub at: i64,
    pub ptt: f64,
}

#[derive(Debug, Clone, Default)]
pub struct MeCurve {
    pub points: Vec<MeCurvePoint>,
    pub peak: f64,
    pub peak_at: Option<i64>,
    pub recent7: Option<f64>,
    pub recent30: Option<f64>,
    pub records30: Option<i32>,
    pub first: Option<i64>,
    pub last: Option<i64>,
}

impl MeCurve {
    pub fn empty(&self) -> bool {
        self.points.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct MeBand {
    pub kind: Difficulty,
    pub name: String,
    pub played: i32,
    pub total: i32,
    pub cleared: i32,
    pub full_combo: i32,
    pub pm: i32,
    pub ex_plus: i32,
    pub ex: i32,
    pub loss_played: f64,
    pub loss_total: f64,
}

impl MeBand {
    pub fn new(kind: Difficulty) -> Self {
        Self {
            kind,
            name: kind.as_str().to_string(),
            played: 0,
            total: 0,
            cleared: 0,
            full_combo: 0,
            pm: 0,
            ex_plus: 0,
            ex: 0,
            loss_played: 0.0,
            loss_total: 0.0,
        }
    }

    pub fn counts_loss(&self) -> bool {
        !matches!(self.kind, Difficulty::Past | Difficulty::Present)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MeBin {
    pub constant: f64,
    pub low: f64,
    pub high: f64,
    pub avg: f64,
    pub count: i32,
}

#[derive(Debug, Clone, Default)]
pub struct MeScatter {
    pub bins: Vec<MeBin>,
    pub max_ptt: f64,
    pub avg_ptt: Option<f64>,
    pub ability: Option<f64>,
}

impl MeScatter {
    pub fn empty(&self) -> bool {
        self.bins.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
pub struct MeBoardRow {
    pub rank: i32,
    pub name: String,
    pub avatar: Option<String>,
    pub ptt: Option<f64>,
    pub rating_frame: Option<i32>,
    pub rating_accent: Option<i32>,
    pub me: bool,
}

#[derive(Debug, Clone, Default)]
pub struct MeBoard {
    pub total: i32,
    pub rows: Vec<MeBoardRow>,
}

impl MeBoard {
    pub fn empty(&self) -> bool {
        self.total == 0 || self.rows.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
pub struct MeChange {
    pub title: String,
    pub diff: String,
    pub old_score: Option<i64>,
    pub new_score: i64,
    pub at: Option<i64>,
    pub jacket: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MeInput {
    pub title: String,
    pub subtitle: String,
    pub theme: Theme,
    pub badge: PlayerBadge,
    pub user_id: String,
    pub region: Region,
    pub joined: String,
    pub updated: String,
    pub best_average: f64,
    pub best: Vec<MeScore>,
    pub recent: Vec<MeScore>,
    pub note: String,
    pub curve: Option<MeCurve>,
    pub bands: Option<Vec<MeBand>>,
    pub pm_small_p: i64,
    pub scatter: Option<MeScatter>,
    pub board: Option<MeBoard>,
    pub changes: Option<Vec<MeChange>>,
    pub rating_frame: Option<i32>,
    pub rating_accent: Option<i32>,
}

impl MeInput {
    pub fn new(badge: PlayerBadge) -> Self {
        Self {
            title: CardKind::Me.label().to_string(),
            subtitle: String::new(),
            theme: Theme::DEFAULT,
            badge,
            user_id: String::new(),
            region: Region::Intl,
            joined: String::new(),
            updated: String::new(),
            best_average: 0.0,
            best: Vec::new(),
            recent: Vec::new(),
            note: String::new(),
            curve: None,
            bands: None,
            pm_small_p: 0,
            scatter: None,
            board: None,
            changes: None,
            rating_frame: None,
            rating_accent: None,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = subtitle.into();
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_user(mut self, user_id: impl Into<String>, region: Region) -> Self {
        self.user_id = user_id.into();
        self.region = region;
        self
    }

    pub fn with_joined(mut self, joined: impl Into<String>) -> Self {
        self.joined = joined.into();
        self
    }

    pub fn with_updated(mut self, updated: impl Into<String>) -> Self {
        self.updated = updated.into();
        self
    }

    pub fn with_best_average(mut self, value: f64) -> Self {
        self.best_average = value;
        self
    }

    pub fn with_best(mut self, best: Vec<MeScore>) -> Self {
        self.best = best;
        self
    }

    pub fn with_recent(mut self, recent: Vec<MeScore>) -> Self {
        self.recent = recent;
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = note.into();
        self
    }

    pub fn with_curve(mut self, curve: MeCurve) -> Self {
        self.curve = Some(curve);
        self
    }

    pub fn with_bands(mut self, bands: Vec<MeBand>) -> Self {
        self.bands = Some(bands);
        self
    }

    pub fn with_pm_small_p(mut self, small_p: i64) -> Self {
        self.pm_small_p = small_p.max(0);
        self
    }

    pub fn with_scatter(mut self, scatter: MeScatter) -> Self {
        self.scatter = Some(scatter);
        self
    }

    pub fn with_board(mut self, board: MeBoard) -> Self {
        self.board = Some(board);
        self
    }

    pub fn with_changes(mut self, changes: Vec<MeChange>) -> Self {
        self.changes = Some(changes);
        self
    }

    pub fn with_rating(mut self, frame: i32, accent: i32) -> Self {
        self.rating_frame = Some(frame);
        self.rating_accent = Some(accent);
        self
    }

    pub fn ptt(&self) -> f64 {
        self.badge.ptt.unwrap_or(0.0)
    }

    pub fn star(&self) -> u8 {
        self.badge.star.unwrap_or(0)
    }

    pub fn best_peak(&self) -> f64 {
        self.best
            .iter()
            .map(|score| score.ptt)
            .fold(0.0_f64, f64::max)
    }

    fn curve_ref(&self) -> Option<&MeCurve> {
        self.curve.as_ref().filter(|curve| !curve.empty())
    }

    fn bands_ref(&self) -> Vec<&MeBand> {
        self.bands
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .filter(|band| band.counts_loss())
            .collect()
    }

    fn changes_ref(&self) -> &[MeChange] {
        self.changes.as_deref().unwrap_or(&[])
    }

    fn board_ref(&self) -> Option<&MeBoard> {
        self.board.as_ref().filter(|board| !board.empty())
    }

    fn scatter_ref(&self) -> Option<&MeScatter> {
        self.scatter.as_ref().filter(|scatter| !scatter.empty())
    }
}

impl Default for MeInput {
    fn default() -> Self {
        Self::new(PlayerBadge::new("Afkeru").with_ptt(12.34).with_star(6)).with_subtitle("玩家档案")
    }
}

pub fn size_of(input: &MeInput) -> (i32, i32) {
    (ME_WIDTH, ME_HEIGHT + (bands_growth(input) * ME_K).round() as i32)
}

const BANDS_ROW_H: f32 = 40.0;
const BANDS_BASE_ROWS: usize = 4;
const BANDS_MAX_ROWS: usize = BANDS_BASE_ROWS + 1;

fn bands_rows(input: &MeInput) -> usize {
    input.bands_ref().len().min(BANDS_MAX_ROWS)
}

fn bands_growth(input: &MeInput) -> f32 {
    let rows = bands_rows(input);
    if rows <= BANDS_BASE_ROWS {
        return 0.0;
    }
    (rows - BANDS_BASE_ROWS) as f32 * BANDS_ROW_H
}

pub fn request(input: &MeInput) -> RenderRequest {
    let (width, height) = size_of(input);
    request_for(width, height)
}

pub fn card(input: MeInput) -> RenderCard {
    let request = request(&input);
    let label = format!(
        "{} {}x{}",
        CardKind::Me.label(),
        request.width,
        request.height
    );
    let cache_key = super::memo::fingerprint("me", &input);
    RenderCard::new(request, label, move |kit| paint(kit, &input)).with_cache_key(cache_key)
}

pub struct CurveScale {
    lo: f64,
    hi: f64,
    knee: bool,
    knee_ptt: f64,
    knee_frac: f64,
}

impl CurveScale {
    fn build(values: &[f64]) -> Self {
        let mut scale = Self {
            lo: 0.0,
            hi: 0.0,
            knee: false,
            knee_ptt: 0.0,
            knee_frac: 0.72,
        };
        if values.is_empty() {
            return scale;
        }
        let mut lo = values[0];
        let mut hi = values[0];
        for value in &values[1..] {
            lo = lo.min(*value);
            hi = hi.max(*value);
        }
        if hi - lo > 0.0 {
            let effective = (hi - lo).max(0.015);
            scale.lo = lo - 0.12 * effective;
            scale.hi = hi + 0.16 * effective;
            if let Some(knee) = curve_knee(values) {
                scale.knee = true;
                scale.knee_ptt = knee;
                scale.hi = hi + 0.16 * (hi - knee).max(0.015);
                scale.lo = lo - 0.12 * (knee - lo).max(0.015);
            }
        } else {
            scale.lo = lo - 0.05;
            scale.hi = lo + 0.05;
        }
        scale
    }

    fn y(&self, value: f64, top: f32, bottom: f32) -> f32 {
        if self.hi <= self.lo {
            return bottom;
        }
        let v = value.clamp(self.lo, self.hi);
        if !self.knee {
            return bottom - (bottom - top) * ((v - self.lo) / (self.hi - self.lo)) as f32;
        }
        let yk = top + self.knee_frac as f32 * (bottom - top);
        if v >= self.knee_ptt {
            if self.hi <= self.knee_ptt {
                return yk;
            }
            return yk - (yk - top) * ((v - self.knee_ptt) / (self.hi - self.knee_ptt)) as f32;
        }
        if self.knee_ptt <= self.lo {
            return yk;
        }
        bottom - (bottom - yk) * ((v - self.lo) / (self.knee_ptt - self.lo)) as f32
    }

    fn value_at(&self, y: f32, top: f32, bottom: f32) -> f64 {
        if self.hi <= self.lo || bottom <= top {
            return self.lo;
        }
        if !self.knee {
            return self.lo + f64::from((bottom - y) / (bottom - top)) * (self.hi - self.lo);
        }
        let yk = top + self.knee_frac as f32 * (bottom - top);
        if y <= yk {
            if yk <= top || self.hi <= self.knee_ptt {
                return self.hi;
            }
            return self.knee_ptt + f64::from((yk - y) / (yk - top)) * (self.hi - self.knee_ptt);
        }
        if bottom <= yk || self.knee_ptt <= self.lo {
            return self.lo;
        }
        self.lo + f64::from((bottom - y) / (bottom - yk)) * (self.knee_ptt - self.lo)
    }
}

fn curve_knee(values: &[f64]) -> Option<f64> {
    let n = values.len();
    if n < 5 {
        return None;
    }
    let mut lo = values[0];
    let mut hi = values[0];
    for value in values {
        lo = lo.min(*value);
        hi = hi.max(*value);
    }
    let from = n / 5;
    let mut recent_min = values[from];
    for value in &values[from..] {
        recent_min = recent_min.min(*value);
    }
    if recent_min <= lo + 1e-9 {
        return None;
    }
    if hi - lo > 2.5 * (hi - recent_min).max(0.02) {
        return Some(recent_min);
    }
    None
}

fn curve_clean(value: f64) -> f64 {
    if value.abs() < 1e-9 {
        return 0.0;
    }
    (value * 1e6).round() / 1e6
}

fn curve_tick_text(value: f64, _step: f64) -> String {
    format!("{:.3}", curve_clean(value))
}

fn curve_axis_label(first: i64, last: i64, index: i32, count: i32) -> Option<String> {
    if count < 2 || index < 0 || index >= count {
        return None;
    }
    let span = last - first;
    if span <= 0 {
        return Some(fmt_month(first));
    }
    let at = first + (span as f64 * f64::from(index) / f64::from(count - 1)) as i64;
    if index == 0 || index == count - 1 {
        return Some(fmt_month(at));
    }
    Some(fmt_day(at))
}

fn curve_value_at(points: &[MeCurvePoint], first: i64, span: f64, plot: Area, x: f32) -> f64 {
    if points.is_empty() {
        return 0.0;
    }
    let x_of = |at: i64| -> f32 {
        if span <= 0.0 {
            return plot.x + plot.w / 2.0;
        }
        plot.x + plot.w * ((at - first) as f64 / span) as f32
    };
    if x <= x_of(points[0].at) {
        return points[0].ptt;
    }
    for index in 1..points.len() {
        let x1 = x_of(points[index].at);
        if x > x1 {
            continue;
        }
        let x0 = x_of(points[index - 1].at);
        if x1 <= x0 {
            return points[index].ptt;
        }
        let t = f64::from((x - x0) / (x1 - x0));
        return points[index - 1].ptt + (points[index].ptt - points[index - 1].ptt) * t;
    }
    points[points.len() - 1].ptt
}

struct ScatterPoint {
    constant: f64,
    low: f64,
    best: f64,
    avg: f64,
    count: i32,
}

impl ScatterPoint {
    fn has_data(&self) -> bool {
        self.count > 0 && self.best > 0.0
    }
}

fn scatter_points(scatter: &MeScatter) -> Vec<ScatterPoint> {
    let mut out: Vec<ScatterPoint> = scatter
        .bins
        .iter()
        .map(|bin| ScatterPoint {
            constant: bin.constant,
            low: bin.low,
            best: bin.high,
            avg: bin.avg,
            count: bin.count,
        })
        .collect();
    out.sort_by(|a, b| a.constant.total_cmp(&b.constant));
    out
}

fn scatter_filled(points: &[ScatterPoint]) -> Vec<ScatterPoint> {
    points
        .iter()
        .filter(|point| point.has_data())
        .map(|point| ScatterPoint {
            constant: point.constant,
            low: point.low.min(point.best),
            best: point.best,
            avg: point.avg,
            count: point.count,
        })
        .collect()
}

fn scatter_clean(value: f64) -> f64 {
    if value.abs() < 1e-6 {
        return 0.0;
    }
    (value * 1000.0).round() / 1000.0
}

fn scatter_domain(points: &[ScatterPoint], ability: Option<f64>) -> (f64, f64) {
    if points.is_empty() {
        return (0.0, 1.0);
    }
    let mut end = points[points.len() - 1].constant;
    let mut start = points[0].constant;
    if let Some(ability) = ability {
        for point in points {
            if point.constant >= ability - 3.2 - 1e-9 {
                start = point.constant;
                break;
            }
        }
    }
    start = (start * 2.0).floor() / 2.0;
    if end - start > 4.5 {
        start = ((end - 4.5) * 2.0).floor() / 2.0;
    }
    if start > end {
        start = end;
    }
    end = end.max(start);
    (start, end)
}

fn scatter_in_range(points: &[ScatterPoint], start: f64) -> Vec<ScatterPoint> {
    points
        .iter()
        .filter(|point| point.constant >= start - 1e-9)
        .map(|point| ScatterPoint {
            constant: point.constant,
            low: point.low,
            best: point.best,
            avg: point.avg,
            count: point.count,
        })
        .collect()
}

const SCATTER_SPAN_MIN: f64 = 4.0;
const SCATTER_CAP_W: f32 = 6.0;
const SCATTER_CAP_MIN_H: f32 = 8.0;
const SCATTER_FLOOR_PCT: f64 = 0.1;
const SCATTER_EPS: f64 = 0.02;

fn scatter_snap_lo(value: f64) -> f64 {
    ((value - SCATTER_EPS) * 2.0).floor() / 2.0
}

fn scatter_snap_hi(value: f64) -> f64 {
    ((value + SCATTER_EPS) * 2.0).ceil() / 2.0
}

fn scatter_window(points: &[ScatterPoint]) -> (f64, f64) {
    if points.is_empty() {
        return (0.0, 1.0);
    }
    let mut lows: Vec<f64> = points.iter().map(|point| point.low).collect();
    lows.sort_by(|a, b| a.total_cmp(b));
    let mut floor = (lows.len() as f64 * SCATTER_FLOOR_PCT).floor() as usize;
    if lows.len() >= 3 {
        floor = floor.max(1).min(lows.len() - 1);
    }
    let base = lows.get(floor).copied().unwrap_or(lows[0]);
    let top = points
        .iter()
        .map(|point| point.best)
        .fold(f64::MIN, f64::max);
    let mut lo = scatter_snap_lo(base);
    let mut hi = scatter_snap_hi(top);
    if hi - lo < SCATTER_SPAN_MIN {
        let mid = (lo + hi) / 2.0;
        lo = scatter_snap_lo(mid - SCATTER_SPAN_MIN / 2.0);
        hi = scatter_snap_hi(mid + SCATTER_SPAN_MIN / 2.0);
    }
    if hi <= lo {
        hi = lo + SCATTER_SPAN_MIN;
    }
    (lo, hi)
}

fn scatter_step(lo: f64, hi: f64) -> f64 {
    let ladder = [0.02, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0];
    for candidate in ladder {
        let n =
            ((hi - (lo / candidate - 1e-9).ceil() * candidate) / candidate + 1e-9).floor() + 1.0;
        if (4.0..=7.0).contains(&n) {
            return candidate;
        }
    }
    ladder[ladder.len() - 1]
}

fn scatter_ticks(lo: f64, hi: f64) -> Vec<f64> {
    if hi <= lo {
        return Vec::new();
    }
    let step = scatter_step(lo, hi);
    let start = (lo / step - 1e-9).ceil() * step;
    let mut out = Vec::new();
    let mut index = 0.0;
    loop {
        let value = scatter_clean(start + index * step);
        if value > hi + 1e-9 {
            break;
        }
        out.push(value);
        index += 1.0;
    }
    out
}

fn scatter_ptt_ticks(lo: f64, hi: f64) -> Vec<f64> {
    if hi <= lo {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut value = lo.ceil();
    while value <= hi + 1e-9 {
        out.push(scatter_clean(value));
        value += 1.0;
    }
    if out.len() < 2 {
        return scatter_ticks(lo, hi);
    }
    out
}

fn scatter_domain_ticks(start: f64, end: f64) -> Vec<f64> {
    if end < start {
        return Vec::new();
    }
    let step = 0.5;
    let first = (start / step - 1e-9).ceil() * step;
    let mut out = Vec::new();
    let mut value = first;
    while value <= end + 1e-9 {
        out.push(scatter_clean(value));
        value += step;
    }
    out
}

fn band_totals(bands: &[&MeBand]) -> (i32, i32) {
    let mut played = 0;
    let mut total = 0;
    for band in bands {
        played += band.played.max(0);
        total += band.total.max(0);
    }
    (played, total)
}

fn head_parts(input: &MeInput) -> (String, String) {
    let (played, total) = band_totals(&input.bands_ref());
    let head = if total > 0 {
        format!("记录 {played} / {total}")
    } else if played > 0 {
        format!("记录 {played}")
    } else {
        String::new()
    };
    let tail = match curve_last(input) {
        Some(last) => format!("  ·  更新于 {}", fmt_datetime(last)),
        None => String::new(),
    };
    (head, tail)
}

fn curve_first(input: &MeInput) -> Option<i64> {
    let curve = input.curve.as_ref()?;
    curve
        .first
        .or_else(|| curve.points.first().map(|point| point.at))
}

fn curve_last(input: &MeInput) -> Option<i64> {
    let curve = input.curve.as_ref()?;
    curve
        .last
        .or_else(|| curve.points.last().map(|point| point.at))
}

fn curve_pill_segments(input: &MeInput) -> Vec<String> {
    let curve = match input.curve_ref() {
        Some(curve) => curve,
        None => return Vec::new(),
    };
    let mut parts: Vec<String> = Vec::new();
    if let Some(value) = curve.recent7 {
        parts.push(format!("7天 {}", signed_ptt(value)));
    }
    if let Some(value) = curve.recent30 {
        parts.push(format!("30天 {}", signed_ptt(value)));
    }
    let peek = if curve.peak > 0.0 {
        format!("峰值 {}", ptt3(curve.peak))
    } else {
        String::new()
    };
    let head = parts.join("  ·  ");
    match (head.is_empty(), peek.is_empty()) {
        (true, true) => Vec::new(),
        (true, false) => vec![peek],
        (false, true) => vec![head],
        (false, false) => vec![head, String::from("  ·  "), peek],
    }
}

fn bands_note(input: &MeInput) -> String {
    let bands = input.bands_ref();
    if bands.is_empty() {
        return String::new();
    }
    let (played_ls, total_ls) = band_main_loss(&bands);
    format!(
        "PM中小P总量 {}  ·  主难已玩总LS {:.3}  ·  主难全曲总LS {:.3}",
        sketch::thousands(input.pm_small_p.max(0)),
        played_ls,
        total_ls
    )
}

fn band_main_loss(bands: &[&MeBand]) -> (f64, f64) {
    let mut played = 0.0;
    let mut total = 0.0;
    for band in bands {
        if !band.counts_loss() {
            continue;
        }
        played += band.loss_played.max(0.0);
        total += band.loss_total.max(0.0);
    }
    (played, total)
}

fn band_loss_text(band: &MeBand) -> String {
    if !band.counts_loss() {
        return String::from("NONE / NONE");
    }
    format!(
        "{:.3} / {:.3}",
        band.loss_played.max(0.0),
        band.loss_total.max(0.0)
    )
}

fn changes_note(input: &MeInput) -> String {
    let changes = input.changes_ref();
    if changes.is_empty() {
        return String::new();
    }
    let mut parts: Vec<String> = Vec::new();
    if let Some(curve) = input.curve.as_ref() {
        if let Some(records) = curve.records30 {
            parts.push(format!("近 30 天新增 {records} 张"));
        }
    }
    parts.push(format!("显示最新 {} 条", changes.len().min(6)));
    parts.join("  ·  ")
}

fn gap_to_prev(board: &MeBoard) -> Option<f64> {
    let rows = &board.rows;
    let me_index = rows.iter().position(|row| row.me)?;
    let mine = &rows[me_index];
    let mine_ptt = mine.ptt?;
    if mine.rank <= 1 {
        return None;
    }
    let mut prev: Option<usize> = None;
    for (index, row) in rows.iter().enumerate() {
        if index == me_index || row.me || row.rank <= 0 || row.rank >= mine.rank {
            continue;
        }
        match prev {
            Some(candidate) if row.rank <= rows[candidate].rank => {}
            _ => prev = Some(index),
        }
    }
    let prev = prev?;
    let prev_ptt = rows[prev].ptt?;
    Some(mine_ptt - prev_ptt)
}

fn board_note(board: &MeBoard) -> String {
    if board.empty() {
        return String::new();
    }
    let mut parts = vec![format!("共 {} 人上榜", board.total)];
    if let Some(delta) = gap_to_prev(board) {
        let sign = if delta < 0.0 { "-" } else { "+" };
        parts.push(format!("距上一位 {sign}{:.3}", delta.abs()));
    }
    parts.join("  ·  ")
}

fn board_list_layout(board: &MeBoard) -> bool {
    for row in &board.rows {
        if row.me && row.rank > 0 {
            return row.rank <= 15;
        }
    }
    true
}

fn pad_board_cells(out: &[MeBoardRow], rows: &[MeBoardRow], target: usize) -> Vec<MeBoardRow> {
    let mut out: Vec<MeBoardRow> = out.to_vec();
    for row in rows {
        if out.len() >= target {
            break;
        }
        if row.rank <= 0 {
            continue;
        }
        if let Some(last) = out.last() {
            if row.rank <= last.rank {
                continue;
            }
        }
        out.push(row.clone());
    }
    out
}

fn board_cells(board: &MeBoard) -> Vec<MeBoardRow> {
    const TOP: usize = 12;
    const LIST: usize = 15;
    const CELLS: usize = 14;
    if board.rows.is_empty() {
        return Vec::new();
    }
    let mut rows: Vec<MeBoardRow> = Vec::new();
    for row in &board.rows {
        if let Some(last) = rows.last() {
            if row.rank > 0 && last.rank == row.rank && last.name == row.name {
                if row.me && !last.me {
                    let index = rows.len() - 1;
                    rows[index] = row.clone();
                }
                continue;
            }
        }
        rows.push(row.clone());
    }
    let mut head: Vec<MeBoardRow> = Vec::new();
    let mut me_at: Option<usize> = None;
    for (index, row) in rows.iter().enumerate() {
        if row.me && me_at.is_none() {
            me_at = Some(index);
        }
        if head.len() < LIST && row.rank > 0 && row.rank <= LIST as i32 {
            head.push(row.clone());
        }
    }
    if board_list_layout(board) {
        return pad_board_cells(&head, &rows, LIST);
    }
    if head.len() > TOP {
        head.truncate(TOP);
    }
    let me_index = match me_at {
        Some(index) => index,
        None => return pad_board_cells(&head, &rows, CELLS),
    };
    let mine = rows[me_index].clone();
    let mut above: Option<usize> = None;
    let mut below: Option<usize> = None;
    for (index, row) in rows.iter().enumerate() {
        if row.me || row.rank <= 0 || row.rank <= TOP as i32 {
            continue;
        }
        if row.rank < mine.rank {
            match above {
                Some(candidate) if row.rank <= rows[candidate].rank => {}
                _ => above = Some(index),
            }
            continue;
        }
        match below {
            Some(candidate) if row.rank >= rows[candidate].rank => {}
            _ => below = Some(index),
        }
    }
    let mut out = head;
    match (above, below) {
        (Some(index), _) => {
            out.push(rows[index].clone());
            out.push(mine);
        }
        (None, Some(index)) => {
            out.push(mine);
            out.push(rows[index].clone());
        }
        (None, None) => out.push(mine),
    }
    pad_board_cells(&out, &rows, CELLS)
}

const BANDS_COL_X: [f32; 6] = [232.0, 280.0, 328.0, 376.0, 424.0, 472.0];

const BANDS_COL_LABEL: [&str; 6] = ["记录", "通关", "PM", "FR", "EX+", "EX"];

const BANDS_LOSS_X: f32 = 512.0;

fn bands_row_at(index: usize) -> Area {
    Area::new(56.0, 414.0 + index as f32 * BANDS_ROW_H, 598.0, 32.0)
}

fn profile_backdrop(kit: &CanvasKit<'_>, input: &MeInput) {
    kit.fill_rect(
        Rect::from_xywh(
            0.0,
            0.0,
            ME_WIDTH as f32,
            ME_HEIGHT as f32 + bands_growth(input),
        ),
        PROF_PAPER,
    );
}

fn profile_top_bar(kit: &CanvasKit<'_>) {
    let x0 = px(40.0);
    let x1 = px(1360.0);
    let y = px(105.0);
    let thick = px(2.0);
    kit.fill_rect(Rect::from_xywh(x0, y, x1 - x0, thick), PROF_INK_RULE);
    let hair_y = y + thick + px(3.0);
    kit.fill_rect(Rect::from_xywh(x0, hair_y, x1 - x0, px(1.0)), PROF_RULE);
}

fn profile_header(kit: &CanvasKit<'_>, input: &MeInput) {
    sketch::text(
        kit,
        &prof_display(32.0, PROF_TEXT_PRIMARY),
        "PLAYER PROFILE",
        px(60.0),
        px(66.0),
        PROF_TEXT_PRIMARY,
    );
    let (head, tail) = head_parts(input);
    let mut x = px(60.0);
    let baseline = px(94.0);
    if !head.is_empty() {
        let style = prof_display(13.0, PROF_ACCENT);
        let width = sketch::text(kit, &style, &head, x, baseline, PROF_ACCENT);
        x += width;
    }
    if tail.is_empty() {
        return;
    }
    if head.is_empty() {
        x = px(60.0);
    }
    sketch::text(
        kit,
        &prof_text(13.0, PROF_TEXT_SECOND),
        &tail,
        x,
        baseline,
        PROF_TEXT_SECOND,
    );
}

fn profile_card(kit: &CanvasKit<'_>, card: Area) {
    kit.rrect_fill(card.rect_d(), 0.0, PROF_BG);
    kit.fill_rect(band(card.x, card.y, card.x1(), card.y + 2.0), PROF_INK_RULE);
}

fn profile_card_head(kit: &CanvasKit<'_>, card: Area, number: i32, title: &str) {
    let num_text = format!("{number:02}");
    let num_style = prof_number(11.0, PROF_TEXT_MUTED);
    let num_x = card.x + 20.0;
    let num_base = card.y + 32.0;
    let num_width = sketch::text(
        kit,
        &num_style,
        &num_text,
        px(num_x),
        px(num_base),
        PROF_TEXT_MUTED,
    );
    let slash_style = prof_text(11.0, PROF_TEXT_MUTED);
    let slash_x = num_x + num_width / ME_K + 6.0;
    let slash_width = sketch::text(
        kit,
        &slash_style,
        "/",
        px(slash_x),
        px(num_base),
        PROF_TEXT_MUTED,
    );
    let title_x = slash_x + slash_width / ME_K + 6.0;
    sketch::text(
        kit,
        &prof_display(16.0, PROF_TEXT_PRIMARY),
        title,
        px(title_x),
        px(card.y + 32.0),
        PROF_TEXT_PRIMARY,
    );
    let rule_y = px(card.y + 42.0);
    kit.fill_rect(
        Rect::from_xywh(px(card.x), rule_y, px(card.x1()) - px(card.x), px(1.0)),
        PROF_RULE_SOFT,
    );
}

fn profile_card_note(kit: &CanvasKit<'_>, card: Area, baseline: f32, note: &str, color: Color) {
    if note.is_empty() {
        return;
    }
    sketch::text_right(
        kit,
        &prof_display(11.0, color),
        note,
        px(card.x1() - 20.0),
        px(baseline),
        color,
    );
}

fn profile_empty_note(kit: &CanvasKit<'_>, text: &str, x: f32, baseline: f32) {
    if text.is_empty() {
        return;
    }
    sketch::text(
        kit,
        &prof_text(11.0, PROF_TEXT_MUTED),
        text,
        px(x),
        px(baseline),
        PROF_TEXT_MUTED,
    );
}

fn profile_curve_pill(kit: &CanvasKit<'_>, input: &MeInput) {
    let segments = curve_pill_segments(input);
    if segments.is_empty() {
        return;
    }
    let style = prof_display(11.0, PROF_ACCENT_DARK);
    let mut total = 0.0;
    for segment in &segments {
        total += sketch::text_width(kit, &style, segment) / ME_K;
    }
    let x1 = 650.0;
    let x0 = x1 - (total + 2.0 * 14.0);
    kit.rrect_fill(band(x0, 122.0, x1, 146.0), 0.0, PROF_PILL);
    let mut x = x0 + 14.0;
    let baseline = px(122.0 + 12.0 + 11.0 * 0.36);
    for segment in &segments {
        let width = sketch::text(kit, &style, segment, px(x), baseline, PROF_ACCENT_DARK);
        x += width / ME_K;
    }
}

fn profile_curve_card(kit: &CanvasKit<'_>, input: &MeInput) {
    let card = Area::new(40.0, 114.0, 630.0, 216.0);
    profile_card(kit, card);
    profile_card_head(kit, card, 1, "成长曲线");
    profile_curve_pill(kit, input);

    let plot = Area::new(60.0, 170.0, 550.0, 125.0);
    let curve = match input.curve_ref() {
        Some(curve) => curve,
        None => {
            profile_empty_note(
                kit,
                "还没有历史 —— 日结每天记一次, 几天后这里就有曲线了",
                plot.x,
                plot.y + plot.h / 2.0,
            );
            return;
        }
    };
    let first = curve_first(input).unwrap_or(curve.points[0].at);
    let last = curve_last(input).unwrap_or(curve.points[curve.points.len() - 1].at);
    let span = (last - first) as f64;
    let values: Vec<f64> = curve.points.iter().map(|point| point.ptt).collect();
    let scale = CurveScale::build(&values);
    let y_of = |value: f64| -> f32 { scale.y(value, plot.y, plot.y1()) };

    let mut points: Vec<(f32, f32)> = Vec::new();
    let mut column = plot.x as i32;
    while column <= plot.x1() as i32 {
        let value = curve_value_at(&curve.points, first, span, plot, column as f32);
        points.push((px(column as f32), px(y_of(value))));
        column += 1;
    }

    let mut dots: Vec<(f32, f32)> = curve
        .points
        .iter()
        .map(|point| {
            let x = if span <= 0.0 {
                plot.x + plot.w / 2.0
            } else {
                plot.x + plot.w * ((point.at - first) as f64 / span) as f32
            };
            (x, y_of(point.ptt))
        })
        .collect();
    dots.sort_by(|left, right| left.0.total_cmp(&right.0));
    let line_points: Vec<(f32, f32)> = dots.iter().map(|(x, y)| (px(*x), px(*y))).collect();

    let bottom = px(plot.y1());
    for (x, y) in &points {
        let top = y.round();
        if bottom <= top {
            continue;
        }
        kit.fill_rect(
            Rect::from_xywh(x.round(), top, px(1.0), bottom - top),
            PROF_AREA_INK,
        );
    }

    let segments = 4;
    let grid_step = if scale.hi > scale.lo {
        (scale.hi - scale.lo) / f64::from(segments)
    } else {
        0.0
    };
    let tick_style = prof_number(10.0, PROF_TEXT_MUTED);
    for index in 0..=segments {
        let y = plot.y + (plot.y1() - plot.y) * index as f32 / segments as f32;
        let value = scale.value_at(y, plot.y, plot.y1());
        kit.fill_rect(
            Rect::from_xywh(
                px(plot.x) + px(12.0),
                px(y),
                px(plot.x1()) - px(12.0) - (px(plot.x) + px(12.0)),
                px(1.0),
            ),
            PROF_HAIRLINE,
        );
        sketch::text_right(
            kit,
            &tick_style,
            &curve_tick_text(value, grid_step),
            px(650.0),
            px(y + 10.0 * 0.36),
            PROF_TEXT_MUTED,
        );
    }

    let date_style = prof_number(10.0, PROF_TEXT_MUTED);
    for index in 0..5 {
        let label = match curve_axis_label(first, last, index, 5) {
            Some(label) => label,
            None => continue,
        };
        let x = plot.x + plot.w * index as f32 / 4.0;
        let baseline = px(312.0);
        match index {
            0 => {
                sketch::text(
                    kit,
                    &date_style,
                    &label,
                    px(plot.x),
                    baseline,
                    PROF_TEXT_MUTED,
                );
            }
            4 => {
                sketch::text_right(
                    kit,
                    &date_style,
                    &label,
                    px(plot.x1()),
                    baseline,
                    PROF_TEXT_MUTED,
                );
            }
            _ => {
                sketch::text_center(kit, &date_style, &label, px(x), baseline, PROF_TEXT_MUTED);
            }
        }
    }

    let clip = Rect::from_xywh(
        px(plot.x) - px(2.0),
        px(plot.y) - px(2.0),
        px(plot.x1()) + px(2.0) - (px(plot.x) - px(2.0)),
        px(plot.y1()) + px(2.0) - (px(plot.y) - px(2.0)),
    );
    stroke_curve(kit, line_points.as_slice(), 2.0 * ME_K, PROF_ACCENT, clip, false);
    let mut peak_position: Option<(f32, f32)> = None;
    if curve.peak > 0.0 {
        if let Some(peak_at) = curve.peak_at {
            let x = if span <= 0.0 {
                plot.x + plot.w / 2.0
            } else {
                plot.x + plot.w * ((peak_at - first) as f64 / span) as f32
            };
            peak_position = Some((x, y_of(curve.peak)));
        }
    }
    let last_point = curve.points[curve.points.len() - 1];
    let lx = if span <= 0.0 {
        plot.x + plot.w / 2.0
    } else {
        plot.x + plot.w * ((last_point.at - first) as f64 / span) as f32
    };
    let ly = y_of(last_point.ptt);
    let overlaps = match peak_position {
        Some((peak_x, peak_y)) => (lx - peak_x).abs() < 0.6 && (ly - peak_y).abs() < 0.6,
        None => false,
    };
    if !overlaps {
        sketch::fill_round(
            kit,
            band(lx - 3.0, ly - 3.0, lx + 3.0, ly + 3.0),
            3.0 * ME_K,
            PROF_ACCENT,
        );
    }
    if let Some((peak_x, peak_y)) = peak_position {
        sketch::diamond(kit, px(peak_x), px(peak_y), 5.0 * ME_K, PROF_ACCENT);
    }
}

fn bands_legend(kit: &CanvasKit<'_>, baseline: f32) {
    let items = [
        ("PM", PROF_BAR_PM),
        ("FR", PROF_BAR_FR),
        ("通关", PROF_BAR_CLEAR),
        ("未通", PROF_BAR_PLAYED),
    ];
    let mut x = 56.0;
    let style = prof_text(11.0, PROF_TEXT_SECOND);
    let center_y = baseline - 11.0 * 0.36;
    for (label, ink) in items {
        let side = 7.0;
        kit.fill_rect(
            band(x, center_y - side / 2.0, x + side, center_y + side / 2.0),
            ink,
        );
        x += side + 5.0;
        let width = sketch::text(kit, &style, label, px(x), px(baseline), PROF_TEXT_SECOND);
        x += width / ME_K + 14.0;
    }
}

fn bands_table(kit: &CanvasKit<'_>, rows: usize) {
    let first = bands_row_at(0);
    let x0 = px(first.x);
    let x1 = px(first.x1());
    let mut ys = vec![px(408.0)];
    for index in 0..rows {
        ys.push(px(bands_row_at(index).y1()));
    }
    for y in ys {
        kit.fill_rect(Rect::from_xywh(x0, y, x1 - x0, px(1.0)), PROF_HAIRLINE);
    }
}

fn bands_played(kit: &CanvasKit<'_>, center_x: f32, baseline: f32, band_data: &MeBand) {
    let played = band_data.played.max(0).to_string();
    let head_style = prof_number(12.0, PROF_TEXT_PRIMARY);
    let tail = if band_data.total > 0 {
        format!("/{}", band_data.total)
    } else {
        String::new()
    };
    let tail_style = prof_number(10.0, PROF_TEXT_MUTED);
    let head_width = sketch::text_width(kit, &head_style, &played) / ME_K;
    let tail_width = if tail.is_empty() {
        0.0
    } else {
        sketch::text_width(kit, &tail_style, &tail) / ME_K
    };
    let x = center_x - (head_width + tail_width) / 2.0;
    sketch::text(
        kit,
        &head_style,
        &played,
        px(x),
        px(baseline),
        PROF_TEXT_PRIMARY,
    );
    if tail.is_empty() {
        return;
    }
    sketch::text(
        kit,
        &tail_style,
        &tail,
        px(x + head_width),
        px(baseline),
        PROF_TEXT_MUTED,
    );
}

fn band_bar(kit: &CanvasKit<'_>, bar: Area, band_data: &MeBand) {
    kit.fill_rect(bar.rect_d(), PROF_GROOVE);
    let total = band_data.total.max(0);
    if total <= 0 {
        return;
    }
    let pm = band_data.pm.max(0).min(total);
    let fc = band_data.full_combo.max(pm).min(total);
    let cleared = band_data.cleared.max(fc).min(total);
    let played = band_data.played.max(cleared).min(total);
    let segments = [
        (pm, PROF_BAR_PM),
        (fc - pm, PROF_BAR_FR),
        (cleared - fc, PROF_BAR_CLEAR),
        (played - cleared, PROF_BAR_PLAYED),
    ];
    let mut x = bar.x;
    for (weight, ink) in segments {
        if weight <= 0 {
            continue;
        }
        let width = bar.w * weight as f32 / total as f32;
        let rect = band(x, bar.y, x + width, bar.y1());
        if rect.width() > 0.0 {
            kit.fill_rect(rect, ink);
        }
        x += width;
    }
}

fn band_row(kit: &CanvasKit<'_>, row: Area, band_data: &MeBand) {
    let chip = Area::new(64.0, row.y + 7.0, 38.0, 18.0);
    kit.fill_rect(chip.rect_d(), diff_ink(&band_data.name));
    sketch::text_center(
        kit,
        &prof_display(10.0, PROF_WHITE),
        &band_data.name,
        px(chip.x + chip.w / 2.0),
        px(chip.y + chip.h / 2.0 + 10.0 * 0.36),
        PROF_WHITE,
    );
    band_bar(kit, Area::new(120.0, row.y + 12.0, 86.0, 8.0), band_data);

    let baseline = row.y + 21.0;
    bands_played(kit, BANDS_COL_X[0], baseline, band_data);
    let values: [i32; 5] = [
        band_data.cleared.max(0),
        band_data.pm.max(0),
        band_data.full_combo.max(0),
        band_data.ex_plus.max(0),
        band_data.ex.max(0),
    ];
    for (index, value) in values.iter().enumerate() {
        sketch::text_center(
            kit,
            &prof_text(12.0, PROF_TEXT_SECOND),
            &value.to_string(),
            px(BANDS_COL_X[index + 1]),
            px(baseline),
            PROF_TEXT_SECOND,
        );
    }
    let ink = if band_data.counts_loss() {
        PROF_TEXT_PRIMARY
    } else {
        PROF_TEXT_MUTED
    };
    sketch::text(
        kit,
        &prof_number(10.0, ink),
        &band_loss_text(band_data),
        px(BANDS_LOSS_X),
        px(baseline),
        ink,
    );
}

fn profile_bands_card(kit: &CanvasKit<'_>, input: &MeInput) {
    let rows = bands_rows(input);
    let card = Area::new(40.0, 346.0, 630.0, 236.0 + bands_growth(input));
    profile_card(kit, card);
    profile_card_head(kit, card, 2, "完成度");
    profile_card_note(kit, card, 377.0, &bands_note(input), PROF_TEXT_SECOND);
    bands_legend(kit, 402.0);
    for (index, label) in BANDS_COL_LABEL.iter().enumerate() {
        sketch::text_center(
            kit,
            &prof_display(11.0, PROF_TEXT_MUTED),
            label,
            px(BANDS_COL_X[index]),
            px(402.0),
            PROF_TEXT_MUTED,
        );
    }
    sketch::text(
        kit,
        &prof_display(11.0, PROF_TEXT_MUTED),
        "失分",
        px(BANDS_LOSS_X),
        px(402.0),
        PROF_TEXT_MUTED,
    );
    let bands = input.bands_ref();
    if bands.is_empty() {
        profile_empty_note(
            kit,
            "还没有成绩 —— 打过一局的难度档会出现在这里",
            56.0,
            432.0,
        );
        return;
    }
    for (index, band_data) in bands.into_iter().take(rows).enumerate() {
        band_row(kit, bands_row_at(index), band_data);
    }
    bands_table(kit, rows);
}

fn change_cell_at(index: usize, growth: f32) -> Area {
    let column = (index % 2) as f32;
    let row = (index / 2) as f32;
    Area::new(60.0 + column * 300.0, 646.0 + growth + row * 68.0, 290.0, 60.0)
}

fn change_note(change: &MeChange) -> String {
    let mut note = String::from("NEW RECORD");
    if let Some(old) = change.old_score {
        note = format!("from {}", sketch::thousands(old));
    }
    if let Some(at) = change.at {
        note.push_str("  ·  ");
        note.push_str(&fmt_datetime(at));
    }
    note
}

fn profile_change_cell(kit: &CanvasKit<'_>, cell: Area, change: &MeChange) {
    hair(kit, cell, PROF_RULE);

    let thumb = Area::new(cell.x + 4.0, cell.y + 4.0, 52.0, 52.0);
    match change.jacket.as_deref().filter(|key| !key.is_empty()) {
        Some(key) => {
            if kit.draw_asset_cover(key, thumb.rect_d(), 0.0).is_err() {
                kit.fill_rect(thumb.rect_d(), PROF_SUB_CARD_ICE);
            }
        }
        None => {
            kit.fill_rect(thumb.rect_d(), PROF_SUB_CARD_ICE);
        }
    }
    hair(kit, thumb, PROF_RULE);

    if !change.diff.is_empty() {
        let bar_x = px(cell.x) + px(1.0);
        kit.fill_rect(
            Rect::from_xywh(
                bar_x,
                px(cell.y) + px(1.0),
                px(3.0),
                px(cell.y1()) - px(1.0) - (px(cell.y) + px(1.0)),
            ),
            diff_ink(&change.diff),
        );
    }

    let info_x = cell.x + 64.0;
    if !change.title.is_empty() {
        let style = prof_display(13.0, PROF_TEXT_PRIMARY);
        let text = sketch::ellipsize(kit, &style, &change.title, px(137.0));
        sketch::text(
            kit,
            &style,
            &text,
            px(info_x),
            px(cell.y + 20.0),
            PROF_TEXT_PRIMARY,
        );
    }

    let line2 = cell.y + 37.0;
    let mut x = info_x;
    if !change.diff.is_empty() {
        let style = prof_display(11.0, diff_ink(&change.diff));
        let width = sketch::text(
            kit,
            &style,
            &change.diff,
            px(x),
            px(line2),
            diff_ink(&change.diff),
        );
        x += width / ME_K;
    }
    sketch::text(
        kit,
        &prof_number(12.0, PROF_TEXT_PRIMARY),
        &format!("  {}", sketch::thousands(change.new_score)),
        px(x),
        px(line2),
        PROF_TEXT_PRIMARY,
    );

    let note = change_note(change);
    let note_style = prof_number(10.0, PROF_TEXT_SECOND);
    let note_text = sketch::ellipsize(kit, &note_style, &note, px(cell.w - 72.0));
    sketch::text(
        kit,
        &note_style,
        &note_text,
        px(info_x),
        px(cell.y + 51.0),
        PROF_TEXT_SECOND,
    );

    let right = cell.x1() - 8.0;
    match change.old_score {
        None => {
            let stamp = Area::new(right - 46.0, cell.y + (60.0 - 18.0) / 2.0, 46.0, 18.0);
            hair(kit, stamp, PROF_WINE);
            sketch::text_center(
                kit,
                &prof_display(12.0, PROF_WINE),
                "NEW",
                px(stamp.x + stamp.w / 2.0),
                px(stamp.y + stamp.h / 2.0 + 12.0 * 0.36),
                PROF_WINE,
            );
        }
        Some(old) => {
            let delta = change.new_score - old;
            sketch::text_right(
                kit,
                &prof_number(12.0, PROF_GAIN),
                &thousands(delta),
                px(right),
                px(cell.y + 36.0),
                PROF_GAIN,
            );
        }
    }
}

fn profile_changes_card(kit: &CanvasKit<'_>, input: &MeInput) {
    let growth = bands_growth(input);
    let card = Area::new(40.0, 598.0 + growth, 630.0, 262.0);
    profile_card(kit, card);
    profile_card_head(kit, card, 3, "最近变化");
    profile_card_note(
        kit,
        card,
        card.y + 32.0,
        &changes_note(input),
        PROF_TEXT_SECOND,
    );
    let changes = input.changes_ref();
    if changes.is_empty() {
        profile_empty_note(
            kit,
            "还没有记录到变化 —— 再打一局、分数或评级比上次高, 这里就会长出一格",
            60.0,
            708.0 + growth,
        );
        return;
    }
    for (index, change) in changes.iter().take(6).enumerate() {
        profile_change_cell(kit, change_cell_at(index, growth), change);
    }
}

fn scatter_legend_label(kit: &CanvasKit<'_>, x_right: f32, baseline: f32, text: &str) -> f32 {
    let style = prof_display(11.0, PROF_TEXT_SECOND);
    let width = sketch::text_width(kit, &style, text) / ME_K;
    sketch::text_right(
        kit,
        &style,
        text,
        px(x_right),
        px(baseline),
        PROF_TEXT_SECOND,
    );
    x_right - width
}

fn profile_scatter_legend(kit: &CanvasKit<'_>, card: Area) {
    let baseline = card.y + 32.0;
    let mid = baseline - 11.0 * 0.36;
    let mut x = card.x1() - 20.0;

    x = scatter_legend_label(kit, x, baseline, "成绩区间");
    x -= 6.0;
    kit.rrect_fill(
        band(x - SCATTER_CAP_W, baseline - 13.0, x, baseline - 1.0),
        px(SCATTER_CAP_W / 2.0),
        PROF_CAPSULE,
    );
    x -= SCATTER_CAP_W + 4.0 + 14.0;

    x = scatter_legend_label(kit, x, baseline, "定数最高");
    x -= 5.0;
    sketch::diamond(kit, px(x - 3.5), px(mid), 4.0 * ME_K, PROF_ACCENT_DARK);

    let style = prof_display(11.0, PROF_TEXT_SECOND);
    let axis = "横轴 定数 / 纵轴 单曲 PTT";
    sketch::text_right(
        kit,
        &style,
        axis,
        px(x - 16.0),
        px(baseline),
        PROF_TEXT_MUTED,
    );
}

fn scatter_tag_width(kit: &CanvasKit<'_>, text: &str) -> f32 {
    let style = prof_display(10.0, PROF_ACCENT_DARK);
    sketch::text_width(kit, &style, text) / ME_K + 10.0
}

fn scatter_tag_clear(anchors: &[(f32, f32)], x: f32, y: f32, width: f32) -> bool {
    !anchors.iter().any(|(anchor_x, anchor_y)| {
        *anchor_y >= y - 4.0
            && *anchor_y <= y + 22.0
            && *anchor_x >= x - 5.0
            && *anchor_x <= x + width + 5.0
    })
}

fn scatter_tag(kit: &CanvasKit<'_>, x: f32, y: f32, text: &str) {
    let style = prof_display(10.0, PROF_ACCENT_DARK);
    let width = sketch::text_width(kit, &style, text) / ME_K;
    kit.rrect_fill(band(x, y, x + width + 10.0, y + 18.0), 0.0, PROF_PILL);
    sketch::text(
        kit,
        &style,
        text,
        px(x + 5.0),
        px(y + 18.0 - 5.0),
        PROF_ACCENT_DARK,
    );
}

fn scatter_tag_right(kit: &CanvasKit<'_>, x_right: f32, y: f32, text: &str) {
    let style = prof_display(10.0, PROF_ACCENT_DARK);
    let width = sketch::text_width(kit, &style, text) / ME_K + 10.0;
    scatter_tag(kit, x_right - width, y, text);
}

fn profile_scatter_card(kit: &CanvasKit<'_>, input: &MeInput) {
    let card = Area::new(686.0, 114.0, 674.0, 300.0);
    profile_card(kit, card);
    profile_card_head(kit, card, 4, "单曲 PTT 分布");
    profile_scatter_legend(kit, card);

    let plot = Area::new(716.0, 175.0, 584.0, 200.0);
    let scatter = match input.scatter_ref() {
        Some(scatter) => scatter,
        None => {
            profile_empty_note(
                kit,
                "还没有算得出单曲 PTT 的成绩",
                plot.x,
                plot.y + plot.h / 2.0,
            );
            return;
        }
    };
    let all = scatter_points(scatter);
    let has_ability = scatter.ability.is_some();
    let (start, end) = scatter_domain(&all, scatter.ability);
    let points = scatter_in_range(&all, start);
    if points.is_empty() {
        profile_empty_note(
            kit,
            "还没有算得出单曲 PTT 的成绩",
            plot.x,
            plot.y + plot.h / 2.0,
        );
        return;
    }
    let data_left = plot.x + 6.0;
    let data_right = plot.x1() - 6.0;
    let filled = scatter_filled(&points);
    let basis: &[ScatterPoint] = if filled.is_empty() { &points } else { &filled };
    let (lo, hi) = scatter_window(basis);
    let x_of = |constant: f64| -> f32 {
        if end <= start {
            return (data_left + data_right) / 2.0;
        }
        data_left + (data_right - data_left) * ((constant - start) / (end - start)) as f32
    };
    let y_of = |value: f64| -> f32 {
        if hi <= lo {
            return plot.y1();
        }
        let mapped = plot.y1() - (plot.y1() - plot.y) * ((value - lo) / (hi - lo)) as f32;
        mapped.max(plot.y).min(plot.y1())
    };

    let tick_style = prof_number(10.0, PROF_TEXT_MUTED);
    for value in scatter_ptt_ticks(lo, hi) {
        let y = y_of(value);
        dash_rule_1(kit, plot.x + 12.0, plot.x1() - 12.0, y, PROF_HAIRLINE);
        sketch::text_right(
            kit,
            &tick_style,
            &ptt3(value),
            px(1335.0),
            px(y + 10.0 * 0.36),
            PROF_TEXT_MUTED,
        );
    }
    for constant in scatter_domain_ticks(start, end) {
        sketch::text_center(
            kit,
            &tick_style,
            &format!("{constant:.1}"),
            px(x_of(constant)),
            px(392.0),
            PROF_TEXT_MUTED,
        );
    }

    let mut anchors: Vec<(f32, f32)> = Vec::new();
    for point in basis {
        let x = x_of(point.constant);
        let mean = y_of(point.avg);
        let top = y_of(point.best)
            .min(plot.y1() - SCATTER_CAP_MIN_H)
            .max(plot.y);
        let bottom = y_of(point.low)
            .max(top + SCATTER_CAP_MIN_H)
            .min(plot.y1());
        anchors.push((x, top));
        anchors.push((x, mean));
        kit.rrect_fill(
            band(
                x - SCATTER_CAP_W / 2.0,
                top,
                x + SCATTER_CAP_W / 2.0,
                bottom,
            ),
            px(SCATTER_CAP_W / 2.0),
            PROF_CAPSULE,
        );
        sketch::diamond(kit, px(x), px(top), 4.0 * ME_K, PROF_ACCENT_DARK);
    }

    if let Some(avg) = scatter.avg_ptt {
        if avg > lo && avg < hi {
            let y = y_of(avg);
            dash_h(
                kit,
                plot.x,
                plot.x1(),
                y,
                DashStyle {
                    dash: Dash { len: 6.0, gap: 4.0 },
                    thick: 1.5,
                    radius: 1.5 / 2.0 * ME_K,
                    color: PROF_DASH_INK,
                },
            );
            let text = format!("均值 {}", ptt3(avg));
            let width = scatter_tag_width(kit, &text);
            let right = plot.x1() - 2.0;
            let left = plot.x + 2.0;
            let tag_y = (y - 18.0 - 4.0).max(plot.y + 1.0);
            if scatter_tag_clear(&anchors, right - width, tag_y, width)
                || !scatter_tag_clear(&anchors, left, tag_y, width)
            {
                scatter_tag_right(kit, right, tag_y, &text);
            } else {
                scatter_tag(kit, left, tag_y, &text);
            }
        }
    }
    if has_ability {
        if let Some(ability) = scatter.ability {
            if ability > start && ability < end {
                let x = x_of(ability);
                dash_v(
                    kit,
                    x,
                    plot.y,
                    432.0,
                    DashStyle {
                        dash: Dash { len: 6.0, gap: 4.0 },
                        thick: 1.5,
                        radius: 1.5 / 2.0 * ME_K,
                        color: PROF_DASH_INK,
                    },
                );
                let text = format!("能力定数 {}", ptt3(ability));
                let width = scatter_tag_width(kit, &text);
                let mut tag_x = x + 6.0;
                if tag_x + width > plot.x1() - 2.0 {
                    tag_x = (x - 6.0 - width).max(plot.x + 2.0);
                }
                scatter_tag(kit, tag_x, plot.y + 2.0, &text);
            }
        }
    }
}

const BOARD_GRID_TOP: f32 = 480.0;
const BOARD_ROW_H: f32 = 68.0;
const BOARD_GRID_ROWS: usize = 4;

fn board_row_h(growth: f32) -> f32 {
    BOARD_ROW_H + growth / BOARD_GRID_ROWS as f32
}

fn board_grid_bottom(growth: f32) -> f32 {
    BOARD_GRID_TOP + BOARD_GRID_ROWS as f32 * board_row_h(growth)
}

fn board_cell_at(index: usize, growth: f32) -> Area {
    const COLS: usize = 3;
    let grid = BOARD_GRID_ROWS * COLS;
    if index >= grid {
        let shift = (index - grid).min(1) as f32;
        let y = board_grid_bottom(growth) + 12.0;
        return Area::new(706.0 + shift * 211.0, y, 211.0, BOARD_ROW_H);
    }
    let column = (index % COLS) as f32;
    let row = (index / COLS) as f32;
    let h = board_row_h(growth);
    Area::new(706.0 + column * 211.0, BOARD_GRID_TOP + row * h, 211.0, h)
}

fn board_list_cell_at(index: usize, growth: f32) -> Area {
    let column = (index % 3) as f32;
    let row = (index / 3) as f32;
    let h = board_row_h(growth);
    Area::new(706.0 + column * 211.0, BOARD_GRID_TOP + row * h, 211.0, h)
}

fn board_grid(kit: &CanvasKit<'_>, rows: usize, growth: f32) {
    let x0 = px(706.0);
    let x1 = px(706.0 + 211.0 * 3.0);
    let h = board_row_h(growth);
    for row in 1..=rows {
        let y = px(BOARD_GRID_TOP + row as f32 * h);
        kit.fill_rect(
            Rect::from_xywh(x0, y - px(1.0), x1 - x0, px(1.0)),
            PROF_HAIRLINE,
        );
    }
}

fn board_rule(kit: &CanvasKit<'_>, growth: f32) {
    let y = px(board_grid_bottom(growth) + 6.0);
    let soft = Color::new(0x2C, 0x2A, 0x27, 0xB4);
    let mut x: f32 = 706.0;
    while x < 1339.0 {
        let xe = (x + 7.0).min(1339.0);
        let width = px(xe) - px(x);
        kit.fill_rect(Rect::from_xywh(px(x), y, width, px(1.0)), PROF_INK_RULE);
        kit.fill_rect(
            Rect::from_xywh(px(x), y + px(1.0), width, (px(1.0) / 2.0).max(1.0)),
            soft,
        );
        x += 12.0;
    }
}

struct AvatarSpec<'a> {
    avatar: Option<&'a str>,
    ptt: Option<f64>,
    frame: Option<i32>,
    accent: Option<i32>,
    with_digits: bool,
}

fn draw_avatar_plain(kit: &CanvasKit<'_>, cx: f32, cy: f32, radius: f32, spec: AvatarSpec<'_>) {
    let avatar = super::avatar::Avatar::new(spec.avatar)
        .with_ptt(spec.ptt)
        .with_frame(spec.frame);
    let avatar = if spec.with_digits {
        avatar
    } else {
        avatar.without_digits()
    };
    let _ = spec.accent;
    super::avatar::draw(kit, cx, cy, radius, &avatar);
}

fn board_cell(kit: &CanvasKit<'_>, cell: Area, entry: &MeBoardRow, foot: bool) {
    let center_y = cell.y + cell.h / 2.0;
    let baseline = center_y + 5.0;
    if entry.me && entry.rank > 0 {
        let bottom = px(cell.y1()) - px(1.0);
        kit.fill_rect(
            Rect::from_xywh(
                px(cell.x),
                px(cell.y),
                px(cell.x1()) - px(cell.x),
                bottom - px(cell.y),
            ),
            PROF_SELF_ROW,
        );
        kit.fill_rect(
            Rect::from_xywh(px(cell.x), px(cell.y), px(3.0), bottom - px(cell.y)),
            PROF_WINE,
        );
    }
    if foot {
        hair(kit, cell, PROF_RULE);
    }
    let rank_ink = if entry.rank > 0 && entry.rank <= 3 {
        PROF_ACCENT
    } else {
        PROF_TEXT_SECOND
    };
    let rank = if entry.rank > 0 {
        format!("#{}", entry.rank)
    } else {
        String::from("—")
    };
    let rank_size = if entry.rank >= 100 { 11.0 } else { 13.0 };
    sketch::text_right(
        kit,
        &prof_mono(rank_size, rank_ink),
        &rank,
        px(cell.x + 42.0),
        px(baseline),
        rank_ink,
    );
    draw_avatar_plain(
        kit,
        px(cell.x + 66.0),
        px(center_y),
        px(18.0),
        AvatarSpec {
            avatar: entry.avatar.as_deref(),
            ptt: entry.ptt,
            frame: entry.rating_frame,
            accent: entry.rating_accent,
            with_digits: true,
        },
    );
    let style = prof_display(13.0, PROF_TEXT_PRIMARY);
    let name = if entry.name.is_empty() {
        String::from("—")
    } else {
        entry.name.clone()
    };
    let text = sketch::ellipsize(kit, &style, &name, px(106.0));
    sketch::text(
        kit,
        &style,
        &text,
        px(cell.x + 96.0),
        px(center_y + 5.0),
        PROF_TEXT_PRIMARY,
    );
}

fn board_gap(kit: &CanvasKit<'_>, board: &MeBoard, growth: f32) {
    let cell = Area::new(1128.0, board_grid_bottom(growth) + 12.0, 211.0, BOARD_ROW_H);
    hair(kit, cell, PROF_RULE);
    let x = cell.x + 14.0;

    let label_style = prof_text(9.0, PROF_TEXT_MUTED);
    let gap_label_width = sketch::text_width(kit, &label_style, "GAP") / ME_K;
    sketch::text(
        kit,
        &label_style,
        "GAP",
        px(x),
        px(cell.y + 28.0),
        PROF_TEXT_MUTED,
    );
    let gap_text = match gap_to_prev(board) {
        Some(delta) => signed_ptt4(delta),
        None => String::from("—"),
    };
    sketch::text(
        kit,
        &prof_mono(13.0, PROF_WINE),
        &gap_text,
        px(x + gap_label_width + 8.0),
        px(cell.y + 28.0),
        PROF_WINE,
    );

    let rank_text = board
        .rows
        .iter()
        .find(|row| row.me && row.rank > 0)
        .map(|row| format!("#{}", row.rank))
        .unwrap_or_else(|| String::from("—"));
    let rank_label_width = sketch::text_width(kit, &label_style, "CURRENT RANK") / ME_K;
    sketch::text(
        kit,
        &label_style,
        "CURRENT RANK",
        px(x),
        px(cell.y + 50.0),
        PROF_TEXT_MUTED,
    );
    sketch::text(
        kit,
        &prof_mono(12.0, PROF_TEXT_PRIMARY),
        &rank_text,
        px(x + rank_label_width + 8.0),
        px(cell.y + 50.0),
        PROF_TEXT_PRIMARY,
    );
}

fn profile_board_card(kit: &CanvasKit<'_>, input: &MeInput) {
    let growth = bands_growth(input);
    let card = Area::new(686.0, 430.0, 674.0, 430.0 + growth);
    profile_card(kit, card);
    profile_card_head(kit, card, 5, "全站排行");
    let board = match input.board_ref() {
        Some(board) => board,
        None => {
            profile_card_note(kit, card, card.y + 32.0, "", PROF_TEXT_SECOND);
            profile_empty_note(kit, "榜单还没有数据", 706.0, 542.0);
            return;
        }
    };
    profile_card_note(
        kit,
        card,
        card.y + 32.0,
        &board_note(board),
        PROF_TEXT_SECOND,
    );
    let cells = board_cells(board);
    if cells.is_empty() {
        profile_empty_note(kit, "榜单还没有数据", 706.0, 542.0);
        return;
    }
    if board_list_layout(board) {
        let mut rows = cells.len().div_ceil(3);
        rows = rows.clamp(1, 5);
        board_grid(kit, rows, growth);
        for (index, entry) in cells.iter().take(15).enumerate() {
            board_cell(kit, board_list_cell_at(index, growth), entry, false);
        }
        return;
    }
    board_grid(kit, 4, growth);
    if cells.len() > 12 {
        board_rule(kit, growth);
    }
    for (index, entry) in cells.iter().take(14).enumerate() {
        board_cell(kit, board_cell_at(index, growth), entry, index >= 12);
    }
    if cells.len() > 12 {
        board_gap(kit, board, growth);
    }
}

fn profile_footer(kit: &CanvasKit<'_>, input: &MeInput) {
    sketch::text(
        kit,
        &prof_display(11.0, PROF_TEXT_SECOND),
        "GENERATED BY OKAMIKKO",
        px(40.0),
        px(886.0 + bands_growth(input)),
        PROF_TEXT_SECOND,
    );
}

fn profile_player(kit: &CanvasKit<'_>, input: &MeInput) {
    let block = player::Block::new(&input.badge, 1360.0 * ME_K, 66.99 * ME_K)
        .with_left(72.0 * ME_K)
        .with_scale(ME_K)
        .without_shadow()
        .with_rating(input.rating_frame, input.rating_accent);
    player::paint(kit, &block);
}

fn profile_top_cards_rule(kit: &CanvasKit<'_>) {
    kit.fill_rect(band(40.0, 114.0, 1360.0, 116.0), PROF_INK_RULE);
}

pub fn paint(kit: &CanvasKit<'_>, input: &MeInput) {
    profile_backdrop(kit, input);
    profile_top_bar(kit);
    profile_top_cards_rule(kit);
    profile_header(kit, input);
    profile_curve_card(kit, input);
    profile_bands_card(kit, input);
    profile_changes_card(kit, input);
    profile_scatter_card(kit, input);
    profile_board_card(kit, input);
    profile_footer(kit, input);
    profile_player(kit, input);
}

#[derive(Debug, Clone, Copy)]
struct PickWall {
    cols: i32,
    aspect: f32,
    inner: f32,
}

const PICK_SQUARE: PickWall = PickWall {
    cols: 8,
    aspect: 1.0,
    inner: 0.8,
};

const PICK_WIDE: PickWall = PickWall {
    cols: 3,
    aspect: 4.2,
    inner: 0.94,
};

impl PickWall {
    fn cols_for(self, count: usize) -> i32 {
        if count == 0 {
            return 1;
        }
        if (count as i32) < self.cols {
            return count as i32;
        }
        self.cols
    }

    fn rows(self, count: usize) -> i32 {
        if count == 0 {
            return 1;
        }
        let cols = self.cols_for(count);
        (count as i32 + cols - 1) / cols
    }

    fn cell_w(self, cols: i32) -> f32 {
        let cols = cols.max(self.cols);
        (PICK_BASE_W - 2.0 * PICK_MARGIN_X - (cols - 1) as f32 * PICK_GAP) / cols as f32
    }

    fn cell_size(self, count: usize) -> (i32, i32) {
        let width = self.cell_w(self.cols_for(count)).round();
        (pti(width), pti((width / self.aspect).round()))
    }

    fn width(self, count: usize) -> f32 {
        let cols = self.cols_for(count);
        2.0 * PICK_MARGIN_X + cols as f32 * self.cell_w(cols) + (cols - 1) as f32 * PICK_GAP
    }

    fn height(self, count: usize) -> f32 {
        let rows = self.rows(count);
        let cell_h = self.cell_w(self.cols_for(count)) / self.aspect;
        PICK_GRID_TOP + rows as f32 * cell_h + (rows - 1) as f32 * PICK_GAP + PICK_FOOTER_H
    }

    fn canvas(self, count: usize) -> (i32, i32) {
        (
            pti(self.width(count).round()),
            pti(self.height(count).round()),
        )
    }

    fn cell_rect(self, index: usize, count: usize) -> (i32, i32, i32, i32) {
        if count == 0 || index >= count {
            return (0, 0, 0, 0);
        }
        let cols = self.cols_for(count);
        let row = index as i32 / cols;
        let column = index as i32 % cols;
        let mut in_row = count as i32 - row * cols;
        if in_row > cols {
            in_row = cols;
        }
        let gap = pti(PICK_GAP);
        let (cw, ch) = self.cell_size(count);
        let step_x = cw + gap;
        let step_y = ch + gap;
        let offset = (cols - in_row) * step_x / 2;
        let x = pti(PICK_MARGIN_X) + offset + column * step_x;
        let y = pti(PICK_GRID_TOP) + row * step_y;
        (x, y, cw, ch)
    }
}

fn pick_ground(kit: &CanvasKit<'_>, width: f32, height: f32) {
    let area = Rect::from_xywh(0.0, 0.0, width, height);
    kit.fill_rect(area, PICK_GROUND);
}

fn pick_head(kit: &CanvasKit<'_>, width: f32, title: &str, note: &str, hint: &str) {
    let margin = pt(PICK_MARGIN_X);
    let head_style = wall_text(46.0, PICK_TEXT_MAIN);
    let note_style = wall_text(24.0 * 0.93, PICK_TEXT_DIM);
    let hint_style = wall_text(24.0 * 0.93, PICK_TEXT_DIM);
    let hint_width = sketch::text_width(kit, &hint_style, hint);
    let max_width = width - margin - margin - hint_width - pt(24.0);
    let title = sketch::ellipsize(kit, &head_style, title, max_width.max(0.0));
    sketch::text(kit, &head_style, &title, margin, pt(126.0), PICK_TEXT_MAIN);
    let note = sketch::ellipsize(kit, &note_style, note, max_width.max(0.0));
    sketch::text(kit, &note_style, &note, margin, pt(172.0), PICK_TEXT_DIM);
    if !hint.is_empty() {
        sketch::text_right(
            kit,
            &hint_style,
            hint,
            width - pt(PICK_MARGIN_X),
            pt(126.0),
            PICK_TEXT_DIM,
        );
    }
}

fn pick_footer(kit: &CanvasKit<'_>, width: f32, height: f32, foot: &str) {
    let bottom = height - pt(PICK_FOOTER_H);
    let margin = pt(PICK_MARGIN_X);
    let right = width - margin;
    let rule = pt(2.0).max(1.0);
    let y = bottom + pt(44.0);
    kit.fill_rect(Rect::from_xywh(margin, y, right - margin, rule), PICK_RULE);
    if foot.is_empty() {
        return;
    }
    let style = wall_text(24.0 * 0.93, PICK_TEXT_DIM);
    let text = sketch::ellipsize(kit, &style, foot, right - margin);
    sketch::text(
        kit,
        &style,
        &text,
        margin,
        bottom + pt(96.0),
        PICK_TEXT_DIM,
    );
}

fn pick_badge(kit: &CanvasKit<'_>, x: i32, y: i32, cw: i32, ch: i32, index: i32) {
    let text = index.to_string();
    let digits = text.chars().count() as i32;
    let height = ch * 190 / 1000;
    let width = ((digits as f32 * 0.34).max(0.56) * height as f32).round() as i32;
    let pad = ch * 22 / 1000;
    let x0 = (x + cw - pad - width) as f32;
    let y0 = (y + ch - pad - height) as f32;
    sketch::fill_round(
        kit,
        Rect::from_xywh(x0, y0, width as f32, height as f32),
        height as f32 * 0.22,
        PICK_CHIP,
    );
    let size = height as f32 * 0.62;
    sketch::text_center(
        kit,
        &wall_number(size, PICK_TEXT_MAIN),
        &text,
        x0 + width as f32 / 2.0,
        y0 + height as f32 / 2.0 + size * 0.36,
        PICK_TEXT_MAIN,
    );
}

fn pick_cell(kit: &CanvasKit<'_>, rect: (i32, i32, i32, i32), key: &str, index: i32, inner: f32) {
    let (x, y, cw, ch) = rect;
    if cw <= 0 || ch <= 0 {
        return;
    }
    let radius = ch as f32 * 0.09;
    kit.rrect_fill(
        Rect::from_xywh(x as f32, y as f32, cw as f32, ch as f32),
        radius,
        PICK_PLATE,
    );
    let inner_w = cw as f32 * inner;
    let inner_h = ch as f32 * inner;
    if key.is_empty() {
        let area = Rect::from_xywh(x as f32, y as f32, inner_w, inner_h);
        kit.rrect_fill(area, 0.0, PICK_RULE);
    } else {
        let size = kit
            .asset(key)
            .ok()
            .and_then(|texture| texture.dimensions());
        match size {
            Some((iw, ih)) if iw > 0 && ih > 0 => {
                let k = (inner_w / iw as f32).min(inner_h / ih as f32);
                let w = (iw as f32 * k).round();
                let h = (ih as f32 * k).round();
                let ox = x + (cw - w as i32) / 2;
                let oy = y + (ch - h as i32) / 2;
                let _ = kit.draw_asset(key, Rect::from_xywh(ox as f32, oy as f32, w, h), 0.0);
            }
            _ => {
                let area = Rect::from_xywh(x as f32, y as f32, inner_w, inner_h);
                kit.rrect_fill(area, 0.0, PICK_RULE);
            }
        }
    }
    pick_badge(kit, x, y, cw, ch, index);
}

fn paint_wall(
    kit: &CanvasKit<'_>,
    wall: PickWall,
    title: &str,
    note: &str,
    hint: &str,
    foot: &str,
    cells: &[(String, i32)],
) {
    let count = cells.len();
    let (width, height) = wall.canvas(count);
    let width = width as f32;
    let height = height as f32;
    pick_ground(kit, width, height);
    pick_head(kit, width, title, note, hint);
    for (index, (key, number)) in cells.iter().enumerate() {
        pick_cell(kit, wall.cell_rect(index, count), key, *number, wall.inner);
    }
    pick_footer(kit, width, height, foot);
}

#[derive(Debug, Clone)]
pub struct CharCell {
    pub name: String,
    pub label: String,
    pub thumb: String,
    pub art: String,
    pub unlocked: bool,
}

impl CharCell {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let thumb = format!("char_thumbs/{name}.png");
        let art = format!("characters/{name}.png");
        Self {
            label: name.clone(),
            name,
            thumb,
            art,
            unlocked: true,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn with_thumb(mut self, key: impl Into<String>) -> Self {
        self.thumb = key.into();
        self
    }

    pub fn with_art(mut self, key: impl Into<String>) -> Self {
        self.art = key.into();
        self
    }

    pub fn with_unlocked(mut self, unlocked: bool) -> Self {
        self.unlocked = unlocked;
        self
    }

    fn image_key(&self) -> String {
        if !self.thumb.is_empty() {
            self.thumb.clone()
        } else {
            self.art.clone()
        }
    }
}

#[derive(Debug, Clone)]
pub struct CharInput {
    pub title: String,
    pub subtitle: String,
    pub theme: Theme,
    pub cells: Vec<CharCell>,
    pub group: String,
    pub note: String,
}

impl CharInput {
    pub fn new() -> Self {
        Self {
            title: CardKind::Char.label().to_string(),
            subtitle: String::new(),
            theme: Theme::DEFAULT,
            cells: Vec::new(),
            group: String::new(),
            note: String::new(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = subtitle.into();
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.group = group.into();
        self
    }

    pub fn group_text(&self) -> String {
        if self.group.trim().is_empty() {
            String::from("全部立绘")
        } else {
            self.group.clone()
        }
    }

    pub fn unlocked_count(&self) -> usize {
        self.cells.iter().filter(|cell| cell.unlocked).count()
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = note.into();
        self
    }

    pub fn with_cell(mut self, cell: CharCell) -> Self {
        self.cells.push(cell);
        self
    }

    pub fn with_cells(mut self, cells: Vec<CharCell>) -> Self {
        self.cells = cells;
        self
    }
}

impl Default for CharInput {
    fn default() -> Self {
        Self::new().with_subtitle("角色立绘一览")
    }
}

pub fn char_size(input: &CharInput) -> (i32, i32) {
    let (_, height) = PICK_SQUARE.canvas(input.cells.len());
    (CHAR_WIDTH, height.clamp(MIN_HEIGHT, MAX_HEIGHT))
}

pub fn char_request(input: &CharInput) -> RenderRequest {
    let (width, height) = char_size(input);
    request_for(width, height)
}

pub fn char_card(input: CharInput) -> RenderCard {
    let request = char_request(&input);
    let label = format!(
        "{} {}x{}",
        CardKind::Char.label(),
        request.width,
        request.height
    );
    let cache_key = super::memo::fingerprint("char", &input);
    RenderCard::new(request, label, move |kit| paint_char(kit, &input)).with_cache_key(cache_key)
}

pub fn paint_char(kit: &CanvasKit<'_>, input: &CharInput) {
    let cells: Vec<(String, i32)> = input
        .cells
        .iter()
        .enumerate()
        .map(|(index, cell)| (cell.image_key(), index as i32 + 1))
        .collect();
    paint_wall(
        kit,
        PICK_SQUARE,
        &input.title,
        &input.subtitle,
        &input.group_text(),
        &input.note,
        &cells,
    );
}

#[derive(Debug, Clone)]
pub struct BannerCell {
    pub key: String,
    pub label: String,
    pub unlocked: bool,
}

impl BannerCell {
    pub fn new(key: impl Into<String>) -> Self {
        let key = key.into();
        let label = key
            .rsplit('/')
            .next()
            .unwrap_or(&key)
            .trim_end_matches(".png")
            .to_string();
        Self {
            key,
            label,
            unlocked: true,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn with_unlocked(mut self, unlocked: bool) -> Self {
        self.unlocked = unlocked;
        self
    }

    pub fn is_course(&self) -> bool {
        self.label.starts_with("course_")
    }

    pub fn is_online(&self) -> bool {
        self.label.starts_with("online_")
    }
}

#[derive(Debug, Clone)]
pub struct BannerInput {
    pub title: String,
    pub subtitle: String,
    pub theme: Theme,
    pub cells: Vec<BannerCell>,
    pub group: String,
    pub note: String,
}

impl BannerInput {
    pub fn new() -> Self {
        Self {
            title: CardKind::Banner.label().to_string(),
            subtitle: String::new(),
            theme: Theme::DEFAULT,
            cells: Vec::new(),
            group: String::new(),
            note: String::new(),
        }
    }

    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.group = group.into();
        self
    }

    pub fn group_text(&self) -> String {
        if self.group.trim().is_empty() {
            String::from("全部横幅")
        } else {
            self.group.clone()
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = subtitle.into();
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = note.into();
        self
    }

    pub fn with_cell(mut self, cell: BannerCell) -> Self {
        self.cells.push(cell);
        self
    }

    pub fn with_cells(mut self, cells: Vec<BannerCell>) -> Self {
        self.cells = cells;
        self
    }

    pub fn course_count(&self) -> usize {
        self.cells.iter().filter(|cell| cell.is_course()).count()
    }

    pub fn online_count(&self) -> usize {
        self.cells.iter().filter(|cell| cell.is_online()).count()
    }
}

impl Default for BannerInput {
    fn default() -> Self {
        Self::new().with_subtitle("资料卡横幅一览")
    }
}

pub fn banner_size(input: &BannerInput) -> (i32, i32) {
    let (_, height) = PICK_WIDE.canvas(input.cells.len());
    (BANNER_WIDTH, height.clamp(MIN_HEIGHT, MAX_HEIGHT))
}

pub fn banner_request(input: &BannerInput) -> RenderRequest {
    let (width, height) = banner_size(input);
    request_for(width, height)
}

pub fn banner_card(input: BannerInput) -> RenderCard {
    let request = banner_request(&input);
    let label = format!(
        "{} {}x{}",
        CardKind::Banner.label(),
        request.width,
        request.height
    );
    let cache_key = super::memo::fingerprint("banner", &input);
    RenderCard::new(request, label, move |kit| paint_banner(kit, &input)).with_cache_key(cache_key)
}

pub fn paint_banner(kit: &CanvasKit<'_>, input: &BannerInput) {
    let cells: Vec<(String, i32)> = input
        .cells
        .iter()
        .enumerate()
        .map(|(index, cell)| (cell.key.clone(), index as i32 + 1))
        .collect();
    paint_wall(
        kit,
        PICK_WIDE,
        &input.title,
        &input.subtitle,
        &input.group_text(),
        &input.note,
        &cells,
    );
}

#[derive(Debug, Clone)]
pub struct ThemeEntry {
    pub name: String,
    pub label: String,
    pub preview: Option<String>,
    pub active: bool,
}

impl ThemeEntry {
    pub fn new(name: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            preview: None,
            active: false,
        }
    }

    pub fn with_preview(mut self, key: impl Into<String>) -> Self {
        let key = key.into();
        if !key.is_empty() {
            self.preview = Some(key);
        }
        self
    }

    pub fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

}

#[derive(Debug, Clone)]
pub struct ThemeInput {
    pub title: String,
    pub subtitle: String,
    pub theme: Theme,
    pub themes: Vec<ThemeEntry>,
    pub note: String,
}

impl ThemeInput {
    pub fn new() -> Self {
        Self {
            title: String::from("出图主题"),
            subtitle: String::new(),
            theme: Theme::DEFAULT,
            themes: Vec::new(),
            note: String::new(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = subtitle.into();
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = note.into();
        self
    }

    pub fn with_entry(mut self, entry: ThemeEntry) -> Self {
        self.themes.push(entry);
        self
    }

    pub fn with_entries(mut self, themes: Vec<ThemeEntry>) -> Self {
        self.themes = themes;
        self
    }

    pub fn active_name(&self) -> String {
        self.themes
            .iter()
            .find(|entry| entry.active)
            .map(|entry| entry.label.clone())
            .unwrap_or_else(|| self.theme.label.to_string())
    }
}

impl Default for ThemeInput {
    fn default() -> Self {
        Self::new().with_subtitle("主题一览")
    }
}

fn theme_row_rect(index: usize) -> Area {
    let row_height = pt(THEME_ROW_H);
    let row_gap = pt(THEME_ROW_GAP);
    Area::new(
        pt(PICK_MARGIN_X),
        pt(PICK_GRID_TOP) + index as f32 * (row_height + row_gap),
        pt(PICK_BASE_W - 2.0 * PICK_MARGIN_X),
        row_height,
    )
}

pub fn theme_size(input: &ThemeInput) -> (i32, i32) {
    let count = input.themes.len().max(1) as f32;
    let height = pt(PICK_GRID_TOP)
        + count * pt(THEME_ROW_H)
        + (count - 1.0) * pt(THEME_ROW_GAP)
        + pt(PICK_FOOTER_H);
    (
        THEME_WIDTH,
        (height.round() as i32).clamp(MIN_HEIGHT, MAX_HEIGHT),
    )
}

pub fn theme_request(input: &ThemeInput) -> RenderRequest {
    let (width, height) = theme_size(input);
    request_for(width, height)
}

pub fn theme_card(input: ThemeInput) -> RenderCard {
    let request = theme_request(&input);
    let label = format!(
        "{} {}x{}",
        CardKind::Theme.label(),
        request.width,
        request.height
    );
    let cache_key = super::memo::fingerprint("theme", &input);
    RenderCard::new(request, label, move |kit| paint_theme(kit, &input)).with_cache_key(cache_key)
}

fn theme_row_no(kit: &CanvasKit<'_>, x: f32, y: f32, side: f32, number: i32) {
    let size = side * 0.40;
    sketch::text_center(
        kit,
        &TextStyle::new(theme::FONT_TITLE, font_wall(size)).with_color(PICK_TEXT_DIM),
        &number.to_string(),
        x + side / 2.0,
        y + side / 2.0 + size * 0.36,
        PICK_TEXT_DIM,
    );
}

fn theme_current_tag(kit: &CanvasKit<'_>, right: f32, center_y: f32) -> f32 {
    let style = wall_text(22.0 * 0.93, PICK_PLATE);
    let width = sketch::text_width(kit, &style, "当前");
    let pad_x = pt(16.0);
    let height = pt(38.0);
    let x1 = right;
    let x0 = x1 - width - 2.0 * pad_x;
    kit.rrect_fill(
        Rect::from_xywh(x0, center_y - height / 2.0, x1 - x0, height),
        height / 2.0,
        PICK_TEXT_MAIN,
    );
    sketch::text_center(
        kit,
        &style,
        "当前",
        (x0 + x1) / 2.0,
        center_y + style.size as f32 * 0.36,
        PICK_PLATE,
    );
    width + 2.0 * pad_x + pt(16.0)
}

fn theme_row(
    kit: &CanvasKit<'_>,
    row: Area,
    number: i32,
    name: &str,
    subtitle: &str,
    preview: Option<&str>,
    active: bool,
) {
    let pad = pt(20.0);
    let x0 = row.x;
    let y0 = row.y;
    let x1 = row.x1();
    kit.rrect_fill(
        Rect::from_xywh(row.x, row.y, row.w, row.h),
        row.h * 0.10,
        PICK_PLATE,
    );

    if active {
        kit.rrect_fill(
            Rect::from_xywh(x0, row.y + pad, pt(6.0), row.h - 2.0 * pad),
            pt(3.0),
            PICK_TEXT_MAIN,
        );
    }

    let side = row.h - 2.0 * pad;
    theme_row_no(kit, x0 + pad, y0 + pad, side, number);

    let mut text_x = x0 + pad + side + pt(24.0);
    if let Some(key) = preview.filter(|key| !key.is_empty()) {
        let area = Rect::from_xywh(text_x, y0 + pad, side * THEME_PREVIEW_RATIO, side);
        sketch::paste_cover_blade(
            kit,
            key,
            area,
            sketch::BladeCorners::even(pt(6.0)),
            PICK_CHIP,
        );
        text_x = area.right() + pt(24.0);
    }

    let mut right = x1 - pad;
    if active {
        right -= theme_current_tag(kit, right, y0 + (row.h) / 2.0);
    }

    let name_style = wall_text(34.0, PICK_TEXT_MAIN);
    let sub_style = wall_text(23.0 * 0.93, PICK_TEXT_DIM);
    let max_width = (right - text_x).max(0.0);
    let name = sketch::ellipsize(kit, &name_style, name, max_width);
    sketch::text(
        kit,
        &name_style,
        &name,
        text_x,
        y0 + row.h * 0.44,
        PICK_TEXT_MAIN,
    );
    let subtitle = sketch::ellipsize(kit, &sub_style, subtitle, max_width);
    sketch::text(
        kit,
        &sub_style,
        &subtitle,
        text_x,
        y0 + row.h * 0.76,
        PICK_TEXT_DIM,
    );
}

pub fn paint_theme(kit: &CanvasKit<'_>, input: &ThemeInput) {
    let (width, height) = theme_size(input);
    let width = width as f32;
    let height = height as f32;
    pick_ground(kit, width, height);
    pick_head(kit, width, &input.title, "", &input.subtitle);
    for (index, entry) in input.themes.iter().enumerate() {
        theme_row(
            kit,
            theme_row_rect(index),
            index as i32,
            &entry.name,
            &entry.label,
            entry.preview.as_deref(),
            entry.active,
        );
    }
    pick_footer(kit, width, height, &input.note);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::harness::{self, Harness, Pixels};

    fn case_168(actual: [u8; 4], expect: [u8; 3]) -> bool {
        actual[..3]
            .iter()
            .zip(expect)
            .all(|(a, e)| (*a as i32 - i32::from(e)).abs() <= 28)
    }

    fn case_169(input: &MeInput) -> Pixels {
        let app = Harness::new();
        let bytes = app.render(request(input), |kit| paint(kit, input));
        harness::decode(&bytes).expect("档案图必须可解码")
    }

    fn case_170(input: &CharInput) -> Pixels {
        let app = Harness::new();
        let bytes = app.render(char_request(input), |kit| paint_char(kit, input));
        harness::decode(&bytes).expect("立绘图必须可解码")
    }

    fn case_171(input: &BannerInput) -> Pixels {
        let app = Harness::new();
        let bytes = app.render(banner_request(input), |kit| paint_banner(kit, input));
        harness::decode(&bytes).expect("横幅图必须可解码")
    }

    fn case_172(input: &ThemeInput) -> Pixels {
        let app = Harness::new();
        let bytes = app.render(theme_request(input), |kit| paint_theme(kit, input));
        harness::decode(&bytes).expect("主题图必须可解码")
    }

    fn case_173(pixels: &Pixels, x: u32, y: u32, w: u32, h: u32) -> f64 {
        pixels.ink_ratio(x, y, w, h, 1)
    }

    fn case_174() -> MeInput {
        let curve = MeCurve {
            points: (0..40)
                .map(|index| MeCurvePoint {
                    at: 1_700_000_000 + index * 86_400,
                    ptt: 12.0 + index as f64 * 0.01,
                })
                .collect(),
            peak: 12.39,
            peak_at: Some(1_700_000_000 + 39 * 86_400),
            recent7: Some(0.07),
            recent30: Some(0.21),
            records30: Some(12),
            first: Some(1_700_000_000),
            last: Some(1_700_000_000 + 39 * 86_400),
        };
        let kinds = [
            Difficulty::Beyond,
            Difficulty::Eternal,
            Difficulty::Future,
            Difficulty::Present,
            Difficulty::Past,
        ];
        let bands: Vec<MeBand> = kinds
            .iter()
            .enumerate()
            .map(|(index, kind)| {
                let mut band = MeBand::new(*kind);
                band.played = 40 - index as i32 * 4;
                band.total = 48;
                band.cleared = 30 - index as i32 * 3;
                band.full_combo = 18 - index as i32 * 2;
                band.pm = 8 - index as i32;
                band.ex_plus = 14 - index as i32;
                band.ex = 22 - index as i32;
                band.loss_played = 1234.567 + index as f64 * 1000.0;
                band.loss_total = 2345.678 + index as f64 * 2000.0;
                band
            })
            .collect();
        let scatter = MeScatter {
            bins: (0..40)
                .map(|index| MeBin {
                    constant: 8.0 + index as f64 * 0.1,
                    low: 9.0 + index as f64 * 0.05,
                    high: 9.5 + index as f64 * 0.08,
                    avg: 9.2 + index as f64 * 0.06,
                    count: 3,
                })
                .collect(),
            max_ptt: 12.6,
            avg_ptt: Some(10.9),
            ability: Some(11.4),
        };
        let board = MeBoard {
            total: 2048,
            rows: (1..=16)
                .map(|rank| MeBoardRow {
                    rank,
                    name: format!("玩家{rank}"),
                    avatar: None,
                    ptt: Some(13.0 - rank as f64 * 0.05),
                    rating_frame: Some(6),
                    rating_accent: Some(4),
                    me: rank == 14,
                })
                .collect(),
        };
        let changes: Vec<MeChange> = (0..6)
            .map(|index| MeChange {
                title: format!("曲目 {index}"),
                diff: String::from("FTR"),
                old_score: if index == 0 { None } else { Some(9_800_000) },
                new_score: 9_900_000 + index,
                at: Some(1_700_000_000 + index * 3600),
                jacket: None,
            })
            .collect();
        MeInput::new(PlayerBadge::new("Afkeru").with_ptt(12.41).with_star(6))
            .with_curve(curve)
            .with_bands(bands)
            .with_scatter(scatter)
            .with_board(board)
            .with_changes(changes)
            .with_rating(6, 4)
    }

    fn case_175() -> (Vec<ScatterPoint>, f64, f64, f64, f64) {
        let input = case_174();
        let scatter = input.scatter_ref().expect("夹具要有散点");
        let all = scatter_points(scatter);
        let (start, end) = scatter_domain(&all, scatter.ability);
        let points = scatter_in_range(&all, start);
        let filled = scatter_filled(&points);
        let (lo, hi) = scatter_window(&filled);
        (filled, start, end, lo, hi)
    }

    #[test]
    fn case_176() {
        let 空桶 = ScatterPoint {
            constant: 6.0,
            low: 0.0,
            best: 0.0,
            avg: 0.0,
            count: 0,
        };
        let 低桶 = ScatterPoint {
            constant: 8.0,
            low: 8.4,
            best: 9.1,
            avg: 8.7,
            count: 5,
        };
        let 高桶 = ScatterPoint {
            constant: 11.9,
            low: 11.2,
            best: 12.6,
            avg: 11.9,
            count: 5,
        };
        let points = vec![空桶, 低桶, 高桶];
        let filled = scatter_filled(&points);
        assert_eq!(filled.len(), 2, "没打过的桶不进纵轴窗口");
        let (lo, hi) = scatter_window(&filled);
        assert!(lo >= 8.0, "不再从 0.00 起步，实际 {lo}");
        assert!(hi > 12.6, "最高点之上要留余量，实际 {hi}");
        assert!(hi - lo >= SCATTER_SPAN_MIN - 1e-9, "跨度保底 {SCATTER_SPAN_MIN}");
        assert_eq!(lo * 2.0, (lo * 2.0).round(), "下限落在 0.5 的格子上");
        assert_eq!(hi * 2.0, (hi * 2.0).round(), "上限落在 0.5 的格子上");
    }

    #[test]
    fn case_177() {
        let (filled, start, end, lo, hi) = case_175();
        let pixels = case_169(&case_174());
        let point = filled.last().expect("至少要有一个桶");
        let x = 722.0 + (1294.0 - 722.0) * ((point.constant - start) / (end - start)) as f32;
        let y_of = |value: f64| 375.0 - 200.0 * ((value - lo) / (hi - lo)) as f32;
        let mid = (y_of(point.best) + y_of(point.low)) / 2.0;
        let center = pixels.at(px(x) as u32, px(mid) as u32);
        assert!(center[0] < 240, "发挥区间胶囊要盖在纸面上，实际 {center:?}");
        let mut wide = 0;
        for dx in -14..=14 {
            let sample = pixels.at((px(x) as i32 + dx) as u32, px(mid) as u32);
            if sample[0] < 240 {
                wide += 1;
            }
        }
        assert!((8..=18).contains(&wide), "胶囊宽约 12 设备像素，实际 {wide}");
        let best = pixels.at(px(x) as u32, px(y_of(point.best)) as u32);
        assert!(best[0] < 120, "最高点是一枚深色菱形，实际 {best:?}");
    }

    #[test]
    fn case_178() {
        let (_, start, end, lo, hi) = case_175();
        let pixels = case_169(&case_174());
        let 虚线 = |x: u32, y: u32| {
            let sample = pixels.at(x, y);
            (sample[0] as i32 - 0xB5).abs() <= 14
                && (sample[1] as i32 - 0xAD).abs() <= 14
                && (sample[2] as i32 - 0x9D).abs() <= 14
        };
        let y_of = |value: f64| 375.0 - 200.0 * ((value - lo) / (hi - lo)) as f32;
        let y = px(y_of(10.9)) as u32;
        let mut across = 0;
        for x in px(722.0) as u32..px(1294.0) as u32 {
            if (y - 3..=y + 3).any(|row| 虚线(x, row)) {
                across += 1;
            }
        }
        assert!(across >= 120, "均值横虚线要横穿绘图区，实际 {across} 列");
        let ability = 11.4;
        let x = px(722.0 + (1294.0 - 722.0) * ((ability - start) / (end - start)) as f32) as u32;
        let mut down = 0;
        for row in px(175.0) as u32..px(375.0) as u32 {
            if (x - 3..=x + 3).any(|col| 虚线(col, row)) {
                down += 1;
            }
        }
        assert!(down >= 60, "能力定数是一条竖虚线，实际 {down} 行");
    }

    #[test]
    fn case_179() {
        assert!(scatter_tag_clear(&[], 0.0, 0.0, 10.0), "附近没点就不用让");
        let anchors = vec![(100.0_f32, 50.0_f32)];
        assert!(
            !scatter_tag_clear(&anchors, 90.0, 40.0, 30.0),
            "标签框压到点时要换边"
        );
        assert!(
            scatter_tag_clear(&anchors, 90.0, 80.0, 30.0),
            "错开纵向就还是安全的"
        );
    }

    #[test]
    fn case_180() {
        let input = case_174();
        assert_eq!(size_of(&input), (2800, 1820));
        let request = request(&input);
        assert_eq!((request.width, request.height), (2800, 1820));
    }

    #[test]
    fn case_181() {
        let pixels = case_169(&case_174());
        let paper = pixels.at(4, 4);
        assert!(case_168(paper, [0xF6, 0xF3, 0xED]), "底纸是米白宣纸：{paper:?}");
        let rule = pixels.at(200, 212);
        assert!(case_168(rule, [0x25, 0x24, 0x21]), "刊头粗线在 105×2 处：{rule:?}");
        let hairline = pixels.at(200, 221);
        assert!(case_168(hairline, [0xC7, 0xC0, 0xB2]), "细线在粗线下方 5px 处：{hairline:?}");
        assert!(case_173(&pixels, 100, 100, 400, 60) > 0.01, "标题必须画出来");
    }

    #[test]
    fn case_182() {
        let pixels = case_169(&case_174());
        assert!(
            case_173(&pixels, 120, 340, 1100, 250) > 0.02,
            "曲线绘图区必须有墨"
        );
        assert!(
            case_173(&pixels, 1150, 330, 180, 260) > 0.01,
            "纵轴刻度右对齐在 650×2 那一列"
        );
    }

    #[test]
    fn case_183() {
        let pixels = case_169(&case_174());
        let byd = diff_ink("BYD");
        let chip = pixels.at(170, 845);
        assert!(
            case_168(chip, [byd.r, byd.g, byd.b]),
            "第一行是 BYD 的块（难度从高到低）：{chip:?}"
        );
        let ftr = diff_ink("FTR");
        let last = pixels.at(170, 1020);
        assert!(
            case_168(last, [ftr.r, ftr.g, ftr.b]),
            "最后一行是 FTR 的块（PST/PRS 不上卡）：{last:?}"
        );
        let bar = pixels.at(260, 860);
        assert_ne!(bar[..3], [0xE6, 0xE1, 0xD6], "分段条盖住底槽");
        assert!(
            case_173(&pixels, 100, 820, 1200, 400) > 0.05,
            "三行内容都要有墨"
        );
    }

    #[test]
    fn case_184() {
        let pixels = case_169(&case_174());
        let frame = pixels.at(120, 1372);
        assert!(case_168(frame, [0xDB, 0xD7, 0xCD]), "微卡外框在 60×2 / 686×2：{frame:?}");
        let second = pixels.at(120 + 600, 1372);
        assert!(case_168(second, [0xE2, 0xDE, 0xD7]), "第二列微卡隔 300×2：{second:?}");
    }

    #[test]
    fn case_185() {
        let pixels = case_169(&case_174());
        assert!(
            case_173(&pixels, 1432, 350, 1100, 480) > 0.02,
            "散布图必须有东西"
        );
    }

    #[test]
    fn case_186() {
        let mut rows: Vec<MeBoardRow> = (1..=16)
            .map(|rank| MeBoardRow {
                rank,
                name: format!("玩家{rank}"),
                me: rank == 3,
                ..MeBoardRow::default()
            })
            .collect();
        rows.insert(
            3,
            MeBoardRow {
                rank: 3,
                name: String::from("玩家3"),
                me: true,
                ..MeBoardRow::default()
            },
        );
        let board = MeBoard { total: 100, rows };
        assert!(board_list_layout(&board));
        let cells = board_cells(&board);
        assert_eq!(cells.len(), 15, "名录版式铺满 3×5");
        assert!(
            cells.iter().filter(|row| row.me).count() == 1,
            "同一个人只留一格"
        );
        assert!(cells.iter().all(|row| row.rank > 0), "不留空位");
    }

    #[test]
    fn case_187() {
        let rows: Vec<MeBoardRow> = (1..=20)
            .map(|rank| MeBoardRow {
                rank,
                name: format!("玩家{rank}"),
                me: rank == 17,
                ptt: None,
                ..MeBoardRow::default()
            })
            .collect();
        let board = MeBoard { total: 2048, rows };
        assert!(!board_list_layout(&board));
        let cells = board_cells(&board);
        assert_eq!(cells.len(), 14, "对比版式 12 + 2");
        assert_eq!(cells[11].rank, 12, "前十二名一摞");
        assert_eq!(cells[12].rank, 16, "第十三格是我上一位");
        assert_eq!(cells[13].rank, 17, "第十四格是我");
        assert!(gap_to_prev(&board).is_none(), "没有潜力值就没有差值");
    }

    #[test]
    fn case_188() {
        let wall = PICK_SQUARE;
        assert_eq!(
            wall.canvas(8),
            (2016, pti((236.0f32 + 155.75 + 120.0).round()))
        );
        let (cw, ch) = wall.cell_size(8);
        assert_eq!((cw, ch), (218, 218), "方图格子是正方形");
        assert_eq!(wall.cell_rect(0, 8), (67, 330, 218, 218));
        assert_eq!(
            wall.cell_rect(1, 8).0,
            67 + 218 + 20,
            "格子间距 14×1.4 取整"
        );
    }

    #[test]
    fn case_189() {
        let wall = PICK_WIDE;
        let (cw, ch) = wall.cell_size(9);
        assert_eq!(cw, 615);
        assert_eq!(ch, pti((439.0f32 / 4.2).round()));
        assert_eq!(wall.canvas(9).0, 2016);
        let rect = wall.cell_rect(8, 9);
        assert_eq!(rect.2, cw);
        assert!(rect.1 > 330, "最后一行在网格下方");
    }

    #[test]
    fn case_190() {
        let (width, height) = theme_size(&ThemeInput::default());
        assert_eq!(width, 2016);
        assert_eq!(height, pti(236.0) + pti(152.0) + pti(120.0));
        let row = theme_row_rect(1);
        assert_eq!(row.y, pt(236.0) + pt(152.0) + pt(18.0));
        assert_eq!(row.h, pt(152.0));
        assert_eq!(row.w, pt(1344.0));
    }

    #[test]
    fn case_191() {
        let 立绘 = case_170(&CharInput::new());
        assert!(!立绘.blank() || 立绘.width == 2016);
        let 横幅 = case_171(&BannerInput::new());
        assert!(!横幅.blank() || 横幅.width == 2016);
        let 主题 = case_172(&ThemeInput::new());
        assert_eq!(主题.width, 2016);
        let me_picture = case_169(&MeInput::new(PlayerBadge::new("")));
        assert_eq!((me_picture.width, me_picture.height), (2800, 1820));
    }

    #[test]
    fn case_192() {
        let cells: Vec<CharCell> = (0..9)
            .map(|index| CharCell {
                name: "超长名字".repeat(12),
                label: String::new(),
                thumb: format!("不存在/缺失_{index}.png"),
                art: String::new(),
                unlocked: false,
            })
            .collect();
        let input = CharInput::new()
            .with_title("角色立绘一览".repeat(8))
            .with_cells(cells);
        let pixels = case_170(&input);
        assert_eq!(pixels.width, 2016);

        let banner = BannerInput::new()
            .with_cells(vec![BannerCell::new("缺失/横幅.png")])
            .with_note("页脚小注");
        let pixels = case_171(&banner);
        assert_eq!(pixels.width, 2016);

        let theme = ThemeInput::new()
            .with_entries(vec![
                ThemeEntry::new("default", "默认初始主题").with_active(true),
                ThemeEntry::new("arknights", "明日方舟")
                    .with_preview("themes/arknights/background/score.png"),
            ])
            .with_note("页脚");
        let pixels = case_172(&theme);
        assert_eq!(pixels.width, 2016);
    }

    #[test]
    fn case_193() {
        let input = MeInput::new(PlayerBadge::new("空档案"));
        let pixels = case_169(&input);
        assert_eq!((pixels.width, pixels.height), (2800, 1820));
        assert!(
            case_173(&pixels, 120, 140, 1200, 1200) > 0.0,
            "五张卡的空态都要有字"
        );
    }

    #[test]
    fn case_194() {
        let 名录: Vec<MeBoardRow> = (1..=5)
            .map(|rank| MeBoardRow {
                rank,
                name: format!("玩家{rank}"),
                me: rank == 3,
                ..MeBoardRow::default()
            })
            .collect();
        let pixels = case_169(&case_174().with_board(MeBoard {
            total: 5,
            rows: 名录,
        }));
        assert!(
            case_173(&pixels, 1420, 1040, 1260, 700) > 0.01,
            "名录必须画在榜面里"
        );
    }

    #[test]
    fn case_195() {
        let mut ftr = MeBand::new(Difficulty::Future);
        ftr.played = 10;
        ftr.total = 100;
        ftr.cleared = 10;
        ftr.full_combo = 6;
        ftr.pm = 3;
        ftr.ex = 9;
        ftr.ex_plus = 4;
        ftr.loss_played = 1234.567;
        ftr.loss_total = 2345.678;
        assert_eq!(band_loss_text(&ftr), "1234.567 / 2345.678");
        assert_eq!(band_loss_text(&MeBand::new(Difficulty::Past)), "NONE / NONE");
        assert_eq!(
            band_loss_text(&MeBand::new(Difficulty::Present)),
            "NONE / NONE"
        );
        assert_eq!(
            band_main_loss(&[&ftr, &MeBand::new(Difficulty::Past)]),
            (1234.567, 2345.678)
        );

        let ins = MeBand::new(Difficulty::Ins);
        assert_eq!(ins.name, "INS", "Inscribed 要显示成 INS");
        assert!(ins.counts_loss(), "INS 属于主难, 要计入表头失分");
        assert!(MeBand::new(Difficulty::Beyond).counts_loss());
        assert!(MeBand::new(Difficulty::Eternal).counts_loss());

        let input = case_174().with_bands(vec![ftr.clone(), MeBand::new(Difficulty::Present)]);
        let note = bands_note(&input);
        assert!(note.contains("PM中小P总量"), "表头要写 PM 中小P总量: {note}");
        assert!(
            note.contains("主难已玩总LS 1234.567"),
            "表头要写主难已玩失分: {note}"
        );
        assert!(
            note.contains("主难全曲总LS 2345.678"),
            "表头要写主难全曲失分: {note}"
        );

        let pixels = case_169(&input);
        let groove = [0xE6u8, 0xE1, 0xD6];
        let bar_x = |x: f32| (x * ME_K) as u32;
        let mid = ((414.0 + 12.0 + 4.0) * ME_K) as u32;
        assert_ne!(
            pixels.at(bar_x(124.0), mid)[..3],
            groove[..],
            "分母是曲库总数, 已玩的那一截仍要画出来"
        );
        for x in bar_x(130.0)..bar_x(205.0) {
            assert_eq!(
                pixels.at(x, mid)[..3],
                groove[..],
                "分母是曲库总数: 100 张里只玩了 10 张, 其余是底槽"
            );
        }
        let fresh = case_169(&case_174().with_bands(vec![MeBand::new(Difficulty::Future)]));
        for x in bar_x(121.0)..bar_x(205.0) {
            assert_eq!(
                fresh.at(x, mid)[..3],
                groove[..],
                "一条都没记录时进度条应当是空的"
            );
        }
    }

    #[test]
    fn case_196() {
        let change = MeChange {
            title: String::from("Testify"),
            diff: String::from("FTR"),
            old_score: Some(9_800_000),
            new_score: 9_850_000,
            at: Some(1_762_592_000),
            jacket: None,
        };
        let note = change_note(&change);
        assert_eq!(
            note,
            format!(
                "from {}  ·  {}",
                sketch::thousands(9_800_000),
                fmt_datetime(1_762_592_000)
            )
        );
        assert!(
            note.contains(':'),
            "最近变化要精确到几点几分: {note}"
        );
        let fresh = MeChange {
            old_score: None,
            ..change.clone()
        };
        assert!(change_note(&fresh).starts_with("NEW RECORD"));
        let undated = MeChange { at: None, ..change };
        assert_eq!(
            change_note(&undated),
            format!("from {}", sketch::thousands(9_800_000))
        );
    }
}
