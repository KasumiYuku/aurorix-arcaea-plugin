use aurorix_render::skia_safe::{
    paint::Style, ColorType, Image, Paint, Path, PathBuilder, Point, RRect, Rect,
};
use aurorix_render::{CanvasKit, Color, Gradient, LinearGradient, TextStyle};

use super::theme;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BladeCorners {
    pub tl: f32,
    pub tr: f32,
    pub br: f32,
    pub bl: f32,
}

impl BladeCorners {
    pub const fn new(tl: f32, tr: f32, br: f32, bl: f32) -> Self {
        Self { tl, tr, br, bl }
    }

    pub const fn even(radius: f32) -> Self {
        Self::new(radius, radius, radius, radius)
    }

    fn radii(self) -> [Point; 4] {
        [
            Point::new(self.tl, self.tl),
            Point::new(self.tr, self.tr),
            Point::new(self.br, self.br),
            Point::new(self.bl, self.bl),
        ]
    }
}

pub fn blade_rrect(area: Rect, corners: BladeCorners) -> RRect {
    let radii = corners.radii();
    RRect::new_rect_radii(area, &radii)
}

pub fn blade_path(area: Rect, corners: BladeCorners) -> Path {
    let mut builder = PathBuilder::new();
    builder.add_rrect(blade_rrect(area, corners), None, None);
    builder.detach()
}

pub fn slant_path(area: Rect, cut: f32) -> Path {
    let mut builder = PathBuilder::new();
    builder.move_to((area.left() + cut, area.top()));
    builder.line_to((area.right(), area.top()));
    builder.line_to((area.right() - cut, area.bottom()));
    builder.line_to((area.left(), area.bottom()));
    builder.close();
    builder.detach()
}

fn case_098(color: Color) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(color.to_skia());
    paint
}

fn case_099(color: Color, thick: f32) -> Paint {
    let mut paint = case_098(color);
    paint.set_style(Style::Stroke);
    paint.set_stroke_width(thick.max(0.0));
    paint
}

fn case_100(gradient: &Gradient, area: Rect) -> Option<Paint> {
    let shader = gradient.shader(area).ok()?;
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_shader(shader);
    Some(paint)
}

pub fn fill_blade(kit: &CanvasKit<'_>, area: Rect, corners: BladeCorners, color: Color) {
    kit.canvas()
        .draw_path(&blade_path(area, corners), &case_098(color));
}

pub fn fill_blade_gradient(
    kit: &CanvasKit<'_>,
    area: Rect,
    corners: BladeCorners,
    gradient: &Gradient,
) {
    if let Some(paint) = case_100(gradient, area) {
        kit.canvas().draw_path(&blade_path(area, corners), &paint);
    }
}

pub fn stroke_blade(
    kit: &CanvasKit<'_>,
    area: Rect,
    corners: BladeCorners,
    thick: f32,
    color: Color,
) {
    kit.canvas()
        .draw_path(&blade_path(area, corners), &case_099(color, thick));
}

pub fn fill_round(kit: &CanvasKit<'_>, area: Rect, radius: f32, color: Color) {
    kit.rrect_fill(area, radius, color);
}

pub fn stroke_round(kit: &CanvasKit<'_>, area: Rect, radius: f32, thick: f32, color: Color) {
    kit.stroke(area, radius, aurorix_render::Border::new(color, thick));
}

pub fn fill_round_gradient(
    kit: &CanvasKit<'_>,
    area: Rect,
    radius: f32,
    top: Color,
    bottom: Color,
) {
    let gradient = Gradient::Linear(LinearGradient::vertical([
        aurorix_render::GradientStop::new(0.0, top),
        aurorix_render::GradientStop::new(1.0, bottom),
    ]));
    let _ = kit.fill_gradient(area, radius, &gradient);
}

pub fn fill_slant(kit: &CanvasKit<'_>, area: Rect, cut: f32, color: Color) {
    kit.canvas().draw_path(&slant_path(area, cut), &case_098(color));
}

pub fn stroke_slant(kit: &CanvasKit<'_>, area: Rect, cut: f32, thick: f32, color: Color) {
    kit.canvas()
        .draw_path(&slant_path(area, cut), &case_099(color, thick));
}

pub fn fill_polygon(kit: &CanvasKit<'_>, points: &[(f32, f32)], color: Color) {
    if points.len() < 3 {
        return;
    }
    let mut builder = PathBuilder::new();
    builder.move_to(points[0]);
    for point in &points[1..] {
        builder.line_to(*point);
    }
    builder.close();
    kit.canvas().draw_path(&builder.detach(), &case_098(color));
}

pub fn diamond(kit: &CanvasKit<'_>, cx: f32, cy: f32, radius: f32, color: Color) {
    fill_polygon(
        kit,
        &[
            (cx, cy - radius),
            (cx + radius, cy),
            (cx, cy + radius),
            (cx - radius, cy),
        ],
        color,
    );
}

pub fn stroke_diamond(
    kit: &CanvasKit<'_>,
    cx: f32,
    cy: f32,
    radius: f32,
    thick: f32,
    color: Color,
) {
    let mut builder = PathBuilder::new();
    builder.move_to((cx, cy - radius));
    builder.line_to((cx + radius, cy));
    builder.line_to((cx, cy + radius));
    builder.line_to((cx - radius, cy));
    builder.close();
    kit.canvas()
        .draw_path(&builder.detach(), &case_099(color, thick));
}

pub fn clip_round<R>(kit: &CanvasKit<'_>, area: Rect, radius: f32, body: impl FnOnce() -> R) -> R {
    let canvas = kit.canvas();
    canvas.save();
    canvas.clip_rrect(blade_rrect(area, BladeCorners::even(radius)), None, true);
    let out = body();
    canvas.restore();
    out
}

pub fn clip_blade<R>(
    kit: &CanvasKit<'_>,
    area: Rect,
    corners: BladeCorners,
    body: impl FnOnce() -> R,
) -> R {
    let canvas = kit.canvas();
    canvas.save();
    canvas.clip_rrect(blade_rrect(area, corners), None, true);
    let out = body();
    canvas.restore();
    out
}

pub fn paste_cover_round(kit: &CanvasKit<'_>, key: &str, area: Rect, radius: f32, fallback: Color) {
    clip_round(kit, area, radius, || {
        if kit.draw_asset_cover(key, area, 0.0).is_err() {
            fill_round(kit, area, radius, fallback);
        }
    });
}

pub fn paste_cover_blade(
    kit: &CanvasKit<'_>,
    key: &str,
    area: Rect,
    corners: BladeCorners,
    fallback: Color,
) {
    clip_blade(kit, area, corners, || {
        if kit.draw_asset_cover(key, area, 0.0).is_err() {
            fill_blade(kit, area, corners, fallback);
        }
    });
}

pub fn paste_fit_round(kit: &CanvasKit<'_>, key: &str, area: Rect, radius: f32, fallback: Color) {
    clip_round(kit, area, radius, || {
        if kit.draw_asset(key, area, 0.0).is_err() {
            fill_round(kit, area, radius, fallback);
        }
    });
}

pub fn is_han(ch: char) -> bool {
    matches!(
        ch,
        '\u{2E80}'..='\u{9FFF}'
            | '\u{F900}'..='\u{FAFF}'
            | '\u{FF00}'..='\u{FFEF}'
            | '\u{20000}'..='\u{2FA1F}'
    )
}

pub fn split_runs(value: &str) -> Vec<(String, bool)> {
    let mut runs: Vec<(String, bool)> = Vec::with_capacity(4);
    for ch in value.chars() {
        let han = is_han(ch);
        match runs.last_mut() {
            Some((run, flag)) if *flag == han => run.push(ch),
            _ => runs.push((ch.to_string(), han)),
        }
    }
    runs
}

fn run_style(base: &TextStyle, han: bool) -> TextStyle {
    let mut style = base.clone();
    if han {
        style.family = String::from(theme::FONT_FALLBACK);
    }
    if base.family == theme::FONT_DISPLAY {
        style.size = ((base.size as f32) * theme::DISPLAY_INK_SCALE)
            .round()
            .max(1.0) as u32;
    }
    style
}

pub fn text(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    value: &str,
    x: f32,
    baseline: f32,
    color: Color,
) -> f32 {
    let mut cursor = x.round();
    let mut total = 0.0;
    for (run, han) in split_runs(value) {
        let mut style = run_style(style, han);
        style.color = color;
        let width = kit
            .draw_text(&run, cursor, baseline, &style)
            .map(|metrics| metrics.width)
            .unwrap_or(0.0);
        cursor += width;
        total += width;
    }
    total
}

pub fn text_right(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    value: &str,
    right: f32,
    baseline: f32,
    color: Color,
) -> f32 {
    let width = text_width(kit, style, value);
    text(kit, style, value, right - width, baseline, color);
    width
}

pub fn text_center(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    value: &str,
    center_x: f32,
    baseline: f32,
    color: Color,
) {
    let width = text_width(kit, style, value);
    text(kit, style, value, center_x - width / 2.0, baseline, color);
}

pub fn text_shadowed(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    value: &str,
    x: f32,
    baseline: f32,
    color: Color,
    shadow: Color,
) {
    text(kit, style, value, x + 1.0, baseline + 1.0, shadow);
    text(kit, style, value, x, baseline, color);
}

pub fn text_outlined(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    value: &str,
    x: f32,
    baseline: f32,
    color: Color,
    outline: Color,
) {
    for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
        text(kit, style, value, x + dx, baseline + dy, outline);
    }
    text(kit, style, value, x, baseline, color);
}

pub fn text_shadow_outlined(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    value: &str,
    x: f32,
    baseline: f32,
    color: Color,
) {
    let offset = 1.0_f32;
    let shade = Color::new(12, 16, 32, 150);
    for (dx, dy) in [
        (-offset, 0.0),
        (offset, 0.0),
        (0.0, -offset),
        (0.0, offset),
    ] {
        text(kit, style, value, x + dx, baseline + dy, shade);
    }
    text(
        kit,
        style,
        value,
        x + offset,
        baseline + offset,
        Color::new(10, 14, 28, 130),
    );
    text(kit, style, value, x, baseline, color);
}

pub fn text_outlined_right(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    value: &str,
    right: f32,
    baseline: f32,
    color: Color,
    outline: Color,
) {
    let width = text_width(kit, style, value);
    text_outlined(kit, style, value, right - width, baseline, color, outline);
}

pub fn text_width(kit: &CanvasKit<'_>, style: &TextStyle, value: &str) -> f32 {
    split_runs(value)
        .into_iter()
        .map(|(run, han)| {
            kit.measure(&run, &run_style(style, han))
                .map(|metrics| metrics.width)
                .unwrap_or(0.0)
        })
        .sum()
}

pub fn ink_box(kit: &CanvasKit<'_>, style: &TextStyle, value: &str) -> Option<Rect> {
    let layout = kit.text_layout(style).ok()?;
    let (_, ink) = layout.primary_font().measure_text(value, None);
    Some(ink)
}

pub fn ink_center(kit: &CanvasKit<'_>, style: &TextStyle, value: &str) -> f32 {
    match ink_box(kit, style, value) {
        Some(bounds) => -((bounds.top() + bounds.bottom()) / 2.0),
        None => style.size as f32 * 0.36,
    }
}

pub fn ink_center_x(kit: &CanvasKit<'_>, style: &TextStyle, value: &str) -> f32 {
    match ink_box(kit, style, value) {
        Some(bounds) => (bounds.left() + bounds.right()) / 2.0,
        None => text_width(kit, style, value) / 2.0,
    }
}

pub fn spaced_step(spaced: f32) -> f32 {
    let mut step = ((spaced as i32) as f32 * 1.4).round();
    if spaced > 0.0 && step == 0.0 {
        step = 1.0;
    }
    step * 1.4
}

fn unspaced(style: &TextStyle) -> TextStyle {
    let mut style = style.clone();
    style.letter_spacing = 0.0;
    style
}

pub fn text_spaced_width(kit: &CanvasKit<'_>, style: &TextStyle, value: &str, spaced: f32) -> f32 {
    let style = unspaced(style);
    let step = spaced_step(spaced);
    let mut total = 0.0;
    let mut count = 0usize;
    for ch in value.chars() {
        total += text_width(kit, &style, &ch.to_string());
        count += 1;
    }
    total + step * (count.saturating_sub(1)) as f32
}

pub fn text_spaced(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    value: &str,
    x: f32,
    baseline: f32,
    color: Color,
    spaced: f32,
) -> f32 {
    let style = unspaced(style);
    let step = spaced_step(spaced);
    let total = value.chars().count();
    let mut cursor = x.round();
    let start = cursor;
    for (index, ch) in value.chars().enumerate() {
        let width = text(kit, &style, &ch.to_string(), cursor, baseline, color);
        cursor += width;
        if index + 1 < total {
            cursor += step;
        }
    }
    cursor - start
}

pub fn text_spaced_center(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    value: &str,
    center_x: f32,
    baseline: f32,
    color: Color,
    spaced: f32,
) -> f32 {
    let width = text_spaced_width(kit, style, value, spaced);
    text_spaced(kit, style, value, center_x - width / 2.0, baseline, color, spaced);
    width
}

pub fn ellipsize(kit: &CanvasKit<'_>, style: &TextStyle, value: &str, max_width: f32) -> String {
    const MARK: char = '…';
    if max_width <= 0.0 {
        return String::new();
    }
    if text_width(kit, style, value) <= max_width {
        return value.to_string();
    }
    let mark_width = text_width(kit, style, &MARK.to_string());
    if mark_width > max_width {
        return String::new();
    }
    let mut out = String::new();
    for ch in value.chars() {
        let mut probe = out.clone();
        probe.push(ch);
        if text_width(kit, style, &probe) + mark_width > max_width {
            break;
        }
        out = probe;
    }
    out.push(MARK);
    out
}

pub fn paste_crop_clip(kit: &CanvasKit<'_>, key: &str, crop: Rect, area: Rect) -> bool {
    if crop.width() <= 0.0 || crop.height() <= 0.0 {
        return false;
    }
    let Ok(texture) = kit.asset(key) else {
        return false;
    };
    let Some(image) = texture.image() else {
        return false;
    };
    let scale_x = area.width() / crop.width();
    let scale_y = area.height() / crop.height();
    let full = Rect::from_xywh(
        area.left() - crop.left() * scale_x,
        area.top() - crop.top() * scale_y,
        image.width() as f32 * scale_x,
        image.height() as f32 * scale_y,
    );
    let canvas = kit.canvas();
    canvas.save();
    canvas.clip_rect(area, None, true);
    kit.image_rect(image, full);
    canvas.restore();
    true
}

pub fn soft_shadow_blade(
    kit: &CanvasKit<'_>,
    area: Rect,
    corners: BladeCorners,
    dy: f32,
    radius: f32,
    color: Color,
) {
    kit.drop_shadow_in(area, color, 0.0, dy, radius, |inner| {
        fill_blade(inner, area, corners, Color::WHITE);
    });
}

pub fn thousands(n: i64) -> String {
    let digits = n.max(0).to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push('\'');
        }
        out.push(digit);
    }
    out
}

pub fn split_every(text: &str, every: usize) -> Vec<String> {
    if every == 0 {
        return vec![text.to_string()];
    }
    let chars: Vec<char> = text.chars().collect();
    chars
        .chunks(every)
        .map(|chunk| chunk.iter().collect())
        .collect()
}

pub fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color::new(mix(a.r, b.r), mix(a.g, b.g), mix(a.b, b.b), mix(a.a, b.a))
}

pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn clamp8(value: i32) -> u8 {
    value.clamp(0, 255) as u8
}

const RATING_CONTENT: [(&str, f32, f32, f32, f32); 10] = [
    ("rating_0", 8.0, 8.0, 103.0, 103.0),
    ("rating_1", 8.0, 8.0, 103.0, 103.0),
    ("rating_2", 8.0, 8.0, 103.0, 103.0),
    ("rating_3", 6.0, 8.0, 106.0, 107.0),
    ("rating_4", 6.0, 8.0, 106.0, 107.0),
    ("rating_5", 6.0, 8.0, 106.0, 108.0),
    ("rating_6", 3.0, 5.0, 112.0, 114.0),
    ("rating_7", 1.0, 2.0, 75.0, 77.0),
    ("rating_8", 6.0, 8.0, 106.0, 107.0),
    ("rating_off", 8.0, 8.0, 103.0, 103.0),
];

pub fn rating_crop(key: &str) -> Option<Rect> {
    let stem = key.rsplit('/').next()?.trim_end_matches(".png");
    RATING_CONTENT
        .iter()
        .find(|(name, ..)| *name == stem)
        .map(|(_, x, y, width, height)| Rect::from_xywh(*x, *y, *width, *height))
}

pub fn content_box(image: &Image) -> Option<Rect> {
    const STEP: usize = 2;
    const MIN_RUN: usize = 8;
    let (w, h) = (image.width() as usize, image.height() as usize);
    if w == 0 || h == 0 {
        return None;
    }
    let pixmap = image.peek_pixels()?;
    match pixmap.info().color_type() {
        ColorType::RGBA8888 | ColorType::BGRA8888 => {}
        _ => return None,
    }
    let pixels: &[u8] = pixmap.pixels()?;
    let stride = pixmap.row_bytes();
    let solid = |x: usize, y: usize| pixels[y * stride + x * 4 + 3] > 16;

    let mut top: Option<usize> = None;
    let mut bottom = 0usize;
    for y in 0..h {
        let mut n = 0usize;
        for x in (0..w).step_by(STEP) {
            if solid(x, y) {
                n += 1;
            }
        }
        if n >= MIN_RUN {
            top.get_or_insert(y);
            bottom = y;
        }
    }
    let top = top?;

    let mut left = w;
    let mut right = 0usize;
    let mut found = false;
    for x in 0..w {
        let mut n = 0usize;
        for y in (top..=bottom).step_by(STEP) {
            if solid(x, y) {
                n += 1;
            }
        }
        if n >= MIN_RUN / 2 {
            found = true;
            left = left.min(x);
            right = right.max(x);
        }
    }
    if !found {
        return None;
    }
    let x0 = left.saturating_sub(STEP);
    let y0 = top.saturating_sub(STEP);
    let x1 = (right + STEP).min(w);
    let y1 = (bottom + STEP).min(h);
    Some(Rect::from_xywh(
        x0 as f32,
        y0 as f32,
        (x1 - x0) as f32,
        (y1 - y0) as f32,
    ))
}

pub fn vertical_gradient(top: Color, bottom: Color) -> Gradient {
    Gradient::Linear(LinearGradient::vertical([
        aurorix_render::GradientStop::new(0.0, top),
        aurorix_render::GradientStop::new(1.0, bottom),
    ]))
}

pub fn gradient_stops(stops: &[(f32, Color)]) -> Gradient {
    let items: Vec<aurorix_render::kit::GradientStop> = stops
        .iter()
        .map(|(offset, color)| aurorix_render::GradientStop::new(*offset, *color))
        .collect();
    Gradient::Linear(LinearGradient::vertical(items))
}

const SPRITE_INK: [(&str, f32, f32, f32, f32); 49] = [
    ("avatar/char_icon_border.png", 3.0, 4.0, 144.0, 144.0),
    ("avatar/unknown_icon.png", 0.0, 0.0, 169.0, 170.0),
    ("clear_type/clear_fail.png", 38.0, 3.0, 769.0, 70.0),
    ("clear_type/clear_full.png", 21.0, 16.0, 783.0, 70.0),
    ("clear_type/clear_normal.png", 38.0, 3.0, 769.0, 70.0),
    ("clear_type/clear_pure.png", 45.0, 6.0, 745.0, 90.0),
    ("difficulty/max-recall-beyond.png", 0.0, 3.0, 358.0, 104.0),
    ("difficulty/max-recall-eternal.png", 0.0, 3.0, 381.0, 104.0),
    ("difficulty/max-recall-future.png", 0.0, 3.0, 358.0, 104.0),
    ("difficulty/max-recall-inscribed.png", 0.0, 2.0, 358.0, 105.0),
    ("difficulty/max-recall-past.png", 0.0, 3.0, 358.0, 104.0),
    ("difficulty/max-recall-present.png", 0.0, 3.0, 358.0, 104.0),
    ("grade/A.png", 74.0, 6.0, 81.0, 101.0),
    ("grade/AA.png", 35.0, 6.0, 161.0, 101.0),
    ("grade/B.png", 85.0, 6.0, 69.0, 100.0),
    ("grade/C.png", 65.0, 5.0, 89.0, 102.0),
    ("grade/D.png", 76.0, 6.0, 82.0, 100.0),
    ("grade/EX+.png", 50.0, 1.0, 178.0, 109.0),
    ("grade/EX.png", 4.0, 3.0, 142.0, 120.0),
    ("logo/arcaea_logo.png", 82.0, 7.0, 1312.0, 345.0),
    ("songselect/cell/song_cell_right.png", 0.0, 7.0, 807.0, 170.0),
    ("songselect/cell/song_cell_right_beyond.png", 0.0, 7.0, 807.0, 170.0),
    ("songselect/cell/song_cell_right_beyond_selected.png", 0.0, 7.0, 812.0, 170.0),
    ("songselect/cell/song_cell_right_selected.png", 0.0, 1.0, 817.0, 182.0),
    ("songselect/cell/song_cell_selected_piece.png", 5.0, 4.0, 116.0, 179.0),
    ("songselect/cell/song_cell_selected_piece_beyond.png", 5.0, 6.0, 116.0, 175.0),
    ("songselect/cell/start.png", 2.0, 2.0, 152.0, 53.0),
    ("songselect/diff/difficulty_selector_0.png", 0.0, 0.0, 293.0, 124.0),
    ("songselect/diff/difficulty_selector_0_selected.png", 0.0, 0.0, 293.0, 124.0),
    ("songselect/diff/difficulty_selector_1.png", 1.0, 0.0, 217.0, 124.0),
    ("songselect/diff/difficulty_selector_1_selected.png", 1.0, 0.0, 217.0, 124.0),
    ("songselect/diff/difficulty_selector_2.png", 1.0, 0.0, 217.0, 124.0),
    ("songselect/diff/difficulty_selector_2_selected.png", 1.0, 0.0, 217.0, 124.0),
    ("songselect/diff/difficulty_selector_3-1.png", 0.0, 0.0, 208.0, 124.0),
    ("songselect/diff/difficulty_selector_3-1_selected.png", 0.0, 0.0, 208.0, 124.0),
    ("songselect/diff/difficulty_selector_3.png", 4.0, 2.0, 205.0, 122.0),
    ("songselect/diff/difficulty_selector_3_selected.png", 4.0, 2.0, 205.0, 122.0),
    ("songselect/diff/difficulty_selector_4.png", 2.0, 4.0, 207.0, 120.0),
    ("songselect/diff/difficulty_selector_4_selected.png", 2.0, 4.0, 207.0, 120.0),
    ("songselect/diff/song_score_tab.png", 0.0, 1.0, 594.0, 224.0),
    ("songselect/grade/a.png", 74.0, 6.0, 81.0, 101.0),
    ("songselect/grade/aa.png", 35.0, 6.0, 161.0, 101.0),
    ("songselect/grade/b.png", 85.0, 6.0, 69.0, 100.0),
    ("songselect/grade/c.png", 65.0, 5.0, 89.0, 102.0),
    ("songselect/grade/d.png", 76.0, 6.0, 82.0, 100.0),
    ("songselect/grade/ex.png", 4.0, 3.0, 142.0, 120.0),
    ("songselect/grade/explus.png", 50.0, 1.0, 178.0, 109.0),
    ("songselect/misc/main_button.png", 6.0, 22.0, 486.0, 128.0),
    ("songselect/pack/song_currentpack.png", 0.0, 0.0, 437.0, 171.0),
];

pub fn sprite_ink(key: &str) -> Option<Rect> {
    SPRITE_INK
        .iter()
        .find(|(name, ..)| *name == key)
        .map(|(_, x, y, width, height)| Rect::from_xywh(*x, *y, *width, *height))
}

pub fn paste_sprite(kit: &CanvasKit<'_>, key: &str, cx: f32, cy: f32, ink_height: f32) -> bool {
    let Some(ink) = sprite_ink(key) else {
        return false;
    };
    let Ok(texture) = kit.asset(key) else {
        return false;
    };
    let Some((full_w, full_h)) = texture.dimensions() else {
        return false;
    };
    if ink.height() <= 0.0 || full_h <= 0 {
        return false;
    }
    let scale = ink_height / ink.height();
    let width = scale * full_w as f32;
    let height = scale * full_h as f32;
    let x0 = cx - (ink.left() + ink.width() / 2.0) * scale;
    let y0 = cy - (ink.top() + ink.height() / 2.0) * scale;
    let area = Rect::from_xywh(x0.round(), y0.round(), width, height);
    let canvas = kit.canvas();
    canvas.save();
    canvas.clip_rect(area, None, true);
    if let Some(image) = texture.image() {
        kit.image_rect(image, area);
    }
    canvas.restore();
    true
}
