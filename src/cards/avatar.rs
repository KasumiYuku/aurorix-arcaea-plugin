use aurorix_render::skia_safe::Rect;
use aurorix_render::{CanvasKit, Color};

use super::sketch;
use super::theme;

pub const BORDER: &str = "avatar/icon_back.png";

pub const PLACEHOLDER: &str = "avatar/unknown_icon.png";

pub const UNKNOWN_CHAR: &str = "char_icons/光_hikari_0_icon.png";

const ICON_UNITS: f32 = 128.0;

const FRAME_UNITS: f32 = 150.0;
const FRAME_HALF_DIAGONAL: f32 = 70.0;

const FRAME_BOX_HALF: f32 = 75.0;

pub fn frame_half_width(radius: f32) -> f32 {
    radius * (FRAME_UNITS / ICON_UNITS) * (FRAME_HALF_DIAGONAL / FRAME_BOX_HALF)
}


const BADGE_UNITS: f32 = 85.0;

const BADGE_OFFSET: f32 = 0.11;

const PTT_FONT_SCALE: f32 = 1.0;

const PTT_DEC_RATIO: f32 = 0.875;

const AVATAR_SOLID: Color = Color::new(0x64, 0x58, 0x7E, 0xFF);

const PTT_FILL: Color = Color::new(246, 249, 255, 255);

const PTT_EDGE: Color = Color::new(0, 0, 0, 255);

const RANK_UNIT: f32 = 2.0 / ICON_UNITS;

const SHAPE_RANKED: &str = "avatar/ranking/usercell_shape_bg.png";

const SHAPE_DIAMOND_RANKED: f32 = 207.0;

const SHAPE_CENTER_RANKED: (f32, f32) = (145.5, 145.0);

const SHAPE_SCALE: f32 = 0.73;

const WING_CENTER: (f32, f32) = (124.0, -27.5);

const WING_UNITS_WIDTH: f32 = 69.0;

const RANK_FONT: f32 = 26.0;

const RANK_INK: Color = PTT_FILL;

const RANK_HASH: &str = "avatar/ranking/hash.png";

const RANK_HASH_UNITS: (f32, f32) = (22.0, 23.0);

const RANK_HASH_SCALE: f32 = 2.0 / 3.0;

const RANK_HASH_GAP: f32 = 1.0;

#[derive(Debug, Clone, Copy)]
pub struct Avatar<'a> {
    pub icon: Option<&'a str>,
    pub ptt: Option<f64>,
    pub frame: Option<i32>,
    pub global_rank: Option<i32>,
    pub digits: bool,
}

impl Default for Avatar<'_> {
    fn default() -> Self {
        Self {
            icon: None,
            ptt: None,
            frame: None,
            global_rank: None,
            digits: true,
        }
    }
}

impl<'a> Avatar<'a> {
    pub fn new(icon: Option<&'a str>) -> Self {
        Self {
            icon,
            ..Self::default()
        }
    }

    pub fn with_ptt(mut self, ptt: Option<f64>) -> Self {
        self.ptt = ptt.filter(|value| *value > 0.0);
        self
    }

    pub fn with_frame(mut self, frame: Option<i32>) -> Self {
        self.frame = frame;
        self
    }

    pub fn with_global_rank(mut self, rank: Option<i32>) -> Self {
        self.global_rank = rank.filter(|value| *value >= 1);
        self
    }

    pub fn without_digits(mut self) -> Self {
        self.digits = false;
        self
    }
}

pub fn draw(kit: &CanvasKit<'_>, cx: f32, cy: f32, radius: f32, avatar: &Avatar<'_>) {
    if radius <= 0.0 {
        return;
    }
    if ranked(avatar.global_rank) {
        shape(kit, cx, cy, radius);
    }
    icon(kit, avatar.icon, cx, cy, radius);
    border(kit, cx, cy, radius);
    if let Some(rank) = avatar.global_rank.filter(|rank| *rank > 0) {
        rank_text(kit, cx, cy, radius, rank);
    }
    badge(kit, avatar, cx, cy, radius);
}

pub fn draw_icon_and_border(
    kit: &CanvasKit<'_>,
    cx: f32,
    cy: f32,
    radius: f32,
    avatar: &Avatar<'_>,
) {
    if radius <= 0.0 {
        return;
    }
    if ranked(avatar.global_rank) {
        shape(kit, cx, cy, radius);
    }
    icon(kit, avatar.icon, cx, cy, radius);
    border(kit, cx, cy, radius);
}

pub fn draw_rank_text_at(kit: &CanvasKit<'_>, cx: f32, cy: f32, radius: f32, rank: i32) {
    if rank < 1 || radius <= 0.0 {
        return;
    }
    rank_text(kit, cx, cy, radius, rank);
}

fn icon(kit: &CanvasKit<'_>, key: Option<&str>, cx: f32, cy: f32, radius: f32) {
    let mut keys: [&str; 3] = [UNKNOWN_CHAR, PLACEHOLDER, PLACEHOLDER];
    let mut count = 2;
    if let Some(key) = key.filter(|key| !key.is_empty()) {
        keys[0] = key;
        count = 3;
    }
    let side = radius * 2.0;
    for key in &keys[..count] {
        let Ok(texture) = kit.asset(key) else {
            continue;
        };
        let Some(image) = texture.image() else {
            continue;
        };
        kit.image_rect(
            image,
            Rect::from_xywh(cx - radius, cy - radius, side, side),
        );
        return;
    }
    sketch::diamond(kit, cx, cy, radius, AVATAR_SOLID);
}

fn border(kit: &CanvasKit<'_>, cx: f32, cy: f32, radius: f32) {
    let side = radius * 2.0 * (FRAME_UNITS / ICON_UNITS);
    let _ = kit.draw_asset(
        BORDER,
        Rect::from_xywh(cx - side / 2.0, cy - side / 2.0, side, side),
        0.0,
    );
}

fn badge(kit: &CanvasKit<'_>, avatar: &Avatar<'_>, cx: f32, cy: f32, radius: f32) {
    let key = rating_key(avatar.ptt, avatar.frame);
    let Some(crop) = sketch::rating_crop(&key) else {
        return;
    };
    let node = radius * 2.0 * (BADGE_UNITS / ICON_UNITS);
    let tex_w = kit
        .asset(&key)
        .ok()
        .and_then(|texture| texture.dimensions())
        .map(|(width, _)| width as f32)
        .filter(|width| *width > 0.0)
        .unwrap_or(crop.width().max(1.0));
    let width = (node * crop.width() / tex_w).round().max(1.0);
    let height = (width * crop.height() / crop.width().max(1.0))
        .round()
        .max(1.0);
    let offset = (radius * BADGE_OFFSET).round();
    let area = Rect::from_xywh(cx + offset, cy + offset, width, height);
    if !sketch::paste_crop_clip(kit, &key, crop, area) {
        return;
    }
    if !avatar.digits {
        return;
    }
    ptt_digits(
        kit,
        area.left() + width / 2.0,
        area.top() + height / 2.0 + (height * 0.02).round(),
        width / 1.4 / 2.2 * PTT_FONT_SCALE,
        avatar.ptt,
    );
}

pub fn resolved_frame(ptt: Option<f64>, frame: Option<i32>) -> Option<i32> {
    frame.filter(|value| *value >= 0).or_else(|| {
        ptt.filter(|value| *value > 0.0)
            .map(|value| i32::from(crate::domain::rating_frame_of(value)))
    })
}

fn rating_key(ptt: Option<f64>, frame: Option<i32>) -> String {
    if ptt.filter(|value| *value > 0.0).is_none() {
        return String::from("avatar/rating/rating_off.png");
    }
    match resolved_frame(ptt, frame) {
        Some(frame) => format!("avatar/rating/rating_{frame}.png"),
        None => String::from("avatar/rating/rating_off.png"),
    }
}

fn shape_scale(radius: f32) -> f32 {
    let diamond = radius * 2.0 * (SHAPE_DIAMOND_RANKED * SHAPE_SCALE / ICON_UNITS);
    (diamond / SHAPE_DIAMOND_RANKED).max(f32::EPSILON)
}

fn ranked(rank: Option<i32>) -> bool {
    rank.is_some_and(|value| value > 0)
}

fn shape(kit: &CanvasKit<'_>, cx: f32, cy: f32, radius: f32) {
    let key = SHAPE_RANKED;
    let center = SHAPE_CENTER_RANKED;
    let Ok(texture) = kit.asset(key) else {
        return;
    };
    let Some((width, height)) = texture.dimensions() else {
        return;
    };
    let s = shape_scale(radius);
    let x = cx - center.0 * s;
    let y = cy - center.1 * s;
    let _ = kit.draw_asset(
        key,
        Rect::from_xywh(x, y, width as f32 * s, height as f32 * s),
        0.0,
    );
}

fn rank_text(kit: &CanvasKit<'_>, cx: f32, cy: f32, radius: f32, rank: i32) {
    if rank < 1 {
        return;
    }
    let unit = radius * RANK_UNIT;
    let s = shape_scale(radius);
    let center_x = cx + WING_CENTER.0 * s;
    let center_y = cy + WING_CENTER.1 * s;
    let hash_w = RANK_HASH_UNITS.0 * RANK_HASH_SCALE * unit;
    let hash_h = RANK_HASH_UNITS.1 * RANK_HASH_SCALE * unit;
    let gap = RANK_HASH_GAP * unit;
    let room = WING_UNITS_WIDTH * unit;
    let text = rank.to_string();
    let mut size = (RANK_FONT * unit).round().max(1.0);
    let mut style = theme::style(theme::FONT_NUMBER, size as u32, RANK_INK);
    let Some(initial) = sketch::ink_box(kit, &style, &text) else {
        return;
    };
    let mut ink = initial;
    let mut text_w = ink.right() - ink.left();
    if text_w > room {
        size = (size * room / text_w.max(1.0)).round().max(1.0);
        style = theme::style(theme::FONT_NUMBER, size as u32, RANK_INK);
        let Some(again) = sketch::ink_box(kit, &style, &text) else {
            return;
        };
        ink = again;
        text_w = ink.right() - ink.left();
    }
    let ink_left = center_x - text_w / 2.0;
    sketch::text(
        kit,
        &style,
        &text,
        ink_left - ink.left(),
        center_y + sketch::ink_center(kit, &style, &text),
        RANK_INK,
    );
    let area = Rect::from_xywh(ink_left - gap - hash_w, center_y - hash_h, hash_w, hash_h);
    let _ = kit.draw_asset(RANK_HASH, area, 0.0);
}
fn split_ptt(ptt: f64) -> (String, String) {
    let text = format!("{ptt:.3}");
    match text.split_once('.') {
        Some((head, tail)) => (head.to_string(), format!(".{tail}")),
        None => (text, String::new()),
    }
}

fn ptt_digits(kit: &CanvasKit<'_>, cx: f32, cy: f32, int_size: f32, ptt: Option<f64>) {
    let has_ptt = ptt.is_some_and(|value| value > 0.0);
    let (head, tail) = if has_ptt {
        split_ptt(ptt.unwrap_or(0.0))
    } else {
        (String::from("--"), String::new())
    };
    let head_size = int_size.round().max(1.0) as u32;
    let head_style = theme::style(theme::FONT_NUMBER, head_size, PTT_FILL);
    let head_width = sketch::text_width(kit, &head_style, &head);
    let left = sketch::ink_box(kit, &head_style, &head)
        .map(|ink| ink.left())
        .unwrap_or(0.0);
    let mut right = head_width;
    let mut runs: Vec<(String, aurorix_render::TextStyle, f32)> =
        vec![(head.clone(), head_style.clone(), 0.0)];
    if !tail.is_empty() {
        let tail_style = theme::style(
            theme::FONT_NUMBER,
            (head_size as f32 * PTT_DEC_RATIO).round().max(1.0) as u32,
            PTT_FILL,
        );
        if let Some(ink) = sketch::ink_box(kit, &tail_style, &tail) {
            right = head_width + ink.right();
        }
        runs.push((tail, tail_style, head_width));
    }
    let origin = cx - (left + right) / 2.0;
    let baseline = cy + sketch::ink_center(kit, &head_style, &head);
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
        for (value, style, advance) in &runs {
            sketch::text(kit, style, value, origin + advance + dx, baseline + dy, PTT_EDGE);
        }
    }
    for (value, style, advance) in &runs {
        sketch::text(kit, style, value, origin + advance, baseline, PTT_FILL);
    }
}
