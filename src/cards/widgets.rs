use aurorix_render::skia_safe::{Canvas, Paint, Rect};
use aurorix_render::{Border, CanvasKit, Color, RecordedLayer, TextAlign, TextStyle};

use super::theme::{self, Theme};
use crate::domain::{clear_asset, difficulty_asset, grade_asset};
use crate::model::{Difficulty, Grade};

pub const CARD_PAD: f32 = 34.0;

pub const HEADER_HEIGHT: f32 = 132.0;

pub const FOOTER_HEIGHT: f32 = 54.0;

pub const TITLE_SIZE: u32 = 46;

pub const SUBTITLE_SIZE: u32 = 24;

pub const ROW_HEIGHT: f32 = 62.0;

pub const FOOTER_NOTE: &str = "Okamikko · Arcaea";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HAlign {
    Left,
    Center,
    Right,
}

impl HAlign {
    fn to_text_align(self) -> TextAlign {
        match self {
            HAlign::Left => TextAlign::Left,
            HAlign::Center => TextAlign::Center,
            HAlign::Right => TextAlign::Right,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PlayerBadge {
    pub name: String,
    pub subtitle: String,
    pub ptt: Option<f64>,
    pub star: Option<u8>,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub character: Option<String>,
    pub global_rank: Option<i32>,
    pub frame: Option<i32>,
}

impl PlayerBadge {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Self::default()
        }
    }

    pub fn with_ptt(mut self, ptt: f64) -> Self {
        if ptt > 0.0 {
            self.ptt = Some(ptt);
        }
        self
    }

    pub fn with_star(mut self, star: i32) -> Self {
        if star > 0 {
            self.star = Some(star.min(12) as u8);
        }
        self
    }

    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = subtitle.into();
        self
    }

    pub fn with_avatar(mut self, key: impl Into<String>) -> Self {
        let key = key.into();
        if !key.is_empty() {
            self.avatar = Some(key);
        }
        self
    }

    pub fn with_banner(mut self, key: impl Into<String>) -> Self {
        let key = key.into();
        if !key.is_empty() {
            self.banner = Some(key);
        }
        self
    }

    pub fn with_character(mut self, key: impl Into<String>) -> Self {
        let key = key.into();
        if !key.is_empty() {
            self.character = Some(key);
        }
        self
    }

    pub fn with_global_rank(mut self, rank: Option<i32>) -> Self {
        self.global_rank = rank.filter(|value| *value >= 1);
        self
    }

    pub fn with_frame(mut self, frame: Option<i32>) -> Self {
        self.frame = frame.filter(|value| *value >= 0);
        self
    }

    pub fn ptt_text(&self) -> String {
        match self.ptt {
            Some(value) => format!("{value:.2}"),
            None => String::from("--"),
        }
    }

    pub fn star_text(&self) -> String {
        stars(self.star)
    }
}

pub fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect::from_xywh(x, y, width.max(0.0), height.max(0.0))
}

pub fn stars(star: Option<u8>) -> String {
    let count = star.unwrap_or(0) as usize;
    "★".repeat(count.min(12))
}

pub fn number(value: f64, digits: usize) -> String {
    format!("{value:.digits$}")
}

pub fn ellipsize(kit: &CanvasKit<'_>, style: &TextStyle, text: &str, max_width: f32) -> String {
    if max_width <= 0.0 {
        return String::new();
    }
    kit.truncate_with_ellipsis(text, style, max_width)
}

pub fn text_width(kit: &CanvasKit<'_>, style: &TextStyle, text: &str) -> f32 {
    kit.measure(text, style)
        .map(|metrics| metrics.width)
        .unwrap_or(0.0)
}

pub fn put(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    text: &str,
    x: f32,
    y: f32,
    align: HAlign,
) -> f32 {
    let style = style.clone().with_align(align.to_text_align());
    match kit.draw_text(text, x, y, &style) {
        Ok(metrics) => metrics.width,
        Err(_) => 0.0,
    }
}

pub fn put_fitted(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    text: &str,
    x: f32,
    y: f32,
    align: HAlign,
    max_width: f32,
) -> f32 {
    let clipped = ellipsize(kit, style, text, max_width);
    put(kit, style, &clipped, x, y, align)
}

pub fn backdrop(kit: &CanvasKit<'_>, theme: Theme, width: f32, height: f32) {
    let full = rect(0.0, 0.0, width, height);
    kit.fill_rect(full, theme.slot);
    if kit.draw_asset_cover(theme.backdrop, full, 0.0).is_err() {
        let _ = kit.draw_asset_cover(theme.backdrop_fallback, full, 0.0);
    }
    kit.fill_rect(full, theme.panel);
}

pub fn panel(kit: &CanvasKit<'_>, theme: Theme, area: Rect, radius: f32) {
    kit.rrect_fill(area, radius, theme.panel_alt);
    kit.stroke(area, radius, Border::new(theme.divider, 1.0));
}

pub fn row_panel(kit: &CanvasKit<'_>, theme: Theme, area: Rect, index: usize) {
    kit.rrect_fill(area, theme::ROW_RADIUS, theme.row_fill(index));
}

pub fn divider(kit: &CanvasKit<'_>, theme: Theme, x0: f32, x1: f32, y: f32) {
    if x1 <= x0 {
        return;
    }
    kit.fill_rect(rect(x0, y, x1 - x0, 1.0), theme.divider);
}

pub fn header(kit: &CanvasKit<'_>, theme: Theme, width: f32, title: &str, subtitle: &str) -> f32 {
    let style = theme::title_style(TITLE_SIZE, theme.text);
    put_fitted(
        kit,
        &style,
        title,
        CARD_PAD,
        CARD_PAD + TITLE_SIZE as f32,
        HAlign::Left,
        width - CARD_PAD * 2.0,
    );
    if !subtitle.is_empty() {
        let sub = theme::body_style(SUBTITLE_SIZE, theme.muted);
        put_fitted(
            kit,
            &sub,
            subtitle,
            CARD_PAD,
            CARD_PAD + TITLE_SIZE as f32 + SUBTITLE_SIZE as f32 + 6.0,
            HAlign::Left,
            width - CARD_PAD * 2.0,
        );
    }
    let baseline = CARD_PAD + HEADER_HEIGHT;
    divider(kit, theme, CARD_PAD, width - CARD_PAD, baseline);
    baseline + 18.0
}

pub fn footer(kit: &CanvasKit<'_>, theme: Theme, width: f32, height: f32, note: &str) {
    if height <= FOOTER_HEIGHT {
        return;
    }
    let y = height - FOOTER_HEIGHT;
    divider(kit, theme, CARD_PAD, width - CARD_PAD, y);
    let style = theme::narrow_style(20, theme.muted);
    put(kit, &style, FOOTER_NOTE, CARD_PAD, y + 34.0, HAlign::Left);
    if !note.is_empty() {
        put_fitted(
            kit,
            &style,
            note,
            width - CARD_PAD,
            y + 34.0,
            HAlign::Right,
            width * 0.6,
        );
    }
}

pub fn top_bar(kit: &CanvasKit<'_>, theme: Theme, badge: &PlayerBadge, area: Rect) {
    panel(kit, theme, area, theme::ROW_RADIUS);
    let avatar = rect(
        area.left() + 12.0,
        area.top() + 12.0,
        area.height() - 24.0,
        area.height() - 24.0,
    );
    if let Some(key) = badge.avatar.as_deref() {
        if kit.draw_asset(key, avatar, avatar.height() / 2.0).is_err() {
            kit.circle(
                (
                    avatar.left() + avatar.width() / 2.0,
                    avatar.top() + avatar.height() / 2.0,
                ),
                avatar.height() / 2.0,
                theme::difficulty_color(Difficulty::Future),
            );
        }
    } else {
        kit.circle(
            (
                avatar.left() + avatar.width() / 2.0,
                avatar.top() + avatar.height() / 2.0,
            ),
            avatar.height() / 2.0,
            theme.panel,
        );
    }
    if let Some(key) = badge.banner.as_deref() {
        let banner = rect(
            area.right() - 220.0,
            area.top() + area.height() - 26.0,
            208.0,
            14.0,
        );
        let _ = kit.draw_asset_cover(key, banner, 6.0);
    }
    let name_x = avatar.right() + 18.0;
    let name_style = theme::title_style(34, theme.text);
    put_fitted(
        kit,
        &name_style,
        &badge.name,
        name_x,
        area.top() + 46.0,
        HAlign::Left,
        area.width() - 320.0,
    );
    let sub_style = theme::body_style(20, theme.muted);
    let subtitle = if badge.subtitle.is_empty() {
        String::from("Arcaea")
    } else {
        badge.subtitle.clone()
    };
    put_fitted(
        kit,
        &sub_style,
        &subtitle,
        name_x,
        area.top() + area.height() - 20.0,
        HAlign::Left,
        area.width() - 320.0,
    );
    let ptt_style = theme::number_style(38, theme.accent);
    put(
        kit,
        &ptt_style,
        &badge.ptt_text(),
        area.right() - 20.0,
        area.top() + 52.0,
        HAlign::Right,
    );
    let star_style = theme::body_style(20, theme::grade_color(Grade::ExPlus));
    put(
        kit,
        &star_style,
        &badge.star_text(),
        area.right() - 20.0,
        area.top() + area.height() - 18.0,
        HAlign::Right,
    );
}

pub fn difficulty_pill(kit: &CanvasKit<'_>, difficulty: Difficulty, area: Rect, label: &str) {
    kit.rrect_fill(
        area,
        area.height() / 2.0,
        theme::difficulty_deep(difficulty),
    );
    kit.stroke(
        area,
        area.height() / 2.0,
        Border::new(theme::difficulty_color(difficulty), 2.0),
    );
    let style = theme::bold_style((area.height() * 0.52) as u32, Color::WHITE);
    let text = if label.is_empty() {
        difficulty.as_str()
    } else {
        label
    };
    put(
        kit,
        &style,
        text,
        area.left() + area.width() / 2.0,
        area.top() + area.height() * 0.68,
        HAlign::Center,
    );
}

pub fn difficulty_icon(kit: &CanvasKit<'_>, difficulty: Difficulty, area: Rect) -> bool {
    kit.draw_asset(difficulty_asset(difficulty), area, 0.0)
        .is_ok()
}

pub const GRADE_INK_MUTED: Color = Color::new(0xCB, 0xD5, 0xE1, 0xFF);

pub fn grade_badge(kit: &CanvasKit<'_>, cx: f32, cy: f32, ink_height: f32, grade: Option<Grade>) {
    if let Some(grade) = grade {
        if super::sketch::paste_sprite(kit, grade_asset(grade), cx, cy, ink_height) {
            return;
        }
    }
    let size = ink_height * 0.8;
    let style = theme::style(
        theme::FONT_NARROW_BOLD,
        size.round().max(1.0) as u32,
        GRADE_INK_MUTED,
    );
    super::sketch::text_center(
        kit,
        &style,
        "—",
        cx,
        cy + size * 0.36,
        GRADE_INK_MUTED,
    );
}

pub fn clear_badge(kit: &CanvasKit<'_>, clear_type: Option<i32>, area: Rect) -> bool {
    kit.draw_asset(clear_asset(clear_type.unwrap_or(-1)), area, 0.0)
        .is_ok()
}

pub fn stripe_layer(width: f32, height: f32, color: Color) -> Option<RecordedLayer> {
    let bounds = rect(0.0, 0.0, width.max(1.0), height.max(1.0));
    RecordedLayer::record(bounds, |canvas| paint_stripes(canvas, bounds, color)).ok()
}

fn paint_stripes(canvas: &Canvas, bounds: Rect, color: Color) {
    let mut paint = Paint::default();
    paint.set_color(color.to_skia());
    paint.set_anti_alias(true);
    let span = bounds.width() + bounds.height();
    let step = 26.0;
    let mut offset = -bounds.height();
    while offset < span {
        let from = (bounds.left() + offset, bounds.bottom());
        let to = (bounds.left() + offset + bounds.height(), bounds.top());
        canvas.draw_line(from, to, &paint);
        offset += step;
    }
}

pub fn empty_slot(kit: &CanvasKit<'_>, theme: Theme, area: Rect, index: usize) {
    kit.rrect_fill(area, theme::SLOT_RADIUS, theme.slot);
    kit.stroke(area, theme::SLOT_RADIUS, Border::new(theme.divider, 1.0));
    if let Some(layer) = stripe_layer(area.width(), area.height(), theme.stripe) {
        let origin = aurorix_render::skia_safe::Point::new(area.left(), area.top());
        let clipped = layer.draw_at(kit.canvas(), origin);
        let _ = clipped;
    }
    let style = theme::narrow_style(18, theme.muted);
    put(
        kit,
        &style,
        &format!("#{}", index + 1),
        area.left() + area.width() / 2.0,
        area.top() + area.height() / 2.0 + 6.0,
        HAlign::Center,
    );
}

pub fn label_value(
    kit: &CanvasKit<'_>,
    theme: Theme,
    x: f32,
    y: f32,
    label: &str,
    value: &str,
    align: HAlign,
) {
    let label_style = theme::narrow_style(18, theme.muted);
    put(kit, &label_style, label, x, y, align);
    let value_style = theme::bold_style(24, theme.text);
    put(kit, &value_style, value, x, y + 26.0, align);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_279() {
        assert_eq!(stars(None), "");
        assert_eq!(stars(Some(0)), "");
        assert_eq!(stars(Some(3)), "★★★");
        assert_eq!(stars(Some(200)).chars().count(), 12);
    }

    #[test]
    fn case_280() {
        let mut badge = PlayerBadge::new("Afkeru");
        assert_eq!(badge.ptt_text(), "--");
        badge = badge.with_ptt(12.345);
        assert_eq!(badge.ptt_text(), "12.35");
        let zero = PlayerBadge::new("Afkeru").with_ptt(0.0);
        assert_eq!(zero.ptt_text(), "--");
    }

    #[test]
    fn case_281() {
        let badge = PlayerBadge::new("x").with_star(0);
        assert_eq!(badge.star, None);
        let badge = PlayerBadge::new("x").with_star(99);
        assert_eq!(badge.star, Some(12));
    }

    #[test]
    fn case_282() {
        assert_eq!(number(1234.5678, 2), "1234.57");
        assert_eq!(number(0.0, 1), "0.0");
    }
}
