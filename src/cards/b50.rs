use std::sync::{Mutex, OnceLock};

use aurorix_render::skia_safe::canvas::SrcRectConstraint;
use aurorix_render::skia_safe::paint::Style;
use aurorix_render::skia_safe::{
    images, AlphaType, BlendMode, ClipOp, ColorFilter, ColorType, Data, Image, ImageInfo, Paint,
    PathBuilder, Point, Rect,
};
use aurorix_render::{
    CanvasKit, Color, Gradient, GradientStop, LinearGradient, RadialGradient, RenderRequest,
    TextStyle,
};

use super::avatar;
use super::jacket_pool;
use super::memo::{self, Memo};
use super::sketch::{self, BladeCorners};
use super::theme::{self, Theme};
use super::widgets::{self, PlayerBadge};
use crate::domain::{band_of, BoardKind, BoardRow, OverBand};
use crate::model::Difficulty;

const DESIGN_WIDTH: i32 = 1300;

const DESIGN_HEIGHT: i32 = 1524;

const fn design_px(value: i32) -> i32 {
    (value * 14 + 5) / 10
}

pub const CARD_WIDTH: i32 = design_px(DESIGN_WIDTH);

pub const CARD_HEIGHT: i32 = design_px(DESIGN_HEIGHT);

pub const FILE_NAME: &str = "b50.png";

pub const MAIN_SLOTS: usize = 50;

pub const RECENT_SLOTS: usize = 10;

pub const GRID_COLUMNS: usize = 5;

const RS: f32 = 1.4;

fn sx(value: f32) -> f32 {
    (value * RS).round()
}

fn sf(value: f32) -> f32 {
    value * RS
}

fn font_size(design: f32) -> u32 {
    (design * RS).round().max(1.0) as u32
}

fn style(family: &str, design: f32, ink: Color) -> TextStyle {
    theme::style(family, font_size(design), ink)
}

fn wide(text: &str) -> bool {
    text.chars().any(|ch| ch as u32 > 0x7F)
}

fn has_lower(text: &str) -> bool {
    text.chars().any(|ch| ch.is_ascii_lowercase())
}

const PAPER_A: Color = Color::new(0xF6, 0xF8, 0xFD, 255);

const PAPER_B: Color = Color::new(0xF3, 0xF6, 0xFC, 255);

const PAPER_C: Color = Color::new(0xF1, 0xF4, 0xFB, 255);

const PAPER_D: Color = Color::new(0xEE, 0xF2, 0xFA, 255);

const CARD_TOP: Color = Color::new(0xFF, 0xFF, 0xFF, 255);

const CARD_BOTTOM: Color = Color::new(0xFC, 0xFD, 0xFF, 255);

const CARD_RIM: Color = Color::new(0xC9, 0xCF, 0xDC, 0x9C);

const CARD_GLOW: Color = Color::new(0xFF, 0xFF, 0xFF, 0xF0);

const CARD_SHADOW: Color = Color::new(0x5A, 0x4B, 0x41, 0x22);

const CARD_SHADOW_FAR: Color = Color::new(0x5A, 0x4B, 0x41, 0x12);

const HEADER_SHADOW_FAR: Color = Color::new(28, 24, 48, 46);

const INK_NAME: Color = Color::new(0x25, 0x20, 0x36, 255);

const INK_STAT: Color = Color::new(0x3A, 0x33, 0x4B, 255);

const INK_STAT_LABEL: Color = Color::new(0x6E, 0x65, 0x7C, 255);

const INK_LABEL: Color = Color::new(0x60, 0x57, 0x6E, 255);

const INK_CARD_TITLE: Color = Color::new(0x2A, 0x2F, 0x3D, 255);

const INK_SCORE: Color = Color::new(0x22, 0x1D, 0x33, 255);

const INK_SUB: Color = Color::new(0x5E, 0x56, 0x70, 255);

const INK_FAINT: Color = Color::new(0x8C, 0x86, 0x9B, 255);

const INK_ACCENT: Color = Color::new(0xD1, 0x4F, 0x87, 255);

const INK_GOLD: Color = Color::new(0xA8, 0x76, 0x1A, 255);

const INK_TEAL: Color = Color::new(0x17, 0x79, 0x93, 255);

const PLATE_NAME_LIGHT: Color = Color::new(0xFF, 0xFF, 0xFF, 255);

const JUDGE_P: Color = Color::new(0x3C, 0x7B, 0xA6, 255);

const JUDGE_F: Color = Color::new(0x7E, 0x82, 0x74, 255);

const JUDGE_L: Color = Color::new(0xAC, 0x45, 0x5F, 255);

const CLEAR_TL: Color = Color::new(0x74, 0x16, 0x2C, 255);

const CLEAR_TC: Color = Color::new(0x48, 0x23, 0x48, 255);

const CLEAR_FR: Color = Color::new(0x98, 0x4E, 0x96, 255);

const CLEAR_PM: Color = Color::new(0x22, 0x70, 0x87, 255);

const ART_TRAY: Color = Color::new(0xE6, 0xDF, 0xEE, 255);

const ART_TRAY_IN: Color = Color::new(0x8C, 0x96, 0xBE, 0x30);

const ART_EMPTY: Color = Color::new(0xEF, 0xE7, 0xF4, 255);

const ART_EMPTY_RIM: Color = Color::new(0xD8, 0xCC, 0xE4, 0xC0);

const ART_RIM: Color = Color::new(0xC6, 0xCC, 0xDC, 0x8C);

const ART_INNER_RIM: Color = Color::new(0x2E, 0x26, 0x44, 0x1C);

const ART_GLINT: Color = Color::new(0xFF, 0xFF, 0xFF, 0x70);

const JUDGE_NUM: Color = INK_SUB;

const JUDGE_NOTE: Color = INK_FAINT;

const PLATE_FALLBACK: Color = Color::new(0x1E, 0x1C, 0x28, 0xEE);

const WHITE: Color = Color::new(0xFF, 0xFF, 0xFF, 255);

const VIGNETTE: Color = Color::new(0x8A, 0x7C, 0x94, 0x0A);

const CHAR_GLOW: Color = Color::new(0xD8, 0xE6, 0xFC, 0x5C);

const TICK_DISASTER: Color = Color::new(0x8E, 0x16, 0x29, 255);

const TICK_HIGH: Color = Color::new(0xC4, 0x3B, 0x4A, 255);

const TICK_MID: Color = Color::new(0xD1, 0x76, 0x2A, 255);

const TICK_LOW: Color = Color::new(0x2C, 0x99, 0xB4, 255);

const TICK_CLEARED: Color = Color::new(0x5E, 0x6B, 0x8C, 255);

const ACCENT_VIOLET: Color = Color::new(0x6E, 0x55, 0xC8, 255);

const ACCENT_ROSE: Color = Color::new(0xBE, 0x49, 0x94, 255);

const ACCENT_GREY: Color = Color::new(0x5E, 0x6B, 0x8C, 255);

const ACCENT_GOLD: Color = Color::new(0xC7, 0x96, 0x2C, 255);

const ACCENT_ORANGE: Color = Color::new(0xD1, 0x76, 0x2A, 255);

const ACCENT_RED: Color = Color::new(0xC4, 0x3B, 0x4A, 255);

const ACCENT_TEAL: Color = Color::new(0x2F, 0xA0, 0xBD, 255);

const GRID_X0: f32 = 44.0;

const GRID_DW: f32 = 246.0;

const GRID_CW: f32 = 228.0;

const TOP10_Y0: f32 = 312.0;

const BEST10_Y0: f32 = 312.0 + 2.0 * 116.0;

const CARD_HEIGHT_DESIGN: f32 = 98.0;

const ROW_DH: f32 = 116.0;

const TOP_ROWS: usize = 2;

const BEST_ROWS: usize = 8;

const CARD_PAD: f32 = 15.0;

const CARD_ART: f32 = 66.0;

const CARD_TEXT_X: f32 = 18.0;

const CARD_TEXT_R: f32 = 11.0;

const CARD_RANK_Y: f32 = 20.0;

const CARD_TITLE_Y: f32 = 45.0;

const CARD_SCORE_Y: f32 = 66.0;

const CARD_JUDGE_Y: f32 = 84.0;

const CARD_TICK_Y0: f32 = 35.0;

const CARD_TICK_Y1: f32 = 65.0;

const CARD_TITLE_SIZE: f32 = 13.4;

const CARD_SCORE_MAX: f32 = 19.0;

const CARD_SCORE_MIN: f32 = 13.0;

const CARD_CHIP_TOP: f32 = 10.0;

const CARD_CHIP_BOTTOM: f32 = 24.0;

const CARD_RTL: f32 = 18.0;

const CARD_RTR: f32 = 18.0;

const CARD_RBR: f32 = 18.0;

const CARD_RBL: f32 = 30.0;

const HEADER_BAND_H: f32 = 360.0;

const HEADER_DROP: f32 = 9.0;

const HEADER_OUTDENT: f32 = 9.0;

const HEADER_NAME_OPTICAL: f32 = -17.0;

const HEADER_PLATE_MAX_W: f32 = 600.0;

const HEADER_PLATE_RIGHT_COLS: f32 = 1.62;

const HEADER_PLATE_NAME_SCALE: f32 = 0.76;

const HEADER_PLATE_NAME_MIN_RATIO: f32 = 0.72;

const HEADER_PLATE_TAIL: f32 = 96.0;

const HEADER_PLATE_SLACK: f32 = 4.0;

const HEADER_PLATE_NAME_SHIFT: f32 = 0.6;

const LEDGER_PLATE_H: f32 = 54.0 * 0.75;

const LEDGER_AVATAR_R: f32 = LEDGER_PLATE_H * 1.6 / 2.0;

const LEDGER_PLATE_NAME_SIZE: f32 = LEDGER_PLATE_H * 0.32 / 0.72;

const LEDGER_PLATE_INNER_PAD: f32 = LEDGER_PLATE_H * 0.25;

const LEDGER_PLATE_LEFT_PAD: f32 = LEDGER_PLATE_H * 0.65;

const AVATAR_CX: f32 = 85.0;

const AVATAR_CY: f32 = 152.0;

const AVATAR_R: f32 = 39.0;

const LOGO_X: f32 = 44.0;

const LOGO_Y: f32 = 26.0;

const LOGO_W: f32 = 200.0;

const LOGO_MARK: f32 = 0.64;

const LOGO_PLATE_FROM: f64 = 208.0;

const LOGO_PLATE_TO: f64 = 244.0;

const LOGO_INK_ALPHA: u8 = 96;

const LOGO_INK_OFFSET: f32 = 5.0;

const KICKER_INK_PAD: f32 = 3.0;

const LOGO_RULE_Y: f32 = 92.0;

const STAT_TOP: f32 = 230.0;

const CHAR_TOP: f32 = 12.0;

const CHAR_HMUL: f32 = 1.55;

const CHAR_GAP: f32 = -200.0;

const CHAR_RIGHT_PAD: f32 = 58.0;

const CHAR_VISIBLE_LIFT: f32 = 10.0;

const CHAR_BOTTOM_FADE: f32 = 0.16;

const DEFAULT_CHARACTER: &str = "characters/对立_tairitsu_1u.png";

const FOOTER_BASE_LIFT: f32 = 4.0;

const STAT_LABEL_TOP: f32 = 13.0;

const STAT_VALUE_TOP: f32 = 39.0;

fn grid_x0() -> f32 {
    sx(GRID_X0)
}

fn grid_dw() -> f32 {
    sx(GRID_DW)
}

fn grid_cw() -> f32 {
    sx(GRID_CW)
}

fn card_h() -> f32 {
    sx(CARD_HEIGHT_DESIGN)
}

fn row_dh() -> f32 {
    sx(ROW_DH)
}

fn top10_y0() -> f32 {
    sx(TOP10_Y0)
}

fn best10_y0() -> f32 {
    sx(BEST10_Y0)
}

fn header_avatar_cy() -> f32 {
    sx(AVATAR_CY) + sx(HEADER_DROP)
}

fn avatar_cx() -> f32 {
    sx(AVATAR_CX) - sx(HEADER_OUTDENT)
}

fn avatar_r() -> f32 {
    sx(AVATAR_R)
}
fn plate_peak_at(left: f32) -> f32 {
    let frame_left = avatar_cx() - avatar::frame_half_width(avatar_r());
    header_avatar_cy() + (left - frame_left)
}

fn plate_peak(area: Rect) -> f32 {
    plate_peak_at(area.left()).clamp(area.top(), area.bottom())
}

fn card_corners() -> BladeCorners {
    BladeCorners::new(sf(CARD_RTL), sf(CARD_RTR), sf(CARD_RBR), sf(CARD_RBL))
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    widgets::rect(x, y, width, height)
}

fn solid(ink: Color) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(ink.to_skia());
    paint
}

fn shader_paint(gradient: &Gradient, area: Rect, style: Style, thick: f32) -> Option<Paint> {
    let shader = gradient.shader(area).ok()?;
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(style);
    if thick > 0.0 {
        paint.set_stroke_width(thick);
    }
    paint.set_shader(shader);
    Some(paint)
}

fn fill_vertical(kit: &CanvasKit<'_>, area: Rect, stops: &[(f32, Color)]) {
    let gradient = sketch::gradient_stops(stops);
    if let Some(paint) = shader_paint(&gradient, area, Style::Fill, 0.0) {
        kit.canvas().draw_rect(area, &paint);
    }
}

fn stroke_blade_gradient(
    kit: &CanvasKit<'_>,
    area: Rect,
    corners: BladeCorners,
    thick: f32,
    gradient: &Gradient,
) {
    if let Some(paint) = shader_paint(gradient, area, Style::Stroke, thick) {
        kit.canvas()
            .draw_path(&sketch::blade_path(area, corners), &paint);
    }
}

fn radial_paint(gradient: &Gradient, area: Rect) -> Option<Paint> {
    let shader = gradient.shader(area).ok()?;
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_shader(shader);
    Some(paint)
}

fn plate_path(area: Rect, peak_y: f32) -> aurorix_render::skia_safe::Path {
    let top = area.top();
    let bottom = area.bottom();
    let d_top = (peak_y - top).max(0.0);
    let d_bottom = (bottom - peak_y).max(0.0);
    let mut builder = PathBuilder::new();
    builder.move_to((area.left() + d_top, top));
    builder.line_to((area.right() - d_top, top));
    builder.line_to((area.right(), peak_y));
    builder.line_to((area.right() - d_bottom, bottom));
    builder.line_to((area.left() + d_bottom, bottom));
    builder.line_to((area.left(), peak_y));
    builder.close();
    builder.detach()
}

fn b50_shadow_sigma(radius: f32) -> f32 {
    (radius * 0.18).max(0.6)
}

fn b50_blur_sigma(radius: f32) -> f32 {
    let window = radius * 2.0 + 1.0;
    (((window * window - 1.0) / 12.0).max(0.0)).sqrt()
}

fn b50_mix(ink: Color, next: Color, t: f32) -> Color {
    if t <= 0.0 {
        return ink;
    }
    if t >= 1.0 {
        return next;
    }
    let blend = |a: u8, b: u8| -> u8 {
        (f64::from(a) + (f64::from(b) - f64::from(a)) * f64::from(t)) as u8
    };
    Color::new(
        blend(ink.r, next.r),
        blend(ink.g, next.g),
        blend(ink.b, next.b),
        blend(ink.a, next.a),
    )
}

fn b50_veil(kit: &CanvasKit<'_>, area: Rect, colors: &[Color], grain: bool) {
    if area.width() <= 0.0 || area.height() <= 0.0 || colors.is_empty() {
        return;
    }
    let last = colors.len().saturating_sub(1);
    let top = area.top();
    let height = area.height();
    let mut row = 0;
    while row < height.ceil() as i32 {
        let u = (f64::from(row) + 0.5) / f64::from(height);
        let mut ink = if last == 0 {
            colors[0]
        } else {
            let f = u * f64::from(last as u32);
            let index = (f as usize).min(last);
            if index >= last {
                colors[last]
            } else {
                b50_mix(colors[index], colors[index + 1], (f - index as f64) as f32)
            }
        };
        let absolute = (top + row as f32).round() as i32;
        if grain
            && (absolute as u32).wrapping_mul(0x9E37_79B1) & 0x8000 == 0
            && ink.r > 0
            && ink.g > 0
            && ink.b > 0
        {
            ink = Color::new(ink.r - 1, ink.g - 1, ink.b - 1, ink.a);
        }
        kit.fill_rect(rect(area.left(), top + row as f32, area.width(), 1.0), ink);
        row += 1;
    }
}

fn b50_vignette(kit: &CanvasKit<'_>, width: f32, height: f32) {
    if width <= 0.0 || height <= 0.0 {
        return;
    }
    let band = (height * 0.10).max(1.0);
    let paint_row = |y: f32, depth: f32| {
        let u = (depth / 0.10).min(1.0);
        let alpha = (1.0 - u) * (1.0 - u) * 0x0A as f32;
        if alpha >= 0.5 {
            kit.fill_rect(
                rect(0.0, y, width, 1.0),
                Color::new(VIGNETTE.r, VIGNETTE.g, VIGNETTE.b, alpha as u8),
            );
        }
    };
    let mut row = 0;
    while row < band as i32 {
        paint_row(row as f32, (row as f32 + 0.5) / height);
        paint_row(
            height - band + row as f32,
            (band - row as f32 - 0.5) / height,
        );
        row += 1;
    }
}

fn b50_wash_ellipse(kit: &CanvasKit<'_>, cx: f32, cy: f32, rx: f32, ry: f32, ink: Color) {
    if rx <= 0.0 || ry <= 0.0 || ink.a == 0 {
        return;
    }
    let steps = 32;
    let mut items: Vec<GradientStop> = Vec::with_capacity(steps + 1);
    for index in 0..=steps {
        let t = index as f32 / steps as f32;
        let fall = (1.0 - t) * (1.0 - t);
        let alpha = (f32::from(ink.a) * fall) as u8;
        items.push(GradientStop::new(t, Color::new(ink.r, ink.g, ink.b, alpha)));
    }
    let gradient = Gradient::Radial(RadialGradient::new(items).with_center(Point::new(0.0, 0.0)));
    let unit = rect(-1.0, -1.0, 2.0, 2.0);
    if let Some(paint) = radial_paint(&gradient, unit) {
        let canvas = kit.canvas();
        canvas.save();
        canvas.translate(Point::new(cx, cy));
        canvas.scale((rx, ry));
        canvas.draw_rect(unit, &paint);
        canvas.restore();
    }
}

fn b50_shadow_paths(
    kit: &CanvasKit<'_>,
    paths: &[&aurorix_render::skia_safe::Path],
    ink: Color,
    dx: f32,
    dy: f32,
    radius: f32,
) {
    let sigma = b50_shadow_sigma(radius);
    let mut paint = solid(ink);
    if let Some(filter) = aurorix_render::skia_safe::MaskFilter::blur(
        aurorix_render::skia_safe::BlurStyle::Normal,
        sigma,
        false,
    ) {
        paint.set_mask_filter(filter);
    }
    let canvas = kit.canvas();
    canvas.save();
    canvas.translate((dx, dy));
    for path in paths {
        canvas.draw_path(path, &paint);
    }
    canvas.restore();
}
fn b50_tone_filter(keep: f32, gain: f32) -> ColorFilter {
    let keep = keep.clamp(0.0, 1.0);
    let spread = 1.0 - keep;
    let (lr, lg, lb) = (0.299_f32, 0.587_f32, 0.114_f32);
    let matrix: [f32; 20] = [
        (keep + spread * lr) * gain,
        spread * lg * gain,
        spread * lb * gain,
        0.0,
        0.0,
        spread * lr * gain,
        (keep + spread * lg) * gain,
        spread * lb * gain,
        0.0,
        0.0,
        spread * lr * gain,
        spread * lg * gain,
        (keep + spread * lb) * gain,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
    ];
    aurorix_render::skia_safe::color_filters::matrix_row_major(&matrix, None)
}

fn b50_cover_with(
    kit: &CanvasKit<'_>,
    key: &str,
    area: Rect,
    filter: Option<&ColorFilter>,
    anchor: f32,
) -> bool {
    if area.width() <= 0.0 || area.height() <= 0.0 {
        return false;
    }
    let Ok(texture) = kit.asset(key) else {
        return false;
    };
    let Some(image) = texture.image() else {
        return false;
    };
    let src = if anchor < 0.0 {
        aurorix_render::cover_source_rect(image, area)
    } else {
        let image_w = image.width() as f32;
        let image_h = image.height() as f32;
        let want = area.width() / area.height();
        if image_w / image_h > want {
            let crop_w = image_h * want;
            Rect::from_xywh((image_w - crop_w) / 2.0, 0.0, crop_w, image_h)
        } else {
            let crop_h = image_w / want;
            Rect::from_xywh(
                0.0,
                (image_h - crop_h) * anchor.clamp(0.0, 1.0),
                image_w,
                crop_h,
            )
        }
    };
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    if let Some(filter) = filter {
        paint.set_color_filter(filter.clone());
    }
    kit.canvas().draw_image_rect_with_sampling_options(
        image,
        Some((&src, SrcRectConstraint::Fast)),
        area,
        aurorix_render::sampling_for((src.width(), src.height()), area),
        &paint,
    );
    true
}

fn b50_mask_band(kit: &CanvasKit<'_>, area: Rect, vertical: bool, from: u8, to: u8) {
    if area.width() <= 0.0 || area.height() <= 0.0 {
        return;
    }
    let steps = 8u32;
    let mut items: Vec<GradientStop> = Vec::with_capacity(steps as usize + 1);
    for index in 0..=steps {
        let t = index as f32 / steps as f32;
        let eased = t * t * (3.0 - 2.0 * t);
        let alpha = (f32::from(from) + (f32::from(to) - f32::from(from)) * eased)
            .round()
            .clamp(0.0, 255.0) as u8;
        items.push(GradientStop::new(t, Color::new(255, 255, 255, alpha)));
    }
    let gradient = Gradient::Linear(if vertical {
        LinearGradient::vertical(items)
    } else {
        LinearGradient::horizontal(items)
    });
    let Ok(shader) = gradient.shader(area) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_shader(shader);
    paint.set_blend_mode(BlendMode::DstIn);
    kit.canvas().draw_rect(area, &paint);
}

fn b50_glow_text(
    kit: &CanvasKit<'_>,
    face: &TextStyle,
    value: &str,
    x: f32,
    baseline: f32,
    ink: Color,
    radius: f32,
) {
    if value.is_empty() || radius <= 0.0 || ink.a == 0 {
        return;
    }
    let Some(bounds) = sketch::ink_box(kit, face, value) else {
        return;
    };
    let pad = radius * 2.0;
    let area = Rect::from_ltrb(
        x + bounds.left() - pad,
        baseline + bounds.top() - pad,
        x + bounds.right() + pad,
        baseline + bounds.bottom() + pad,
    );
    let sigma = b50_blur_sigma(radius);
    kit.layer(
        area,
        f32::from(ink.a) / 255.0,
        BlendMode::SrcOver,
        |outer| {
            outer.blur_layer_in(area, sigma, |inner| {
                sketch::text(
                    inner,
                    face,
                    value,
                    x,
                    baseline,
                    Color::new(255, 255, 255, 255),
                );
            });
        },
    );
}

#[derive(Debug, Clone)]
pub struct B50Input {
    pub badge: PlayerBadge,
    pub theme: Theme,
    pub board: Option<BoardKind>,
    pub best: Vec<BoardRow>,
    pub recent: Vec<BoardRow>,
    pub ptt: Option<f64>,
    pub note: String,
    pub generated_at: Option<String>,
    pub rank_frame: Option<i32>,
    pub board_stat: Option<(String, String)>,
    pub anchor_upper: f64,
    pub over_count: Option<usize>,
    pub cn: bool,
    pub best10_avg: Option<f64>,
    pub best50_avg: Option<f64>,
    pub best_count: Option<usize>,
    pub max_ptt: Option<f64>,
}

impl B50Input {
    pub fn new(badge: PlayerBadge, theme: Theme) -> Self {
        Self {
            badge,
            theme,
            board: None,
            best: Vec::new(),
            recent: Vec::new(),
            ptt: None,
            note: String::new(),
            generated_at: None,
            rank_frame: None,
            board_stat: None,
            anchor_upper: 0.0,
            over_count: None,
            cn: false,
            best10_avg: None,
            best50_avg: None,
            best_count: None,
            max_ptt: None,
        }
    }

    pub fn with_board(mut self, board: BoardKind) -> Self {
        self.board = Some(board);
        self
    }

    pub fn with_best(mut self, rows: Vec<BoardRow>) -> Self {
        self.best = rows;
        self
    }

    pub fn with_recent(mut self, rows: Vec<BoardRow>) -> Self {
        self.recent = rows;
        self
    }

    pub fn with_ptt(mut self, ptt: f64) -> Self {
        if ptt > 0.0 {
            self.ptt = Some(ptt);
        }
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = note.into();
        self
    }

    pub fn with_generated_at(mut self, stamp: impl Into<String>) -> Self {
        let stamp = stamp.into();
        if !stamp.is_empty() {
            self.generated_at = Some(stamp);
        }
        self
    }

    pub fn with_rank_frame(mut self, frame: i32) -> Self {
        if frame >= 0 {
            self.rank_frame = Some(frame);
        }
        self
    }

    pub fn with_board_stat(mut self, label: impl Into<String>, value: impl Into<String>) -> Self {
        self.board_stat = Some((label.into(), value.into()));
        self
    }

    pub fn with_anchor_upper(mut self, anchor_upper: f64) -> Self {
        self.anchor_upper = anchor_upper;
        self
    }

    pub fn with_over_count(mut self, over_count: usize) -> Self {
        self.over_count = Some(over_count);
        self
    }

    pub fn with_averages(
        mut self,
        avg_best10: Option<f64>,
        avg_best50: Option<f64>,
        best_count: usize,
    ) -> Self {
        self.best10_avg = avg_best10.filter(|value| *value > 0.0);
        self.best50_avg = avg_best50.filter(|value| *value > 0.0);
        if best_count > 0 {
            self.best_count = Some(best_count);
        }
        self
    }

    pub fn with_max_ptt(mut self, max_ptt: f64) -> Self {
        if max_ptt > 0.0 {
            self.max_ptt = Some(max_ptt);
        }
        self
    }

    pub fn with_cn(mut self, cn: bool) -> Self {
        self.cn = cn;
        self
    }

    pub fn empty_slots(&self) -> usize {
        MAIN_SLOTS.saturating_sub(self.board_rows().len())
    }

    pub fn board_rows(&self) -> Vec<&BoardRow> {
        self.best
            .iter()
            .chain(self.recent.iter())
            .take(MAIN_SLOTS)
            .collect()
    }
}

pub fn request(_input: &B50Input) -> RenderRequest {
    super::harness::card_request(CARD_WIDTH, CARD_HEIGHT)
}

pub fn card(input: B50Input) -> aurorix_plugin::RenderCard {
    let request = request(&input);
    let cache_key = memo::fingerprint("b50-hd2", &input);
    aurorix_plugin::RenderCard::new(request, FILE_NAME, move |kit| paint(kit, &input))
        .with_cache_key(cache_key)
}

fn board_kicker(board: Option<BoardKind>) -> &'static str {
    match board {
        None => "PLAYER BEST 50",
        Some(BoardKind::Pm50) => "PLAYER PURE MEMORY 50",
        Some(BoardKind::Fr50) => "PLAYER FULL RECALL 50",
        Some(BoardKind::X50) => "PLAYER SO CLOSE 50",
        Some(BoardKind::Nm50) => "PLAYER NEAR MAX 50",
        Some(BoardKind::Mx50) => "PLAYER MAX PM 50",
        Some(BoardKind::Ol50) => "PLAYER OVER LEVEL 50",
    }
}

fn accent_rgb(board: Option<BoardKind>) -> Color {
    match board {
        Some(BoardKind::Pm50) => ACCENT_VIOLET,
        Some(BoardKind::Fr50) => ACCENT_ROSE,
        Some(BoardKind::X50) => ACCENT_RED,
        Some(BoardKind::Nm50) => ACCENT_GREY,
        Some(BoardKind::Mx50) => ACCENT_GOLD,
        Some(BoardKind::Ol50) => ACCENT_ORANGE,
        _ => ACCENT_TEAL,
    }
}

fn mix_with_paper(ink: Color, paper: Color, ratio: f64) -> Color {
    let blend = |value: u8, base: u8| -> u8 {
        let mixed = ratio * f64::from(value) + (1.0 - ratio) * f64::from(base);
        mixed.round().clamp(0.0, 255.0) as u8
    };
    Color::new(
        blend(ink.r, paper.r),
        blend(ink.g, paper.g),
        blend(ink.b, paper.b),
        ink.a,
    )
}

fn pure_of(row: &BoardRow) -> i64 {
    (row.object_amount - row.far - row.lost).max(0)
}

fn row_band(row: &BoardRow) -> Option<OverBand> {
    row.band.or_else(|| {
        let known = row.span != 0.0 || row.pressure != 0.0 || row.has_judgement;
        if known {
            band_of(row.span, row.score)
        } else {
            None
        }
    })
}

fn jacket_variant(row: &BoardRow) -> String {
    match row.difficulty {
        Difficulty::Beyond => format!("{}_byd", row.song_id),
        Difficulty::Eternal => format!("{}_etr", row.song_id),
        _ => row.song_id.clone(),
    }
}

const DIFFICULTY_SUFFIXES: [&str; 6] = ["_pst", "_prs", "_ftr", "_byd", "_etr", "_ins"];

fn has_difficulty_suffix(song_id: &str) -> bool {
    DIFFICULTY_SUFFIXES
        .iter()
        .any(|suffix| song_id.ends_with(suffix))
}

fn jacket_key(kit: &CanvasKit<'_>, row: &BoardRow) -> Option<String> {
    jacket_key_of(kit, &row.song_id, row.difficulty)
}

fn jacket_key_of(kit: &CanvasKit<'_>, song_id: &str, difficulty: Difficulty) -> Option<String> {
    if song_id.is_empty() {
        return None;
    }
    let variant = if has_difficulty_suffix(song_id) {
        song_id.to_string()
    } else {
        match difficulty {
            Difficulty::Beyond => format!("{song_id}_byd"),
            Difficulty::Eternal => format!("{song_id}_etr"),
            _ => song_id.to_string(),
        }
    };
    let mut keys: Vec<String> = Vec::with_capacity(4);
    for name in [variant.as_str(), song_id] {
        keys.push(format!("jacket_thumbs/{name}.jpg"));
        keys.push(format!("jackets/{name}.jpg"));
    }
    keys.into_iter()
        .find(|candidate| kit.asset(candidate).is_ok())
}

pub fn grade_ink(score: i64) -> Color {
    if score >= 10_000_000 {
        Color::new(0xC7, 0x96, 0x2C, 255)
    } else if score >= 9_900_000 {
        Color::new(0x75, 0x9A, 0xB8, 255)
    } else if score >= 9_800_000 {
        Color::new(0x4B, 0x62, 0x8B, 255)
    } else if score >= 9_500_000 {
        Color::new(0x95, 0x72, 0x91, 255)
    } else {
        Color::new(0xB9, 0xAE, 0xCB, 255)
    }
}

fn score_ink(row: &BoardRow) -> Color {
    match row.clear_type {
        Some(3) | Some(6) => CLEAR_PM,
        Some(2) => CLEAR_FR,
        Some(1) | Some(4) | Some(5) => CLEAR_TC,
        Some(0) => CLEAR_TL,
        _ => {
            if row.score >= 10_000_000 {
                INK_GOLD
            } else if row.score >= 9_900_000 {
                INK_TEAL
            } else {
                INK_SCORE
            }
        }
    }
}

fn tick_ink(board: Option<BoardKind>, row: &BoardRow) -> Color {
    if board != Some(BoardKind::Ol50) {
        return grade_ink(row.score);
    }
    match row_band(row) {
        Some(OverBand::Held) => TICK_CLEARED,
        Some(OverBand::Underperform) | Some(OverBand::Normal) => TICK_LOW,
        Some(OverBand::Mild) => TICK_MID,
        Some(OverBand::Heavy) => TICK_HIGH,
        Some(OverBand::Disaster) => TICK_DISASTER,
        None => {
            if row.span >= 0.8 {
                TICK_HIGH
            } else if row.span >= 0.3 {
                TICK_MID
            } else {
                TICK_LOW
            }
        }
    }
}

fn chip_colors(difficulty: Difficulty) -> (Color, Color) {
    match difficulty {
        Difficulty::Past => (
            Color::new(0xDB, 0xEF, 0xF7, 255),
            Color::new(0x10, 0x59, 0x6C, 255),
        ),
        Difficulty::Present => (
            Color::new(0xE9, 0xF2, 0xCE, 255),
            Color::new(0x43, 0x56, 0x13, 255),
        ),
        Difficulty::Future => (
            Color::new(0xF7, 0xE1, 0xED, 255),
            Color::new(0x6F, 0x1F, 0x55, 255),
        ),
        Difficulty::Beyond => (
            Color::new(0xFB, 0xDD, 0xE2, 255),
            Color::new(0x8E, 0x16, 0x29, 255),
        ),
        Difficulty::Eternal => (
            Color::new(0xEB, 0xE5, 0xF5, 255),
            Color::new(0x4A, 0x3C, 0x6B, 255),
        ),
        Difficulty::Ins => (
            Color::new(0xE2, 0xE6, 0xFB, 255),
            Color::new(0x2E, 0x3F, 0x8F, 255),
        ),
    }
}

fn chip_label(row: &BoardRow) -> String {
    if row.has_constant && row.constant > 0.0 {
        format!("{} {:.1}", row.difficulty.as_str(), row.constant)
    } else {
        row.difficulty.as_str().to_string()
    }
}

fn aside_text(board: Option<BoardKind>, row: &BoardRow) -> Option<String> {
    if board != Some(BoardKind::Ol50) {
        return None;
    }
    if let Some(band) = row_band(row) {
        return Some(band.word().to_string());
    }
    if !row.has_over_level || !row.has_judgement {
        return None;
    }
    Some(format!("糊 {:.2}", row.mess))
}

fn tail_text(board: Option<BoardKind>, row: &BoardRow) -> Option<String> {
    if board != Some(BoardKind::Ol50) {
        return None;
    }
    if !row.has_judgement {
        return Some("— · — · —".to_string());
    }
    Some(format!("P{} · F{} · L{}", pure_of(row), row.far, row.lost))
}

fn corner_text(board: Option<BoardKind>, row: &BoardRow) -> Option<String> {
    if board != Some(BoardKind::Ol50) {
        return None;
    }
    if !row.has_over_level {
        return None;
    }
    if row.span < 0.05 {
        return None;
    }
    let tenths = (row.span * 10.0).round();
    let signed = if tenths == 0.0 {
        "0.0".to_string()
    } else if tenths < 0.0 {
        format!("-{:.1}", tenths.abs() / 10.0)
    } else {
        format!("+{:.1}", tenths / 10.0)
    };
    Some(format!("越级{signed}"))
}

fn stat_text(unknown: bool, value: i64) -> String {
    if unknown {
        "—".to_string()
    } else {
        value.to_string()
    }
}

fn mean_rating(rows: &[&BoardRow]) -> f64 {
    if rows.is_empty() {
        return 0.0;
    }
    let sum: f64 = rows.iter().map(|row| row.rating).sum();
    sum / rows.len() as f64
}

fn best_avg_label(base: &str, wanted: usize, count: usize) -> String {
    if count == 0 || count >= wanted {
        return base.to_string();
    }
    format!("BEST {count} AVG")
}

fn header_stats(input: &B50Input, rows: &[&BoardRow]) -> Vec<(String, String)> {
    let top10: Vec<&BoardRow> = rows.iter().take(10).copied().collect();
    let best10 = input.best10_avg.unwrap_or_else(|| mean_rating(&top10));
    let best50 = input.best50_avg.unwrap_or_else(|| mean_rating(rows));
    let count = input.best_count.unwrap_or(0);
    if input.board == Some(BoardKind::Ol50) {
        if let Some((label, value)) = input.board_stat.clone() {
            let mut out = vec![(label, value)];
            if input.anchor_upper > 0.0 {
                out.push((
                    String::from("攻克上限"),
                    format!("{:.2}", input.anchor_upper),
                ));
            }
            if let Some(over) = input.over_count {
                out.push((String::from("越级谱面"), format!("{over} 张")));
            }
            return out;
        }
    }
    let mut out = vec![
        (
            best_avg_label("BEST 10 AVG", 10, count),
            format!("{best10:.5}"),
        ),
        (
            best_avg_label("BEST 50 AVG", 50, count),
            format!("{best50:.5}"),
        ),
    ];
    let third = match (input.board_stat.clone(), input.board) {
        (Some((label, value)), _) => (label, value),
        (None, None) => (
            String::from("MAX PTT"),
            format!("{:.5}", input.max_ptt.unwrap_or(best50)),
        ),
        (None, _) => (String::from("收录条数"), rows.len().to_string()),
    };
    out.push(third);
    out
}

pub fn paint(kit: &CanvasKit<'_>, input: &B50Input) {
    let rows = input.board_rows();
    draw_wall(kit, input, &rows);
    draw_header(kit, input, &rows);
    draw_character(kit, input);
    draw_cards(kit, input, &rows);
    draw_footer(kit, input);
}

fn wall_seed_parts(input: &B50Input, rows: &[&BoardRow]) -> Vec<String> {
    let mut parts: Vec<String> = Vec::with_capacity(1 + rows.len() * 3);
    parts.push(match input.board {
        Some(board) => board.as_str().to_string(),
        None => String::from("-"),
    });
    for row in rows {
        parts.push(row.song_id.clone());
        parts.push(row.difficulty.as_str().to_string());
        parts.push(row.score.to_string());
    }
    parts
}

fn wall_jacket(input: &B50Input, rows: &[&BoardRow]) -> Option<String> {
    let parts = wall_seed_parts(input, rows);
    let refs: Vec<&str> = parts.iter().map(String::as_str).collect();
    if let Some(id) = jacket_pool::board_art(&refs) {
        return Some(id.to_string());
    }
    let ids: Vec<String> = rows
        .iter()
        .filter(|row| !row.song_id.is_empty())
        .map(|row| jacket_variant(row))
        .collect();
    if ids.is_empty() {
        return None;
    }
    let seed = jacket_pool::seed64(&refs);
    Some(ids[(seed % ids.len() as u64) as usize].clone())
}

fn image_pixels(image: &Image) -> Option<(Vec<u8>, usize, usize, usize)> {
    let width = image.width();
    let height = image.height();
    if width <= 0 || height <= 0 {
        return None;
    }
    let info = ImageInfo::new(
        (width, height),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let row = (width as usize) * 4;
    let mut data = vec![0u8; row * height as usize];
    let hint = aurorix_render::skia_safe::image::CachingHint::Allow;
    if !image.read_pixels(&info, &mut data, row, (0, 0), hint) {
        return None;
    }
    Some((data, row, width as usize, height as usize))
}
fn art_luma(r: u8, g: u8, b: u8) -> f64 {
    0.2126 * f64::from(r) + 0.7152 * f64::from(g) + 0.0722 * f64::from(b)
}

fn art_clamp(value: f64) -> u8 {
    if value <= 0.0 {
        0
    } else if value >= 255.0 {
        255
    } else {
        (value + 0.5) as u8
    }
}

fn art_shrink(data: &[u8], sw: usize, sh: usize, dw: usize, dh: usize) -> Vec<u8> {
    let sx = sw as f64 / dw as f64;
    let sy = sh as f64 / dh as f64;
    let mut out = vec![0u8; dw * dh * 4];
    for oy in 0..dh {
        let y0 = (oy as f64 * sy) as usize;
        let mut y1 = ((oy + 1) as f64 * sy) as usize;
        if y1 <= y0 {
            y1 = y0 + 1;
        }
        if y1 > sh {
            y1 = sh;
        }
        for ox in 0..dw {
            let x0 = (ox as f64 * sx) as usize;
            let mut x1 = ((ox + 1) as f64 * sx) as usize;
            if x1 <= x0 {
                x1 = x0 + 1;
            }
            if x1 > sw {
                x1 = sw;
            }
            let mut sr = 0.0;
            let mut sg = 0.0;
            let mut sb = 0.0;
            let mut sa = 0.0;
            let mut n = 0.0;
            for yy in y0..y1 {
                for xx in x0..x1 {
                    let at = (yy * sw + xx) * 4;
                    let a = f64::from(data[at + 3]);
                    sr += f64::from(data[at]) * a;
                    sg += f64::from(data[at + 1]) * a;
                    sb += f64::from(data[at + 2]) * a;
                    sa += a;
                    n += 1.0;
                }
            }
            if n == 0.0 || sa == 0.0 {
                continue;
            }
            let at = (oy * dw + ox) * 4;
            out[at] = art_clamp(sr / sa + 0.5);
            out[at + 1] = art_clamp(sg / sa + 0.5);
            out[at + 2] = art_clamp(sb / sa + 0.5);
            out[at + 3] = art_clamp(sa / n + 0.5);
        }
    }
    out
}

fn art_blur_pass(px: &mut [u8], w: usize, h: usize, radius: usize, vertical: bool) {
    if radius == 0 || w == 0 || h == 0 {
        return;
    }
    let rad = radius + 1;
    let (outer, inner) = if vertical { (w, h) } else { (h, w) };
    let top = radius.min(inner - 1);
    let mut ring = vec![[0u8; 3]; rad];
    for o in 0..outer {
        let mut sums = [0i64; 3];
        let mut n = 0i64;
        let at_of = |i: usize| -> usize {
            if vertical {
                (i * w + o) * 4
            } else {
                (o * w + i) * 4
            }
        };
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

fn art_normalize(px: &mut [u8], center: f64) {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    let mut i = 0;
    while i + 3 < px.len() {
        let luma = art_luma(px[i], px[i + 1], px[i + 2]);
        if luma < lo {
            lo = luma;
        }
        if luma > hi {
            hi = luma;
        }
        i += 4;
    }
    if !lo.is_finite() || !hi.is_finite() {
        return;
    }
    if hi - lo < 1.0 {
        hi = lo + 1.0;
    }
    let mut i = 0;
    while i + 3 < px.len() {
        let luma = art_luma(px[i], px[i + 1], px[i + 2]);
        let want = center - 40.0 + (luma - lo) / (hi - lo) * 46.0;
        let k = if luma > 1.0 { want / luma } else { 1.0 };
        px[i] = art_clamp(f64::from(px[i]) * k);
        px[i + 1] = art_clamp(f64::from(px[i + 1]) * k);
        px[i + 2] = art_clamp(f64::from(px[i + 2]) * k);
        i += 4;
    }
}

type ArtPlate = (Vec<u8>, usize, usize);

static PLATE_MEMO: OnceLock<Mutex<Memo<ArtPlate>>> = OnceLock::new();

fn wall_art_plate(kit: &CanvasKit<'_>, key: &str) -> Option<(Vec<u8>, usize, usize)> {
    memo::cached(&PLATE_MEMO, key, || {
        wall_art_plate_build(kit, key).map(|plate| {
            let bytes = plate.0.len();
            (plate, bytes)
        })
    })
}

fn wall_art_plate_build(kit: &CanvasKit<'_>, key: &str) -> Option<(Vec<u8>, usize, usize)> {
    let texture = kit.asset(key).ok()?;
    let image = texture.image()?;
    let (data, _row, sw, sh) = image_pixels(image)?;
    if sw == 0 || sh == 0 {
        return None;
    }
    let dw = 300usize;
    let dh = sh * dw / sw;
    if dh == 0 {
        return None;
    }
    let mut plate = art_shrink(&data, sw, sh, dw, dh);
    art_blur_pass(&mut plate, dw, dh, 6, true);
    art_blur_pass(&mut plate, dw, dh, 6, false);
    art_normalize(&mut plate, f64::from(paper_luma()));
    Some((plate, dw, dh))
}

fn draw_plate_cover(kit: &CanvasKit<'_>, data: &[u8], dw: usize, dh: usize, area: Rect) {
    let info = ImageInfo::new(
        (dw as i32, dh as i32),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let data = aurorix_render::skia_safe::Data::new_copy(data);
    let Some(image) = aurorix_render::skia_safe::images::raster_from_data(&info, data, dw * 4)
    else {
        return;
    };
    let scale = (area.width() / dw as f32).max(area.height() / dh as f32);
    let w = dw as f32 * scale;
    let h = dh as f32 * scale;
    let x = area.left() + (area.width() - w) / 2.0;
    let y = area.top() + (area.height() - h) / 2.0;
    kit.image_rect(&image, rect(x, y, w, h));
}

fn paper_luma() -> f32 {
    0.2126 * f32::from(0xF1_u8) + 0.7152 * f32::from(0xF4_u8) + 0.0722 * f32::from(0xFB_u8)
}

fn draw_wall(kit: &CanvasKit<'_>, input: &B50Input, rows: &[&BoardRow]) {
    let width = CARD_WIDTH as f32;
    let height = CARD_HEIGHT as f32;
    let full = rect(0.0, 0.0, width, height);
    b50_veil(kit, full, &[PAPER_A, PAPER_B, PAPER_C, PAPER_D], false);
    if let Some(variant) = wall_jacket(input, rows) {
        let key = format!("jackets/{variant}.jpg");
        let thumb = format!("jacket_thumbs/{variant}.jpg");
        let key = if kit.asset(&key).is_ok() {
            Some(key)
        } else {
            None
        };
        let plate_key = if kit.asset(&thumb).is_ok() {
            Some(thumb)
        } else {
            key.clone()
        };
        if let Some(plate_key) = plate_key.as_deref() {
            if let Some((plate, dw, dh)) = wall_art_plate(kit, plate_key) {
                draw_plate_cover(kit, &plate, dw, dh, full);
            }
        }
        if let Some(key) = key {
            b50_veil(
                kit,
                full,
                &[
                    Color::new(0xF6, 0xF8, 0xFD, 0xA8),
                    Color::new(0xF1, 0xF4, 0xFB, 0xB0),
                    Color::new(0xF1, 0xF4, 0xFB, 0xB0),
                    Color::new(0xF1, 0xF4, 0xFB, 0xB0),
                    Color::new(0xEE, 0xF2, 0xFA, 0xAC),
                ],
                true,
            );
            let band = rect(0.0, 0.0, width, (f64::from(height) * 0.42) as f32);
            let filter = b50_tone_filter(0.60, 1.0);
            let fade = band.height() * 0.36;
            kit.layer(
                rect(0.0, 0.0, width, band.bottom() + 2.0),
                1.0,
                BlendMode::SrcOver,
                |inner| {
                    b50_cover_with(inner, &key, band, Some(&filter), 0.10);
                    b50_veil(
                        inner,
                        band,
                        &[
                            Color::new(0xF6, 0xF8, 0xFD, 0x2C),
                            Color::new(0xF3, 0xF6, 0xFC, 0x70),
                            Color::new(0xF1, 0xF4, 0xFB, 0xB4),
                            Color::new(0xF1, 0xF4, 0xFB, 0xDE),
                            Color::new(0xF1, 0xF4, 0xFB, 0xFF),
                        ],
                        true,
                    );
                    b50_mask_band(
                        inner,
                        rect(0.0, band.bottom() - fade, width, fade + 2.0),
                        true,
                        255,
                        0,
                    );
                },
            );
        }
    }
    b50_wash_ellipse(
        kit,
        sx(1120.0),
        sx(90.0),
        sx(640.0),
        sx(540.0),
        Color::new(0xE4, 0xEC, 0xFA, 0x1C),
    );
    b50_vignette(kit, width, height);
}

fn draw_header(kit: &CanvasKit<'_>, input: &B50Input, rows: &[&BoardRow]) {
    let band = rect(0.0, 0.0, CARD_WIDTH as f32, sx(HEADER_BAND_H));
    fill_vertical(
        kit,
        band,
        &[
            (0.0, Color::new(0xFF, 0xFF, 0xFF, 0x62)),
            (0.5, Color::new(0xFF, 0xFF, 0xFF, 0x38)),
            (1.0, Color::new(0xFF, 0xFF, 0xFF, 0x00)),
        ],
    );
    draw_logo(kit);
    draw_kicker(kit, input);
    draw_header_shadow(kit, input);
    draw_header_plate(kit, input);
    draw_avatar(kit, input);
    draw_stats(kit, input, rows);
}

fn draw_logo(kit: &CanvasKit<'_>) {
    let Some((data, w, h)) = logo_mark(kit) else {
        return;
    };
    if w == 0 || h == 0 {
        return;
    }
    let width = w as f32;
    let height = h as f32;
    let x = (LOGO_X - LOGO_INK_OFFSET) * RS;
    let y = (LOGO_Y - (4.0 * RS).round()) * RS;
    let info = ImageInfo::new(
        (w as i32, h as i32),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let Some(image) = images::raster_from_data(&info, Data::new_copy(&data), w * 4) else {
        return;
    };
    kit.image_rect(&image, rect(x, y, width, height));
}

static LOGO_MARK_PIXELS: OnceLock<Option<(Vec<u8>, usize, usize)>> = OnceLock::new();

fn logo_mark(kit: &CanvasKit<'_>) -> Option<(Vec<u8>, usize, usize)> {
    if let Some(hit) = LOGO_MARK_PIXELS.get() {
        if hit.is_some() {
            return hit.clone();
        }
    }
    let built = logo_mark_build(kit);
    if built.is_some() {
        let _ = LOGO_MARK_PIXELS.set(built.clone());
    }
    built
}

fn logo_mark_build(kit: &CanvasKit<'_>) -> Option<(Vec<u8>, usize, usize)> {
    (|| {
            let texture = kit.asset("logo/arcaea_logo.png").ok()?;
            let image = texture.image()?;
            let (src, row, w, h) = image_pixels(image)?;
            let mut ink = vec![0u8; src.len()];
            for y in 0..h {
                for x in 0..w {
                    let i = y * row + x * 4;
                    let a = f64::from(src[i + 3]);
                    if a == 0.0 {
                        continue;
                    }
                    let r = f64::from(src[i]);
                    let g = f64::from(src[i + 1]);
                    let b = f64::from(src[i + 2]);
                    let lum = 0.299 * r + 0.587 * g + 0.114 * b;
                    let mut f = 1.0;
                    if lum > LOGO_PLATE_FROM {
                        f = 1.0 - (lum - LOGO_PLATE_FROM) / (LOGO_PLATE_TO - LOGO_PLATE_FROM);
                        if f <= 0.0 {
                            continue;
                        }
                    }
                    ink[i] = logo_round(r * f);
                    ink[i + 1] = logo_round(g * f);
                    ink[i + 2] = logo_round(b * f);
                    ink[i + 3] = logo_round(a * f);
                }
            }
            let cut = (h as f64 * f64::from(LOGO_MARK)) as usize;
            let cut = cut.clamp(1, h);
            let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0usize, 0usize);
            for y in 0..cut {
                for x in 0..w {
                    if ink[y * row + x * 4 + 3] <= LOGO_INK_ALPHA {
                        continue;
                    }
                    x0 = x0.min(x);
                    x1 = x1.max(x);
                    y0 = y0.min(y);
                    y1 = y1.max(y);
                }
            }
            if x1 < x0 || y1 < y0 {
                return None;
            }
            let (mw, mh) = (x1 - x0 + 1, y1 - y0 + 1);
            let mut mark = vec![0u8; mw * mh * 4];
            for y in 0..mh {
                let from = (y0 + y) * row + x0 * 4;
                let to = y * mw * 4;
                mark[to..to + mw * 4].copy_from_slice(&ink[from..from + mw * 4]);
            }
            let dw = sx(LOGO_W).round() as usize;
            let dh = (dw as f32 * mh as f32 / mw as f32).round() as usize;
            if dw == 0 || dh == 0 {
                return None;
            }
            Some((logo_resize(&mark, mw, mh, dw, dh), dw, dh))
        })()
}

fn logo_round(value: f64) -> u8 {
    if value <= 0.0 {
        0
    } else if value >= 255.0 {
        255
    } else {
        (value + 0.5) as u8
    }
}

fn logo_resize(src: &[u8], sw: usize, sh: usize, dw: usize, dh: usize) -> Vec<u8> {
    let mut out = vec![0u8; dw * dh * 4];
    if sw == 0 || sh == 0 || dw == 0 || dh == 0 {
        return out;
    }
    let sx = sw as f64 / dw as f64;
    let sy = sh as f64 / dh as f64;
    for oy in 0..dh {
        let y0 = (oy as f64 * sy) as usize;
        let mut y1 = ((oy + 1) as f64 * sy) as usize;
        if y1 <= y0 {
            y1 = y0 + 1;
        }
        let y1 = y1.min(sh);
        for ox in 0..dw {
            let x0 = (ox as f64 * sx) as usize;
            let mut x1 = ((ox + 1) as f64 * sx) as usize;
            if x1 <= x0 {
                x1 = x0 + 1;
            }
            let x1 = x1.min(sw);
            let (mut sr, mut sg, mut sb, mut sa, mut n) = (0.0, 0.0, 0.0, 0.0, 0.0);
            for y in y0..y1 {
                for x in x0..x1 {
                    let i = (y * sw + x) * 4;
                    let a = f64::from(src[i + 3]);
                    sr += f64::from(src[i]) * a;
                    sg += f64::from(src[i + 1]) * a;
                    sb += f64::from(src[i + 2]) * a;
                    sa += a;
                    n += 1.0;
                }
            }
            if n == 0.0 || sa == 0.0 {
                continue;
            }
            let o = (oy * dw + ox) * 4;
            out[o] = logo_round(sr / sa);
            out[o + 1] = logo_round(sg / sa);
            out[o + 2] = logo_round(sb / sa);
            out[o + 3] = logo_round(sa / n);
        }
    }
    out
}

fn draw_kicker(kit: &CanvasKit<'_>, input: &B50Input) {
    let kicker = board_kicker(input.board);
    let face = style(theme::FONT_TITLE, 11.0, INK_LABEL);
    let x = sx(LOGO_X + KICKER_INK_PAD);
    let baseline = sx(LOGO_RULE_Y - (10.0 * RS).round());
    sketch::text_spaced(kit, &face, kicker, x, baseline, INK_LABEL, sf(3.2));
}

struct HeaderPlate {
    area: Rect,
    name: String,
    cx: f32,
    cy: f32,
    size: f32,
}

fn header_plate(
    kit: &CanvasKit<'_>,
    input: &B50Input,
    char_left: Option<f32>,
) -> Option<HeaderPlate> {
    let name = input.badge.name.trim();
    if name.is_empty() {
        return None;
    }
    let k = avatar_r() / LEDGER_AVATAR_R;
    let plate_h = LEDGER_PLATE_H * k;
    let base_size = LEDGER_PLATE_NAME_SIZE * k * HEADER_PLATE_NAME_SCALE;
    let mut size = base_size;
    let inner_pad = LEDGER_PLATE_INNER_PAD * k;
    let left_pad = LEDGER_PLATE_LEFT_PAD * k;
    let x0 = (sx(AVATAR_CX) - avatar_r()) - sx(HEADER_OUTDENT);
    let y0 = header_avatar_cy() - plate_h / 2.0;
    let peak_y = plate_peak_at(x0);
    let avatar_w = 2.0 * avatar_r();
    let mut right = grid_x0() + 3.0 * grid_dw();
    if let Some(left) = char_left {
        let limit = left - sx(16.0);
        if limit < right {
            right = limit;
        }
    }
    let cap_w = (right - x0).min(HEADER_PLATE_MAX_W);
    if cap_w <= 0.0 {
        return None;
    }
    let mut tail = sx(HEADER_PLATE_TAIL);
    let mut pad_r = left_pad;
    let mut pad_l = inner_pad;
    let ink_h = |size: f32| size * 0.9;
    let width_of = |text_w: f32, size: f32, tail: f32, pad_r: f32, pad_l: f32| -> f32 {
        let wedge = (peak_y - (y0 + plate_h / 2.0)) + ink_h(size) / 2.0;
        avatar_w
            + pad_l
            + text_w
            + wedge
            + pad_r
            + tail
            + tail * HEADER_PLATE_NAME_SHIFT
            + HEADER_PLATE_SLACK
    };
    let name_style = |size: f32| style(theme::FONT_NARROW_BOLD, size / RS, INK_NAME);
    let measure = |size: f32| sketch::text_width(kit, &name_style(size), name);
    let mut text_w = measure(size);
    if let Some(over) = Some(width_of(text_w, size, tail, pad_r, pad_l) - cap_w) {
        if over > 0.0 {
            let cut = over.min(tail);
            tail -= cut;
            let over = over - cut;
            if over > 0.0 {
                let cut = over.min(pad_r * 0.6);
                pad_r -= cut;
                let over = over - cut;
                if over > 0.0 {
                    pad_l -= over.min(pad_l * 0.5);
                }
            }
        }
    }
    if width_of(text_w, size, tail, pad_r, pad_l) > cap_w {
        let fixed = width_of(0.0, size, tail, pad_r, pad_l);
        if text_w > 0.0 {
            let per_size = text_w / size;
            if per_size > 0.0 {
                let fitted = (cap_w - fixed) / per_size;
                size = fitted
                    .max(base_size * HEADER_PLATE_NAME_MIN_RATIO)
                    .min(size);
                text_w = measure(size);
            }
        }
    }
    let mut w = width_of(text_w, size, tail, pad_r, pad_l).min(cap_w);
    let want = grid_x0() + HEADER_PLATE_RIGHT_COLS * grid_dw() - x0;
    if want > w {
        w = want.min(cap_w);
    }
    if w <= 0.0 {
        return None;
    }
    let wedge = (peak_y - (y0 + plate_h / 2.0)) + ink_h(size) / 2.0;
    let diamond_right = x0 + avatar_w + sx(5.0);
    let right_inner = x0 + w;
    let name_gap = sx(8.0);
    let span = right_inner - wedge - diamond_right - 2.0 * name_gap;
    if span <= 0.0 {
        return None;
    }
    let mut text = name.to_string();
    if text_w > span {
        let per_size = text_w / size;
        if per_size > 0.0 {
            let fitted = (span / per_size).max(base_size * HEADER_PLATE_NAME_MIN_RATIO);
            size = fitted.min(size);
            text_w = measure(size);
        }
        if text_w > span {
            text = sketch::ellipsize(kit, &name_style(size), name, span);
        }
    }
    Some(HeaderPlate {
        area: rect(x0, y0, w, plate_h),
        name: text,
        cx: (diamond_right + right_inner) / 2.0,
        cy: y0 + plate_h / 2.0,
        size,
    })
}

fn plate_area(input: &B50Input, kit: &CanvasKit<'_>) -> Option<Rect> {
    let char_left = char_area(kit, input).map(|(dst, _, _)| dst.left());
    header_plate(kit, input, char_left).map(|plate| plate.area)
}

fn draw_header_shadow(kit: &CanvasKit<'_>, input: &B50Input) {
    let Some(area) = plate_area(input, kit) else {
        return;
    };
    let peak = plate_peak(area);
    let shape = plate_path(area, peak);
    let halo = avatar_r() + sx(5.0);
    let cut = header_cut_path(area, peak, avatar_cx(), header_avatar_cy(), halo);
    let canvas = kit.canvas();
    canvas.save();
    canvas.clip_path(&cut, aurorix_render::skia_safe::ClipOp::Difference, true);
    stamp_cast(kit, &shape, HEADER_SHADOW_FAR, 0.0, sx(10.0));
    canvas.restore();
}

fn header_cut_path(
    area: Rect,
    peak: f32,
    cx: f32,
    cy: f32,
    halo: f32,
) -> aurorix_render::skia_safe::Path {
    let mut builder = PathBuilder::new();
    let top = area.top();
    let bottom = area.bottom();
    let d_top = (peak - top).max(0.0);
    let d_bottom = (bottom - peak).max(0.0);
    builder.move_to((area.left() + d_top, top));
    builder.line_to((area.right() - d_top, top));
    builder.line_to((area.right(), peak));
    builder.line_to((area.right() - d_bottom, bottom));
    builder.line_to((area.left() + d_bottom, bottom));
    builder.line_to((area.left(), peak));
    builder.close();
    builder.move_to((cx, cy - halo));
    builder.line_to((cx + halo, cy));
    builder.line_to((cx, cy + halo));
    builder.line_to((cx - halo, cy));
    builder.close();
    builder.detach()
}

fn stamp_cast(
    kit: &CanvasKit<'_>,
    shape: &aurorix_render::skia_safe::Path,
    ink: Color,
    dx: f32,
    dy: f32,
) {
    let canvas = kit.canvas();
    canvas.save();
    canvas.translate((dx, dy));
    canvas.draw_path(shape, &solid(ink));
    canvas.restore();
}
fn draw_header_plate(kit: &CanvasKit<'_>, input: &B50Input) {
    let char_left = char_area(kit, input).map(|(dst, _, _)| dst.left());
    let Some(plate) = header_plate(kit, input, char_left) else {
        return;
    };
    let peak = plate_peak(plate.area);
    let shape = plate_path(plate.area, peak);
    let painted = input
        .badge
        .banner
        .as_deref()
        .map(|banner| kit.asset(banner).is_ok())
        .unwrap_or(false);
    let (ink, glow) = if painted {
        super::player::banner_plate(kit, plate.area, input.badge.banner.as_deref(), true)
    } else {
        kit.canvas().draw_path(&shape, &solid(PLATE_FALLBACK));
        (PLATE_NAME_LIGHT, Color::new(0, 0, 0, 0))
    };
    let size = font_size(plate.size);
    let name_style = theme::style(theme::FONT_NARROW_BOLD, size, ink);
    let Some(bounds) = sketch::ink_box(kit, &name_style, &plate.name) else {
        return;
    };
    let x = plate.cx.round() - (bounds.left() + bounds.right()) / 2.0 + HEADER_NAME_OPTICAL;
    let baseline = (plate.cy - (bounds.top() + bounds.bottom()) / 2.0 + 0.5).round();
    if glow.a != 0 {
        b50_glow_text(
            kit,
            &name_style,
            &plate.name,
            x,
            baseline,
            glow,
            sx(3.0).max(1.0),
        );
    }
    sketch::text(kit, &name_style, &plate.name, x, baseline, ink);
}

fn draw_avatar(kit: &CanvasKit<'_>, input: &B50Input) {
    let frame = crate::cards::avatar::resolved_frame(input.badge.ptt, input.rank_frame.or(input.badge.frame));
    let avatar = avatar::Avatar::new(input.badge.avatar.as_deref())
        .with_ptt(input.badge.ptt)
        .with_frame(frame)
        .with_global_rank(input.badge.global_rank);
    avatar::draw(kit, avatar_cx(), header_avatar_cy(), avatar_r(), &avatar);
}

fn draw_stats(kit: &CanvasKit<'_>, input: &B50Input, rows: &[&BoardRow]) {
    let stats = header_stats(input, rows);
    if stats.is_empty() {
        return;
    }
    let col_w = grid_dw();
    let x0 = sx(44.0);
    let top = sx(STAT_TOP) + sx(HEADER_DROP);
    let base_label = top + sx(STAT_LABEL_TOP);
    let base_value = top + sx(STAT_VALUE_TOP);
    let tick = mix_with_paper(accent_rgb(input.board), PAPER_A, 0.72);
    let tick = Color::new(tick.r, tick.g, tick.b, 0xCC);
    let bounds = rect(
        x0,
        top - sx(6.0),
        stats.len() as f32 * col_w,
        sx(58.0) + sx(6.0),
    );
    let body = |inner: &CanvasKit<'_>| {
        for (index, (label, value)) in stats.iter().enumerate() {
            let x = x0 + index as f32 * col_w;
            let label_style = if wide(label) {
                style(theme::FONT_NARROW, 9.0, INK_STAT_LABEL)
            } else {
                style(theme::FONT_TITLE, 10.0, INK_STAT_LABEL)
            };
            let value_style = if wide(value) {
                style(theme::FONT_NARROW, 20.0, INK_STAT)
            } else {
                style(theme::FONT_TITLE, 22.0, INK_STAT)
            };
            let label_box = sketch::ink_box(inner, &label_style, label);
            let value_box = sketch::ink_box(inner, &value_style, value);
            let label_left = label_box.map(|ink| ink.left()).unwrap_or(0.0);
            let label_top = label_box.map(|ink| ink.top()).unwrap_or(0.0);
            let value_left = value_box.map(|ink| ink.left()).unwrap_or(0.0);
            let value_bottom = value_box.map(|ink| ink.bottom()).unwrap_or(0.0);
            let tick_top = base_label + label_top;
            sketch::fill_round(
                inner,
                rect(
                    x,
                    tick_top,
                    sx(2.0).max(2.0),
                    base_value + value_bottom - tick_top,
                ),
                sf(1.0),
                tick,
            );
            sketch::text_spaced(
                inner,
                &label_style,
                label,
                x + sx(13.0) - label_left,
                base_label,
                INK_STAT_LABEL,
                sf(2.0),
            );
            sketch::text(
                inner,
                &value_style,
                value,
                x + sx(13.0) - sx(1.0) - value_left,
                base_value,
                INK_STAT,
            );
        }
    };
    kit.layer(bounds, 0.86, BlendMode::Multiply, body);
}

fn content_box(image: &Image) -> Option<(f32, f32, f32, f32)> {
    let (data, row, width, height) = image_pixels(image)?;
    if width == 0 || height == 0 {
        return None;
    }
    const STEP: usize = 2;
    const MIN_RUN: usize = 8;
    let mut min_y: isize = -1;
    let mut max_y: isize = -1;
    for y in 0..height {
        let mut hits = 0usize;
        let mut x = 0usize;
        while x < width {
            if data[y * row + x * 4 + 3] > 16 {
                hits += 1;
            }
            x += STEP;
        }
        if hits >= MIN_RUN {
            if min_y < 0 {
                min_y = y as isize;
            }
            max_y = y as isize;
        }
    }
    if min_y < 0 {
        return Some((0.0, 0.0, width as f32, height as f32));
    }
    let mut min_x = width as isize;
    let mut max_x: isize = -1;
    for x in 0..width {
        let mut hits = 0usize;
        let mut y = min_y as usize;
        while y <= max_y as usize {
            if data[y * row + x * 4 + 3] > 16 {
                hits += 1;
            }
            y += STEP;
        }
        if hits >= MIN_RUN / 2 {
            if (x as isize) < min_x {
                min_x = x as isize;
            }
            if (x as isize) > max_x {
                max_x = x as isize;
            }
        }
    }
    if max_x < min_x {
        min_x = 0;
        max_x = width as isize;
    }
    let step = STEP as isize;
    let x0 = (min_x - step).max(0);
    let y0 = (min_y - step).max(0);
    let x1 = (max_x + step).min(width as isize);
    let y1 = (max_y + step).min(height as isize);
    Some((x0 as f32, y0 as f32, (x1 - x0) as f32, (y1 - y0) as f32))
}

fn char_area(kit: &CanvasKit<'_>, input: &B50Input) -> Option<(Rect, Rect, String)> {
    let key = input
        .badge
        .character
        .as_deref()
        .filter(|key| !key.is_empty())
        .unwrap_or(DEFAULT_CHARACTER);
    let texture = kit.asset(key).ok()?;
    let image = texture.image()?;
    let (cx, cy, cw, ch) = content_box(image)?;
    if cw <= 0.0 || ch <= 0.0 {
        return None;
    }
    let head_h = top10_y0() - sx(LOGO_Y);
    let vis_top = sx(CHAR_TOP);
    let gap = sx(CHAR_GAP);
    let mut vis_h = head_h * CHAR_HMUL;
    let max_vis_h = top10_y0() - vis_top - gap;
    if vis_h > max_vis_h {
        vis_h = max_vis_h;
    }
    let mut h = vis_h.trunc();
    let y0 = vis_top;
    if h <= 0.0 {
        return None;
    }
    let vis_bottom = top10_y0() - sx(CHAR_VISIBLE_LIFT);
    let keep = (vis_bottom - y0) / h;
    let mut sh = ch;
    if keep > 0.0 && keep < 1.0 {
        let cropped = (ch * keep).trunc();
        if cropped <= 0.0 {
            return None;
        }
        sh = cropped;
        h = vis_bottom - y0;
    }
    if h <= 0.0 {
        return None;
    }
    let w = h * cw / sh;
    let x0 = (CARD_WIDTH as f32 - grid_x0() - sx(CHAR_RIGHT_PAD)) - w;
    Some((rect(x0, y0, w, h), rect(cx, cy, cw, sh), key.to_string()))
}

fn draw_character(kit: &CanvasKit<'_>, input: &B50Input) {
    let Some((dst, src, key)) = char_area(kit, input) else {
        return;
    };
    let Ok(texture) = kit.asset(&key) else {
        return;
    };
    let Some(image) = texture.image() else {
        return;
    };
    b50_wash_ellipse(
        kit,
        dst.left() + dst.width() / 2.0,
        dst.top() + dst.height() * 2.0 / 5.0,
        dst.width() * 9.0 / 10.0,
        dst.height() * 7.0 / 10.0,
        CHAR_GLOW,
    );
    kit.layer(dst, 1.0, BlendMode::SrcOver, |inner| {
        let canvas = inner.canvas();
        canvas.save();
        canvas.clip_rect(
            rect(0.0, 0.0, CARD_WIDTH as f32, CARD_HEIGHT as f32),
            ClipOp::Intersect,
            true,
        );
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        canvas.draw_image_rect_with_sampling_options(
            image,
            Some((&src, SrcRectConstraint::Fast)),
            dst,
            aurorix_render::sampling_for((src.width(), src.height()), dst),
            &paint,
        );
        canvas.restore();
        let fade_h = dst.height() * CHAR_BOTTOM_FADE;
        b50_mask_band(
            inner,
            rect(dst.left(), dst.bottom() - fade_h, dst.width(), fade_h),
            true,
            255,
            0,
        );
    });
}

fn card_cell(index: usize) -> Option<(f32, f32)> {
    let column = index % GRID_COLUMNS;
    let line = index / GRID_COLUMNS;
    if line >= TOP_ROWS + BEST_ROWS {
        return None;
    }
    let top = if line < TOP_ROWS {
        top10_y0()
    } else {
        best10_y0()
    };
    let row_in_block = if line < TOP_ROWS {
        line
    } else {
        line - TOP_ROWS
    };
    let y = top + row_in_block as f32 * row_dh();
    if y + card_h() > CARD_HEIGHT as f32 {
        return None;
    }
    Some((grid_x0() + column as f32 * grid_dw(), y))
}

fn draw_cards(kit: &CanvasKit<'_>, input: &B50Input, rows: &[&BoardRow]) {
    let corners = card_corners();
    let mut placed: Vec<(usize, f32, f32)> = Vec::new();
    for index in 0..rows.len() {
        if let Some((x, y)) = card_cell(index) {
            placed.push((index, x, y));
        }
    }
    for (_, x, y) in &placed {
        let area = rect(*x, *y, grid_cw(), card_h());
        let path = sketch::blade_path(area, corners);
        b50_shadow_paths(kit, &[&path], CARD_SHADOW_FAR, 0.0, sx(6.0), sx(14.0));
        b50_shadow_paths(kit, &[&path], CARD_SHADOW, 0.0, sx(2.0), sx(3.0));
    }
    for (index, x, y) in placed {
        let row = rows[index];
        let rank = if row.rank > 0 { row.rank } else { index + 1 };
        draw_card(kit, input, row, rank, x, y);
    }
}

fn draw_card(kit: &CanvasKit<'_>, input: &B50Input, row: &BoardRow, rank: usize, x: f32, y: f32) {
    let width = grid_cw();
    let height = card_h();
    let area = rect(x, y, width, height);
    let corners = card_corners();
    sketch::fill_blade_gradient(
        kit,
        area,
        corners,
        &sketch::vertical_gradient(CARD_TOP, CARD_BOTTOM),
    );
    sketch::stroke_blade(kit, area, corners, sf(1.1), CARD_RIM);
    let inner_corners = BladeCorners::new(
        corners.tl - 1.4,
        corners.tr - 1.4,
        corners.br - 1.4,
        corners.bl - 1.4,
    );
    let inner_area = rect(x + 1.0, y + 1.0, width - 2.0, height - 2.0);
    stroke_blade_gradient(
        kit,
        inner_area,
        inner_corners,
        sf(1.2),
        &sketch::vertical_gradient(CARD_GLOW, Color::new(0xFF, 0xFF, 0xFF, 0x00)),
    );

    let art = sx(CARD_ART);
    let art_y = y + (height - art) / 2.0;
    let win = rect(x + width - sx(CARD_PAD) - art, art_y, art, art);
    let rad = art * 0.13;
    match jacket_key(kit, row) {
        Some(key) => {
            sketch::fill_round(
                kit,
                rect(
                    win.left() - sx(2.0),
                    win.top() - sx(2.0),
                    win.width() + sx(4.0),
                    win.height() + sx(4.0),
                ),
                rad + 2.0,
                ART_TRAY,
            );
            sketch::fill_round(
                kit,
                rect(win.left(), win.top() + sx(1.0), win.width(), win.height()),
                rad,
                ART_TRAY_IN,
            );
            sketch::paste_cover_round(kit, &key, win, rad, ART_EMPTY);
            sketch::stroke_round(
                kit,
                rect(
                    win.left() - 1.0,
                    win.top() - 1.0,
                    win.width() + 2.0,
                    win.height() + 2.0,
                ),
                rad + 1.0,
                sf(1.0),
                ART_RIM,
            );
            sketch::stroke_round(kit, win, rad, sf(1.0), ART_INNER_RIM);
            sketch::fill_round(
                kit,
                rect(
                    win.left() + sx(3.0),
                    win.top() + sx(2.0),
                    win.width() - sx(6.0),
                    sx(1.0).max(1.0),
                ),
                1.0,
                ART_GLINT,
            );
        }
        None => {
            sketch::fill_round(kit, win, rad, ART_EMPTY);
            sketch::stroke_round(kit, win, rad, sf(1.4), ART_EMPTY_RIM);
        }
    }

    let tx = x + sx(CARD_TEXT_X);
    let text_r = win.left() - sx(CARD_TEXT_R);
    let avail = text_r - tx;

    let hash_style = style(theme::FONT_LATIN, 9.5, INK_FAINT);
    let rank_style = style(theme::FONT_TITLE, 12.5, INK_SUB);
    let rank_text = format!("{rank:02}");
    let hash_w = sketch::text_width(kit, &hash_style, "#");
    sketch::text(
        kit,
        &hash_style,
        "#",
        tx,
        y + sx(CARD_RANK_Y + 1.0),
        INK_FAINT,
    );
    let rank_x = tx + hash_w + sx(1.0);
    sketch::text(
        kit,
        &rank_style,
        &rank_text,
        rank_x,
        y + sx(CARD_RANK_Y + 1.0),
        INK_SUB,
    );
    let rank_end = rank_x + sketch::text_width(kit, &rank_style, &rank_text);

    let label = chip_label(row);
    let chip_spaced = sf(0.9);
    let chip_style = style(theme::FONT_NARROW_BOLD, 8.5, WHITE);
    let (bg, fg) = chip_colors(row.difficulty);
    let chip_w = sketch::text_spaced_width(kit, &chip_style, &label, chip_spaced) + sx(15.0);
    let chip_x1 = text_r;
    let chip_x0 = chip_x1 - chip_w;
    sketch::fill_round(
        kit,
        rect(
            chip_x0,
            y + sx(CARD_CHIP_TOP),
            chip_w,
            sx(CARD_CHIP_BOTTOM - CARD_CHIP_TOP),
        ),
        sf(8.0),
        bg,
    );
    let chip_base = y + sx(CARD_RANK_Y + 1.0);
    sketch::text_spaced(
        kit,
        &chip_style,
        &label,
        chip_x0 + sx(7.0) + sx(1.0),
        chip_base,
        fg,
        chip_spaced,
    );

    if let Some(aside) = aside_text(input.board, row) {
        let mid = (rank_end + chip_x0) / 2.0;
        for size in [9.5_f32, 8.5] {
            let face = style(theme::FONT_TEXT, size, INK_FAINT);
            let aside_w = sketch::text_width(kit, &face, &aside);
            if mid - aside_w / 2.0 > rank_end + sx(4.0) && mid + aside_w / 2.0 < chip_x0 - sx(4.0) {
                sketch::text(
                    kit,
                    &face,
                    &aside,
                    mid - aside_w / 2.0,
                    y + sx(CARD_RANK_Y),
                    INK_FAINT,
                );
                break;
            }
        }
    }

    if avail > 0.0 {
        let mut size = CARD_TITLE_SIZE;
        if !has_lower(&row.title) {
            size -= 0.9;
        }
        let title_style = style(theme::FONT_NARROW_BOLD, size, INK_CARD_TITLE);
        let text = sketch::ellipsize(kit, &title_style, &row.title, avail);
        sketch::text(
            kit,
            &title_style,
            &text,
            tx,
            y + sx(CARD_TITLE_Y),
            INK_CARD_TITLE,
        );
    }

    if avail > 0.0 {
        let score_text = sketch::thousands(row.score);
        let score_style = fit_score_style(kit, &score_text, avail);
        draw_score_text(
            kit,
            &score_style,
            tx,
            y + sx(CARD_SCORE_Y),
            &score_text,
            score_ink(row),
            INK_FAINT,
        );
    }

    let tail = tail_text(input.board, row);
    let corner = corner_text(input.board, row);
    let meta_style = style(theme::FONT_TITLE, 10.5, INK_ACCENT);
    let ptt_text = if row.rating > 0.0 {
        format!("{:.3}", row.rating)
    } else {
        String::from("--")
    };
    match tail {
        Some(tail) => {
            let ptt_left = text_r;
            let corner_style = corner.as_ref().map(|text| corner_face(kit, text, avail));
            let tail_r = match (&corner_style, &corner) {
                (Some(face), Some(text)) => text_r - sketch::text_width(kit, face, text) - sx(10.0),
                _ => ptt_left,
            };
            draw_tail(kit, y, tx, tail_r, &tail);
            if let (Some(face), Some(text)) = (corner_style, corner) {
                let width = sketch::text_width(kit, &face, &text);
                sketch::text(
                    kit,
                    &face,
                    &text,
                    text_r - width,
                    y + sx(CARD_JUDGE_Y),
                    INK_ACCENT,
                );
            }
        }
        None => {
            let mut ptt_left = text_r;
            if avail > 0.0 {
                ptt_left = text_r - sketch::text_width(kit, &meta_style, &ptt_text) - sx(10.0);
            }
            draw_judge_row(kit, row, tx, ptt_left, y);
            if avail > 0.0 {
                let width = sketch::text_width(kit, &meta_style, &ptt_text);
                sketch::text(
                    kit,
                    &meta_style,
                    &ptt_text,
                    text_r - width,
                    y + sx(CARD_JUDGE_Y),
                    INK_ACCENT,
                );
            }
        }
    }

    let nx = x + sx(5.0);
    sketch::fill_round(
        kit,
        rect(
            nx,
            y + sx(CARD_TICK_Y0),
            sx(3.0).max(3.0),
            sx(CARD_TICK_Y1) - sx(CARD_TICK_Y0),
        ),
        sf(1.5),
        tick_ink(input.board, row),
    );
}

fn fit_score_style(kit: &CanvasKit<'_>, text: &str, avail: f32) -> TextStyle {
    let mut size = CARD_SCORE_MAX;
    while size >= CARD_SCORE_MIN {
        let face = style(theme::FONT_TITLE, size, INK_SCORE);
        if sketch::text_width(kit, &face, text) <= avail {
            return face;
        }
        size -= 1.0;
    }
    style(theme::FONT_TITLE, CARD_SCORE_MIN, INK_SCORE)
}

fn draw_score_text(
    kit: &CanvasKit<'_>,
    face: &TextStyle,
    x: f32,
    baseline: f32,
    text: &str,
    main: Color,
    quiet: Color,
) {
    sketch::text(kit, face, text, x, baseline, main);
    let mut cx = x;
    for ch in text.chars() {
        let piece = ch.to_string();
        if ch == '\'' {
            sketch::text(kit, face, &piece, cx, baseline, quiet);
        }
        cx += sketch::text_width(kit, face, &piece);
    }
}

fn corner_face(kit: &CanvasKit<'_>, text: &str, avail: f32) -> TextStyle {
    let budget = avail / 2.0;
    let mut chosen = style(theme::FONT_NARROW, 8.5, INK_ACCENT);
    for size in [10.0_f32, 9.0, 8.5] {
        let face = style(theme::FONT_TEXT, size, INK_ACCENT);
        chosen = face.clone();
        if sketch::text_width(kit, &face, text) <= budget {
            return face;
        }
    }
    chosen
}

fn draw_tail(kit: &CanvasKit<'_>, y: f32, x0: f32, x1: f32, text: &str) {
    let avail = x1 - x0;
    if avail <= 0.0 || text.is_empty() {
        return;
    }
    let mut chosen = None;
    for size in [10.0_f32, 9.0, 8.5] {
        let face = style(theme::FONT_TEXT, size, INK_SUB);
        let fitted = sketch::text_width(kit, &face, text) <= avail;
        chosen = Some(face);
        if fitted {
            break;
        }
    }
    if let Some(face) = chosen {
        sketch::text(kit, &face, text, x0, y + sx(CARD_JUDGE_Y), INK_SUB);
    }
}

fn judge_face(size: f32, ink: Color) -> TextStyle {
    style(theme::FONT_NARROW, size, ink)
}

fn judge_segments(row: &BoardRow) -> [(String, String, Color); 3] {
    [
        (
            String::from("P"),
            match (!row.has_judgement, row.normal_perfect) {
                (true, _) => stat_text(!row.has_judgement, pure_of(row)),
                (false, normal) if normal > 0 => {
                    format!("{}(-{normal})", stat_text(false, pure_of(row)))
                }
                (false, _) => stat_text(false, pure_of(row)),
            },
            JUDGE_P,
        ),
        (
            String::from("F"),
            stat_text(!row.has_judgement, row.far),
            JUDGE_F,
        ),
        (
            String::from("L"),
            stat_text(!row.has_judgement, row.lost),
            JUDGE_L,
        ),
    ]
}

fn judge_row_space(kit: &CanvasKit<'_>, segs: &[(String, String, Color); 3], size: f32) -> f32 {
    let face = judge_face(size, INK_SUB);
    let gap = sx(3.0);
    let mut width = 0.0;
    for (index, (label, number, _)) in segs.iter().enumerate() {
        if index > 0 {
            width += gap;
        }
        width += sketch::text_width(kit, &face, label);
        width += sketch::text_width(kit, &face, number);
    }
    width
}

fn judge_size(kit: &CanvasKit<'_>, segs: &[(String, String, Color); 3], avail: f32) -> f32 {
    for size in [10.0_f32, 9.5, 9.0, 8.5] {
        if judge_row_space(kit, segs, size) <= avail {
            return size;
        }
    }
    8.5
}

fn draw_judge_row(kit: &CanvasKit<'_>, row: &BoardRow, tx: f32, tail_r: f32, y: f32) {
    let segs = judge_segments(row);
    let size = judge_size(kit, &segs, tail_r - tx);
    let face = judge_face(size, INK_SUB);
    let gap = sx(3.0);
    let baseline = y + sx(CARD_JUDGE_Y);
    let mut cur = tx;
    for (index, (label, number, ink)) in segs.iter().enumerate() {
        if index > 0 {
            cur += gap;
        }
        let (main, note) = match number.find('(') {
            Some(at) => (&number[..at], &number[at..]),
            None => (number.as_str(), ""),
        };
        let label_w = sketch::text_width(kit, &face, label);
        let main_w = sketch::text_width(kit, &face, main);
        let note_w = sketch::text_width(kit, &face, note);
        sketch::text(kit, &face, label, cur, baseline, *ink);
        cur += label_w;
        sketch::text(kit, &face, main, cur, baseline, JUDGE_NUM);
        cur += main_w;
        if !note.is_empty() {
            sketch::text(kit, &face, note, cur, baseline, JUDGE_NOTE);
            cur += note_w;
        }
    }
}

fn draw_footer(kit: &CanvasKit<'_>, input: &B50Input) {
    let last_card_bottom = best10_y0() + 7.0 * row_dh() + card_h();
    let line_y = (last_card_bottom + CARD_HEIGHT as f32) / 2.0 + sx(FOOTER_BASE_LIFT);
    let face = style(theme::FONT_NARROW, 15.0, INK_LABEL);
    if let Some(stamp) = input.generated_at.as_deref() {
        let text = format!("Created At {stamp}");
        sketch::text(kit, &face, &text, grid_x0(), line_y, INK_LABEL);
    }
    let credit = if input.cn {
        String::from("Designed By Okamikko")
    } else {
        String::from("Generated By Okamikko  /  Designed By 克鲁")
    };
    let right = CARD_WIDTH as f32 - grid_x0();
    let width = sketch::text_width(kit, &face, &credit);
    sketch::text(kit, &face, &credit, right - width, line_y, INK_LABEL);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::{harness, CardKind};
    use crate::model::Grade;

    fn sample_row(index: usize, judged: bool) -> BoardRow {
        BoardRow {
            rank: index + 1,
            song_id: String::from("abstrusedilemma"),
            title: format!("Abstruse Dilemma {index:02}"),
            difficulty: Difficulty::Future,
            constant: 9.6 + (index % 5) as f64 * 0.1,
            has_constant: true,
            score: 9_978_342 - (index as i64 % 7) * 1_000,
            rating: 12.4 - index as f64 * 0.01,
            grade: Grade::Ex,
            far: if judged { 2 + (index % 4) as i64 } else { 0 },
            normal_perfect: if judged { 3 + (index % 6) as i64 } else { 0 },
            lost: if judged {
                if index.is_multiple_of(9) {
                    1
                } else {
                    0
                }
            } else {
                0
            },
            object_amount: 1100,
            clear_type: Some(2 + (index % 2) as i32),
            has_judgement: judged,
            band: Some(OverBand::ALL[index % OverBand::ALL.len()]),
            span: 0.3 + index as f64 * 0.01,
            pressure: 0.0,
            play_at: None,
            mess: 0.12,
            over_delta: 0.08,
            has_over_level: true,
            over_level: Some(0.3),
            over_band: None,
            over_mess: None,
            backend_pressure: None,
        }
    }

    fn sample_input(best: usize, recent: usize, judged: bool) -> B50Input {
        B50Input::new(
            PlayerBadge::new("Afkeru")
                .with_ptt(12.31)
                .with_star(8)
                .with_subtitle("国际服 · 好友码 431343680")
                .with_avatar("char_icons/unknown_icon.png")
                .with_banner("banners/course_banner_01.png")
                .with_character("characters/71u_s1.png"),
            Theme::DEFAULT,
        )
        .with_best((0..best).map(|index| sample_row(index, judged)).collect())
        .with_recent((0..recent).map(|index| sample_row(index, judged)).collect())
        .with_ptt(12.31)
        .with_note("B30 与最近成绩")
        .with_generated_at("2025/11/20 12:34")
    }

    fn render_with(input: &B50Input) -> Vec<u8> {
        let harness = harness::Harness::new();
        harness.render(request(input), |kit| paint(kit, input))
    }

    fn decoded(input: &B50Input) -> (Vec<u8>, harness::Pixels) {
        let bytes = render_with(input);
        let pixels = harness::decode(&bytes).expect("解码 b50");
        (bytes, pixels)
    }

    fn ink(pixels: &harness::Pixels, area: (f32, f32, f32, f32)) -> f64 {
        pixels.ink_ratio(
            area.0 as u32,
            area.1 as u32,
            area.2 as u32,
            area.3 as u32,
            2,
        )
    }

    fn dark(pixels: &harness::Pixels, area: (f32, f32, f32, f32), threshold: u32) -> usize {
        let mut count = 0;
        let mut y = area.1 as u32;
        while y < (area.1 + area.3) as u32 && y < pixels.height {
            let mut x = area.0 as u32;
            while x < (area.0 + area.2) as u32 && x < pixels.width {
                let [r, g, b, _] = pixels.at(x, y);
                if (u32::from(r) + u32::from(g) + u32::from(b)) / 3 < threshold {
                    count += 1;
                }
                x += 1;
            }
            y += 1;
        }
        count
    }

    fn region_diff(
        left: &harness::Pixels,
        right: &harness::Pixels,
        area: (f32, f32, f32, f32),
    ) -> usize {
        let mut count = 0;
        let mut y = area.1 as u32;
        while y < (area.1 + area.3) as u32 && y < left.height.min(right.height) {
            let mut x = area.0 as u32;
            while x < (area.0 + area.2) as u32 && x < left.width.min(right.width) {
                if left.at(x, y) != right.at(x, y) {
                    count += 1;
                }
                x += 1;
            }
            y += 1;
        }
        count
    }

    fn signature(pixels: &harness::Pixels) -> Vec<u32> {
        let mut out = Vec::new();
        let mut y = 0;
        while y < pixels.height {
            let mut x = 0;
            while x < pixels.width {
                out.push(u32::from_be_bytes(pixels.at(x, y)));
                x += 6;
            }
            y += 6;
        }
        out
    }

    fn differences(left: &[u32], right: &[u32]) -> usize {
        left.iter().zip(right).filter(|(a, b)| a != b).count()
    }

    #[test]
    fn case_208() {
        assert_eq!(CARD_WIDTH, 1820, "1300 × 1.4 = 1820");
        assert_eq!(CARD_HEIGHT, 2134, "1524 × 1.4 = 2133.6, 取整 2134");
        assert!(
            CardKind::B50.spec().accepts(CARD_WIDTH, CARD_HEIGHT),
            "规格表必须登记设备像素尺寸 1820×2134"
        );
        let input = sample_input(50, 10, true);
        let (bytes, pixels) = decoded(&input);
        assert_eq!(
            harness::image_size(&bytes),
            Some((CARD_WIDTH as u32, CARD_HEIGHT as u32)),
            "出图必须是 1820×2134 设备像素"
        );
        assert!(
            pixels.ink_ratio(0, 0, CARD_WIDTH as u32, CARD_HEIGHT as u32, 4) > 0.03,
            "整张卡片必须有足够墨迹"
        );
        let tick = pixels.at(70, (top10_y0() + sx(50.0)) as u32);
        assert_eq!(
            &tick[..3],
            &[0x75, 0x9A, 0xB8],
            "左缘竖签在卡边内侧 5, 颜色按 9_978_342 那一档(9_900_000 以上)评级色"
        );
    }

    #[test]
    fn case_209() {
        assert_eq!(GRID_COLUMNS, 5, "横向永远五列");
        assert_eq!(TOP_ROWS + BEST_ROWS, 10, "前十两行 + 后四十行八行");
        let mut per_row: Vec<usize> = vec![0; 10];
        for index in 0..50 {
            let (x, y) = card_cell(index).expect("五十格都要落在版面上");
            let column = ((x - grid_x0()) / grid_dw()).round() as usize;
            let row = if y < best10_y0() {
                ((y - top10_y0()) / row_dh()).round() as usize
            } else {
                TOP_ROWS + ((y - best10_y0()) / row_dh()).round() as usize
            };
            assert_eq!(column, index % 5, "第 {index} 张的列号");
            assert_eq!(row, index / 5, "第 {index} 张的行号");
            per_row[row] += 1;
        }
        assert!(per_row.iter().all(|count| *count == 5), "每行正好五张卡");
        let (last_x, last_y) = card_cell(49).expect("第五十格");
        assert!(
            last_x + grid_cw() <= CARD_WIDTH as f32,
            "最后一列 {} 不能越过画布右边",
            last_x + grid_cw()
        );
        assert!(
            last_y + card_h() <= CARD_HEIGHT as f32,
            "最后一行 {} 要给页脚留出位置",
            last_y + card_h()
        );
    }

    #[test]
    fn case_210() {
        assert_eq!(card_h(), 137.0, "卡高 98 × 1.4 = 137.2, 取整 137");
        assert_eq!(grid_cw(), 319.0, "卡宽 228 × 1.4 = 319.2, 取整 319");
        assert_eq!(row_dh(), 162.0, "行距 116 × 1.4 = 162.4, 取整 162");
        assert_eq!(grid_dw(), 344.0, "列距 246 × 1.4 = 344.4, 取整 344");
        assert_eq!(grid_x0(), 62.0, "左轴 44 × 1.4 = 61.6, 取整 62");
        assert_eq!(top10_y0(), 437.0, "前十顶 312 × 1.4 = 436.8, 取整 437");
        assert_eq!(best10_y0(), 762.0, "后四十顶 544 × 1.4 = 761.6, 取整 762");
        assert_eq!(grid_dw() - grid_cw(), 25.0, "横向缝 18 × 1.4 ≈ 25");
        assert_eq!(row_dh() - card_h(), 25.0, "纵向缝 18 × 1.4 ≈ 25");
    }

    #[test]
    fn case_211() {
        let top_bottom = top10_y0() + row_dh() + card_h();
        let section = best10_y0() - top_bottom;
        assert_eq!(
            section, 26.0,
            "go 的网格上下同距: 段距 26 与常规缝 25 只差一次取整, 没有额外留白"
        );
        assert_eq!(
            best10_y0() - top10_y0(),
            325.0,
            "两块之间正好是两个行距(2 × 162 = 324, 差 1 来自 544 与 2 × 116 的取整)"
        );
        let union_top = top10_y0();
        let union_bottom = best10_y0() + 7.0 * row_dh() + card_h();
        assert!(
            union_bottom < CARD_HEIGHT as f32,
            "最后一排卡底 {union_bottom} 必须收在画布内"
        );
        assert!(union_top > 0.0, "第一排卡顶必须落在画布内");
    }

    #[test]
    fn case_212() {
        let harness = harness::Harness::new();
        let mut seen: Vec<(&'static str, Vec<u32>)> = Vec::new();
        let mut over_pixels = None;
        for kind in BoardKind::ALL {
            let mut input = sample_input(50, 10, true);
            input.board = Some(kind);
            let bytes = harness.render(request(&input), |kit| paint(kit, &input));
            let pixels = harness::decode(&bytes).expect("解码榜面");
            assert_eq!(
                harness::image_size(&bytes),
                Some((CARD_WIDTH as u32, CARD_HEIGHT as u32)),
                "{kind} 也必须出 1820×2134"
            );
            assert!(!pixels.blank(), "{kind} 不能是空白图");
            assert!(
                ink(&pixels, (58.0, 100.0, 980.0, 56.0)) > 0.02,
                "{kind} 刊头必须画上去"
            );
            seen.push((kind.as_str(), signature(&pixels)));
            if kind == BoardKind::Ol50 {
                over_pixels = Some(pixels);
            }
        }
        assert_eq!(seen.len(), BoardKind::ALL.len(), "六个榜面都要渲染一次");
        for left in 0..seen.len() {
            for right in (left + 1)..seen.len() {
                let changed = differences(&seen[left].1, &seen[right].1);
                assert!(
                    changed > 40,
                    "{} 与 {} 的输出几乎一样, 只有 {changed} 个采样点不同",
                    seen[left].0,
                    seen[right].0
                );
            }
        }
        let over = over_pixels.expect("越级榜必须渲染过");
        assert!(
            dark(&over, (75.0, 450.0, 60.0, 120.0), 200) > 60,
            "越级榜第一格的档位词与名次必须画上去"
        );
    }

    #[test]
    fn case_213() {
        let judged = sample_input(50, 10, true);
        let plain = sample_input(50, 10, false);
        let (_, judged_pixels) = decoded(&judged);
        let (_, plain_pixels) = decoded(&plain);
        assert!(
            dark(&judged_pixels, (88.0, 540.0, 100.0, 40.0), 170) > 60,
            "卡底 P/F/L 三列必须真的画出来, 深色像素只有 {}",
            dark(&judged_pixels, (88.0, 540.0, 100.0, 40.0), 170)
        );
        let changed = differences(&signature(&judged_pixels), &signature(&plain_pixels));
        assert!(
            changed > 20,
            "有判定和无判定的画面必须不同, 实际只差 {changed} 个采样点"
        );
    }

    #[test]
    fn case_214() {
        let mut low = sample_input(50, 10, true);
        let mut high = low.clone();
        low.badge.ptt = Some(9.87);
        high.badge.ptt = Some(12.31);
        let (_, low_pixels) = decoded(&low);
        let (_, high_pixels) = decoded(&high);
        let badge_area = (120.0, 210.0, 90.0, 90.0);
        assert!(
            region_diff(&low_pixels, &high_pixels, badge_area) > 30,
            "段位徽章里的 PTT 必须真的画上去"
        );
        assert!(
            ink(&low_pixels, (60.0, 150.0, 110.0, 150.0)) > 0.05,
            "页头头像菱形必须画上去"
        );
    }

    #[test]
    fn case_215() {
        let empty = B50Input::new(PlayerBadge::new("Afkeru"), Theme::DEFAULT);
        let (bytes, pixels) = decoded(&empty);
        assert_eq!(
            harness::image_size(&bytes),
            Some((CARD_WIDTH as u32, CARD_HEIGHT as u32)),
            "空榜也必须出 1820×2134"
        );
        assert!(!pixels.blank(), "空榜不能是空白图");
        assert!(
            ink(&pixels, (0.0, 0.0, CARD_WIDTH as f32, CARD_HEIGHT as f32)) > 0.01,
            "空榜也要有纸面与页头"
        );
        let mut bare = sample_input(50, 10, true);
        bare.badge.avatar = None;
        bare.badge.character = None;
        bare.badge.banner = None;
        for row in bare.best.iter_mut() {
            row.song_id = String::from("no-such-jacket");
        }
        let (bytes, pixels) = decoded(&bare);
        assert!(
            harness::image_size(&bytes) == Some((CARD_WIDTH as u32, CARD_HEIGHT as u32)),
            "没有曲绘与立绘时仍然是同一张画布"
        );
        assert!(!pixels.blank(), "没有素材时不能画出空白图");
        assert!(
            ink(&pixels, (0.0, 430.0, CARD_WIDTH as f32, 300.0)) > 0.03,
            "没有曲绘时卡片仍要画出来"
        );
    }

    #[test]
    fn case_216() {
        let mut input = sample_input(50, 10, true);
        input.generated_at = Some(String::from("2025/11/20 12:34"));
        let (_, with_stamp) = decoded(&input);
        input.generated_at = None;
        let (_, without) = decoded(&input);
        let footer = (58.0, 2050.0, 700.0, 60.0);
        assert!(
            dark(&with_stamp, footer, 200) > 40,
            "有生成时间时左端要写 Created At"
        );
        assert!(
            region_diff(&with_stamp, &without, footer) > 40,
            "没有生成时间时这一整段不该有任何字形"
        );
        assert!(
            dark(&without, footer, 200) < dark(&with_stamp, footer, 200),
            "去掉生成时间之后页脚左端的墨迹必须变少"
        );
    }

    #[test]
    fn case_217() {
        let wrapped = card(sample_input(50, 10, true));
        assert_eq!(wrapped.request.width, CARD_WIDTH);
        assert_eq!(wrapped.request.height, CARD_HEIGHT);
        assert_eq!(wrapped.label, FILE_NAME);
    }

    #[test]
    fn case_218_avg_labels_follow_denominator() {
        let window = sample_input(10, 0, true).with_averages(Some(12.4), Some(2.48), 0);
        let rows = window.board_rows();
        let stats = header_stats(&window, &rows);
        assert_eq!(stats[0].0, "BEST 10 AVG", "窗口口径: 标签照满榜数写");
        assert_eq!(stats[1].0, "BEST 50 AVG");
        assert_eq!(stats[1].1, "2.48000", "后端按窗口除, 插件只透传不再除一次");

        let counted = window.with_averages(Some(12.4), Some(12.4), 10);
        let rows = counted.board_rows();
        let stats = header_stats(&counted, &rows);
        assert_eq!(stats[0].0, "BEST 10 AVG");
        assert_eq!(stats[1].0, "BEST 10 AVG", "条数口径: 分母就是后端的 count");
    }

    #[test]
    fn case_219_x50_stats_use_real_counts() {
        let short = sample_input(3, 0, true)
            .with_board(BoardKind::X50)
            .with_averages(Some(9.740133), Some(9.740133), 3)
            .with_board_stat("F + L ≤ 2", "3");
        let rows = short.board_rows();
        let stats = header_stats(&short, &rows);
        assert_eq!(stats[1].0, "BEST 3 AVG", "不满 50 张时标签写实际条数");
        assert_eq!(stats[1].1, "9.74013", "值除以实际条数");
        assert_eq!(stats[2].0, "F + L ≤ 2");
        assert_eq!(stats[2].1, "3");
    }
}
