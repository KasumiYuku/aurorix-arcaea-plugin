use aurorix_plugin::RenderCard;
use aurorix_render::skia_safe::{ClipOp, PathBuilder, Rect};
use aurorix_render::{CanvasKit, Color, RenderRequest, TextStyle};

use super::player;
use super::sketch;
use super::theme::{self, Theme};
use super::widgets::{self, PlayerBadge};
use crate::cards::CardKind;
use crate::domain::ptt;
use crate::model::{diff_display_weight, Difficulty, Grade};

const K: f32 = 1.0;

pub const LIST_WIDTH: i32 = 1188;

pub const TABLE_WIDTH: i32 = 1409;

pub const AHV_WIDTH: i32 = 1600;

pub const MIN_HEIGHT: i32 = 360;

pub const MAX_HEIGHT: i32 = 12000;

const LIST_PAD_L: f32 = 36.0 * 1.25 * K;
const LIST_CARD_W: f32 = 210.0 * K;
const LIST_CARD_H: f32 = 98.0 * K;
const LIST_CARD_COLS: usize = 5;
const LIST_CARD_GAP_X: f32 = 12.0 * K;
const LIST_CARD_GAP_Y: f32 = 12.0 * K;
const LIST_CARD_R: f32 = 8.0 * K;
const LIST_STEP_X: f32 = LIST_CARD_W + LIST_CARD_GAP_X;
const LIST_STEP_Y: f32 = LIST_CARD_H + LIST_CARD_GAP_Y;
const LIST_GRID_TOP: f32 = 126.0 * K;
const LIST_FOOT_GAP: f32 = 30.0 * K;
const LIST_BOTTOM_PAD: f32 = 62.0 * K;
const LIST_RIGHT: f32 = LIST_PAD_L
    + LIST_CARD_COLS as f32 * LIST_CARD_W
    + (LIST_CARD_COLS as f32 - 1.0) * LIST_CARD_GAP_X;
const LIST_TITLE_X: f32 = LIST_PAD_L;
const LIST_TITLE_BASE: f32 = 56.0 * K;
const LIST_TITLE_SIZE: f32 = 32.0 * K;
const LIST_SUB_BASE: f32 = 80.0 * K;
const LIST_SUB_SIZE: f32 = 13.0 * K;

const LIST_SPLIT_Y: f32 = 58.0 * K;
const LIST_ICE_W: f32 = 142.0 * K;
const LIST_LEAN_X: f32 = 132.0 * K;
const LIST_PAD_X: f32 = 10.0 * K;
const LIST_GRADE_CX: f32 = 176.0 * K;
const LIST_GRADE_CY: f32 = LIST_SPLIT_Y / 2.0;
const LIST_GRADE_SZ: f32 = 24.0 * K;
const LIST_THUMB_X: f32 = 10.0 * K;
const LIST_THUMB_Y: f32 = 8.0 * K;
const LIST_THUMB_SIZE: f32 = 42.0 * K;
const LIST_THUMB_R: f32 = 6.0 * K;
const LIST_ENTRY_TITLE_X: f32 = 58.0 * K;
const LIST_ENTRY_TITLE_BASE: f32 = 21.0 * K;
const LIST_ENTRY_TITLE_SIZE: f32 = 13.0 * K;
const LIST_ENTRY_TITLE_W: f32 = 88.0 * K;
const LIST_ARTIST_BASE: f32 = 35.0 * K;
const LIST_ARTIST_SIZE: f32 = 10.0 * K;
const LIST_ARTIST_W: f32 = 88.0 * K;
const LIST_META_BASE: f32 = 49.0 * K;
const LIST_META_SIZE: f32 = 11.0 * K;
const LIST_TARGET_BASE: f32 = 75.0 * K;
const LIST_TARGET_SIZE: f32 = 14.0 * K;
const LIST_FROM_BASE: f32 = 90.0 * K;
const LIST_FROM_SIZE: f32 = 10.0 * K;
const LIST_LOSS_CX: f32 = 176.0 * K;
const LIST_LOSS_BASE: f32 = 83.0 * K;
const LIST_LOSS_SIZE: f32 = 12.0 * K;
const LIST_BAND_H: f32 = 30.0 * K;
const LIST_BAND_GAP: f32 = 12.0 * K;
const LIST_BAND_TERM: f32 = 20.0 * K;
const LIST_BAND_CHAM: f32 = 9.0 * K;
const LIST_BAND_SIZE: f32 = 14.0 * K;
const LIST_BAND_SUB_SIZE: f32 = 12.0 * K;
const LIST_BAND_MARK_W: f32 = 7.0 * K;
const LIST_BAND_TEXT_X: f32 = 18.0 * K;
const LIST_BAND_TEXT_GAP: f32 = 12.0 * K;

const CG_PAD_X: f32 = 50.0 * K;
const CG_LEFT: f32 = CG_PAD_X;
const CG_RAIL_W: f32 = 154.0 * K;
const CG_RAIL_GAP: f32 = 25.0 * K;
const CG_COLS: usize = 5;
const CG_CARD_W: f32 = 206.0 * K;
const CG_COVER_H: f32 = 206.0 * K;
const CG_PLATE_H: f32 = 50.0 * K;
const CG_CARD_H: f32 = 256.0 * K;
const CG_GAP_X: f32 = 25.0 * K;
const CG_GAP_Y: f32 = 22.0 * K;
const CG_PITCH_X: f32 = CG_CARD_W + CG_GAP_X;
const CG_PITCH_Y: f32 = CG_CARD_H + CG_GAP_Y;
const CG_GROUP_GAP: f32 = 28.0 * K;
const CG_CONTENT_Y: f32 = 218.0 * K;
const CG_COL_X0: f32 = CG_LEFT + CG_RAIL_W + CG_RAIL_GAP;
const CG_RIGHT: f32 = CG_COL_X0 + CG_COLS as f32 * CG_CARD_W + (CG_COLS as f32 - 1.0) * CG_GAP_X;
const CG_TILE_COLS: usize = CG_COLS;
const CG_TILE_CARD_W: f32 =
    ((CG_RIGHT - CG_LEFT - (CG_COLS as f32 - 1.0) * CG_GAP_X) / CG_COLS as f32) as i32 as f32;
const CG_STRIP_H: f32 = 34.0 * K;
const CG_TILE_H: f32 = CG_TILE_CARD_W + CG_PLATE_H + CG_STRIP_H;
const CG_TILE_PITCH_X: f32 = CG_TILE_CARD_W + CG_GAP_X;
const CG_TILE_PITCH_Y: f32 = CG_TILE_H + CG_GAP_Y;
const CG_STRIP_DIFF_SIZE: f32 = 10.0 * K;
const CG_STRIP_CONST_SIZE: f32 = 14.0 * K;
const CG_STRIP_TICK: f32 = 3.0 * K;
const CG_STRIP_DIFF_BASE: f32 = 13.0 * K;
const CG_STRIP_CONST_BASE: f32 = 29.0 * K;
const CG_STRIP_FONT: &str = "NotoSansMono-Bold";
const CG_TALL: f32 = 134.0 * K;
const CG_HEAD_T: f32 = 50.0 * K;
const CG_HEAD_X0: f32 = CG_LEFT;
const CG_HEAD_CHAMF: f32 = 39.0 * K;
const CG_HEAD_SHADOW: f32 = 6.0 * K;
const CG_TITLE_BASE: f32 = 100.0 * K;
const CG_TITLE_SIZE: f32 = 45.0 * K;
const CG_LOGO_H: f32 = 78.0 * K;
const CG_LOGO_RIGHT: f32 = CG_RIGHT - 46.0 * K;
const CG_LOGO_TOP: f32 = 76.0 * K;
const CG_LOGO_MARK_X: f32 = 267.0;
const CG_LOGO_MARK_Y: f32 = 25.0;
const CG_LOGO_MARK_W: f32 = 995.0;
const CG_LOGO_MARK_H: f32 = 227.0;
const SLOPE_K: f32 = 0.26;
const CG_MARK_W: f32 = 22.0 * K;
const CG_MARK_THIN: f32 = 6.0 * K;
const CG_MARK_SLANT: f32 = CG_TALL * SLOPE_K;
const CG_TITLE_X: f32 = CG_HEAD_X0 + CG_MARK_SLANT + CG_MARK_W + CG_MARK_THIN
    - (CG_TITLE_BASE - CG_HEAD_T - CG_TITLE_SIZE * 0.72) * SLOPE_K
    + 25.0 * K;
const CG_CHIP_TOP: f32 = 123.0 * K;
const CG_CHIP_H: f32 = 34.0 * K;
const CG_CHIP_GAP: f32 = 6.0 * K;
const CG_CHIP_PAD_X: f32 = 17.0 * K;
const CG_CHIP_SIZE: f32 = 15.0 * K;
const CG_CHIP_LEAN: f32 = SLOPE_K;
const CG_CHIP_X0: f32 =
    CG_TITLE_X - (CG_CHIP_TOP + CG_CHIP_H - (CG_TITLE_BASE - CG_TITLE_SIZE * 0.72)) * SLOPE_K;
const CG_CHAM_TR: f32 = 25.0 * K;
const CG_CHAM_BL: f32 = 20.0 * K;
const CG_SHADOW_X: f32 = 6.0 * K;
const CG_SHADOW_Y: f32 = 7.0 * K;
const CG_TITLE_PAD: f32 = 17.0 * K;
const CG_CARD_TITLE_SIZE: f32 = 18.0 * K;
const CG_TITLE_FONT: &str = "NotoSans-Bold";
const CG_CARD_TITLE_W: f32 = CG_CARD_W - 2.0 * CG_TITLE_PAD;
const CG_TRI_SZ: f32 = 8.0 * K;
const CG_TAG_H: f32 = 31.0 * K;
const CG_TAG_SIZE: f32 = 15.0 * K;
const CG_TAG_PAD_X: f32 = 13.0 * K;
const CG_TAG_SLANT: f32 = 14.0 * K;
const CG_RAIL_CHAM_TL: f32 = 22.0 * K;
const CG_RAIL_SEAM: f32 = 8.0 * K;
const CG_RAIL_TOP_L: f32 = 182.0 * K;
const CG_RAIL_TOP_R: f32 = 133.0 * K;
const CG_RAIL_LOW_CHAM: f32 = 34.0 * K;
const CG_RAIL_WATER_SZ: f32 = 161.0 * K;
const CG_RAIL_WATER_FT: f32 = 70.0 * K;
const CG_RAIL_SINGLE_L: f32 = 0.58;
const CG_RAIL_SINGLE_R: f32 = 0.42;
const CG_RAIL_LOW_CHAM1: f32 = 25.0 * K;
const CG_RAIL_WATER_SZ1: f32 = 101.0 * K;
const CG_RAIL_WATER_FT1: f32 = 20.0 * K;
const CG_LEVEL_SIZE: f32 = 62.0 * K;
const CG_LEVEL_INK_RISE: f32 = 0.36;
const CG_RAIL_MERGE_SIZE: f32 = 32.0 * K;
const CG_RAIL_MERGE_TOP: f32 = 58.0 * K;
const CG_RAIL_MERGE_BOT: f32 = 112.0 * K;
const CG_RAIL_MERGE_SEAM: f32 = 74.0 * K;
const CG_LOCK_STEP: i32 = 15;
const CG_LOCK_LINE: i32 = 3;
const CG_LOCK_SHADE: f32 = 11.0 * K;
const CG_LOCK_HI: f32 = 2.0 * K;
const CG_FOOT_GAP: f32 = 28.0 * K;
const CG_FOOT_H: f32 = 22.0 * K;
const CG_FOOT_PAD: f32 = 25.0 * K;
const CG_FOOT_SIZE: f32 = 15.0 * K;

const AHV_K: f32 = 1600.0 / 1532.0;
const AHV_LEFT: f32 = 50.0 * K * AHV_K;
const AHV_RAIL_W: f32 = 154.0 * K * AHV_K;
const AHV_RAIL_GAP: f32 = 25.0 * K * AHV_K;
const AHV_CARD_W: f32 = 188.0 * K * AHV_K;
const AHV_GAP_X: f32 = 25.0 * K * AHV_K;
const AHV_GAP_Y: f32 = 22.0 * K * AHV_K;
const AHV_COLS: usize = 6;
const AHV_COVER_H: f32 = AHV_CARD_W;
const AHV_CARD_H: f32 = 242.0 * K * AHV_K;
const AHV_PITCH_X: f32 = AHV_CARD_W + AHV_GAP_X;
const AHV_PITCH_Y: f32 = AHV_CARD_H + AHV_GAP_Y;
const AHV_COL_X0: f32 = AHV_LEFT + AHV_RAIL_W + AHV_RAIL_GAP;
const AHV_RIGHT: f32 =
    AHV_COL_X0 + AHV_COLS as f32 * AHV_CARD_W + (AHV_COLS as f32 - 1.0) * AHV_GAP_X;
const AHV_CHAM_TR: f32 = 16.0 * K * AHV_K;
const AHV_CHAM_BL: f32 = 14.0 * K * AHV_K;
const AHV_SHADOW_DX: f32 = CG_SHADOW_X * AHV_K;
const AHV_SHADOW_DY: f32 = CG_SHADOW_Y * AHV_K;
const AHV_INFO_PAD: f32 = 8.0 * K * AHV_K;
const AHV_ROW1_H: f32 = 28.0 * K * AHV_K;
const AHV_ROW2_TOP: f32 = AHV_COVER_H + AHV_ROW1_H;
const AHV_ROW2_H: f32 = 26.0 * K * AHV_K;
const AHV_TITLE_SIZE: f32 = 15.0 * K * AHV_K;
const AHV_TITLE_BASE: f32 = 207.0 * K * AHV_K;
const AHV_TITLE_MAX: f32 = 118.0 * K * AHV_K;
const AHV_PILL_W: f32 = 50.0 * K * AHV_K;
const AHV_PILL_H: f32 = 19.0 * K * AHV_K;
const AHV_PILL_RIGHT: f32 = 6.0 * K * AHV_K;
const AHV_PILL_R: f32 = 2.0 * K * AHV_K;
const AHV_PILL_SIZE: f32 = 13.0 * K * AHV_K;
const AHV_SCORE_SIZE: f32 = 15.0 * K * AHV_K;
const AHV_SCORE_BASE: f32 = 234.0 * K * AHV_K;
const AHV_JUDGE_SIZE: f32 = 11.0 * K * AHV_K;
const AHV_JUDGE_BASE: f32 = 233.0 * K * AHV_K;
const AHV_TAB_H: f32 = 26.0 * K * AHV_K;
const AHV_TAB_TOP_W: f32 = 54.0 * K * AHV_K;
const AHV_TAB_BOT_W: f32 = 64.0 * K * AHV_K;
const AHV_TAB_SIZE: f32 = 16.0 * K * AHV_K;
const AHV_FOLD: f32 = 5.0 * K * AHV_K;
const AHV_SLOT_SIZE: f32 = 11.0 * K * AHV_K;
const AHV_BAR_INSET: f32 = 6.0 * K * AHV_K;
const AHV_HEAD_X0: f32 = CG_HEAD_X0 * AHV_K;
const AHV_HEAD_T: f32 = CG_HEAD_T * AHV_K;
const AHV_TALL: f32 = CG_TALL * AHV_K;
const AHV_HEAD_CHAMF: f32 = CG_HEAD_CHAMF * AHV_K;
const AHV_HEAD_SHADOW: f32 = CG_HEAD_SHADOW * AHV_K;
const AHV_MARK_W: f32 = CG_MARK_W * AHV_K;
const AHV_MARK_THIN: f32 = CG_MARK_THIN * AHV_K;
const AHV_MARK_SLANT: f32 = AHV_TALL * SLOPE_K;
const AHV_HEAD_TITLE_SIZE: f32 = CG_TITLE_SIZE * AHV_K;
const AHV_HEAD_TITLE_BASE: f32 = CG_TITLE_BASE * AHV_K;
const AHV_HEAD_TITLE_X: f32 = AHV_HEAD_X0 + AHV_MARK_SLANT + AHV_MARK_W + AHV_MARK_THIN
    - (AHV_HEAD_TITLE_BASE - AHV_HEAD_T - AHV_HEAD_TITLE_SIZE * 0.72) * SLOPE_K
    + 25.0 * K * AHV_K;
const AHV_CHIP_TOP: f32 = CG_CHIP_TOP * AHV_K;
const AHV_CHIP_H: f32 = CG_CHIP_H * AHV_K;
const AHV_CHIP_SIZE: f32 = CG_CHIP_SIZE * AHV_K;
const AHV_CHIP_PAD_X: f32 = CG_CHIP_PAD_X * AHV_K;
const AHV_CHIP_GAP: f32 = 8.0 * K * AHV_K;
const AHV_CHIP_X0: f32 = AHV_HEAD_TITLE_X
    - (AHV_CHIP_TOP + AHV_CHIP_H - (AHV_HEAD_TITLE_BASE - AHV_HEAD_TITLE_SIZE * 0.72)) * SLOPE_K;
const AHV_PLAYER_PAD_RIGHT: f32 = 56.0 * K * AHV_K;
const AHV_PLAYER_SCALE: f32 = 1.25;
const AHV_BAND_GAP: f32 = 34.0 * K * AHV_K;
const AHV_FOOT_GAP: f32 = CG_FOOT_GAP * AHV_K;
const AHV_FOOT_H: f32 = CG_FOOT_H * AHV_K;
const AHV_FOOT_PAD: f32 = CG_FOOT_PAD * AHV_K;
const AHV_FOOT_SIZE: f32 = CG_FOOT_SIZE * AHV_K;
const AHV_CONTENT_Y: f32 = CG_CONTENT_Y * AHV_K;

const INK_STRONG: Color = Color::new(0x24, 0x1D, 0x2F, 0xFF);
const INK_SUB: Color = Color::new(0x5A, 0x4F, 0x6A, 0xFF);
const PAPER_TOP: Color = Color::new(0xFB, 0xFC, 0xFE, 0xFF);
const PAPER_HI: Color = Color::new(0xFD, 0xFD, 0xFF, 0xFF);
const PAPER_MID: Color = Color::new(0xFA, 0xFB, 0xFE, 0xFF);
const PAPER_LOW: Color = Color::new(0xF8, 0xF9, 0xFD, 0xFF);
const INK_KICKER: Color = Color::new(0x6D, 0x65, 0x7A, 0xFF);
const LEDGER_RULE: Color = Color::new(0xC8, 0xC1, 0xD2, 0x78);
const CG_RAIL_WATER: Color = Color::new(0xFF, 0xFF, 0xFF, 0xC0);

const BRAND_LOGO_W: f32 = 216.0;
const BRAND_LOGO_Y: f32 = 26.25;
const BRAND_LOGO_MARK: f32 = 0.64;
const BRAND_KICKER_SIZE: f32 = 12.0;
const BRAND_KICKER_TRACK: f32 = 2.4;

const SPACED_RS: f32 = 1.4;
const BRAND_KICKER_BASE: f32 = 96.0;
const BRAND_KICKER_OPT_X: f32 = 3.0;
const BRAND_RULE_Y: f32 = 106.5;
const LEDGER_HEAD_GAP: f32 = 27.0;

const PUSH_CARD_WHITE: Color = Color::new(0xFF, 0xFF, 0xFF, 0xFF);
const PUSH_CARD_ICE: Color = Color::new(0xF1, 0xF5, 0xF9, 0xFF);
const PUSH_ACCENT: Color = Color::new(0x0E, 0xA5, 0xE9, 0xFF);
const PUSH_ACCENT_BG: Color = Color::new(0xE0, 0xF2, 0xFE, 0xFF);
const PUSH_INK: Color = Color::new(0x1E, 0x29, 0x3B, 0xFF);
const PUSH_INK_SUB: Color = Color::new(0x64, 0x74, 0x8B, 0xFF);
const PUSH_INK_MUTED: Color = Color::new(0xCB, 0xD5, 0xE1, 0xFF);
const PUSH_BG_MASK: Color = Color::new(0xF8, 0xFA, 0xFC, 0xDB);
const SLOT_BASE: Color = Color::new(0xE2, 0xE9, 0xF2, 0xFF);
const SLOT_LINE: Color = Color::new(0xD5, 0xE0, 0xED, 0xFF);
const SLOT_HI: Color = Color::new(0xFF, 0xFF, 0xFF, 0xD9);

const CG_BG: Color = Color::new(0xEE, 0xF2, 0xF6, 0xFF);
const CG_WHITE: Color = Color::new(0xFF, 0xFF, 0xFF, 0xFF);
const CG_FACE_A: Color = Color::new(0xE0, 0xE8, 0xF3, 0xFF);
const CG_SKY: Color = Color::new(0x38, 0xBD, 0xF8, 0xFF);
const CG_INK: Color = Color::new(0x1E, 0x29, 0x3B, 0xFF);
const CG_SHADOW: Color = Color::new(0xDD, 0xE5, 0xF0, 0xFF);
const CG_FOOTER_INK: Color = Color::new(0x94, 0xA3, 0xB8, 0xFF);
const CG_RAIL_LOW: Color = Color::new(0xE2, 0xEB, 0xF5, 0xFF);
const CG_DIVIDER: Color = Color::new(0xBA, 0xE6, 0xFD, 0xFF);
const CG_LOCK_SHADE_INK: Color = Color::new(0x1E, 0x29, 0x3B, 0x1C);
const AHV_GRADE_INK: Color = Color::new(0x02, 0x84, 0xC7, 0xFF);
const AHV_PILL_FILL: Color = Color::new(0xE6, 0xF2, 0xFC, 0xFF);
const AHV_ROW2_FILL: Color = Color::new(0xF4, 0xF7, 0xFB, 0xFF);
const AHV_ROW2_FILL_OFF: Color = Color::new(0xEE, 0xF2, 0xF7, 0xFF);
const AHV_INK_SCORE: Color = Color::new(0x1E, 0x29, 0x3B, 0xFF);
const AHV_INK_JUDGE: Color = Color::new(0x64, 0x74, 0x8B, 0xFF);
const AHV_UNLIT_TITLE: Color = Color::new(0x64, 0x74, 0x8B, 0xFF);
const AHV_UNLIT_TAG: Color = Color::new(0x94, 0xA3, 0xB8, 0xFF);
const AHV_FOLD_INK: Color = Color::new(0xCB, 0xD5, 0xE1, 0xFF);
const AHV_UNLIT_FACE: Color = Color::new(0xE7, 0xEC, 0xF4, 0xFF);
const AHV_UNLIT_SHADOW: Color = Color::new(0xE6, 0xEC, 0xF3, 0xFF);
const AHV_PAPER_MIST: Color = Color::new(0xE2, 0xE8, 0xF0, 0xFF);

const LIST_HEAD: &str = "SCORE LIST";

const TABLE_TITLE_TEXT: &str = "CHART CONSTANTS";

const TABLE_FOOTER_CHARTS: &str = "VIEW  CHARTS  //  BANDED BY CONSTANT";

const TABLE_FOOTER_SONGS: &str = "VIEW  SONGS  //  SORTED BY MAX CONSTANT";

const AHV_TITLE_TEXT: &str = "CHART COMPLETION";
const FOOT_SIGN: &str = "Generated By Okamikko";

fn clamp_height(value: f32) -> i32 {
    (value.round() as i32).clamp(MIN_HEIGHT, MAX_HEIGHT)
}

fn request_for(width: i32, height: i32) -> RenderRequest {
    super::harness::card_request(width, height)
}

fn st(family: &str, size: f32, color: Color) -> TextStyle {
    theme::style(family, size.round().max(1.0) as u32, color)
}

fn font_text(size: f32, color: Color) -> TextStyle {
    st(theme::FONT_NARROW, size, color)
}

fn font_display(size: f32, color: Color) -> TextStyle {
    st(theme::FONT_NARROW_BOLD, size, color)
}

fn font_number(size: f32, color: Color) -> TextStyle {
    st(theme::FONT_TITLE, size, color)
}

fn font_score(size: f32, color: Color) -> TextStyle {
    st(theme::FONT_SOFT, size, color)
}

fn font_bold(size: f32, color: Color) -> TextStyle {
    st(CG_TITLE_FONT, size, color)
}

fn area(x: f32, y: f32, w: f32, h: f32) -> Rect {
    widgets::rect(x, y, w, h)
}

fn fade(color: Color) -> Color {
    Color::new(
        color.r,
        color.g,
        color.b,
        (color.a as f32 * 0.94).round() as u8,
    )
}

fn with_alpha(color: Color, alpha: u8) -> Color {
    Color::new(color.r, color.g, color.b, alpha)
}

fn mix(a: u8, b: u8, k: f32) -> u8 {
    let value = a as f32 + (b as f32 - a as f32) * k;
    value.clamp(0.0, 255.0).round() as u8
}

fn hatch(kit: &CanvasKit<'_>, box_: Rect, step: i32, width: i32, color: Color) {
    let w = box_.width();
    let h = box_.height();
    if w <= 0.0 || h <= 0.0 || step <= 0 {
        return;
    }
    sketch::clip_round(kit, box_, 0.0, || {
        let step = step as f32;
        let line = width.max(1) as f32;
        let mut offset = 0.0;
        while offset < w + h {
            sketch::fill_polygon(
                kit,
                &[
                    (box_.left() + offset, box_.top()),
                    (box_.left() + offset + line, box_.top()),
                    (box_.left() + offset + line - h, box_.bottom()),
                    (box_.left() + offset - h, box_.bottom()),
                ],
                color,
            );
            offset += step;
        }
    });
}

fn paper_stop(u: f32) -> Color {
    let stops = [PAPER_TOP, PAPER_HI, PAPER_MID, PAPER_LOW];
    let last = stops.len() - 1;
    let f = (u.clamp(0.0, 1.0)) * last as f32;
    let index = f as usize;
    if index >= last {
        return stops[last];
    }
    let k = f - index as f32;
    let from = stops[index];
    let to = stops[index + 1];
    let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * k) as u8;
    Color::new(
        mix(from.r, to.r),
        mix(from.g, to.g),
        mix(from.b, to.b),
        mix(from.a, to.a),
    )
}

fn ledger_paper(kit: &CanvasKit<'_>, width: f32, height: f32) {
    if width <= 0.0 || height <= 0.0 {
        return;
    }
    let rows = height.round().max(1.0) as i32;
    for y in 0..rows {
        let u = (y as f32 + 0.5) / rows as f32;
        let mut color = paper_stop(u);
        let grain = (y as u32).wrapping_mul(0x9E37_79B1) & 0x8000 == 0;
        if grain && color.r > 0 && color.g > 0 && color.b > 0 {
            color = Color::new(color.r - 1, color.g - 1, color.b - 1, color.a);
        }
        kit.fill_rect(area(0.0, y as f32, width, 1.0), color);
    }
}

fn art_wash(kit: &CanvasKit<'_>, key: &str, width: f32, height: f32, top: u8, mid: u8, bottom: u8) {
    if key.is_empty() || width <= 0.0 || height <= 0.0 {
        return;
    }
    let _ = kit.draw_asset_cover(key, area(0.0, 0.0, width, height), 0.0);
    let upper = mix(top, mid, 0.60);
    let lower = mix(bottom, mid, 0.85);
    let gradient = sketch::gradient_stops(&[
        (0.0, with_alpha(PAPER_TOP, top)),
        (0.25, with_alpha(PAPER_TOP, upper)),
        (0.5, with_alpha(PAPER_TOP, mid)),
        (0.75, with_alpha(PAPER_TOP, lower)),
        (1.0, with_alpha(PAPER_TOP, bottom)),
    ]);
    let _ = kit.fill_gradient(area(0.0, 0.0, width, height), 0.0, &gradient);
}

fn ledger_art_wash(kit: &CanvasKit<'_>, key: &str, width: f32, height: f32) {
    art_wash(kit, key, width, height, 0xC4, 0xE6, 0xF2);
}

fn thumb(kit: &CanvasKit<'_>, x: f32, y: f32, size: f32, radius: f32, key: &str, fallback: Color) {
    let box_ = area(x, y, size, size);
    if key.is_empty() || kit.draw_asset_cover(key, box_, radius).is_err() {
        sketch::fill_round(kit, box_, radius, fallback);
    }
}

fn player_block(kit: &CanvasKit<'_>, badge: &PlayerBadge, right: f32, center_y: f32, scale: f32) {
    let scale = if scale > 0.0 { scale } else { 1.0 };
    let block = player::Block::new(badge, right, center_y)
        .with_left(LIST_PAD_L)
        .with_scale(scale);
    player::paint(kit, &block);
}

fn clip_polygon<R>(kit: &CanvasKit<'_>, points: &[(f32, f32)], body: impl FnOnce() -> R) -> R {
    let mut builder = PathBuilder::new();
    if let Some((x, y)) = points.first() {
        builder.move_to((*x, *y));
        for (x, y) in &points[1..] {
            builder.line_to((*x, *y));
        }
        builder.close();
    }
    let path = builder.detach();
    let canvas = kit.canvas();
    canvas.save();
    canvas.clip_path(&path, None, true);
    let out = body();
    canvas.restore();
    out
}

struct LedgerFrame {
    left: f32,
    right: f32,
    title_base: f32,
    title_size: f32,
    note_base: f32,
    note_size: f32,
    brand_right: bool,
    brand_rule_y: f32,
    no_brand: bool,
}

struct LedgerHead<'a> {
    head: &'a str,
    kicker: &'a str,
    note: &'a str,
}

fn ledger_logo(kit: &CanvasKit<'_>, x: f32, y: f32, width: f32) {
    let key = "logo/arcaea_logo.png";
    let Ok(texture) = kit.asset(key) else {
        return;
    };
    let Some((iw, ih)) = texture.dimensions() else {
        return;
    };
    if iw <= 0 || ih <= 0 || width <= 0.0 {
        return;
    }
    let Some(image) = texture.image() else {
        return;
    };
    let full_h = width * ih as f32 / iw as f32;
    let canvas = kit.canvas();
    canvas.save();
    canvas.clip_rect(
        area(x, y, width, full_h * BRAND_LOGO_MARK),
        ClipOp::Intersect,
        true,
    );
    let _ = kit.image_rect(image, area(x, y, width, full_h));
    canvas.restore();
}

fn ledger_brand(kit: &CanvasKit<'_>, kicker: &str, frame: &LedgerFrame) {
    let logo_x = if frame.brand_right {
        frame.right - BRAND_LOGO_W
    } else {
        frame.left
    };
    ledger_logo(kit, logo_x, BRAND_LOGO_Y, BRAND_LOGO_W);
    if !kicker.is_empty() {
        let track = BRAND_KICKER_TRACK / SPACED_RS;
        let style = font_display(BRAND_KICKER_SIZE, INK_KICKER);
        let width = sketch::text_spaced_width(kit, &style, kicker, track);
        let x = if frame.brand_right {
            logo_x + (BRAND_LOGO_W - width) / 2.0
        } else {
            logo_x + BRAND_KICKER_OPT_X
        };
        sketch::text_spaced(
            kit,
            &style,
            kicker,
            x,
            BRAND_KICKER_BASE,
            INK_KICKER,
            track,
        );
    }
    let rule_y = if frame.brand_rule_y > 0.0 {
        frame.brand_rule_y
    } else {
        BRAND_RULE_Y
    };
    kit.fill_rect(
        area(frame.left, rule_y, (frame.right - frame.left).max(0.0), 1.0),
        LEDGER_RULE,
    );
}

fn ledger_head(kit: &CanvasKit<'_>, head: LedgerHead<'_>, frame: LedgerFrame) {
    let (title, kicker) = if head.head.trim().is_empty() {
        (head.kicker, "")
    } else {
        (head.head, head.kicker)
    };
    if !frame.no_brand {
        ledger_brand(kit, kicker, &frame);
    }
    let mut avail = (frame.right - frame.left).max(0.0);
    if frame.brand_right {
        let room = frame.right - BRAND_LOGO_W - LEDGER_HEAD_GAP - frame.left;
        if room > 0.0 && room < avail {
            avail = room;
        }
    }
    if avail < 1.0 {
        return;
    }
    if !title.is_empty() {
        let style = font_display(frame.title_size, INK_STRONG);
        let text = sketch::ellipsize(kit, &style, title, avail);
        sketch::text(kit, &style, &text, frame.left, frame.title_base, INK_STRONG);
    }
    if !head.note.is_empty() {
        let style = font_text(frame.note_size, INK_SUB);
        let text = sketch::ellipsize(kit, &style, head.note, avail);
        sketch::text(kit, &style, &text, frame.left, frame.note_base, INK_SUB);
    }
}

fn ledger_foot(kit: &CanvasKit<'_>, foot_y: f32, x0: f32, x1: f32) {
    for offset in [4.0, 7.0] {
        kit.fill_rect(
            area(x0, foot_y + offset * 0.75, (x1 - x0).max(0.0), 1.0),
            Color::new(0xC8, 0xC1, 0xD2, 0x78),
        );
    }
    let base = foot_y + 4.0 * 0.75 + 22.0;
    sketch::text(kit, &font_text(15.0, INK_SUB), FOOT_SIGN, x0, base, INK_SUB);
}

fn empty_slot(kit: &CanvasKit<'_>, box_: Rect, radius: f32) {
    if box_.width() <= 0.0 || box_.height() <= 0.0 {
        return;
    }
    sketch::clip_round(kit, box_, radius, || {
        sketch::fill_round(kit, box_, radius, fade(SLOT_BASE));
        hatch(kit, box_, 11, 2, fade(SLOT_LINE));
        kit.fill_rect(
            area(box_.left(), box_.bottom() - 2.0, box_.width(), 2.0),
            fade(SLOT_HI),
        );
        kit.fill_rect(
            area(box_.right() - 2.0, box_.top(), 2.0, box_.height()),
            fade(SLOT_HI),
        );
    });
}

fn grade_label(grade: Grade) -> &'static str {
    match grade {
        Grade::ExPlus => "EX+",
        Grade::Pm => "PM",
        Grade::Fr => "FR",
        Grade::Ex => "EX",
        Grade::Aa => "AA",
        Grade::A => "A",
        Grade::B => "B",
        Grade::C => "C",
        Grade::D => "D",
    }
}

fn thumb_key(song_id: &str) -> String {
    if song_id.is_empty() {
        String::new()
    } else {
        format!("jacket_thumbs/{song_id}.jpg")
    }
}

fn thousands(value: i64) -> String {
    sketch::thousands(value)
}

#[derive(Debug, Clone, Copy)]
struct CardPos {
    x: f32,
    y: f32,
    entry: usize,
}

#[derive(Debug, Clone)]
struct BandHead {
    box_: Rect,
    difficulty: Difficulty,
    count: usize,
}

#[derive(Debug, Clone, Default)]
struct CardPlan {
    cards: Vec<CardPos>,
    slots: Vec<CardPos>,
    heads: Vec<BandHead>,
    banded: bool,
    foot_y: f32,
    height: f32,
}

fn plan_list_cards(count: usize) -> CardPlan {
    let mut plan = CardPlan::default();
    for index in 0..count {
        let col = index % LIST_CARD_COLS;
        let row = index / LIST_CARD_COLS;
        plan.cards.push(CardPos {
            x: LIST_PAD_L + col as f32 * LIST_STEP_X,
            y: LIST_GRID_TOP + row as f32 * LIST_STEP_Y,
            entry: index,
        });
    }
    let rows = count.div_ceil(LIST_CARD_COLS);
    let slots = (LIST_CARD_COLS - count % LIST_CARD_COLS) % LIST_CARD_COLS;
    if slots > 0 {
        let first = LIST_CARD_COLS - slots;
        let y = LIST_GRID_TOP + (rows as f32 - 1.0) * LIST_STEP_Y;
        for index in 0..slots {
            plan.slots.push(CardPos {
                x: LIST_PAD_L + (first + index) as f32 * LIST_STEP_X,
                y,
                entry: 0,
            });
        }
    }
    plan.foot_y = LIST_GRID_TOP + rows as f32 * LIST_STEP_Y + LIST_FOOT_GAP;
    plan.height = plan.foot_y + LIST_BOTTOM_PAD - LIST_FOOT_GAP;
    plan
}

const LIST_BAND_ORDER: [Difficulty; 6] = [
    Difficulty::Beyond,
    Difficulty::Ins,
    Difficulty::Eternal,
    Difficulty::Future,
    Difficulty::Present,
    Difficulty::Past,
];

pub const TABLE_CELL_ORDER: [Difficulty; 5] = [
    Difficulty::Past,
    Difficulty::Present,
    Difficulty::Future,
    Difficulty::Eternal,
    Difficulty::Beyond,
];

fn difficulty_bands(entries: &[ListEntry]) -> Vec<Difficulty> {
    let mut bands: Vec<Difficulty> = LIST_BAND_ORDER
        .into_iter()
        .filter(|band| entries.iter().any(|entry| entry.difficulty == *band))
        .collect();
    bands.sort_by_key(|band| diff_display_weight(*band));
    bands
}

fn plan_list_bands(entries: &[ListEntry], bands: &[Difficulty]) -> CardPlan {
    let mut plan = CardPlan {
        banded: true,
        ..CardPlan::default()
    };
    let mut y = LIST_GRID_TOP;
    for band in bands {
        let members: Vec<usize> = entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.difficulty == *band)
            .map(|(index, _)| index)
            .collect();
        plan.heads.push(BandHead {
            box_: area(LIST_PAD_L, y, LIST_RIGHT - LIST_PAD_L, LIST_BAND_H),
            difficulty: *band,
            count: members.len(),
        });
        y += LIST_BAND_H + LIST_BAND_GAP;
        for (slot, index) in members.iter().enumerate() {
            plan.cards.push(CardPos {
                x: LIST_PAD_L + (slot % LIST_CARD_COLS) as f32 * LIST_STEP_X,
                y: y + (slot / LIST_CARD_COLS) as f32 * LIST_STEP_Y,
                entry: *index,
            });
        }
        let rows = members.len().div_ceil(LIST_CARD_COLS);
        let rest = (LIST_CARD_COLS - members.len() % LIST_CARD_COLS) % LIST_CARD_COLS;
        if rest > 0 {
            let last = y + (rows as f32 - 1.0) * LIST_STEP_Y;
            for slot in 0..rest {
                let col = LIST_CARD_COLS - rest + slot;
                plan.slots.push(CardPos {
                    x: LIST_PAD_L + col as f32 * LIST_STEP_X,
                    y: last,
                    entry: 0,
                });
            }
        }
        y += rows as f32 * LIST_STEP_Y - LIST_CARD_GAP_Y + LIST_BAND_TERM;
    }
    let content = (y - LIST_BAND_TERM).max(LIST_GRID_TOP);
    plan.foot_y = content + LIST_FOOT_GAP;
    plan.height = plan.foot_y + LIST_BOTTOM_PAD - LIST_FOOT_GAP;
    plan
}

fn list_pfl_text(entry: &ListEntry) -> String {
    if !entry.has_judgement {
        return String::from("—");
    }
    let mut text = format!("P{}", entry.perfect);
    if entry.shiny > 0 {
        text.push_str(&format!("(-{})", entry.perfect - entry.shiny));
    }
    format!("{text} F{} L{}", entry.far, entry.lost)
}

fn list_const_text(entry: &ListEntry) -> String {
    if !entry.has_constant || entry.constant <= 0.0 {
        return String::from("未收录");
    }
    format!("{:.1}", entry.constant)
}

fn list_loss_text(entry: &ListEntry) -> String {
    if !entry.has_constant || entry.constant <= 0.0 {
        return String::from("—");
    }
    match entry.loss {
        Some(loss) if loss > 0.0 => format!("LS {loss:.1}"),
        _ => String::from("—"),
    }
}

#[derive(Debug, Clone)]
pub struct ListEntry {
    pub song_id: String,
    pub title: String,
    pub artist: String,
    pub difficulty: Difficulty,
    pub level: String,
    pub constant: f64,
    pub has_constant: bool,
    pub score: i64,
    pub has_score: bool,
    pub has_judgement: bool,
    pub perfect: i64,
    pub shiny: i64,
    pub far: i64,
    pub lost: i64,
    pub loss: Option<f64>,
    pub grade: Option<Grade>,
    pub clear_type: Option<i32>,
    pub jacket: Option<String>,
}

impl ListEntry {
    pub fn new(
        song_id: impl Into<String>,
        title: impl Into<String>,
        difficulty: Difficulty,
        constant: f64,
    ) -> Self {
        Self {
            song_id: song_id.into(),
            title: title.into(),
            artist: String::new(),
            difficulty,
            level: String::new(),
            constant,
            has_constant: constant > 0.0,
            score: 0,
            has_judgement: false,
            perfect: 0,
            shiny: 0,
            far: 0,
            lost: 0,
            loss: None,
            has_score: false,
            grade: None,
            clear_type: None,
            jacket: None,
        }
    }

    pub fn with_artist(mut self, artist: impl Into<String>) -> Self {
        self.artist = artist.into();
        self
    }

    pub fn with_level(mut self, level: impl Into<String>) -> Self {
        self.level = level.into();
        self
    }

    pub fn with_score(mut self, score: i64) -> Self {
        self.score = score;
        self.has_score = score > 0;
        self
    }

    pub fn with_judgement(mut self, perfect: i64, shiny: i64, far: i64, lost: i64) -> Self {
        self.has_judgement = true;
        self.perfect = perfect;
        self.shiny = shiny;
        self.far = far;
        self.lost = lost;
        self
    }

    pub fn with_loss(mut self, loss: f64) -> Self {
        self.loss = Some(loss);
        self
    }

    pub fn with_grade(mut self, grade: Grade) -> Self {
        self.grade = Some(grade);
        self
    }

    pub fn with_clear(mut self, clear_type: i32) -> Self {
        self.clear_type = Some(clear_type);
        self
    }

    pub fn with_jacket(mut self, key: impl Into<String>) -> Self {
        let key = key.into();
        if !key.is_empty() {
            self.jacket = Some(key);
        }
        self
    }

    pub fn jacket_key(&self) -> String {
        match self.jacket.as_deref() {
            Some(key) => key.to_string(),
            None => thumb_key(&self.song_id),
        }
    }

    pub fn rating(&self) -> f64 {
        if self.has_score && self.has_constant {
            ptt::single_rating(self.constant, self.score)
        } else {
            0.0
        }
    }

    pub fn grade_or_computed(&self) -> Option<Grade> {
        self.grade
            .or_else(|| self.has_score.then(|| Grade::from_score(self.score)))
    }
}

#[derive(Debug, Clone)]
pub struct ListInput {
    pub title: String,
    pub subtitle: String,
    pub theme: Theme,
    pub badge: PlayerBadge,
    pub entries: Vec<ListEntry>,
    pub note: String,
    pub backdrop: String,
}

impl ListInput {
    pub fn new(badge: PlayerBadge) -> Self {
        Self {
            title: CardKind::List.label().to_string(),
            subtitle: String::new(),
            theme: Theme::DEFAULT,
            badge,
            entries: Vec::new(),
            note: String::new(),
            backdrop: String::new(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = subtitle.into();
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = note.into();
        self
    }

    pub fn with_entry(mut self, entry: ListEntry) -> Self {
        self.entries.push(entry);
        self
    }

    pub fn with_entries(mut self, entries: Vec<ListEntry>) -> Self {
        self.entries = entries;
        self
    }

    fn plan(&self) -> CardPlan {
        let bands = difficulty_bands(&self.entries);
        if bands.len() > 1 {
            return plan_list_bands(&self.entries, &bands);
        }
        plan_list_cards(self.entries.len())
    }
    fn art_key(&self) -> String {
        if !self.backdrop.is_empty() {
            return format!("jackets/{}.jpg", self.backdrop);
        }
        self.entries
            .iter()
            .map(|entry| entry.song_id.as_str())
            .find(|id| !id.is_empty())
            .map(|id| format!("jackets/{id}.jpg"))
            .unwrap_or_default()
    }
}

impl Default for ListInput {
    fn default() -> Self {
        Self::new(PlayerBadge::new("Afkeru")).with_subtitle("曲目清单按单曲 PTT 排")
    }
}

pub fn list_rows(count: usize) -> usize {
    count.div_ceil(LIST_CARD_COLS)
}

pub fn list_body(count: usize) -> f32 {
    if count == 0 {
        return LIST_CARD_H;
    }
    list_rows(count) as f32 * LIST_STEP_Y - LIST_CARD_GAP_Y
}

pub fn list_card_pos(index: usize) -> Rect {
    let plan = plan_list_cards(index + 1);
    let pos = plan.cards[index];
    area(pos.x, pos.y, LIST_CARD_W, LIST_CARD_H)
}

pub fn size_of(input: &ListInput) -> (i32, i32) {
    let plan = input.plan();
    (LIST_WIDTH, clamp_height(plan.height))
}

pub fn request(input: &ListInput) -> RenderRequest {
    let (width, height) = size_of(input);
    request_for(width, height).with_scale(1.5)
}

pub fn card(input: ListInput) -> RenderCard {
    let request = request(&input);
    let label = format!(
        "{} {}x{}",
        CardKind::List.label(),
        request.width,
        request.height
    );
    let cache_key = super::memo::fingerprint("list", &input);
    RenderCard::new(request, label, move |kit| paint(kit, &input)).with_cache_key(cache_key)
}

pub fn paint(kit: &CanvasKit<'_>, input: &ListInput) {
    let plan = input.plan();
    let width = LIST_WIDTH as f32;
    let height = size_of(input).1 as f32;
    ledger_paper(kit, width, height);
    ledger_art_wash(kit, &input.art_key(), width, height);
    kit.fill_rect(area(0.0, 0.0, width, height), PUSH_BG_MASK);
    ledger_head(
        kit,
        LedgerHead {
            head: LIST_HEAD,
            kicker: "",
            note: &input.subtitle,
        },
        LedgerFrame {
            left: LIST_TITLE_X,
            right: LIST_RIGHT,
            title_base: LIST_TITLE_BASE,
            title_size: LIST_TITLE_SIZE,
            note_base: LIST_SUB_BASE,
            note_size: LIST_SUB_SIZE,
            brand_right: false,
            brand_rule_y: 0.0,
            no_brand: true,
        },
    );
    for head in &plan.heads {
        paint_band_head(kit, head);
    }
    if plan.cards.is_empty() {
        sketch::text(
            kit,
            &font_text(14.0 * K, PUSH_INK_SUB),
            "没有命中的成绩",
            LIST_PAD_L,
            LIST_GRID_TOP,
            PUSH_INK_SUB,
        );
    }
    for pos in &plan.cards {
        if let Some(entry) = input.entries.get(pos.entry) {
            paint_card(kit, *pos, entry);
        }
    }
    for pos in &plan.slots {
        list_slot(
            kit,
            area(pos.x, pos.y, LIST_CARD_W, LIST_CARD_H),
            plan.banded,
        );
    }
    ledger_foot(kit, plan.foot_y, LIST_PAD_L, LIST_RIGHT);
    player_block(
        kit,
        &input.badge,
        LIST_RIGHT,
        super::stats::compact_center_y(),
        1.0,
    );
}

fn paint_band_head(kit: &CanvasKit<'_>, head: &BandHead) {
    let tone = tone_of(head.difficulty);
    let path = cg_chamfer(head.box_, 0.0, LIST_BAND_CHAM, 0.0, LIST_BAND_CHAM);
    clip_polygon(kit, &path, || {
        sketch::fill_polygon(kit, &path, fade(PUSH_CARD_WHITE));
        kit.fill_rect(
            area(
                head.box_.left(),
                head.box_.top(),
                LIST_BAND_MARK_W,
                head.box_.height(),
            ),
            tone.bg,
        );
        kit.fill_rect(
            area(
                head.box_.left(),
                head.box_.top(),
                head.box_.width(),
                1.0 * K,
            ),
            PUSH_INK_MUTED,
        );
    });
    let label_style = font_display(LIST_BAND_SIZE, tone.bg);
    let label = head.difficulty.as_str();
    let baseline = head.box_.top() + LIST_BAND_H / 2.0 + LIST_BAND_SIZE * 0.36;
    sketch::text(
        kit,
        &label_style,
        label,
        head.box_.left() + LIST_BAND_TEXT_X,
        baseline,
        tone.bg,
    );
    let label_w = sketch::text_width(kit, &label_style, label);
    sketch::text(
        kit,
        &font_text(LIST_BAND_SUB_SIZE, PUSH_INK_SUB),
        &format!("{} 首", head.count),
        head.box_.left() + LIST_BAND_TEXT_X + label_w + LIST_BAND_TEXT_GAP,
        head.box_.top() + LIST_BAND_H / 2.0 + LIST_BAND_SUB_SIZE * 0.36,
        PUSH_INK_SUB,
    );
}

fn list_slot(kit: &CanvasKit<'_>, box_: Rect, banded: bool) {
    if !banded {
        empty_slot(kit, box_, LIST_CARD_R);
        return;
    }
    let path = cg_chamfer(box_, 0.0, LIST_BAND_CHAM, 0.0, LIST_BAND_CHAM);
    clip_polygon(kit, &path, || {
        sketch::fill_polygon(kit, &path, fade(PUSH_CARD_ICE));
        hatch(kit, box_, CG_LOCK_STEP, CG_LOCK_LINE, SLOT_LINE);
    });
}

fn paint_card(kit: &CanvasKit<'_>, pos: CardPos, entry: &ListEntry) {
    let x = pos.x;
    let y = pos.y;
    let card = area(x, y, LIST_CARD_W, LIST_CARD_H);
    sketch::clip_round(kit, card, LIST_CARD_R, || {
        kit.fill_rect(card, fade(PUSH_CARD_WHITE));
        kit.fill_rect(
            area(
                x,
                y + LIST_SPLIT_Y,
                LIST_ICE_W,
                LIST_CARD_H - LIST_SPLIT_Y,
            ),
            fade(PUSH_CARD_ICE),
        );
        sketch::fill_polygon(
            kit,
            &[
                (x + LIST_ICE_W, y + LIST_SPLIT_Y),
                (x + LIST_CARD_W, y + LIST_SPLIT_Y),
                (x + LIST_CARD_W, y + LIST_CARD_H),
                (x + LIST_LEAN_X, y + LIST_CARD_H),
            ],
            fade(PUSH_ACCENT_BG),
        );
    });
    thumb(
        kit,
        x + LIST_THUMB_X,
        y + LIST_THUMB_Y,
        LIST_THUMB_SIZE,
        LIST_THUMB_R,
        &entry.jacket_key(),
        PUSH_CARD_ICE,
    );
    widgets::grade_badge(
        kit,
        x + LIST_GRADE_CX,
        y + LIST_GRADE_CY,
        LIST_GRADE_SZ,
        entry.grade_or_computed(),
    );
    let title_style = font_display(LIST_ENTRY_TITLE_SIZE, PUSH_INK);
    if !entry.title.is_empty() {
        let text = sketch::ellipsize(kit, &title_style, &entry.title, LIST_ENTRY_TITLE_W);
        sketch::text(
            kit,
            &title_style,
            &text,
            x + LIST_ENTRY_TITLE_X,
            y + LIST_ENTRY_TITLE_BASE,
            PUSH_INK,
        );
    }
    let artist = entry.artist.trim();
    if !artist.is_empty() {
        let style = font_text(LIST_ARTIST_SIZE, PUSH_INK_SUB);
        let text = sketch::ellipsize(kit, &style, artist, LIST_ARTIST_W);
        sketch::text(
            kit,
            &style,
            &text,
            x + LIST_ENTRY_TITLE_X,
            y + LIST_ARTIST_BASE,
            PUSH_INK_SUB,
        );
    }
    let cons_style = font_display(LIST_META_SIZE, PUSH_ACCENT);
    let cons = list_const_text(entry);
    let cons_x = x + LIST_ENTRY_TITLE_X;
    let cons_base = y + LIST_META_BASE;
    let cons_width = sketch::text(kit, &cons_style, &cons, cons_x, cons_base, PUSH_ACCENT);
    let mut cursor = cons_x + cons_width + 8.0;
    let rating = entry.rating();
    if rating > 0.0 {
        cursor += sketch::text(
            kit,
            &font_display(LIST_META_SIZE, PUSH_INK_SUB),
            &format!("> {rating:.3}"),
            cursor,
            cons_base,
            PUSH_INK_SUB,
        ) + 8.0;
    }
    let diff = entry.difficulty.as_str();
    if !diff.trim().is_empty() {
        sketch::text(
            kit,
            &font_display(LIST_META_SIZE, PUSH_INK_SUB),
            &format!("· {diff}"),
            cursor,
            cons_base,
            PUSH_INK_SUB,
        );
    }
    sketch::text(
        kit,
        &font_number(LIST_TARGET_SIZE, PUSH_INK),
        &thousands(entry.score),
        x + LIST_PAD_X,
        y + LIST_TARGET_BASE,
        PUSH_INK,
    );
    sketch::text(
        kit,
        &font_text(LIST_FROM_SIZE, PUSH_INK_SUB),
        &list_pfl_text(entry),
        x + LIST_PAD_X,
        y + LIST_FROM_BASE,
        PUSH_INK_SUB,
    );
    sketch::text_center(
        kit,
        &font_number(LIST_LOSS_SIZE, PUSH_ACCENT),
        &list_loss_text(entry),
        x + LIST_LOSS_CX,
        y + LIST_LOSS_BASE,
        PUSH_ACCENT,
    );
}

#[derive(Debug, Clone)]
pub struct TableChart {
    pub song_id: String,
    pub title: String,
    pub difficulty: Difficulty,
    pub constant: f64,
    pub has_constant: bool,
    pub score: i64,
    pub has_score: bool,
    pub jacket: Option<String>,
}

impl TableChart {
    pub fn new(
        song_id: impl Into<String>,
        title: impl Into<String>,
        difficulty: Difficulty,
        constant: f64,
    ) -> Self {
        Self {
            song_id: song_id.into(),
            title: title.into(),
            difficulty,
            constant,
            has_constant: constant > 0.0,
            score: 0,
            has_score: false,
            jacket: None,
        }
    }

    pub fn with_score(mut self, score: i64) -> Self {
        self.score = score;
        self.has_score = score > 0;
        self
    }

    pub fn with_jacket(mut self, key: impl Into<String>) -> Self {
        let key = key.into();
        if !key.is_empty() {
            self.jacket = Some(key);
        }
        self
    }

    pub fn jacket_key(&self) -> String {
        match self.jacket.as_deref() {
            Some(key) => key.to_string(),
            None => thumb_key(&self.song_id),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TableSheet {
    pub band: String,
    pub charts: Vec<TableChart>,
}

impl TableSheet {
    pub fn new(band: impl Into<String>) -> Self {
        Self {
            band: band.into(),
            charts: Vec::new(),
        }
    }

    pub fn with_chart(mut self, chart: TableChart) -> Self {
        self.charts.push(chart);
        self
    }

    pub fn with_charts(mut self, charts: Vec<TableChart>) -> Self {
        self.charts = charts;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableMode {
    #[default]
    Charts,
    Songs,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TableCell {
    pub difficulty: Difficulty,
    pub real: Difficulty,
    pub constant: f64,
    pub known: bool,
}

impl TableCell {
    pub fn new(difficulty: Difficulty, constant: f64) -> Self {
        Self {
            difficulty,
            real: difficulty,
            constant,
            known: constant > 0.0,
        }
    }

    pub fn unknown(difficulty: Difficulty) -> Self {
        Self {
            difficulty,
            real: difficulty,
            constant: 0.0,
            known: false,
        }
    }

    pub fn with_real(mut self, real: Difficulty) -> Self {
        self.real = real;
        self
    }
}

#[derive(Debug, Clone)]
pub struct TableSong {
    pub song_id: String,
    pub title: String,
    pub artist: String,
    pub cells: Vec<TableCell>,
    pub jacket: Option<String>,
}

impl TableSong {
    pub fn new(song_id: impl Into<String>, title: impl Into<String>, cells: Vec<TableCell>) -> Self {
        Self {
            song_id: song_id.into(),
            title: title.into(),
            artist: String::new(),
            cells,
            jacket: None,
        }
    }

    pub fn with_artist(mut self, artist: impl Into<String>) -> Self {
        self.artist = artist.into();
        self
    }

    pub fn with_jacket(mut self, key: impl Into<String>) -> Self {
        let key = key.into();
        if !key.is_empty() {
            self.jacket = Some(key);
        }
        self
    }

    pub fn jacket_key(&self) -> String {
        match self.jacket.as_deref() {
            Some(key) => key.to_string(),
            None => thumb_key(&self.song_id),
        }
    }

    pub fn known_cells(&self) -> Vec<TableCell> {
        TABLE_CELL_ORDER
            .into_iter()
            .flat_map(|difficulty| {
                self.cells
                    .iter()
                    .copied()
                    .filter(move |cell| cell.known && cell.difficulty == difficulty)
            })
            .collect()
    }

    pub fn top_difficulty(&self) -> Option<Difficulty> {
        self.known_cells()
            .into_iter()
            .map(|cell| cell.difficulty)
            .min_by_key(|difficulty| diff_display_weight(*difficulty))
    }

    pub fn max_constant(&self) -> f64 {
        self.known_cells()
            .iter()
            .map(|cell| cell.constant)
            .fold(0.0, f64::max)
    }

    pub fn second_constant(&self) -> f64 {
        let mut values: Vec<f64> = self.known_cells().iter().map(|cell| cell.constant).collect();
        values.sort_by(|left, right| right.total_cmp(left));
        values.get(1).copied().unwrap_or(0.0)
    }
}

pub fn table_song_order(left: &TableSong, right: &TableSong) -> std::cmp::Ordering {
    let weight = |song: &TableSong| {
        song.top_difficulty()
            .map(diff_display_weight)
            .unwrap_or(u8::MAX)
    };
    right
        .max_constant()
        .total_cmp(&left.max_constant())
        .then_with(|| weight(left).cmp(&weight(right)))
        .then_with(|| right.second_constant().total_cmp(&left.second_constant()))
        .then_with(|| title_key(&left.title).cmp(&title_key(&right.title)))
        .then_with(|| left.song_id.cmp(&right.song_id))
}

pub fn table_songs_sorted(songs: &[TableSong]) -> Vec<TableSong> {
    let mut out = songs.to_vec();
    out.sort_by(table_song_order);
    out
}

#[derive(Debug, Clone)]
pub struct TableInput {
    pub title: String,
    pub subtitle: String,
    pub theme: Theme,
    pub badge: PlayerBadge,
    pub library: usize,
    pub mode: TableMode,
    pub sheets: Vec<TableSheet>,
    pub songs: Vec<TableSong>,
    pub note: String,
}

impl TableInput {
    pub fn new(badge: PlayerBadge) -> Self {
        Self {
            title: CardKind::Table.label().to_string(),
            subtitle: String::new(),
            theme: Theme::DEFAULT,
            badge,
            library: 0,
            mode: TableMode::Charts,
            sheets: Vec::new(),
            songs: Vec::new(),
            note: String::new(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = subtitle.into();
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = note.into();
        self
    }

    pub fn with_sheet(mut self, sheet: TableSheet) -> Self {
        self.sheets.push(sheet);
        self
    }

    pub fn with_sheets(mut self, sheets: Vec<TableSheet>) -> Self {
        self.sheets = sheets;
        self
    }

    pub fn with_library(mut self, library: usize) -> Self {
        self.library = library;
        self
    }

    pub fn with_mode(mut self, mode: TableMode) -> Self {
        self.mode = mode;
        self
    }
    pub fn with_song(mut self, song: TableSong) -> Self {
        self.songs.push(song);
        self
    }

    pub fn with_songs(mut self, songs: Vec<TableSong>) -> Self {
        self.songs = songs;
        self
    }
}

impl Default for TableInput {
    fn default() -> Self {
        Self::new(PlayerBadge::new("Afkeru")).with_subtitle("定数表按等级降序")
    }
}

fn title_key(title: &str) -> String {
    title.to_lowercase()
}

pub fn table_band_key(constant: f64) -> i64 {
    if constant <= 0.0 {
        return 0;
    }
    (constant * 10.0 + 1e-9).floor() as i64
}

pub fn table_band_label(constant: f64) -> String {
    let key = table_band_key(constant);
    if key < 10 {
        return String::from("—");
    }
    format!("{:.1}", key as f64 / 10.0)
}

fn table_chart_order(left: &TableChart, right: &TableChart) -> std::cmp::Ordering {
    table_band_key(right.constant)
        .cmp(&table_band_key(left.constant))
        .then_with(|| diff_display_weight(left.difficulty).cmp(&diff_display_weight(right.difficulty)))
        .then_with(|| title_key(&left.title).cmp(&title_key(&right.title)))
        .then_with(|| left.song_id.cmp(&right.song_id))
}

fn table_groups(input: &TableInput) -> Vec<(String, Vec<&TableChart>)> {
    input
        .sheets
        .iter()
        .filter(|sheet| !sheet.charts.is_empty())
        .map(|sheet| {
            let mut charts: Vec<&TableChart> = sheet.charts.iter().collect();
            charts.sort_by(|left, right| table_chart_order(left, right));
            (sheet.band.clone(), charts)
        })
        .collect()
}

#[derive(Debug, Clone, Default)]
struct TableBand {
    top_label: String,
    bot_label: String,
    top: f32,
    bottom: f32,
    seq: usize,
    rows: usize,
    rail: Rect,
    cards: Vec<Rect>,
    slots: Vec<Rect>,
}

#[derive(Debug, Clone, Default)]
struct TableLayout {
    bands: Vec<TableBand>,
    foot_y: f32,
    height: f32,
}

fn group_height(count: usize) -> f32 {
    let rows = count.div_ceil(CG_COLS);
    if rows == 0 {
        return 0.0;
    }
    rows as f32 * CG_CARD_H + (rows as f32 - 1.0) * CG_GAP_Y
}

fn card_rect(row_top: f32, slot: usize, index: usize) -> Rect {
    let col = (slot + index) % CG_COLS;
    let row = (slot + index) / CG_COLS;
    area(
        CG_COL_X0 + col as f32 * CG_PITCH_X,
        row_top + row as f32 * CG_PITCH_Y,
        CG_CARD_W,
        CG_CARD_H,
    )
}

fn slot_rects(row_top: f32, slot: usize) -> Vec<Rect> {
    if slot >= CG_COLS {
        return Vec::new();
    }
    (slot..CG_COLS)
        .map(|col| {
            area(
                CG_COL_X0 + col as f32 * CG_PITCH_X,
                row_top,
                CG_CARD_W,
                CG_CARD_H,
            )
        })
        .collect()
}

fn table_layout(groups: &[(String, usize)]) -> TableLayout {
    let mut layout = TableLayout::default();
    let mut y = CG_CONTENT_Y;
    let mut seq = 0;
    let mut index = 0;
    while index < groups.len() {
        let (label, count) = &groups[index];
        if *count == 0 {
            index += 1;
            continue;
        }
        if *count > CG_COLS {
            let rows = count.div_ceil(CG_COLS);
            let height = group_height(*count);
            let mut band = TableBand {
                top_label: label.clone(),
                bot_label: label.clone(),
                top: y,
                bottom: y + height,
                seq,
                rows,
                rail: area(CG_LEFT, y, CG_RAIL_W, height),
                ..TableBand::default()
            };
            for k in 0..*count {
                band.cards.push(card_rect(y, 0, k));
            }
            if *count % CG_COLS != 0 {
                let last_top = y + (rows as f32 - 1.0) * CG_PITCH_Y;
                band.slots = slot_rects(last_top, *count % CG_COLS);
            }
            seq += *count;
            layout.bands.push(band);
            y += height + CG_GROUP_GAP;
            index += 1;
            continue;
        }
        let mut set: Vec<(String, usize)> = vec![(label.clone(), *count)];
        let mut total = *count;
        let mut next = index + 1;
        while next < groups.len() {
            let (other, count) = &groups[next];
            if *count == 0 {
                next += 1;
                continue;
            }
            if total + *count > CG_COLS {
                break;
            }
            set.push((other.clone(), *count));
            total += *count;
            next += 1;
        }
        let mut band = TableBand {
            top_label: set.first().map(|item| item.0.clone()).unwrap_or_default(),
            bot_label: set.last().map(|item| item.0.clone()).unwrap_or_default(),
            top: y,
            bottom: y + CG_CARD_H,
            seq,
            rows: 1,
            rail: area(CG_LEFT, y, CG_RAIL_W, CG_CARD_H),
            ..TableBand::default()
        };
        let mut col = 0;
        for (_, count) in &set {
            for k in 0..*count {
                band.cards.push(card_rect(y, col + k, 0));
            }
            col += *count;
            seq += *count;
        }
        if col < CG_COLS {
            band.slots = slot_rects(y, col);
        }
        layout.bands.push(band);
        y += CG_CARD_H + CG_GROUP_GAP;
        index = next;
    }
    let mut content = y - CG_GROUP_GAP;
    if content < CG_CONTENT_Y {
        content = CG_CONTENT_Y;
    }
    layout.foot_y = content + CG_FOOT_GAP;
    layout.height = layout.foot_y + CG_FOOT_H + CG_FOOT_PAD;
    layout
}

fn table_bands(input: &TableInput) -> (TableLayout, Vec<(String, usize)>) {
    let groups = table_groups(input);
    let ins: Vec<(String, usize)> = groups
        .iter()
        .map(|(band, charts)| (band.clone(), charts.len()))
        .collect();
    (table_layout(&ins), ins)
}

#[derive(Debug, Clone, Default)]
struct TileLayout {
    cards: Vec<Rect>,
    slots: Vec<Rect>,
    foot_y: f32,
    height: f32,
}

fn tile_rect(top: f32, index: usize) -> Rect {
    let col = index % CG_TILE_COLS;
    let row = index / CG_TILE_COLS;
    area(
        CG_LEFT + col as f32 * CG_TILE_PITCH_X,
        top + row as f32 * CG_TILE_PITCH_Y,
        CG_TILE_CARD_W,
        CG_TILE_H,
    )
}

fn tile_slots(top: f32, from: usize) -> Vec<Rect> {
    if from >= CG_TILE_COLS {
        return Vec::new();
    }
    (from..CG_TILE_COLS)
        .map(|col| {
            area(
                CG_LEFT + col as f32 * CG_TILE_PITCH_X,
                top,
                CG_TILE_CARD_W,
                CG_TILE_H,
            )
        })
        .collect()
}

fn tile_layout(count: usize) -> TileLayout {
    let mut layout = TileLayout::default();
    let mut content = CG_CONTENT_Y;
    if count > 0 {
        let rows = count.div_ceil(CG_TILE_COLS);
        for index in 0..count {
            layout.cards.push(tile_rect(CG_CONTENT_Y, index));
        }
        if let Some(rest) = count.checked_rem(CG_TILE_COLS).filter(|rest| *rest != 0) {
            let last = CG_CONTENT_Y + (rows as f32 - 1.0) * CG_TILE_PITCH_Y;
            layout.slots = tile_slots(last, rest);
        }
        content = CG_CONTENT_Y + rows as f32 * CG_TILE_H + (rows as f32 - 1.0) * CG_GAP_Y;
    }
    layout.foot_y = content + CG_FOOT_GAP;
    layout.height = layout.foot_y + CG_FOOT_H + CG_FOOT_PAD;
    layout
}

pub fn table_size(input: &TableInput) -> (i32, i32) {
    if input.mode == TableMode::Songs {
        return (TABLE_WIDTH, clamp_height(tile_layout(input.songs.len()).height));
    }
    let (layout, _) = table_bands(input);
    (TABLE_WIDTH, clamp_height(layout.height))
}

pub fn table_request(input: &TableInput) -> RenderRequest {
    let (width, height) = table_size(input);
    request_for(width, height)
}

pub fn table_card(input: TableInput) -> RenderCard {
    let request = table_request(&input);
    let label = format!(
        "{} {}x{}",
        CardKind::Table.label(),
        request.width,
        request.height
    );
    let cache_key = super::memo::fingerprint("table", &input);
    RenderCard::new(request, label, move |kit| paint_table(kit, &input)).with_cache_key(cache_key)
}

fn cg_head_plate(box_: Rect, slant: f32, chamf: f32) -> Vec<(f32, f32)> {
    vec![
        (box_.left() + slant, box_.top()),
        (box_.right(), box_.top()),
        (box_.right(), box_.bottom() - chamf),
        (box_.right() - chamf, box_.bottom()),
        (box_.left(), box_.bottom()),
    ]
}

fn cg_head_bar(box_: Rect, slant: f32, off: f32, width: f32) -> Vec<(f32, f32)> {
    vec![
        (box_.left() + off, box_.top()),
        (box_.left() + off + width, box_.top()),
        (box_.left() + off + width - slant, box_.bottom()),
        (box_.left() + off - slant, box_.bottom()),
    ]
}

fn cg_chamfer(box_: Rect, tl: f32, tr: f32, br: f32, bl: f32) -> Vec<(f32, f32)> {
    vec![
        (box_.left() + tl, box_.top()),
        (box_.right() - tr, box_.top()),
        (box_.right(), box_.top() + tr),
        (box_.right(), box_.bottom() - br),
        (box_.right() - br, box_.bottom()),
        (box_.left() + bl, box_.bottom()),
        (box_.left(), box_.bottom() - bl),
        (box_.left(), box_.top() + tl),
    ]
}

fn paragraphs(box_: Rect, lean: f32) -> Vec<(f32, f32)> {
    let dx = lean * box_.height();
    vec![
        (box_.left() + dx, box_.top()),
        (box_.right() + dx, box_.top()),
        (box_.right(), box_.bottom()),
        (box_.left(), box_.bottom()),
    ]
}

fn tag_shape(box_: Rect, top_w: f32, bottom_w: f32) -> Vec<(f32, f32)> {
    vec![
        (box_.left(), box_.top()),
        (box_.left() + top_w, box_.top()),
        (box_.left() + bottom_w, box_.bottom()),
        (box_.left(), box_.bottom()),
    ]
}

fn tri_br(box_: Rect, size: f32) -> Vec<(f32, f32)> {
    vec![
        (box_.right() - size, box_.bottom()),
        (box_.right(), box_.bottom()),
        (box_.right(), box_.bottom() - size),
    ]
}

#[derive(Debug, Clone, Copy)]
struct Tone {
    bg: Color,
    ink: Color,
    tri: Color,
}

fn tone_of(difficulty: Difficulty) -> Tone {
    match difficulty {
        Difficulty::Past => Tone {
            bg: Color::new(0x00, 0xB0, 0xF0, 0xFF),
            ink: CG_WHITE,
            tri: Color::new(0x00, 0xB0, 0xF0, 0xFF),
        },
        Difficulty::Present => Tone {
            bg: Color::new(0xA9, 0xD1, 0x8E, 0xFF),
            ink: CG_WHITE,
            tri: Color::new(0xA9, 0xD1, 0x8E, 0xFF),
        },
        Difficulty::Future => Tone {
            bg: Color::new(0xB8, 0x54, 0x97, 0xFF),
            ink: CG_WHITE,
            tri: Color::new(0xB8, 0x54, 0x97, 0xFF),
        },
        Difficulty::Eternal => Tone {
            bg: Color::new(0x64, 0x74, 0x8B, 0xFF),
            ink: CG_WHITE,
            tri: Color::new(0x64, 0x74, 0x8B, 0xFF),
        },
        Difficulty::Beyond => Tone {
            bg: Color::new(0xC0, 0x00, 0x00, 0xFF),
            ink: CG_WHITE,
            tri: Color::new(0xC0, 0x00, 0x00, 0xFF),
        },
        Difficulty::Ins => Tone {
            bg: Color::new(0x20, 0x30, 0x70, 0xFF),
            ink: CG_WHITE,
            tri: Color::new(0x20, 0x30, 0x70, 0xFF),
        },
    }
}

fn table_chips(kit: &CanvasKit<'_>, origin_x: f32, origin_y: f32, texts: &[(String, bool)]) {
    let mut x = CG_CHIP_X0 - origin_x;
    let y = CG_CHIP_TOP - origin_y;
    for (text, dark) in texts {
        let (bg, ink) = if *dark {
            (CG_INK, CG_WHITE)
        } else {
            (CG_BG, CG_INK)
        };
        let style = font_display(CG_CHIP_SIZE, ink);
        let width =
            sketch::text_width(kit, &style, text) + 2.0 * CG_CHIP_PAD_X + CG_CHIP_H * CG_CHIP_LEAN;
        let box_ = area(x, y, width, CG_CHIP_H);
        sketch::fill_polygon(kit, &paragraphs(box_, CG_CHIP_LEAN), bg);
        sketch::text(
            kit,
            &style,
            text,
            x + CG_CHIP_PAD_X + CG_CHIP_H * CG_CHIP_LEAN * 0.5,
            y + CG_CHIP_H / 2.0 + CG_CHIP_SIZE * 0.36,
            ink,
        );
        x += width + CG_CHIP_GAP;
    }
}

fn cg_header(kit: &CanvasKit<'_>, chips: &[(String, bool)]) {
    let box_ = area(CG_HEAD_X0, CG_HEAD_T, CG_RIGHT - CG_HEAD_X0, CG_TALL);
    let slant = CG_MARK_SLANT;
    let chamf = CG_HEAD_CHAMF;
    let shadow = area(
        box_.left() + CG_HEAD_SHADOW,
        box_.top() + CG_HEAD_SHADOW,
        box_.width(),
        box_.height(),
    );
    sketch::fill_polygon(kit, &cg_head_plate(shadow, slant, chamf), CG_SHADOW);
    let plate = cg_head_plate(box_, slant, chamf);
    sketch::clip_round(kit, box_, 0.0, || {
        sketch::fill_polygon(kit, &plate, CG_WHITE);
        sketch::fill_polygon(
            kit,
            &cg_head_bar(
                area(box_.left(), box_.top(), box_.width(), box_.height()),
                slant,
                slant,
                CG_MARK_W,
            ),
            CG_INK,
        );
        sketch::fill_polygon(
            kit,
            &cg_head_bar(
                area(box_.left(), box_.top(), box_.width(), box_.height()),
                slant,
                slant + CG_MARK_W,
                CG_MARK_THIN,
            ),
            CG_SKY,
        );
        sketch::text(
            kit,
            &font_display(CG_TITLE_SIZE, CG_INK),
            TABLE_TITLE_TEXT,
            CG_TITLE_X,
            CG_TITLE_BASE,
            CG_INK,
        );
        table_chips(kit, 0.0, 0.0, chips);
        cg_logo(kit);
    });
}

fn cg_logo(kit: &CanvasKit<'_>) {
    let key = "logo/arcaea_logo.png";
    let Ok(texture) = kit.asset(key) else {
        return;
    };
    let Some((iw, ih)) = texture.dimensions() else {
        return;
    };
    if iw <= 0 || ih <= 0 {
        return;
    }
    let Some(image) = texture.image() else {
        return;
    };
    let k = CG_LOGO_H / CG_LOGO_MARK_H;
    let ink_w = CG_LOGO_MARK_W * k;
    let x = CG_LOGO_RIGHT - ink_w;
    let ox = x - CG_LOGO_MARK_X * k;
    let oy = CG_LOGO_TOP - CG_LOGO_MARK_Y * k;
    let canvas = kit.canvas();
    canvas.save();
    canvas.clip_rect(area(x, CG_LOGO_TOP, ink_w, CG_LOGO_H), ClipOp::Intersect, true);
    let _ = kit.image_rect(
        image,
        area(ox, oy, iw as f32 * k, ih as f32 * k),
    );
    canvas.restore();
}

fn rail_k(k: f32) -> f32 {
    if k <= 0.0 {
        1.0
    } else {
        k
    }
}

fn cg_rail_geometry(rows: usize, height: f32, k: f32) -> (f32, f32, f32, f32, f32, f32, bool) {
    if rows <= 1 {
        let top_l = height * CG_RAIL_SINGLE_L;
        let top_r = height * CG_RAIL_SINGLE_R;
        (
            top_l,
            top_r,
            top_l + CG_RAIL_SEAM * k,
            top_r + CG_RAIL_SEAM * k,
            CG_RAIL_LOW_CHAM1 * k,
            CG_RAIL_WATER_SZ1 * k,
            false,
        )
    } else {
        (
            CG_RAIL_TOP_L * k,
            CG_RAIL_TOP_R * k,
            (CG_RAIL_TOP_L + CG_RAIL_SEAM) * k,
            (CG_RAIL_TOP_R + CG_RAIL_SEAM) * k,
            CG_RAIL_LOW_CHAM * k,
            CG_RAIL_WATER_SZ * k,
            true,
        )
    }
}

fn cg_rail(
    kit: &CanvasKit<'_>,
    band: &TableBand,
    label_top: &str,
    label_bot: &str,
    merged: bool,
    scale: f32,
) {
    let k = rail_k(scale);
    let height = band.bottom - band.top;
    if height <= 0.0 {
        return;
    }
    let (top_l, top_r, low_l, low_r, cham, water_sz, slashes) =
        cg_rail_geometry(band.rows, height, k);
    let cham_tl = CG_RAIL_CHAM_TL * k;
    let level_size = CG_LEVEL_SIZE * k;

    let x0 = band.rail.left();
    let x1 = band.rail.right();
    sketch::fill_polygon(
        kit,
        &[
            (x0 + cham_tl, band.top),
            (x1, band.top),
            (x1, band.top + top_r),
            (x0, band.top + top_l),
            (x0, band.top + cham_tl),
        ],
        CG_WHITE,
    );
    let low = vec![
        (x0, band.top + low_l),
        (x1, band.top + low_r),
        (x1, band.bottom - cham),
        (x1 - cham, band.bottom),
        (x0, band.bottom),
    ];
    sketch::fill_polygon(kit, &low, CG_RAIL_LOW);
    if merged {
        sketch::text_center(
            kit,
            &font_display(CG_RAIL_MERGE_SIZE, CG_INK),
            label_top,
            band.rail.left() + band.rail.width() / 2.0,
            band.top + CG_RAIL_MERGE_TOP,
            CG_INK,
        );
        sketch::text_center(
            kit,
            &font_display(CG_RAIL_MERGE_SIZE, CG_INK),
            label_bot,
            band.rail.left() + band.rail.width() / 2.0,
            band.top + CG_RAIL_MERGE_BOT,
            CG_INK,
        );
        kit.fill_rect(
            area(
                band.rail.left() + 20.0 * K,
                band.top + CG_RAIL_MERGE_SEAM,
                band.rail.width() - 40.0 * K,
                2.0 * K,
            ),
            CG_DIVIDER,
        );
    } else {
        let mid = (top_l + top_r) / 4.0;
        sketch::text_center(
            kit,
            &font_display(level_size, CG_INK),
            &format!("{:.1}", label_top.parse::<f32>().unwrap_or(0.0)),
            band.rail.left() + band.rail.width() / 2.0,
            band.top + mid + level_size * CG_LEVEL_INK_RISE,
            CG_INK,
        );
    }
    let frac = match label_bot.rsplit_once('.') {
        Some((_, frac)) => format!(".{frac}"),
        None => label_bot.to_string(),
    };
    let water_ft = if merged {
        CG_RAIL_WATER_FT * k
    } else {
        CG_RAIL_WATER_FT1 * k
    };
    clip_polygon(kit, &low, || {
        sketch::text_right(
            kit,
            &font_display(water_sz, CG_RAIL_WATER),
            &frac,
            x1 - 6.0 * K,
            band.bottom - water_ft,
            CG_RAIL_WATER,
        );
    });
    if slashes {
        let low_h = height - low_l;
        if low_h >= 30.0 * K {
            let count = (low_h / (320.0 * K)) as i32 + 2;
            let rail_w = band.rail.width();
            clip_polygon(kit, &low, || {
                for index in 1..=count {
                    let frac = index as f32 / (count + 1) as f32;
                    let anchor = band.top + low_l + low_h * frac;
                    sketch::fill_polygon(
                        kit,
                        &[
                            (x0, anchor),
                            (x0 + 2.0 * K, anchor),
                            (x0 + 2.0 * K + rail_w, anchor + rail_w),
                            (x0 + rail_w, anchor + rail_w),
                        ],
                        CG_WHITE,
                    );
                }
            });
        }
    }
}

fn cg_card_tag(kit: &CanvasKit<'_>, card: Rect, text: &str, tone: Tone, k: f32) {
    if text.is_empty() {
        return;
    }
    let size = CG_TAG_SIZE * k;
    let style = font_display(size, tone.ink);
    let tw = sketch::text_width(kit, &style, text);
    let bottom = tw + 2.0 * CG_TAG_PAD_X * k;
    let top = bottom + CG_TAG_SLANT * k;
    let height = CG_TAG_H * k;
    let box_ = area(card.left(), card.top(), bottom, height);
    sketch::fill_polygon(kit, &tag_shape(box_, top, bottom), tone.bg);
    sketch::text(
        kit,
        &style,
        text,
        card.left() + CG_TAG_PAD_X * k,
        card.top() + height / 2.0 + size * 0.36,
        tone.ink,
    );
}

fn cover_of(
    kit: &CanvasKit<'_>,
    song_id: &str,
    difficulty: Option<Difficulty>,
    fallback: &str,
    cover: Rect,
) -> bool {
    if !song_id.is_empty() {
        if let Some(difficulty) = difficulty {
            let key = format!(
                "jackets/{song_id}_{}.jpg",
                difficulty.as_str().to_lowercase()
            );
            if kit.draw_asset_cover(&key, cover, 0.0).is_ok() {
                return true;
            }
        }
    }
    kit.draw_asset_cover(fallback, cover, 0.0).is_ok()
}

fn cg_card(kit: &CanvasKit<'_>, card: Rect, chart: &TableChart) {
    let tone = tone_of(chart.difficulty);
    let shadow = area(
        card.left() + CG_SHADOW_X,
        card.top() + CG_SHADOW_Y,
        card.width(),
        card.height(),
    );
    sketch::fill_polygon(
        kit,
        &cg_chamfer(shadow, 0.0, CG_CHAM_TR, 0.0, CG_CHAM_BL),
        CG_SHADOW,
    );
    sketch::fill_polygon(
        kit,
        &cg_chamfer(card, 0.0, CG_CHAM_TR, 0.0, CG_CHAM_BL),
        CG_WHITE,
    );
    let cover = area(card.left(), card.top(), CG_CARD_W, CG_COVER_H);
    let cover_path = cg_chamfer(cover, 0.0, CG_CHAM_TR, 0.0, 0.0);
    clip_polygon(kit, &cover_path, || {
        if !cover_of(
            kit,
            &chart.song_id,
            Some(chart.difficulty),
            &chart.jacket_key(),
            cover,
        ) {
            sketch::fill_polygon(kit, &cover_path, CG_FACE_A);
        }
    });
    let plate = area(card.left(), card.top() + CG_COVER_H, CG_CARD_W, CG_PLATE_H);
    sketch::fill_polygon(kit, &cg_chamfer(plate, 0.0, 0.0, 0.0, CG_CHAM_BL), CG_WHITE);
    let tag_text = if chart.has_constant && chart.constant > 0.0 {
        format!("{} {:.1}", chart.difficulty.as_str(), chart.constant)
    } else {
        chart.difficulty.as_str().to_string()
    };
    cg_card_tag(kit, card, &tag_text, tone, 1.0);
    sketch::fill_polygon(kit, &tri_br(card, CG_TRI_SZ), tone.tri);
    sketch::text_center(
        kit,
        &font_bold(CG_CARD_TITLE_SIZE, CG_INK),
        &sketch::ellipsize(
            kit,
            &font_bold(CG_CARD_TITLE_SIZE, CG_INK),
            &chart.title,
            CG_CARD_TITLE_W,
        ),
        card.left() + CG_CARD_W / 2.0,
        card.top() + CG_COVER_H + CG_PLATE_H / 2.0 + CG_CARD_TITLE_SIZE * 0.36,
        CG_INK,
    );
}

fn cg_locked(kit: &CanvasKit<'_>, box_: Rect) {
    let path = cg_chamfer(box_, 0.0, CG_CHAM_TR, 0.0, CG_CHAM_BL);
    clip_polygon(kit, &path, || {
        sketch::fill_polygon(kit, &path, SLOT_BASE);
        hatch(kit, box_, CG_LOCK_STEP, CG_LOCK_LINE, SLOT_LINE);
        let steps = 9;
        for index in 0..steps {
            let d = CG_LOCK_SHADE * index as f32 / steps as f32;
            let alpha = (28.0 * (1.0 - d / CG_LOCK_SHADE)).round() as u8;
            kit.fill_rect(
                area(box_.left() + d, box_.top() + d, box_.width() - d * 2.0, 1.0),
                with_alpha(CG_LOCK_SHADE_INK, alpha),
            );
            kit.fill_rect(
                area(
                    box_.left() + d,
                    box_.top() + d,
                    1.0,
                    box_.height() - d * 2.0,
                ),
                with_alpha(CG_LOCK_SHADE_INK, alpha),
            );
        }
        kit.fill_rect(
            area(
                box_.left() + CG_CHAM_BL,
                box_.bottom() - CG_LOCK_HI,
                box_.width() - CG_CHAM_BL,
                CG_LOCK_HI,
            ),
            SLOT_HI,
        );
        kit.fill_rect(
            area(
                box_.right() - CG_LOCK_HI,
                box_.top() + CG_CHAM_TR,
                CG_LOCK_HI,
                box_.height() - CG_CHAM_TR,
            ),
            SLOT_HI,
        );
    });
}

pub fn paint_table(kit: &CanvasKit<'_>, input: &TableInput) {
    if input.mode == TableMode::Songs {
        paint_table_songs(kit, input);
        return;
    }
    paint_table_charts(kit, input);
}

fn cond_chips(cond: &str, fallback: &str) -> Vec<(String, bool)> {
    let mut out: Vec<(String, bool)> = cond
        .split('\n')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| (part.to_string(), false))
        .collect();
    if out.is_empty() {
        out.push((fallback.to_string(), true));
    } else if let Some(first) = out.first_mut() {
        first.1 = true;
    }
    out
}

fn paint_table_charts(kit: &CanvasKit<'_>, input: &TableInput) {
    let (layout, ins) = table_bands(input);
    let width = TABLE_WIDTH as f32;
    let height = table_size(input).1 as f32;
    kit.fill_rect(area(0.0, 0.0, width, height), CG_BG);
    let mut chips = cond_chips(&input.subtitle, "ALL CHARTS");
    let charts: usize = input.sheets.iter().map(|sheet| sheet.charts.len()).sum();
    if charts > 0 {
        chips.push((format!("MATCHED  {charts}"), false));
    }
    if input.library > 0 {
        chips.push((format!("TOTAL  {}", input.library), false));
    }
    cg_header(kit, &chips);
    let groups = table_groups(input);
    let mut flat: Vec<&TableChart> = Vec::new();
    for (_, charts) in &groups {
        flat.extend(charts.iter().copied());
    }
    for band in &layout.bands {
        let from = band.seq.min(flat.len());
        let merged = band.top_label != band.bot_label;
        cg_rail(kit, band, &band.top_label, &band.bot_label, merged, 1.0);
        for (index, card) in band.cards.iter().enumerate() {
            if let Some(chart) = flat.get(from + index) {
                cg_card(kit, *card, chart);
            }
        }
        for slot in &band.slots {
            cg_locked(kit, *slot);
        }
    }
    let foot_mid = layout.foot_y + CG_FOOT_H - 6.0 * K;
    let mut left = String::from(TABLE_FOOTER_CHARTS);
    if ins.is_empty() {
        left.push_str("   ·   EMPTY");
    }
    sketch::text(
        kit,
        &font_text(CG_FOOT_SIZE, CG_FOOTER_INK),
        &left,
        CG_LEFT,
        foot_mid,
        CG_FOOTER_INK,
    );
    sketch::text_right(
        kit,
        &font_text(CG_FOOT_SIZE, CG_FOOTER_INK),
        FOOT_SIGN.to_uppercase().as_str(),
        CG_RIGHT,
        foot_mid,
        CG_FOOTER_INK,
    );
}

fn cg_tile(kit: &CanvasKit<'_>, card: Rect, song: &TableSong) {
    let cover = area(card.left(), card.top(), CG_TILE_CARD_W, CG_TILE_CARD_W);
    let shadow = area(
        card.left() + CG_SHADOW_X,
        card.top() + CG_SHADOW_Y,
        card.width(),
        card.height(),
    );
    sketch::fill_polygon(
        kit,
        &cg_chamfer(shadow, 0.0, CG_CHAM_TR, 0.0, CG_CHAM_BL),
        CG_SHADOW,
    );
    let full = cg_chamfer(card, 0.0, CG_CHAM_TR, 0.0, CG_CHAM_BL);
    sketch::fill_polygon(kit, &full, CG_WHITE);
    clip_polygon(kit, &full, || {
        if !cover_of(
            kit,
            &song.song_id,
            song.top_difficulty(),
            &song.jacket_key(),
            cover,
        ) {
            sketch::fill_polygon(
                kit,
                &cg_chamfer(cover, 0.0, CG_CHAM_TR, 0.0, 0.0),
                CG_FACE_A,
            );
        }
    });
    let style = font_bold(CG_CARD_TITLE_SIZE, CG_INK);
    let max_w = CG_TILE_CARD_W - 2.0 * CG_TITLE_PAD;
    sketch::text_center(
        kit,
        &style,
        &sketch::ellipsize(kit, &style, &song.title, max_w),
        card.left() + CG_TILE_CARD_W / 2.0,
        card.top() + CG_TILE_CARD_W + CG_PLATE_H / 2.0 + CG_CARD_TITLE_SIZE * 0.36,
        CG_INK,
    );
    cg_strip(kit, card, &song.cells);
}

fn cg_strip(kit: &CanvasKit<'_>, card: Rect, cells: &[TableCell]) {
    let strip = area(
        card.left(),
        card.bottom() - CG_STRIP_H,
        CG_TILE_CARD_W,
        CG_STRIP_H,
    );
    let known: Vec<TableCell> = TABLE_CELL_ORDER
        .into_iter()
        .flat_map(|difficulty| {
            cells
                .iter()
                .copied()
                .filter(move |cell| cell.known && cell.difficulty == difficulty)
        })
        .collect();
    let count = known.len();
    if count == 0 {
        return;
    }
    let width = CG_TILE_CARD_W / count as f32;
    for (index, cell) in known.iter().enumerate() {
        let cell_box = area(
            strip.left() + index as f32 * width,
            strip.top(),
            width,
            strip.height(),
        );
        let tone = tone_of(cell.real);
        kit.fill_rect(
            area(
                cell_box.left(),
                cell_box.top(),
                cell_box.width(),
                CG_STRIP_TICK,
            ),
            tone.bg,
        );
        sketch::text_center(
            kit,
            &st(CG_STRIP_FONT, CG_STRIP_DIFF_SIZE, tone.bg),
            cell.real.as_str(),
            cell_box.left() + cell_box.width() / 2.0,
            cell_box.top() + CG_STRIP_DIFF_BASE,
            tone.bg,
        );
        sketch::text_center(
            kit,
            &st(CG_STRIP_FONT, CG_STRIP_CONST_SIZE, CG_INK),
            &format!("{:.1}", cell.constant),
            cell_box.left() + cell_box.width() / 2.0,
            cell_box.top() + CG_STRIP_CONST_BASE,
            CG_INK,
        );
    }
}

fn paint_table_songs(kit: &CanvasKit<'_>, input: &TableInput) {
    let layout = tile_layout(input.songs.len());
    let width = TABLE_WIDTH as f32;
    let height = table_size(input).1 as f32;
    kit.fill_rect(area(0.0, 0.0, width, height), CG_BG);
    let mut chips = cond_chips(&input.subtitle, "ALL CHARTS");
    let charts: usize = input
        .songs
        .iter()
        .map(|song| song.known_cells().len())
        .sum();
    if charts > 0 {
        chips.push((format!("MATCHED  {charts}"), false));
    }
    if input.library > 0 {
        chips.push((format!("TOTAL  {}", input.library), false));
    }
    cg_header(kit, &chips);
    for (index, card) in layout.cards.iter().enumerate() {
        if let Some(song) = input.songs.get(index) {
            cg_tile(kit, *card, song);
        }
    }
    for slot in &layout.slots {
        cg_locked(kit, *slot);
    }
    let foot_mid = layout.foot_y + CG_FOOT_H - 6.0 * K;
    let mut left = String::from(TABLE_FOOTER_SONGS);
    if input.songs.is_empty() {
        left.push_str("   ·   EMPTY");
    }
    sketch::text(
        kit,
        &font_text(CG_FOOT_SIZE, CG_FOOTER_INK),
        &left,
        CG_LEFT,
        foot_mid,
        CG_FOOTER_INK,
    );
    sketch::text_right(
        kit,
        &font_text(CG_FOOT_SIZE, CG_FOOTER_INK),
        FOOT_SIGN.to_uppercase().as_str(),
        CG_RIGHT,
        foot_mid,
        CG_FOOTER_INK,
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AhvBand {
    Pm,
    Fr,
    ExPlus,
    Ex,
    Other,
}

impl AhvBand {
    pub const ALL: [AhvBand; 5] = [
        AhvBand::Pm,
        AhvBand::Fr,
        AhvBand::ExPlus,
        AhvBand::Ex,
        AhvBand::Other,
    ];

    pub fn label(self) -> &'static str {
        match self {
            AhvBand::Pm => "PM 纯色",
            AhvBand::Fr => "FR 全连",
            AhvBand::ExPlus => "EX+ 990 万",
            AhvBand::Ex => "EX 980 万",
            AhvBand::Other => "其余达成",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            AhvBand::Pm => "PM",
            AhvBand::Fr => "FR",
            AhvBand::ExPlus => "EX+",
            AhvBand::Ex => "EX",
            AhvBand::Other => "OK",
        }
    }

    pub fn color(self, theme: Theme) -> Color {
        match self {
            AhvBand::Pm => theme::grade_color(Grade::ExPlus),
            AhvBand::Fr => theme::grade_color(Grade::A),
            AhvBand::ExPlus => theme.accent,
            AhvBand::Ex => theme::grade_color(Grade::B),
            AhvBand::Other => theme.muted,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AhvRecord {
    pub song_id: String,
    pub title: String,
    pub difficulty: Difficulty,
    pub constant: f64,
    pub has_constant: bool,
    pub score: i64,
    pub score_max: i64,
    pub far: i64,
    pub lost: i64,
    pub has_judgement: bool,
    pub clear_type: Option<i32>,
    pub grade: Option<Grade>,
    pub jacket: Option<String>,
    pub pure: Option<i64>,
    pub shiny: Option<i64>,
    pub rating: Option<f64>,
    pub loss: Option<f64>,
}

impl AhvRecord {
    pub fn new(
        song_id: impl Into<String>,
        title: impl Into<String>,
        difficulty: Difficulty,
        constant: f64,
    ) -> Self {
        Self {
            song_id: song_id.into(),
            title: title.into(),
            difficulty,
            constant,
            has_constant: constant > 0.0,
            score: 0,
            score_max: ptt::FULL_SCORE,
            far: 0,
            lost: 0,
            has_judgement: false,
            clear_type: None,
            jacket: None,
            pure: None,
            shiny: None,
            rating: None,
            loss: None,
            grade: None,
        }
    }

    pub fn with_score(mut self, score: i64) -> Self {
        self.score = score;
        self
    }

    pub fn with_judgement(mut self, far: i64, lost: i64) -> Self {
        self.far = far;
        self.lost = lost;
        self.has_judgement = true;
        self
    }

    pub fn with_clear(mut self, clear_type: i32) -> Self {
        self.clear_type = Some(clear_type);
        self
    }

    pub fn with_grade(mut self, grade: Grade) -> Self {
        self.grade = Some(grade);
        self
    }

    pub fn with_jacket(mut self, key: impl Into<String>) -> Self {
        let key = key.into();
        if !key.is_empty() {
            self.jacket = Some(key);
        }
        self
    }

    pub fn with_pure(mut self, pure: i64, shiny: i64) -> Self {
        self.pure = Some(pure.max(0));
        self.shiny = Some(shiny.max(0));
        self
    }

    pub fn with_rating(mut self, rating: f64) -> Self {
        self.rating = Some(rating);
        self
    }

    pub fn with_loss(mut self, loss: f64) -> Self {
        self.loss = Some(loss);
        self
    }

    pub fn jacket_key(&self) -> String {
        match self.jacket.as_deref() {
            Some(key) => key.to_string(),
            None => thumb_key(&self.song_id),
        }
    }

    pub fn is_pm(&self) -> bool {
        match self.clear_type {
            Some(code) => code == 3,
            None => self.has_judgement && self.far == 0 && self.lost == 0,
        }
    }

    pub fn is_fr(&self) -> bool {
        match self.clear_type {
            Some(code) => code == 3 || code == 2,
            None => self.has_judgement && self.lost == 0,
        }
    }

    pub fn band(&self) -> AhvBand {
        if self.is_pm() {
            return AhvBand::Pm;
        }
        if self.is_fr() {
            return AhvBand::Fr;
        }
        if self.score >= ptt::EX_PLUS_SCORE {
            return AhvBand::ExPlus;
        }
        if self.score >= ptt::EX_SCORE {
            return AhvBand::Ex;
        }
        AhvBand::Other
    }

    pub fn rating(&self) -> f64 {
        if self.score > 0 && self.has_constant {
            ptt::single_rating(self.constant, self.score)
        } else {
            0.0
        }
    }

    fn played(&self) -> bool {
        self.has_judgement || self.score > 0
    }

    fn pure_value(&self) -> i64 {
        self.pure.unwrap_or(0)
    }

    fn rating_value(&self) -> f64 {
        match self.rating {
            Some(rating) => rating,
            None => self.rating(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AhvInput {
    pub title: String,
    pub subtitle: String,
    pub theme: Theme,
    pub badge: PlayerBadge,
    pub records: Vec<AhvRecord>,
    pub note: String,
    pub generated_at: Option<String>,
}

impl AhvInput {
    pub fn new(badge: PlayerBadge) -> Self {
        Self {
            title: CardKind::Ahv.label().to_string(),
            subtitle: String::new(),
            theme: Theme::DEFAULT,
            badge,
            records: Vec::new(),
            note: String::new(),
            generated_at: None,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = subtitle.into();
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = note.into();
        self
    }

    pub fn with_record(mut self, record: AhvRecord) -> Self {
        self.records.push(record);
        self
    }

    pub fn with_records(mut self, records: Vec<AhvRecord>) -> Self {
        self.records = records;
        self
    }

    pub fn with_generated_at(mut self, stamp: impl Into<String>) -> Self {
        let stamp = stamp.into();
        if !stamp.trim().is_empty() {
            self.generated_at = Some(stamp);
        }
        self
    }

    pub fn count_of(&self, band: AhvBand) -> usize {
        self.records
            .iter()
            .filter(|record| record.band() == band)
            .count()
    }
}

impl Default for AhvInput {
    fn default() -> Self {
        Self::new(PlayerBadge::new("Afkeru")).with_subtitle("达成分组按 PM / FR / EX+ 排")
    }
}

fn ahv_label(constant: f64) -> String {
    let tenth = (constant * 10.0).round() as i64;
    format!("{}.{}", tenth.div_euclid(10), tenth.rem_euclid(10))
}

fn clear_rank(record: &AhvRecord) -> i32 {
    record.clear_type.unwrap_or(0)
}

fn ahv_record_order(left: &AhvRecord, right: &AhvRecord) -> std::cmp::Ordering {
    table_band_key(right.constant)
        .cmp(&table_band_key(left.constant))
        .then_with(|| right.played().cmp(&left.played()))
        .then_with(|| right.score.cmp(&left.score))
        .then_with(|| diff_display_weight(left.difficulty).cmp(&diff_display_weight(right.difficulty)))
        .then_with(|| clear_rank(right).cmp(&clear_rank(left)))
        .then_with(|| title_key(&left.title).cmp(&title_key(&right.title)))
        .then_with(|| left.song_id.cmp(&right.song_id))
}

fn ahv_groups(input: &AhvInput) -> Vec<(String, Vec<&AhvRecord>)> {
    let mut ordered: Vec<&AhvRecord> = input.records.iter().collect();
    ordered.sort_by(|left, right| ahv_record_order(left, right));
    let mut groups: Vec<(String, Vec<&AhvRecord>)> = Vec::new();
    for record in ordered {
        let label = ahv_label(record.constant);
        let same = matches!(groups.last(), Some((last, _)) if *last == label);
        if same {
            if let Some((_, records)) = groups.last_mut() {
                records.push(record);
            }
            continue;
        }
        groups.push((label, vec![record]));
    }
    groups
}

fn ahv_rows(count: usize) -> usize {
    count.div_ceil(AHV_COLS)
}

fn ahv_section_h(count: usize) -> f32 {
    let rows = ahv_rows(count);
    if rows == 0 {
        return 0.0;
    }
    rows as f32 * AHV_CARD_H + (rows as f32 - 1.0) * AHV_GAP_Y
}

#[derive(Debug, Clone, Default)]
struct AhvBandPlan {
    label: String,
    rail: Rect,
    rows: usize,
    cards: Vec<Rect>,
    slots: Vec<Rect>,
}

#[derive(Debug, Clone, Default)]
struct AhvLayout {
    bands: Vec<AhvBandPlan>,
    foot_y: f32,
    height: f32,
}

fn ahv_layout(groups: &[(String, usize)]) -> AhvLayout {
    let mut layout = AhvLayout::default();
    if groups.is_empty() {
        layout.foot_y = AHV_CONTENT_Y + CG_FOOT_GAP + CG_FOOT_H;
        layout.height = layout.foot_y + CG_FOOT_PAD;
        return layout;
    }
    let mut y = AHV_CONTENT_Y;
    let last = groups.len();
    for (index, (label, count)) in groups.iter().enumerate() {
        if *count == 0 {
            continue;
        }
        let height = ahv_section_h(*count);
        let mut band = AhvBandPlan {
            label: label.clone(),
            rail: area(AHV_LEFT, y, AHV_RAIL_W, height),
            rows: ahv_rows(*count),
            ..AhvBandPlan::default()
        };
        for item in 0..*count {
            let col = item % AHV_COLS;
            let row = item / AHV_COLS;
            band.cards.push(area(
                AHV_COL_X0 + col as f32 * AHV_PITCH_X,
                y + row as f32 * AHV_PITCH_Y,
                AHV_CARD_W,
                AHV_CARD_H,
            ));
        }
        if *count % AHV_COLS != 0 {
            let last_top = y + (ahv_rows(*count) as f32 - 1.0) * AHV_PITCH_Y;
            for col in (*count % AHV_COLS)..AHV_COLS {
                band.slots.push(area(
                    AHV_COL_X0 + col as f32 * AHV_PITCH_X,
                    last_top,
                    AHV_CARD_W,
                    AHV_CARD_H,
                ));
            }
        }
        layout.bands.push(band);
        y += height;
        if index + 1 < last {
            y += AHV_BAND_GAP;
        }
    }
    layout.foot_y = y + AHV_FOOT_GAP + AHV_FOOT_H / 2.0;
    layout.height = y + AHV_FOOT_GAP + AHV_FOOT_H + AHV_FOOT_PAD;
    layout
}

pub fn ahv_size(input: &AhvInput) -> (i32, i32) {
    let groups = ahv_groups(input);
    let ins: Vec<(String, usize)> = groups
        .iter()
        .map(|(label, records)| (label.clone(), records.len()))
        .collect();
    let layout = ahv_layout(&ins);
    (AHV_WIDTH, clamp_height(layout.height))
}

pub fn ahv_request(input: &AhvInput) -> RenderRequest {
    let (width, height) = ahv_size(input);
    request_for(width, height)
}

pub fn ahv_card(input: AhvInput) -> RenderCard {
    let request = ahv_request(&input);
    let label = format!(
        "{} {}x{}",
        CardKind::Ahv.label(),
        request.width,
        request.height
    );
    let cache_key = super::memo::fingerprint("ahv", &input);
    RenderCard::new(request, label, move |kit| paint_ahv(kit, &input)).with_cache_key(cache_key)
}

fn ahv_chips(kit: &CanvasKit<'_>, origin_x: f32, origin_y: f32, texts: &[(String, bool)]) {
    let mut x = AHV_CHIP_X0 - origin_x;
    let y = AHV_CHIP_TOP - origin_y;
    for (text, dark) in texts {
        let (bg, ink) = if *dark {
            (CG_INK, CG_WHITE)
        } else {
            (CG_BG, CG_INK)
        };
        let style = font_display(AHV_CHIP_SIZE, ink);
        let width = sketch::text_width(kit, &style, text)
            + 2.0 * AHV_CHIP_PAD_X
            + AHV_CHIP_H * CG_CHIP_LEAN;
        let box_ = area(x, y, width, AHV_CHIP_H);
        sketch::fill_polygon(kit, &paragraphs(box_, CG_CHIP_LEAN), bg);
        sketch::text(
            kit,
            &style,
            text,
            x + AHV_CHIP_PAD_X + AHV_CHIP_H * CG_CHIP_LEAN * 0.5,
            y + AHV_CHIP_H / 2.0 + AHV_CHIP_SIZE * 0.36,
            ink,
        );
        x += width + AHV_CHIP_GAP;
    }
}

fn ahv_header(kit: &CanvasKit<'_>, chips: &[(String, bool)], badge: &PlayerBadge) {
    let box_ = area(AHV_HEAD_X0, AHV_HEAD_T, AHV_RIGHT - AHV_HEAD_X0, AHV_TALL);
    let slant = AHV_MARK_SLANT;
    let chamf = AHV_HEAD_CHAMF;
    let shadow = area(
        box_.left() + AHV_HEAD_SHADOW,
        box_.top() + AHV_HEAD_SHADOW,
        box_.width(),
        box_.height(),
    );
    sketch::fill_polygon(kit, &cg_head_plate(shadow, slant, chamf), CG_SHADOW);
    let plate = cg_head_plate(box_, slant, chamf);
    sketch::clip_round(kit, box_, 0.0, || {
        sketch::fill_polygon(kit, &plate, CG_WHITE);
        sketch::fill_polygon(kit, &cg_head_bar(box_, slant, slant, AHV_MARK_W), CG_INK);
        sketch::fill_polygon(
            kit,
            &cg_head_bar(box_, slant, slant + AHV_MARK_W, AHV_MARK_THIN),
            CG_SKY,
        );
        sketch::text(
            kit,
            &font_display(AHV_HEAD_TITLE_SIZE, CG_INK),
            AHV_TITLE_TEXT,
            AHV_HEAD_TITLE_X,
            AHV_HEAD_TITLE_BASE,
            CG_INK,
        );
        ahv_chips(kit, 0.0, 0.0, chips);
        player_block(
            kit,
            badge,
            box_.left() + box_.width() - AHV_PLAYER_PAD_RIGHT,
            box_.top() + box_.height() / 2.0,
            AHV_PLAYER_SCALE,
        );
    });
}

fn ahv_judge_text(record: &AhvRecord) -> String {
    if !record.played() {
        return String::new();
    }
    if !record.has_judgement {
        return String::from("");
    }
    let mut pure = format!("P{}", record.pure_value());
    if let Some(shiny) = record.shiny {
        if shiny > 0 {
            pure.push_str(&format!("(-{})", record.pure_value() - shiny));
        }
    }
    format!("{pure} F{} L{}", record.far, record.lost)
}

fn ahv_grade_text(grade: Option<Grade>, band: AhvBand) -> String {
    match grade {
        Some(grade) => grade_label(grade).to_string(),
        None => band.short().to_string(),
    }
}

fn ahv_tab(kit: &CanvasKit<'_>, card: Rect, grade: Option<Grade>, band: AhvBand) {
    let x1 = card.left() + AHV_CARD_W;
    let y1 = card.top() + AHV_COVER_H;
    let top = y1 - AHV_TAB_H;
    sketch::fill_polygon(
        kit,
        &[
            (x1 - AHV_TAB_TOP_W, top),
            (x1, top),
            (x1, y1),
            (x1 - AHV_TAB_BOT_W, y1),
        ],
        CG_WHITE,
    );
    let cx = x1 - (AHV_TAB_TOP_W + AHV_TAB_BOT_W) / 4.0;
    sketch::text_center(
        kit,
        &font_text(AHV_TAB_SIZE, AHV_GRADE_INK),
        &ahv_grade_text(grade, band),
        cx,
        y1 - AHV_TAB_H / 2.0 + AHV_TAB_SIZE * 0.36,
        AHV_GRADE_INK,
    );
}

fn ahv_mist(kit: &CanvasKit<'_>, box_: Rect, key: &str) {
    if key.is_empty() || kit.draw_asset_cover(key, box_, 0.0).is_err() {
        sketch::fill_round(kit, box_, 0.0, CG_FACE_A);
        return;
    }
    sketch::clip_round(kit, box_, 0.0, || {
        let _ = kit.draw_asset_cover(key, box_, 0.0);
        kit.fill_rect(box_, with_alpha(AHV_PAPER_MIST, 0xC0));
    });
}

fn ahv_card_paint(
    kit: &CanvasKit<'_>,
    card: Rect,
    record: &AhvRecord,
    band: AhvBand,
    theme: Theme,
) {
    let played = record.played();
    let face = if played { CG_WHITE } else { AHV_UNLIT_FACE };
    let shadow = if played { CG_SHADOW } else { AHV_UNLIT_SHADOW };
    let shadow_box = area(
        card.left() + AHV_SHADOW_DX,
        card.top() + AHV_SHADOW_DY,
        card.width(),
        card.height(),
    );
    sketch::fill_polygon(
        kit,
        &cg_chamfer(shadow_box, 0.0, AHV_CHAM_TR, 0.0, AHV_CHAM_BL),
        shadow,
    );
    let card_path = cg_chamfer(card, 0.0, AHV_CHAM_TR, 0.0, AHV_CHAM_BL);
    sketch::fill_polygon(kit, &card_path, face);
    let cover = area(card.left(), card.top(), AHV_CARD_W, AHV_COVER_H);
    let cover_path = cg_chamfer(cover, 0.0, AHV_CHAM_TR, 0.0, 0.0);
    clip_polygon(kit, &cover_path, || {
        if played {
            if kit
                .draw_asset_cover(&record.jacket_key(), cover, 0.0)
                .is_err()
            {
                sketch::fill_polygon(kit, &cover_path, CG_FACE_A);
            }
        } else {
            ahv_mist(kit, cover, &record.jacket_key());
        }
    });
    let tone = if played {
        tone_of(record.difficulty)
    } else {
        Tone {
            bg: AHV_UNLIT_TAG,
            ink: CG_WHITE,
            tri: AHV_UNLIT_TAG,
        }
    };
    let tag = match record.loss {
        Some(loss) if loss > 0.0 => format!("LS {loss:.1}"),
        _ => record.difficulty.as_str().to_string(),
    };
    cg_card_tag(kit, card, &tag, tone, AHV_K);
    let title_ink = if played { CG_INK } else { AHV_UNLIT_TITLE };
    let title_style = font_text(AHV_TITLE_SIZE, title_ink);
    let title = sketch::ellipsize(kit, &title_style, &record.title, AHV_TITLE_MAX);
    sketch::text(
        kit,
        &title_style,
        &title,
        card.left() + AHV_INFO_PAD,
        card.top() + AHV_TITLE_BASE,
        title_ink,
    );
    let bar = area(
        card.left() + AHV_BAR_INSET,
        card.top() + AHV_ROW2_TOP,
        AHV_CARD_W - 2.0 * AHV_BAR_INSET,
        AHV_ROW2_H,
    );
    let bar_fill = if played {
        AHV_ROW2_FILL
    } else {
        AHV_ROW2_FILL_OFF
    };
    sketch::clip_round(kit, card, 0.0, || {
        kit.fill_rect(bar, bar_fill);
    });
    let pill = area(
        card.left() + AHV_CARD_W - AHV_PILL_RIGHT - AHV_PILL_W,
        card.top() + AHV_COVER_H + (AHV_ROW1_H - AHV_PILL_H) / 2.0,
        AHV_PILL_W,
        AHV_PILL_H,
    );
    if !played {
        sketch::text_center(
            kit,
            &font_text(AHV_SLOT_SIZE, AHV_UNLIT_TAG),
            "N O T   R E C O R D E D",
            card.left() + AHV_CARD_W / 2.0,
            card.top() + AHV_ROW2_TOP + AHV_ROW2_H / 2.0 + AHV_SLOT_SIZE * 0.36,
            AHV_UNLIT_TAG,
        );
        sketch::fill_round(kit, pill, AHV_PILL_R, AHV_ROW2_FILL_OFF);
        sketch::text_center(
            kit,
            &font_score(AHV_PILL_SIZE, AHV_UNLIT_TAG),
            "0.0000",
            pill.left() + pill.width() / 2.0,
            pill.top() + pill.height() / 2.0 + AHV_PILL_SIZE * 0.36,
            AHV_UNLIT_TAG,
        );
        sketch::fill_polygon(kit, &tri_br(card, AHV_FOLD), AHV_FOLD_INK);
        return;
    }
    ahv_tab(kit, card, record.grade, band);
    sketch::fill_round(kit, pill, AHV_PILL_R, AHV_PILL_FILL);
    let rating = record.rating_value();
    if rating > 0.0 {
        sketch::text_center(
            kit,
            &font_score(AHV_PILL_SIZE, AHV_GRADE_INK),
            &format!("{rating:.4}"),
            pill.left() + pill.width() / 2.0,
            pill.top() + pill.height() / 2.0 + AHV_PILL_SIZE * 0.36,
            AHV_GRADE_INK,
        );
    }
    sketch::text(
        kit,
        &font_score(AHV_SCORE_SIZE, AHV_INK_SCORE),
        &thousands(record.score),
        card.left() + AHV_INFO_PAD,
        card.top() + AHV_SCORE_BASE,
        AHV_INK_SCORE,
    );
    sketch::text_right(
        kit,
        &font_text(AHV_JUDGE_SIZE, AHV_INK_JUDGE),
        &ahv_judge_text(record),
        card.left() + AHV_CARD_W - AHV_INFO_PAD,
        card.top() + AHV_JUDGE_BASE,
        AHV_INK_JUDGE,
    );
    let _ = theme;
    sketch::fill_polygon(kit, &tri_br(card, AHV_FOLD), AHV_FOLD_INK);
}

pub fn paint_ahv(kit: &CanvasKit<'_>, input: &AhvInput) {
    let groups = ahv_groups(input);
    let ins: Vec<(String, usize)> = groups
        .iter()
        .map(|(label, records)| (label.clone(), records.len()))
        .collect();
    let layout = ahv_layout(&ins);
    let width = AHV_WIDTH as f32;
    let height = ahv_size(input).1 as f32;
    kit.fill_rect(area(0.0, 0.0, width, height), CG_BG);
    let matched = input.records.len();
    let played = input
        .records
        .iter()
        .filter(|record| record.played())
        .count();
    let loss: f64 = input.records.iter().filter_map(|record| record.loss).sum();
    let mut chips = cond_chips(&input.subtitle, "ALL");
    if matched > 0 {
        chips.push((format!("CHARTS  {matched}"), false));
        chips.push((format!("RECORDS  {played}"), false));
        chips.push((format!("TOTAL LOSS  {loss:.3}"), false));
    }
    ahv_header(kit, &chips, &input.badge);
    for (index, plan) in layout.bands.iter().enumerate() {
        let Some((label, records)) = groups.get(index) else {
            break;
        };
        let table_band = TableBand {
            top_label: plan.label.clone(),
            bot_label: plan.label.clone(),
            top: plan.rail.top(),
            bottom: plan.rail.bottom(),
            seq: 0,
            rows: plan.rows,
            rail: plan.rail,
            cards: plan.cards.clone(),
            slots: plan.slots.clone(),
        };
        cg_rail(kit, &table_band, label, label, false, AHV_K);
        for slot in &plan.slots {
            cg_locked(kit, area(slot.left(), slot.top(), AHV_CARD_W, AHV_CARD_H));
        }
        for (item, card) in plan.cards.iter().enumerate() {
            if let Some(record) = records.get(item) {
                ahv_card_paint(kit, *card, record, record.band(), input.theme);
            }
        }
    }
    let foot_mid = layout.foot_y + AHV_FOOT_H - 6.0 * K * AHV_K;
    let left = match input.generated_at.as_deref() {
        Some(stamp) => format!("Created At {stamp}"),
        None => String::from("Created At"),
    };
    sketch::text(
        kit,
        &font_text(AHV_FOOT_SIZE, CG_FOOTER_INK),
        &left,
        AHV_LEFT,
        foot_mid,
        CG_FOOTER_INK,
    );
    sketch::text_right(
        kit,
        &font_text(AHV_FOOT_SIZE, CG_FOOTER_INK),
        FOOT_SIGN.to_uppercase().as_str(),
        AHV_RIGHT,
        foot_mid,
        CG_FOOTER_INK,
    );
}

pub fn ahv_badge_rect(card: Rect) -> Rect {
    area(
        card.left(),
        card.top() + AHV_COVER_H - AHV_TAB_H,
        AHV_TAB_BOT_W,
        AHV_TAB_H,
    )
}

pub fn ahv_flag_rect(card: Rect) -> Rect {
    area(
        card.right() - AHV_FOLD,
        card.bottom() - AHV_FOLD,
        AHV_FOLD,
        AHV_FOLD,
    )
}

pub fn ahv_card_pos(index: usize, top: f32) -> Rect {
    let col = index % AHV_COLS;
    let row = index / AHV_COLS;
    area(
        AHV_COL_X0 + col as f32 * AHV_PITCH_X,
        top + row as f32 * AHV_PITCH_Y,
        AHV_CARD_W,
        AHV_CARD_H,
    )
}

pub fn table_card_pos(index: usize, top: f32) -> Rect {
    card_rect(top, 0, index)
}

pub fn table_group_height(count: usize) -> f32 {
    group_height(count)
}

pub fn ahv_body(input: &AhvInput) -> f32 {
    let groups = ahv_groups(input);
    let ins: Vec<(String, usize)> = groups
        .iter()
        .map(|(label, records)| (label.clone(), records.len()))
        .collect();
    ahv_layout(&ins).height
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::harness::{decode, image_size, request as 画布, Harness, Pixels};
    use crate::cards::Extent;

    fn case_112(pixels: &Pixels, area: Rect) -> (f64, f64, f64) {
        let (mut r, mut g, mut b, mut n) = (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64);
        let mut y = area.top().max(0.0) as u32;
        while y < (area.bottom() as u32).min(pixels.height) {
            let mut x = area.left().max(0.0) as u32;
            while x < (area.right() as u32).min(pixels.width) {
                let pixel = pixels.at(x, y);
                r += pixel[0] as f64;
                g += pixel[1] as f64;
                b += pixel[2] as f64;
                n += 1.0;
                x += 1;
            }
            y += 1;
        }
        if n == 0.0 {
            return (0.0, 0.0, 0.0);
        }
        (r / n, g / n, b / n)
    }

    fn case_113(a: (f64, f64, f64), b: (f64, f64, f64)) -> f64 {
        let dr = a.0 - b.0;
        let dg = a.1 - b.1;
        let db = a.2 - b.2;
        (dr * dr + dg * dg + db * db).sqrt()
    }

    fn case_114(input: ListInput) -> (Vec<u8>, i32, i32) {
        let request = request(&input).with_scale(1.0);
        let (width, height) = (request.width, request.height);
        let harness = Harness::new();
        let bytes = harness.render(request, |kit| paint(kit, &input));
        (bytes, width, height)
    }

    fn case_115(input: TableInput) -> (Vec<u8>, i32, i32) {
        let request = table_request(&input);
        let (width, height) = (request.width, request.height);
        let harness = Harness::new();
        let bytes = harness.render(request, |kit| paint_table(kit, &input));
        (bytes, width, height)
    }

    fn case_116(input: AhvInput) -> (Vec<u8>, i32, i32) {
        let request = ahv_request(&input);
        let (width, height) = (request.width, request.height);
        let harness = Harness::new();
        let bytes = harness.render(request, |kit| paint_ahv(kit, &input));
        (bytes, width, height)
    }

    fn case_117(index: usize) -> ListEntry {
        ListEntry::new(
            format!("song{index:02}"),
            format!("曲目 {index:02}"),
            Difficulty::ALL[index % 5],
            9.0 + index as f64 * 0.1,
        )
        .with_artist("测试曲师")
        .with_score(9_700_000 + index as i64 * 1000)
        .with_grade(Grade::Ex)
        .with_clear(1)
    }

    fn case_118(count: usize) -> ListInput {
        ListInput::new(PlayerBadge::new("Afkeru").with_ptt(12.34).with_star(5))
            .with_subtitle("按单曲 PTT 排")
            .with_entries((0..count).map(case_117).collect())
    }

    fn case_119(constant: f64, index: usize) -> TableChart {
        TableChart::new(
            format!("song{index:02}"),
            format!("定数曲 {index:02}"),
            Difficulty::Future,
            constant,
        )
        .with_score(9_500_000 + index as i64)
    }

    fn case_120(sheets: usize, per_sheet: usize) -> TableInput {
        let mut input = TableInput::new(PlayerBadge::new("Afkeru").with_ptt(12.34));
        for index in 0..sheets {
            let constant = 12.0 - index as f64 * 0.3;
            let band = table_band_label(constant);
            let charts: Vec<TableChart> = (0..per_sheet)
                .map(|item| case_119(constant, item + index))
                .collect();
            input = input.with_sheet(TableSheet::new(band).with_charts(charts));
        }
        input
    }

    fn case_121(index: usize) -> AhvRecord {
        let score = 9_600_000 + index as i64 * 10_000;
        AhvRecord::new(
            format!("song{index:02}"),
            format!("成就曲 {index:02}"),
            Difficulty::ALL[index % 5],
            9.0 + index as f64 * 0.2,
        )
        .with_score(score)
        .with_judgement((index % 3) as i64, ((index / 3) % 2) as i64)
    }

    fn case_122(count: usize) -> AhvInput {
        AhvInput::new(PlayerBadge::new("Afkeru").with_ptt(12.34))
            .with_subtitle("达成分组")
            .with_records((0..count).map(case_121).collect())
    }

    #[test]
    fn case_123() {
        let input = case_118(12);
        let (bytes, width, height) = case_114(input.clone());
        assert_eq!(width, LIST_WIDTH);
        assert_eq!(height, size_of(&input).1);
        assert_eq!(LIST_WIDTH, 1188);
        assert!(
            bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
            "渲染结果必须是 PNG"
        );
        assert_eq!(image_size(&bytes), Some((width as u32, height as u32)));
        assert!(CardKind::List.spec().height.accepts(height));
    }

    #[test]
    fn case_124() {
        let plan = plan_list_cards(12);
        assert_eq!(plan.cards.len(), 12);
        assert_eq!(plan.cards[0].x, LIST_PAD_L);
        assert_eq!(plan.cards[0].y, LIST_GRID_TOP);
        assert!((LIST_PAD_L - 45.0).abs() < 0.01);
        assert!((LIST_GRID_TOP - 126.0).abs() < 0.01);
        assert!((LIST_CARD_W - 210.0).abs() < 0.01);
        assert!((LIST_CARD_H - 98.0).abs() < 0.01);
        for index in 1..LIST_CARD_COLS {
            assert_eq!(plan.cards[index].y, plan.cards[0].y);
            assert!(plan.cards[index].x > plan.cards[index - 1].x);
        }
        let wrapped = plan.cards[LIST_CARD_COLS];
        assert!(wrapped.y > plan.cards[0].y);
        assert_eq!(wrapped.x, plan.cards[0].x);
        assert!(
            (plan.cards[LIST_CARD_COLS - 1].x + LIST_CARD_W - (LIST_RIGHT)).abs() < 0.01,
            "五列要铺满内容区"
        );
        assert_eq!(plan.slots.len(), 3, "十二张只剩两格，最后一行补三格空槽");
        let five = plan_list_cards(4);
        assert_eq!(five.slots.len(), 1);
        assert!(five.slots[0].x > five.cards[3].x);
    }

    #[test]
    fn case_125() {
        let empty = ListInput::default();
        let (_, width, height) = case_114(empty.clone());
        assert_eq!(width, LIST_WIDTH);
        assert_eq!(height, size_of(&empty).1);
        let plan = plan_list_cards(0);
        assert_eq!(plan.height, LIST_GRID_TOP + LIST_BOTTOM_PAD);
        assert!(size_of(&case_118(30)).1 > size_of(&case_118(1)).1);
        let pixels = decode(&case_114(empty).0).expect("解码空清单");
        assert!(!pixels.blank());
        let ink = pixels.ink_ratio(
            LIST_PAD_L as u32,
            (LIST_GRID_TOP - 14.0 * K) as u32,
            400,
            40,
            2,
        );
        assert!(ink > 0.01, "空清单要画没有命中的成绩，实际 {ink}");
    }

    #[test]
    fn case_126() {
        let mut 有 = case_150(Difficulty::Future, 3);
        有.entries[0] = 有.entries[0]
            .clone()
            .with_jacket("jacket_thumbs/acheron.jpg");
        let mut 无 = case_150(Difficulty::Future, 3);
        无.entries[0] = 无.entries[0]
            .clone()
            .with_jacket("jacket_thumbs/没有这张曲绘.jpg");
        let 有像素 = decode(&case_114(有).0).expect("解码清单卡");
        let 无像素 = decode(&case_114(无).0).expect("解码清单卡");
        let thumb = area(
            LIST_PAD_L + LIST_THUMB_X,
            LIST_GRID_TOP + LIST_THUMB_Y,
            LIST_THUMB_SIZE,
            LIST_THUMB_SIZE,
        );
        let ink = 有像素.ink_ratio(
            thumb.left() as u32,
            thumb.top() as u32,
            LIST_THUMB_SIZE as u32,
            LIST_THUMB_SIZE as u32,
            1,
        );
        assert!(ink > 0.05, "曲绘缩略图必须真的画上去，实际 {ink}");
        let 差 = case_113(case_112(&有像素, thumb), case_112(&无像素, thumb));
        assert!(差 > 20.0, "有曲绘与缺曲绘必须明显不同，实际 {差}");
    }

    #[test]
    fn case_127() {
        let (bytes, _, _) = case_114(case_150(Difficulty::Future, 3));
        let pixels = decode(&bytes).expect("解码清单卡");
        let white = area(
            LIST_PAD_L + 60.0 * K,
            LIST_GRID_TOP + 4.0 * K,
            60.0 * K,
            30.0 * K,
        );
        let (r, g, b) = case_112(&pixels, white);
        assert!(r > 200.0 && g > 200.0 && b > 200.0, "上半部要是纸白");
        let ice = area(
            LIST_PAD_L + 20.0 * K,
            LIST_GRID_TOP + 66.0 * K,
            60.0 * K,
            20.0 * K,
        );
        let (r, g, b) = case_112(&pixels, ice);
        assert!(b > r, "左下冷灰要偏蓝：({r}, {g}, {b})");
        let accent = area(
            LIST_PAD_L + 195.0 * K,
            LIST_GRID_TOP + 62.0 * K,
            12.0 * K,
            8.0 * K,
        );
        let (r, g, b) = case_112(&pixels, accent);
        assert!(b > r + 20.0, "右下冰川蓝斜切要偏蓝：({r}, {g}, {b})");
    }

    #[test]
    fn case_128() {
        let input = case_120(3, 4);
        let (bytes, width, height) = case_115(input.clone());
        assert_eq!(width, TABLE_WIDTH);
        assert_eq!(TABLE_WIDTH, 1409);
        assert_eq!(height, table_size(&input).1);
        assert_eq!(image_size(&bytes), Some((width as u32, height as u32)));
        assert!(CardKind::Table.spec().height.accepts(height));
        let first = table_card_pos(0, CG_CONTENT_Y);
        assert!((first.left() - CG_COL_X0).abs() < 0.01);
        assert!((CG_COL_X0 - 229.0).abs() < 0.01);
        assert!((CG_CARD_W - 206.0).abs() < 0.01);
        assert!((CG_CARD_H - 256.0).abs() < 0.01);
        assert!(
            (table_card_pos(CG_COLS - 1, CG_CONTENT_Y).right() - CG_RIGHT).abs() < 0.01,
            "五列要铺满右缘"
        );
        assert!(table_card_pos(CG_COLS, CG_CONTENT_Y).top() > first.top());
        assert!((table_group_height(1) - CG_CARD_H).abs() < 0.01);
        assert!((table_group_height(CG_COLS * 2) - (CG_CARD_H * 2.0 + CG_GAP_Y)).abs() < 0.01);
    }

    #[test]
    fn case_129() {
        let input = TableInput::new(PlayerBadge::new("Afkeru"))
            .with_sheet(
                TableSheet::new("12.0")
                    .with_chart(case_119(12.0, 0))
                    .with_chart(case_119(12.0, 1)),
            )
            .with_sheet(
                TableSheet::new("11.7")
                    .with_chart(case_119(11.7, 2))
                    .with_chart(case_119(11.7, 3)),
            );
        let (layout, ins) = table_bands(&input);
        assert_eq!(ins.len(), 2);
        assert_eq!(layout.bands.len(), 1, "四张卡不超过一行，合成一段");
        let band = &layout.bands[0];
        assert_eq!(band.top_label, "12.0");
        assert_eq!(band.bot_label, "11.7");
        assert_eq!(band.cards.len(), 4);
        assert_eq!(band.rows, 1);
        assert_eq!(band.cards.len(), 4);
        assert_eq!(band.slots.len(), 1, "最后一行右侧补一个空槽");
        assert_eq!(band.rail.height(), CG_CARD_H);
    }

    #[test]
    fn case_130() {
        let input = case_120(1, 7);
        let (layout, _) = table_bands(&input);
        assert_eq!(layout.bands.len(), 1);
        let band = &layout.bands[0];
        assert_eq!(band.rows, 2);
        assert_eq!(band.top_label, band.bot_label);
        assert!((band.rail.height() - (CG_CARD_H * 2.0 + CG_GAP_Y)).abs() < 0.01);
        assert_eq!(band.slots.len(), 3, "七张卡两行，第二行缺三个槽");
        assert!((band.rail.bottom() - (CG_CONTENT_Y + CG_CARD_H * 2.0 + CG_GAP_Y)).abs() < 0.01);
    }

    #[test]
    fn case_131() {
        let mut input = TableInput::new(PlayerBadge::new("Afkeru"));
        input = input.with_sheet(TableSheet::new("11.0"));
        input = input.with_sheet(TableSheet::new("10.0").with_chart(case_119(10.0, 0)));
        let groups = table_groups(&input);
        assert_eq!(groups.len(), 1, "空档不许占位");
        assert_eq!(groups[0].0, "10.0");
        let 空 = table_size(&TableInput::default());
        let 一 = table_size(&case_120(1, 1));
        let 三十 = table_size(&case_120(1, 30));
        assert!(空.1 <= 一.1);
        assert!(一.1 < 三十.1);
        for 高 in [空.1, 一.1, 三十.1] {
            assert!(CardKind::Table.spec().height.accepts(高));
        }
        let (bytes, _, _) = case_115(TableInput::default());
        let pixels = decode(&bytes).expect("解码空定数表");
        assert!(!pixels.blank());
    }

    #[test]
    fn case_132() {
        let (bytes, _, _) = case_115(case_120(2, 3));
        let pixels = decode(&bytes).expect("解码定数表");
        let mark = area(
            CG_HEAD_X0 + CG_MARK_SLANT + 3.0,
            CG_HEAD_T + 3.0,
            CG_MARK_W - 6.0,
            24.0,
        );
        let (r, g, b) = case_112(&pixels, mark);
        assert!(
            r < 120.0 && g < 120.0 && b < 140.0,
            "左缘要是深墨斜条：({r}, {g}, {b})"
        );
        let thin = area(
            CG_HEAD_X0 + CG_MARK_SLANT + CG_MARK_W + 2.0,
            CG_HEAD_T + 3.0,
            (CG_MARK_THIN - 4.0).max(1.0),
            24.0,
        );
        let (r, g, b) = case_112(&pixels, thin);
        assert!(b > r && b > 180.0, "细条是冰川蓝：({r}, {g}, {b})");
        let chip = area(
            CG_CHIP_X0 + CG_CHIP_H * CG_CHIP_LEAN + 6.0,
            CG_CHIP_TOP + CG_CHIP_H * 0.7,
            40.0,
            CG_CHIP_H * 0.25,
        );
        let (r, _, _) = case_112(&pixels, chip);
        assert!(r < 90.0, "第一枚数据条是深底：{r}");
        let logo = area(
            CG_LOGO_RIGHT - CG_LOGO_H * 1.6,
            CG_LOGO_TOP,
            CG_LOGO_H * 0.8,
            CG_LOGO_H * 0.8,
        );
        let ink = pixels.ink_ratio(
            logo.left() as u32,
            logo.top() as u32,
            logo.width() as u32,
            logo.height() as u32,
            1,
        );
        assert!(ink > 0.02, "页头右侧要贴 logo，实际墨迹 {ink}");
    }

    #[test]
    fn case_133() {
        let mut 有 = case_120(1, 2);
        有.sheets[0].charts[0] = 有.sheets[0].charts[0]
            .clone()
            .with_jacket("jacket_thumbs/acheron.jpg");
        let mut 无 = case_120(1, 2);
        无.sheets[0].charts[0] = 无.sheets[0].charts[0]
            .clone()
            .with_jacket("jacket_thumbs/没有这张曲绘.jpg");
        let 有像素 = decode(&case_115(有).0).expect("解码");
        let 无像素 = decode(&case_115(无).0).expect("解码");
        let card = table_card_pos(0, CG_CONTENT_Y);
        let cover = area(card.left(), card.top(), CG_CARD_W, CG_COVER_H);
        let 差 = case_113(case_112(&有像素, cover), case_112(&无像素, cover));
        assert!(差 > 20.0, "有曲绘与缺曲绘必须不同，实际 {差}");
        let tag = area(card.left() + 10.0, card.top() + 10.0, 30.0, 12.0);
        let (r, g, b) = case_112(&有像素, tag);
        assert!(r > g && b > g, "FTR 标签是洋红：({r}, {g}, {b})");
        let tri = area(
            card.right() - CG_TRI_SZ,
            card.bottom() - CG_TRI_SZ,
            CG_TRI_SZ,
            CG_TRI_SZ,
        );
        let (r, g, b) = case_112(&有像素, tri);
        assert!(r > g && b > g, "右下难度三角要 FTR 色：({r}, {g}, {b})");
    }

    #[test]
    fn case_134() {
        let (bytes, _, _) = case_115(case_120(1, 2));
        let pixels = decode(&bytes).expect("解码");
        let (layout, _) = table_bands(&case_120(1, 2));
        let slot = layout.bands[0].slots[0];
        let (r, g, b) = case_112(
            &pixels,
            area(
                slot.left() + 8.0,
                slot.top() + 8.0,
                CG_CARD_W - 16.0,
                CG_CARD_H - 16.0,
            ),
        );
        assert!(b > r && r > 180.0, "空槽是冰灰底：({r}, {g}, {b})");
        let mut darkest = 255u8;
        let mut y = slot.top() as u32 + 8;
        while y < (slot.top() + CG_CARD_H - 8.0) as u32 {
            let mut x = slot.left() as u32 + 8;
            while x < (slot.left() + CG_CARD_W - 8.0) as u32 {
                let pixel = pixels.at(x, y);
                darkest = darkest.min(pixel[0]).min(pixel[1]).min(pixel[2]);
                x += 3;
            }
            y += 3;
        }
        assert!(darkest > 180, "空槽里不许写字，最深像素 {darkest}");
    }

    #[test]
    fn case_135() {
        let input = case_122(12);
        let (bytes, width, height) = case_116(input.clone());
        assert_eq!(width, AHV_WIDTH);
        assert_eq!(AHV_WIDTH, 1600);
        assert_eq!(AHV_COLS, 6);
        assert_eq!(height, ahv_size(&input).1);
        assert_eq!(image_size(&bytes), Some((width as u32, height as u32)));
        assert!(CardKind::Ahv.spec().height.accepts(height));
        let first = ahv_card_pos(0, AHV_CONTENT_Y);
        assert!((first.left() - AHV_COL_X0).abs() < 0.01);
        assert!(ahv_card_pos(1, AHV_CONTENT_Y).left() > first.left());
        assert!(
            (ahv_card_pos(AHV_COLS - 1, AHV_CONTENT_Y).right() - AHV_RIGHT).abs() < 0.01,
            "六列要铺满右缘"
        );
        assert!(ahv_card_pos(AHV_COLS, AHV_CONTENT_Y).top() > first.top());
        assert!((AHV_CARD_H - AHV_COVER_H - AHV_ROW2_H - AHV_ROW1_H).abs() < 0.01);
    }

    #[test]
    fn case_136() {
        let input = AhvInput::new(PlayerBadge::new("Afkeru"))
            .with_record(
                AhvRecord::new("a", "A", Difficulty::Future, 10.0)
                    .with_score(10_000_000)
                    .with_judgement(0, 0),
            )
            .with_record(
                AhvRecord::new("b", "B", Difficulty::Future, 10.0)
                    .with_score(9_995_000)
                    .with_judgement(3, 0),
            )
            .with_record(
                AhvRecord::new("c", "C", Difficulty::Future, 10.0)
                    .with_score(9_950_000)
                    .with_judgement(1, 0),
            )
            .with_record(
                AhvRecord::new("d", "D", Difficulty::Future, 10.0)
                    .with_score(9_850_000)
                    .with_judgement(1, 1),
            )
            .with_record(
                AhvRecord::new("e", "E", Difficulty::Future, 9.8)
                    .with_score(9_000_000)
                    .with_judgement(1, 1),
            );
        assert_eq!(input.count_of(AhvBand::Pm), 1);
        assert_eq!(input.count_of(AhvBand::Fr), 2, "一粒 far 仍然是 FR");
        assert_eq!(input.count_of(AhvBand::Ex), 1, "有 lost 但 985 万仍算 EX");
        assert_eq!(input.count_of(AhvBand::Other), 1);
        let groups = ahv_groups(&input);
        assert_eq!(groups.len(), 2, "同一个定数只占一段");
        assert_eq!(groups[0].0, "10.0");
        assert_eq!(groups[1].0, "9.8");
        assert_eq!(groups[0].1.len(), 4);
        assert!(groups.iter().all(|(_, records)| !records.is_empty()));
        let (layout, ins) = {
            let ins: Vec<(String, usize)> = groups
                .iter()
                .map(|(label, records)| (label.clone(), records.len()))
                .collect();
            (ahv_layout(&ins), ins)
        };
        assert_eq!(layout.bands.len(), 2);
        assert_eq!(ins[0].0, "10.0");
        let second_top = layout.bands[1].rail.top();
        assert!(
            (second_top - (AHV_CONTENT_Y + ahv_section_h(4) + AHV_BAND_GAP)).abs() < 0.01,
            "段与段之间留 34K 的呼吸"
        );
    }

    #[test]
    fn case_137() {
        let record = AhvRecord::new("没有这张曲绘", "无名曲", Difficulty::Future, 10.0)
            .with_score(9_995_000)
            .with_judgement(2, 0)
            .with_clear(2)
            .with_pure(1200, 1100)
            .with_rating(12.34);
        assert_eq!(record.band(), AhvBand::Fr);
        assert_eq!(ahv_judge_text(&record), "P1200(-100) F2 L0");
        let input = AhvInput::new(PlayerBadge::new("Afkeru")).with_record(record);
        let (bytes, _, _) = case_116(input);
        let pixels = decode(&bytes).expect("解码成就表");
        let card = ahv_card_pos(0, AHV_CONTENT_Y);
        let tab = ahv_badge_rect(card);
        let (r, g, b) = case_112(
            &pixels,
            area(
                tab.left() + 6.0,
                tab.top() + 8.0,
                tab.width() - 18.0,
                tab.height() - 12.0,
            ),
        );
        assert!(
            r > 220.0 && g > 220.0 && b > 220.0,
            "评级折角是纯白：({r}, {g}, {b})"
        );
        let judge = area(
            card.left() + AHV_CARD_W - AHV_INFO_PAD - 130.0,
            card.top() + AHV_JUDGE_BASE - AHV_JUDGE_SIZE,
            130.0,
            AHV_JUDGE_SIZE + 6.0,
        );
        let ink = pixels.ink_ratio(
            judge.left() as u32,
            judge.top() as u32,
            judge.width() as u32,
            judge.height() as u32,
            1,
        );
        assert!(ink > 0.02, "判定那一行要有字，实际 {ink}");
        let blank = AhvRecord::new("x", "y", Difficulty::Future, 10.0);
        assert!(!blank.played());
    }

    #[test]
    fn case_138() {
        let record = AhvRecord::new("x", "y", Difficulty::Future, 10.0);
        assert!(!record.played());
        let input = AhvInput::new(PlayerBadge::new("Afkeru")).with_record(record);
        let (bytes, _, _) = case_116(input);
        let pixels = decode(&bytes).expect("解码");
        let card = ahv_card_pos(0, AHV_CONTENT_Y);
        let bar = area(
            card.left() + 40.0,
            card.top() + AHV_ROW2_TOP + 2.0,
            120.0,
            AHV_ROW2_H - 4.0,
        );
        let (r, g, b) = case_112(&pixels, bar);
        assert!(
            r > 200.0 && b >= r,
            "未记录那条底条更灰一档：({r}, {g}, {b})"
        );
    }

    #[test]
    fn case_139() {
        let base = AhvInput::new(PlayerBadge::new("Afkeru")).with_record(AhvRecord::new(
            "a",
            "A",
            Difficulty::Future,
            10.0,
        ));
        let stamped = base.clone().with_generated_at("2025-11-23 10:30:00");
        let 有 = decode(&case_116(stamped).0).expect("解码");
        let 无 = decode(&case_116(base).0).expect("解码");
        let foot = area(AHV_LEFT, 有.height as f32 - AHV_FOOT_H, 620.0, AHV_FOOT_H);
        let 差 = case_113(case_112(&有, foot), case_112(&无, foot));
        assert!(差 > 0.5, "左下要写出图时刻，实际差 {差}");
    }

    #[test]
    fn case_140() {
        let long = "这是一个非常非常长的曲名用来验证省略号截断不会溢出卡片边界而且还要更长一些";
        let mut input = case_118(2);
        input.entries[0] = input.entries[0]
            .clone()
            .with_jacket("jacket_thumbs/缺失.jpg");
        input.entries[0].title = long.to_string();
        input.entries[0].artist = long.to_string();
        let (bytes, _, _) = case_114(input);
        assert!(!decode(&bytes).expect("解码").blank());

        let mut table = case_120(1, 1);
        table.sheets[0].charts[0].title = long.to_string();
        table.sheets[0].charts[0] = table.sheets[0].charts[0]
            .clone()
            .with_jacket("jacket_thumbs/缺失.jpg");
        assert!(!decode(&case_115(table).0).expect("解码").blank());

        let mut ahv = case_122(2);
        ahv.records[0].title = long.to_string();
        assert!(!decode(&case_116(ahv).0).expect("解码").blank());
    }

    #[test]
    fn case_141() {
        let input = TableInput::new(PlayerBadge::new("Afkeru")).with_sheet(
            TableSheet::new("9.0").with_chart(case_119(9.0, 0)),
        );
        let (layout, _) = table_bands(&input);
        assert_eq!(layout.bands.len(), 1);
        assert_eq!(layout.bands[0].rows, 1);
        assert_eq!(layout.bands[0].slots.len(), 4);
        assert!(!decode(&case_115(input).0).expect("解码").blank());
        let single = AhvInput::new(PlayerBadge::new("Afkeru")).with_record(case_121(0));
        assert!(ahv_size(&single).1 >= MIN_HEIGHT);
        assert!(!decode(&case_116(single).0).expect("解码").blank());
    }

    #[test]
    fn case_142() {
        for (名字, (bytes, width, height)) in [
            ("清单", case_114(case_118(9))),
            ("定数表", case_115(case_120(2, 3))),
            ("成就表", case_116(case_122(9))),
        ] {
            let pixels = decode(&bytes).expect("解码");
            let ink = pixels.ink_ratio(0, 0, width as u32, height as u32, 3);
            assert!(ink > 0.02, "{名字}整图墨迹不足：{ink}");
        }
    }

    #[test]
    fn case_143() {
        for height in [
            size_of(&ListInput::default()).1,
            size_of(&case_118(30)).1,
            table_size(&TableInput::default()).1,
            ahv_size(&AhvInput::default()).1,
        ] {
            assert!(
                (MIN_HEIGHT..=MAX_HEIGHT).contains(&height),
                "高度越界：{height}"
            );
        }
        assert_eq!(CardKind::List.spec().height, Extent::dynamic(360, 12000));
    }
    #[test]
    fn case_144() {
        let harness = Harness::new();
        let (width, height) = (400, 300);
        let box_ = area(40.0, 40.0, 210.0, 98.0);
        let bytes = harness.render(画布(width, height), |kit| {
            kit.fill_rect(area(0.0, 0.0, width as f32, height as f32), CG_BG);
            cg_locked(kit, box_);
        });
        let pixels = decode(&bytes).expect("要能解码");
        let corner = pixels.at((box_.right() - 2.0) as u32, (box_.top() + 2.0) as u32);
        assert_eq!(
            corner,
            [0xEE, 0xF2, 0xF6, 0xFF],
            "切角外侧必须是底层底色，实际 {corner:?}"
        );
        let inside = pixels.at((box_.left() + 2.0) as u32, (box_.top() + 2.0) as u32);
        assert_ne!(
            inside,
            [0xEE, 0xF2, 0xF6, 0xFF],
            "切角内侧必须还是补位块底色"
        );
        for (dx, dy) in [(5.0, 5.0), (12.0, 5.0), (5.0, 12.0)] {
            let cut = pixels.at(
                (box_.right() - dx) as u32,
                (box_.top() + dy) as u32,
            );
            assert_eq!(
                cut,
                [0xEE, 0xF2, 0xF6, 0xFF],
                "切角三角区里 ({dx},{dy}) 处必须是底色，实际 {cut:?}"
            );
        }
        let near = pixels.at(
            (box_.right() - CG_CHAM_TR - 6.0) as u32,
            (box_.top() + 6.0) as u32,
        );
        assert_ne!(
            near,
            [0xEE, 0xF2, 0xF6, 0xFF],
            "斜边内侧必须还是补位块底色"
        );
    }

    #[test]
    fn case_145() {
        let records = vec![
            AhvRecord::new("z", "Zeta", Difficulty::Beyond, 10.7).with_score(3_000_000),
            AhvRecord::new("a", "Alpha", Difficulty::Eternal, 10.7)
                .with_score(9_900_000)
                .with_clear(3),
            AhvRecord::new("b1", "Beta", Difficulty::Future, 10.7).with_score(9_000_000),
            AhvRecord::new("b2", "Beta", Difficulty::Future, 10.7)
                .with_score(9_900_000)
                .with_clear(1),
            AhvRecord::new("p2", "aaa2", Difficulty::Past, 10.7),
            AhvRecord::new("p1", "AAA", Difficulty::Present, 10.7),
        ];
        let input = AhvInput::new(PlayerBadge::new("Afkeru")).with_records(records);
        let groups = ahv_groups(&input);
        assert_eq!(groups.len(), 1, "10.7 只能有一档");
        let order: Vec<&str> = groups[0].1.iter().map(|record| record.song_id.as_str()).collect();
        assert_eq!(
            order,
            vec!["a", "b2", "b1", "z", "p1", "p2"],
            "亮卡按分数降序（同分先看难度），暗卡垫在后面按难度排"
        );
        let scores: Vec<i64> = groups[0].1.iter().map(|record| record.score).collect();
        assert_eq!(scores, vec![9_900_000, 9_900_000, 9_000_000, 3_000_000, 0, 0]);
    }

    #[test]
    fn case_146() {
        let records = vec![
            AhvRecord::new("low", "Low", Difficulty::Future, 9.6).with_score(9_900_000),
            AhvRecord::new("high", "High", Difficulty::Past, 10.6).with_score(0),
        ];
        let input = AhvInput::new(PlayerBadge::new("Afkeru")).with_records(records);
        let groups = ahv_groups(&input);
        let labels: Vec<&str> = groups.iter().map(|(label, _)| label.as_str()).collect();
        assert_eq!(labels, vec!["10.6", "9.6"]);
    }

    #[test]
    fn case_147() {
        let sheet = TableSheet::new("10.7").with_charts(vec![
            TableChart::new("s1", "Zeta", Difficulty::Future, 10.7),
            TableChart::new("s2", "Borealis", Difficulty::Beyond, 10.6),
            TableChart::new("s3", "Beta", Difficulty::Eternal, 10.7),
            TableChart::new("s4", "delta", Difficulty::Future, 10.7),
        ]);
        let input = TableInput::new(PlayerBadge::new("Afkeru")).with_sheet(sheet);
        let groups = table_groups(&input);
        let order: Vec<&str> = groups[0].1.iter().map(|chart| chart.song_id.as_str()).collect();
        assert_eq!(
            order,
            vec!["s3", "s4", "s1", "s2"],
            "10.7 档里 ETR 在前，FTR 按曲名忽略大小写，10.6 档排后面"
        );
    }

    #[test]
    fn case_148() {
        let input = case_120(1, 3);
        let (layout, _) = table_bands(&input);
        let card = layout.bands[0].cards[0];
        let harness = Harness::new();
        let bytes = harness.render(table_request(&input), |kit| paint_table(kit, &input));
        let pixels = decode(&bytes).expect("要能解码");
        let corner = pixels.at((card.right() - 3.0) as u32, (card.top() + 3.0) as u32);
        assert_eq!(
            corner,
            [0xEE, 0xF2, 0xF6, 0xFF],
            "卡面切角外侧必须是底色，实际 {corner:?}"
        );
        let inside = pixels.at((card.left() + 3.0) as u32, (card.top() + 3.0) as u32);
        assert_ne!(inside, [0xEE, 0xF2, 0xF6, 0xFF], "卡面内部不能是底色");
    }

    #[test]
    fn case_149() {
        let input = case_122(4);
        let groups = ahv_groups(&input);
        let ins: Vec<(String, usize)> = groups
            .iter()
            .map(|(label, records)| (label.clone(), records.len()))
            .collect();
        let layout = ahv_layout(&ins);
        let card = layout.bands[0].cards[0];
        let harness = Harness::new();
        let bytes = harness.render(ahv_request(&input), |kit| paint_ahv(kit, &input));
        let pixels = decode(&bytes).expect("要能解码");
        let corner = pixels.at((card.right() - 3.0) as u32, (card.top() + 3.0) as u32);
        assert_eq!(
            corner,
            [0xEE, 0xF2, 0xF6, 0xFF],
            "卡面切角外侧必须是底色，实际 {corner:?}"
        );
    }

    fn case_150(difficulty: Difficulty, count: usize) -> ListInput {
        let entries: Vec<ListEntry> = (0..count)
            .map(|index| {
                ListEntry::new(
                    format!("song-{index:02}"),
                    format!("曲目 {index:02}"),
                    difficulty,
                    10.0 + index as f64 * 0.1,
                )
                .with_artist("测试曲师")
                .with_score(9_800_000 + index as i64 * 1000)
                .with_grade(Grade::Ex)
            })
            .collect();
        ListInput::new(PlayerBadge::new("Afkeru"))
            .with_subtitle("按单曲 PTT 排")
            .with_entries(entries)
    }

    fn case_151(id: &str, title: &str, cells: &[(Difficulty, f64)]) -> TableSong {
        let mut all: Vec<TableCell> = TABLE_CELL_ORDER
            .iter()
            .map(|difficulty| TableCell::unknown(*difficulty))
            .collect();
        for (difficulty, constant) in cells {
            if let Some(slot) = all.iter_mut().find(|cell| cell.difficulty == *difficulty) {
                *slot = TableCell::new(*difficulty, *constant);
            }
        }
        TableSong::new(id, title, all)
    }

    fn case_152(count: usize) -> TableInput {
        let songs: Vec<TableSong> = (0..count)
            .map(|index| {
                case_151(
                    &format!("song{index:02}"),
                    &format!("曲目 {index:02}"),
                    &[
                        (Difficulty::Past, 5.0),
                        (Difficulty::Present, 8.0 + index as f64 * 0.1),
                        (Difficulty::Future, 10.2),
                    ],
                )
            })
            .collect();
        TableInput::new(PlayerBadge::new("Afkeru"))
            .with_subtitle("定数表 · 全曲目")
            .with_mode(TableMode::Songs)
            .with_songs(songs)
    }

    #[test]
    fn case_153() {
        let 全档: Vec<ListEntry> = [
            Difficulty::Past,
            Difficulty::Present,
            Difficulty::Future,
            Difficulty::Eternal,
            Difficulty::Beyond,
        ]
        .into_iter()
        .map(|difficulty| ListEntry::new("song", "曲目", difficulty, 10.0))
        .collect();
        let bands = difficulty_bands(&全档);
        assert_eq!(
            bands,
            vec![
                Difficulty::Beyond,
                Difficulty::Eternal,
                Difficulty::Future,
                Difficulty::Present,
                Difficulty::Past,
            ],
            "分带从超越排到过去"
        );
        let weights: Vec<u8> = bands
            .iter()
            .map(|difficulty| diff_display_weight(*difficulty))
            .collect();
        let mut sorted = weights.clone();
        sorted.sort_unstable();
        assert_eq!(weights, sorted, "分带顺序必须就是 diff_display_weight 的次序");
    }

    #[test]
    fn case_154() {
        let input = case_150(Difficulty::Future, 14);
        let plan = input.plan();
        assert!(!plan.banded, "单一难度不切分带");
        assert!(plan.heads.is_empty(), "不分带就没有档头");
        assert_eq!(plan.cards.len(), 14);
        assert_eq!(plan.cards[0].x, LIST_PAD_L);
        assert_eq!(plan.cards[0].y, LIST_GRID_TOP);
        assert_eq!(plan.cards[0].entry, 0);
        assert_eq!(plan.slots.len(), 1);
        assert_eq!(size_of(&input), (LIST_WIDTH, 518), "08-list 夹具是 1188×518");
    }

    #[test]
    fn case_155() {
        let input = case_118(12);
        let plan = input.plan();
        assert!(plan.banded);
        let order: Vec<Difficulty> = plan.heads.iter().map(|head| head.difficulty).collect();
        assert_eq!(
            order,
            vec![
                Difficulty::Beyond,
                Difficulty::Eternal,
                Difficulty::Future,
                Difficulty::Present,
                Difficulty::Past,
            ]
        );
        assert_eq!(plan.heads.iter().map(|head| head.count).sum::<usize>(), 12);
        assert!(plan
            .heads
            .windows(2)
            .all(|pair| pair[0].box_.top() < pair[1].box_.top()));
        for head in &plan.heads {
            assert_eq!(head.box_.height(), LIST_BAND_H);
            assert_eq!(head.box_.width(), LIST_RIGHT - LIST_PAD_L, "档头要铺满内容区");
        }
        let (bytes, width, height) = case_114(input.clone());
        assert_eq!(width, LIST_WIDTH);
        assert_eq!(height, size_of(&input).1);
        assert!(CardKind::List.spec().height.accepts(height));
        let pixels = decode(&bytes).expect("要能解码");
        let head = plan.heads[0].box_;
        let ink = pixels.ink_ratio(
            head.left() as u32,
            head.top() as u32,
            head.width() as u32,
            head.height() as u32,
            2,
        );
        assert!(ink > 0.02, "档头上要写难度名与首数，实际 {ink}");
        let tone = tone_of(Difficulty::Beyond);
        let accent = pixels.at(
            (head.left() + 3.0) as u32,
            (head.top() + head.height() / 2.0) as u32,
        );
        assert_eq!(
            accent,
            [tone.bg.r, tone.bg.g, tone.bg.b, tone.bg.a],
            "档头左侧色条要用难度色"
        );
    }

    #[test]
    fn case_156() {
        let input = case_118(9);
        let plan = input.plan();
        for (index, head) in plan.heads.iter().enumerate() {
            let low = head.box_.bottom();
            let high = plan
                .heads
                .get(index + 1)
                .map_or(f32::MAX, |next| next.box_.top());
            let members: Vec<&CardPos> = plan
                .cards
                .iter()
                .filter(|card| card.y >= low && card.y < high)
                .collect();
            assert_eq!(members.len(), head.count, "{} 档的卡数要对得上", head.difficulty.as_str());
            assert!(
                members
                    .iter()
                    .all(|card| input.entries[card.entry].difficulty == head.difficulty),
                "{} 档下面不能混进别的难度",
                head.difficulty.as_str()
            );
        }
    }

    #[test]
    fn case_157() {
        let song = case_151(
            "x",
            "X",
            &[
                (Difficulty::Beyond, 10.9),
                (Difficulty::Past, 5.0),
                (Difficulty::Eternal, 10.5),
            ],
        );
        let cells = song.known_cells();
        assert_eq!(
            cells
                .iter()
                .map(|cell| cell.difficulty)
                .collect::<Vec<Difficulty>>(),
            vec![Difficulty::Past, Difficulty::Eternal, Difficulty::Beyond],
            "难度条按 PST→BYD 展示"
        );
        assert_eq!(song.top_difficulty(), Some(Difficulty::Beyond));
        assert_eq!(song.max_constant(), 10.9);
        assert_eq!(song.second_constant(), 10.5);
    }

    #[test]
    fn case_158() {
        let input = case_152(7);
        assert_eq!(input.mode, TableMode::Songs);
        assert!((CG_TILE_CARD_W - 241.0).abs() < 0.01, "卡宽 {CG_TILE_CARD_W}");
        assert!((CG_TILE_H - 325.0).abs() < 0.01, "卡高 {CG_TILE_H}");
        let layout = tile_layout(7);
        assert_eq!(layout.cards.len(), 7);
        assert_eq!(layout.cards[0].left(), CG_LEFT);
        assert_eq!(layout.cards[0].top(), CG_CONTENT_Y);
        assert_eq!(layout.cards[5].top(), CG_CONTENT_Y + CG_TILE_PITCH_Y);
        assert_eq!(layout.cards[5].left(), CG_LEFT);
        assert_eq!(layout.slots.len(), 3, "第二行只剩两格，补三格空槽");
        let (width, height) = table_size(&input);
        assert_eq!(width, TABLE_WIDTH);
        assert_eq!(height, layout.height.round() as i32);
        assert!(CardKind::Table.spec().height.accepts(height));
        let (bytes, _, _) = case_115(input.clone());
        assert_eq!(image_size(&bytes), Some((width as u32, height as u32)));
    }

    #[test]
    fn case_159() {
        let input = case_152(1);
        let card = tile_layout(1).cards[0];
        let strip = area(
            card.left(),
            card.bottom() - CG_STRIP_H,
            CG_TILE_CARD_W,
            CG_STRIP_H,
        );
        let (bytes, _, _) = case_115(input.clone());
        let pixels = decode(&bytes).expect("要能解码");
        let known = input.songs[0].known_cells();
        assert_eq!(known.len(), 3);
        let width = CG_TILE_CARD_W / known.len() as f32;
        for (index, cell) in known.iter().enumerate() {
            let tone = tone_of(cell.difficulty);
            let tick = pixels.at(
                (strip.left() + index as f32 * width + width / 2.0) as u32,
                (strip.top() + 1.0) as u32,
            );
            assert_eq!(
                tick,
                [tone.bg.r, tone.bg.g, tone.bg.b, tone.bg.a],
                "{} 那格的顶边色条要跟难度同色",
                cell.difficulty.as_str()
            );
            let ink = pixels.ink_ratio(
                (strip.left() + index as f32 * width) as u32,
                strip.top() as u32,
                width as u32,
                CG_STRIP_H as u32,
                2,
            );
            assert!(ink > 0.01, "{} 那格要写难度与定数，实际 {ink}", cell.difficulty.as_str());
        }
    }

    #[test]
    fn case_160() {
        let input = case_152(3);
        let card = tile_layout(3).cards[0];
        let (bytes, _, _) = case_115(input.clone());
        let pixels = decode(&bytes).expect("要能解码");
        let corner = pixels.at((card.right() - 3.0) as u32, (card.top() + 3.0) as u32);
        assert_eq!(
            corner,
            [0xEE, 0xF2, 0xF6, 0xFF],
            "曲目卡切角外侧必须是底色，实际 {corner:?}"
        );
        let inside = pixels.at((card.left() + 3.0) as u32, (card.top() + 3.0) as u32);
        assert_ne!(inside, [0xEE, 0xF2, 0xF6, 0xFF], "曲目卡内部不能是底色");
    }

    #[test]
    fn case_161() {
        assert_eq!(TABLE_FOOTER_SONGS, "VIEW  SONGS  //  SORTED BY MAX CONSTANT");
        assert!(
            TABLE_FOOTER_SONGS.contains("MAX CONSTANT"),
            "曲目视图已按最高定数排序，页脚不能还写 FTR"
        );
        assert!(!TABLE_FOOTER_SONGS.contains("FTR"), "页脚不该再提固定难度");
        assert_eq!(TABLE_FOOTER_CHARTS, "VIEW  CHARTS  //  BANDED BY CONSTANT");
    }

    #[test]
    fn case_162() {
        let songs = vec![
            case_151(
                "a",
                "Alpha",
                &[(Difficulty::Beyond, 10.0), (Difficulty::Future, 9.5)],
            ),
            case_151(
                "b",
                "Beta",
                &[(Difficulty::Beyond, 10.0), (Difficulty::Future, 9.8)],
            ),
            case_151("c", "Gamma", &[(Difficulty::Future, 10.0)]),
            case_151("d", "Delta", &[(Difficulty::Eternal, 10.0)]),
            case_151(
                "e",
                "alpha",
                &[(Difficulty::Beyond, 10.0), (Difficulty::Future, 9.8)],
            ),
        ];
        let sorted = table_songs_sorted(&songs);
        let ids: Vec<&str> = sorted
            .iter()
            .map(|song| song.song_id.as_str())
            .collect();
        assert_eq!(
            ids,
            vec!["e", "b", "a", "d", "c"],
            "先比最高定数，再比最高定数所属难度的权重，再比次高定数，最后比曲名忽略大小写"
        );
    }

    #[test]
    fn ahv_grade_text_uses_real_grade() {
        for (grade, want) in [
            (Grade::Pm, "PM"),
            (Grade::Fr, "FR"),
            (Grade::ExPlus, "EX+"),
            (Grade::Ex, "EX"),
            (Grade::Aa, "AA"),
            (Grade::A, "A"),
            (Grade::B, "B"),
            (Grade::C, "C"),
            (Grade::D, "D"),
        ] {
            assert_eq!(ahv_grade_text(Some(grade), AhvBand::Ex), want);
            assert_eq!(grade_label(grade), grade.as_str());
        }
        assert_eq!(ahv_grade_text(None, AhvBand::Pm), "PM");
        assert_eq!(ahv_grade_text(None, AhvBand::Fr), "FR");
        assert_eq!(ahv_grade_text(None, AhvBand::ExPlus), "EX+");
        assert_eq!(ahv_grade_text(None, AhvBand::Ex), "EX");
        assert_eq!(ahv_grade_text(None, AhvBand::Other), "OK");
    }
}
