use std::sync::OnceLock;

use aurorix_render::skia_safe::{
    image::CachingHint, images, AlphaType, ColorType, Data, ImageInfo, Rect,
};
use aurorix_render::CanvasKit;

pub const LOGO: &str = "logo/arcaea_logo.png";

const PLATE_FROM: f64 = 208.0;

const PLATE_TO: f64 = 244.0;

const MARK_FRAC: f64 = 0.64;

const INK_ALPHA: u8 = 96;

fn round_byte(value: f64) -> u8 {
    if value <= 0.0 {
        0
    } else if value >= 255.0 {
        255
    } else {
        (value + 0.5) as u8
    }
}

fn pixels(image: &aurorix_render::skia_safe::Image) -> Option<(Vec<u8>, usize, usize, usize)> {
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
    if !image.read_pixels(&info, &mut data, row, (0, 0), CachingHint::Allow) {
        return None;
    }
    Some((data, row, width as usize, height as usize))
}

pub fn lockup(kit: &CanvasKit<'_>) -> Option<(Vec<u8>, usize, usize)> {
    static PIXELS: OnceLock<Option<(Vec<u8>, usize, usize)>> = OnceLock::new();
    PIXELS
        .get_or_init(|| {
            let texture = kit.asset(LOGO).ok()?;
            let image = texture.image()?;
            let (src, row, w, h) = pixels(image)?;
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
                    if lum > PLATE_FROM {
                        f = 1.0 - (lum - PLATE_FROM) / (PLATE_TO - PLATE_FROM);
                        if f <= 0.0 {
                            continue;
                        }
                    }
                    ink[i] = round_byte(r * f);
                    ink[i + 1] = round_byte(g * f);
                    ink[i + 2] = round_byte(b * f);
                    ink[i + 3] = round_byte(a * f);
                }
            }
            let cut = ((h as f64 * MARK_FRAC) as usize).clamp(1, h);
            let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0usize, 0usize);
            for y in 0..cut {
                for x in 0..w {
                    if ink[y * row + x * 4 + 3] <= INK_ALPHA {
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
            Some((mark, mw, mh))
        })
        .clone()
}

pub fn draw_logo(kit: &CanvasKit<'_>, cx: f32, cy: f32, width: f32) {
    if width <= 0.0 {
        return;
    }
    let Some((data, w, h)) = lockup(kit) else {
        return;
    };
    if w == 0 || h == 0 {
        return;
    }
    let info = ImageInfo::new(
        (w as i32, h as i32),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let Some(image) = images::raster_from_data(&info, Data::new_copy(&data), w * 4) else {
        return;
    };
    let height = width * h as f32 / w as f32;
    kit.image_rect(
        &image,
        Rect::from_xywh(cx - width / 2.0, cy - height / 2.0, width, height),
    );
}
