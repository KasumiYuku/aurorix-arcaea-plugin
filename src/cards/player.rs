use aurorix_render::skia_safe::canvas::SaveLayerRec;
use aurorix_render::skia_safe::{
    image_filters, surfaces, AlphaType, ClipOp, Color4f, ColorType, Data, Image, ImageInfo, Paint,
    PathBuilder, Rect,
};
use aurorix_render::{CanvasKit, Color, TextStyle};

use super::avatar;
use super::sketch;
use super::theme;
use super::widgets::PlayerBadge;

const OUT: f32 = 0.75;

const LEDGER_W: f32 = 1440.0 * OUT;

const PLATE_PEAK: f32 = 0.62;

const PLATE_NAME_SCALE: f32 = 0.92;

const PLATE_WIDTH_RATIO: f32 = 400.0 / 1300.0;

const PLATE_NAME_MIN_RATIO: f32 = 0.55;

const PLATE_MAX_ASPECT: f32 = 10.0;

const PLATE_MIN_ASPECT: f32 = 5.0;

const HEAD_MIN_W: f32 = 340.0 * OUT;

const HEAD_BAR_GAP: f32 = 36.0 * OUT;

const PRINT_HUE_MIX: f32 = 0.35;

const PRINT_SAT: f32 = 0.70;

const PAPER_HUE: f64 = 277.0;

const LUMA_CUT: f64 = 150.0;

pub const DEFAULT_BANNER: &str = "banners/online_banner_2026_10.png";

const PLATE_SOLID: Color = Color::new(30, 28, 40, 238);

const PLATE_UNDER: Color = Color::new(0xF6, 0xF5, 0xF1, 0xFF);

const PLATE_NAME_DARK: Color = Color::new(0x5B, 0x4F, 0x53, 0xFF);

const PLATE_NAME_LIGHT: Color = Color::new(0xFF, 0xFF, 0xFF, 0xFF);

const PLATE_GLOW: Color = Color::new(0xFF, 0xFF, 0xFF, 0xC0);

const SHADOW_FAR: Color = Color::new(28, 24, 48, 46);

const SHADOW_NEAR: Color = Color::new(28, 24, 48, 31);

fn px(value: f32) -> f32 {
    value * OUT
}

fn sx(value: f32) -> f32 {
    (value * 1.4).round()
}

fn sf(value: f32) -> f32 {
    value * 1.4
}

fn size_of(value: f32) -> u32 {
    value.round().max(1.0) as u32
}

pub fn span(scale: f32) -> (f32, f32) {
    let radius = (px(54.0) * 0.8 * scale).round();
    let up = radius + sx(5.0) + sf(1.0) * 0.5;
    let badge = (radius * 0.11).round() + (radius * 2.0 * 0.62).round();
    (up, up.max(badge))
}

fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::from_xywh(x, y, w.max(0.0), h.max(0.0))
}

pub struct Block<'a> {
    pub badge: &'a PlayerBadge,
    pub right: f32,
    pub left: f32,
    pub center_y: f32,
    pub scale: f32,
    pub shadow: bool,
    pub frame: Option<i32>,
    pub accent: Option<i32>,
}

impl<'a> Block<'a> {
    pub fn new(badge: &'a PlayerBadge, right: f32, center_y: f32) -> Self {
        Self {
            badge,
            right,
            left: right - LEDGER_W,
            center_y,
            scale: 1.0,
            shadow: true,
            frame: badge.frame,
            accent: None,
        }
    }

    pub fn with_left(mut self, left: f32) -> Self {
        self.left = left;
        self
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        if scale > 0.0 {
            self.scale = scale;
        }
        self
    }

    pub fn without_shadow(mut self) -> Self {
        self.shadow = false;
        self
    }

    pub fn with_rating(mut self, frame: Option<i32>, accent: Option<i32>) -> Self {
        self.frame = frame.or(self.frame);
        self.accent = accent.or(self.accent);
        self
    }

    fn plate_h(&self) -> f32 {
        px(54.0) * self.scale
    }

    fn avatar_r(&self) -> f32 {
        px(54.0) * 0.8 * self.scale
    }

    fn inner_pad(&self) -> f32 {
        px(54.0) * 0.25
    }

    fn left_pad(&self) -> f32 {
        px(54.0) * 0.65
    }

    fn name_base(&self) -> f32 {
        px(54.0) * 0.32 / 0.72 * PLATE_NAME_SCALE * self.scale
    }

    fn max_width(&self) -> f32 {
        let cap = self.plate_h() * PLATE_MAX_ASPECT;
        if self.left > 0.0 {
            let room = self.right - self.left - HEAD_MIN_W - HEAD_BAR_GAP;
            if room > 0.0 && room < cap {
                return room;
            }
        }
        cap
    }

    fn outdent(&self) -> f32 {
        sx(9.0) * self.scale
    }

    fn style(&self, size: f32) -> TextStyle {
        theme::style(theme::FONT_DISPLAY, size_of(size), PLATE_NAME_DARK)
    }
}

#[derive(Debug, Clone)]
pub struct Plate {
    pub area: Rect,
    pub name: String,
    pub size: f32,
    pub cx: f32,
    pub center: f32,
}

fn wedge(ink_h: f32) -> f32 {
    (PLATE_PEAK - 0.5) * px(54.0) + ink_h / 2.0
}

fn avatar_r0() -> f32 {
    px(54.0) * 0.8
}

fn ink_height(kit: &CanvasKit<'_>, style: &TextStyle, value: &str) -> f32 {
    sketch::ink_box(kit, style, value)
        .map(|bounds| bounds.height().max(0.0))
        .unwrap_or(0.0)
}

fn plate_rect(block: &Block<'_>, width: f32) -> Rect {
    let h = block.plate_h();
    let w = width.max(h * PLATE_MIN_ASPECT);
    let y = block.center_y - PLATE_PEAK * h;
    rect(block.right - w, y, w, h)
}

fn safe_band(block: &Block<'_>, area: Rect, ink_h: f32) -> (f32, f32) {
    let s = block.scale;
    (
        area.left() + wedge(ink_h) + block.left_pad() * s,
        area.right() - 2.0 * block.avatar_r() - block.inner_pad() * s,
    )
}

pub fn plate(kit: &CanvasKit<'_>, block: &Block<'_>) -> Option<Plate> {
    let name = block.badge.name.as_str();
    if name.trim().is_empty() {
        return None;
    }
    let s = block.scale;
    let cap_w = block.max_width();
    let floor_w = (LEDGER_W * PLATE_WIDTH_RATIO * s).min(cap_w);
    let base = block.name_base();
    let min_size = base * PLATE_NAME_MIN_RATIO;
    let mut size = base;
    let mut area = rect(0.0, 0.0, 0.0, 0.0);
    let mut band = 0.0;
    let mut ink_h = 0.0;
    for _ in 0..3 {
        ink_h = ink_height(kit, &block.style(size), name);
        let width = sketch::text_width(kit, &block.style(size), name);
        let need = (width
            + wedge(ink_h)
            + 2.0 * avatar_r0()
            + block.inner_pad()
            + block.left_pad())
        .max(floor_w)
        .min(cap_w);
        area = plate_rect(block, need);
        let (x0, x1) = safe_band(block, area, ink_h);
        band = (x1 - x0).max(0.0);
        if width <= band || size <= min_size {
            break;
        }
        size = (size * band / width).max(min_size);
    }
    let out = block.outdent();
    area = rect(
        area.left() + out,
        area.top(),
        area.width(),
        area.height(),
    );
    let style = block.style(size);
    let mut cx = (area.left() + area.right()) / 2.0;
    let half = sketch::text_width(kit, &style, name) / 2.0;
    let (x0, x1) = safe_band(block, area, ink_h);
    if x1 > x0 {
        cx = cx.max(x0 + half).min(x1 - half);
    }
    Some(Plate {
        area,
        name: sketch::ellipsize(kit, &style, name, band),
        size,
        cx,
        center: area.top() + area.height() / 2.0,
    })
}

pub fn plate_box(kit: &CanvasKit<'_>, block: &Block<'_>) -> (f32, f32) {
    match plate(kit, block) {
        Some(value) => (value.area.width(), value.area.left()),
        None => (0.0, 0.0),
    }
}

fn plate_shape(area: Rect, peak_y: f32) -> aurorix_render::skia_safe::Path {
    let top = area.top();
    let bottom = area.bottom();
    let upper = (peak_y - top).max(0.0);
    let lower = (bottom - peak_y).max(0.0);
    let mut builder = PathBuilder::new();
    builder.move_to((area.left() + upper, top));
    builder.line_to((area.right() - upper, top));
    builder.line_to((area.right(), peak_y));
    builder.line_to((area.right() - lower, bottom));
    builder.line_to((area.left() + lower, bottom));
    builder.line_to((area.left(), peak_y));
    builder.close();
    builder.detach()
}

fn blur_sigma(radius: f32) -> f32 {
    let window = radius * 2.0 + 1.0;
    (((window * window - 1.0) / 12.0).max(0.0)).sqrt()
}

fn glow_text(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    value: &str,
    x: f32,
    baseline: f32,
    ink: Color,
    radius: f32,
) {
    if value.is_empty() || radius <= 0.0 || ink.a == 0 {
        return;
    }
    let Some(bounds) = sketch::ink_box(kit, style, value) else {
        return;
    };
    let pad = radius * 2.0;
    let area = Rect::from_ltrb(
        x + bounds.left() - pad,
        baseline + bounds.top() - pad,
        x + bounds.right() + pad,
        baseline + bounds.bottom() + pad,
    );
    let sigma = blur_sigma(radius);
    kit.layer(
        area,
        f32::from(ink.a) / 255.0,
        aurorix_render::skia_safe::BlendMode::SrcOver,
        |outer| {
            outer.blur_layer_in(area, sigma, |inner| {
                sketch::text(inner, style, value, x, baseline, Color::new(255, 255, 255, 255));
            });
        },
    );
}

fn soft_shadow(kit: &CanvasKit<'_>, area: Rect, dx: f32, dy: f32, radius: f32, ink: Color) {
    let sigma = blur_sigma(radius);
    let Some(filter) = image_filters::drop_shadow_only(
        (dx, dy),
        (sigma, sigma),
        Color4f::from(ink.to_skia()),
        None,
        None,
        None,
    ) else {
        return;
    };
    let padded = Rect::from_ltrb(
        area.left() - sigma * 3.0 - dx.abs(),
        area.top() - sigma * 3.0 - dy.abs(),
        area.right() + sigma * 3.0 + dx.abs(),
        area.bottom() + sigma * 3.0 + dy.abs(),
    );
    let mut paint = Paint::default();
    paint.set_image_filter(filter);
    let record = SaveLayerRec::default().bounds(&padded).paint(&paint);
    let canvas = kit.canvas();
    canvas.save_layer(&record);
    let shape = plate_shape(area, area.top() + area.height() * PLATE_PEAK);
    canvas.draw_path(&shape, &solid(Color::new(255, 255, 255, 255)));
    canvas.restore();
}

fn solid(ink: Color) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(ink.to_skia());
    paint
}

fn player_shadow(kit: &CanvasKit<'_>, area: Rect, out: f32, s: f32) {
    let dx = (sx(3.0) * s).round().copysign(out);
    soft_shadow(
        kit,
        area,
        dx,
        (sx(10.0) * s).round(),
        (sx(16.0) * s).round(),
        SHADOW_FAR,
    );
    soft_shadow(
        kit,
        area,
        (dx / 2.0).round(),
        (sx(2.0) * s).round(),
        (sx(4.0) * s).round(),
        SHADOW_NEAR,
    );
}

fn rgb_hsv(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let rf = f64::from(r) / 255.0;
    let gf = f64::from(g) / 255.0;
    let bf = f64::from(b) / 255.0;
    let mx = rf.max(gf).max(bf);
    let mn = rf.min(gf).min(bf);
    if mx <= 0.0 {
        return (0.0, 0.0, 0.0);
    }
    let d = mx - mn;
    let s = d / mx;
    if d == 0.0 {
        return (0.0, s, mx);
    }
    let mut h = if (mx - rf).abs() < f64::EPSILON {
        60.0 * ((gf - bf) / d).rem_euclid(6.0)
    } else if (mx - gf).abs() < f64::EPSILON {
        60.0 * ((bf - rf) / d + 2.0)
    } else {
        60.0 * ((rf - gf) / d + 4.0)
    };
    if h < 0.0 {
        h += 360.0;
    }
    (h, s, mx)
}

fn hsv_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0).rem_euclid(2.0) - 1.0).abs());
    let m = v - c;
    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    let to = |value: f64| -> u8 { ((value + m) * 255.0).clamp(0.0, 255.0).round() as u8 };
    (to(r), to(g), to(b))
}

fn harmonize(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let (h, saturation, value) = rgb_hsv(r, g, b);
    if value <= 0.0 {
        return (r, g, b);
    }
    let mut delta = PAPER_HUE - h;
    while delta > 180.0 {
        delta -= 360.0;
    }
    while delta < -180.0 {
        delta += 360.0;
    }
    let mut hue = h + delta * f64::from(PRINT_HUE_MIX);
    while hue < 0.0 {
        hue += 360.0;
    }
    while hue >= 360.0 {
        hue -= 360.0;
    }
    hsv_rgb(hue, (saturation * f64::from(PRINT_SAT)).min(1.0), value)
}

fn bake_banner(
    kit: &CanvasKit<'_>,
    banner: Option<&str>,
    area: Rect,
    paper: bool,
) -> (Option<Image>, f64) {
    let mut keys: Vec<&str> = Vec::with_capacity(2);
    if let Some(key) = banner.filter(|key| !key.is_empty()) {
        keys.push(key);
    }
    keys.push(DEFAULT_BANNER);
    for key in keys {
        let Ok(texture) = kit.asset(key) else {
            continue;
        };
        let Some(source) = texture.image() else {
            continue;
        };
        let zoom = kit
            .canvas()
            .local_to_device_as_3x3()
            .scale_x()
            .abs()
            .clamp(0.5, 4.0);
        let width = (area.width() * zoom).round().max(1.0) as i32;
        let height = (area.height() * zoom).round().max(1.0) as i32;
        let info = ImageInfo::new(
            (width, height),
            ColorType::RGBA8888,
            AlphaType::Premul,
            None,
        );
        let Some(mut surface) = surfaces::raster(&info, None, None) else {
            continue;
        };
        surface.canvas().clear(aurorix_render::skia_safe::Color::TRANSPARENT);
        let target = Rect::from_xywh(0.0, 0.0, width as f32, height as f32);
        surface.canvas().draw_image_rect_with_sampling_options(
            source,
            None,
            target,
            aurorix_render::sampling_for(
                (source.width() as f32, source.height() as f32),
                target,
            ),
            &Paint::default(),
        );
        let row = (width as usize) * 4;
        let mut data = vec![0u8; row * height as usize];
        if !surface.read_pixels(&info, &mut data, row, (0, 0)) {
            continue;
        }
        let mut sum = 0.0f64;
        let mut count = 0usize;
        for y in 0..height as usize {
            for x in 0..width as usize {
                let at = y * row + x * 4;
                if data[at + 3] == 0 {
                    continue;
                }
                let (r, g, b) = if paper {
                    harmonize(data[at], data[at + 1], data[at + 2])
                } else {
                    (data[at], data[at + 1], data[at + 2])
                };
                data[at] = r;
                data[at + 1] = g;
                data[at + 2] = b;
                if data[at + 3] >= 200 && y % 8 == 0 && x % 8 == 0 {
                    sum += 0.299 * f64::from(r) + 0.587 * f64::from(g) + 0.114 * f64::from(b);
                    count += 1;
                }
            }
        }
        let luma = if count == 0 { 0.0 } else { sum / count as f64 };
        let Some(image) = aurorix_render::skia_safe::images::raster_from_data(
            &info,
            Data::new_copy(&data),
            row,
        ) else {
            continue;
        };
        return (Some(image), luma);
    }
    (None, 0.0)
}

pub fn banner_plate(
    kit: &CanvasKit<'_>,
    area: Rect,
    banner: Option<&str>,
    paper: bool,
) -> (Color, Color) {
    if area.width() <= 0.0 || area.height() <= 0.0 {
        return (PLATE_NAME_LIGHT, Color::new(0, 0, 0, 0));
    }
    let (image, luma) = bake_banner(kit, banner, area, paper);
    let shape = plate_shape(area, area.top() + area.height() * PLATE_PEAK);
    let canvas = kit.canvas();
    canvas.save();
    canvas.clip_path(&shape, ClipOp::Intersect, true);
    match image {
        Some(image) => {
            kit.fill_rect(area, PLATE_UNDER);
            let _ = kit.image_rect(&image, area);
        }
        None => {
            kit.fill_rect(area, PLATE_SOLID);
        }
    }
    canvas.restore();
    if luma > LUMA_CUT {
        (PLATE_NAME_DARK, PLATE_GLOW)
    } else {
        (PLATE_NAME_LIGHT, Color::new(0, 0, 0, 0))
    }
}

fn plate_name(kit: &CanvasKit<'_>, block: &Block<'_>, plate_value: &Plate, ink: Color, glow: Color) {
    let style = block.style(plate_value.size);
    let Some(bounds) = sketch::ink_box(kit, &style, &plate_value.name) else {
        return;
    };
    let x = plate_value.cx.round() - (bounds.left() + bounds.right()) / 2.0;
    let baseline = (plate_value.center - (bounds.top() + bounds.bottom()) / 2.0 + 0.5).round();
    if glow.a != 0 {
        glow_text(kit, &style, &plate_value.name, x, baseline, glow, sx(3.0).max(1.0));
    }
    sketch::text(kit, &style, &plate_value.name, x, baseline, ink);
}

fn draw_avatar(kit: &CanvasKit<'_>, block: &Block<'_>, cx: f32, cy: f32, radius: f32) {
    let avatar = avatar::Avatar::new(block.badge.avatar.as_deref())
        .with_ptt(block.badge.ptt)
        .with_frame(block.frame)
        .with_global_rank(block.badge.global_rank);
    avatar::draw(kit, cx, cy, radius, &avatar);
}

pub fn paint(kit: &CanvasKit<'_>, block: &Block<'_>) {
    let Some(plate_value) = plate(kit, block) else {
        return;
    };
    let out = block.outdent();
    let radius = block.avatar_r();
    let avatar_cx = (block.right - radius + out).round();
    let avatar_cy = block.center_y.round();
    if block.shadow {
        player_shadow(kit, plate_value.area, out, block.scale);
    }
    let (ink, glow) = banner_plate(kit, plate_value.area, block.badge.banner.as_deref(), true);
    plate_name(kit, block, &plate_value, ink, glow);
    draw_avatar(kit, block, avatar_cx, avatar_cy, radius.round());
}

#[cfg(test)]
mod tests {
    use super::{harmonize, Block};
    use crate::cards::widgets::PlayerBadge;

    #[test]
    fn banner_hsv_matches_go_paper_tone() {
        assert_eq!(harmonize(107, 142, 161), (123, 131, 161));
    }

    #[test]
    fn block_inherits_badge_rating_frame() {
        let badge = PlayerBadge::new("Afkeru").with_frame(Some(4));
        assert_eq!(Block::new(&badge, 640.0, 60.0).frame, Some(4));
        assert_eq!(
            Block::new(&badge, 640.0, 60.0)
                .with_rating(None, None)
                .frame,
            Some(4)
        );
        assert_eq!(
            Block::new(&badge, 640.0, 60.0)
                .with_rating(Some(7), Some(7))
                .frame,
            Some(7)
        );
        let bare = PlayerBadge::new("Afkeru");
        assert_eq!(Block::new(&bare, 640.0, 60.0).frame, None);
    }
}
