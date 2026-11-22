use std::sync::{Arc, Once};

use aurorix_plugin::PluginAssets;
use aurorix_render::skia_safe::{AlphaType, ColorType, Data, Image, ImageInfo};
use aurorix_render::{
    CacheStats, CanvasKit, RenderContext, RenderFormat, RenderRequest, SkiaBackend,
};

use super::theme;

static BOOTSTRAP: Once = Once::new();

pub const TEXTURE_CAPACITY: u64 = 256 * 1024 * 1024;

pub const FONT_CAPACITY: usize = 64;

pub fn assets() -> Arc<PluginAssets> {
    Arc::new(crate::assets())
}

pub fn bootstrap(store: &Arc<PluginAssets>) {
    BOOTSTRAP.call_once(|| {
        let registry = Arc::new(aurorix_template::Registry::new());
        let _ = store.bootstrap(&registry);
    });
}

pub fn request(width: i32, height: i32) -> RenderRequest {
    RenderRequest::new(width, height)
        .with_format(RenderFormat::Png)
}

pub fn request_with(width: i32, height: i32, format: RenderFormat) -> RenderRequest {
    RenderRequest::new(width, height)
        .with_format(format)
}

pub const SINGLE_CARD_QUALITY: u8 = 92;

pub const MID_CARD_QUALITY: u8 = 90;

pub const BIG_CARD_QUALITY: u8 = 85;

pub const MID_CARD_PIXELS: i64 = 6_000_000;

pub const BIG_CARD_PIXELS: i64 = 10_000_000;

pub fn quality_for(width: i32, height: i32) -> u8 {
    let pixels = i64::from(width.max(0)) * i64::from(height.max(0));
    if pixels > BIG_CARD_PIXELS {
        BIG_CARD_QUALITY
    } else if pixels > MID_CARD_PIXELS {
        MID_CARD_QUALITY
    } else {
        SINGLE_CARD_QUALITY
    }
}

pub fn card_request(width: i32, height: i32) -> RenderRequest {
    RenderRequest::new(width, height).with_quality(quality_for(width, height))
}

pub struct Harness {
    assets: Arc<PluginAssets>,
    context: Arc<RenderContext>,
}

impl Harness {
    pub fn new() -> Self {
        let assets = assets();
        bootstrap(&assets);
        let context = RenderContext::new(SkiaBackend::handle(), FONT_CAPACITY, TEXTURE_CAPACITY)
            .with_asset_loader(assets.loader());
        Self { assets, context }
    }

    pub fn assets(&self) -> &Arc<PluginAssets> {
        &self.assets
    }

    pub fn context(&self) -> &Arc<RenderContext> {
        &self.context
    }

    pub fn render<F>(&self, request: RenderRequest, painter: F) -> Vec<u8>
    where
        F: FnOnce(&CanvasKit<'_>),
    {
        let request = request.with_format(RenderFormat::Png);
        match self.context.draw(request, |canvas, context| {
            let kit = CanvasKit::with_chain(canvas, context, theme::chain());
            painter(&kit);
        }) {
            Ok(bytes) => bytes.to_vec(),
            Err(error) => panic!("渲染失败: {error}"),
        }
    }

    pub fn render_card(&self, card: &aurorix_plugin::RenderCard) -> Vec<u8> {
        if let Some(key) = card.cache_key {
            if let Some(hit) = aurorix_render::card_cache::peek(key) {
                return hit.to_vec();
            }
        }
        let painter = Arc::clone(&card.painter);
        let bytes = self.render(card.request, move |kit| painter(kit));
        if let Some(key) = card.cache_key {
            aurorix_render::card_cache::store(key, bytes.clone().into());
        }
        bytes
    }

    pub fn stats(&self) -> CacheStats {
        self.context.textures().stats()
    }

    pub fn decodes(&self) -> u64 {
        self.stats().decodes
    }
}

impl Default for Harness {
    fn default() -> Self {
        Harness::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pixels {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Pixels {
    pub fn at(&self, x: u32, y: u32) -> [u8; 4] {
        if x >= self.width || y >= self.height {
            return [0, 0, 0, 0];
        }
        let index = ((y * self.width + x) * 4) as usize;
        [
            self.data[index],
            self.data[index + 1],
            self.data[index + 2],
            self.data[index + 3],
        ]
    }

    pub fn distinct_colors(&self, step: u32) -> usize {
        let stride = step.max(1);
        let mut seen: Vec<u32> = Vec::new();
        let mut y = 0;
        while y < self.height {
            let mut x = 0;
            while x < self.width {
                let pixel = self.at(x, y);
                let packed = u32::from_be_bytes(pixel);
                if !seen.contains(&packed) {
                    seen.push(packed);
                }
                x += stride;
            }
            y += stride;
        }
        seen.len()
    }

    pub fn ink_ratio(&self, x: u32, y: u32, width: u32, height: u32, step: u32) -> f64 {
        let stride = step.max(1);
        let mut counts: Vec<(u32, usize)> = Vec::new();
        let mut total = 0usize;
        let mut py = y;
        while py < (y + height).min(self.height) {
            let mut px = x;
            while px < (x + width).min(self.width) {
                let packed = u32::from_be_bytes(self.at(px, py));
                match counts.iter_mut().find(|(color, _)| *color == packed) {
                    Some((_, count)) => *count += 1,
                    None => counts.push((packed, 1)),
                }
                total += 1;
                px += stride;
            }
            py += stride;
        }
        if total == 0 {
            return 0.0;
        }
        let dominant = counts.iter().map(|(_, count)| *count).max().unwrap_or(0);
        1.0 - (dominant as f64 / total as f64)
    }

    pub fn blank(&self) -> bool {
        self.distinct_colors(4) <= 1
    }
}

pub fn decode(bytes: &[u8]) -> Option<Pixels> {
    let image = Image::from_encoded(Data::new_copy(bytes))?;
    let width = image.width() as u32;
    let height = image.height() as u32;
    if width == 0 || height == 0 {
        return None;
    }
    let info = ImageInfo::new(
        (width as i32, height as i32),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let mut data = vec![0u8; (width as usize) * (height as usize) * 4];
    let hint = aurorix_render::skia_safe::image::CachingHint::Allow;
    if !image.read_pixels(&info, &mut data, (width as usize) * 4, (0, 0), hint) {
        return None;
    }
    Some(Pixels {
        width,
        height,
        data,
    })
}

pub fn image_size(bytes: &[u8]) -> Option<(u32, u32)> {
    decode(bytes).map(|pixels| (pixels.width, pixels.height))
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurorix_render::Color;

    #[test]
    fn case_224() {
        let harness = Harness::new();
        let bytes = harness.render(request(64, 48), |kit| {
            kit.fill_rect(
                aurorix_render::skia_safe::Rect::from_xywh(0.0, 0.0, 64.0, 48.0),
                Color::rgb(20, 20, 30),
            );
        });
        assert!(!bytes.is_empty(), "渲染结果不能为空");
        assert_eq!(image_size(&bytes), Some((64, 48)), "像素尺寸必须精确");
    }

    #[test]
    fn case_225() {
        let harness = Harness::new();
        let flat = harness.render(request(32, 32), |kit| {
            kit.fill_rect(
                aurorix_render::skia_safe::Rect::from_xywh(0.0, 0.0, 32.0, 32.0),
                Color::rgb(10, 10, 10),
            );
        });
        let flat_pixels = decode(&flat).expect("解码纯色图");
        assert!(flat_pixels.blank(), "纯色画布应当被判成空白");

        let busy = harness.render(request(32, 32), |kit| {
            let rects: Vec<_> = (0..32)
                .map(|index| {
                    aurorix_render::skia_safe::Rect::from_xywh(index as f32, index as f32, 8.0, 8.0)
                })
                .collect();
            kit.fill_rects_batch(&rects, Color::rgb(220, 180, 60));
        });
        let busy_pixels = decode(&busy).expect("解码有内容的图");
        assert!(
            busy_pixels.ink_ratio(0, 0, 32, 32, 1) > 0.05,
            "有内容的画布不该是空白"
        );
    }

    #[test]
    fn case_226() {
        let harness = Harness::new();
        let before = harness.decodes();
        let first = harness
            .context()
            .asset("grade/EX.png")
            .expect("评级素材必须可读");
        let second = harness
            .context()
            .asset("grade/EX.png")
            .expect("评级素材必须可读");
        assert!(
            std::sync::Arc::ptr_eq(&first, &second),
            "第二次取用必须命中缓存里的同一份纹理"
        );
        let decodes = harness.decodes() - before;
        assert_eq!(decodes, 1, "同一素材只解码一次, 实际 {decodes}");
    }

    #[test]
    fn case_227() {
        let harness = Harness::new();
        let bytes = harness.render(request(240, 64), |kit| {
            kit.fill_rect(
                aurorix_render::skia_safe::Rect::from_xywh(0.0, 0.0, 240.0, 64.0),
                Color::BLACK,
            );
            let style = theme::title_style(32, Color::WHITE);
            kit.draw_text("测试中文 Testify", 8.0, 42.0, &style)
                .expect("插件字体必须可用");
        });
        let pixels = decode(&bytes).expect("解码");
        assert!(
            pixels.ink_ratio(0, 0, 240, 64, 1) > 0.02,
            "文字必须真的画上去"
        );
    }
}
