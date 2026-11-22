use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use aurorix_render::skia_safe::canvas::SrcRectConstraint;
use aurorix_render::skia_safe::{
    surfaces, AlphaType, BlendMode, ClipOp, ColorType, Image, ImageInfo, Paint, PathBuilder, Rect,
};
use aurorix_render::{sampling_for, CanvasKit, Color, RenderRequest, TextStyle};
use chrono::{DateTime, FixedOffset, Utc};
use super::avatar;

use super::memo::{self, Memo};
use super::theme::{self, Theme, ThemePage};
use super::sketch;
use super::widgets::PlayerBadge;
use crate::domain::{clear_asset, format_rating, grade_asset};
use crate::model::{Difficulty, Grade};

pub const CARD_WIDTH: i32 = 2016;

pub const CARD_HEIGHT: i32 = 1333;

pub const FALLBACK_HEIGHT: i32 = 1333;

pub const SCORE_FILE: &str = "score.png";

pub const RECENT_FILE: &str = "recent.png";

const DESIGN_W: f64 = 1280.0;

const DESIGN_H: f64 = 720.0;

const VIEW_H: f64 = 888.8889;

const BASE_W: f64 = 1440.0;

const RS: f64 = 1.4;

const UNIT: f64 = BASE_W / DESIGN_W * RS;

const MID_DY: f64 = -34.0;

const VIEW_MID: f64 = (VIEW_H + DESIGN_H) / 2.0;

const TOP_BAR_H: f64 = 69.0;

const TOP_BAND_H: f64 = 50.0;

const CARD_TOP_ROW: f64 = 143.0;

const CARD_BOT_ROW: f64 = 772.0;

const DARK_BAND_H: f64 = 122.0;

const DARK_BOT_ROW: f64 = CARD_TOP_ROW + DARK_BAND_H;

const TITLE_ROW: f64 = 186.5;

const ARTIST_ROW: f64 = 234.5;

const TITLE_CX: f64 = 640.0;

const EQUAL_GAP: f64 = 0.045 * VIEW_H;

const MAIN_CARD_H: f64 = CARD_BOT_ROW - CARD_TOP_ROW;

const BOTTOM_BAR_H: f64 = 68.3;

const SCORE_VIEW_H: f64 = TOP_BAR_H + EQUAL_GAP + MAIN_CARD_H + EQUAL_GAP + BOTTOM_BAR_H;

const TOP_STRIP_LEFT: f64 = 640.47 - 1561.0 / 2.0;

const PLATE_LEFT: f64 = 280.0;

const PLATE_RIGHT: f64 = 691.0;

const PLATE_PEAK: f64 = 31.0;

const AVATAR_SIZE: f64 = 80.0;

const AVATAR_CX: f64 = PLATE_RIGHT - AVATAR_SIZE / 2.0;

const AVATAR_CY: f64 = 32.0;

const PTT_INT_SIZE: f64 = 14.0;

const PTT_DEC_RATIO: f64 = 35.0 / 40.0;

const WATERMARK_LEFT: f64 = 875.0;

const TOP_TEXT_ROW: f64 = 28.5;

const RATING_KEY_DIR: &str = "avatar/rating";

const GAUGE_DIR: &str = "result/hp";

const GAUGE_SCALE: f64 = 0.77;

const GAUGE_H: f64 = 487.0 * GAUGE_SCALE;

const GAUGE_X: f64 = 30.0 + 375.0;

const GAUGE_TOP_Y: f64 = 452.93;

const GAUGE_RIGHT: f64 = 437.86;

const GAUGE_W: f64 = GAUGE_RIGHT - GAUGE_X;

const GAUGE_BAR_OPACITY: f64 = 0.8;

const JUDGE_ART_SCALE: f64 = 0.667;

const LEVEL_INK_SHIFT: f32 = 3.7;

const LEVEL_INK_RISE: f32 = 2.6;

const JUDGE_INK_ALPHA: u8 = 30;

const CHAR_LAYER_H: f64 = 720.0;

const CHAR_NODE_X: f64 = 750.0;

const CHAR_NODE_POS_Y: f64 = 670.031982421875;

const CHAR_GLOBAL_DX: f64 = -120.0;

const CHAR_ACTION_DX: f64 = -200.0;

const CHAR_SCREEN_SHIFT_Y: f64 = 100.0;

const CHAR_SCREEN_SHIFT_R: f64 = 240.0;

const CHAR_ART_SCALE: f64 = 2.0 / 3.0;

const CHAR_UNKNOWN_CX: f64 = 0.80 * DESIGN_W;

const DEFAULT_CHARACTER: &str = "characters/对立_tairitsu_1u.png";

const BOT_NAME: &str = "Okamikko";

const SCORE_BACKDROP: &str = "background/score.png";

const SCORE_TITLE: &str = "Player Score";

const SCORE_RECENT_TITLE: &str = "Player Recent";

const INK_TOP: Color = Color::new(44, 45, 48, 255);

const WHITE: Color = Color::WHITE;

const JUDGE_INK: Color = Color::new(26, 38, 56, 255);

const SHADOW: Color = Color::new(0, 0, 0, 170);

const POTENTIAL_INK: Color = Color::new(52, 48, 74, 255);

const NAME_PLATE_FALLBACK: Color = Color::new(30, 28, 40, 238);

const NAME_PLATE_LIGHT_INK: Color = Color::new(91, 79, 83, 255);

const PTT_EDGE: Color = Color::BLACK;

const BACKDROP_FALLBACK: Color = Color::new(232, 234, 242, 255);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    Score,
    Recent,
}

impl Face {
    pub fn caption(self) -> &'static str {
        match self {
            Face::Score => "单曲成绩",
            Face::Recent => "最近结算",
        }
    }

    pub fn file(self) -> &'static str {
        match self {
            Face::Score => SCORE_FILE,
            Face::Recent => RECENT_FILE,
        }
    }

    pub fn page_title(self) -> &'static str {
        match self {
            Face::Score => SCORE_TITLE,
            Face::Recent => SCORE_RECENT_TITLE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Judge {
    Pure,
    Far,
    Lost,
}

impl Judge {
    fn label(self) -> &'static str {
        match self {
            Judge::Pure => "PURE",
            Judge::Far => "FAR",
            Judge::Lost => "LOST",
        }
    }

    fn key(self) -> &'static str {
        match self {
            Judge::Pure => "pure",
            Judge::Far => "far",
            Judge::Lost => "lost",
        }
    }

    fn file(self) -> &'static str {
        match self {
            Judge::Pure => "judge/pure.png",
            Judge::Far => "judge/far.png",
            Judge::Lost => "judge/lost.png",
        }
    }
    fn glow(self) -> Color {
        match self {
            Judge::Pure => Color::rgb(180, 71, 159),
            Judge::Far => Color::rgb(226, 170, 130),
            Judge::Lost => Color::rgb(210, 158, 165),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScoreInput {
    pub badge: PlayerBadge,
    pub theme: Theme,
    pub song_id: String,
    pub title: String,
    pub artist: String,
    pub difficulty: Difficulty,
    pub constant: Option<f64>,
    pub level: String,
    pub score: i64,
    pub grade: Grade,
    pub pure: i64,
    pub far: i64,
    pub lost: i64,
    pub shiny_perfect: Option<i64>,
    pub normal_perfect: Option<i64>,
    pub rating: f64,
    pub ptt: Option<f64>,
    pub rating_frame: Option<i32>,
    pub ptt_delta: Option<f64>,
    pub previous_rating: Option<f64>,
    pub clear_type: Option<i32>,
    pub skill: String,
    pub jacket: Option<String>,
    pub play_at: Option<DateTime<Utc>>,
    pub note: String,
    pub recollection: Option<f64>,
}

impl ScoreInput {
    pub fn new(title: impl Into<String>, score: i64) -> Self {
        Self {
            badge: PlayerBadge::default(),
            theme: Theme::DEFAULT,
            song_id: String::new(),
            title: title.into(),
            artist: String::new(),
            difficulty: Difficulty::Future,
            constant: None,
            level: String::new(),
            score,
            grade: crate::domain::grade_of(score),
            pure: 0,
            far: 0,
            lost: 0,
            shiny_perfect: None,
            normal_perfect: None,
            rating: 0.0,
            ptt: None,
            ptt_delta: None,
            rating_frame: None,
            previous_rating: None,
            clear_type: None,
            skill: String::new(),
            jacket: None,
            play_at: None,
            note: String::new(),
            recollection: None,
        }
    }

    pub fn with_badge(mut self, badge: PlayerBadge) -> Self {
        self.badge = badge;
        self
    }

    pub fn with_rating_frame(mut self, frame: Option<i32>) -> Self {
        self.rating_frame = frame.filter(|value| *value >= 0);
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_song(mut self, song_id: impl Into<String>) -> Self {
        self.song_id = song_id.into();
        self
    }

    pub fn with_artist(mut self, artist: impl Into<String>) -> Self {
        self.artist = artist.into();
        self
    }

    pub fn with_difficulty(mut self, difficulty: Difficulty, constant: f64) -> Self {
        self.difficulty = difficulty;
        if constant > 0.0 {
            self.constant = Some(constant);
        }
        self
    }

    pub fn with_level(mut self, level: impl Into<String>) -> Self {
        self.level = level.into();
        self
    }

    pub fn with_judgement(mut self, pure: i64, far: i64, lost: i64) -> Self {
        self.pure = pure;
        self.far = far;
        self.lost = lost;
        self
    }

    pub fn with_perfect(mut self, shiny: i64, normal: i64) -> Self {
        self.shiny_perfect = Some(shiny);
        self.normal_perfect = Some(normal);
        self
    }

    pub fn with_grade(mut self, grade: Grade) -> Self {
        self.grade = grade;
        self
    }

    pub fn with_rating(mut self, rating: f64) -> Self {
        self.rating = rating;
        self
    }

    pub fn with_ptt(mut self, ptt: f64) -> Self {
        if ptt > 0.0 {
            self.ptt = Some(ptt);
        }
        self
    }

    pub fn with_delta(mut self, delta: f64, previous: f64) -> Self {
        self.ptt_delta = Some(delta);
        if previous > 0.0 {
            self.previous_rating = Some(previous);
        }
        self
    }

    pub fn with_clear(mut self, clear_type: i32) -> Self {
        self.clear_type = Some(clear_type);
        self
    }

    pub fn with_skill(mut self, skill: impl Into<String>) -> Self {
        self.skill = skill.into();
        self
    }

    pub fn with_jacket(mut self, key: impl Into<String>) -> Self {
        let key = key.into();
        if !key.is_empty() {
            self.jacket = Some(key);
        }
        self
    }

    pub fn with_play_at(mut self, play_at: DateTime<Utc>) -> Self {
        self.play_at = Some(play_at);
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = note.into();
        self
    }

    pub fn with_recollection(mut self, percent: f64) -> Self {
        self.recollection = Some(percent.clamp(0.0, 200.0));
        self
    }

    pub fn has_jacket(&self) -> bool {
        self.jacket.is_some()
    }

    pub fn has_clear(&self) -> bool {
        self.clear_type.is_some()
    }

    pub fn clear_type(&self) -> i32 {
        self.clear_type.unwrap_or(0)
    }

    pub fn cleared(&self) -> bool {
        self.has_clear() && self.clear_type() != 0
    }

    pub fn has_ptt(&self) -> bool {
        self.ptt.or(self.badge.ptt).is_some()
    }

    pub fn ptt_or_badge(&self) -> Option<f64> {
        self.ptt.or(self.badge.ptt)
    }

    pub fn char_id(&self) -> Option<i32> {
        self.badge.character.as_deref().and_then(char_id_from_key)
    }

    pub fn overflow(&self) -> bool {
        if matches!(self.clear_type, Some(2) | Some(6)) {
            return true;
        }
        matches!(self.recollection, Some(value) if value > 100.0)
    }

    pub fn recall_ratio(&self) -> f64 {
        match self.recollection {
            Some(value) => (value / 100.0).clamp(0.0, 1.0),
            None if self.cleared() => 1.0,
            None => 0.0,
        }
    }

    pub fn height(&self) -> i32 {
        CARD_HEIGHT
    }
}

fn dr(v: f64) -> f32 {
    (v * UNIT).round() as f32
}

fn dqy(y: f64) -> f32 {
    dr(VIEW_MID - y + MID_DY)
}

fn dvy(v: f64) -> f32 {
    dr(v + MID_DY)
}

fn ltrb(left: f32, top: f32, right: f32, bottom: f32) -> Rect {
    Rect::from_ltrb(left, top, right, bottom)
}

fn crect(x: f64, y_top: f64, w: f64, h: f64) -> Rect {
    ltrb(dr(x), dqy(y_top), dr(x + w), dqy(y_top - h))
}

fn srect(x0: f64, y0: f64, x1: f64, y1: f64) -> Rect {
    ltrb(dr(x0), dr(y0), dr(x1), dr(y1))
}

fn ccenter(cx: f64, cy: f64, w: f64, h: f64) -> Rect {
    crect(cx - w / 2.0, cy + h / 2.0, w, h)
}

fn fs(size: f64) -> u32 {
    (size * UNIT).round().max(1.0) as u32
}

fn style(family: &str, size: f64) -> TextStyle {
    TextStyle::new(family, fs(size))
}

fn font_text(size: f64) -> TextStyle {
    style(theme::FONT_NARROW, size)
}

fn font_display(size: f64) -> TextStyle {
    style(theme::family(theme::FONT_DISPLAY), size)
}

fn font_number(size: f64) -> TextStyle {
    style(theme::FONT_TITLE, size)
}

fn font_latin(size: f64) -> TextStyle {
    style(theme::FONT_LATIN, size)
}

fn font_score(size: f64) -> TextStyle {
    style(theme::FONT_SOFT, size)
}

fn cap_height(kit: &CanvasKit<'_>, style: &TextStyle) -> f32 {
    kit.text_layout(style)
        .ok()
        .map(|layout| {
            let (_, metrics) = layout.primary_font().metrics();
            metrics.cap_height
        })
        .filter(|value| *value > 0.0)
        .unwrap_or(style.size as f32 * 0.72)
}

fn ascent_half(kit: &CanvasKit<'_>, style: &TextStyle) -> f32 {
    kit.text_layout(style)
        .ok()
        .map(|layout| {
            let (_, metrics) = layout.primary_font().metrics();
            (-metrics.ascent).ceil() / 2.0
        })
        .unwrap_or(style.size as f32 * 0.4)
}

fn ink_box_of(kit: &CanvasKit<'_>, style: &TextStyle, text: &str) -> Option<Rect> {
    let layout = kit.text_layout(style).ok()?;
    if !layout.primary_covers(text) {
        return None;
    }
    let (_, ink) = layout.primary_font().measure_text(text, None);
    if ink.width() <= 0.0 || ink.height() <= 0.0 {
        return None;
    }
    Some(ink)
}

fn ink_center_y(kit: &CanvasKit<'_>, style: &TextStyle, text: &str) -> f32 {
    match ink_box_of(kit, style, text) {
        Some(ink) => -(ink.top() + ink.bottom()) / 2.0,
        None => cap_height(kit, style) / 2.0,
    }
}

fn ink_center_x(kit: &CanvasKit<'_>, style: &TextStyle, text: &str) -> f32 {
    match ink_box_of(kit, style, text) {
        Some(ink) => (ink.left() + ink.right()) / 2.0,
        None => width(kit, style, text) / 2.0,
    }
}

fn ink_edge(kit: &CanvasKit<'_>, style: &TextStyle, text: &str, left: bool) -> f32 {
    sketch::ink_box(kit, style, text)
        .map(|ink| if left { ink.left() } else { ink.right() })
        .unwrap_or(0.0)
}

fn width(kit: &CanvasKit<'_>, style: &TextStyle, text: &str) -> f32 {
    kit.measure(text, style).map(|m| m.width).unwrap_or(0.0)
}

fn draw(kit: &CanvasKit<'_>, style: &TextStyle, text: &str, x: f32, base: f32, color: Color) {
    sketch::text(kit, style, text, x, base, color);
}

fn put_left(kit: &CanvasKit<'_>, style: &TextStyle, text: &str, x: f32, row: f32, color: Color) {
    if text.is_empty() {
        return;
    }
    let base = row + ink_center_y(kit, style, text);
    draw(kit, style, text, x + dr(2.0), base + dr(2.0), SHADOW);
    draw(kit, style, text, x, base, color);
}

fn put_left_flat(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    text: &str,
    x: f32,
    row: f32,
    color: Color,
) {
    if text.is_empty() {
        return;
    }
    draw(kit, style, text, x, row + ink_center_y(kit, style, text), color);
}

fn put_center(kit: &CanvasKit<'_>, style: &TextStyle, text: &str, cx: f32, row: f32, color: Color) {
    let x = cx - width(kit, style, text) / 2.0;
    put_left(kit, style, text, x, row, color);
}

fn put_center_flat(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    text: &str,
    cx: f32,
    row: f32,
    color: Color,
) {
    let x = cx - width(kit, style, text) / 2.0;
    put_left_flat(kit, style, text, x, row, color);
}

fn put_center_base(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    text: &str,
    cx: f32,
    base: f32,
    color: Color,
) {
    let x = cx - width(kit, style, text) / 2.0;
    draw(kit, style, text, x + dr(2.0), base + dr(2.0), SHADOW);
    draw(kit, style, text, x, base, color);
}

fn mix_toward_white(color: Color, t: f32) -> Color {
    let mix = |v: u8| (v as f32 + (255.0 - v as f32) * t + 0.5) as u8;
    Color::new(mix(color.r), mix(color.g), mix(color.b), color.a)
}

const RING8: [(f32, f32); 8] = [
    (-1.0, 0.0),
    (1.0, 0.0),
    (0.0, -1.0),
    (0.0, 1.0),
    (-1.0, -1.0),
    (1.0, -1.0),
    (-1.0, 1.0),
    (1.0, 1.0),
];

struct GlowInk {
    fill: Color,
    glow: Color,
    halo: f32,
}

fn halo_offsets(radius: f32) -> Vec<(f32, f32)> {
    let reach = radius + 0.5;
    let limit = reach * reach;
    let span = reach.floor() as i32;
    let mut out = Vec::new();
    for dy in -span..=span {
        for dx in -span..=span {
            if ((dx * dx + dy * dy) as f32) <= limit {
                out.push((dx as f32, dy as f32));
            }
        }
    }
    out
}

fn glow_area(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    text: &str,
    x: f32,
    base: f32,
    halo: f32,
) -> Rect {
    let ink = ink_box_of(kit, style, text).unwrap_or_else(|| {
        let cap = cap_height(kit, style).max(1.0);
        Rect::from_xywh(0.0, -cap, width(kit, style, text).max(1.0), cap)
    });
    let pad = halo * 3.0 + 2.0;
    ltrb(
        (x + ink.left() - pad).floor(),
        (base + ink.top() - pad).floor(),
        (x + ink.right() + pad).ceil(),
        (base + ink.bottom() + pad).ceil(),
    )
}

fn text_glow(kit: &CanvasKit<'_>, style: &TextStyle, text: &str, x: f32, row: f32, ink: GlowInk) {
    let GlowInk { fill, glow, halo } = ink;
    if text.is_empty() {
        return;
    }
    let base = row + ink_center_y(kit, style, text);
    let halo = halo.max(1.0);
    let mut halo_color = mix_toward_white(glow, 0.15);
    halo_color.a = 120;
    let area = glow_area(kit, style, text, x, base, halo);
    kit.blur_layer_in(area, halo * 0.5, |inner: &CanvasKit<'_>| {
        draw(inner, style, text, x, base, halo_color);
    });
    let mut thin = JUDGE_INK;
    thin.a = 110;
    for (dx, dy) in halo_offsets(1.0) {
        draw(kit, style, text, x + dx, base + dy, thin);
    }
    for (dx, dy) in halo_offsets(2.0) {
        draw(kit, style, text, x + dx, base + dy, JUDGE_INK);
    }
    draw(kit, style, text, x, base, fill);
    draw(kit, style, text, x + 1.0, base, fill);
    draw(kit, style, text, x, base + 1.0, fill);
}

fn put_center_glow(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    text: &str,
    cx: f32,
    row: f32,
    ink: GlowInk,
) {
    let x = cx - width(kit, style, text) / 2.0;
    text_glow(kit, style, text, x, row, ink);
}

fn put_right_glow(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    text: &str,
    right: f32,
    row: f32,
    ink: GlowInk,
) {
    let x = right - width(kit, style, text);
    text_glow(kit, style, text, x, row, ink);
}

fn put_left_glow(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    text: &str,
    x: f32,
    row: f32,
    ink: GlowInk,
) {
    text_glow(kit, style, text, x, row, ink);
}

fn blit(kit: &CanvasKit<'_>, image: &Image, src: Option<Rect>, dst: Rect, alpha: u8) {
    if dst.width() <= 0.0 || dst.height() <= 0.0 {
        return;
    }
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_dither(true);
    if alpha < 255 {
        paint.set_alpha(alpha);
    }
    let source: Option<(&Rect, SrcRectConstraint)> =
        src.as_ref().map(|rect| (rect, SrcRectConstraint::Fast));
    let target = Rect::from_xywh(0.0, 0.0, dst.width(), dst.height());
    let source_size = src
        .as_ref()
        .map_or((image.width() as f32, image.height() as f32), |r| {
            (r.width(), r.height())
        });
    kit.canvas().draw_image_rect_with_sampling_options(
        image,
        source,
        dst,
        sampling_for(source_size, target),
        &paint,
    );
}

fn image_of(kit: &CanvasKit<'_>, key: &str) -> Option<Image> {
    kit.asset(key)
        .ok()
        .and_then(|texture| texture.image().cloned())
}

fn image_size(image: &Image) -> (f64, f64) {
    (image.width() as f64, image.height() as f64)
}

fn themed_key(theme: Theme, rel: &str) -> String {
    theme.asset(ThemePage::Score, &format!("result/{rel}"))
}

fn themed_image(kit: &CanvasKit<'_>, theme: Theme, rel: &str) -> Option<Image> {
    image_of(kit, &themed_key(theme, rel)).or_else(|| image_of(kit, &format!("result/{rel}")))
}

fn backdrop_key(theme: Theme) -> String {
    match theme.is_official() {
        true => String::from(SCORE_BACKDROP),
        false => String::from(theme.backdrop),
    }
}

fn backdrop_image(kit: &CanvasKit<'_>, theme: Theme) -> Option<Image> {
    image_of(kit, &backdrop_key(theme)).or_else(|| image_of(kit, SCORE_BACKDROP))
}

fn paste_feathered(
    kit: &CanvasKit<'_>,
    image: &Image,
    src: Rect,
    dst: Rect,
    top_fade: f32,
    bot_fade: f32,
) {
    let height = dst.height();
    if height <= 0.0 || dst.width() <= 0.0 {
        return;
    }
    let ft = top_fade.max(0.0).round();
    let fb = bot_fade.max(0.0).round();
    if (ft <= 0.0 && fb <= 0.0) || ft + fb >= height {
        blit(kit, image, Some(src), dst, 255);
        return;
    }
    let alpha_at = |y: f32| -> u8 {
        let mut t = 1.0f32;
        if y < ft {
            t = (y + 1.0) / (ft + 1.0);
        }
        if fb > 0.0 && y >= height - fb {
            t = t.min((height - y) / (fb + 1.0));
        }
        (t * 255.0) as u8
    };
    let src_h = src.height();
    let mut y = 0.0f32;
    while y < height {
        let alpha = alpha_at(y);
        let mut y1 = y + 1.0;
        while y1 < height && alpha_at(y1) == alpha {
            y1 += 1.0;
        }
        let band_top = src.top() + src_h * y / height;
        let band_bot = src.top() + src_h * y1 / height;
        let band_src = ltrb(src.left(), band_top, src.right(), band_bot);
        let band_dst = ltrb(dst.left(), dst.top() + y, dst.right(), dst.top() + y1);
        blit(kit, image, Some(band_src), band_dst, alpha);
        y = y1;
    }
}

fn pixel_at(data: &[u8], width: i32, x: i32, y: i32) -> [u8; 4] {
    let index = ((y as usize) * (width as usize) + x as usize) * 4;
    [
        data[index],
        data[index + 1],
        data[index + 2],
        data[index + 3],
    ]
}

fn mix8(from: u8, to: u8, t: f32) -> u8 {
    (from as f32 + (to as f32 - from as f32) * t + 0.5) as u8
}

fn blank_section_label(theme: Theme, image: &Image) -> Option<Image> {
    let width = image.width();
    let height = image.height();
    if width <= 0 || height <= 0 {
        return None;
    }
    let (x0, x1, y0, y1) = panel_ink(theme);
    let bx0 = x0 as i32;
    let bx1 = x1 as i32;
    let by0 = y0 as i32;
    let by1 = y1 as i32;
    let span = by1 - by0;
    if span <= 0 || bx1 <= bx0 || bx0 < 1 || bx1 >= width {
        return None;
    }
    let info = ImageInfo::new(
        (width, height),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let row_bytes = width as usize * 4;
    let mut data = vec![0u8; row_bytes * height as usize];
    let hint = aurorix_render::skia_safe::image::CachingHint::Allow;
    if !image.read_pixels(&info, &mut data, row_bytes, (0, 0), hint) {
        return None;
    }
    if theme.score.panel.has_custom_ink() {
        if by0 < 1 || by1 >= height {
            return None;
        }
        let count = (bx1 - bx0) as usize;
        let mut top = vec![[0u8; 4]; count];
        let mut bot = vec![[0u8; 4]; count];
        for (index, slot) in top.iter_mut().enumerate() {
            *slot = pixel_at(&data, width, bx0 + index as i32, by0 - 1);
        }
        for (index, slot) in bot.iter_mut().enumerate() {
            *slot = pixel_at(&data, width, bx0 + index as i32, by1);
        }
        for y in by0..by1 {
            let t = (y - by0 + 1) as f32 / (span + 1) as f32;
            for index in 0..count {
                let x = bx0 + index as i32;
                let offset = ((y as usize) * (width as usize) + x as usize) * 4;
                for channel in 0..4 {
                    data[offset + channel] = mix8(top[index][channel], bot[index][channel], t);
                }
            }
        }
    } else {
        let mut left = vec![[0u8; 4]; span as usize];
        let mut right = vec![[0u8; 4]; span as usize];
        for (index, slot) in left.iter_mut().enumerate() {
            *slot = pixel_at(&data, width, bx0 - 1, by0 + index as i32);
        }
        for (index, slot) in right.iter_mut().enumerate() {
            *slot = pixel_at(&data, width, bx1, by0 + index as i32);
        }
        for y in by0..by1 {
            let row = (y - by0) as usize;
            for x in bx0..bx1 {
                let t = (x - bx0) as f32 / (bx1 - bx0) as f32;
                let offset = ((y as usize) * (width as usize) + x as usize) * 4;
                for channel in 0..4 {
                    data[offset + channel] = mix8(left[row][channel], right[row][channel], t);
                }
            }
        }
    }
    let bytes = aurorix_render::skia_safe::Data::new_copy(&data);
    aurorix_render::skia_safe::images::raster_from_data(&info, bytes, row_bytes)
}

fn clip_polygon<R>(kit: &CanvasKit<'_>, points: &[(f32, f32)], body: impl FnOnce() -> R) -> R {
    let mut builder = PathBuilder::new();
    builder.move_to(points[0]);
    for point in &points[1..] {
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

pub fn score_request(input: &ScoreInput) -> RenderRequest {
    super::harness::card_request(CARD_WIDTH, input.height())
}

pub fn recent_request(input: &ScoreInput) -> RenderRequest {
    super::harness::card_request(CARD_WIDTH, input.height())
}

pub fn paint_score(kit: &CanvasKit<'_>, input: &ScoreInput) {
    let _scope = theme::enter(input.theme);
    paint_face(kit, input, Face::Score);
}

pub fn paint_recent(kit: &CanvasKit<'_>, input: &ScoreInput) {
    let _scope = theme::enter(input.theme);
    paint_face(kit, input, Face::Recent);
}

pub fn score_card(input: ScoreInput) -> aurorix_plugin::RenderCard {
    let request = score_request(&input);
    let cache_key = super::memo::fingerprint("score", &input);
    aurorix_plugin::RenderCard::new(request, SCORE_FILE, move |kit| paint_score(kit, &input))
        .with_cache_key(cache_key)
}

pub fn recent_card(input: ScoreInput) -> aurorix_plugin::RenderCard {
    let request = recent_request(&input);
    let cache_key = super::memo::fingerprint("recent", &input);
    aurorix_plugin::RenderCard::new(request, RECENT_FILE, move |kit| paint_recent(kit, &input))
        .with_cache_key(cache_key)
}

fn paint_face(kit: &CanvasKit<'_>, input: &ScoreInput, face: Face) {
    let theme = input.theme;
    draw_face_base(kit, theme);
    draw_result_titles(kit, input);
    draw_result_character(kit, input);
    draw_result_clear(kit, input);
    draw_result_difficulty(kit, input);
    draw_result_section(kit, input);
    draw_result_jacket(kit, input);
    draw_result_gauge(kit, input);
    draw_result_score(kit, input);
    draw_result_grade(kit, input);
    draw_result_counts(kit, input);
    draw_result_top_bar(kit, input, face);
    draw_result_buttons(kit, theme);
}

static FACE_BASE_MEMO: OnceLock<Mutex<Memo<(Image, u32, u32)>>> = OnceLock::new();

fn draw_face_base(kit: &CanvasKit<'_>, theme: Theme) {
    let full = ltrb(0.0, 0.0, CARD_WIDTH as f32, dr(SCORE_VIEW_H));
    if let Some(image) = face_base(kit, theme) {
        kit.image_rect(&image, full);
        return;
    }
    draw_result_backdrop(kit, theme);
    draw_result_banner(kit, theme);
}

fn face_base(kit: &CanvasKit<'_>, theme: Theme) -> Option<Image> {
    let key = format!("face-base|{}", theme.name);
    let built = memo::cached(&FACE_BASE_MEMO, &key, || {
        let width = CARD_WIDTH;
        let height = dr(SCORE_VIEW_H).round() as i32;
        if width <= 0 || height <= 0 {
            return None;
        }
        let info = ImageInfo::new_n32_premul((width, height), None);
        let mut surface = surfaces::raster(&info, None, None)?;
        surface
            .canvas()
            .clear(aurorix_render::skia_safe::Color::TRANSPARENT);
        let inner = CanvasKit::new(surface.canvas(), kit.context());
        draw_result_backdrop(&inner, theme);
        draw_result_banner(&inner, theme);
        let image = surface.image_snapshot();
        let bytes = (width as usize) * (height as usize) * 4;
        Some(((image, width as u32, height as u32), bytes))
    })?;
    Some(built.0)
}

fn draw_result_backdrop(kit: &CanvasKit<'_>, theme: Theme) {
    let full = ltrb(0.0, 0.0, CARD_WIDTH as f32, dr(SCORE_VIEW_H));
    let Some(image) = backdrop_image(kit, theme) else {
        kit.fill_rect(full, BACKDROP_FALLBACK);
        return;
    };
    let (iw, ih) = image_size(&image);
    if iw <= 0.0 {
        kit.fill_rect(full, BACKDROP_FALLBACK);
        return;
    }
    let height = full.width() as f64 * ih / iw;
    blit(
        kit,
        &image,
        None,
        ltrb(0.0, 0.0, full.right(), height as f32),
        255,
    );
}

fn draw_result_banner(kit: &CanvasKit<'_>, theme: Theme) {
    let Some(image) = themed_image(kit, theme, "panel/banner.png") else {
        return;
    };
    let (iw, ih) = image_size(&image);
    if iw <= 0.0 || ih <= 0.0 {
        return;
    }
    let width = 1920.0 * 0.667;
    let x = TITLE_CX - width / 2.0;
    let dark_src = ltrb(0.0, 0.0, 1920.0, 190.0);
    let dark_dst = crect(
        x,
        VIEW_MID - CARD_TOP_ROW,
        width,
        DARK_BOT_ROW - CARD_TOP_ROW,
    );
    paste_feathered(kit, &image, dark_src, dark_dst, 0.0, 0.0);
    let white_src = ltrb(0.0, 190.0, 1920.0, 944.0);
    let white_dst = crect(
        x,
        VIEW_MID - DARK_BOT_ROW,
        width,
        CARD_BOT_ROW - DARK_BOT_ROW,
    );
    paste_feathered(kit, &image, white_src, white_dst, 0.0, dr(8.0));
}

fn draw_result_titles(kit: &CanvasKit<'_>, input: &ScoreInput) {
    if !input.title.is_empty() {
        let style = font_text(48.0);
        let x = dr(TITLE_CX) - ink_center_x(kit, &style, &input.title);
        put_left(kit, &style, &input.title, x, dvy(TITLE_ROW), WHITE);
    }
    if !input.artist.is_empty() {
        let style = font_text(28.0);
        let x = dr(TITLE_CX) - ink_center_x(kit, &style, &input.artist);
        put_left(kit, &style, &input.artist, x, dvy(ARTIST_ROW), WHITE);
    }
}

#[derive(Debug, Clone, Copy)]
struct CharFit {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

#[derive(Debug, Clone, Copy)]
struct CharClip {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    u0: f64,
    v0: f64,
    u1: f64,
    v1: f64,
}

#[derive(Debug, Clone, Copy, Default)]
struct CharInk {
    top: i32,
    bottom: i32,
    left: i32,
    right: i32,
    cx: i32,
    cy: i32,
}

fn clamp_unit(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn lerp_i(a: f64, b: f64, t: f64) -> f64 {
    (a + (b - a) * t).trunc()
}

fn lerp_f(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn fit_char_sprite(art_w: f64, art_h: f64, canvas_h: f64, id: i32) -> CharFit {
    if art_w <= 0.0 || art_h <= 0.0 || canvas_h <= 0.0 {
        return CharFit {
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
        };
    }
    let layer_top = (canvas_h - CHAR_LAYER_H) / 2.0;
    let t = clamp_unit((canvas_h - CHAR_LAYER_H) / CHAR_SCREEN_SHIFT_R);
    let (dx_a, dy_a) = char_offset_a(id, t);
    let (dx_b, dy_b) = char_offset_b(id, t);
    CharFit {
        x: CHAR_NODE_X + CHAR_GLOBAL_DX + CHAR_ACTION_DX + dx_a + dx_b,
        y: layer_top + (CHAR_LAYER_H - CHAR_NODE_POS_Y) - CHAR_SCREEN_SHIFT_Y * t - dy_a - dy_b,
        w: art_w * CHAR_ART_SCALE,
        h: art_h * CHAR_ART_SCALE,
    }
}

fn view_t(canvas_h: f64) -> f64 {
    clamp_unit((canvas_h - CHAR_LAYER_H) / CHAR_SCREEN_SHIFT_R)
}

fn place_result_character(fit: CharFit, ink: CharInk, art_w: f64, id: i32) -> CharFit {
    if fit.w <= 0.0 || art_w <= 0.0 || ink.top < 0 {
        return fit;
    }
    let scale = fit.w / art_w;
    let mut out = fit;
    out.y = TOP_BAR_H - ink.top as f64 * scale;
    if !char_offset_known(id) {
        let cx = fit.x + ink.cx as f64 * scale;
        if cx < CHAR_UNKNOWN_CX {
            out.x += CHAR_UNKNOWN_CX - cx;
        }
    }
    out.x += keyed_value(CHAR_FINE_DX, id);
    out.y += keyed_value(CHAR_FINE_DY, id);
    out
}

fn char_clip_rect(fit: CharFit, canvas_w: f64, canvas_h: f64) -> CharClip {
    let empty = CharClip {
        x: 0.0,
        y: 0.0,
        w: 0.0,
        h: 0.0,
        u0: 0.0,
        v0: 0.0,
        u1: 0.0,
        v1: 0.0,
    };
    if fit.w <= 0.0 || fit.h <= 0.0 {
        return empty;
    }
    let x0 = fit.x.max(0.0);
    let y0 = fit.y.max(0.0);
    let x1 = (fit.x + fit.w).min(canvas_w);
    let y1 = (fit.y + fit.h).min(canvas_h);
    if x1 <= x0 || y1 <= y0 {
        return empty;
    }
    CharClip {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
        u0: (x0 - fit.x) / fit.w,
        u1: (x1 - fit.x) / fit.w,
        v0: (y0 - fit.y) / fit.h,
        v1: (y1 - fit.y) / fit.h,
    }
}

fn ink_static() -> &'static Mutex<HashMap<String, CharInk>> {
    static CACHE: OnceLock<Mutex<HashMap<String, CharInk>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn content_ink_box(image: &Image) -> CharInk {
    let width = image.width();
    let height = image.height();
    let mut ink = CharInk {
        top: -1,
        bottom: -1,
        left: -1,
        right: -1,
        cx: 0,
        cy: 0,
    };
    if width <= 0 || height <= 0 {
        return ink;
    }
    let info = ImageInfo::new(
        (width, height),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let mut data = vec![0u8; (width as usize) * (height as usize) * 4];
    let hint = aurorix_render::skia_safe::image::CachingHint::Allow;
    if !image.read_pixels(&info, &mut data, (width as usize) * 4, (0, 0), hint) {
        return ink;
    }
    let mut wsum = 0.0f64;
    let mut xsum = 0.0f64;
    let mut ysum = 0.0f64;
    for y in 0..height {
        let mut hit = false;
        for x in 0..width {
            let index = ((y as usize) * (width as usize) + x as usize) * 4;
            let alpha = data[index + 3];
            if alpha <= JUDGE_INK_ALPHA {
                continue;
            }
            hit = true;
            if ink.left < 0 || x < ink.left {
                ink.left = x;
            }
            if x > ink.right {
                ink.right = x;
            }
            let weight = alpha as f64;
            wsum += weight;
            xsum += weight * x as f64;
            ysum += weight * y as f64;
        }
        if hit {
            if ink.top < 0 {
                ink.top = y;
            }
            ink.bottom = y;
        }
    }
    if wsum > 0.0 {
        ink.cx = (xsum / wsum).round() as i32;
        ink.cy = (ysum / wsum).round() as i32;
    }
    ink
}

fn char_ink(kit: &CanvasKit<'_>, key: &str, image: &Image) -> CharInk {
    if let Ok(cache) = ink_static().lock() {
        if let Some(hit) = cache.get(key) {
            return *hit;
        }
    }
    let computed = content_ink_box(image);
    if let Ok(mut cache) = ink_static().lock() {
        cache.insert(key.to_string(), computed);
    }
    let _ = kit;
    computed
}

fn luma_static() -> &'static Mutex<HashMap<String, u32>> {
    static CACHE: OnceLock<Mutex<HashMap<String, u32>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn banner_mean_luma(image: &Image) -> f64 {
    let width = image.width();
    let height = image.height();
    if width <= 0 || height <= 0 {
        return 0.0;
    }
    let info = ImageInfo::new(
        (width, height),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let mut data = vec![0u8; (width as usize) * (height as usize) * 4];
    let hint = aurorix_render::skia_safe::image::CachingHint::Allow;
    if !image.read_pixels(&info, &mut data, (width as usize) * 4, (0, 0), hint) {
        return 0.0;
    }
    let mut sum = 0.0f64;
    let mut count = 0usize;
    let mut y = 0i32;
    while y < height {
        let mut x = 0i32;
        while x < width {
            let index = ((y as usize) * (width as usize) + x as usize) * 4;
            if data[index + 3] >= 200 {
                sum += 0.299 * data[index] as f64
                    + 0.587 * data[index + 1] as f64
                    + 0.114 * data[index + 2] as f64;
                count += 1;
            }
            x += 8;
        }
        y += 8;
    }
    if count == 0 {
        return 0.0;
    }
    sum / count as f64
}

fn cached_luma(kit: &CanvasKit<'_>, key: &str) -> Option<f64> {
    if let Ok(cache) = luma_static().lock() {
        if let Some(hit) = cache.get(key) {
            return Some(*hit as f64 / 1000.0);
        }
    }
    let image = image_of(kit, key)?;
    let value = banner_mean_luma(&image);
    if let Ok(mut cache) = luma_static().lock() {
        cache.insert(key.to_string(), (value * 1000.0).round() as u32);
    }
    Some(value)
}

fn name_plate_ink(kit: &CanvasKit<'_>, banner: Option<&str>) -> Color {
    let Some(key) = banner else {
        return WHITE;
    };
    match cached_luma(kit, key) {
        Some(luma) if luma > 150.0 => NAME_PLATE_LIGHT_INK,
        _ => WHITE,
    }
}

fn draw_result_character(kit: &CanvasKit<'_>, input: &ScoreInput) {
    let key = input
        .badge
        .character
        .clone()
        .unwrap_or_else(|| String::from(DEFAULT_CHARACTER));
    let Some(image) = image_of(kit, &key) else {
        return;
    };
    let (art_w, art_h) = image_size(&image);
    let id = char_id_from_key(&key).unwrap_or(-1);
    let mut fit = fit_char_sprite(art_w, art_h, VIEW_H, id);
    if fit.w <= 0.0 || fit.h <= 0.0 {
        return;
    }
    let ink = char_ink(kit, &key, &image);
    if ink.top < 0 {
        return;
    }
    let art = stem_of(&key);
    if let Some(scale) = art_scale_of(&art) {
        if scale > 0.0 && scale != 1.0 {
            let old_w = fit.w;
            fit.w *= scale;
            fit.h *= scale;
            let s_old = old_w / art_w;
            let s_new = fit.w / art_w;
            fit.x += ink.cx as f64 * (s_old - s_new);
        }
    }
    fit = place_result_character(fit, ink, art_w, id);
    fit.x += art_dx_of(&art);
    fit.y += art_dy_of(&art);
    let clip = char_clip_rect(fit, DESIGN_W, SCORE_VIEW_H);
    if clip.w <= 0.0 || clip.h <= 0.0 {
        return;
    }
    let src = ltrb(
        (clip.u0 * art_w).round() as f32,
        (clip.v0 * art_h).round() as f32,
        (clip.u1 * art_w).round() as f32,
        (clip.v1 * art_h).round() as f32,
    );
    if src.width() <= 0.0 || src.height() <= 0.0 {
        return;
    }
    let dst = srect(clip.x, clip.y, clip.x + clip.w, clip.y + clip.h);
    blit(kit, &image, Some(src), dst, 255);
}

fn draw_result_clear(kit: &CanvasKit<'_>, input: &ScoreInput) {
    if !input.has_clear() {
        return;
    }
    let file = clear_asset(input.clear_type());
    let Some(image) = image_of(kit, file) else {
        return;
    };
    let (iw, ih) = image_size(&image);
    if iw <= 0.0 || ih <= 0.0 {
        return;
    }
    let target_w = 600.0;
    let target_h = target_w * ih / iw;
    let dst = ccenter(TITLE_CX, VIEW_MID - 312.0, target_w, target_h);
    blit(kit, &image, None, dst, 255);
}

fn recall_asset(difficulty: Difficulty) -> &'static str {
    match difficulty {
        Difficulty::Past => "difficulty/max-recall-past.png",
        Difficulty::Present => "difficulty/max-recall-present.png",
        Difficulty::Beyond => "difficulty/max-recall-beyond.png",
        Difficulty::Eternal => "difficulty/max-recall-eternal.png",
        Difficulty::Future => "difficulty/max-recall-future.png",
        Difficulty::Ins => "difficulty/max-recall-inscribed.png",
    }
}

fn diff_name(difficulty: Difficulty) -> &'static str {
    match difficulty {
        Difficulty::Past => "PAST",
        Difficulty::Present => "PRESENT",
        Difficulty::Beyond => "BEYOND",
        Difficulty::Eternal => "ETERNAL",
        Difficulty::Future => "FUTURE",
        Difficulty::Ins => "INSCRIBED",
    }
}

fn diff_color(difficulty: Difficulty) -> Color {
    match difficulty {
        Difficulty::Past => Color::rgb(38, 181, 217),
        Difficulty::Present => Color::rgb(181, 206, 105),
        Difficulty::Future => Color::rgb(160, 59, 128),
        Difficulty::Beyond => Color::rgb(207, 32, 62),
        Difficulty::Eternal => Color::rgb(137, 116, 164),
        Difficulty::Ins => Color::rgb(56, 85, 198),
    }
}

fn level_mark(input: &ScoreInput) -> (String, bool) {
    let level = input.level.trim();
    if let Some(rest) = level.strip_suffix('+') {
        return (rest.trim().to_string(), true);
    }
    if !level.is_empty() {
        return (level.to_string(), false);
    }
    if let Some(constant) = input.constant.filter(|constant| *constant > 0.0) {
        let tenth = (constant * 10.0).round() as i64;
        return (format!("{}", tenth / 10), tenth % 10 >= 7);
    }
    (String::new(), false)
}

fn draw_result_difficulty(kit: &CanvasKit<'_>, input: &ScoreInput) {
    if let Some(image) = image_of(kit, recall_asset(input.difficulty)) {
        let dst = ccenter(
            135.0,
            492.36,
            391.0 * JUDGE_ART_SCALE,
            110.0 * JUDGE_ART_SCALE,
        );
        blit(kit, &image, None, dst, 255);
    }
    let exo34 = font_latin(34.0);
    let l2_20 = font_text(20.0);
    let (value, plus) = level_mark(input);
    if !value.is_empty() {
        let shift = match plus {
            true => LEVEL_INK_SHIFT,
            false => 0.0,
        };
        let ink_x = dr(54.0) + shift - ink_center_x(kit, &exo34, &value);
        let ink_y = dqy(491.893) - LEVEL_INK_RISE + ink_center_y(kit, &exo34, &value);
        draw(kit, &exo34, &value, ink_x + dr(2.0), ink_y + dr(2.0), SHADOW);
        draw(kit, &exo34, &value, ink_x, ink_y, WHITE);
        if plus {
            put_left(kit, &exo34, "+", dr(68.0), dqy(501.789), WHITE);
        }
    }
    put_left_flat(
        kit,
        &l2_20,
        diff_name(input.difficulty),
        dr(96.0),
        dqy(504.267),
        diff_color(input.difficulty),
    );
    let (stamp, label) = match input.play_at {
        Some(at) => {
            let offset = FixedOffset::east_opt(8 * 3600).expect("固定时区");
            (
                at.with_timezone(&offset)
                    .format("%Y-%m-%d %H:%M")
                    .to_string(),
                "Played At ",
            )
        }
        None => (
            Utc::now()
                .with_timezone(&FixedOffset::east_opt(8 * 3600).expect("固定时区"))
                .format("%Y-%m-%d %H:%M")
                .to_string(),
            "Created At ",
        ),
    };
    let style = font_text(14.0);
    put_left_flat(
        kit,
        &style,
        &format!("{label}{stamp}"),
        dr(98.0),
        dqy(477.0),
        POTENTIAL_INK,
    );
}

fn draw_result_section(kit: &CanvasKit<'_>, input: &ScoreInput) {
    let theme = input.theme;
    let Some(image) = themed_image(kit, theme, "panel/section_high.png") else {
        return;
    };
    let (iw, ih) = image_size(&image);
    if iw <= 0.0 || ih <= 0.0 {
        return;
    }
    let blanked = blank_section_label(theme, &image);
    let image = blanked.unwrap_or(image);
    let dst = crect(437.86, 469.12, 691.0 * 0.66, 394.0 * 0.66);
    let src = ltrb(0.0, 0.0, iw as f32, ih as f32);
    paste_feathered(kit, &image, src, dst, dr(10.0), dr(10.0));
    draw_potential_row(kit, input, panel_word_dx(theme));
}

fn panel_ink(theme: Theme) -> (f64, f64, f64, f64) {
    theme.score.panel.ink()
}

fn panel_word_dx(theme: Theme) -> f64 {
    theme.score.panel.word_dx()
}

fn draw_potential_row(kit: &CanvasKit<'_>, input: &ScoreInput, word_dx: f64) {
    let label = font_text(19.0);
    put_center(
        kit,
        &label,
        "POTENTIAL",
        dr(525.0 + word_dx),
        dvy(456.0),
        WHITE,
    );
    let value = font_score(22.0);
    let constant = input.constant.unwrap_or(0.0);
    put_left(
        kit,
        &value,
        &format!("{constant:.1}  >"),
        dr(631.0 + word_dx),
        dvy(456.0),
        WHITE,
    );
    put_left(
        kit,
        &value,
        &format_rating(input.rating),
        dr(701.0 + word_dx),
        dvy(456.0),
        WHITE,
    );
}

fn difficulty_suffix(difficulty: Difficulty) -> &'static str {
    match difficulty {
        Difficulty::Past => "pst",
        Difficulty::Present => "prs",
        Difficulty::Beyond => "byd",
        Difficulty::Eternal => "etr",
        Difficulty::Future => "ftr",
        Difficulty::Ins => "ins",
    }
}

fn split_key(key: &str) -> (&str, &str) {
    match key.rsplit_once('/') {
        Some((dir, name)) => (dir, name),
        None => ("", key),
    }
}

fn stem_of(key: &str) -> String {
    let base = key.rsplit('/').next().unwrap_or(key);
    match base.rsplit_once('.') {
        Some((stem, _)) => stem.to_string(),
        None => base.to_string(),
    }
}

fn jacket_candidates(input: &ScoreInput) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    if !input.song_id.is_empty() {
        let suffix = difficulty_suffix(input.difficulty);
        keys.push(format!("jackets/{}_{suffix}.jpg", input.song_id));
        keys.push(format!("jackets/{}.jpg", input.song_id));
        keys.push(format!("jacket_thumbs/{}_{suffix}.jpg", input.song_id));
        keys.push(format!("jacket_thumbs/{}.jpg", input.song_id));
    }
    let Some(raw) = input.jacket.as_deref() else {
        return keys;
    };
    let key = raw.trim();
    if key.is_empty() {
        return Vec::new();
    }
    let (dir, file) = split_key(key);
    let (stem, ext) = match file.rsplit_once('.') {
        Some((stem, ext)) => (stem, ext),
        None => (file, "jpg"),
    };
    if dir.is_empty() || dir == "jackets" {
        keys.push(format!(
            "jackets/{stem}_{}.{ext}",
            difficulty_suffix(input.difficulty)
        ));
        keys.push(format!("jackets/{stem}.{ext}"));
    }
    keys.push(key.to_string());
    keys
}

fn draw_result_jacket(kit: &CanvasKit<'_>, input: &ScoreInput) {
    if input.song_id.is_empty() {
        return;
    }
    for key in jacket_candidates(input) {
        if let Some(image) = image_of(kit, &key) {
            let (iw, ih) = image_size(&image);
            if iw <= 0.0 || ih <= 0.0 {
                return;
            }
            blit(kit, &image, None, crect(30.0, 452.93, 375.0, 375.0), 255);
            return;
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct GaugeLayer {
    file: &'static str,
    multiply: bool,
}

const fn overlay_layer(file: &'static str) -> GaugeLayer {
    GaugeLayer {
        file,
        multiply: false,
    }
}

const fn multiply_layer(file: &'static str) -> GaugeLayer {
    GaugeLayer {
        file,
        multiply: true,
    }
}

#[derive(Debug, Clone)]
struct GaugeSkin {
    bar: &'static str,
    bar_clear: &'static str,
    base: Vec<GaugeLayer>,
    overlay: Vec<GaugeLayer>,
    top: &'static str,
    grid: &'static str,
}

fn gauge_skin_of(clear_type: Option<i32>, char_id: Option<i32>) -> GaugeSkin {
    let cleared = clear_type.is_some_and(|value| value != 0);
    let kind = clear_type.unwrap_or(0);
    let base_skin = |bar: &'static str, bar_clear: &'static str| GaugeSkin {
        bar,
        bar_clear,
        base: Vec::new(),
        overlay: Vec::new(),
        top: "base/hp_top.png",
        grid: "base/hp_grid.png",
    };
    match char_id.unwrap_or(-1) {
        24 => {
            if cleared && kind == 5 {
                return GaugeSkin {
                    bar: "gauge/hp_bar_hard.png",
                    bar_clear: "",
                    base: Vec::new(),
                    overlay: Vec::new(),
                    top: "base/hp_top_red.png",
                    grid: "base/hp_grid.png",
                };
            }
            base_skin(
                "char/chuni/hp_bar_chuni.png",
                "char/chuni/hp_bar_chuni_clear.png",
            )
        }
        23 => GaugeSkin {
            bar: "char/saya2/hp_bar_saya2_lv1.png",
            bar_clear: "",
            base: vec![multiply_layer("char/saya2/hp_bar_saya2_base.png")],
            overlay: vec![overlay_layer("char/saya2/hp_bar_saya2_overlay.png")],
            top: "base/hp_top.png",
            grid: "base/hp_grid.png",
        },
        35 => GaugeSkin {
            bar: if cleared {
                "char/tempest/hp_bar_tempest_up.png"
            } else {
                "char/tempest/hp_bar_tempest_down.png"
            },
            bar_clear: "",
            base: Vec::new(),
            overlay: Vec::new(),
            top: "base/hp_top_red.png",
            grid: "base/hp_grid.png",
        },
        55 => GaugeSkin {
            bar: "char/fatalis/hp_bar_fatalis_overlay.png",
            bar_clear: "",
            base: Vec::new(),
            overlay: vec![overlay_layer("char/fatalis/hp_bar_fatalis_missing.png")],
            top: "base/hp_top.png",
            grid: "char/fatalis/hp_bar_fatalis_grid.png",
        },
        98 => GaugeSkin {
            bar: "char/insight/hp-insight.png",
            bar_clear: "",
            base: Vec::new(),
            overlay: vec![overlay_layer("char/insight/hp-insight-hard.png")],
            top: "base/hp_top.png",
            grid: "base/hp_grid.png",
        },
        78 => GaugeSkin {
            bar: "base/hp_bar.png",
            bar_clear: "base/hp_bar_clear.png",
            base: Vec::new(),
            overlay: vec![overlay_layer("char/nell/hp_bar_limit_cover_nell.png")],
            top: "base/hp_top.png",
            grid: "base/hp_grid.png",
        },
        15 => base_skin(
            "gauge/hp_bar_overflow.png",
            "gauge/hp_bar_overflow_clear.png",
        ),
        34 => base_skin("gauge/hp_bar_grade.png", ""),
        _ => match kind {
            4 => base_skin("gauge/hp_bar_easy.png", "gauge/hp_bar_easy_clear.png"),
            5 => GaugeSkin {
                bar: "gauge/hp_bar_hard.png",
                bar_clear: "",
                base: Vec::new(),
                overlay: Vec::new(),
                top: "base/hp_top_red.png",
                grid: "base/hp_grid.png",
            },
            _ => base_skin("base/hp_bar.png", "base/hp_bar_clear.png"),
        },
    }
}

fn gauge_key(name: &str) -> String {
    format!("{GAUGE_DIR}/{name}")
}

fn gauge_scale_of(th: f64) -> f64 {
    if th <= 487.0 {
        GAUGE_H / 487.0
    } else {
        GAUGE_H / th
    }
}

fn gauge_fit(tw: f64, th: f64) -> Rect {
    let scale = gauge_scale_of(th);
    let w = tw * scale;
    let h = th * scale;
    let cx = GAUGE_X + GAUGE_W / 2.0;
    let cy = GAUGE_TOP_Y - GAUGE_H / 2.0;
    crect(cx - w / 2.0, cy + h / 2.0, w, h)
}

fn gauge_body_fit(th: f64) -> Rect {
    crect(GAUGE_X, GAUGE_TOP_Y, GAUGE_W, th * gauge_scale_of(th))
}

fn gauge_top_band(tw: f64, th: f64) -> Rect {
    let scale = gauge_scale_of(th);
    let w = tw * scale;
    crect(
        GAUGE_X + GAUGE_W / 2.0 - w / 2.0,
        GAUGE_TOP_Y,
        w,
        th * scale,
    )
}

fn gauge_art_size(kit: &CanvasKit<'_>, file: &str) -> (f64, f64) {
    image_of(kit, &gauge_key(file))
        .map(|image| image_size(&image))
        .unwrap_or((39.0, 487.0))
}

fn paste_gauge(kit: &CanvasKit<'_>, layer: GaugeLayer, area: Rect) {
    let Some(image) = image_of(kit, &gauge_key(layer.file)) else {
        return;
    };
    if layer.multiply {
        kit.layer(area, 1.0, BlendMode::Multiply, |inner: &CanvasKit<'_>| {
            blit(inner, &image, None, area, 255);
        });
        return;
    }
    blit(kit, &image, None, area, 255);
}

fn draw_gauge_label(kit: &CanvasKit<'_>, text: &str) {
    let style = font_number(19.0);
    let cx = dr(GAUGE_X + GAUGE_W / 2.0);
    let base = dqy(GAUGE_TOP_Y) + dr(8.0) + ascent_half(kit, &style);
    let x = cx - width(kit, &style, text) / 2.0;
    draw(kit, &style, text, x + dr(1.0), base + dr(1.0), INK_TOP);
    draw(kit, &style, text, x, base, WHITE);
}

fn draw_result_gauge(kit: &CanvasKit<'_>, input: &ScoreInput) {
    let cleared = input.cleared();
    let skin = gauge_skin_of(input.clear_type, input.char_id());
    if cleared {
        let bar = if skin.bar_clear.is_empty() {
            skin.bar
        } else {
            skin.bar_clear
        };
        if !bar.is_empty() {
            let (_, th) = gauge_art_size(kit, bar);
            if let Some(image) = image_of(kit, &gauge_key(bar)) {
                blit(
                    kit,
                    &image,
                    None,
                    gauge_body_fit(th),
                    (GAUGE_BAR_OPACITY * 255.0).round() as u8,
                );
            }
        }
    }
    for layer in &skin.base {
        let (tw, th) = gauge_art_size(kit, layer.file);
        let area = gauge_fit(tw, th);
        paste_gauge(kit, *layer, area);
    }
    for layer in &skin.overlay {
        let (tw, th) = gauge_art_size(kit, layer.file);
        let area = gauge_fit(tw, th);
        paste_gauge(kit, *layer, area);
    }
    let grid = if skin.grid.is_empty() {
        "base/hp_grid.png"
    } else {
        skin.grid
    };
    let (tw, th) = gauge_art_size(kit, grid);
    let area = gauge_top_band(tw, th);
    paste_gauge(
        kit,
        GaugeLayer {
            file: grid,
            multiply: false,
        },
        area,
    );
    let _ = skin.top;
    if cleared {
        draw_gauge_label(kit, "100");
    }
}

fn draw_result_score(kit: &CanvasKit<'_>, input: &ScoreInput) {
    let style = font_score(58.0);
    put_center_base(
        kit,
        &style,
        &score_text(input.score),
        dr(640.0),
        dqy(390.0),
        WHITE,
    );
}

fn draw_result_grade(kit: &CanvasKit<'_>, input: &ScoreInput) {
    let Some(image) = image_of(kit, grade_asset(input.grade)) else {
        return;
    };
    let (iw, ih) = image_size(&image);
    if iw <= 0.0 || ih <= 0.0 {
        return;
    }
    let target_h = 100.0;
    let w = target_h * iw / ih;
    blit(kit, &image, None, ccenter(640.0, 278.55, w, target_h), 255);
}

fn draw_result_counts(kit: &CanvasKit<'_>, input: &ScoreInput) {
    let theme = input.theme;
    let l2_20 = font_text(20.0);
    let count = font_display(24.0);
    let shiny = font_display(17.0);
    struct Row {
        judge: Judge,
        value: i64,
        label_right: f64,
        cy: f64,
        w: f64,
        h: f64,
        badge_x: f64,
    }
    let mut rows = [
        Row {
            judge: Judge::Pure,
            value: input.pure,
            label_right: 629.0,
            cy: 186.0,
            w: 0.0,
            h: 0.0,
            badge_x: 0.0,
        },
        Row {
            judge: Judge::Far,
            value: input.far,
            label_right: 622.0,
            cy: 149.0,
            w: 0.0,
            h: 0.0,
            badge_x: 0.0,
        },
        Row {
            judge: Judge::Lost,
            value: input.lost,
            label_right: 629.0,
            cy: 112.5,
            w: 0.0,
            h: 0.0,
            badge_x: 0.0,
        },
    ];
    let style = theme.score.judge;
    for row in rows.iter_mut() {
        let badge = style.badge(row.judge.key());
        row.w = f64::from(badge.w) * JUDGE_ART_SCALE;
        row.h = f64::from(badge.h) * JUDGE_ART_SCALE;
        row.badge_x = f64::from(badge.cx);
    }
    let count_x = f64::from(style.count_x());
    let count_dy = f64::from(style.count_dy());
    let label_from_art = style.label_from_art;
    let shiny_gap = dr(14.0);
    let shiny_dy = dr(count_dy);
    let shiny_right = style.count_right().map(|value| dr(f64::from(value)));
    for row in rows {
        let badge_x = if row.badge_x > 0.0 {
            row.badge_x
        } else {
            623.0
        };
        let area = ccenter(badge_x, row.cy, row.w, row.h);
        if let Some(image) = themed_image(kit, theme, row.judge.file()) {
            blit(kit, &image, None, area, 255);
        }
        if !label_from_art {
            put_right_glow(
                kit,
                &l2_20,
                row.judge.label(),
                dr(style.label_right(row.label_right)),
                dqy(row.cy),
                GlowInk {
                    fill: WHITE,
                    glow: row.judge.glow(),
                    halo: 2.0,
                },
            );
        }
        let num = format!("{}", row.value);
        let tag = match (row.judge, input.shiny_perfect) {
            (Judge::Pure, Some(shiny)) if shiny > 0 => {
                let separator = if label_from_art { " " } else { "" };
                format!("+{separator}{shiny}")
            }
            _ => String::new(),
        };
        put_center_glow(
            kit,
            &count,
            &num,
            dr(count_x),
            dqy(row.cy) + dr(count_dy),
            GlowInk {
                fill: WHITE,
                glow: row.judge.glow(),
                halo: 3.0,
            },
        );
        if tag.is_empty() {
            continue;
        }
        let x = match shiny_right {
            Some(right) => right + dr(4.0),
            None => dr(count_x) + width(kit, &count, &num) / 2.0 + shiny_gap,
        };
        put_left_glow(
            kit,
            &shiny,
            &tag,
            x,
            dqy(row.cy) + shiny_dy,
            GlowInk {
                fill: WHITE,
                glow: row.judge.glow(),
                halo: 2.0,
            },
        );
    }
}

fn draw_result_top_strip(kit: &CanvasKit<'_>, theme: Theme) {
    let Some(image) = themed_image(kit, theme, "topbar/top_bar_bg.png") else {
        return;
    };
    let (iw, ih) = image_size(&image);
    if iw <= 0.0 || ih <= 0.0 {
        return;
    }
    let x0 = dr(TOP_STRIP_LEFT);
    let y0 = dr(0.0);
    let dst = ltrb(x0, y0, x0 + dr(iw), y0 + dr(ih));
    blit(kit, &image, None, dst, 255);
}

fn draw_name_plate(kit: &CanvasKit<'_>, banner: Option<&str>) {
    let plate = srect(PLATE_LEFT, 0.0, PLATE_RIGHT, TOP_BAND_H);
    if plate.width() <= 0.0 || plate.height() <= 0.0 {
        return;
    }
    let peak = dr(PLATE_PEAK);
    let mid = peak + 0.5;
    let top_d = (mid - plate.top()).abs();
    let bot_d = (plate.bottom() - mid).abs();
    let points = [
        (plate.left() + top_d, plate.top()),
        (plate.right() - top_d, plate.top()),
        (plate.right(), mid),
        (plate.right() - bot_d, plate.bottom()),
        (plate.left() + bot_d, plate.bottom()),
        (plate.left(), mid),
    ];
    clip_polygon(kit, &points, || {
        match banner.and_then(|key| image_of(kit, key)) {
            Some(image) => blit(kit, &image, None, plate, 255),
            None => {
                kit.fill_rect(plate, NAME_PLATE_FALLBACK);
            }
        }
    });
}

fn draw_avatar(kit: &CanvasKit<'_>, input: &ScoreInput) {
    let avatar = avatar::Avatar::new(input.badge.avatar.as_deref())
        .with_ptt(input.badge.ptt)
        .with_frame(input.rating_frame.or(input.badge.frame))
        .with_global_rank(input.badge.global_rank);
    let cx = dr(AVATAR_CX);
    let cy = dr(AVATAR_CY);
    let radius = dr(AVATAR_SIZE) / 2.0;
    avatar::draw_icon_and_border(kit, cx, cy, radius, &avatar);
    if let Some(rank) = avatar.global_rank {
        avatar::draw_rank_text_at(kit, cx, cy, radius, rank);
    }
}

fn rating_badge_key(frame: Option<i32>, ptt: Option<f64>) -> String {
    match (ptt.filter(|value| *value > 0.0), crate::cards::avatar::resolved_frame(ptt, frame)) {
        (Some(_), Some(frame)) => format!("{RATING_KEY_DIR}/rating_{frame}.png"),
        _ => format!("{RATING_KEY_DIR}/rating_off.png"),
    }
}

fn split_ptt(ptt: f64) -> (String, String) {
    let text = format!("{ptt:.3}");
    match text.split_once('.') {
        Some((head, tail)) => (head.to_string(), format!(".{tail}")),
        None => (text, String::new()),
    }
}

fn draw_runs_outlined(
    kit: &CanvasKit<'_>,
    runs: &[(TextStyle, f32, String)],
    base: f32,
    edge: Color,
) {
    for (dx, dy) in RING8 {
        for (style, x, text) in runs {
            draw(kit, style, text, x + dx, base + dy, edge);
        }
    }
    for (style, x, text) in runs {
        draw(kit, style, text, *x, base, WHITE);
    }
}

fn draw_ptt_digits(kit: &CanvasKit<'_>, cx: f32, cy: f32, size: f64, ptt: Option<f64>) {
    let int_face = font_number(size);
    let (head, tail) = match ptt {
        Some(value) => split_ptt(value),
        None => (String::from("--"), String::new()),
    };
    let dec_face = if tail.is_empty() {
        int_face.clone()
    } else {
        font_number(size * PTT_DEC_RATIO)
    };
    let head_width = width(kit, &int_face, &head);
    let ink_left = ink_edge(kit, &int_face, &head, true);
    let ink_right = if tail.is_empty() {
        head_width
    } else {
        head_width + ink_edge(kit, &dec_face, &tail, false)
    };
    let x = cx - (ink_left + ink_right) / 2.0;
    let base = cy + ink_center_y(kit, &int_face, &head);
    let mut runs: Vec<(TextStyle, f32, String)> = Vec::new();
    runs.push((int_face, x, head));
    if !tail.is_empty() {
        runs.push((dec_face, x + head_width, tail));
    }
    draw_runs_outlined(kit, &runs, base, PTT_EDGE);
}

fn draw_result_top_bar(kit: &CanvasKit<'_>, input: &ScoreInput, face: Face) {
    let theme = input.theme;
    draw_result_top_strip(kit, theme);
    let title_face = font_text(24.0);
    let name_face = font_latin(26.0);
    put_left_ink(
        kit,
        &title_face,
        face.page_title(),
        dr(18.0),
        dr(TOP_TEXT_ROW),
        INK_TOP,
    );
    if !input.badge.name.is_empty() {
        draw_name_plate(kit, input.badge.banner.as_deref());
        let ink = name_plate_ink(kit, input.badge.banner.as_deref());
        put_center_ink(
            kit,
            &name_face,
            &input.badge.name,
            dr((PLATE_LEFT + PLATE_RIGHT) / 2.0),
            dr(TOP_BAND_H / 2.0),
            ink,
        );
    }
    draw_avatar(kit, input);
    let ptt = input.ptt_or_badge();
    let area = srect(656.0, 39.0, 701.0, 82.0);
    let badge = rating_badge_key(input.rating_frame, ptt);
    let drawn = match image_of(kit, &badge) {
        Some(image) => {
            blit(kit, &image, sketch::rating_crop(&badge), area, 255);
            true
        }
        None => false,
    };
    if ptt.is_some() || drawn {
        draw_ptt_digits(kit, dr(678.5), dr(60.5), PTT_INT_SIZE, ptt);
    }
    put_left_ink(
        kit,
        &title_face,
        &format!("Generated by {BOT_NAME}"),
        dr(WATERMARK_LEFT),
        dr(TOP_TEXT_ROW),
        INK_TOP,
    );
}

fn put_left_ink(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    text: &str,
    x: f32,
    row: f32,
    color: Color,
) {
    if text.is_empty() {
        return;
    }
    draw(kit, style, text, x, row + ink_center_y(kit, style, text), color);
}

fn put_center_ink(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    text: &str,
    cx: f32,
    row: f32,
    color: Color,
) {
    if text.is_empty() {
        return;
    }
    let x = cx - ink_center_x(kit, style, text);
    put_left_ink(kit, style, text, x, row, color);
}

fn draw_result_buttons(kit: &CanvasKit<'_>, theme: Theme) {
    let face = font_display(21.5);
    let buttons = [
        ("button/back.png", "返回", 107.05, 216.1, 68.0, 97.0),
        (
            "button/mid_button.png",
            "分享",
            637.034,
            220.0,
            66.7,
            634.034,
        ),
        ("button/retry.png", "重试", 1171.45, 217.1, 68.3, 1183.765),
    ];
    let label_up = 42.0;
    for (file, label, cx, w, h, label_x) in buttons {
        let Some(image) = themed_image(kit, theme, file) else {
            continue;
        };
        let top = SCORE_VIEW_H - h;
        let dst = srect(cx - w / 2.0, top, cx + w / 2.0, SCORE_VIEW_H);
        blit(kit, &image, None, dst, 255);
        put_center_flat(
            kit,
            &face,
            label,
            dr(label_x),
            dr(SCORE_VIEW_H - label_up),
            INK_TOP,
        );
    }
}

fn score_text(score: i64) -> String {
    let mut digits = score.max(0).to_string();
    while digits.len() < 8 {
        digits.insert(0, '0');
    }
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push('\'');
        }
        out.push(digit);
    }
    out
}

pub fn judge_key(theme: Theme, name: &str) -> String {
    theme.asset(ThemePage::Score, &format!("result/judge/{name}.png"))
}

fn char_offset_a(id: i32, t: f64) -> (f64, f64) {
    match id {
        0 => (0.0, 0.0),
        1 => (0.0, 0.0),
        2 => (100.0, lerp_i(50.0, 10.0, t)),
        4 => (0.0, lerp_i(0.0, 20.0, t)),
        5 => (140.0, 80.0),
        6 => (50.0, 0.0),
        7 => (50.0, 0.0),
        8 => (90.0, 0.0),
        9 => (100.0, 0.0),
        10 => (0.0, 0.0),
        11 => (0.0, 0.0),
        12 => (0.0, 50.0),
        13 => (30.0, 0.0),
        14 => (105.0, 0.0),
        15 => (90.0, 0.0),
        18 => (65.0, 0.0),
        19 => (0.0, 0.0),
        20 => (130.0, 0.0),
        21 => (120.0, 0.0),
        23 => (0.0, 0.0),
        24 => (0.0, 0.0),
        28 => (0.0, 0.0),
        29 => (70.0, 0.0),
        30 => (0.0, lerp_i(0.0, 20.0, t)),
        33 => (90.0, 0.0),
        34 => (0.0, lerp_i(0.0, 20.0, t)),
        35 => (0.0, lerp_i(90.0, 40.0, t)),
        36 => (0.0, lerp_i(120.0, 90.0, t)),
        37 => (30.0, -30.0),
        38 => (0.0, 0.0),
        39 => (0.0, lerp_i(75.0, 40.0, t)),
        40 => (110.0, lerp_i(0.0, 20.0, t)),
        42 => (0.0, 0.0),
        43 => (0.0, 0.0),
        44 => (180.0, 0.0),
        45 => (-30.0, 10.0),
        46 => (50.0, 10.0),
        50 => (0.0, 0.0),
        51 => (20.0, 0.0),
        52 => (50.0, 0.0),
        53 => (170.0, -30.0),
        54 => (195.0, lerp_f(50.0, 0.0, t)),
        55 => (80.0, 130.0),
        56 => (230.0, 85.0),
        57 => (195.0, 95.0),
        58 => (230.0, 0.0),
        59 => (110.0, 0.0),
        60 => (110.0, 0.0),
        61 => (80.0, lerp_f(35.0, 20.0, t)),
        62 => (170.0, -30.0),
        63 => (170.0, -30.0),
        64 => (20.0, -5.0),
        65 => (210.0, 0.0),
        66 => (90.0, 20.0),
        67 => (30.0, lerp_f(70.0, 40.0, t)),
        68 => (100.0, 135.0),
        69 => (50.0, lerp_i(110.0, 80.0, t)),
        70 => (lerp_i(0.0, 0.0, t), lerp_i(70.0, 50.0, t)),
        71 => (0.0, lerp_i(245.0, 185.0, t)),
        72 => (0.0, 20.0),
        73 => (0.0, 40.0),
        74 => (90.0, 0.0),
        75 => (50.0, 0.0),
        76 => (120.0, 70.0),
        77 => (-15.0, 70.0),
        78 => (-60.0, 140.0),
        79 => (-90.0, 145.0),
        80 => (-40.0, 145.0),
        81 => (20.0, 140.0),
        82 => (-10.0, 235.0),
        83 => (-5.0, lerp_f(200.0, 145.0, t)),
        84 => (0.0, 80.0),
        85 => (165.0, lerp_f(120.0, 70.0, t)),
        86 => (15.0, lerp_f(120.0, 30.0, t)),
        87 => (15.0, lerp_f(80.0, 60.0, t)),
        88 => (10.0, lerp_f(90.0, 80.0, t)),
        89 => (105.0, lerp_f(175.0, 150.0, t)),
        90 => (50.0, lerp_f(100.0, 95.0, t)),
        91 => (60.0, 0.0),
        92 => (-25.0, lerp_i(165.0, 135.0, t)),
        93 => (45.0, 0.0),
        95 => (-10.0, lerp_f(-20.0, 0.0, t)),
        96 => (125.0, 75.0),
        97 => (40.0, 0.0),
        98 => (-45.0, 120.0),
        100 => (55.0, 0.0),
        101 => (10.0, lerp_f(265.0, 180.0, t)),
        102 => (0.0, lerp_i(120.0, 5.0, t)),
        _ => (0.0, 0.0),
    }
}

fn char_offset_b(id: i32, t: f64) -> (f64, f64) {
    match id {
        0 => (0.0, lerp_i(0.0, -80.0, t)),
        1 => (70.0, 0.0),
        5 => (30.0, 8.0),
        11 => (0.0, lerp_i(0.0, -90.0, t)),
        12 => (40.0, 0.0),
        18 => (0.0, lerp_i(70.0, 0.0, t)),
        23 => (lerp_f(20.0, 30.0, t), 0.0),
        36 => (-40.0, 0.0),
        40 => (0.0, lerp_i(70.0, 0.0, t)),
        42 => (-20.0, 0.0),
        47 => (25.0, 0.0),
        51 => (0.0, 0.0),
        52 => (45.0, 0.0),
        53 => (-10.0, 0.0),
        55 => (15.0, 0.0),
        57 => (0.0, 10.0),
        59 => (0.0, 25.0),
        61 => (0.0, -15.0),
        65 => (60.0, 30.0),
        99 => (60.0, 30.0),
        66 => (-20.0, 0.0),
        69 => (lerp_f(-50.0, 20.0, t), lerp_f(0.0, -10.0, t)),
        70 => (0.0, lerp_i(10.0, 30.0, t)),
        71 => (0.0, lerp_i(0.0, 20.0, t)),
        72 => (-10.0, 0.0),
        98 => (-10.0, 0.0),
        101 => (-10.0, 0.0),
        73 => (lerp_f(-10.0, 30.0, t), 0.0),
        74 => (20.0, 0.0),
        77 => (20.0, 0.0),
        76 => (40.0, 0.0),
        81 => (0.0, lerp_f(0.0, 20.0, t)),
        84 => (20.0, 0.0),
        86 => (0.0, lerp_f(0.0, -10.0, t)),
        87 => (0.0, lerp_f(0.0, -10.0, t)),
        88 => (20.0, lerp_f(0.0, -10.0, t)),
        91 => (50.0, 0.0),
        96 => (0.0, 30.0),
        97 => (0.0, 0.0),
        _ => (0.0, 0.0),
    }
}

const CHAR_UNKNOWN: &[i32] = &[3, 16, 17, 22, 25, 26, 27, 31, 32, 41, 48, 49, 94];

const CHAR_FINE_DX: &[(i32, f64)] = &[
    (0, 140.0),
    (3, 20.0),
    (10, 140.0),
    (11, 140.0),
    (19, 140.0),
    (23, 180.0),
    (30, 180.0),
    (34, 60.0),
    (47, 100.0),
    (71, 20.0),
    (32, -120.0),
    (65, -120.0),
    (68, -120.0),
    (89, -80.0),
    (70, -40.0),
    (74, -80.0),
    (55, -80.0),
    (66, -40.0),
    (76, -40.0),
    (5, -80.0),
    (24, -40.0),
    (69, -60.0),
    (4, 80.0),
    (99, 40.0),
    (17, -40.0),
    (54, -40.0),
    (58, -80.0),
    (56, -40.0),
    (27, -80.0),
    (93, -40.0),
    (73, 40.0),
    (46, 40.0),
    (79, 400.0),
    (80, 360.0),
    (16, 40.0),
    (31, 40.0),
    (83, 40.0),
    (82, 40.0),
    (61, -40.0),
];

const CHAR_FINE_DY: &[(i32, f64)] = &[
    (14, 40.0),
    (16, 80.0),
    (55, -120.0),
    (2, -60.0),
    (35, -120.0),
    (56, -80.0),
    (68, -60.0),
    (82, -60.0),
    (98, -80.0),
    (101, -600.0),
    (5, -60.0),
    (69, -100.0),
    (30, -20.0),
    (33, -80.0),
    (61, -120.0),
    (71, -40.0),
    (93, -40.0),
    (81, -40.0),
    (86, -40.0),
    (94, -40.0),
    (18, -40.0),
    (77, -60.0),
    (39, -40.0),
    (24, -60.0),
    (27, -40.0),
    (36, -160.0),
    (85, -80.0),
    (79, 40.0),
    (87, -20.0),
    (43, -40.0),
    (102, -20.0),
    (80, 120.0),
];

const CHAR_ART_DX: &[(&str, f64)] = &[
    ("野乃香_nonoka_89u", 120.0),
    ("光_hikari_0u", -160.0),
    ("叶永_kanae_30u", -200.0),
    ("摩耶_maya_71u", -40.0),
    ("依莉丝_ilith_10u", -120.0),
    ("露娜_luna_12u", -40.0),
    ("忘却_lethe_4u", -80.0),
    ("どろしー_doroc_34u", -120.0),
    ("摩耶_maya_71l", -40.0),
    ("调_shirabe_13", 40.0),
    ("咲弥_saya_23u", -40.0),
    ("爱托_eto_11u", -40.0),
    ("彩梦_ayu_19u", -40.0),
    ("轩辕十四_mda21_29u", -40.0),
    ("伊莎贝尔_isabelle_43u", 40.0),
    ("对立_tairitsu_1l", 240.0),
    ("对立_tairitsu_1o", 40.0),
    ("71u_s1", -80.0),
    ("红_kou_2", 40.0),
    ("野乃香_nonoka_89", -40.0),
];

const CHAR_ART_DY: &[(&str, f64)] = &[
    ("摩耶_maya_71u", -200.0),
    ("依莉丝_ilith_10u", -100.0),
    ("野乃香_nonoka_89u", -120.0),
    ("摩耶_maya_71l", -40.0),
    ("露娜_luna_12u", -40.0),
    ("忘却_lethe_4u", -40.0),
    ("爱托_eto_11u", -40.0),
    ("彩梦_ayu_19u", -60.0),
    ("轩辕十四_mda21_29u", -40.0),
    ("赛润妮·雪儿_selene_82u", -100.0),
    ("71u_s1", -120.0),
    ("希尔_seele_42u", -40.0),
    ("露恩_luin_73u", -40.0),
    ("榛名_haruna_26u", -40.0),
    ("对立_tairitsu_1l", 60.0),
    ("布丽兰特_brilliante_36", 100.0),
    ("咲弥_saya_23u", 20.0),
    ("露娜_luna_12", 20.0),
    ("红_kou_2", 0.0),
    ("对立（Tempest）_tairitsu tempest_35", -40.0),
];

const CHAR_ART_SCALE_TABLE: &[(&str, f64)] = &[("红_kou_2", 1.1)];

fn keyed_value(table: &[(i32, f64)], key: i32) -> f64 {
    table
        .iter()
        .find(|(id, _)| *id == key)
        .map(|(_, value)| *value)
        .unwrap_or(0.0)
}

fn named_value(table: &[(&str, f64)], key: &str) -> f64 {
    table
        .iter()
        .find(|(name, _)| *name == key)
        .map(|(_, value)| *value)
        .unwrap_or(0.0)
}

fn art_dx_of(art: &str) -> f64 {
    named_value(CHAR_ART_DX, art)
}

fn art_dy_of(art: &str) -> f64 {
    named_value(CHAR_ART_DY, art)
}

fn art_scale_of(art: &str) -> Option<f64> {
    CHAR_ART_SCALE_TABLE
        .iter()
        .find(|(name, _)| *name == art)
        .map(|(_, value)| *value)
}

pub fn char_offset_known(id: i32) -> bool {
    (0..=102).contains(&id) && !CHAR_UNKNOWN.contains(&id)
}

pub fn char_offset(id: i32, height: f32) -> (f32, f32) {
    let t = view_t(height as f64);
    let (ax, ay) = char_offset_a(id, t);
    let (bx, by) = char_offset_b(id, t);
    ((ax + bx) as f32, (ay + by) as f32)
}

pub fn char_id_from_key(key: &str) -> Option<i32> {
    let base = key.rsplit('/').next().unwrap_or(key);
    let mut id = 0i32;
    let mut seen = false;
    for ch in base.chars() {
        if ch.is_ascii_digit() {
            id = id * 10 + (ch as i32 - '0' as i32);
            seen = true;
            continue;
        }
        if seen {
            break;
        }
    }
    if seen {
        Some(id)
    } else {
        None
    }
}

pub fn char_art_shift(key: &str) -> (f32, f32, f32) {
    let art = stem_of(key);
    (
        art_dx_of(&art) as f32,
        art_dy_of(&art) as f32,
        art_scale_of(&art).unwrap_or(1.0) as f32,
    )
}

pub fn partner_offset(key: &str, height: f32) -> (f32, f32) {
    let id = char_id_from_key(key).unwrap_or(-1);
    let (auto_x, auto_y) = char_offset(id, height);
    let art = stem_of(key);
    (
        auto_x + keyed_value(CHAR_FINE_DX, id) as f32 + art_dx_of(&art) as f32,
        auto_y + keyed_value(CHAR_FINE_DY, id) as f32 + art_dy_of(&art) as f32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::harness;
    use crate::cards::CardKind;
    use chrono::TimeZone;

    const JACKET: &str = "jackets/abstrusedilemma.jpg";

    fn sample_input(jacket: Option<&str>) -> ScoreInput {
        let mut input = ScoreInput::new("Abstruse Dilemma", 9_978_342)
            .with_badge(
                PlayerBadge::new("Afkeru")
                    .with_ptt(12.31)
                    .with_star(8)
                    .with_subtitle("国际服 · 好友码 431343680")
                    .with_avatar("char_icons/对立_tairitsu_1_icon.png")
                    .with_banner("banners/course_banner_01.png")
                    .with_character("characters/咲弥_saya_23.png"),
            )
            .with_song("abstrusedilemma")
            .with_artist("あおいしげる")
            .with_difficulty(Difficulty::Future, 10.6)
            .with_judgement(1302, 24, 3)
            .with_perfect(1180, 122)
            .with_rating(12.83)
            .with_ptt(12.31)
            .with_delta(0.03, 12.28)
            .with_clear(3)
            .with_skill("咲弥（穿透）")
            .with_play_at(
                Utc.with_ymd_and_hms(2026, 11, 22, 21, 9, 0)
                    .single()
                    .expect("时间"),
            )
            .with_note("第 12 次游玩");
        if let Some(key) = jacket {
            input = input.with_jacket(key);
        }
        input
    }

    fn render_score(input: &ScoreInput) -> Vec<u8> {
        let harness = harness::Harness::new();
        harness.render(score_request(input), |kit| paint_score(kit, input))
    }

    fn render_recent(input: &ScoreInput) -> Vec<u8> {
        let harness = harness::Harness::new();
        harness.render(recent_request(input), |kit| paint_recent(kit, input))
    }

    fn pixels_of(input: &ScoreInput) -> harness::Pixels {
        harness::decode(&render_score(input)).expect("解码成绩卡")
    }

    fn differing(
        left: &harness::Pixels,
        right: &harness::Pixels,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        step: u32,
    ) -> usize {
        let mut count = 0;
        let mut py = y;
        while py < (y + height).min(left.height).min(right.height) {
            let mut px = x;
            while px < (x + width).min(left.width).min(right.width) {
                if left.at(px, py) != right.at(px, py) {
                    count += 1;
                }
                px += step;
            }
            py += step;
        }
        count
    }

    fn rect_of(area: Rect) -> (i32, i32, i32, i32) {
        (
            area.left() as i32,
            area.top() as i32,
            area.right() as i32,
            area.bottom() as i32,
        )
    }

    #[test]
    fn case_228() {
        assert_eq!(UNIT, 1.575);
        assert_eq!(dr(DESIGN_W) as i32, CARD_WIDTH);
        assert_eq!(dr(SCORE_VIEW_H) as i32, CARD_HEIGHT);
        assert!((SCORE_VIEW_H - 846.3).abs() < 1e-5);
        assert_eq!(dr(880.0), 1386.0);
        assert_eq!(dqy(390.0), 599.0);
        assert_eq!(dvy(456.0), 665.0);
        assert_eq!(dqy(452.93), 500.0);
        assert_eq!(dr(TOP_STRIP_LEFT), -221.0);
        assert_eq!(dvy(TITLE_ROW), 240.0);
        assert_eq!(dvy(ARTIST_ROW), 316.0);
    }

    #[test]
    fn case_229() {
        for jacket in [Some(JACKET), None] {
            let input = sample_input(jacket);
            assert_eq!(input.height(), CARD_HEIGHT);
            let bytes = render_score(&input);
            assert_eq!(&bytes[..4], b"\x89PNG", "成绩卡必须是 PNG");
            assert_eq!(
                harness::image_size(&bytes),
                Some((CARD_WIDTH as u32, CARD_HEIGHT as u32))
            );
        }
        let recent = render_recent(&sample_input(None));
        assert_eq!(
            harness::image_size(&recent),
            Some((CARD_WIDTH as u32, CARD_HEIGHT as u32))
        );
    }

    #[test]
    fn case_230() {
        assert_eq!(
            rect_of(crect(30.0, 452.93, 375.0, 375.0)),
            (47, 500, 638, 1091)
        );
        assert_eq!(dr(GAUGE_X), 638.0);
        assert_eq!(dr(GAUGE_RIGHT), 690.0);
        assert_eq!(rect_of(gauge_body_fit(487.0)), (638, 500, 690, 1091));
        assert!((GAUGE_H - 374.99).abs() < 0.01);
        assert_eq!(dqy(GAUGE_TOP_Y), 500.0);
        assert_eq!(dr(GAUGE_X + GAUGE_W / 2.0), 664.0);
    }

    #[test]
    fn case_231() {
        assert_eq!(
            rect_of(crect(437.86, 469.12, 691.0 * 0.66, 394.0 * 0.66)),
            (690, 475, 1408, 884)
        );
        assert_eq!(
            rect_of(ccenter(
                135.0,
                492.36,
                391.0 * JUDGE_ART_SCALE,
                110.0 * JUDGE_ART_SCALE
            )),
            (7, 380, 418, 496)
        );
        assert_eq!(dqy(491.893), 439.0);
        assert_eq!(dqy(501.789), 423.0);
        assert_eq!(dqy(504.267), 419.0);
        assert_eq!(dqy(477.0), 462.0);
        assert_eq!(dr(96.0), 151.0);
        assert_eq!(dr(525.0), 827.0);
        assert_eq!(dr(631.0), 994.0);
        assert_eq!(dr(701.0), 1104.0);
        assert_eq!(dr(631.0 + 12.0), 1013.0);
    }

    #[test]
    fn case_232() {
        assert_eq!(
            rect_of(ccenter(
                623.0,
                186.0,
                249.0 * JUDGE_ART_SCALE,
                45.0 * JUDGE_ART_SCALE
            )),
            (850, 897, 1112, 944)
        );
        assert_eq!(
            rect_of(ccenter(
                623.0,
                149.0,
                248.0 * JUDGE_ART_SCALE,
                43.0 * JUDGE_ART_SCALE
            )),
            (851, 956, 1111, 1001)
        );
        assert_eq!(
            rect_of(ccenter(
                623.0,
                186.0,
                367.0 * JUDGE_ART_SCALE,
                56.0 * JUDGE_ART_SCALE
            )),
            (788, 891, 1174, 950)
        );
        assert_eq!(dqy(186.0), 921.0);
        assert_eq!(dqy(149.0), 979.0);
        assert_eq!(dqy(112.5), 1036.0);
        assert_eq!(dr(700.0), 1103.0);
        assert_eq!(dr(709.0), 1117.0);
        assert_eq!(dr(14.0), 22.0);
        assert_eq!(dr(2.0), 3.0);
    }

    #[test]
    fn case_233() {
        assert_eq!(
            rect_of(srect(656.0, 39.0, 701.0, 82.0)),
            (1033, 61, 1104, 129)
        );
        assert_eq!(dr(678.5), 1069.0);
        assert_eq!(dr(60.5), 95.0);
        assert_eq!(
            crate::cards::avatar::resolved_frame(Some(3.20), None),
            Some(0)
        );
        assert_eq!(
            crate::cards::avatar::resolved_frame(Some(13.25), None),
            Some(7)
        );
        assert!(rating_badge_key(None, None).ends_with("rating_off.png"));
        assert!(rating_badge_key(Some(8), Some(10.0)).ends_with("rating_8.png"));
        assert!(rating_badge_key(None, Some(13.25)).ends_with("rating_7.png"));
    }

    #[test]
    fn case_234() {
        assert_eq!(
            rect_of(srect(PLATE_LEFT, 0.0, PLATE_RIGHT, TOP_BAND_H)),
            (441, 0, 1088, 79)
        );
        assert_eq!(dr(PLATE_PEAK), 49.0);
        assert_eq!(
            rect_of(srect(
                AVATAR_CX - AVATAR_SIZE / 2.0,
                AVATAR_CY - AVATAR_SIZE / 2.0,
                AVATAR_CX + AVATAR_SIZE / 2.0,
                AVATAR_CY + AVATAR_SIZE / 2.0
            )),
            (962, -13, 1088, 113)
        );
        assert_eq!(dr((PLATE_LEFT + PLATE_RIGHT) / 2.0), 765.0);
    }

    #[test]
    fn case_235() {
        let strip = ltrb(
            dr(TOP_STRIP_LEFT),
            dr(0.0),
            dr(TOP_STRIP_LEFT) + dr(1561.0),
            dr(69.0),
        );
        assert_eq!(rect_of(strip), (-221, 0, 2238, 109));
        assert_eq!(
            rect_of(srect(
                1171.45 - 217.1 / 2.0,
                SCORE_VIEW_H - 68.3,
                1171.45 + 217.1 / 2.0,
                SCORE_VIEW_H
            )),
            (1674, 1225, 2016, 1333)
        );
        assert_eq!(
            rect_of(srect(
                107.05 - 216.1 / 2.0,
                SCORE_VIEW_H - 68.0,
                107.05 + 216.1 / 2.0,
                SCORE_VIEW_H
            )),
            (-2, 1226, 339, 1333)
        );
        assert_eq!(dr(SCORE_VIEW_H - 42.0), 1267.0);
    }

    #[test]
    fn case_236() {
        assert_eq!(
            rect_of(ccenter(
                TITLE_CX,
                VIEW_MID - 312.0,
                600.0,
                600.0 * 77.0 / 839.0
            )),
            (536, 394, 1481, 481)
        );
        assert_eq!(
            rect_of(ccenter(
                TITLE_CX,
                VIEW_MID - 312.0,
                600.0,
                600.0 * 103.0 / 837.0
            )),
            (536, 380, 1481, 496)
        );
    }

    #[test]
    fn case_237() {
        assert_eq!(
            rect_of(ccenter(640.0, 278.55, 100.0 * 229.0 / 111.0, 100.0)),
            (846, 696, 1170, 853)
        );
    }

    #[test]
    fn case_238() {
        assert_eq!(score_text(9_978_342), "09'978'342");
        assert_eq!(score_text(10_000_000), "10'000'000");
        assert_eq!(score_text(0), "00'000'000");
        assert_eq!(score_text(123), "00'000'123");
        assert_eq!(split_ptt(12.31), (String::from("12"), String::from(".310")));
    }

    #[test]
    fn case_239() {
        let base = sample_input(Some(JACKET));
        let lost = base.clone().with_clear(0);
        let cleared = base.clone().with_clear(3);
        assert!(!lost.cleared());
        assert!(cleared.cleared());
        let lost_pixels = pixels_of(&lost);
        let cleared_pixels = pixels_of(&cleared);
        let changed = differing(&lost_pixels, &cleared_pixels, 638, 500, 52, 590, 2);
        assert!(
            changed > 200,
            "未通关与通关的回忆条必须不同，采样到 {changed} 个不同的点"
        );
        assert!(
            cleared_pixels.ink_ratio(638, 500, 52, 590, 2) > 0.05,
            "通关时回忆条必须有墨迹"
        );
        let empty = pixels_of(&sample_input(None));
        assert!(
            empty.ink_ratio(0, 0, 2016, 1333, 24) > 0.03,
            "缺曲绘也不能空白"
        );
    }

    #[test]
    fn case_240() {
        let mut input = sample_input(Some(JACKET));
        input.clear_type = None;
        assert!(!input.has_clear());
        let empty = pixels_of(&input);
        let full = pixels_of(&sample_input(Some(JACKET)));
        assert!(
            differing(&empty, &full, 536, 380, 945, 110, 3) > 50,
            "缺通关类型时横幅那一带必须更空"
        );
    }

    #[test]
    fn case_241() {
        let plain = sample_input(Some(JACKET));
        let mut themed = plain.clone();
        themed.theme = Theme::by_name("arknights");
        let left = pixels_of(&plain);
        let right = pixels_of(&themed);
        let changed = differing(&left, &right, 0, 0, 2016, 1333, 24);
        assert!(
            changed > 300,
            "换主题后画面必须真的变了，采样到 {changed} 个不同的点"
        );
        assert!(
            right.ink_ratio(690, 475, 718, 409, 4) > 0.05,
            "明日方舟主题的成绩面板必须画上去"
        );
        assert!(
            left.ink_ratio(840, 880, 290, 200, 3) > 0.02,
            "判定行必须有墨迹"
        );
        assert_eq!(
            judge_key(Theme::by_name("arknights"), "pure"),
            "themes/arknights/result/judge/pure.png"
        );
        assert_eq!(judge_key(Theme::DEFAULT, "pure"), "result/judge/pure.png");
        assert_eq!(panel_ink(Theme::by_name("arknights")), (46.0, 232.0, 170.0, 198.0));
        assert_eq!(panel_word_dx(Theme::by_name("arknights")), 12.0);
        assert_eq!(panel_ink(Theme::DEFAULT), (20.0, 261.0, 156.0, 209.0));
        assert_eq!(panel_word_dx(Theme::DEFAULT), 0.0);
    }

    #[test]
    fn case_242() {
        let input = sample_input(Some(JACKET));
        let recent = render_recent(&input);
        assert_eq!(
            harness::image_size(&recent),
            Some((CARD_WIDTH as u32, CARD_HEIGHT as u32))
        );
        let recent_pixels = harness::decode(&recent).expect("解码最近结算");
        let score_pixels = pixels_of(&input);
        let changed = differing(&score_pixels, &recent_pixels, 0, 0, 2016, 1333, 8);
        assert!(
            changed > 20,
            "两张脸只在顶栏页标题上不同，采样到 {changed} 个不同的点"
        );
        assert_eq!(Face::Score.page_title(), "Player Score");
        assert_eq!(Face::Recent.page_title(), "Player Recent");
    }

    #[test]
    fn case_243() {
        let input = sample_input(Some(JACKET));
        let score = score_card(input.clone());
        assert_eq!(score.request.width, CARD_WIDTH);
        assert_eq!(score.request.height, CARD_HEIGHT);
        assert_eq!(score.label, SCORE_FILE);
        let recent = recent_card(input);
        assert_eq!(recent.request.height, CARD_HEIGHT);
        assert_eq!(recent.label, RECENT_FILE);
        let bare = score_card(sample_input(None));
        assert_eq!(bare.request.height, CARD_HEIGHT);
        assert!(CardKind::Score.spec().accepts(CARD_WIDTH, CARD_HEIGHT));
    }

    #[test]
    fn case_244() {
        assert_eq!(char_id_from_key("characters/咲弥_saya_23.png"), Some(23));
        assert_eq!(
            char_id_from_key("characters/阿米娅_amiya_102u.png"),
            Some(102)
        );
        assert_eq!(char_id_from_key("71u_s1.png"), Some(71));
        assert_eq!(char_id_from_key("characters/无编号.png"), None);
        assert!(char_offset_known(23));
        assert!(char_offset_known(102));
        assert!(!char_offset_known(3));
        assert!(!char_offset_known(94));
        assert!(!char_offset_known(200));
        let (dx, dy) = char_offset(2, VIEW_H as f32);
        assert_eq!(dx, 100.0);
        assert_eq!(dy, 21.0);
        assert_eq!(char_offset(3, VIEW_H as f32), (0.0, 0.0));
        let (ax, ay) = partner_offset("characters/咲弥_saya_23.png", VIEW_H as f32);
        assert!((ax - 207.037).abs() < 0.01, "实际 {ax}");
        assert_eq!(ay, 0.0);
        let (kx, ky) = partner_offset("characters/红_kou_2.png", VIEW_H as f32);
        assert_eq!(kx, 140.0);
        assert_eq!(ky, -39.0);
        assert_eq!(char_art_shift("characters/无此角色.png").2, 1.0);
        assert_eq!(char_art_shift("characters/红_kou_2.png"), (40.0, 0.0, 1.1));
        assert!((view_t(VIEW_H) - 0.70370).abs() < 1e-4);
    }

    #[test]
    fn case_245() {
        assert_eq!(CHAR_FINE_DX.len(), 39);
        assert_eq!(CHAR_FINE_DY.len(), 32);
        assert_eq!(CHAR_ART_DX.len(), 20);
        assert_eq!(CHAR_ART_DY.len(), 20);
        assert_eq!(CHAR_ART_SCALE_TABLE.len(), 1);
        assert_eq!(CHAR_UNKNOWN.len(), 13);
        let known = (0..=102).filter(|id| char_offset_known(*id)).count();
        assert_eq!(known, 90);
    }

    #[test]
    fn case_246() {
        let plain = gauge_skin_of(Some(1), None);
        assert_eq!(plain.bar, "base/hp_bar.png");
        assert_eq!(plain.bar_clear, "base/hp_bar_clear.png");
        let hard = gauge_skin_of(Some(5), None);
        assert_eq!(hard.bar, "gauge/hp_bar_hard.png");
        assert_eq!(hard.bar_clear, "");
        assert_eq!(hard.top, "base/hp_top_red.png");
        let easy = gauge_skin_of(Some(4), None);
        assert_eq!(easy.bar, "gauge/hp_bar_easy.png");
        let chuni = gauge_skin_of(Some(5), Some(24));
        assert_eq!(chuni.bar, "gauge/hp_bar_hard.png");
        let chuni_plain = gauge_skin_of(Some(1), Some(24));
        assert_eq!(chuni_plain.bar, "char/chuni/hp_bar_chuni.png");
        let saya = gauge_skin_of(Some(1), Some(23));
        assert_eq!(saya.bar, "char/saya2/hp_bar_saya2_lv1.png");
        assert_eq!(saya.base.len(), 1);
        assert!(saya.base[0].multiply);
        let tempest = gauge_skin_of(Some(0), Some(35));
        assert_eq!(tempest.bar, "char/tempest/hp_bar_tempest_down.png");
        let tempest_clear = gauge_skin_of(Some(1), Some(35));
        assert_eq!(tempest_clear.bar, "char/tempest/hp_bar_tempest_up.png");
        let fatalis = gauge_skin_of(Some(1), Some(55));
        assert_eq!(fatalis.grid, "char/fatalis/hp_bar_fatalis_grid.png");
        assert_eq!(fatalis.overlay.len(), 1);
        assert_eq!(
            gauge_skin_of(Some(1), Some(15)).bar,
            "gauge/hp_bar_overflow.png"
        );
        assert_eq!(
            gauge_skin_of(Some(1), Some(34)).bar,
            "gauge/hp_bar_grade.png"
        );
        assert_eq!(gauge_skin_of(Some(1), Some(78)).overlay.len(), 1);
    }

    #[test]
    fn case_247() {
        assert!((gauge_scale_of(487.0) - 375.0 / 487.0).abs() < 1e-3);
        assert!((gauge_scale_of(730.0) - 375.0 / 730.0).abs() < 1e-3);
        assert!((gauge_scale_of(486.0) - 375.0 / 487.0).abs() < 1e-3);
        let band = gauge_top_band(39.0, 486.0);
        assert!(band.top() < band.bottom());
        assert!(band.left() > 638.0);
        assert!(band.right() < 690.0);
        assert_eq!(rect_of(gauge_fit(39.0, 487.0)), (640, 500, 687, 1091));
    }

    #[test]
    fn case_248() {
        let mut input = sample_input(Some(JACKET));
        input.recollection = None;
        input.clear_type = Some(0);
        assert!(input.recall_ratio() < 0.01);
        assert!(!input.overflow());
        input.clear_type = Some(3);
        assert!((input.recall_ratio() - 1.0).abs() < 1e-9);
        assert!(!input.overflow());
        input.clear_type = Some(2);
        assert!(input.overflow());
        input.clear_type = Some(1);
        input.recollection = Some(120.0);
        assert!(input.overflow());
    }

    #[test]
    fn case_249() {
        assert_eq!(diff_name(Difficulty::Past), "PAST");
        assert_eq!(diff_name(Difficulty::Present), "PRESENT");
        assert_eq!(diff_name(Difficulty::Future), "FUTURE");
        assert_eq!(diff_name(Difficulty::Beyond), "BEYOND");
        assert_eq!(diff_name(Difficulty::Eternal), "ETERNAL");
        assert_eq!(diff_color(Difficulty::Past), Color::rgb(38, 181, 217));
        assert_eq!(diff_color(Difficulty::Beyond), Color::rgb(207, 32, 62));
        assert_eq!(
            recall_asset(Difficulty::Beyond),
            "difficulty/max-recall-beyond.png"
        );
        assert_eq!(
            recall_asset(Difficulty::Eternal),
            "difficulty/max-recall-eternal.png"
        );
    }

    #[test]
    fn case_250() {
        let input = sample_input(Some("jackets/abstrusedilemma.jpg"));
        let keys = jacket_candidates(&input);
        assert_eq!(keys[0], "jackets/abstrusedilemma_ftr.jpg");
        assert_eq!(keys[1], "jackets/abstrusedilemma.jpg");
        let mut bare = sample_input(None);
        bare.jacket = None;
        bare.song_id.clear();
        assert!(jacket_candidates(&bare).is_empty());
    }

    #[test]
    fn case_251() {
        let mut input = sample_input(None);
        input.badge.character = None;
        input.badge.banner = None;
        input.badge.avatar = None;
        input.recollection = None;
        input.clear_type = None;
        input.ptt = None;
        input.badge.ptt = None;
        let bytes = render_score(&input);
        let pixels = harness::decode(&bytes).expect("解码");
        assert_eq!(
            harness::image_size(&bytes),
            Some((CARD_WIDTH as u32, CARD_HEIGHT as u32))
        );
        assert!(
            pixels.ink_ratio(0, 0, 2016, 1333, 24) > 0.03,
            "整幅必须有内容"
        );
        assert!(
            pixels.ink_ratio(441, 0, 647, 120, 4) > 0.05,
            "没有横幅时名牌必须回落到纯色六边形"
        );
        assert!(
            pixels.ink_ratio(962, 0, 126, 126, 3) > 0.02,
            "没有头像时菱形必须回落到占位图"
        );
    }

    #[test]
    fn case_252() {
        let input = sample_input(Some(JACKET));
        let pixels = pixels_of(&input);
        let mut bare = input.clone();
        bare.badge.character = None;
        let bare_pixels = pixels_of(&bare);
        let drawn = differing(&pixels, &bare_pixels, 1000, 60, 1010, 1270, 6);
        assert!(
            drawn > 500,
            "立绘必须真的画在画布右侧，采样到 {drawn} 个不同的点"
        );
        assert!(
            pixels.ink_ratio(690, 475, 718, 409, 4) > 0.05,
            "成绩面板必须压在立绘之上"
        );
        assert!(
            pixels.ink_ratio(47, 500, 591, 591, 6) > 0.05,
            "曲绘必须画上去"
        );
    }

    #[test]
    fn case_253() {
        let mut input = ScoreInput::new("Xterfusion", 9_900_000)
            .with_difficulty(Difficulty::Eternal, 10.4);
        input.level = String::from("10+");
        assert_eq!(level_mark(&input), (String::from("10"), true));
        input.constant = Some(10.7);
        assert_eq!(level_mark(&input), (String::from("10"), true));
        input.constant = Some(10.4);

        input.level = String::from("10");
        assert_eq!(level_mark(&input), (String::from("10"), false));

        input.level.clear();
        assert_eq!(level_mark(&input), (String::from("10"), false));
        input.constant = Some(10.8);
        assert_eq!(level_mark(&input), (String::from("10"), true));
    }
}
