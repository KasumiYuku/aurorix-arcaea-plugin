use std::collections::HashMap;
use std::sync::Mutex;

use aurorix_plugin::RenderCard;
use aurorix_render::skia_safe::{
    color_filters, paint::Style, shaders, surfaces, BlendMode, ClipOp, Image, ImageInfo, ISize,
    Paint, PathBuilder, Point, Rect, RRect, Surface,
};
use aurorix_render::{
    Border, CanvasKit, Color, Gradient, GradientStop, LinearGradient, RenderRequest, TextStyle,
    sampling_for, sampling_linear, sampling_nearest,
};

use super::jacket_pool;
use super::player;
use super::sketch;
use super::song;
use super::stats;
use super::theme;
use super::widgets::{self, PlayerBadge};
use crate::model::Difficulty;

pub const DIFF_WIDTH: i32 = 804;

const MIN_HEIGHT: i32 = 240;
const MAX_HEIGHT: i32 = 12000;

const MARGIN: f32 = 38.0;
const RIGHT: f32 = DIFF_WIDTH as f32 - MARGIN;
const SHEET: f32 = RIGHT - MARGIN;

const HEAD_CY: f32 = 58.0;
const TITLE_SIZE: u32 = 32;
const NOTE_SIZE: u32 = 13;
const NOTE_PITCH: f32 = 24.0;
const RULE_GAP: f32 = 16.0;
const RULE_THICK: f32 = 1.0;
const GRID_GAP: f32 = 24.0;

const COLS: usize = 2;
const CARD_W: f32 = 356.0;
const CARD_H: f32 = 94.0;
const GAP_X: f32 = 16.0;
const GAP_Y: f32 = 12.0;
const STEP_X: f32 = CARD_W + GAP_X;
const STEP_Y: f32 = CARD_H + GAP_Y;
const SHELL_BLEED: f32 = 26.0;
const BACKDROP_DIVISOR: f32 = 6.0;
const SPRITE_SCALE: f32 = 1.5;

const PAD: f32 = 6.0;
const PAD_R: f32 = 14.0;
const CARD_R: f32 = 6.0;
const JACKET_R: f32 = 4.0;
const DECOR_SPAN: f32 = 150.0;
const JACKET_SIDE: f32 = CARD_H - PAD * 2.0;
const THUMB_GAP: f32 = 12.0;
const GRADE_INK_H: f32 = 27.0;
const TEXT_X: f32 = PAD + JACKET_SIDE + THUMB_GAP;

const TAG_W: f32 = 38.0;
const TAG_H: f32 = 20.0;
const TAG_SKEW: f32 = 6.0;
const TAG_SIZE: u32 = 9;

const GRADE_ALPHA: u8 = 195;
const GRADE_CY: f32 = PAD + JACKET_SIDE * 0.62;

const NAME_BASE: f32 = PAD + 11.4;
const NAME_SIZE: u32 = 16;
const CLOCK_SIZE: u32 = 10;
const NAME_GAP: f32 = 12.0;
const SCORE_BASE: f32 = PAD + 38.7;
const SCORE_SIZE: u32 = 20;
const DELTA_GAP: f32 = 8.0;
const DELTA_SIZE: u32 = 11;
const PTT_BASE: f32 = PAD + 59.8;
const PTT_SIZE: u32 = 10;
const JUDGE_BASE: f32 = CARD_H - PAD - 1.1;
const JUDGE_SIZE: u32 = 10;
const JUDGE_GAP: f32 = 9.0;
const LOSS_GAP: f32 = 12.0;
const PILL_GAP: f32 = 8.0;
const PILL_PAD: f32 = 5.0;

const FOOT_GAP: f32 = 28.0;
const BOTTOM: f32 = 28.0;
const FOOT_SIZE: u32 = 11;

const PAPER: Color = Color::new(0xF4, 0xF1, 0xEA, 0xFF);
const GLASS_TOP: Color = Color::new(0xFF, 0xFF, 0xFF, 0xF0);
const GLASS_BOTTOM: Color = Color::new(0xF5, 0xF8, 0xFF, 0xC8);
const GLOW: Color = Color::new(0x1E, 0x32, 0x50, 0x16);
const EDGE_WHITE: Color = Color::new(0xFF, 0xFF, 0xFF, 0xFF);
const EDGE_INNER: Color = Color::new(0xB4, 0xC8, 0xE6, 0x3C);
const DECOR: Color = Color::new(0xC8, 0xD7, 0xF0, 0x19);
const JACKET_EDGE: Color = Color::new(0x00, 0x00, 0x00, 0x19);
const TAG_EDGE: Color = Color::new(0xFF, 0xFF, 0xFF, 0xB4);
const CHIP: Color = Color::new(0xF4, 0xF9, 0xFF, 0xFF);
const ICE: Color = Color::new(0xE8, 0xEE, 0xF5, 0xFF);
const INK: Color = Color::new(0x1E, 0x24, 0x30, 0xFF);
const SCORE_INK: Color = Color::new(0x15, 0x19, 0x22, 0xFF);
const PM_INK: Color = Color::new(0x2B, 0x58, 0x76, 0xFF);
const INK_SECOND: Color = Color::new(0x5A, 0x66, 0x76, 0xFF);
const INK_MUTED: Color = Color::new(0x8C, 0x98, 0xA8, 0xFF);
const LABEL: Color = Color::new(0x7E, 0x94, 0xA8, 0xFF);
const JUDGE_INK: Color = Color::new(0x3A, 0x47, 0x56, 0xFF);
const RULE_LINE: Color = Color::new(0xBE, 0xC6, 0xD0, 0xFF);
const GAIN: Color = Color::new(0x18, 0x80, 0x68, 0xFF);
const DROP: Color = Color::new(0xB8, 0x32, 0x40, 0xFF);
const GAIN_PILL: Color = Color::new(0x18, 0x80, 0x68, 0x19);
const DROP_PILL: Color = Color::new(0xB8, 0x32, 0x40, 0x19);
const WINE: Color = Color::new(0x6B, 0x4C, 0x5B, 0xFF);
const MIST: Color = Color::new(0xF6, 0xF8, 0xFC, 0xDC);

const DESATURATE: [f32; 20] = [
    0.724, 0.250, 0.025, 0.0, 0.0, 0.074, 0.900, 0.025, 0.0, 0.0, 0.074, 0.250, 0.675, 0.0, 0.0,
    0.0, 0.0, 0.0, 1.0, 0.0,
];

#[derive(Debug, Clone)]
pub struct DiffEntry {
    pub song_id: String,
    pub title: String,
    pub difficulty: Difficulty,
    pub constant: f64,
    pub has_constant: bool,
    pub score: i64,
    pub delta: Option<i64>,
    pub ptt: Option<f64>,
    pub ptt_delta: Option<f64>,
    pub loss: Option<f64>,
    pub has_judgement: bool,
    pub perfect: i64,
    pub shiny: i64,
    pub far: i64,
    pub lost: i64,
    pub at: Option<String>,
}

impl DiffEntry {
    pub fn new(song_id: impl Into<String>, title: impl Into<String>, difficulty: Difficulty) -> Self {
        Self {
            song_id: song_id.into(),
            title: title.into(),
            difficulty,
            constant: 0.0,
            has_constant: false,
            score: 0,
            delta: None,
            ptt: None,
            ptt_delta: None,
            loss: None,
            has_judgement: false,
            perfect: 0,
            shiny: 0,
            far: 0,
            lost: 0,
            at: None,
        }
    }

    pub fn with_constant(mut self, constant: Option<f64>) -> Self {
        if let Some(constant) = constant.filter(|value| *value > 0.0) {
            self.constant = constant;
            self.has_constant = true;
        }
        self
    }

    pub fn with_score(mut self, score: i64) -> Self {
        self.score = score;
        self
    }

    pub fn with_delta(mut self, delta: Option<i64>) -> Self {
        self.delta = delta;
        self
    }

    pub fn with_rating(mut self, ptt: Option<f64>) -> Self {
        self.ptt = ptt.filter(|value| *value > 0.0);
        self
    }

    pub fn with_rating_delta(mut self, delta: Option<f64>) -> Self {
        self.ptt_delta = delta.filter(|value| value.abs() > 0.0005);
        self
    }

    pub fn with_loss(mut self, loss: Option<f64>) -> Self {
        self.loss = loss.filter(|value| *value > 0.0);
        self
    }

    pub fn with_judgement(mut self, perfect: i64, shiny: i64, far: i64, lost: i64) -> Self {
        self.has_judgement = perfect + far + lost > 0;
        self.perfect = perfect;
        self.shiny = shiny;
        self.far = far;
        self.lost = lost;
        self
    }

    pub fn with_at(mut self, at: Option<String>) -> Self {
        self.at = at;
        self
    }

    fn jacket_key(&self) -> String {
        thumb_key(&self.song_id, self.difficulty)
    }
}

#[derive(Debug, Clone)]
pub struct DiffInput {
    pub badge: PlayerBadge,
    pub days: u32,
    pub songs: usize,
    pub changes: usize,
    pub ptt_delta: Option<f64>,
    pub created: String,
    pub entries: Vec<DiffEntry>,
}

impl DiffInput {
    pub fn new(badge: PlayerBadge, days: u32) -> Self {
        Self {
            badge,
            days,
            songs: 0,
            changes: 0,
            ptt_delta: None,
            created: String::new(),
            entries: Vec::new(),
        }
    }

    pub fn with_summary(mut self, songs: usize, changes: usize, ptt_delta: Option<f64>) -> Self {
        self.songs = songs;
        self.changes = changes;
        self.ptt_delta = ptt_delta.filter(|value| value.abs() > 0.0005);
        self
    }

    pub fn with_created(mut self, created: impl Into<String>) -> Self {
        self.created = created.into();
        self
    }

    pub fn with_entry(mut self, entry: DiffEntry) -> Self {
        self.entries.push(entry);
        self
    }

    pub fn with_entries(mut self, entries: Vec<DiffEntry>) -> Self {
        self.entries = entries;
        self
    }
    fn art_key(&self) -> String {
        let days = self.days.to_string();
        let songs = self.songs.to_string();
        let changes = self.changes.to_string();
        let parts = [
            "diff",
            self.badge.name.as_str(),
            days.as_str(),
            songs.as_str(),
            changes.as_str(),
        ];
        jacket_pool::ledger_art(&parts)
            .map(|id| format!("jackets/{id}.jpg"))
            .unwrap_or_default()
    }

}

impl Default for DiffInput {
    fn default() -> Self {
        Self::new(PlayerBadge::new("Afkeru"), 7)
    }
}

fn clamp_height(value: f32) -> i32 {
    (value.round() as i32).clamp(MIN_HEIGHT, MAX_HEIGHT)
}

fn area(x: f32, y: f32, w: f32, h: f32) -> Rect {
    widgets::rect(x, y, w, h)
}

fn font_text(size: u32, color: Color) -> TextStyle {
    theme::style(theme::FONT_NARROW, size, color)
}

fn font_display(size: u32, color: Color) -> TextStyle {
    theme::style(theme::FONT_NARROW_BOLD, size, color)
}

fn font_number(size: u32, color: Color) -> TextStyle {
    theme::style(theme::FONT_TITLE, size, color)
}

fn thumb_key(song_id: &str, difficulty: Difficulty) -> String {
    if song_id.is_empty() {
        return String::new();
    }
    let suffix = crate::cards::song::difficulty_suffix(difficulty);
    if !song_id.ends_with(&format!("_{suffix}")) {
        let variant = format!("jacket_thumbs/{song_id}_{suffix}.jpg");
        if crate::assets_root().join(&variant).is_file() {
            return variant;
        }
    }
    format!("jacket_thumbs/{song_id}.jpg")
}

fn apostrophe(value: i64) -> String {
    sketch::thousands(value).replace(',', "'")
}

fn ink_top(kit: &CanvasKit<'_>, style: &TextStyle, sample: &str) -> f32 {
    sketch::ink_box(kit, style, sample).map_or(style.size as f32 * 0.72, |bounds| {
        (-bounds.top()).max(0.0)
    })
}

fn ink_bottom(kit: &CanvasKit<'_>, style: &TextStyle, sample: &str) -> f32 {
    sketch::ink_box(kit, style, sample).map_or(style.size as f32 * 0.20, |bounds| {
        bounds.bottom().max(0.0)
    })
}

fn player_cy() -> f32 {
    let (up, down) = player::span(1.0);
    HEAD_CY - (down - up) / 2.0
}

fn rule_y() -> f32 {
    let (up, down) = player::span(1.0);
    HEAD_CY + (up + down) / 2.0 + RULE_GAP
}

fn grid_top() -> f32 {
    rule_y() + RULE_THICK + GRID_GAP
}

pub fn rows(count: usize) -> usize {
    count.div_ceil(COLS).max(1)
}

pub fn body_height(count: usize) -> f32 {
    match count {
        0 => CARD_H,
        _ => rows(count) as f32 * STEP_Y - GAP_Y,
    }
}

pub fn diff_size(input: &DiffInput) -> (i32, i32) {
    let height = grid_top() + body_height(input.entries.len()) + FOOT_GAP + BOTTOM;
    (DIFF_WIDTH, clamp_height(height))
}

pub fn diff_request(input: &DiffInput) -> RenderRequest {
    let (width, height) = diff_size(input);
    super::harness::card_request(width, height).with_scale(SPRITE_SCALE)
}

pub fn diff_card(input: DiffInput) -> RenderCard {
    let request = diff_request(&input);
    let label = format!("成绩变化 {}x{}", request.width, request.height);
    let cache_key = super::memo::fingerprint("diff", &input);
    let scale = request.effective_scale();
    RenderCard::new(request, label, move |kit| paint(kit, &input, scale))
        .with_cache_key(cache_key)
}

fn snap(value: f32) -> f32 {
    (value * SPRITE_SCALE).round() / SPRITE_SCALE
}

pub fn card_pos(index: usize) -> Rect {
    let col = index % COLS;
    let row = index / COLS;
    area(
        snap(MARGIN + col as f32 * STEP_X),
        snap(grid_top() + row as f32 * STEP_Y),
        CARD_W,
        CARD_H,
    )
}

pub fn paint(kit: &CanvasKit<'_>, input: &DiffInput, scale: f32) {
    let height = diff_size(input).1 as f32;
    let full = area(0.0, 0.0, DIFF_WIDTH as f32, height);
    paint_backdrop(kit, &input.art_key(), full, scale);
    header(kit, input);
    kit.fill_rect(area(MARGIN, rule_y(), SHEET, RULE_THICK), RULE_LINE);
    match input.entries.is_empty() {
        true => {
            let base = grid_top() + 24.0;
            sketch::text(
                kit,
                &font_text(NOTE_SIZE, INK_MUTED),
                "这段时间还没有记录到变化",
                MARGIN,
                base,
                INK_MUTED,
            );
        }
        false => {
            let shell = shell_sprite(kit, scale);
            for (index, entry) in input.entries.iter().enumerate() {
                paint_card(kit, card_pos(index), entry, shell.as_ref(), scale);
            }
        }
    }
    foot(kit, input);
    grain(kit, full, scale);
}

type SpriteKey = (u8, i32, i32, u64);
type SpriteSlots = Mutex<HashMap<SpriteKey, Image>>;

struct SpriteBank {
    slots: std::sync::OnceLock<SpriteSlots>,
}

impl SpriteBank {
    const fn new() -> Self {
        Self {
            slots: std::sync::OnceLock::new(),
        }
    }

    fn bank(&self) -> &SpriteSlots {
        self.slots.get_or_init(|| Mutex::new(HashMap::new()))
    }

    fn take(&self, key: SpriteKey) -> Option<Image> {
        let slots = self.bank().lock().unwrap_or_else(|poison| poison.into_inner());
        slots.get(&key).cloned()
    }

    fn keep(&self, key: SpriteKey, image: &Image) {
        let mut slots = self.bank().lock().unwrap_or_else(|poison| poison.into_inner());
        slots.insert(key, image.clone());
    }
}

static SHELLS: SpriteBank = SpriteBank::new();
static GRADES: SpriteBank = SpriteBank::new();

fn offscreen(width: f32, height: f32) -> Option<Surface> {
    let info = ImageInfo::new_n32_premul(((width.ceil() as i32).max(1), (height.ceil() as i32).max(1)), None);
    surfaces::raster(&info, None, None)
}

fn paint_backdrop(kit: &CanvasKit<'_>, key: &str, full: Rect, scale: f32) {
    let small_w = (full.width() * scale / BACKDROP_DIVISOR).ceil().max(1.0);
    let small_h = (full.height() * scale / BACKDROP_DIVISOR).ceil().max(1.0);
    let Some(mut surface) = offscreen(small_w, small_h) else {
        kit.fill_rect(full, PAPER);
        stats::ledger_art_wash(kit, key, full.width(), full.height());
        kit.fill_rect(full, MIST);
        return;
    };
    {
        let canvas = surface.canvas();
        canvas.clear(PAPER.to_skia());
        canvas.scale((small_w / full.width(), small_h / full.height()));
        let small = CanvasKit::with_chain(canvas, kit.context(), theme::chain());
        stats::ledger_art_wash(&small, key, full.width(), full.height());
        small.fill_rect(full, MIST);
    }
    let image = surface.image_snapshot();
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    kit.canvas().draw_image_rect_with_sampling_options(
        &image,
        None,
        full,
        sampling_linear(),
        &paint,
    );
}

fn shell_sprite(kit: &CanvasKit<'_>, scale: f32) -> Option<Image> {
    let span_w = CARD_W + SHELL_BLEED * 2.0;
    let span_h = CARD_H + SHELL_BLEED * 2.0;
    let pixel_w = (span_w * scale).round() as i32;
    let pixel_h = (span_h * scale).round() as i32;
    let key = (0, pixel_w, pixel_h, 0);
    if let Some(hit) = SHELLS.take(key) {
        return Some(hit);
    }
    let mut surface = offscreen(pixel_w as f32, pixel_h as f32)?;
    {
        let canvas = surface.canvas();
        canvas.scale((scale, scale));
        canvas.translate((SHELL_BLEED, SHELL_BLEED));
        let shell = CanvasKit::with_chain(canvas, kit.context(), theme::chain());
        paint_shell(&shell, area(0.0, 0.0, CARD_W, CARD_H));
    }
    let image = surface.image_snapshot();
    SHELLS.keep(key, &image);
    Some(image)
}

fn blit_shell(kit: &CanvasKit<'_>, sprite: &Image, card: Rect) {
    let dst = area(
        card.left() - SHELL_BLEED,
        card.top() - SHELL_BLEED,
        CARD_W + SHELL_BLEED * 2.0,
        CARD_H + SHELL_BLEED * 2.0,
    );
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    kit.canvas().draw_image_rect_with_sampling_options(
        sprite,
        None,
        dst,
        sampling_nearest(),
        &paint,
    );
}

static THUMBS: SpriteBank = SpriteBank::new();

fn sprite_hash(text: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    hash
}

fn thumb_sprite(kit: &CanvasKit<'_>, entry: &DiffEntry, scale: f32) -> Option<Image> {
    let pixel = (JACKET_SIDE * scale).round() as i32;
    let key = (4, pixel, pixel, sprite_hash(&entry.jacket_key()));
    if let Some(hit) = THUMBS.take(key) {
        return Some(hit);
    }
    let mut surface = offscreen(pixel as f32, pixel as f32)?;
    {
        let canvas = surface.canvas();
        canvas.scale((scale, scale));
        let off = CanvasKit::with_chain(canvas, kit.context(), theme::chain());
        let rect = area(0.0, 0.0, JACKET_SIDE, JACKET_SIDE);
        let asset = entry.jacket_key();
        if asset.is_empty() || off.draw_asset_cover(&asset, rect, JACKET_R).is_err() {
            off.rrect_fill(rect, JACKET_R, ICE);
        }
        off.stroke(rect, JACKET_R, Border::new(JACKET_EDGE, 1.0));
    }
    let image = surface.image_snapshot();
    THUMBS.keep(key, &image);
    Some(image)
}

fn blit_sprite(kit: &CanvasKit<'_>, sprite: &Image, dst: Rect) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    kit.canvas().draw_image_rect_with_sampling_options(
        sprite,
        None,
        dst,
        sampling_nearest(),
        &paint,
    );
}

static GRAIN: SpriteBank = SpriteBank::new();

const GRAIN_TILE: f32 = 512.0;

fn grain_tile(scale: f32) -> Option<Image> {
    let pixel = (GRAIN_TILE * scale).round() as i32;
    let key = (3, pixel, pixel, 0);
    if let Some(hit) = GRAIN.take(key) {
        return Some(hit);
    }
    let mut surface = offscreen(pixel as f32, pixel as f32)?;
    {
        let canvas = surface.canvas();
        canvas.scale((scale, scale));
        let shader = shaders::fractal_noise((0.9, 0.9), 3, 11.0, None::<ISize>)?;
        let mut paint = Paint::default();
        paint.set_shader(shader);
        paint.set_alpha(12);
        canvas.draw_rect(area(0.0, 0.0, GRAIN_TILE, GRAIN_TILE), &paint);
    }
    let image = surface.image_snapshot();
    GRAIN.keep(key, &image);
    Some(image)
}

fn grain(kit: &CanvasKit<'_>, full: Rect, scale: f32) {
    let Some(tile) = grain_tile(scale) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_blend_mode(BlendMode::Multiply);
    let canvas = kit.canvas();
    canvas.save();
    canvas.clip_rect(full, ClipOp::Intersect, true);
    let cols = (full.width() / GRAIN_TILE).ceil() as usize;
    let rows = (full.height() / GRAIN_TILE).ceil() as usize;
    for row in 0..rows {
        for col in 0..cols {
            let dst = area(
                full.left() + col as f32 * GRAIN_TILE,
                full.top() + row as f32 * GRAIN_TILE,
                GRAIN_TILE,
                GRAIN_TILE,
            );
            canvas.draw_image_rect_with_sampling_options(
                &tile,
                None,
                dst,
                sampling_linear(),
                &paint,
            );
        }
    }
    canvas.restore();
}

fn header(kit: &CanvasKit<'_>, input: &DiffInput) {
    let title_style = font_display(TITLE_SIZE, INK);
    let note_style = font_display(NOTE_SIZE, INK_SECOND);
    let head = ink_top(kit, &title_style, "SCORE DIFF");
    let tail = ink_bottom(kit, &note_style, "Ag");
    let title_base = HEAD_CY - (head + NOTE_PITCH + tail) / 2.0 + head;
    let note_base = title_base + NOTE_PITCH;
    sketch::text(
        kit,
        &title_style,
        "SCORE DIFF",
        MARGIN,
        title_base,
        INK,
    );
    let (lead, mark) = note_parts(input);
    let x = MARGIN + sketch::text(kit, &note_style, &lead, MARGIN, note_base, INK_SECOND);
    let ink = ptt_ink(input.ptt_delta);
    sketch::text(kit, &font_number(NOTE_SIZE, ink), &mark, x, note_base, ink);
    let block = player::Block::new(&input.badge, RIGHT, player_cy())
        .with_left(MARGIN)
        .without_shadow();
    player::paint(kit, &block);
}

fn ptt_ink(delta: Option<f64>) -> Color {
    match delta {
        Some(value) if value > 0.0 => GAIN,
        Some(_) => DROP,
        None => INK_MUTED,
    }
}

fn note_parts(input: &DiffInput) -> (String, String) {
    let lead = format!(
        "近 {} 天   ·   {} 首   ·   {} 条变化   ·   ",
        input.days, input.songs, input.changes
    );
    let mark = match input.ptt_delta {
        Some(delta) if delta > 0.0 => format!("PTT ▲ +{delta:.3}"),
        Some(delta) => format!("PTT ▼ {delta:.3}"),
        None => String::from("PTT —"),
    };
    (lead, mark)
}

fn foot(kit: &CanvasKit<'_>, input: &DiffInput) {
    let base = grid_top() + body_height(input.entries.len()) + FOOT_GAP;
    let created = match input.created.is_empty() {
        true => String::from("CREATED AT —"),
        false => format!("CREATED AT {}", input.created),
    };
    sketch::text(
        kit,
        &font_display(FOOT_SIZE, INK_SECOND),
        &created,
        MARGIN,
        base,
        INK_SECOND,
    );
    sketch::text_right(
        kit,
        &font_display(FOOT_SIZE, INK_SECOND),
        "GENERATED BY OKAMIKKO",
        RIGHT,
        base,
        INK_SECOND,
    );
}

fn paint_card(kit: &CanvasKit<'_>, card: Rect, entry: &DiffEntry, shell: Option<&Image>, scale: f32) {
    match shell {
        Some(sprite) => blit_shell(kit, sprite, card),
        None => paint_shell(kit, card),
    }
    let thumb = area(card.left() + PAD, card.top() + PAD, JACKET_SIDE, JACKET_SIDE);
    match thumb_sprite(kit, entry, scale) {
        Some(sprite) => blit_sprite(kit, &sprite, thumb),
        None => {
            kit.rrect_fill(thumb, JACKET_R, ICE);
            kit.stroke(thumb, JACKET_R, Border::new(JACKET_EDGE, 1.0));
        }
    }
    difficulty_badge(kit, thumb, entry);
    let x = card.left() + TEXT_X;
    name_line(kit, card, x, entry);
    score_line(kit, card, x, entry);
    rating_line(kit, card, x, entry);
    judgement(kit, card, x, entry);
    grade_badge(
        kit,
        card.left() + CARD_W - PAD_R,
        card.top() + GRADE_CY,
        entry.score,
        scale,
    );
}

fn paint_shell(kit: &CanvasKit<'_>, card: Rect) {
    let glass = Gradient::Linear(
        LinearGradient::new([
            GradientStop::new(0.0, GLASS_TOP),
            GradientStop::new(1.0, GLASS_BOTTOM),
        ])
        .with_points(
            Point::new(card.left(), card.top()),
            Point::new(card.right(), card.bottom()),
        ),
    );
    kit.drop_shadow_in(card, GLOW, 0.0, 4.0, 6.0, |kit| {
        let _ = kit.fill_gradient(card, CARD_R, &glass);
    });
    let canvas = kit.canvas();
    canvas.save();
    canvas.clip_rrect(
        RRect::new_rect_xy(card, CARD_R, CARD_R),
        ClipOp::Intersect,
        true,
    );
    sketch::fill_polygon(
        kit,
        &[
            (card.right() - DECOR_SPAN, card.bottom()),
            (card.right() - DECOR_SPAN * 0.28, card.bottom()),
            (card.right(), card.bottom() - DECOR_SPAN * 0.72),
            (card.right(), card.bottom() - DECOR_SPAN),
        ],
        DECOR,
    );
    canvas.restore();
    kit.stroke(card, CARD_R, Border::new(EDGE_WHITE, 1.2));
    let inner = Rect::from_ltrb(
        card.left() + 1.6,
        card.top() + 1.6,
        card.right() - 1.6,
        card.bottom() - 1.6,
    );
    kit.stroke(inner, CARD_R - 0.8, Border::new(EDGE_INNER, 1.0));
}

fn gradient_paint(rect: Rect, from: Color, to: Color, style: Style, width: f32) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(style);
    paint.set_stroke_width(width);
    let gradient = LinearGradient::new([
        GradientStop::new(0.0, from),
        GradientStop::new(1.0, to),
    ])
    .with_points(
        Point::new(rect.left(), rect.top()),
        Point::new(rect.right(), rect.bottom()),
    );
    if let Ok(shader) = Gradient::Linear(gradient).shader(rect) {
        paint.set_shader(shader);
    }
    paint
}

fn solid_paint(color: Color, style: Style, width: f32) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(style);
    paint.set_stroke_width(width);
    paint.set_color(color.to_skia());
    paint
}

fn grade_badge(kit: &CanvasKit<'_>, right: f32, cy: f32, score: i64, scale: f32) {
    let key = crate::domain::grade::grade_asset_for_score(score);
    let Ok(texture) = kit.asset(key) else {
        return;
    };
    let Some(image) = texture.image() else {
        return;
    };
    let img_w = image.width() as f32;
    let img_h = image.height() as f32;
    let file = key.rsplit('/').next().unwrap_or(key);
    let (_, min_y, max_x, max_y) = song::grade_ink_box(file).unwrap_or((0.0, 0.0, img_w, img_h));
    let ink_scale = GRADE_INK_H / (max_y - min_y).max(1.0);
    let dst = Rect::from_xywh(
        right - max_x * ink_scale,
        cy - (min_y + max_y) / 2.0 * ink_scale,
        img_w * ink_scale,
        img_h * ink_scale,
    );
    let pixel_w = (dst.width() * scale).round() as i32;
    let pixel_h = (dst.height() * scale).round() as i32;
    let slot = (1, pixel_w, pixel_h, grade_slot(file));
    if let Some(sprite) = GRADES.take(slot) {
        blit_grade(kit, &sprite, dst);
        return;
    }
    let Some(mut surface) = offscreen(pixel_w as f32, pixel_h as f32) else {
        return;
    };
    {
        let canvas = surface.canvas();
        canvas.scale((scale, scale));
        let target = area(0.0, 0.0, dst.width(), dst.height());
        canvas.draw_image_rect_with_sampling_options(
            image,
            None,
            target,
            sampling_for((img_w, img_h), target),
            &grade_paint(),
        );
    }
    let sprite = surface.image_snapshot();
    GRADES.keep(slot, &sprite);
    blit_grade(kit, &sprite, dst);
}

fn grade_slot(file: &str) -> u64 {
    match file {
        "EX+.png" => 0,
        "EX.png" => 1,
        "AA.png" => 2,
        "A.png" => 3,
        "B.png" => 4,
        "C.png" => 5,
        _ => 6,
    }
}

fn grade_paint() -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_alpha(GRADE_ALPHA);
    paint.set_color_filter(color_filters::matrix_row_major(&DESATURATE, None));
    paint
}

fn blit_grade(kit: &CanvasKit<'_>, sprite: &Image, dst: Rect) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_blend_mode(BlendMode::Multiply);
    kit.canvas().draw_image_rect_with_sampling_options(
        sprite,
        None,
        dst,
        sampling_linear(),
        &paint,
    );
}

fn difficulty_badge(kit: &CanvasKit<'_>, thumb: Rect, entry: &DiffEntry) {
    let (top, bottom) = tone_of(entry.difficulty);
    let (right, base) = (thumb.right(), thumb.bottom());
    let points = [
        (right - TAG_W, base - TAG_H),
        (right, base - TAG_H),
        (right, base),
        (right - TAG_W - TAG_SKEW, base),
    ];
    let mut shape = PathBuilder::new();
    shape.move_to(points[0]);
    shape.line_to(points[1]);
    shape.line_to(points[2]);
    shape.line_to(points[3]);
    shape.close();
    let shape = shape.detach();
    let box_ = area(
        right - TAG_W - TAG_SKEW,
        base - TAG_H,
        TAG_W + TAG_SKEW,
        TAG_H,
    );
    let canvas = kit.canvas();
    canvas.save();
    canvas.clip_rrect(
        RRect::new_rect_xy(thumb, JACKET_R, JACKET_R),
        ClipOp::Intersect,
        true,
    );
    canvas.clip_path(&shape, ClipOp::Intersect, true);
    canvas.draw_rect(box_, &gradient_paint(box_, top, bottom, Style::Fill, 0.0));
    canvas.restore();
    let mut edge = PathBuilder::new();
    edge.move_to(points[3]);
    edge.line_to(points[0]);
    edge.line_to(points[1]);
    canvas.draw_path(&edge.detach(), &solid_paint(TAG_EDGE, Style::Stroke, 1.0));
    let text = match entry.has_constant {
        true => format!("{:.1}", entry.constant),
        false => String::from("—"),
    };
    let style = font_number(TAG_SIZE, CHIP);
    let baseline = match kit.measure(&text, &style) {
        Ok(metrics) => base - TAG_H / 2.0 + metrics.baseline - metrics.height / 2.0,
        Err(_) => base - TAG_H / 2.0,
    };
    sketch::text_center(
        kit,
        &style,
        &text,
        right - TAG_W / 2.0 - 1.5,
        baseline,
        CHIP,
    );
}

fn name_line(kit: &CanvasKit<'_>, card: Rect, x: f32, entry: &DiffEntry) {
    let clock = entry.at.as_deref().filter(|value| !value.is_empty());
    let clock_style = font_number(CLOCK_SIZE, INK_SECOND);
    let clock_w = match clock {
        Some(value) => sketch::text_width(kit, &clock_style, value),
        None => 0.0,
    };
    let room = card.left() + CARD_W - PAD_R - clock_w - NAME_GAP - x;
    let style = font_display(NAME_SIZE, INK);
    let title = sketch::ellipsize(kit, &style, &entry.title, room.max(0.0));
    sketch::text(kit, &style, &title, x, card.top() + NAME_BASE, INK);
    if let Some(value) = clock {
        sketch::text_right(
            kit,
            &clock_style,
            value,
            card.left() + CARD_W - PAD_R,
            card.top() + NAME_BASE,
            INK_SECOND,
        );
    }
}

fn score_line(kit: &CanvasKit<'_>, card: Rect, x: f32, entry: &DiffEntry) {
    let ink = match entry.score >= 10_000_000 {
        true => PM_INK,
        false => SCORE_INK,
    };
    let style = font_number(SCORE_SIZE, ink);
    let score = apostrophe(entry.score);
    let span = sketch::text(kit, &style, &score, x, card.top() + SCORE_BASE, ink);
    let (text, tone) = match entry.delta {
        Some(0) => (String::from("±0"), INK_MUTED),
        Some(delta) if delta > 0 => (format!("▲ +{}", apostrophe(delta)), GAIN),
        Some(delta) => (format!("▼ -{}", apostrophe(-delta)), DROP),
        None => (String::from("NEW"), WINE),
    };
    let mark = font_display(DELTA_SIZE, tone);
    let at = x + span + DELTA_GAP;
    if at + sketch::text_width(kit, &mark, &text) > card.left() + CARD_W - PAD_R {
        return;
    }
    let base = card.top() + SCORE_BASE - sketch::ink_center(kit, &style, &score)
        + sketch::ink_center(kit, &mark, &text);
    sketch::text(kit, &mark, &text, at, base, tone);
}

fn rating_line(kit: &CanvasKit<'_>, card: Rect, x: f32, entry: &DiffEntry) {
    let base = card.top() + PTT_BASE;
    let label = font_text(PTT_SIZE, LABEL);
    let value = font_number(PTT_SIZE, INK_SECOND);
    let Some(current) = entry.ptt else {
        sketch::text(kit, &label, "PTT  —", x, base, LABEL);
        return;
    };
    let mut cursor = x + sketch::text(kit, &label, "PTT  ", x, base, LABEL);
    let Some(delta) = entry.ptt_delta else {
        sketch::text(
            kit,
            &value,
            &format!("{current:.3}"),
            cursor,
            base,
            INK_SECOND,
        );
        return;
    };
    let previous = current - delta;
    cursor += sketch::text(
        kit,
        &value,
        &format!("{previous:.3}"),
        cursor,
        base,
        INK_SECOND,
    );
    cursor += sketch::text(kit, &label, "  →  ", cursor, base, LABEL);
    cursor += sketch::text(
        kit,
        &value,
        &format!("{current:.3}"),
        cursor,
        base,
        INK_SECOND,
    );
    let (mark, ink, pill) = match delta > 0.0 {
        true => ("▲", GAIN, GAIN_PILL),
        false => ("▼", DROP, DROP_PILL),
    };
    let text = format!("{mark}{:.3}", delta.abs());
    let style = font_number(PTT_SIZE, ink);
    let pill_box = area(
        cursor + PILL_GAP,
        base - PTT_SIZE as f32 - 1.5,
        sketch::text_width(kit, &style, &text) + PILL_PAD * 2.0,
        PTT_SIZE as f32 + 5.0,
    );
    kit.rrect_fill(pill_box, pill_box.height() / 2.0, pill);
    sketch::text(
        kit,
        &style,
        &text,
        pill_box.left() + PILL_PAD,
        base,
        ink,
    );
}

fn judgement(kit: &CanvasKit<'_>, card: Rect, x: f32, entry: &DiffEntry) {
    let base = card.top() + JUDGE_BASE;
    if !entry.has_judgement {
        let label = font_text(JUDGE_SIZE, LABEL);
        let mut cursor =
            x + sketch::text(kit, &label, "判定数据缺失", x, base, LABEL);
        if let Some(loss) = entry.loss {
            let value = font_number(JUDGE_SIZE, JUDGE_INK);
            cursor += LOSS_GAP;
            cursor += sketch::text(kit, &label, "LS ", cursor, base, LABEL);
            sketch::text(
                kit,
                &value,
                &format!("{loss:.1}"),
                cursor,
                base,
                JUDGE_INK,
            );
        }
        return;
    }
    let label = font_text(JUDGE_SIZE, LABEL);
    let value = font_number(JUDGE_SIZE, JUDGE_INK);
    let mut cursor = x;
    cursor += sketch::text(kit, &label, "P: ", cursor, base, LABEL);
    cursor += sketch::text(
        kit,
        &value,
        &sketch::thousands(entry.perfect),
        cursor,
        base,
        JUDGE_INK,
    );
    let small = entry.perfect - entry.shiny;
    if small > 0 {
        cursor += sketch::text(
            kit,
            &label,
            &format!(" (-{})", sketch::thousands(small)),
            cursor,
            base,
            LABEL,
        );
    }
    cursor += JUDGE_GAP;
    cursor += sketch::text(kit, &label, "F: ", cursor, base, LABEL);
    cursor += sketch::text(
        kit,
        &value,
        &sketch::thousands(entry.far),
        cursor,
        base,
        JUDGE_INK,
    );
    cursor += JUDGE_GAP;
    cursor += sketch::text(kit, &label, "L: ", cursor, base, LABEL);
    cursor += sketch::text(
        kit,
        &value,
        &sketch::thousands(entry.lost),
        cursor,
        base,
        JUDGE_INK,
    );
    if let Some(loss) = entry.loss {
        cursor += LOSS_GAP;
        cursor += sketch::text(kit, &label, "LS ", cursor, base, LABEL);
        sketch::text(
            kit,
            &value,
            &format!("{loss:.1}"),
            cursor,
            base,
            JUDGE_INK,
        );
    }
}

fn tone_of(difficulty: Difficulty) -> (Color, Color) {
    match difficulty {
        Difficulty::Past => (Color::new(0x4A, 0x6B, 0x85, 0xFF), Color::new(0x33, 0x50, 0x6A, 0xFF)),
        Difficulty::Present => (Color::new(0x4B, 0x8A, 0x47, 0xFF), Color::new(0x2E, 0x5C, 0x2B, 0xFF)),
        Difficulty::Future => (Color::new(0x6A, 0x35, 0x7A, 0xFF), Color::new(0x4A, 0x20, 0x56, 0xFF)),
        Difficulty::Eternal => (Color::new(0x55, 0x60, 0x6B, 0xFF), Color::new(0x3E, 0x47, 0x50, 0xFF)),
        Difficulty::Beyond => (Color::new(0x9E, 0x1F, 0x32, 0xFF), Color::new(0x6B, 0x11, 0x1E, 0xFF)),
        Difficulty::Ins => (Color::new(0x3E, 0x44, 0x68, 0xFF), Color::new(0x2A, 0x2F, 0x49, 0xFF)),
    }
}
