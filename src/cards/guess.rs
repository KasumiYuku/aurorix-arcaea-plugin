use aurorix_plugin::RenderCard;
use aurorix_render::skia_safe::image::CachingHint;
use aurorix_render::skia_safe::images;
use aurorix_render::skia_safe::{AlphaType, ColorType, Data, ImageInfo, Rect};
use aurorix_render::{CanvasKit, Color, RenderRequest};

use super::sketch;
use super::theme;
use crate::cards::lagged_rng::LaggedRng;
use crate::domain::Song;

pub const GUESS_BASE_W: f64 = 660.0;

pub const GUESS_BASE_H: f64 = 898.0;

pub const GUESS_BASE_PAD: f64 = 56.0;

pub const GUESS_SHRINK: f64 = 0.8;

pub const GUESS_TITLE_SIZE: f64 = 40.0;

pub const GUESS_CODE_SIZE: f64 = 26.0;

pub const GUESS_TITLE_BASE: f64 = 98.0;

pub const GUESS_CODE_BASE: f64 = 90.0;

pub const GUESS_IMAGE_TOP: f64 = 168.0;

pub const GUESS_GROUND_TOP: Color = Color::new(0x0B, 0x10, 0x1C, 0xFF);

const FX_PHASE_SPAN: f64 = 628.0 / 100.0;

pub const GUESS_GROUND_BOTTOM: Color = Color::new(0x04, 0x06, 0x0C, 0xFF);

pub const GUESS_COVER_FALLBACK: Color = Color::new(30, 35, 58, 255);

pub const COL_HAIRLINE: Color = Color::new(128, 152, 210, 46);

pub const COL_ACCENT_PURPLE: Color = Color::new(176, 152, 240, 255);

pub const COL_TEXT_MAIN: Color = Color::new(246, 249, 255, 255);

pub const COL_SH_OUTLINE: Color = Color::new(12, 16, 32, 150);

pub const COL_SH_OUTLINE_DEEP: Color = Color::new(10, 14, 28, 130);

pub const GUESS_BLUR: f32 = 14.0;

pub const GUESS_RESAMPLE: f32 = 16.0;

pub const FRAME_DIM: Color = Color::new(4, 5, 10, 120);

pub const GUESS_TITLE: &str = "Archive";

pub const FX_DEADZONE: f64 = 0.18;

pub const CHART_KEEP: f64 = 0.16;

pub const CHART_REVEAL: f64 = 0.62;

pub const CHART_TINT: u8 = 110;

pub const GUESS_SEED_PATTERN: u64 = 0x9E37_79B9_7F4A_7C15;

pub const FX_SEED_PATTERN: u64 = 0x2545_F491_4F6C_DD1D;

pub fn sx(value: i32) -> i32 {
    (value as f64 * 1.4).round() as i32
}

pub fn sxr(value: f64) -> i32 {
    sx(value.round() as i32)
}

pub fn sf(value: f64) -> f32 {
    (value * 1.4) as f32
}

pub fn canvas_w() -> i32 {
    sxr(GUESS_BASE_W)
}

pub fn canvas_h() -> i32 {
    sxr(GUESS_BASE_H)
}

pub fn out_w() -> i32 {
    (canvas_w() as f64 * GUESS_SHRINK + 0.5) as i32
}

pub fn out_h() -> i32 {
    (canvas_h() as f64 * GUESS_SHRINK + 0.5) as i32
}

pub fn title_base() -> i32 {
    sxr(GUESS_TITLE_BASE)
}

pub fn code_base() -> i32 {
    sxr(GUESS_CODE_BASE)
}

pub fn image_rect() -> Rect {
    let pad = sxr(GUESS_BASE_PAD);
    let side = sxr(GUESS_BASE_W - 2.0 * GUESS_BASE_PAD);
    let top = sxr(GUESS_IMAGE_TOP);
    Rect::from_ltrb(
        pad as f32,
        top as f32,
        (pad + side) as f32,
        (top + side) as f32,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuessStage {
    Question,
    Answer,
}

impl GuessStage {
    pub fn is_answer(self) -> bool {
        matches!(self, GuessStage::Answer)
    }

    pub fn size(self) -> (i32, i32) {
        let _ = self;
        (out_w(), out_h())
    }
}

#[derive(Debug, Clone)]
pub struct GuessInput {
    pub stage: GuessStage,
    pub round: u32,
    pub song_id: String,
    pub title: String,
    pub artist: String,
    pub jacket: Option<String>,
    pub hints: Vec<String>,
    pub answer: String,
    pub correct: bool,
    pub cost: u32,
    pub theme: theme::Theme,
    pub footer_note: String,
    pub seed: Option<i64>,
    pub level: Option<f64>,
    pub zoom: Option<f64>,
    pub interference: Option<f64>,
    pub variant: Option<String>,
    pub code: Option<String>,
}

impl GuessInput {
    pub fn new(stage: GuessStage) -> Self {
        Self {
            stage,
            round: 1,
            song_id: String::new(),
            title: String::new(),
            artist: String::new(),
            jacket: None,
            hints: Vec::new(),
            answer: String::new(),
            correct: false,
            cost: 0,
            theme: theme::Theme::DEFAULT,
            footer_note: String::new(),
            seed: None,
            level: None,
            zoom: None,
            interference: None,
            variant: None,
            code: None,
        }
    }

    pub fn question() -> Self {
        GuessInput::new(GuessStage::Question)
    }

    pub fn answer() -> Self {
        GuessInput::new(GuessStage::Answer)
    }

    pub fn of(song: &Song, stage: GuessStage) -> Self {
        let mut input = GuessInput::new(stage);
        input.song_id = song.id.clone();
        input.title = song.title.clone();
        input.artist = song.artist.clone();
        if !song.id.trim().is_empty() {
            input.jacket = Some(format!("jackets/{}.jpg", song.id));
        }
        input
    }

    pub fn with_round(mut self, round: u32) -> Self {
        self.round = round.max(1);
        self
    }

    pub fn with_song(mut self, song_id: impl Into<String>) -> Self {
        self.song_id = song_id.into();
        self
    }

    pub fn with_jacket(mut self, key: impl Into<String>) -> Self {
        let key = key.into();
        self.jacket = (!key.trim().is_empty()).then_some(key);
        self
    }

    pub fn without_jacket(mut self) -> Self {
        self.jacket = None;
        self
    }

    pub fn with_hints(mut self, hints: &[&str]) -> Self {
        self.hints = hints.iter().map(|hint| hint.to_string()).collect();
        self
    }

    pub fn with_answer(mut self, answer: impl Into<String>, correct: bool) -> Self {
        self.answer = answer.into();
        self.correct = correct;
        self
    }

    pub fn with_seed(mut self, seed: i64) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn with_level(mut self, level: f64) -> Self {
        self.level = Some(level);
        self
    }

    pub fn with_zoom(mut self, zoom: f64) -> Self {
        self.zoom = Some(zoom);
        self
    }

    pub fn with_interference(mut self, interference: f64) -> Self {
        self.interference = Some(interference);
        self
    }

    pub fn with_variant(mut self, variant: impl Into<String>) -> Self {
        self.variant = Some(variant.into());
        self
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    pub fn with_theme(mut self, theme_value: theme::Theme) -> Self {
        self.theme = theme_value;
        self
    }

    pub fn with_footer(mut self, note: impl Into<String>) -> Self {
        self.footer_note = note.into();
        self
    }

    pub fn final_frame(&self) -> bool {
        self.stage.is_answer()
    }

    pub fn seed_value(&self) -> i64 {
        match self.seed {
            Some(seed) => seed,
            None => i64::from(self.round) * 0x9E37_79B9,
        }
    }

    pub fn level_value(&self) -> f64 {
        self.level.unwrap_or(1.0).clamp(0.0, 1.0)
    }

    pub fn zoom_value(&self) -> f64 {
        self.zoom.unwrap_or(1.0).clamp(0.05, 1.0)
    }

    pub fn interference_value(&self) -> f64 {
        self.interference.unwrap_or(0.0).clamp(0.0, 1.0)
    }

    pub fn variant_name(&self) -> &str {
        self.variant.as_deref().unwrap_or("glitch")
    }

    pub fn code_text(&self) -> String {
        if let Some(code) = self.code.as_deref() {
            let trimmed = code.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        format!("{:06X}", (self.seed_value() as u64 & 0x00FF_FFFF) as u32)
    }

    pub fn jacket_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = Vec::new();
        if let Some(key) = self.jacket.as_deref() {
            let trimmed = key.trim();
            if !trimmed.is_empty() {
                keys.push(trimmed.to_string());
            }
        }
        let id = self.song_id.trim();
        if !id.is_empty() {
            for key in [
                format!("jackets/{id}.jpg"),
                format!("jacket_thumbs/{id}.jpg"),
            ] {
                if !keys.contains(&key) {
                    keys.push(key);
                }
            }
        }
        keys
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuessWindow {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

pub fn fragment_center(seed: i64) -> (f64, f64) {
    let mut pcg = Pcg::new(seed as u64, GUESS_SEED_PATTERN);
    let x = unit(pcg.next());
    let y = unit(pcg.next());
    (0.28 + 0.44 * x, 0.28 + 0.44 * y)
}

pub fn guess_window(frame: Rect, seed: i64, reveal: f64) -> Option<GuessWindow> {
    if reveal <= 0.0 || reveal >= 1.0 || frame.width() <= 0.0 || frame.height() <= 0.0 {
        return None;
    }
    let (cx, cy) = fragment_center(seed);
    let left = (0.0f64).max((cx - reveal / 2.0).min(1.0 - reveal));
    let top = (0.0f64).max((cy - reveal / 2.0).min(1.0 - reveal));
    let dw = frame.width() as f64;
    let dh = frame.height() as f64;
    let x0 = frame.left() + ((left * dw) + 0.5) as f32;
    let y0 = frame.top() + ((top * dh) + 0.5) as f32;
    let x1 = frame.left() + (((left + reveal) * dw) + 0.5) as f32;
    let y1 = frame.top() + (((top + reveal) * dh) + 0.5) as f32;
    if x1 - x0 < 12.0 || y1 - y0 < 12.0 {
        return None;
    }
    Some(GuessWindow { x0, y0, x1, y1 })
}

fn unit(value: u64) -> f64 {
    (value & 0x1F_FFFF_FFFF_FFFF) as f64 / 9_007_199_254_740_992.0
}

fn clamp8(value: f64) -> u8 {
    if value <= 0.0 {
        0
    } else if value >= 255.0 {
        255
    } else {
        (value + 0.5) as u8
    }
}

fn clamp8_f32(value: f32) -> u8 {
    let raw = value as i64;
    raw.clamp(0, 255) as u8
}

fn mix_byte(a: u8, b: u8, t: f64) -> u8 {
    clamp8(a as f64 * (1.0 - t) + b as f64 * t)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn luma(c: [u8; 4]) -> f64 {
    0.299 * c[0] as f64 + 0.587 * c[1] as f64 + 0.114 * c[2] as f64
}

fn scale_int(value: i64, amount: f64) -> i64 {
    (value as f64 * amount).round() as i64
}

fn mul64(a: u64, b: u64) -> (u64, u64) {
    let product = (a as u128) * (b as u128);
    ((product >> 64) as u64, product as u64)
}

struct Pcg {
    hi: u64,
    lo: u64,
}

impl Pcg {
    fn new(hi: u64, lo: u64) -> Self {
        Self { hi, lo }
    }

    fn advance(&mut self) -> (u64, u64) {
        const MUL_HI: u64 = 2_549_297_995_355_413_924;
        const MUL_LO: u64 = 4_865_540_595_714_422_341;
        const INC_HI: u64 = 6_364_136_223_846_793_005;
        const INC_LO: u64 = 1_442_695_040_888_963_407;
        let (mut hi, mut lo) = mul64(self.lo, MUL_LO);
        hi = hi
            .wrapping_add(self.hi.wrapping_mul(MUL_LO))
            .wrapping_add(self.lo.wrapping_mul(MUL_HI));
        let (add_lo, carry) = lo.overflowing_add(INC_LO);
        lo = add_lo;
        hi = hi.wrapping_add(INC_HI).wrapping_add(u64::from(carry));
        self.hi = hi;
        self.lo = lo;
        (hi, lo)
    }

    fn next(&mut self) -> u64 {
        const CHEAP_MUL: u64 = 0xda94_2042_e4dd_58b5;
        let (mut hi, lo) = self.advance();
        hi ^= hi >> 32;
        hi = hi.wrapping_mul(CHEAP_MUL);
        hi ^= hi >> 48;
        hi = hi.wrapping_mul(lo | 1);
        hi
    }
}

struct Rand {
    pcg: Pcg,
}

impl Rand {
    fn new(seed: i64) -> Self {
        Self {
            pcg: Pcg::new(seed as u64, FX_SEED_PATTERN),
        }
    }

    fn float(&mut self) -> f64 {
        unit(self.pcg.next())
    }

    fn uint64n(&mut self, n: u64) -> u64 {
        if n == 0 {
            return 0;
        }
        if n & (n - 1) == 0 {
            return self.pcg.next() & (n - 1);
        }
        let (mut hi, mut lo) = mul64(self.pcg.next(), n);
        if lo < n {
            let threshold = n.wrapping_neg() % n;
            while lo < threshold {
                let (next_hi, next_lo) = mul64(self.pcg.next(), n);
                hi = next_hi;
                lo = next_lo;
            }
        }
        hi
    }

    fn int_n(&mut self, n: usize) -> usize {
        self.uint64n(n as u64) as usize
    }

    fn perm(&mut self, n: usize) -> Vec<usize> {
        let mut out = vec![0usize; n];
        for i in 1..n {
            let j = self.int_n(i + 1);
            out[i] = out[j];
            out[j] = i;
        }
        out
    }
}

#[derive(Clone)]
struct Plate {
    w: usize,
    h: usize,
    px: Vec<u8>,
}

impl Plate {
    fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            px: vec![0; w * h * 4],
        }
    }

    fn from_rgba(w: usize, h: usize, px: Vec<u8>) -> Self {
        Self { w, h, px }
    }

    fn at(&self, x: usize, y: usize) -> [u8; 4] {
        let offset = (y * self.w + x) * 4;
        [
            self.px[offset],
            self.px[offset + 1],
            self.px[offset + 2],
            self.px[offset + 3],
        ]
    }

    fn set(&mut self, x: usize, y: usize, color: [u8; 4]) {
        let offset = (y * self.w + x) * 4;
        self.px[offset] = color[0];
        self.px[offset + 1] = color[1];
        self.px[offset + 2] = color[2];
        self.px[offset + 3] = color[3];
    }

    fn sample(&self, fx: f64, fy: f64) -> [u8; 4] {
        if self.w == 0 || self.h == 0 {
            return [0, 0, 0, 255];
        }
        let fx = fx.clamp(0.0, (self.w - 1) as f64);
        let fy = fy.clamp(0.0, (self.h - 1) as f64);
        let x0 = fx as usize;
        let y0 = fy as usize;
        let tx = fx - x0 as f64;
        let ty = fy - y0 as f64;
        let x1 = (x0 + 1).min(self.w - 1);
        let y1 = (y0 + 1).min(self.h - 1);
        let c00 = self.at(x0, y0);
        let c10 = self.at(x1, y0);
        let c01 = self.at(x0, y1);
        let c11 = self.at(x1, y1);
        let mut out = [255u8; 4];
        for channel in 0..3 {
            let top = mix_byte(c00[channel], c10[channel], tx);
            let bottom = mix_byte(c01[channel], c11[channel], tx);
            out[channel] = mix_byte(top, bottom, ty);
        }
        out
    }

    fn map<F: Fn(usize, usize) -> [u8; 4]>(&self, f: F) -> Plate {
        let mut out = self.clone();
        for y in 0..self.h {
            for x in 0..self.w {
                out.set(x, y, f(x, y));
            }
        }
        out
    }

    fn remap<F: Fn(f64, f64) -> (f64, f64)>(&self, at: F) -> Plate {
        self.map(|x, y| {
            let (sx, sy) = at(x as f64, y as f64);
            self.sample(sx, sy)
        })
    }

    fn fade(&self, other: &Plate, t: f64) -> Plate {
        self.map(|x, y| {
            let a = self.at(x, y);
            let b = other.at(x, y);
            [
                clamp8(lerp(a[0] as f64, b[0] as f64, t)),
                clamp8(lerp(a[1] as f64, b[1] as f64, t)),
                clamp8(lerp(a[2] as f64, b[2] as f64, t)),
                255,
            ]
        })
    }
}

fn cubic_weights(t: f64) -> [f32; 4] {
    let t2 = t * t;
    let t3 = t2 * t;
    [
        (-0.5 * t3 + t2 - 0.5 * t) as f32,
        (1.5 * t3 - 2.5 * t2 + 1.0) as f32,
        (-1.5 * t3 + 2.0 * t2 + 0.5 * t) as f32,
        (0.5 * t3 - 0.5 * t2) as f32,
    ]
}

fn wsum(m: [[f32; 4]; 4], w: [f32; 4], channel: usize) -> f32 {
    m[0][channel] * w[0]
        + m[1][channel] * w[1]
        + m[2][channel] * w[2]
        + m[3][channel] * w[3]
}

fn filter_row(src: &Plate, y: usize, ix: &[[usize; 4]], wx: &[[f32; 4]], dw: usize) -> Vec<[f32; 4]> {
    let mut out = vec![[0f32; 4]; dw];
    for x in 0..dw {
        let mut acc = [0f32; 4];
        for k in 0..4 {
            let c = src.at(ix[x][k], y);
            let a = c[3] as f32;
            let weight = wx[x][k];
            for channel in 0..3 {
                acc[channel] += (c[channel] as f32 * a / 255.0) * weight;
            }
            acc[3] += a * weight;
        }
        out[x] = acc;
    }
    out
}

fn resize_cubic(src: &Plate, dw: usize, dh: usize) -> Plate {
    let sx = src.w as f64 / dw as f64;
    let sy = src.h as f64 / dh as f64;
    let mut wx = vec![[0f32; 4]; dw];
    let mut ix = vec![[0usize; 4]; dw];
    for x in 0..dw {
        let fx = (x as f64 + 0.5) * sx - 0.5;
        let base = fx.floor();
        wx[x] = cubic_weights(fx - base);
        for (k, slot) in ix[x].iter_mut().enumerate() {
            let index = base as i64 - 1 + k as i64;
            *slot = index.clamp(0, src.w as i64 - 1) as usize;
        }
    }
    let mut out = Plate::new(dw, dh);
    for y in 0..dh {
        let fy = (y as f64 + 0.5) * sy - 0.5;
        let base = fy.floor();
        let wy = cubic_weights(fy - base);
        let mut rows: Vec<Vec<[f32; 4]>> = Vec::with_capacity(4);
        for k in 0..4 {
            let index = base as i64 - 1 + k as i64;
            let row = index.clamp(0, src.h as i64 - 1) as usize;
            rows.push(filter_row(src, row, &ix, &wx, dw));
        }
        for x in 0..dw {
            let m = [rows[0][x], rows[1][x], rows[2][x], rows[3][x]];
            let a0 = wsum(m, wy, 0);
            let a1 = wsum(m, wy, 1);
            let a2 = wsum(m, wy, 2);
            let a3 = wsum(m, wy, 3);
            if a3 <= 0.5 {
                continue;
            }
            let inv = 255.0f32 / a3;
            out.set(
                x,
                y,
                [
                    clamp8_f32(a0 * inv + 0.5),
                    clamp8_f32(a1 * inv + 0.5),
                    clamp8_f32(a2 * inv + 0.5),
                    clamp8_f32(a3 + 0.5),
                ],
            );
        }
    }
    out
}

fn resize_area(src: &Plate, dw: usize, dh: usize) -> Plate {
    let sx = src.w as f64 / dw as f64;
    let sy = src.h as f64 / dh as f64;
    let mut out = Plate::new(dw, dh);
    for oy in 0..dh {
        let y0 = (oy as f64 * sy) as usize;
        let mut y1 = ((oy + 1) as f64 * sy) as usize;
        if y1 <= y0 {
            y1 = y0 + 1;
        }
        if y1 > src.h {
            y1 = src.h;
        }
        for ox in 0..dw {
            let x0 = (ox as f64 * sx) as usize;
            let mut x1 = ((ox + 1) as f64 * sx) as usize;
            if x1 <= x0 {
                x1 = x0 + 1;
            }
            if x1 > src.w {
                x1 = src.w;
            }
            let mut sr = 0f64;
            let mut sg = 0f64;
            let mut sb = 0f64;
            let mut sa = 0f64;
            let mut n = 0f64;
            for yy in y0..y1 {
                for xx in x0..x1 {
                    let c = src.at(xx, yy);
                    let a = c[3] as f64;
                    sr += c[0] as f64 * a;
                    sg += c[1] as f64 * a;
                    sb += c[2] as f64 * a;
                    sa += a;
                    n += 1.0;
                }
            }
            if n == 0.0 || sa == 0.0 {
                continue;
            }
            out.set(
                ox,
                oy,
                [
                    clamp8(sr / sa + 0.5),
                    clamp8(sg / sa + 0.5),
                    clamp8(sb / sa + 0.5),
                    clamp8(sa / n + 0.5),
                ],
            );
        }
    }
    out
}

fn resize_to(src: &Plate, dw: usize, dh: usize) -> Plate {
    if dw == 0 || dh == 0 {
        return src.clone();
    }
    if src.w == dw && src.h == dh {
        return src.clone();
    }
    let sx = src.w as f64 / dw as f64;
    let sy = src.h as f64 / dh as f64;
    if sx >= 1.0 && sy >= 1.0 {
        resize_area(src, dw, dh)
    } else {
        resize_cubic(src, dw, dh)
    }
}

fn crop_plate(src: &Plate, x0: usize, y0: usize, w: usize, h: usize) -> Plate {
    let mut out = Plate::new(w, h);
    for y in 0..h {
        for x in 0..w {
            out.set(x, y, src.at(x0 + x, y0 + y));
        }
    }
    out
}

fn cover_crop(sw: usize, sh: usize, dw: usize, dh: usize) -> Option<(usize, usize, usize, usize)> {
    if sw == 0 || sh == 0 || dw == 0 || dh == 0 {
        return None;
    }
    let scale = (dw as f64 / sw as f64).max(dh as f64 / sh as f64);
    let use_w = (dw as f64 / scale).min(sw as f64);
    let use_h = (dh as f64 / scale).min(sh as f64);
    let dx = (sw as f64 - use_w) / 2.0;
    let dy = (sh as f64 - use_h) / 2.0;
    let x0 = dx as usize;
    let y0 = dy as usize;
    let x1 = ((dx + use_w + 0.5) as usize).min(sw);
    let y1 = ((dy + use_h + 0.5) as usize).min(sh);
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    Some((x0, y0, x1 - x0, y1 - y0))
}

fn asset_plate(kit: &CanvasKit<'_>, key: &str) -> Option<Plate> {
    let texture = kit.asset(key).ok()?;
    let image = texture.image()?;
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
    let stride = width as usize * 4;
    let mut data = vec![0u8; stride * height as usize];
    if !image.read_pixels(&info, &mut data, stride, (0, 0), CachingHint::Allow) {
        return None;
    }
    Some(Plate::from_rgba(width as usize, height as usize, data))
}

fn guess_art(kit: &CanvasKit<'_>, input: &GuessInput, frame: Rect) -> Option<Plate> {
    let dw = frame.width().max(1.0) as usize;
    let dh = frame.height().max(1.0) as usize;
    for key in input.jacket_keys() {
        let Some(raw) = asset_plate(kit, &key) else {
            continue;
        };
        let Some((x0, y0, w, h)) = cover_crop(raw.w, raw.h, dw, dh) else {
            continue;
        };
        let crop = crop_plate(&raw, x0, y0, w, h);
        return Some(resize_to(&crop, dw, dh));
    }
    None
}

fn draw_plate(kit: &CanvasKit<'_>, plate: &Plate, frame: Rect) {
    if plate.w == 0 || plate.h == 0 {
        return;
    }
    let info = ImageInfo::new(
        (plate.w as i32, plate.h as i32),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let Some(image) = images::raster_from_data(&info, Data::new_copy(&plate.px), plate.w * 4) else {
        return;
    };
    kit.image_rect(&image, frame);
}

fn zoom_step(src: &Plate, reveal: f64, cx: f64, cy: f64) -> Plate {
    if src.w < 2 || src.h < 2 {
        return src.clone();
    }
    let w = src.w as f64;
    let h = src.h as f64;
    let span_x = w * reveal;
    let span_y = h * reveal;
    let left = (cx * w - span_x / 2.0).clamp(0.0, w - span_x);
    let top = (cy * h - span_y / 2.0).clamp(0.0, h - span_y);
    let mut out = Plate::new(src.w, src.h);
    for y in 0..src.h {
        let sy = top + (y as f64 + 0.5) / h * span_y;
        for x in 0..src.w {
            let sx = left + (x as f64 + 0.5) / w * span_x;
            out.set(x, y, src.sample(sx, sy));
        }
    }
    out
}

fn resample_step(src: Plate, ratio: f64, amount: f64) -> Plate {
    if amount <= 0.0 || ratio <= 0.0 {
        return src;
    }
    let sc = 1.0 / (1.0 + ratio * amount);
    let sw = ((src.w as f64 * sc + 0.5) as usize).max(8);
    let sh = ((src.h as f64 * sc + 0.5) as usize).max(8);
    let small = resize_to(&src, sw, sh);
    resize_to(&small, src.w, src.h)
}

fn blur_div(sums: i64, n: i64) -> u8 {
    if n <= 0 {
        return 0;
    }
    let value = if n <= 1024 {
        (sums as u32 as u64 * ((1u64 << 48) / n as u64 + 1)) >> 48
    } else {
        (sums / n) as u64
    };
    value.min(255) as u8
}

fn box_blur(src: Plate, radius: i64) -> Plate {
    let w = src.w as i64;
    let h = src.h as i64;
    if radius < 1 || w <= 0 || h <= 0 {
        return src;
    }
    let rad = radius + 1;
    let mut img = src;
    let mut ring: Vec<[u8; 3]> = vec![[0; 3]; rad as usize];
    for x in 0..w {
        let top = radius.min(h - 1);
        let mut sums = [0i64; 3];
        let mut n = 0i64;
        for y in 0..=top {
            let c = img.at(x as usize, y as usize);
            for channel in 0..3 {
                sums[channel] += c[channel] as i64;
            }
            n += 1;
        }
        for y in 0..h {
            let c = img.at(x as usize, y as usize);
            ring[(y % rad) as usize] = [c[0], c[1], c[2]];
            img.set(
                x as usize,
                y as usize,
                [
                    blur_div(sums[0], n),
                    blur_div(sums[1], n),
                    blur_div(sums[2], n),
                    c[3],
                ],
            );
            let drop = y - radius;
            if drop >= 0 {
                let gone = ring[(drop % rad) as usize];
                for channel in 0..3 {
                    sums[channel] -= gone[channel] as i64;
                }
                n -= 1;
            }
            let add = y + radius + 1;
            if add < h {
                let c = img.at(x as usize, add as usize);
                for channel in 0..3 {
                    sums[channel] += c[channel] as i64;
                }
                n += 1;
            }
        }
    }
    for y in 0..h {
        let top = radius.min(w - 1);
        let mut sums = [0i64; 3];
        let mut n = 0i64;
        for x in 0..=top {
            let c = img.at(x as usize, y as usize);
            for channel in 0..3 {
                sums[channel] += c[channel] as i64;
            }
            n += 1;
        }
        for x in 0..w {
            let c = img.at(x as usize, y as usize);
            ring[(x % rad) as usize] = [c[0], c[1], c[2]];
            img.set(
                x as usize,
                y as usize,
                [
                    blur_div(sums[0], n),
                    blur_div(sums[1], n),
                    blur_div(sums[2], n),
                    c[3],
                ],
            );
            let drop = x - radius;
            if drop >= 0 {
                let gone = ring[(drop % rad) as usize];
                for channel in 0..3 {
                    sums[channel] -= gone[channel] as i64;
                }
                n -= 1;
            }
            let add = x + radius + 1;
            if add < w {
                let c = img.at(add as usize, y as usize);
                for channel in 0..3 {
                    sums[channel] += c[channel] as i64;
                }
                n += 1;
            }
        }
    }
    img
}

fn blur_step(src: Plate, radius: f64, amount: f64) -> Plate {
    let r = (radius * amount + 0.5) as i64;
    if r < 2 {
        return src;
    }
    box_blur(src, r)
}

fn mottle(img: &mut Plate, cell: usize, amp: f64, rng: &mut Rand) {
    let cell = cell.max(1);
    let sw = img.w / cell + 2;
    let sh = img.h / cell + 2;
    let mut stain = Plate::new(sw, sh);
    for y in 0..sh {
        for x in 0..sw {
            let v = clamp8(128.0 + (rng.float() * 2.0 - 1.0) * amp);
            stain.set(x, y, [v, v, v, 255]);
        }
    }
    for y in 0..img.h {
        let fy = (y as f64 + 0.5) / cell as f64;
        for x in 0..img.w {
            let d = stain.sample((x as f64 + 0.5) / cell as f64, fy)[0] as f64 - 128.0;
            let c = img.at(x, y);
            img.set(
                x,
                y,
                [
                    clamp8(c[0] as f64 + d),
                    clamp8(c[1] as f64 + d),
                    clamp8(c[2] as f64 + d),
                    c[3],
                ],
            );
        }
    }
}

fn streaks(img: &mut Plate, amount: f64, rng: &mut Rand) {
    let w = img.w as i64;
    let h = img.h as i64;
    if w <= 0 || h <= 0 {
        return;
    }
    let count = 3 + rng.int_n(4) as i64;
    for _ in 0..count {
        let y0 = rng.int_n(h as usize) as i64;
        let rows_bound = (h as f64 * 0.02) as i64;
        let rows = 1 + rng.int_n(rows_bound.max(1) as usize) as i64;
        let x0 = rng.int_n(w as usize) as i64;
        let width = (w / 2).max(1) as usize;
        let x1 = x0 + w / 4 + rng.int_n(width) as i64;
        let d = (rng.float() * 2.0 - 1.0) * amount * 90.0;
        for y in y0..(y0 + rows).min(h) {
            for x in x0..x1.min(w) {
                let c = img.at(x as usize, y as usize);
                img.set(
                    x as usize,
                    y as usize,
                    [
                        clamp8(c[0] as f64 + d),
                        clamp8(c[1] as f64 + d),
                        clamp8(c[2] as f64 + d),
                        c[3],
                    ],
                );
            }
        }
    }
}

fn grain_step(src: Plate, cell: usize, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 {
        return src;
    }
    let mut out = src;
    mottle(&mut out, cell * 24, amount * 31.0, rng);
    streaks(&mut out, amount * 0.22, rng);
    out
}

fn channel_shift(img: &mut Plate, dr: i64, dg: i64, db: i64) {
    let w = img.w as i64;
    let h = img.h as i64;
    let src = img.clone();
    for y in 0..h {
        for x in 0..w {
            let r = src.at((x + dr).clamp(0, w - 1) as usize, y as usize)[0];
            let g = src.at((x + dg).clamp(0, w - 1) as usize, y as usize)[1];
            let b = src.at((x + db).clamp(0, w - 1) as usize, y as usize)[2];
            let a = img.at(x as usize, y as usize)[3];
            img.set(x as usize, y as usize, [r, g, b, a]);
        }
    }
}

fn band_shift(img: &mut Plate, y0: i64, rows: i64, dx: i64) {
    if dx == 0 {
        return;
    }
    let w = img.w as i64;
    let h = img.h as i64;
    let src = img.clone();
    for y in y0.max(0)..(y0 + rows).min(h) {
        for x in 0..w {
            let c = src.at((x + dx).clamp(0, w - 1) as usize, y as usize);
            let a = img.at(x as usize, y as usize)[3];
            img.set(x as usize, y as usize, [c[0], c[1], c[2], a]);
        }
    }
}

fn variant_glitch(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.w == 0 || src.h == 0 {
        return src;
    }
    let w = src.w as i64;
    let h = src.h as i64;
    let amp = 2 + (34.0 * rng.float()) as i64;
    let bands_len = 2 + rng.int_n(8) as i64;
    let rows_bound = 4i64.max((h as f64 * 0.06) as i64);
    let mut bands: Vec<(i64, i64, i64)> = Vec::with_capacity(bands_len as usize);
    for _ in 0..bands_len {
        let rows = 4 + rng.int_n(rows_bound as usize) as i64;
        let y0 = rng.int_n((h - rows).max(1) as usize) as i64;
        let dx = ((rng.float() * 2.0 - 1.0) * w as f64 * 0.22) as i64;
        bands.push((y0, rows, dx));
    }
    let mut out = src;
    let span = 2 * amp + 1;
    let dr = scale_int(rng.int_n(span as usize) as i64 - amp, amount);
    let db = -scale_int(rng.int_n(span as usize) as i64 - amp, amount);
    channel_shift(&mut out, dr, 0, db);
    for (y0, rows, dx) in bands {
        band_shift(&mut out, y0, rows, scale_int(dx, amount));
    }
    out
}

fn variant_ripple(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    use std::f64::consts::PI;
    if amount <= 0.0 {
        return src;
    }
    let w = src.w as f64;
    let h = src.h as f64;
    let cx = w * (0.35 + 0.30 * rng.float());
    let cy = h * (0.35 + 0.30 * rng.float());
    let amp = (0.05 + 0.09 * rng.float()) * w * amount;
    let lambda = (0.15 + 0.25 * rng.float()) * w;
    let bend = (0.5 + 1.1 * rng.float()) * amount;
    let mut out = Plate::new(src.w, src.h);
    for y in 0..src.h {
        let fy = y as f64;
        for x in 0..src.w {
            let fx = x as f64;
            let mut sx = fx + amp * (2.0 * PI * fy / lambda).sin();
            let mut sy = fy + amp * (2.0 * PI * fx / lambda).cos();
            let dx = sx - cx;
            let dy = sy - cy;
            let dist = dx.hypot(dy);
            if dist > 0.0 {
                let f = (dist / (w / 2.0)).powf(2.0 * bend);
                sx = cx + dx * f;
                sy = cy + dy * f;
            }
            out.set(x, y, src.sample(sx, sy));
        }
    }
    out
}

fn variant_kaleido(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    use std::f64::consts::PI;
    if amount <= 0.0 {
        return src;
    }
    let w = src.w as f64;
    let h = src.h as f64;
    let cx = w * (0.5 + (rng.float() - 0.5) * 0.20);
    let cy = h * (0.5 + (rng.float() - 0.5) * 0.20);
    let lobes = 3.0 + rng.int_n(6) as f64;
    let seg = 2.0 * PI / lobes;
    let r_max = cx.max(w - cx).hypot(cy.max(h - cy));
    let mut out = Plate::new(src.w, src.h);
    for y in 0..src.h {
        let fy = y as f64;
        for x in 0..src.w {
            let fx = x as f64;
            let dx = fx - cx;
            let dy = fy - cy;
            let mut a = dy.atan2(dx) % seg;
            if a < 0.0 {
                a += seg;
            }
            if a > seg / 2.0 {
                a = seg - a;
            }
            let u = (a * lobes / (2.0 * PI) + 0.5) * w;
            let v = dx.hypot(dy) / r_max * h;
            out.set(x, y, src.sample(lerp(fx, u, amount), lerp(fy, v, amount)));
        }
    }
    out
}

fn variant_outline(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 {
        return src;
    }
    let gain = 4.0 + 14.0 * rng.float();
    let w = src.w;
    let h = src.h;
    let mut gray = vec![0f64; w * h];
    for y in 0..h {
        for x in 0..w {
            gray[y * w + x] = luma(src.at(x, y));
        }
    }
    let mut lines = Plate::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let gx = gray[y * w + (x + 1).min(w - 1)] - gray[y * w + x.saturating_sub(1)];
            let gy = gray[(y + 1).min(h - 1) * w + x] - gray[y.saturating_sub(1) * w + x];
            let line = 255 - clamp8(gx.hypot(gy) * gain);
            lines.set(x, y, [line, line, line, 255]);
        }
    }
    src.fade(&lines, amount)
}

fn variant_hue(src: Plate, amount: f64) -> Plate {
    use std::f64::consts::PI;
    if amount <= 0.0 {
        return src;
    }
    let cos = (amount * PI).cos();
    let sin = (amount * PI).sin();
    let m = [
        0.213 + cos * 0.787 - sin * 0.213,
        0.715 - cos * 0.715 - sin * 0.715,
        0.072 - cos * 0.072 + sin * 0.928,
        0.213 - cos * 0.213 + sin * 0.143,
        0.715 + cos * 0.285 + sin * 0.140,
        0.072 - cos * 0.072 - sin * 0.283,
        0.213 - cos * 0.213 - sin * 0.787,
        0.715 - cos * 0.715 + sin * 0.715,
        0.072 + cos * 0.928 + sin * 0.072,
    ];
    let mut out = src;
    for index in (0..out.px.len()).step_by(4) {
        let r = out.px[index] as f64;
        let g = out.px[index + 1] as f64;
        let b = out.px[index + 2] as f64;
        out.px[index] = clamp8(m[0] * r + m[1] * g + m[2] * b);
        out.px[index + 1] = clamp8(m[3] * r + m[4] * g + m[5] * b);
        out.px[index + 2] = clamp8(m[6] * r + m[7] * g + m[8] * b);
    }
    out
}

fn spread(src: &Plate) -> Vec<[u8; 4]> {
    let total = src.w * src.h;
    let step = if total > 6000 { total / 6000 + 1 } else { 1 };
    let mut out = Vec::with_capacity(6000);
    let mut index = 0;
    while index < total {
        let c = src.at(index % src.w, index / src.w);
        out.push([c[0], c[1], c[2], 255]);
        index += step;
    }
    out
}

fn nearest(color: [u8; 4], centers: &[[u8; 4]]) -> usize {
    let mut best = 0usize;
    let mut best_distance = f64::INFINITY;
    for (index, center) in centers.iter().enumerate() {
        let dr = color[0] as f64 - center[0] as f64;
        let dg = color[1] as f64 - center[1] as f64;
        let db = color[2] as f64 - center[2] as f64;
        let distance = dr * dr + dg * dg + db * db;
        if distance < best_distance {
            best = index;
            best_distance = distance;
        }
    }
    best
}

fn dominant(src: &Plate, k: usize) -> Vec<[u8; 4]> {
    if src.w == 0 || src.h == 0 || k == 0 {
        return Vec::new();
    }
    let pool = spread(src);
    if pool.is_empty() {
        return Vec::new();
    }
    let k = k.min(pool.len());
    let seed = (pool.len() as i64)
        .wrapping_mul(2_654_435_761)
        .wrapping_add(17);
    let mut init = LaggedRng::new(seed);
    let mut centers: Vec<[u8; 4]> = Vec::with_capacity(k);
    for _ in 0..k {
        centers.push(pool[init.intn(pool.len())]);
    }
    let mut counts = vec![0usize; k];
    let mut sums = vec![[0i64; 3]; k];
    for _ in 0..6 {
        for count in counts.iter_mut() {
            *count = 0;
        }
        for sum in sums.iter_mut() {
            *sum = [0; 3];
        }
        for sample in &pool {
            let index = nearest(*sample, &centers);
            for channel in 0..3 {
                sums[index][channel] += sample[channel] as i64;
            }
            counts[index] += 1;
        }
        for index in 0..k {
            if counts[index] == 0 {
                continue;
            }
            centers[index] = [
                (sums[index][0] / counts[index] as i64) as u8,
                (sums[index][1] / counts[index] as i64) as u8,
                (sums[index][2] / counts[index] as i64) as u8,
                255,
            ];
        }
    }
    let mut order: Vec<usize> = (0..k).filter(|index| counts[*index] > 0).collect();
    order.sort_by(|a, b| counts[*b].cmp(&counts[*a]));
    order.into_iter().map(|index| centers[index]).collect()
}

fn quantize(src: &Plate, k: usize) -> Plate {
    let centers = dominant(src, k);
    if centers.is_empty() {
        return src.clone();
    }
    src.map(|x, y| {
        let c = src.at(x, y);
        let center = centers[nearest([c[0], c[1], c[2], 255], &centers)];
        [center[0], center[1], center[2], c[3]]
    })
}

fn cross_fade(a: &Plate, b: &Plate, t: f64) -> Plate {
    let mut out = a.clone();
    let len = out.px.len().min(b.px.len());
    let mut index = 0;
    while index + 3 < len {
        for channel in 0..3 {
            out.px[index + channel] = mix_byte(out.px[index + channel], b.px[index + channel], t);
        }
        index += 4;
    }
    out
}

fn variant_wash_shuffle(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    let washed = if amount <= 0.0 {
        src.clone()
    } else {
        let k = 4 + rng.int_n(8);
        let limited = quantize(&src, k);
        cross_fade(&src, &limited, amount)
    };
    shuffle(washed, amount, rng)
}

fn shuffle(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 {
        return src;
    }
    let w = src.w as i64;
    let h = src.h as i64;
    let width = w as f64 / (8 + rng.int_n(9)) as f64;
    let n = 2i64.max((w as f64 / width).ceil() as i64);
    let perm = rng.perm(n as usize);
    let mut out = Plate::new(src.w, src.h);
    for dst in 0..n {
        let from = perm[dst as usize] as i64;
        let x0 = (dst as f64 * width) as i64;
        let x1 = (((dst + 1) as f64 * width) as i64).min(w);
        for x in x0..x1 {
            let target = ((from as f64 * width) as i64 + x - x0).min(w - 1);
            let sx = lerp(x as f64, target as f64, amount);
            for y in 0..h {
                out.set(x as usize, y as usize, src.sample(sx, y as f64));
            }
        }
    }
    out
}

struct ChartShape {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    radius: f64,
    lean: f64,
}

fn shape_span(shape: &ChartShape, y: i64) -> Option<(f64, f64)> {
    let cy = y as f64 + 0.5;
    let top = shape.y;
    let bottom = shape.y + shape.h;
    if cy < top || cy > bottom {
        return None;
    }
    let dx = if shape.lean != 0.0 {
        shape.lean * (shape.y + shape.h / 2.0 - cy)
    } else {
        0.0
    };
    let mut x0 = shape.x + dx;
    let mut x1 = shape.x + shape.w + dx;
    if shape.radius > 0.0 {
        let d = cy - top;
        if d < shape.radius {
            let k = shape.radius
                - (shape.radius * shape.radius - (shape.radius - d) * (shape.radius - d)).sqrt();
            x0 += k;
            x1 -= k;
        }
        let d = bottom - cy;
        if d < shape.radius {
            let k = shape.radius
                - (shape.radius * shape.radius - (shape.radius - d) * (shape.radius - d)).sqrt();
            x0 += k;
            x1 -= k;
        }
    }
    if x1 > x0 {
        Some((x0, x1))
    } else {
        None
    }
}

fn map_shape<F: Fn([u8; 4], usize, usize) -> [u8; 4]>(out: &mut Plate, shape: &ChartShape, f: F) {
    let lo_y = shape.y.floor() as i64;
    let hi_y = (shape.y + shape.h).ceil() as i64;
    for y in lo_y.max(0)..hi_y.min(out.h as i64) {
        let Some((x0, x1)) = shape_span(shape, y) else {
            continue;
        };
        let lo_x = (x0.floor() as i64).max(0);
        let hi_x = (x1.ceil() as i64).min(out.w as i64);
        for x in lo_x..hi_x {
            let cx = x as f64 + 0.5;
            if cx < x0 || cx > x1 {
                continue;
            }
            let current = out.at(x as usize, y as usize);
            let next = f(current, x as usize, y as usize);
            out.set(x as usize, y as usize, next);
        }
    }
}

fn variant_chart(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    use std::f64::consts::PI;
    if amount <= 0.0 || src.w == 0 || src.h == 0 {
        return src;
    }
    let w = src.w as f64;
    let h = src.h as f64;
    let mut shapes: Vec<ChartShape> = Vec::with_capacity(64);
    let groups = 2 + rng.int_n(3);
    for _ in 0..groups {
        let cx = w * (0.2 + 0.6 * rng.float());
        let cy = h * (0.2 + 0.6 * rng.float());
        let angle = PI * (0.15 + 0.7 * rng.float());
        let length = w * (0.5 + 0.5 * rng.float());
        let lean = angle.tan().clamp(-2.5, 2.5);
        shapes.push(ChartShape {
            x: cx - length / 2.0,
            y: cy,
            w: length,
            h: h * 0.004,
            radius: 0.0,
            lean,
        });
        let notes = 8 + rng.int_n(12);
        for note in 0..notes {
            let t = (note as f64 + 0.5) / notes as f64;
            let px = cx + angle.cos() * (t - 0.5) * length;
            let py = cy + angle.sin() * (t - 0.5) * length;
            shapes.push(ChartShape {
                x: px - w * 0.003,
                y: py - h * 0.015,
                w: w * 0.006,
                h: h * 0.030,
                radius: 2.0,
                lean: 0.0,
            });
        }
    }
    let dots = 12 + rng.int_n(18);
    for _ in 0..dots {
        let px = rng.float() * w;
        let py = rng.float() * h;
        let size = w * (0.005 + 0.006 * rng.float());
        shapes.push(ChartShape {
            x: px,
            y: py,
            w: size * 2.0,
            h: size * 2.0,
            radius: size,
            lean: 0.0,
        });
    }
    let keep = 1.0 - (1.0 - CHART_KEEP) * amount;
    let mut out = src.clone();
    let mut index = 0;
    while index + 3 < out.px.len() {
        for channel in 0..3 {
            out.px[index + channel] = clamp8(out.px[index + channel] as f64 * keep);
        }
        index += 4;
    }
    let reveal = CHART_REVEAL * amount;
    let tint = CHART_TINT as f64 / 255.0;
    for shape in &shapes {
        map_shape(&mut out, shape, |current, x, y| {
            let mut c = src.at(x, y);
            for channel in c.iter_mut().take(3) {
                *channel = mix_byte(*channel, 255, tint);
            }
            [
                mix_byte(current[0], c[0], reveal),
                mix_byte(current[1], c[1], reveal),
                mix_byte(current[2], c[2], reveal),
                current[3],
            ]
        });
    }
    out
}

fn block_avg(src: &Plate, x0: usize, y0: usize, bw: usize, bh: usize) -> [u8; 4] {
    let mut sr = 0f64;
    let mut sg = 0f64;
    let mut sb = 0f64;
    let mut n = 0f64;
    for y in y0..(y0 + bh).min(src.h) {
        for x in x0..(x0 + bw).min(src.w) {
            let c = src.at(x, y);
            sr += c[0] as f64;
            sg += c[1] as f64;
            sb += c[2] as f64;
            n += 1.0;
        }
    }
    if n == 0.0 {
        return [0, 0, 0, 255];
    }
    [clamp8(sr / n), clamp8(sg / n), clamp8(sb / n), 255]
}

fn quantile(src: &Plate, q: f64) -> f64 {
    let mut hist = [0usize; 64];
    let mut n = 0usize;
    let mut y = 0;
    while y < src.h {
        let mut x = 0;
        while x < src.w {
            let index = ((luma(src.at(x, y)) / 4.0) as usize).min(63);
            hist[index] += 1;
            n += 1;
            x += 2;
        }
        y += 2;
    }
    if n == 0 {
        return 127.5;
    }
    let want = (n as f64 * q) as usize;
    let mut acc = 0usize;
    for (index, count) in hist.iter().enumerate() {
        acc += count;
        if acc >= want {
            return (index * 4) as f64;
        }
    }
    255.0
}

fn box3(buf: &mut [[u8; 4]], w: usize, h: usize) {
    if w < 3 || h < 3 {
        return;
    }
    let src = buf.to_vec();
    for y in 0..h {
        for x in 0..w {
            let mut sr = 0f64;
            let mut sg = 0f64;
            let mut sb = 0f64;
            for dy in -1i64..=1 {
                for dx in -1i64..=1 {
                    let yy = ((y as i64 + dy).clamp(0, h as i64 - 1)) as usize;
                    let xx = ((x as i64 + dx).clamp(0, w as i64 - 1)) as usize;
                    let c = src[yy * w + xx];
                    sr += c[0] as f64;
                    sg += c[1] as f64;
                    sb += c[2] as f64;
                }
            }
            buf[y * w + x] = [clamp8(sr / 9.0), clamp8(sg / 9.0), clamp8(sb / 9.0), 255];
        }
    }
}

fn sample_grid(buf: &[[u8; 4]], w: usize, h: usize, fx: f64, fy: f64) -> [u8; 4] {
    let fx = fx.clamp(0.0, (w - 1) as f64);
    let fy = fy.clamp(0.0, (h - 1) as f64);
    let x0 = fx as usize;
    let y0 = fy as usize;
    let tx = fx - x0 as f64;
    let ty = fy - y0 as f64;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    let c00 = buf[y0 * w + x0];
    let c10 = buf[y0 * w + x1];
    let c01 = buf[y1 * w + x0];
    let c11 = buf[y1 * w + x1];
    let mut out = [255u8; 4];
    for channel in 0..3 {
        let top = mix_byte(c00[channel], c10[channel], tx);
        let bottom = mix_byte(c01[channel], c11[channel], tx);
        out[channel] = mix_byte(top, bottom, ty);
    }
    out
}

fn fx_twist(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.w < 8 {
        return src;
    }
    let w = src.w as f64;
    let h = src.h as f64;
    let cx = w * (0.3 + 0.4 * rng.float());
    let cy = h * (0.3 + 0.4 * rng.float());
    let radius = w.hypot(h) * (0.3 + 0.5 * rng.float());
    let mut spin = (1.1 + 2.8 * rng.float()) * amount;
    if rng.int_n(2) == 0 {
        spin = -spin;
    }
    let steps = 512usize;
    let mut cos_t = vec![0f64; steps + 1];
    let mut sin_t = vec![0f64; steps + 1];
    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        cos_t[i] = (spin * t * t).cos();
        sin_t[i] = (spin * t * t).sin();
    }
    src.remap(|x, y| {
        let dx = x - cx;
        let dy = y - cy;
        let t = 1.0 - dx.hypot(dy) / radius;
        if t <= 0.0 {
            return (x, y);
        }
        let i = ((t * t * steps as f64) as usize).min(steps);
        (
            cx + dx * cos_t[i] - dy * sin_t[i],
            cy + dx * sin_t[i] + dy * cos_t[i],
        )
    })
}

fn fx_mosaic(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.w < 12 || src.h < 12 {
        return src;
    }
    let cell = 4 + rng.int_n(14);
    let cols = src.w.div_ceil(cell);
    let rows = src.h.div_ceil(cell);
    let mut avg = vec![[0u8; 4]; cols * rows];
    for j in 0..rows {
        for i in 0..cols {
            avg[j * cols + i] = block_avg(&src, i * cell, j * cell, cell, cell);
        }
    }
    let blocks = src.map(|x, y| avg[(y / cell) * cols + x / cell]);
    src.fade(&blocks, amount)
}

fn fx_halftone(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.w < 12 || src.h < 12 {
        return src;
    }
    let cell = 4 + rng.int_n(9);
    let gain = 0.8 + 0.6 * rng.float();
    let cols = src.w.div_ceil(cell);
    let rows = src.h.div_ceil(cell);
    let mut avg = vec![[0u8; 4]; cols * rows];
    for j in 0..rows {
        for i in 0..cols {
            avg[j * cols + i] = block_avg(&src, i * cell, j * cell, cell, cell);
        }
    }
    let dots = src.map(|x, y| {
        let c = avg[(y / cell) * cols + x / cell];
        let bx = x / cell;
        let by = y / cell;
        let rad = cell as f64 * 0.55 * (1.0f64.min(luma(c) / 255.0 * gain)).sqrt();
        let dx = x as f64 + 0.5 - (bx * cell) as f64 - cell as f64 / 2.0;
        let dy = y as f64 + 0.5 - (by * cell) as f64 - cell as f64 / 2.0;
        let cov = (rad - dx.hypot(dy) + 0.5).clamp(0.0, 1.0);
        [
            clamp8(c[0] as f64 * cov),
            clamp8(c[1] as f64 * cov),
            clamp8(c[2] as f64 * cov),
            255,
        ]
    });
    src.fade(&dots, amount)
}

fn fx_mirror(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.w < 16 || src.h < 16 {
        return src;
    }
    let w = src.w as i64;
    let h = src.h as i64;
    let vertical = rng.int_n(2) == 0;
    let mut axis = w / 4 + rng.int_n((w / 2).max(1) as usize) as i64;
    if !vertical {
        axis = h / 4 + rng.int_n((h / 2).max(1) as usize) as i64;
    }
    let flipped = src.remap(|x, y| {
        if vertical {
            (2.0 * axis as f64 - x, y)
        } else {
            (x, 2.0 * axis as f64 - y)
        }
    });
    src.fade(&flipped, amount)
}

fn fx_displace(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.w < 16 {
        return src;
    }
    let cols = src.w / 24 + 2;
    let rows = src.h / 24 + 2;
    let mut field = vec![[0f64; 2]; cols * rows];
    for item in field.iter_mut() {
        *item = [rng.float() * 2.0 - 1.0, rng.float() * 2.0 - 1.0];
    }
    let amp = (0.02 + 0.05 * rng.float()) * src.w as f64 * amount;
    src.remap(|x, y| {
        let gx = (x / 24.0).clamp(0.0, cols as f64 - 2.0);
        let gy = (y / 24.0).clamp(0.0, rows as f64 - 2.0);
        let x0 = gx as usize;
        let y0 = gy as usize;
        let tx = gx - x0 as f64;
        let ty = gy - y0 as f64;
        let pick = |dx: usize, dy: usize| field[(y0 + dy).min(rows - 1) * cols + (x0 + dx).min(cols - 1)];
        let vx = lerp(
            lerp(pick(0, 0)[0], pick(1, 0)[0], tx),
            lerp(pick(0, 1)[0], pick(1, 1)[0], tx),
            ty,
        );
        let vy = lerp(
            lerp(pick(0, 0)[1], pick(1, 0)[1], tx),
            lerp(pick(0, 1)[1], pick(1, 1)[1], tx),
            ty,
        );
        (x + vx * amp, y + vy * amp)
    })
}

fn fx_shatter(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.w < 16 {
        return src;
    }
    let w = src.w as f64;
    let h = src.h as f64;
    let count = 5 + rng.int_n(6);
    let mut shards: Vec<[f64; 4]> = Vec::with_capacity(count);
    for _ in 0..count {
        shards.push([
            rng.float() * w,
            rng.float() * h,
            (rng.float() * 2.0 - 1.0) * w * 0.10 * amount,
            (rng.float() * 2.0 - 1.0) * h * 0.10 * amount,
        ]);
    }
    src.remap(|x, y| {
        let mut best = f64::INFINITY;
        let mut dx = 0f64;
        let mut dy = 0f64;
        for shard in &shards {
            let distance = (x - shard[0]).hypot(y - shard[1]);
            if distance < best {
                best = distance;
                dx = shard[2];
                dy = shard[3];
            }
        }
        (x - dx, y - dy)
    })
}

fn fx_fisheye(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.w < 16 {
        return src;
    }
    let w = src.w as f64;
    let h = src.h as f64;
    let cx = w * (0.35 + 0.3 * rng.float());
    let cy = h * (0.35 + 0.3 * rng.float());
    let k = (0.35 + 0.55 * rng.float()) * amount;
    let norm = 1.0 / (w * w / 4.0);
    src.remap(|x, y| {
        let dx = x - cx;
        let dy = y - cy;
        let f = 1.0 + k * (dx * dx + dy * dy) * norm;
        (cx + dx * f, cy + dy * f)
    })
}

fn fx_liquid(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    use std::f64::consts::PI;
    if amount <= 0.0 || src.w < 16 {
        return src;
    }
    let w = src.w as f64;
    let angle = PI * rng.float();
    let lambda = w * (0.08 + 0.14 * rng.float());
    let amp = w * (0.012 + 0.03 * rng.float()) * amount;
    let ux = angle.cos();
    let uy = angle.sin();
    src.remap(|x, y| {
        let d = amp * (2.0 * PI * (x * ux + y * uy) / lambda).sin();
        (x - d * uy, y + d * ux)
    })
}

fn fx_chroma(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.w < 8 {
        return src;
    }
    let w = src.w as i64;
    let amp = 3i64.max(2 + (w as f64 * 0.03) as i64);
    let split = (2 + rng.int_n((amp - 2) as usize) as i64) as f64 * amount;
    src.map(|x, y| {
        let c = src.at(x, y);
        [
            src.sample(x as f64 + split, y as f64)[0],
            c[1],
            src.sample(x as f64 - split, y as f64)[2],
            255,
        ]
    })
}

fn fx_hue(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    use std::f64::consts::PI;
    if amount <= 0.0 {
        return src;
    }
    let angle = PI * (0.4 + 0.6 * rng.float()) * amount;
    let cos = angle.cos();
    let sin = angle.sin();
    let m = [
        0.213 + cos * 0.787 - sin * 0.213,
        0.715 - cos * 0.715 - sin * 0.715,
        0.072 - cos * 0.072 + sin * 0.928,
        0.213 - cos * 0.213 + sin * 0.143,
        0.715 + cos * 0.285 + sin * 0.140,
        0.072 - cos * 0.072 - sin * 0.283,
        0.213 - cos * 0.213 - sin * 0.787,
        0.715 - cos * 0.715 + sin * 0.715,
        0.072 + cos * 0.928 + sin * 0.072,
    ];
    src.map(|x, y| {
        let c = src.at(x, y);
        let r = c[0] as f64;
        let g = c[1] as f64;
        let b = c[2] as f64;
        [
            clamp8(m[0] * r + m[1] * g + m[2] * b),
            clamp8(m[3] * r + m[4] * g + m[5] * b),
            clamp8(m[6] * r + m[7] * g + m[8] * b),
            255,
        ]
    })
}

fn fx_posterize(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 {
        return src;
    }
    let levels = (2 + rng.int_n(6)) as f64;
    let step = 255.0 / (levels - 1.0);
    let quant = src.map(|x, y| {
        let c = src.at(x, y);
        let q = |v: u8| clamp8((v as f64 / step).round() * step);
        [q(c[0]), q(c[1]), q(c[2]), 255]
    });
    src.fade(&quant, amount)
}

fn fx_dither(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.w < 8 {
        return src;
    }
    let bayer = [
        [0.0, 8.0, 2.0, 10.0],
        [12.0, 4.0, 14.0, 6.0],
        [3.0, 11.0, 1.0, 9.0],
        [15.0, 7.0, 13.0, 5.0],
    ];
    let levels = (2 + 2 * rng.int_n(2)) as f64;
    let step = 255.0 / (levels - 1.0);
    let depth = (0.4 + 0.5 * rng.float()) * step;
    let quant = src.map(|x, y| {
        let c = src.at(x, y);
        let bias = (bayer[y % 4][x % 4] / 16.0 - 0.5) * depth;
        let q = |v: u8| clamp8(((v as f64 + bias) / step).round() * step);
        [q(c[0]), q(c[1]), q(c[2]), 255]
    });
    src.fade(&quant, amount)
}

fn fx_duotone(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 {
        return src;
    }
    let mut lo = [
        clamp8(20.0 + 80.0 * rng.float()),
        clamp8(20.0 + 80.0 * rng.float()),
        clamp8(20.0 + 80.0 * rng.float()),
        255,
    ];
    let mut hi = [
        clamp8(150.0 + 105.0 * rng.float()),
        clamp8(150.0 + 105.0 * rng.float()),
        clamp8(150.0 + 105.0 * rng.float()),
        255,
    ];
    if rng.int_n(2) == 0 {
        std::mem::swap(&mut lo, &mut hi);
    }
    let tinted = src.map(|x, y| {
        let t = (luma(src.at(x, y)) / 255.0).clamp(0.0, 1.0);
        [
            clamp8(lerp(lo[0] as f64, hi[0] as f64, t)),
            clamp8(lerp(lo[1] as f64, hi[1] as f64, t)),
            clamp8(lerp(lo[2] as f64, hi[2] as f64, t)),
            255,
        ]
    });
    src.fade(&tinted, amount)
}

fn fx_solarize(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 {
        return src;
    }
    let threshold = 60.0 + 130.0 * rng.float();
    let burn = src.map(|x, y| {
        let c = src.at(x, y);
        let f = |v: u8| {
            if v as f64 > threshold {
                255 - v
            } else {
                v
            }
        };
        [f(c[0]), f(c[1]), f(c[2]), 255]
    });
    src.fade(&burn, amount)
}

fn fx_grade(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 {
        return src;
    }
    let gain = [
        0.75 + 0.5 * rng.float(),
        0.75 + 0.5 * rng.float(),
        0.75 + 0.5 * rng.float(),
    ];
    let lift = [
        (rng.float() - 0.5) * 70.0,
        (rng.float() - 0.5) * 70.0,
        (rng.float() - 0.5) * 70.0,
    ];
    src.map(|x, y| {
        let c = src.at(x, y);
        let mut out = [255u8; 4];
        for channel in 0..3 {
            out[channel] = clamp8(
                (c[channel] as f64 - 128.0) * (1.0 + (gain[channel] - 1.0) * amount)
                    + 128.0
                    + lift[channel] * amount,
            );
        }
        out
    })
}

fn fx_invert(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 {
        return src;
    }
    let amount = if rng.int_n(2) == 0 {
        amount * 0.5
    } else {
        amount
    };
    let flipped = src.map(|x, y| {
        let c = src.at(x, y);
        [255 - c[0], 255 - c[1], 255 - c[2], 255]
    });
    src.fade(&flipped, amount)
}

fn fx_scanlines(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.h < 4 {
        return src;
    }
    let period = 2 + rng.int_n(3);
    let depth = 0.3 + 0.45 * rng.float();
    let wobble = 0.03 + 0.06 * rng.float();
    let phase = rng.float() * 2.0 * std::f64::consts::PI;
    let freq = 0.25 + 0.85 * rng.float();
    src.map(|x, y| {
        let c = src.at(x, y);
        let mut k = 1.0 + wobble * amount * (y as f64 * freq + phase).sin();
        if y % period == 0 {
            k *= 1.0 - depth * amount;
        }
        [
            clamp8(c[0] as f64 * k),
            clamp8(c[1] as f64 * k),
            clamp8(c[2] as f64 * k),
            255,
        ]
    })
}

fn fx_slice(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.w < 8 || src.h < 8 {
        return src;
    }
    let w = src.w as i64;
    let h = src.h as i64;
    let mut shift = vec![0f64; src.h];
    let bands = 4 + rng.int_n(7);
    for _ in 0..bands {
        let rows = 2 + rng.int_n((h / 10).max(2) as usize) as i64;
        let y0 = rng.int_n((h - rows).max(1) as usize) as i64;
        let dx = (rng.float() * 2.0 - 1.0) * w as f64 * 0.32 * amount;
        for y in y0..(y0 + rows).min(h) {
            shift[y as usize] = dx;
        }
    }
    src.map(|x, y| {
        let c = src.at(x, y);
        if shift[y] == 0.0 {
            return c;
        }
        let sx = (x as f64 - shift[y] + w as f64) % w as f64;
        src.sample(sx, y as f64)
    })
}

fn fx_pixel_sort(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.w < 8 || src.h < 4 {
        return src;
    }
    let w = src.w;
    let h = src.h;
    let threshold = quantile(&src, 0.25 + 0.30 * rng.float());
    let mut sorted = src.clone();
    let mut px = vec![[0u8; 4]; w];
    let mut keys = vec![0f64; w];
    for y in 0..h {
        let mut run = 0usize;
        for x in 0..=w {
            let mut lit = false;
            if x < w {
                let c = src.at(x, y);
                let value = luma(c);
                if value > threshold {
                    px[run] = c;
                    keys[run] = value;
                    run += 1;
                    lit = true;
                }
            }
            if lit || run == 0 {
                continue;
            }
            if run >= 2 {
                for i in 1..run {
                    let mut j = i;
                    while j > 0 && keys[j] < keys[j - 1] {
                        px.swap(j, j - 1);
                        keys.swap(j, j - 1);
                        j -= 1;
                    }
                }
                let start = x - run;
                for (offset, target) in (start..x).enumerate() {
                    sorted.set(target, y, px[offset]);
                }
            }
            run = 0;
        }
    }
    src.fade(&sorted, amount)
}

fn fx_bloom(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.w < 32 || src.h < 32 {
        return src;
    }
    let w = src.w;
    let h = src.h;
    let sw = (w / 10).max(16);
    let sh = (h / 10).max(12);
    let mut glow = vec![[0u8; 4]; sw * sh];
    let threshold = 110.0 + 70.0 * rng.float();
    for y in 0..sh {
        let y0 = y * h / sh;
        let y1 = ((y * h / sh + 1).max((y + 1) * h / sh)).min(h);
        for x in 0..sw {
            let x0 = x * w / sw;
            let x1 = ((x * w / sw + 1).max((x + 1) * w / sw)).min(w);
            let c = block_avg(&src, x0, y0, x1 - x0, y1 - y0);
            if luma(c) < threshold {
                glow[y * sw + x] = [0, 0, 0, 255];
                continue;
            }
            glow[y * sw + x] = [c[0], c[1], c[2], 255];
        }
    }
    box3(&mut glow, sw, sh);
    box3(&mut glow, sw, sh);
    let gain = (0.9 + 0.9 * rng.float()) * amount;
    src.map(|x, y| {
        let c = src.at(x, y);
        let g = sample_grid(
            &glow,
            sw,
            sh,
            (x as f64 + 0.5) / w as f64 * sw as f64 - 0.5,
            (y as f64 + 0.5) / h as f64 * sh as f64 - 0.5,
        );
        [
            clamp8(c[0] as f64 + g[0] as f64 * gain),
            clamp8(c[1] as f64 + g[1] as f64 * gain),
            clamp8(c[2] as f64 + g[2] as f64 * gain),
            255,
        ]
    })
}

fn fx_scratches(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.w < 16 {
        return src;
    }
    let w = src.w as i64;
    let h = src.h as i64;
    let count = 7 + rng.int_n(7);
    let mut marks: Vec<[i64; 4]> = Vec::with_capacity(count);
    let mut lights = Vec::with_capacity(count);
    for _ in 0..count {
        let span = h / 2 + rng.int_n((h / 2).max(1) as usize) as i64;
        marks.push([
            rng.int_n(w as usize) as i64,
            rng.int_n((h - span).max(1) as usize) as i64,
            span,
            rng.int_n(3) as i64 - 1,
        ]);
        lights.push(rng.float() * 2.0 - 1.0);
    }
    let width = 2 + rng.int_n(3) as i64;
    let mut rows: Vec<Vec<(i64, i64, f64)>> = vec![Vec::new(); src.h];
    for (index, mark) in marks.iter().enumerate() {
        let d = lights[index] * 110.0 * amount;
        let span = mark[2].max(1);
        for y in mark[1]..(mark[1] + mark[2]).min(h) {
            let x0 = mark[0] + mark[3] * (y - mark[1]) / span;
            rows[y as usize].push((x0, x0 + width, d));
        }
    }
    src.map(|x, y| {
        let c = src.at(x, y);
        for span in &rows[y] {
            if x as i64 >= span.0 && (x as i64) < span.1 {
                return [
                    clamp8(c[0] as f64 + span.2),
                    clamp8(c[1] as f64 + span.2),
                    clamp8(c[2] as f64 + span.2),
                    255,
                ];
            }
        }
        c
    })
}

fn fx_snow(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 {
        return src;
    }
    let a1 = 0.6 + rng.float();
    let b1 = 0.6 + rng.float();
    let a2 = 0.6 + rng.float();
    let b2 = 0.6 + rng.float();
    let p1 = rng.float() * FX_PHASE_SPAN;
    let p2 = rng.float() * FX_PHASE_SPAN;
    let amp = 55.0 * amount;
    src.map(|x, y| {
        let c = src.at(x, y);
        let fx = x as f64;
        let fy = y as f64;
        let n = (fx * a1 + fy * b1 + p1).sin() * (fx * a2 - fy * b2 + p2).sin();
        let d = n * amp;
        [
            clamp8(c[0] as f64 + d),
            clamp8(c[1] as f64 + d),
            clamp8(c[2] as f64 + d),
            255,
        ]
    })
}

fn fx_leak(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    use std::f64::consts::PI;
    if amount <= 0.0 || src.w < 16 {
        return src;
    }
    let angle = PI * rng.float();
    let ux = angle.cos();
    let uy = angle.sin();
    let w = src.w as f64;
    let h = src.h as f64;
    let corners = [(0.0, 0.0), (w, 0.0), (0.0, h), (w, h)];
    let mut lo = 0f64;
    let mut hi = 0f64;
    for (index, corner) in corners.iter().enumerate() {
        let u = corner.0 * ux + corner.1 * uy;
        if index == 0 || u < lo {
            lo = u;
        }
        if index == 0 || u > hi {
            hi = u;
        }
    }
    let norm = 1.0 / (hi - lo).max(1.0);
    let lo = -lo * norm;
    let count = 2 + rng.int_n(3);
    let mut bands: Vec<[f64; 5]> = Vec::with_capacity(count);
    for _ in 0..count {
        bands.push([
            0.15 + 0.7 * rng.float(),
            0.08 + 0.18 * rng.float(),
            clamp8(120.0 + 135.0 * rng.float()) as f64,
            clamp8(120.0 + 135.0 * rng.float()) as f64,
            clamp8(120.0 + 135.0 * rng.float()) as f64,
        ]);
    }
    src.map(|x, y| {
        let c = src.at(x, y);
        let u = (x as f64 * ux + y as f64 * uy) * norm + lo;
        let mut r = c[0] as f64;
        let mut g = c[1] as f64;
        let mut b = c[2] as f64;
        for band in &bands {
            let t = (u - band[0]).abs() / band[1];
            if t >= 1.0 {
                continue;
            }
            let k = (1.0 - t) * amount * 0.85;
            r += band[2] * k;
            g += band[3] * k;
            b += band[4] * k;
        }
        [clamp8(r), clamp8(g), clamp8(b), 255]
    })
}

fn fx_blinds(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.h < 16 {
        return src;
    }
    let w = src.w as i64;
    let period = (8 + rng.int_n(14)) as i64;
    let phase = rng.int_n(period as usize) as i64;
    let shift = (rng.float() * 2.0 - 1.0) * w as f64 * 0.06;
    let depth = 0.45 + 0.35 * rng.float();
    let dx = (shift * amount) as i64;
    let striped = src.map(|x, y| {
        if (y as i64 + phase) % period >= period / 2 {
            return src.at(x, y);
        }
        let k = 1.0 - depth * amount;
        let source = ((x as i64 - dx) % w + w) % w;
        let c = src.at(source as usize, y);
        [
            clamp8(c[0] as f64 * k),
            clamp8(c[1] as f64 * k),
            clamp8(c[2] as f64 * k),
            255,
        ]
    });
    src.fade(&striped, amount)
}

fn fx_melt(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    use std::f64::consts::PI;
    if amount <= 0.0 || src.w < 16 {
        return src;
    }
    let w = src.w as f64;
    let h = src.h as f64;
    let freq = 1.5 + 2.5 * rng.float();
    let phase = rng.float() * 2.0 * PI;
    let amp = h * (0.08 + 0.12 * rng.float()) * amount;
    src.remap(|x, y| {
        let drip = amp * (0.5 + 0.5 * (x / w * freq * 2.0 * PI + phase).sin());
        (x, y - drip * (y / h * 2.0).clamp(0.0, 1.0))
    })
}

fn fx_grid(src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.w < 16 {
        return src;
    }
    let step = (24 + rng.int_n(40)) as i64;
    let thick = (1 + rng.int_n(2)) as i64;
    let ox = rng.int_n(step as usize) as i64;
    let oy = rng.int_n(step as usize) as i64;
    let dark = 0.45 + 0.35 * rng.float();
    src.map(|x, y| {
        let c = src.at(x, y);
        if (x as i64 + ox) % step >= thick && (y as i64 + oy) % step >= thick {
            return c;
        }
        let k = 1.0 - dark * amount;
        [
            clamp8(c[0] as f64 * k),
            clamp8(c[1] as f64 * k),
            clamp8(c[2] as f64 * k),
            255,
        ]
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Step {
    Glitch,
    Ripple,
    WashShuffle,
    Kaleido,
    Outline,
    HueVariant,
    Chart,
    Twist,
    Mosaic,
    Halftone,
    Mirror,
    Displace,
    Shatter,
    Fisheye,
    Liquid,
    Blinds,
    Melt,
    Chroma,
    Hue,
    Posterize,
    Dither,
    Duotone,
    Solarize,
    Grade,
    Invert,
    Scanlines,
    Slice,
    PixelSort,
    Bloom,
    Scratches,
    Snow,
    Leak,
    Grid,
}

fn run_step(step: Step, src: Plate, amount: f64, rng: &mut Rand) -> Plate {
    match step {
        Step::Glitch => variant_glitch(src, amount, rng),
        Step::Ripple => variant_ripple(src, amount, rng),
        Step::WashShuffle => variant_wash_shuffle(src, amount, rng),
        Step::Kaleido => variant_kaleido(src, amount, rng),
        Step::Outline => variant_outline(src, amount, rng),
        Step::HueVariant => variant_hue(src, amount),
        Step::Chart => variant_chart(src, amount, rng),
        Step::Twist => fx_twist(src, amount, rng),
        Step::Mosaic => fx_mosaic(src, amount, rng),
        Step::Halftone => fx_halftone(src, amount, rng),
        Step::Mirror => fx_mirror(src, amount, rng),
        Step::Displace => fx_displace(src, amount, rng),
        Step::Shatter => fx_shatter(src, amount, rng),
        Step::Fisheye => fx_fisheye(src, amount, rng),
        Step::Liquid => fx_liquid(src, amount, rng),
        Step::Blinds => fx_blinds(src, amount, rng),
        Step::Melt => fx_melt(src, amount, rng),
        Step::Chroma => fx_chroma(src, amount, rng),
        Step::Hue => fx_hue(src, amount, rng),
        Step::Posterize => fx_posterize(src, amount, rng),
        Step::Dither => fx_dither(src, amount, rng),
        Step::Duotone => fx_duotone(src, amount, rng),
        Step::Solarize => fx_solarize(src, amount, rng),
        Step::Grade => fx_grade(src, amount, rng),
        Step::Invert => fx_invert(src, amount, rng),
        Step::Scanlines => fx_scanlines(src, amount, rng),
        Step::Slice => fx_slice(src, amount, rng),
        Step::PixelSort => fx_pixel_sort(src, amount, rng),
        Step::Bloom => fx_bloom(src, amount, rng),
        Step::Scratches => fx_scratches(src, amount, rng),
        Step::Snow => fx_snow(src, amount, rng),
        Step::Leak => fx_leak(src, amount, rng),
        Step::Grid => fx_grid(src, amount, rng),
    }
}

const VARIANT_GLITCH: &str = "glitch";
const VARIANT_WARP: &str = "warp";
const VARIANT_COLOR: &str = "color";
const VARIANT_POLAR: &str = "polar";
const VARIANT_OUTLINE: &str = "outline";
const VARIANT_HUE: &str = "hue";

const TAG_WARP: &str = "warp";
const TAG_QUANTIZE: &str = "quantize";
const TAG_LINES: &str = "lines";
const TAG_DUPLICATE: &str = "duplicate";
const TAG_DISPLACE: &str = "displace";
const TAG_COLOR_SHIFT: &str = "color-shift";
const TAG_COLOR_MAP: &str = "color-map";
const TAG_INVERT: &str = "invert";
const TAG_GLOW: &str = "glow";
const TAG_SPECKLE: &str = "speckle";
const TAG_GRAIN: &str = "grain";
const TAG_BANDS: &str = "bands";
const TAG_DOTS: &str = "dots";

fn variant_step(variant: &str) -> Step {
    match variant {
        VARIANT_GLITCH => Step::Glitch,
        VARIANT_WARP => Step::Ripple,
        VARIANT_COLOR => Step::WashShuffle,
        VARIANT_POLAR => Step::Kaleido,
        VARIANT_OUTLINE => Step::Outline,
        VARIANT_HUE => Step::HueVariant,
        _ => Step::Chart,
    }
}

fn variant_weight(variant: &str) -> f64 {
    match variant {
        VARIANT_GLITCH => 1.20,
        VARIANT_WARP => 0.95,
        VARIANT_COLOR => 0.88,
        VARIANT_POLAR => 0.80,
        VARIANT_OUTLINE => 0.31,
        VARIANT_HUE => 0.36,
        _ => 0.30,
    }
}

fn variant_tags(variant: &str) -> Vec<&'static str> {
    match variant {
        VARIANT_GLITCH => vec![TAG_DISPLACE],
        VARIANT_WARP => vec![TAG_WARP],
        VARIANT_COLOR => vec![TAG_QUANTIZE, TAG_DUPLICATE],
        VARIANT_POLAR => vec![TAG_DUPLICATE],
        VARIANT_OUTLINE => vec![TAG_LINES],
        VARIANT_HUE => vec![TAG_COLOR_SHIFT],
        _ => vec![TAG_LINES],
    }
}

struct FxDef {
    step: Step,
    tags: &'static [&'static str],
    gain: f64,
}

const FX_GEOM: [FxDef; 10] = [
    FxDef {
        step: Step::Twist,
        tags: &[TAG_WARP],
        gain: 2.00,
    },
    FxDef {
        step: Step::Mosaic,
        tags: &[TAG_QUANTIZE],
        gain: 2.60,
    },
    FxDef {
        step: Step::Halftone,
        tags: &[TAG_QUANTIZE, TAG_DOTS],
        gain: 0.35,
    },
    FxDef {
        step: Step::Mirror,
        tags: &[TAG_DUPLICATE],
        gain: 0.40,
    },
    FxDef {
        step: Step::Displace,
        tags: &[TAG_WARP],
        gain: 2.40,
    },
    FxDef {
        step: Step::Shatter,
        tags: &[TAG_DUPLICATE],
        gain: 1.30,
    },
    FxDef {
        step: Step::Fisheye,
        tags: &[TAG_WARP],
        gain: 1.30,
    },
    FxDef {
        step: Step::Liquid,
        tags: &[TAG_WARP],
        gain: 2.60,
    },
    FxDef {
        step: Step::Blinds,
        tags: &[TAG_BANDS],
        gain: 1.00,
    },
    FxDef {
        step: Step::Melt,
        tags: &[TAG_WARP],
        gain: 2.30,
    },
];

const FX_TONE: [FxDef; 8] = [
    FxDef {
        step: Step::Chroma,
        tags: &["color-split"],
        gain: 2.90,
    },
    FxDef {
        step: Step::Hue,
        tags: &[TAG_COLOR_SHIFT],
        gain: 0.35,
    },
    FxDef {
        step: Step::Posterize,
        tags: &[TAG_QUANTIZE],
        gain: 1.40,
    },
    FxDef {
        step: Step::Dither,
        tags: &[TAG_QUANTIZE],
        gain: 0.60,
    },
    FxDef {
        step: Step::Duotone,
        tags: &[TAG_COLOR_MAP],
        gain: 0.40,
    },
    FxDef {
        step: Step::Solarize,
        tags: &[TAG_INVERT],
        gain: 0.35,
    },
    FxDef {
        step: Step::Grade,
        tags: &[TAG_COLOR_SHIFT],
        gain: 1.10,
    },
    FxDef {
        step: Step::Invert,
        tags: &[TAG_INVERT],
        gain: 0.35,
    },
];

const FX_TEXTURE: [FxDef; 8] = [
    FxDef {
        step: Step::Scanlines,
        tags: &[TAG_LINES],
        gain: 2.00,
    },
    FxDef {
        step: Step::Slice,
        tags: &[TAG_DISPLACE],
        gain: 3.00,
    },
    FxDef {
        step: Step::PixelSort,
        tags: &[TAG_DISPLACE],
        gain: 3.50,
    },
    FxDef {
        step: Step::Bloom,
        tags: &[TAG_GLOW],
        gain: 0.70,
    },
    FxDef {
        step: Step::Scratches,
        tags: &[TAG_SPECKLE],
        gain: 3.50,
    },
    FxDef {
        step: Step::Snow,
        tags: &[TAG_GRAIN],
        gain: 1.20,
    },
    FxDef {
        step: Step::Leak,
        tags: &[TAG_GLOW],
        gain: 0.60,
    },
    FxDef {
        step: Step::Grid,
        tags: &[TAG_LINES],
        gain: 3.70,
    },
];

const FX_WEIGHTS: [f64; 3] = [0.41, 0.35, 0.29];

const FX_PLAN: [(usize, f64); 5] = [(0, 1.00), (2, 1.00), (1, 1.00), (0, 0.62), (2, 0.55)];

const SURFACE_TAGS: [&str; 5] = [TAG_LINES, TAG_BANDS, TAG_DOTS, TAG_GRAIN, TAG_SPECKLE];

fn fx_family(family: usize) -> &'static [FxDef] {
    match family {
        0 => &FX_GEOM,
        1 => &FX_TONE,
        _ => &FX_TEXTURE,
    }
}

fn has_surface(tags: &[&'static str]) -> bool {
    tags.iter()
        .any(|tag| SURFACE_TAGS.iter().any(|surface| surface == tag))
}

fn fx_clash(candidate: &FxDef, used: &[&'static str]) -> bool {
    candidate
        .tags
        .iter()
        .any(|tag| used.iter().any(|seen| seen == tag))
}

fn choose_fx(
    family: &'static [FxDef],
    order: &[usize],
    used: &[&'static str],
    surface: bool,
) -> Option<&'static FxDef> {
    for index in order {
        let candidate = &family[*index];
        if fx_clash(candidate, used) || (surface && !has_surface(candidate.tags)) {
            continue;
        }
        return Some(candidate);
    }
    None
}

fn fx_picks(seed: i64, used: &[&'static str]) -> Vec<(Step, f64)> {
    let mut rng = Rand::new(seed);
    let mut used: Vec<&'static str> = used.to_vec();
    let mut out: Vec<(Step, f64)> = Vec::with_capacity(FX_PLAN.len());
    for (family_index, scale) in FX_PLAN {
        let family = fx_family(family_index);
        let order = rng.perm(family.len());
        let want_surface = family_index == 2 && !has_surface(&used);
        let mut pick = choose_fx(family, &order, &used, want_surface);
        if pick.is_none() {
            pick = choose_fx(family, &order, &used, false);
        }
        let pick = match pick {
            Some(found) => found,
            None => &family[order[0]],
        };
        out.push((pick.step, FX_WEIGHTS[family_index] * scale * pick.gain));
        used.extend(pick.tags.iter().copied());
    }
    out
}

fn damage_of(interference: f64, weight: f64) -> f64 {
    if interference <= FX_DEADZONE {
        return 0.0;
    }
    (weight * (interference - FX_DEADZONE) / (1.0 - FX_DEADZONE)).min(1.0)
}

fn guess_plate(art: Plate, input: &GuessInput) -> Plate {
    let seed = input.seed_value();
    let variant = input.variant_name();
    let interference = input.interference_value();
    let level = input.level_value();
    let (cx, cy) = fragment_center(seed);
    let mut rng = Rand::new(seed);
    let mut out = art;
    let reveal = input.zoom_value().clamp(0.02, 1.0);
    if reveal < 1.0 {
        out = zoom_step(&out, reveal, cx, cy);
    }
    let blur = 1.0 - level;
    out = resample_step(out, GUESS_RESAMPLE as f64, blur);
    out = blur_step(out, GUESS_BLUR as f64, blur);
    let mut layers: Vec<(Step, f64)> = Vec::with_capacity(6);
    layers.push((variant_step(variant), variant_weight(variant)));
    layers.extend(fx_picks(seed, &variant_tags(variant)));
    for (step, weight) in layers {
        out = run_step(step, out, damage_of(interference, weight), &mut rng);
    }
    grain_step(out, 1, blur, &mut rng)
}

pub fn request(input: &GuessInput) -> RenderRequest {
    let (width, height) = input.stage.size();
    super::harness::request(width, height).with_scale(1.5)
}

pub fn card(input: GuessInput) -> RenderCard {
    let request = request(&input);
    let cache_key = super::memo::fingerprint("guess", &input);
    RenderCard::new(request, "guess.png", move |kit| paint(kit, &input)).with_cache_key(cache_key)
}

pub fn paint(kit: &CanvasKit<'_>, input: &GuessInput) {
    let canvas = kit.canvas();
    canvas.save();
    canvas.scale((GUESS_SHRINK as f32, GUESS_SHRINK as f32));
    paint_ground(kit, canvas_w() as f32, canvas_h() as f32);
    paint_puzzle(kit, input);
    paint_head(kit, input, canvas_w() as f32);
    canvas.restore();
}

fn paint_ground(kit: &CanvasKit<'_>, width: f32, height: f32) {
    let full = Rect::from_xywh(0.0, 0.0, width, height);
    kit.fill_rect(full, GUESS_GROUND_TOP);
    let _ = kit.fill_gradient(
        full,
        0.0,
        &aurorix_render::Gradient::Linear(aurorix_render::LinearGradient::vertical([
            aurorix_render::GradientStop::new(0.0, GUESS_GROUND_TOP),
            aurorix_render::GradientStop::new(1.0, GUESS_GROUND_BOTTOM),
        ])),
    );
}

fn fill_rect_in(kit: &CanvasKit<'_>, area: Rect, color: Color) {
    if area.width() <= 0.0 || area.height() <= 0.0 {
        return;
    }
    kit.fill_rect(area, color);
}

fn paint_puzzle(kit: &CanvasKit<'_>, input: &GuessInput) {
    let frame = image_rect();
    let art = guess_art(kit, input, frame);
    let drawn = art.is_some();
    match art {
        None => fill_rect_in(kit, frame, GUESS_COVER_FALLBACK),
        Some(plate) => {
            let plate = if input.final_frame() {
                plate
            } else {
                guess_plate(plate, input)
            };
            draw_plate(kit, &plate, frame);
        }
    }
    let edge = sx(2) as f32;
    fill_rect_in(
        kit,
        Rect::from_ltrb(
            frame.left() - edge,
            frame.top() - edge,
            frame.right() + edge,
            frame.top(),
        ),
        COL_HAIRLINE,
    );
    fill_rect_in(
        kit,
        Rect::from_ltrb(
            frame.left() - edge,
            frame.bottom(),
            frame.right() + edge,
            frame.bottom() + edge,
        ),
        COL_HAIRLINE,
    );
    fill_rect_in(
        kit,
        Rect::from_ltrb(
            frame.left() - edge,
            frame.top() - edge,
            frame.left(),
            frame.bottom() + edge,
        ),
        COL_HAIRLINE,
    );
    fill_rect_in(
        kit,
        Rect::from_ltrb(
            frame.right(),
            frame.top() - edge,
            frame.right() + edge,
            frame.bottom() + edge,
        ),
        COL_HAIRLINE,
    );
    if drawn {
        paint_frame(kit, input, frame);
    }
}

fn paint_frame(kit: &CanvasKit<'_>, input: &GuessInput, frame: Rect) {
    if !input.final_frame() {
        return;
    }
    let reveal = input.zoom_value();
    let Some(window) = guess_window(frame, input.seed_value(), reveal) else {
        return;
    };
    fill_rect_in(
        kit,
        Rect::from_ltrb(frame.left(), frame.top(), frame.right(), window.y0),
        FRAME_DIM,
    );
    fill_rect_in(
        kit,
        Rect::from_ltrb(frame.left(), window.y1, frame.right(), frame.bottom()),
        FRAME_DIM,
    );
    fill_rect_in(
        kit,
        Rect::from_ltrb(frame.left(), window.y0, window.x0, window.y1),
        FRAME_DIM,
    );
    fill_rect_in(
        kit,
        Rect::from_ltrb(window.x1, window.y0, frame.right(), window.y1),
        FRAME_DIM,
    );
    let line = sx(8).max(6) as f32;
    fill_rect_in(
        kit,
        Rect::from_ltrb(window.x0, window.y0, window.x1, window.y0 + line),
        COL_ACCENT_PURPLE,
    );
    fill_rect_in(
        kit,
        Rect::from_ltrb(window.x0, window.y1 - line, window.x1, window.y1),
        COL_ACCENT_PURPLE,
    );
    fill_rect_in(
        kit,
        Rect::from_ltrb(window.x0, window.y0, window.x0 + line, window.y1),
        COL_ACCENT_PURPLE,
    );
    fill_rect_in(
        kit,
        Rect::from_ltrb(window.x1 - line, window.y0, window.x1, window.y1),
        COL_ACCENT_PURPLE,
    );
    let tick = sx(64).max(26) as f32;
    let thick = sx(22).max(10) as f32;
    let corners = [
        (window.x0, window.y0, 1.0f32, 1.0f32),
        (window.x1, window.y1, -1.0, -1.0),
        (window.x0, window.y1, 1.0, -1.0),
        (window.x1, window.y0, -1.0, 1.0),
    ];
    for (x, y, sx_dir, sy_dir) in corners {
        let a = Rect::from_ltrb(
            x.min(x + sx_dir * tick),
            y.min(y + sy_dir * thick),
            x.max(x + sx_dir * tick),
            y.max(y + sy_dir * thick),
        );
        let b = Rect::from_ltrb(
            x.min(x + sx_dir * thick),
            y.min(y + sy_dir * tick),
            x.max(x + sx_dir * thick),
            y.max(y + sy_dir * tick),
        );
        fill_rect_in(kit, a, COL_ACCENT_PURPLE);
        fill_rect_in(kit, b, COL_ACCENT_PURPLE);
    }
}

fn paint_head(kit: &CanvasKit<'_>, input: &GuessInput, width: f32) {
    let pad = sxr(GUESS_BASE_PAD) as f32;
    let title_style = theme::style(
        theme::FONT_DISPLAY,
        sf(GUESS_TITLE_SIZE).round() as u32,
        COL_TEXT_MAIN,
    );
    sketch::text_shadow_outlined(
        kit,
        &title_style,
        GUESS_TITLE,
        pad,
        title_base() as f32,
        COL_TEXT_MAIN,
    );
    let code = input.code_text();
    if code.is_empty() {
        return;
    }
    let code_style = theme::style(
        theme::FONT_NUMBER,
        sf(GUESS_CODE_SIZE).round() as u32,
        COL_ACCENT_PURPLE,
    );
    let code_w = sketch::text_width(kit, &code_style, &code);
    sketch::text_shadow_outlined(
        kit,
        &code_style,
        &code,
        width - pad - code_w,
        code_base() as f32,
        COL_ACCENT_PURPLE,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::harness::{self, Harness};

    const PNG_MAGIC: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

    fn sample(stage: GuessStage) -> GuessInput {
        GuessInput::new(stage)
            .with_song("acheron")
            .with_jacket("jackets/acheron.jpg")
            .with_round(3)
            .with_seed(20251120)
            .with_level(0.45)
            .with_zoom(0.55)
            .with_interference(0.6)
    }

    fn render(input: &GuessInput) -> Vec<u8> {
        let harness = Harness::new();
        let bytes = harness.render(request(input).with_scale(1.0), |kit| paint(kit, input));
        assert!(bytes.starts_with(&PNG_MAGIC), "必须输出 PNG");
        bytes
    }

    #[test]
    fn case_163() {
        assert_eq!((canvas_w(), canvas_h()), (924, 1257));
        assert_eq!((out_w(), out_h()), (739, 1006));
        assert_eq!(image_rect().left(), 78.0);
        assert_eq!(image_rect().top(), 235.0);
        assert_eq!(image_rect().width(), 767.0);
        assert_eq!(title_base(), 137);
        assert_eq!(code_base(), 126);
        assert_eq!(GuessStage::Question.size(), (739, 1006));
        assert_eq!(GuessStage::Answer.size(), (739, 1006));
    }

    #[test]
    fn case_164() {
        let frame = image_rect();
        let window = guess_window(frame, 20251120, 0.55).expect("必须有一次取景");
        assert!(window.x0 >= frame.left() && window.x1 <= frame.right());
        assert!(window.y0 >= frame.top() && window.y1 <= frame.bottom());
        assert!((window.x1 - window.x0 - frame.width() * 0.55).abs() <= 2.0);
        let same = guess_window(frame, 20251120, 0.55).expect("同种子必须同窗");
        assert_eq!(window, same);
        let other = guess_window(frame, 7, 0.55).expect("另一个种子也要有窗");
        assert_ne!(window, other, "换种子必须换窗口");
        assert!(guess_window(frame, 20251120, 1.0).is_none());
        assert!(guess_window(frame, 20251120, 0.0).is_none());
    }

    #[test]
    fn case_165() {
        let question = sample(GuessStage::Question);
        let bytes = render(&question);
        assert_eq!(
            harness::image_size(&bytes),
            Some((739, 1006)),
            "问句出图尺寸不对"
        );
        let pixels = harness::decode(&bytes).expect("解码谜面");
        let frame = image_rect();
        let shrink = GUESS_SHRINK as f32;
        let ink = pixels.ink_ratio(
            (frame.left() * shrink) as u32,
            (frame.top() * shrink) as u32,
            (frame.width() * shrink) as u32,
            (frame.height() * shrink) as u32,
            6,
        );
        assert!(ink > 0.05, "谜面必须有内容, 实际 {ink}");

        let answer = sample(GuessStage::Answer);
        let bytes = render(&answer);
        assert_eq!(
            harness::image_size(&bytes),
            Some((739, 1006)),
            "揭晓画布尺寸不对"
        );
        let pixels = harness::decode(&bytes).expect("解码揭晓");
        let window = guess_window(frame, answer.seed_value(), answer.zoom_value()).expect("取景框");
        let scale = GUESS_SHRINK as f32;
        let bright = |x: f32, y: f32| {
            let pixel = pixels.at((x * scale) as u32, (y * scale) as u32);
            pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32
        };
        let inside = bright((window.x0 + window.x1) / 2.0, (window.y0 + window.y1) / 2.0);
        let band_x = (frame.left() + window.x0) / 2.0;
        let band_y = (window.y0 + window.y1) / 2.0;
        let outside = bright(band_x, band_y);
        assert!(
            inside > outside,
            "框外那条暗幕必须比框内更暗, 实际 {inside} 对 {outside}"
        );
    }

    #[test]
    fn case_166() {
        let input = sample(GuessStage::Question).without_jacket().with_song("");
        let bytes = render(&input);
        let pixels = harness::decode(&bytes).expect("解码无曲绘谜面");
        assert!(!pixels.blank(), "占位谜面不能是纯色");
        let frame = image_rect();
        let [r, g, b, _] = pixels.at(frame.left() as u32 + 4, frame.top() as u32 + 4);
        assert_eq!(
            (r, g, b),
            (
                GUESS_COVER_FALLBACK.r,
                GUESS_COVER_FALLBACK.g,
                GUESS_COVER_FALLBACK.b
            ),
            "缺曲绘必须铺占位色"
        );
    }

    #[test]
    fn case_167() {
        let input = GuessInput::question().with_seed(0x123456);
        assert_eq!(input.code_text(), "123456");
        let named = input.clone().with_code("ROUND 03");
        assert_eq!(named.code_text(), "ROUND 03");
        assert!(render(&named).starts_with(&PNG_MAGIC));
    }
}
