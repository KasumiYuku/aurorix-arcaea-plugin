use aurorix_plugin::RenderCard;
use aurorix_render::skia_safe::{PathBuilder, Rect};
use aurorix_render::{
    CanvasKit, Color, Gradient, GradientStop, LinearGradient, RenderRequest, TextStyle,
};

use super::player;

use super::sketch;
use super::theme::{self, Theme};
use super::widgets::{self, PlayerBadge};
use super::jacket_pool;

use crate::cards::CardKind;
use crate::domain::suggest::{SuggestBoard, SuggestHint};
use crate::domain::ptt;
use crate::model::{Difficulty, Grade};

const K: f32 = 1.0;

pub const STATS_WIDTH: i32 = 1080;

pub const SUGGEST_WIDTH: i32 = 1400;

pub const CHAP_WIDTH: i32 = 1350;

pub const TREND_WIDTH: i32 = 1080;

pub const HIS_WIDTH: i32 = 1080;

pub const MIN_HEIGHT: i32 = 360;

pub const MAX_HEIGHT: i32 = 12000;

const LEDGER_W: f32 = 1440.0 * 0.75 * K;
const MARGIN_X: f32 = 72.0 * 0.75 * K;
const HEAD_TITLE_BASE: f32 = 210.0 * 0.75 * K;
const HEAD_NOTE_BASE: f32 = 252.0 * 0.75 * K;
const HEAD_TITLE_SIZE: f32 = 38.0 * 0.75 * K;
const HEAD_NOTE_SIZE: f32 = 20.0 * 0.75 * K;
const COMPACT_HEAD_TOP: f32 = 80.0 * 0.75 * K;
const COMPACT_GRID_GAP: f32 = 4.0 * 0.75 * K;
const PLATE_H: f32 = 54.0 * 0.75 * K;
const AVATAR_R: f32 = PLATE_H * 0.8;
const SECTION_TOP: f32 = 24.0 * 0.75 * K;
const SECTION_SIZE: f32 = 30.0 * 0.75 * K;
const SECTION_RULE_GAP: f32 = 10.0 * 0.75 * K;
const SECTION_GAP: f32 = 4.0 * 0.75 * K;
const RULE_H: f32 = 2.0 * 0.75 * K;
const FOOTER_H: f32 = 130.0 * 0.75 * K;
const HEAD_SIZE: f32 = 16.0 * 0.75 * K;
const SIZE_M: f32 = 24.0 * 0.75 * K;
const SIZE_S: f32 = 20.0 * 0.75 * K;
const SIZE_XS: f32 = 16.0 * 0.75 * K;
const COMP_BAR_GAP: f32 = 10.0 * 0.75 * K;
const PROFILE_COL_INSET_R: f32 = 16.0 * 0.75 * K;
const PROFILE_COL_GAP: f32 = 28.0 * 0.75 * K;
const PROFILE_AXIS_GUTTER: f32 = 64.0 * 0.75 * K;
const PROFILE_PLOT_TOP_PAD: f32 = 18.0 * 0.75 * K;

const STATS_BAND_ROW_H: f32 = 76.0 * 0.75 * K;
const STATS_BAND_GAP: f32 = 12.0 * 0.75 * K;
const STATS_BAND_BAR_H: f32 = 16.0 * 0.75 * K;
const STATS_BAND_NAME_W: f32 = 112.0 * 0.75 * K;
const STATS_CONST_PLOT_H: f32 = 210.0 * 0.75 * K;
const STATS_CONST_HEADROOM: f32 = 56.0 * 0.75 * K;
const STATS_CONST_LABEL_SIZE: f32 = SIZE_XS;
const STATS_CONST_VALUE_SIZE: f32 = SIZE_M;
const STATS_CONST_FOOT_ROOM: f32 = 12.0 * 0.75 * K + STATS_CONST_LABEL_SIZE + 10.0 * 0.75 * K;
const STATS_CONST_BAR_MAX_W: f32 = 120.0 * 0.75 * K;
const STATS_GRADE_ROW_H: f32 = 58.0 * 0.75 * K;
const STATS_GRADE_PAIR_W: f32 = 124.0 * 0.75 * K;
const STATS_GRADE_GAP: f32 = 6.0 * 0.75 * K;
const STATS_GRADE_VALUE_SIZE: f32 = 28.0 * 0.75 * K;
const STATS_SHORT_ROW_H: f32 = 64.0 * 0.75 * K;
const STATS_SHORT_THUMB: f32 = 44.0 * 0.75 * K;
const STATS_SHORT_RADIUS: f32 = 8.0 * 0.75 * K;
const STATS_SHORT_GAP: f32 = 18.0 * 0.75 * K;
const STATS_SHORT_NOTE_TOP: f32 = 36.0 * 0.75 * K;
const STATS_BAND_COLS: [f32; 5] = [150.0, 96.0, 96.0, 96.0, 130.0];
const STATS_SHORT_COLS: [f32; 5] = [140.0, 72.0, 170.0, 72.0, 72.0];

const PUSH_K: f32 = 1400.0 / 1120.0;
const PUSH_W: f32 = 1120.0 * PUSH_K * K;
const PUSH_PAD_L: f32 = 36.0 * PUSH_K * K;
const PUSH_PAD_R: f32 = 36.0 * PUSH_K * K;
const PUSH_RIGHT: f32 = PUSH_W - PUSH_PAD_R;
const PUSH_TITLE_BASE: f32 = 56.0 * K;
const PUSH_TITLE_SIZE: f32 = 32.0 * K;
const PUSH_SUB_BASE: f32 = 80.0 * K;
const PUSH_SUB_SIZE: f32 = 13.0 * K;
const PUSH_SUB_GAP: f32 = 10.0 * K;
const PUSH_GRID_TOP: f32 = 126.0 * PUSH_K * K;
const PUSH_CARD_W: f32 = 253.0 * PUSH_K * K;
const PUSH_CARD_H: f32 = 98.0 * PUSH_K * K;
const PUSH_CARD_GAP_X: f32 = 12.0 * PUSH_K * K;
const PUSH_CARD_GAP_Y: f32 = 12.0 * PUSH_K * K;
const PUSH_CARD_R: f32 = 8.0 * PUSH_K * K;
const PUSH_CARD_COLS: usize = 4;
const PUSH_CARD_STEP_X: f32 = PUSH_CARD_W + PUSH_CARD_GAP_X;
const PUSH_CARD_STEP_Y: f32 = PUSH_CARD_H + PUSH_CARD_GAP_Y;
const PUSH_LEVEL_BAR_X: f32 = PUSH_PAD_L;
const PUSH_LEVEL_BAR_W: f32 = 5.0 * PUSH_K * K;
const PUSH_LEVEL_BAR_H: f32 = 16.0 * PUSH_K * K;
const PUSH_LEVEL_LABEL_X: f32 = 52.0 * PUSH_K * K;
const PUSH_LEVEL_COUNT_X: f32 = 122.0 * PUSH_K * K;
const PUSH_LEVEL_BASE: f32 = 14.0 * PUSH_K * K;
const PUSH_LEVEL_LABEL_SIZE: f32 = 15.0 * PUSH_K * K;
const PUSH_LEVEL_COUNT_SIZE: f32 = 13.0 * PUSH_K * K;
const PUSH_LEVEL_H: f32 = 26.0 * PUSH_K * K;
const PUSH_GROUP_TAIL: f32 = 14.0 * PUSH_K * K;
const PUSH_CARD_SPLIT_Y: f32 = 58.0 * PUSH_K * K;
const PUSH_CARD_ICE_W: f32 = 170.0 * PUSH_K * K;
const PUSH_CARD_LEAN_X: f32 = 158.0 * PUSH_K * K;
const PUSH_CARD_PAD_X: f32 = 10.0 * PUSH_K * K;
const PUSH_GRADE_CX: f32 = 210.0 * PUSH_K * K;
const PUSH_GRADE_CY: f32 = PUSH_CARD_SPLIT_Y / 2.0;
const PUSH_GRADE_SZ: f32 = 26.0 * PUSH_K * K;
const PUSH_THUMB_X: f32 = 10.0 * PUSH_K * K;
const PUSH_THUMB_Y: f32 = 8.0 * PUSH_K * K;
const PUSH_THUMB_SIZE: f32 = 42.0 * PUSH_K * K;
const PUSH_THUMB_R: f32 = 6.0 * PUSH_K * K;
const PUSH_TITLE_COL_X: f32 = 60.0 * PUSH_K * K;
const PUSH_CARD_TITLE_BASE: f32 = 21.0 * PUSH_K * K;
const PUSH_TITLE_SZ: f32 = 13.0 * PUSH_K * K;
const PUSH_TITLE_W: f32 = 108.0 * PUSH_K * K;
const PUSH_META_BASE: f32 = 49.0 * PUSH_K * K;
const PUSH_META_SZ: f32 = 11.0 * PUSH_K * K;
const PUSH_CONST_TEXT_X: f32 = 60.0 * PUSH_K * K;
const PUSH_RANK_TEXT_X: f32 = 86.0 * PUSH_K * K;
const PUSH_TARGET_BASE: f32 = 75.0 * PUSH_K * K;
const PUSH_TARGET_SZ: f32 = 20.0 * PUSH_K * K;
const PUSH_FROM_BASE: f32 = 90.0 * PUSH_K * K;
const PUSH_FROM_SZ: f32 = 10.0 * PUSH_K * K;
const PUSH_DELTA_CX: f32 = 210.0 * PUSH_K * K;
const PUSH_DELTA_BASE: f32 = 83.0 * PUSH_K * K;
const PUSH_DELTA_SZ: f32 = 18.0 * PUSH_K * K;
const PUSH_PICK_BAR_W: f32 = 3.0 * PUSH_K * K;
const PUSH_FOOT_GAP: f32 = 16.0 * PUSH_K * K;
const PUSH_FOOT_SZ: f32 = 11.0 * PUSH_K * K;
const PUSH_BOTTOM_PAD: f32 = 48.0 * PUSH_K * K;

const CHAP_SCALE: f32 = 1.5;
const CHAP_W: f32 = 1350.0 * K;
const CHAP_PAD_L: f32 = 60.0 * K;
const CHAP_RIGHT: f32 = CHAP_W - 60.0 * K;
const CHAP_TITLE_BASE: f32 = 95.0 * K;
const CHAP_SUB_BASE: f32 = 134.0 * K;
const CHAP_PLAYER_CY: f32 = 99.0 * K;
const CHAP_GRID_TOP: f32 = 189.0 * K;
const CHAP_BAR_W: f32 = 8.0 * K;
const CHAP_BAR_H: f32 = 21.0 * K;
const CHAP_TITLE_SIZE: f32 = 48.0 * K;
const CHAP_SUB_SIZE: f32 = 19.5 * K;
const CHAP_NAME_X: f32 = 78.0 * K;
const CHAP_HEAD_BASE: f32 = 18.0 * K;
const CHAP_NAME_SIZE: f32 = 22.5 * K;
const CHAP_FOOT_SIZE: f32 = 19.5 * K;
const CHAP_HEAD_H: f32 = 36.0 * K;
const CHAP_SECTION_GAP: f32 = 27.0 * K;
const PACK_COLS: usize = 2;
const PACK_W: f32 = 600.0 * K;
const PACK_H: f32 = 99.0 * K;
const PACK_GAP_X: f32 = 18.0 * K;
const PACK_GAP_Y: f32 = 15.0 * K;
const PACK_R: f32 = 12.0 * K;
const PACK_STEP_X: f32 = PACK_W + PACK_GAP_X;
const PACK_STEP_Y: f32 = PACK_H + PACK_GAP_Y;
const PACK_THUMB_X: f32 = 18.0 * K;
const PACK_THUMB_Y: f32 = 15.0 * K;
const PACK_THUMB_SIZE: f32 = 69.0 * K;
const PACK_THUMB_R: f32 = 9.0 * K;
const PACK_TEXT_X: f32 = 105.0 * K;
const PACK_TEXT_W: f32 = PACK_W - 24.0 * K - PACK_TEXT_X;
const PACK_NAME_BASE: f32 = 30.0 * K;
const PACK_NAME_SIZE: f32 = 21.0 * K;
const PACK_FAN_BASE: f32 = 57.0 * K;
const PACK_FAN_SIZE: f32 = 16.5 * K;
const PACK_BAR_Y: f32 = 75.0 * K;
const PACK_BAR_H: f32 = 9.0 * K;
const PACK_COUNT_BASE: f32 = 33.0 * K;
const PACK_COUNT_SIZE: f32 = 22.5 * K;
const PACK_DENOM_SIZE: f32 = 16.5 * K;
const PACK_MAX_SIZE: f32 = 15.0 * K;
const CHART_COLS: usize = 3;
const CHART_W: f32 = 396.0 * K;
const CHART_H: f32 = 156.0 * K;
const CHART_GAP_X: f32 = 18.0 * K;
const CHART_GAP_Y: f32 = 18.0 * K;
const CHART_R: f32 = 12.0 * K;
const CHART_SPLIT_Y: f32 = 90.0 * K;
const CHART_STEP_X: f32 = CHART_W + CHART_GAP_X;
const CHART_STEP_Y: f32 = CHART_H + CHART_GAP_Y;
const CHART_LEFT_X: f32 = 21.0 * K;
const CHART_RIGHT_CX: f32 = CHART_W - 66.0 * K;
const CHART_THUMB_X: f32 = CHART_LEFT_X;
const CHART_THUMB_Y: f32 = 12.0 * K;
const CHART_THUMB_SIZE: f32 = 66.0 * K;
const CHART_THUMB_R: f32 = 9.0 * K;
const CHART_TITLE_X: f32 = CHART_LEFT_X + CHART_THUMB_SIZE + 9.0 * K;
const CHART_TITLE_BASE: f32 = 39.0 * K;
const CHART_TITLE_SIZE: f32 = 18.0 * K;
const CHART_TITLE_W: f32 = 177.0 * K;
const CHART_META_BASE: f32 = 66.0 * K;
const CHART_META_SIZE: f32 = 16.5 * K;
const CHART_GRADE_H: f32 = 51.0 * K;
const CHART_GRADE_CY: f32 = 45.0 * K;
const CHART_ICE_X: f32 = 273.0 * K;
const CHART_ICE_LEAN: f32 = 18.0 * K;
const CHART_SCORE_X: f32 = CHART_LEFT_X;
const CHART_SCORE_BASE: f32 = 121.0 * K;
const CHART_SCORE_SIZE: f32 = 21.0 * K;
const CHART_JUDGE_BASE: f32 = 139.0 * K;
const CHART_JUDGE_SIZE: f32 = 13.5 * K;
const CHART_PTT_CX: f32 = CHART_RIGHT_CX;
const CHART_PTT_BASE: f32 = 129.0 * K;
const CHART_PTT_SIZE: f32 = 18.0 * K;
const CHART_IDLE_SIZE: f32 = 21.0 * K;
const CHAP_FOOT_GAP: f32 = 45.0 * K;
const CHAP_BOTTOM_PAD: f32 = 93.0 * K;

const TREND_CURVE_H: f32 = 660.0 * 0.75 * K;
const TREND_BLOCK_GAP: f32 = 14.0 * 0.75 * K;
const TREND_PLOT_BOTTOM_PAD: f32 = 62.0 * 0.75 * K;
const TREND_STROKE_W: f32 = 2.8 * K;
const TREND_MAJOR_R: f32 = 7.0 * K;
const TREND_NOW_R: f32 = 11.0 * K;
const TREND_AXIS_SIZE: f32 = SIZE_XS;
const TREND_AXIS_LABEL_GAP: f32 = 12.0 * 0.75 * K;
const TREND_GRID_ON: f32 = 8.0;
const TREND_GRID_OFF: f32 = 6.0;
const TREND_RING_W: f32 = 2.0;
const TREND_WASH_FEATHER: f32 = 56.0;
const TREND_AREA_TOP: u8 = 0x3D;
const TREND_AREA_MID: u8 = 0x1A;
const TREND_AREA_BOT: u8 = 0x05;
const TREND_AREA_MID_FRAC: f32 = 0.5;
const CURVE_PAD_LOW: f64 = 0.12;
const CURVE_PAD_HIGH: f64 = 0.16;
const CURVE_MIN_SPAN: f64 = 0.015;
const CURVE_FLAT_SPAN: f64 = 0.05;
const CURVE_TICK_LADDER: [f64; 12] = [
    0.005, 0.01, 0.02, 0.03, 0.05, 0.08, 0.1, 0.15, 0.2, 0.25, 0.5, 1.0,
];

const HISTORY_ROW_H: f32 = 72.0 * 0.75 * K;
const HISTORY_HEAD_GAP: f32 = 26.0 * 0.75 * K;
const HISTORY_HEAD_H: f32 = 20.0 * 0.75 * K;
const HISTORY_COLS: [f32; 6] = [100.0, 0.0, 80.0, 140.0, 55.0, 120.0];
const HISTORY_DATE_W: f32 = 160.0 * 0.75 * K;
const HISTORY_DATE_SIZE: f32 = SIZE_S;
const HISTORY_SCORE_SIZE: f32 = SIZE_M;
const HISTORY_GRADE_SIZE: f32 = SIZE_S;
const HISTORY_JUDGE_SIZE: f32 = SIZE_XS;
const HISTORY_PTT_SIZE: f32 = SIZE_S;
const HISTORY_DELTA_SIZE: f32 = SIZE_S;
const HISTORY_COL_DELTA: usize = 0;
const HISTORY_COL_PTT: usize = 2;
const HISTORY_COL_JUDGE: usize = 3;
const HISTORY_COL_GRADE: usize = 4;
const HISTORY_COL_SCORE: usize = 5;
const HISTORY_COL_DATE: usize = 6;

const INK_STRONG: Color = Color::new(0x24, 0x1D, 0x2F, 0xFF);
const INK_SUB: Color = Color::new(0x5A, 0x4F, 0x6A, 0xFF);
const INK_HEAD: Color = Color::new(0x47, 0x40, 0x55, 0xFF);
const ROW_RULE: Color = Color::new(0xD8, 0xD2, 0xE0, 0xB4);
const SECTION_RULE: Color = Color::new(0xC1, 0xBA, 0xCB, 0xB4);
const SECTION_RULE_STRONG: Color = Color::new(0xB2, 0xAA, 0xBE, 0xF2);
const PAPER: Color = Color::new(0xEF, 0xE7, 0xF4, 0xFF);
const PAPER_MIST: Color = Color::new(0xFB, 0xFC, 0xFE, 0xFF);
const PAPER_TOP: Color = Color::new(0xFB, 0xFC, 0xFE, 0xFF);
const PAPER_HI: Color = Color::new(0xFD, 0xFD, 0xFF, 0xFF);
const PAPER_MID: Color = Color::new(0xFA, 0xFB, 0xFE, 0xFF);
const PAPER_LOW: Color = Color::new(0xF8, 0xF9, 0xFD, 0xFF);
const PAPER_GRAIN: Color = Color::new(0x00, 0x00, 0x00, 0x01);
const BAND_NONE: Color = Color::new(0xDE, 0xD8, 0xE2, 0xFF);
const BAND_UNCLEAR: Color = Color::new(0xBC, 0xB2, 0xC8, 0xFF);
const BAND_CLEARED: Color = Color::new(0x93, 0x87, 0x9F, 0xFF);
const BAND_FC: Color = Color::new(0x5F, 0x52, 0x69, 0xFF);
const BAND_PM: Color = Color::new(0x7C, 0x5F, 0x33, 0xFF);
const CONST_REST: Color = Color::new(0xC3, 0xBA, 0xCC, 0xFF);
const COL_THUMB_RIM: Color = Color::new(0xDC, 0xD6, 0xE2, 0xB4);

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
const PUSH_BAR_TRACK: Color = Color::new(0xE8, 0xEE, 0xF6, 0xFF);
const PUSH_BAR_CLEARED: Color = Color::new(0x7D, 0xD3, 0xFC, 0xFF);
const PUSH_BAR_FC: Color = Color::new(0x0E, 0xA5, 0xE9, 0xFF);
const PUSH_BAR_PM: Color = Color::new(0x03, 0x69, 0xA1, 0xFF);
const PUSH_BAR_FAIL: Color = Color::new(0xFD, 0xA4, 0xAF, 0xFF);
const CHAP_INK_NAME: Color = Color::new(0x0F, 0x17, 0x2A, 0xFF);
const CHAP_INK_DIM: Color = Color::new(0x94, 0xA3, 0xB8, 0xFF);
const CHAP_INK_BLUE: Color = Color::new(0x02, 0x84, 0xC7, 0xFF);
const CHAP_INK_MINT: Color = Color::new(0x05, 0x96, 0x69, 0xFF);
const CHAP_INK_MAX: Color = Color::new(0x10, 0xB9, 0x81, 0xFF);
const CHAP_INK_STAT: Color = Color::new(0x33, 0x45, 0x55, 0xFF);
const CHAP_SLOT_BG: Color = Color::new(0xF1, 0xF5, 0xF9, 0xFF);
const CHAP_SLOT_INK: Color = Color::new(0xE2, 0xE8, 0xF0, 0xFF);
const CHAP_SLOT_ICE: Color = Color::new(0xE8, 0xF0, 0xF6, 0xFF);
const CURVE_INK: Color = Color::new(0x4A, 0x50, 0x7A, 0xFF);
const CURVE_AREA: Color = Color::new(0x62, 0x66, 0xA8, 0xFF);
const CURVE_DROP: Color = Color::new(0x4A, 0x50, 0x7A, 0x40);
const CURVE_GRID: Color = Color::new(0x00, 0x00, 0x00, 0x0F);
const CURVE_AXIS: Color = Color::new(0x00, 0x00, 0x00, 0x1A);
const CURVE_AXIS_INK: Color = Color::new(0x3B, 0x42, 0x52, 0xFF);
const CURVE_AXIS_SUB: Color = Color::new(0x88, 0x92, 0xB0, 0xFF);
const CURVE_GLASS: Color = Color::new(0xFF, 0xFF, 0xFF, 0x4D);
const CURVE_LABEL_PLATE: Color = Color::new(0xFF, 0xFF, 0xFF, 0xF2);
const CURVE_LABEL_EDGE: Color = Color::new(0xC9, 0xC2, 0xD4, 0xB4);
const TREND_PILL_CLEAR: f32 = 12.0;
const TREND_PILL_CLEAR_NOW: f32 = 18.0;
const TREND_PILL_GAP: f32 = 8.0;
const TREND_BOUND_ON: f32 = 14.0;
const TREND_BOUND_OFF: f32 = 8.0;
const CURVE_BOUND: Color = Color::new(0x4A, 0x50, 0x7A, 0x2E);
const CURVE_BOUND_CURRENT: Color = Color::new(0x4A, 0x50, 0x7A, 0x73);

const INK_KICKER: Color = Color::new(0x6D, 0x65, 0x7A, 0xFF);
const LEDGER_HEAD_BAR_GAP: f32 = 36.0 * 0.75 * K;
const LEDGER_LOGO_Y: f32 = 35.0 * 0.75 * K;
const LEDGER_LOGO_W: f32 = 288.0 * 0.75 * K;
const LEDGER_KICKER_SIZE: f32 = 16.0 * 0.75 * K;
const LEDGER_KICKER_TRACK: f32 = 3.2 * 0.75 * K;

const SPACED_RS: f32 = 1.4;
const LEDGER_KICKER_BASE: f32 = 128.0 * 0.75 * K;
const LEDGER_KICKER_OPT_X: f32 = 4.0 * 0.75 * K;
const LOGO_INK_W_FRAC: f32 = 0.692_579_5;
const LOGO_INK_X_FRAC: f32 = 0.197_173_1;
const LOGO_INK_Y_FRAC: f32 = 0.070_886_1;
const LOGO_INK_H_FRAC: f32 = 0.567_088_6;
const SECTION_RULE_DROP: f32 = 6.0 * 0.75 * K;
const SECTION_NOTE_SIZE: f32 = SIZE_S;
const RULE_FADE_FROM: f32 = 0.78;
const FOOT_TEXT_TOP: f32 = 78.0 * 0.75 * K;
const FOOT_RULE_H: f32 = 1.0 * 0.75 * K;
const PUSH_TITLE: &str = "UPSCORE TARGETS";
const CHAP_TITLE: &str = "PACK PROGRESS";
const KICKER_STATS: &str = "PLAYER REPORT";
const KICKER_TREND: &str = "POTENTIAL";
const FOOT_SIGN: &str = "Generated By Okamikko";
const FOOT_PREFIX: &str = "Created At ";
const PUSH_ARROW: &str = "→";

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

fn thousands(value: i64) -> String {
    sketch::thousands(value)
}

fn hatch(kit: &CanvasKit<'_>, box_: Rect, step: i32, width: i32, color: Color) {
    let w = box_.width();
    let h = box_.height();
    if w <= 0.0 || h <= 0.0 || step <= 0 {
        return;
    }
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
}

fn paper_backdrop(kit: &CanvasKit<'_>, width: f32, height: f32) {
    if width <= 0.0 || height <= 0.0 {
        return;
    }
    let paper = sketch::gradient_stops(&[
        (0.0, PAPER_TOP),
        (1.0 / 3.0, PAPER_HI),
        (2.0 / 3.0, PAPER_MID),
        (1.0, PAPER_LOW),
    ]);
    let _ = kit.fill_gradient(area(0.0, 0.0, width, height), 0.0, &paper);
    let rows = height.round() as i32;
    let grain: Vec<Rect> = (0..rows)
        .filter(|y| (*y as u32).wrapping_mul(0x9E37_79B1) & 0x8000 == 0)
        .map(|y| area(0.0, y as f32, width, 1.0))
        .collect();
    if !grain.is_empty() {
        kit.fill_rects_batch(&grain, PAPER_GRAIN);
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
        (0.0, with_alpha(PAPER_MIST, top)),
        (0.25, with_alpha(PAPER_MIST, upper)),
        (0.5, with_alpha(PAPER_MIST, mid)),
        (0.75, with_alpha(PAPER_MIST, lower)),
        (1.0, with_alpha(PAPER_MIST, bottom)),
    ]);
    let _ = kit.fill_gradient(area(0.0, 0.0, width, height), 0.0, &gradient);
}

pub(crate) fn ledger_art_wash(kit: &CanvasKit<'_>, key: &str, width: f32, height: f32) {
    art_wash(kit, key, width, height, 0xC4, 0xE6, 0xF2);
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

fn block_of(badge: &PlayerBadge, right: f32, center_y: f32) -> player::Block<'_> {
    player::Block::new(badge, right, center_y).with_left(MARGIN_X)
}

fn plate_box(kit: &CanvasKit<'_>, badge: &PlayerBadge, right: f32) -> (f32, f32) {
    player::plate_box(kit, &block_of(badge, right, compact_center_y()))
}

fn player_block(kit: &CanvasKit<'_>, badge: &PlayerBadge, right: f32, center_y: f32) {
    player::paint(kit, &block_of(badge, right, center_y));
}

fn player_bottom(center_y: f32) -> f32 {
    (center_y + AVATAR_R).max(center_y + PLATE_H / 2.0)
}

pub(crate) fn compact_center_y() -> f32 {
    let top = COMPACT_HEAD_TOP - HEAD_TITLE_SIZE * 0.72;
    let bottom = COMPACT_HEAD_TOP + (HEAD_NOTE_BASE - HEAD_TITLE_BASE) + HEAD_NOTE_SIZE * 0.20;
    (top + bottom) / 2.0
}

struct LedgerFrame {
    left: f32,
    right: f32,
    brand_right: bool,
    no_brand: bool,
    brand_rule_y: f32,
}

fn rule_fade(kit: &CanvasKit<'_>, x0: f32, x1: f32, y: f32, color: Color) {
    let width = (x1 - x0).max(0.0);
    if width <= 0.0 {
        return;
    }
    let solid = x0 + width * RULE_FADE_FROM;
    kit.fill_rect(area(x0, y, solid - x0, FOOT_RULE_H), color);
    let gradient = Gradient::Linear(LinearGradient::horizontal([
        GradientStop::new(0.0, color),
        GradientStop::new(1.0, with_alpha(color, 0)),
    ]));
    let _ = kit.fill_gradient(area(solid, y, x1 - solid, FOOT_RULE_H), 0.0, &gradient);
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
    let crop = Rect::from_xywh(
        iw as f32 * LOGO_INK_X_FRAC,
        ih as f32 * LOGO_INK_Y_FRAC,
        iw as f32 * LOGO_INK_W_FRAC,
        ih as f32 * LOGO_INK_H_FRAC,
    );
    if crop.width() <= 0.0 || crop.height() <= 0.0 {
        return;
    }
    let height = width * crop.height() / crop.width();
    let _ = sketch::paste_crop_clip(kit, key, crop, area(x, y, width, height));
}

fn ledger_brand(kit: &CanvasKit<'_>, frame: &LedgerFrame, kicker: &str) {
    let logo_x = if frame.brand_right {
        frame.right - LEDGER_LOGO_W
    } else {
        frame.left
    };
    ledger_logo(kit, logo_x, LEDGER_LOGO_Y, LEDGER_LOGO_W);
    if kicker.is_empty() {
        return;
    }
    let track = LEDGER_KICKER_TRACK / SPACED_RS;
    let style = font_number(LEDGER_KICKER_SIZE, INK_KICKER);
    let x = if frame.brand_right {
        let width = sketch::text_spaced_width(kit, &style, kicker, track);
        logo_x + (LEDGER_LOGO_W - width) / 2.0
    } else {
        logo_x + LEDGER_KICKER_OPT_X
    };
    sketch::text_spaced(
        kit,
        &style,
        kicker,
        x,
        LEDGER_KICKER_BASE,
        INK_KICKER,
        track,
    );
    if frame.brand_rule_y > 0.0 {
        rule_fade(kit, frame.left, frame.right, frame.brand_rule_y, INK_KICKER);
    }
}

fn ledger_head(
    kit: &CanvasKit<'_>,
    frame: &LedgerFrame,
    title: &str,
    note: &str,
    kicker: &str,
    plate_left: f32,
) {
    let mut title = title;
    let mut kicker = kicker;
    if title.is_empty() {
        title = kicker;
        kicker = "";
    }
    if !frame.no_brand {
        ledger_brand(kit, frame, kicker);
    }
    let mut avail = (frame.right - frame.left).max(0.0);
    if plate_left > 0.0 {
        let room = (plate_left - LEDGER_HEAD_BAR_GAP - frame.left).max(0.0);
        if room < avail {
            avail = room;
        }
    }
    if frame.brand_right {
        let room = frame.right - LEDGER_LOGO_W - LEDGER_HEAD_BAR_GAP - frame.left;
        if room > 0.0 && room < avail {
            avail = room;
        }
    }
    if avail < 1.0 {
        return;
    }
    if !title.is_empty() {
        let style = font_text(HEAD_TITLE_SIZE, INK_STRONG);
        let text = sketch::ellipsize(kit, &style, title, avail);
        sketch::text(kit, &style, &text, frame.left, COMPACT_HEAD_TOP, INK_STRONG);
    }
    if !note.is_empty() {
        let style = font_text(HEAD_NOTE_SIZE, INK_SUB);
        let text = sketch::ellipsize(kit, &style, note, avail);
        sketch::text(
            kit,
            &style,
            &text,
            frame.left,
            COMPACT_HEAD_TOP + (HEAD_NOTE_BASE - HEAD_TITLE_BASE),
            INK_SUB,
        );
    }
}

fn foot_rule(kit: &CanvasKit<'_>, foot_y: f32, x0: f32, x1: f32) {
    for offset in [4.0, 7.0] {
        kit.fill_rect(
            area(x0, foot_y + offset * 0.75, (x1 - x0).max(0.0), FOOT_RULE_H),
            Color::new(0xC8, 0xC1, 0xD2, 0x78),
        );
    }
}

fn ledger_foot(kit: &CanvasKit<'_>, foot_y: f32, x0: f32, x1: f32) {
    foot_rule(kit, foot_y, x0, x1);
    sketch::text(
        kit,
        &font_text(SIZE_S, INK_SUB),
        FOOT_SIGN,
        x0,
        foot_y + FOOT_TEXT_TOP,
        INK_SUB,
    );
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

fn now_stamp() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_secs() as i64)
        .unwrap_or(0)
        + 8 * 3600;
    let (year, month, day) = civil_from_days(seconds.div_euclid(86_400));
    let rem = seconds.rem_euclid(86_400);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02}",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

fn ledger_foot_dated(kit: &CanvasKit<'_>, foot_y: f32, x0: f32, x1: f32) {
    foot_rule(kit, foot_y, x0, x1);
    let base = foot_y + FOOT_TEXT_TOP;
    let stamp = format!("{FOOT_PREFIX}{}", now_stamp());
    sketch::text(kit, &font_text(SIZE_S, INK_SUB), &stamp, x0, base, INK_SUB);
    sketch::text_right(
        kit,
        &font_text(SIZE_S, INK_SUB),
        FOOT_SIGN,
        x1,
        base,
        INK_SUB,
    );
}

fn section_head(
    kit: &CanvasKit<'_>,
    x0: f32,
    x1: f32,
    base: f32,
    title: &str,
    note: &str,
    rule: Color,
) {
    let style = font_number(SECTION_SIZE, INK_STRONG);
    let text = sketch::ellipsize(kit, &style, title, (x1 - x0).max(0.0));
    rule_fade(kit, x0, x1, base + SECTION_RULE_DROP, rule);
    sketch::text(kit, &style, &text, x0, base, INK_STRONG);
    if !note.is_empty() {
        sketch::text_right(
            kit,
            &font_text(SECTION_NOTE_SIZE, INK_SUB),
            note,
            x1,
            base,
            INK_SUB,
        );
    }
}

struct MicroHead<'a> {
    title: &'a str,
    note: &'a str,
    x: f32,
    title_base: f32,
    sub_base: f32,
    title_size: f32,
    sub_size: f32,
    avail: f32,
}

fn micro_head(kit: &CanvasKit<'_>, head: MicroHead<'_>) {
    let MicroHead {
        title,
        note,
        x,
        title_base,
        sub_base,
        title_size,
        sub_size,
        avail,
    } = head;
    let title_style = font_display(title_size, PUSH_INK);
    let title = sketch::ellipsize(kit, &title_style, title, avail);
    sketch::text(kit, &title_style, &title, x, title_base, PUSH_INK);
    if note.is_empty() {
        return;
    }
    let note_style = font_text(sub_size, PUSH_INK_SUB);
    let note = sketch::ellipsize(kit, &note_style, note, avail);
    sketch::text(kit, &note_style, &note, x, sub_base, PUSH_INK_SUB);
}

#[derive(Debug, Clone, Copy, Default)]
struct Block {
    have: bool,
    title_base: f32,
    rule_y: f32,
    body_top: f32,
    head_base: f32,
}

#[derive(Debug, Clone, Default)]
struct Flow {
    y: f32,
}

impl Flow {
    fn new(top: f32) -> Self {
        Self { y: top }
    }

    fn open(&mut self, block: &mut Block) -> f32 {
        block.have = true;
        block.title_base = self.y + SECTION_TOP + SECTION_SIZE;
        block.rule_y = block.title_base + SECTION_RULE_GAP;
        block.body_top = block.rule_y + RULE_H + SECTION_GAP;
        self.y = block.body_top;
        self.y
    }

    fn advance(&mut self, height: f32) {
        self.y += height;
    }

    fn rows(&mut self, count: usize, row_h: f32, gap: f32, x0: f32, w: f32) -> Vec<Rect> {
        let mut rows = Vec::with_capacity(count);
        if count == 0 {
            return rows;
        }
        for _ in 0..count {
            rows.push(area(x0, self.y, w, row_h));
            self.y += row_h + gap;
        }
        self.y -= gap;
        rows
    }
}

fn stats_band_scale(width: f32) -> f32 {
    let full = LEDGER_W - 2.0 * MARGIN_X;
    if width <= 0.0 || width >= full {
        return 1.0;
    }
    let need = full - STATS_BAND_NAME_W - 90.0 * 0.75 * K;
    let avail = width - STATS_BAND_NAME_W - 90.0 * 0.75 * K;
    if need <= 0.0 {
        return 1.0;
    }
    (avail / need).max(0.55)
}

fn stats_band_right(x1: f32, width: f32, index: usize) -> f32 {
    let k = stats_band_scale(width);
    let mut x = x1 - PROFILE_COL_INSET_R;
    for col in STATS_BAND_COLS.iter().take(index) {
        x -= col * 0.75 * K * k + PROFILE_COL_GAP * k;
    }
    x
}

fn stats_band_bar(row: Rect) -> Rect {
    let k = stats_band_scale(row.width());
    let last = STATS_BAND_COLS.len() - 1;
    let x0 = row.left() + STATS_BAND_NAME_W;
    let x1 = stats_band_right(row.right(), row.width(), last)
        - STATS_BAND_COLS[last] * 0.75 * K * k
        - 20.0 * 0.75 * K;
    area(
        x0,
        row.top() + (row.height() - STATS_BAND_BAR_H) / 2.0,
        (x1 - x0).max(0.0),
        STATS_BAND_BAR_H,
    )
}

fn stack(kit: &CanvasKit<'_>, box_: Rect, segments: &[(f32, Color)]) {
    let total: f32 = segments.iter().map(|(weight, _)| weight.max(0.0)).sum();
    if total <= 0.0 || box_.width() <= 0.0 || box_.height() <= 0.0 {
        return;
    }
    let mut x = box_.left();
    for (weight, color) in segments {
        let width = box_.width() * (weight.max(0.0) / total);
        if width > 0.0 {
            kit.fill_rect(area(x, box_.top(), width, box_.height()), *color);
        }
        x += width;
    }
}

fn grade_pair_x(x0: f32, width: f32, index: usize, count: usize) -> f32 {
    if count <= 1 {
        return x0;
    }
    let spare = (width - count as f32 * STATS_GRADE_PAIR_W).max(0.0);
    x0 + index as f32 * (STATS_GRADE_PAIR_W + spare / (count as f32 - 1.0))
}

fn grade_pair_cells(width: f32) -> usize {
    let width = if width <= 0.0 {
        LEDGER_W - 2.0 * MARGIN_X
    } else {
        width
    };
    ((width / (120.0 * 0.75 * K + STATS_GRADE_GAP)) as usize).max(1)
}

fn short_col_right(x1: f32, index: usize) -> f32 {
    let mut x = x1;
    for col in STATS_SHORT_COLS.iter().take(index) {
        x -= col * 0.75 * K + 36.0 * 0.75 * K;
    }
    x
}

#[derive(Debug, Clone, Default)]
pub struct StatsBand {
    pub name: String,
    pub total: i32,
    pub played: i32,
    pub cleared: i32,
    pub full_combo: i32,
    pub pm: i32,
    pub avg_score: f64,
}

impl StatsBand {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Self::default()
        }
    }

    pub fn with_counts(mut self, total: i32, played: i32) -> Self {
        self.total = total;
        self.played = played;
        self
    }

    pub fn with_clear(mut self, cleared: i32, full_combo: i32, pm: i32) -> Self {
        self.cleared = cleared;
        self.full_combo = full_combo;
        self.pm = pm;
        self
    }

    pub fn with_avg(mut self, avg_score: f64) -> Self {
        self.avg_score = avg_score;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct StatsConst {
    pub constant: f64,
    pub total: i32,
    pub played: i32,
}

impl StatsConst {
    pub fn new(constant: f64, total: i32, played: i32) -> Self {
        Self {
            constant,
            total,
            played,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StatsGrade {
    pub name: String,
    pub count: i32,
}

impl StatsGrade {
    pub fn new(name: impl Into<String>, count: i32) -> Self {
        Self {
            name: name.into(),
            count,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StatsRow {
    pub song_id: String,
    pub title: String,
    pub diff: String,
    pub constant: f64,
    pub known: bool,
    pub score: i64,
    pub grade: String,
    pub next: String,
    pub need: i64,
}

impl StatsRow {
    pub fn new(
        song_id: impl Into<String>,
        title: impl Into<String>,
        diff: impl Into<String>,
    ) -> Self {
        Self {
            song_id: song_id.into(),
            title: title.into(),
            diff: diff.into(),
            ..Self::default()
        }
    }

    pub fn with_constant(mut self, constant: f64) -> Self {
        self.constant = constant;
        self.known = true;
        self
    }

    pub fn with_score(mut self, score: i64) -> Self {
        self.score = score;
        self
    }

    pub fn with_grade(mut self, grade: impl Into<String>) -> Self {
        self.grade = grade.into();
        self
    }

    pub fn with_need(mut self, next: impl Into<String>, need: i64) -> Self {
        self.next = next.into();
        self.need = need;
        self
    }
}
#[derive(Debug, Clone)]
pub struct StatsInput {
    pub title: String,
    pub note: String,
    pub backdrop: String,
    pub badge: PlayerBadge,
    pub bands: Vec<StatsBand>,
    pub consts: Vec<StatsConst>,
    pub const_note: String,
    pub grades: Vec<StatsGrade>,
    pub short_title: String,
    pub short: Vec<StatsRow>,
    pub short_note: String,
}

impl StatsInput {
    pub fn new(badge: PlayerBadge) -> Self {
        Self {
            title: CardKind::Stats.label().to_string(),
            note: String::new(),
            backdrop: String::new(),
            badge,
            bands: Vec::new(),
            consts: Vec::new(),
            const_note: String::new(),
            grades: Vec::new(),
            short_title: String::new(),
            short: Vec::new(),
            short_note: String::new(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = note.into();
        self
    }

    pub fn with_backdrop(mut self, backdrop: impl Into<String>) -> Self {
        self.backdrop = backdrop.into();
        self
    }
    pub fn with_bands(mut self, bands: Vec<StatsBand>) -> Self {
        self.bands = bands;
        self
    }

    pub fn with_band(mut self, band: StatsBand) -> Self {
        self.bands.push(band);
        self
    }

    pub fn with_consts(mut self, consts: Vec<StatsConst>) -> Self {
        self.consts = consts;
        self
    }

    pub fn with_const_note(mut self, note: impl Into<String>) -> Self {
        self.const_note = note.into();
        self
    }

    pub fn with_grades(mut self, grades: Vec<StatsGrade>) -> Self {
        self.grades = grades;
        self
    }

    pub fn with_short_title(mut self, title: impl Into<String>) -> Self {
        self.short_title = title.into();
        self
    }

    pub fn with_short(mut self, rows: Vec<StatsRow>) -> Self {
        self.short = rows;
        self
    }

    pub fn with_short_note(mut self, note: impl Into<String>) -> Self {
        self.short_note = note.into();
        self
    }

    fn art_key(&self) -> String {
        if !self.backdrop.is_empty() {
            return format!("jackets/{}.jpg", self.backdrop);
        }
        let mut parts: Vec<&str> = vec!["stats", self.title.as_str(), self.note.as_str()];
        parts.extend(self.short.iter().take(8).map(|row| row.song_id.as_str()));
        jacket_pool::ledger_art(&parts)
            .map(|id| format!("jackets/{id}.jpg"))
            .unwrap_or_default()
    }
}

impl Default for StatsInput {
    fn default() -> Self {
        Self::new(PlayerBadge::new("Afkeru"))
    }
}

#[derive(Debug, Clone, Default)]
struct StatsPlan {
    height: f32,
    last_bottom: f32,
    bands: Block,
    consts: Block,
    plot: Rect,
    grades: Block,
    short: Block,
    band_rows: Vec<Rect>,
    short_rows: Vec<Rect>,
}

fn stats_plan(input: &StatsInput) -> StatsPlan {
    let col_x = MARGIN_X;
    let col_w = LEDGER_W - 2.0 * MARGIN_X;
    let grid_top = player_bottom(compact_center_y()) + COMPACT_GRID_GAP;
    let short_note = !input.short_title.is_empty() && input.short.is_empty();
    let mut plan = StatsPlan::default();
    let mut flow = Flow::new(grid_top);
    if !input.bands.is_empty() {
        flow.open(&mut plan.bands);
        plan.bands.head_base = flow.y + 26.0 * 0.75 * K;
        flow.advance(plan.bands.head_base + 18.0 * 0.75 * K - flow.y);
        plan.band_rows =
            flow.rows(input.bands.len(), STATS_BAND_ROW_H, STATS_BAND_GAP, col_x, col_w);
    }
    if !input.consts.is_empty() {
        flow.open(&mut plan.consts);
        flow.advance(STATS_CONST_HEADROOM);
        plan.plot = area(col_x, flow.y, col_w, STATS_CONST_PLOT_H);
        flow.advance(STATS_CONST_PLOT_H + STATS_CONST_FOOT_ROOM);
    }
    if !input.grades.is_empty() {
        flow.open(&mut plan.grades);
        flow.advance(STATS_GRADE_ROW_H);
    }
    if !input.short.is_empty() || short_note {
        flow.open(&mut plan.short);
        if !input.short.is_empty() {
            plan.short.head_base = flow.y + 26.0 * 0.75 * K;
            flow.advance(plan.short.head_base + 18.0 * 0.75 * K - flow.y);
            plan.short_rows = flow.rows(input.short.len(), STATS_SHORT_ROW_H, 0.0, col_x, col_w);
        }
        if short_note {
            let base = plan.short.rule_y + STATS_SHORT_NOTE_TOP;
            flow.advance(base + 10.0 * 0.75 * K - flow.y);
        }
    }
    plan.last_bottom = flow.y;
    plan.height = plan.last_bottom + FOOTER_H;
    plan
}

pub fn stats_size(input: &StatsInput) -> (i32, i32) {
    (STATS_WIDTH, clamp_height(stats_plan(input).height))
}

pub fn stats_request(input: &StatsInput) -> RenderRequest {
    let (width, height) = stats_size(input);
    request_for(width, height).with_scale(1.5)
}

pub fn stats_card(input: StatsInput) -> RenderCard {
    let request = stats_request(&input);
    let label = format!(
        "{} {}x{}",
        CardKind::Stats.label(),
        request.width,
        request.height
    );
    let cache_key = super::memo::fingerprint("stats", &input);
    RenderCard::new(request, label, move |kit| paint_stats(kit, &input)).with_cache_key(cache_key)
}

pub fn paint_stats(kit: &CanvasKit<'_>, input: &StatsInput) {
    let plan = stats_plan(input);
    let width = STATS_WIDTH as f32;
    let height = stats_size(input).1 as f32;
    let right = LEDGER_W - MARGIN_X;
    paper_backdrop(kit, width, height);
    ledger_art_wash(kit, &input.art_key(), width, height);
    let frame = LedgerFrame {
        left: MARGIN_X,
        right,
        brand_right: false,
        no_brand: true,
        brand_rule_y: 0.0,
    };
    let (plate_w, plate_x) = plate_box(kit, &input.badge, right);
    let plate_left = if plate_w > 0.0 { plate_x } else { 0.0 };
    ledger_head(
        kit,
        &frame,
        input.title.trim(),
        &input.note,
        KICKER_STATS,
        plate_left,
    );
    player_block(kit, &input.badge, right, compact_center_y());
    paint_stats_bands(kit, input, &plan, right);
    paint_stats_consts(kit, input, &plan, right);
    paint_stats_grades(kit, input, &plan, right);
    paint_stats_short(kit, input, &plan, right);
    ledger_foot(kit, plan.last_bottom, MARGIN_X, right);
}

fn paint_stats_short(kit: &CanvasKit<'_>, input: &StatsInput, plan: &StatsPlan, right: f32) {
    if input.short_title.is_empty() || !plan.short.have {
        return;
    }
    section_head(
        kit,
        MARGIN_X,
        right,
        plan.short.title_base,
        &input.short_title,
        "",
        SECTION_RULE,
    );
    let (x0, x1) = (MARGIN_X, right);
    if input.short.is_empty() {
        let base = plan.short.rule_y + STATS_SHORT_NOTE_TOP;
        let style = font_text(SECTION_SIZE * 0.62, INK_SUB);
        sketch::text(kit, &style, &input.short_note, x0, base, INK_SUB);
        return;
    }
    let head_style = font_text(HEAD_SIZE, INK_HEAD);
    sketch::text(
        kit,
        &head_style,
        "曲目",
        x0 + STATS_SHORT_THUMB + STATS_SHORT_GAP,
        plan.short.head_base,
        INK_HEAD,
    );
    for (index, name) in stats_short_heads().iter().enumerate() {
        sketch::text_right(
            kit,
            &head_style,
            name,
            short_col_right(x1, index),
            plan.short.head_base,
            INK_HEAD,
        );
    }
    for (index, item) in input.short.iter().enumerate() {
        let Some(row) = plan.short_rows.get(index) else {
            break;
        };
        stats_short_row(kit, *row, item);
        if index + 1 < plan.short_rows.len() {
            kit.fill_rect(area(x0, row.bottom(), x1 - x0, 1.0), ROW_RULE);
        }
    }
}

fn stats_short_row(kit: &CanvasKit<'_>, row: Rect, item: &StatsRow) {
    let box_ = area(
        row.left(),
        row.top() + (row.height() - STATS_SHORT_THUMB) / 2.0,
        STATS_SHORT_THUMB,
        STATS_SHORT_THUMB,
    );
    thumb(kit, box_, &item.song_id);
    let width = stats_short_title_w(row);
    if width > 0.0 && !item.title.is_empty() {
        let style = font_text(SIZE_M, INK_STRONG);
        let text = sketch::ellipsize(kit, &style, &item.title, width);
        let base = row.top() + row.height() / 2.0 + SIZE_M * 0.36;
        sketch::text(kit, &style, &text, stats_short_title_x(row), base, INK_STRONG);
    }
    let base = row.top() + row.height() / 2.0 + SIZE_S * 0.36;
    let cell = font_number(SIZE_S, INK_SUB);
    sketch::text_right(kit, &cell, &item.diff, short_col_right(row.right(), 4), base, INK_SUB);
    sketch::text_right(
        kit,
        &cell,
        &stats_const_text(item),
        short_col_right(row.right(), 3),
        base,
        INK_SUB,
    );
    let score = font_number(SIZE_S, INK_STRONG);
    sketch::text_right(
        kit,
        &score,
        &thousands(item.score),
        short_col_right(row.right(), 2),
        base,
        INK_STRONG,
    );
    let ink = stats_grade_ink(&item.grade);
    let grade = font_number(SIZE_S, ink);
    sketch::text_right(kit, &grade, &item.grade, short_col_right(row.right(), 1), base, ink);
    sketch::text_right(
        kit,
        &cell,
        &stats_need_text(item),
        short_col_right(row.right(), 0),
        base,
        INK_SUB,
    );
}

fn thumb(kit: &CanvasKit<'_>, box_: Rect, song_id: &str) {
    let key = thumb_key(song_id);
    sketch::clip_round(kit, box_, STATS_SHORT_RADIUS, || {
        if key.is_empty() || kit.draw_asset_cover(&key, box_, 0.0).is_err() {
            sketch::fill_round(kit, box_, STATS_SHORT_RADIUS, PAPER);
        }
    });
    sketch::stroke_round(kit, box_, STATS_SHORT_RADIUS, 0.75 * K, COL_THUMB_RIM);
}

fn stats_short_title_x(row: Rect) -> f32 {
    row.left() + STATS_SHORT_THUMB + STATS_SHORT_GAP
}

fn stats_short_title_w(row: Rect) -> f32 {
    short_col_left(row.right(), 4) - 24.0 * 0.75 * K - stats_short_title_x(row)
}

fn short_col_left(x1: f32, index: usize) -> f32 {
    short_col_right(x1, index) - STATS_SHORT_COLS[index] * 0.75 * K
}

fn stats_short_heads() -> [&'static str; 5] {
    ["距下一档", "评级", "分数", "定数", "难度"]
}

fn stats_const_text(item: &StatsRow) -> String {
    if !item.known {
        return String::from("—");
    }
    format!("{:.1}", item.constant)
}

fn stats_need_text(item: &StatsRow) -> String {
    if item.need <= 0 {
        return String::from("满分");
    }
    format!("+{}", thousands(item.need))
}

fn stats_grade_ink(grade: &str) -> Color {
    if grade == "PM" {
        BAND_PM
    } else {
        INK_SUB
    }
}

fn band_heads() -> [&'static str; 5] {
    ["均分", "PM", "FR", "通关", "已记录/总"]
}

fn band_segments(band: &StatsBand) -> Vec<(f32, Color)> {
    let total = band.total.max(0);
    let pm = band.pm.clamp(0, total);
    let fc = band.full_combo.clamp(pm, total);
    let cleared = band.cleared.clamp(fc, total);
    let played = band.played.clamp(cleared, total);
    vec![
        ((total - played) as f32, BAND_NONE),
        ((played - cleared) as f32, BAND_UNCLEAR),
        ((cleared - fc) as f32, BAND_CLEARED),
        ((fc - pm) as f32, BAND_FC),
        (pm as f32, BAND_PM),
    ]
}

fn band_avg_text(band: &StatsBand) -> String {
    if band.played <= 0 {
        return String::from("—");
    }
    thousands((band.avg_score + 0.5) as i64)
}

const BAND_LEGEND: [(&str, Color); 5] = [
    ("未打", BAND_NONE),
    ("未通", BAND_UNCLEAR),
    ("通关", BAND_CLEARED),
    ("FR", BAND_FC),
    ("PM", BAND_PM),
];

fn band_legend(kit: &CanvasKit<'_>, x: f32, base: f32) {
    let style = font_text(HEAD_SIZE, INK_SUB);
    let mut cursor = x;
    cursor += sketch::text(kit, &style, "条 =", cursor, base, INK_SUB) + 10.0 * 0.75 * K;
    for (label, ink) in BAND_LEGEND {
        kit.fill_rect(
            area(cursor, base - 10.0 * 0.75 * K, 6.0 * 0.75 * K, 6.0 * 0.75 * K),
            ink,
        );
        cursor += 6.0 * 0.75 * K + 8.0 * 0.75 * K;
        cursor += sketch::text(kit, &style, label, cursor, base, INK_SUB) + 12.0 * 0.75 * K;
    }
}

fn paint_stats_grades(kit: &CanvasKit<'_>, input: &StatsInput, plan: &StatsPlan, right: f32) {
    if input.grades.is_empty() || !plan.grades.have {
        return;
    }
    section_head(kit, MARGIN_X, right, plan.grades.title_base, "评级构成", "", SECTION_RULE);
    let col_w = right - MARGIN_X;
    let count = input.grades.len().min(grade_pair_cells(col_w));
    let base = plan.grades.rule_y + STATS_GRADE_ROW_H / 2.0 + STATS_GRADE_VALUE_SIZE * 0.34;
    for (index, grade) in input.grades.iter().take(count).enumerate() {
        let pair = grade_pair_x(MARGIN_X, col_w, index, count);
        sketch::text_right(
            kit,
            &font_text(SIZE_M, INK_STRONG),
            &grade.name,
            pair + 44.0 * 0.75 * K,
            base,
            INK_STRONG,
        );
        sketch::text(
            kit,
            &font_number(STATS_GRADE_VALUE_SIZE, INK_STRONG),
            &grade.count.to_string(),
            pair + 60.0 * 0.75 * K,
            base,
            INK_STRONG,
        );
    }
}

fn paint_stats_consts(kit: &CanvasKit<'_>, input: &StatsInput, plan: &StatsPlan, right: f32) {
    if input.consts.is_empty() || plan.plot.width() <= 0.0 {
        return;
    }
    let mut note = String::from("柱高 谱面 · 青色 已玩");
    if !input.const_note.is_empty() {
        note.push_str(" · ");
        note.push_str(&input.const_note);
    }
    section_head(kit, MARGIN_X, right, plan.consts.title_base, "定数细分", &note, SECTION_RULE);
    let max = input.consts.iter().map(|item| item.total).max().unwrap_or(0).max(0) as f32;
    let n = input.consts.len();
    let axis_y = plan.plot.bottom() - 1.0;
    kit.fill_rect(area(plan.plot.left(), axis_y, plan.plot.width(), 1.0), SECTION_RULE);
    let plot_h = (axis_y - plan.plot.top()).max(0.0);
    for (index, item) in input.consts.iter().enumerate() {
        let slot = plan.plot.width() / n as f32;
        let mut width = slot - COMP_BAR_GAP;
        if width <= 0.0 {
            width = slot;
        }
        let mut left = plan.plot.left() + index as f32 * slot + COMP_BAR_GAP / 2.0;
        if width > STATS_CONST_BAR_MAX_W {
            left += (width - STATS_CONST_BAR_MAX_W) / 2.0;
            width = STATS_CONST_BAR_MAX_W;
        }
        let value = item.total as f32;
        let bar_h = if value <= 0.0 || max <= 0.0 || plot_h <= 0.0 {
            0.0
        } else if value >= max {
            plot_h
        } else {
            plot_h * value / max
        };
        let top = axis_y - bar_h;
        if bar_h > 0.0 {
            kit.fill_rect(area(left, top, width, bar_h), CONST_REST);
            let ratio = if item.total <= 0 { 0.0 } else { item.played as f32 / item.total as f32 };
            let fill = if ratio <= 0.0 { 0.0 } else if ratio >= 1.0 { bar_h } else { bar_h * ratio };
            if fill > 0.0 {
                kit.fill_rect(area(left, axis_y - fill, width, fill), BAND_FC);
            }
        }
        let text = format!("{}/{}", item.played, item.total);
        let value_style = font_text(STATS_CONST_VALUE_SIZE, INK_STRONG);
        sketch::text_center(kit, &value_style, &text, left + width / 2.0, top - COMP_BAR_GAP, INK_STRONG);
        let label = format!("{:.1}", item.constant);
        let label_style = font_text(STATS_CONST_LABEL_SIZE, INK_SUB);
        let label_base = plan.plot.bottom() + COMP_BAR_GAP + STATS_CONST_LABEL_SIZE;
        sketch::text_center(kit, &label_style, &label, left + width / 2.0, label_base, INK_SUB);
    }
}

fn paint_stats_bands(kit: &CanvasKit<'_>, input: &StatsInput, plan: &StatsPlan, right: f32) {
    if input.bands.is_empty() || !plan.bands.have {
        return;
    }
    section_head(kit, MARGIN_X, right, plan.bands.title_base, "完成度", "", SECTION_RULE);
    let head_style = font_text(HEAD_SIZE, INK_SUB);
    for (index, name) in band_heads().iter().enumerate() {
        sketch::text_right(
            kit,
            &head_style,
            name,
            stats_band_right(right, right - MARGIN_X, index),
            plan.bands.head_base,
            INK_SUB,
        );
    }
    if let Some(row) = plan.band_rows.first() {
        let bar = stats_band_bar(*row);
        if bar.width() >= 300.0 * 0.75 * K {
            band_legend(kit, bar.left(), plan.bands.head_base);
        }
    }
    for (index, band) in input.bands.iter().enumerate() {
        let Some(row) = plan.band_rows.get(index) else {
            break;
        };
        let base = row.top() + row.height() / 2.0;
        sketch::text(
            kit,
            &font_number(SIZE_M, INK_STRONG),
            &band.name,
            row.left(),
            base + SIZE_M * 0.36,
            INK_STRONG,
        );
        stack(kit, stats_band_bar(*row), &band_segments(band));
        let cells = [
            (band_avg_text(band), INK_STRONG),
            (band.pm.to_string(), INK_SUB),
            (band.full_combo.to_string(), INK_SUB),
            (band.cleared.to_string(), INK_SUB),
            (format!("{}/{}", band.played, band.total), INK_STRONG),
        ];
        for (cell, (value, ink)) in cells.iter().enumerate() {
            sketch::text_right(
                kit,
                &font_number(SIZE_S, *ink),
                value,
                stats_band_right(row.right(), row.width(), cell),
                base + SIZE_S * 0.36,
                *ink,
            );
        }
    }
}

#[derive(Debug, Clone)]
pub struct SuggestInput {
    pub title: String,
    pub subtitle: String,
    pub theme: Theme,
    pub badge: PlayerBadge,
    pub board: SuggestBoard,
    pub note: String,
    pub backdrop: String,
    pub ptt_before: Option<f64>,
    pub ptt_after: Option<f64>,
}

impl SuggestInput {
    pub fn new(badge: PlayerBadge, board: SuggestBoard) -> Self {
        Self {
            title: CardKind::Suggest.label().to_string(),
            subtitle: String::new(),
            theme: Theme::DEFAULT,
            badge,
            board,
            ptt_before: None,
            ptt_after: None,
            note: String::new(),
            backdrop: String::new(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_ptt(mut self, before: f64, after: f64) -> Self {
        self.ptt_before = Some(before);
        self.ptt_after = Some(after);
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
}

impl Default for SuggestInput {
    fn default() -> Self {
        Self::new(
            PlayerBadge::new("Afkeru"),
            SuggestBoard {
                groups: Vec::new(),
                hint_count: 0,
                pick_count: 0,
                pool: 0,
                pushable: 0,
            },
        )
    }
}

fn hint_note_text(hint: &SuggestHint) -> String {
    if !hint.notes.known {
        return String::new();
    }
    let mut parts: Vec<String> = Vec::new();
    if hint.notes.lost_use > 0 {
        parts.push(format!("-{}L", hint.notes.lost_use));
    }
    if hint.notes.far_use > 0 {
        parts.push(format!("-{}F", hint.notes.far_use));
    }
    parts.join(" ")
}

#[derive(Debug, Clone, Copy, Default)]
struct PushGroupPlan {
    top: f32,
    first: usize,
    count: usize,
}

#[derive(Debug, Clone, Default)]
struct PushPlan {
    cards: Vec<(f32, f32)>,
    groups: Vec<PushGroupPlan>,
    foot_y: f32,
    height: f32,
}

fn push_plan(input: &SuggestInput) -> PushPlan {
    let mut plan = PushPlan::default();
    let mut y = PUSH_GRID_TOP;
    for group in &input.board.groups {
        let count = group.hints.len();
        if count == 0 {
            continue;
        }
        let top = y;
        y += PUSH_LEVEL_H;
        let first = plan.cards.len();
        for index in 0..count {
            let col = index % PUSH_CARD_COLS;
            let row = index / PUSH_CARD_COLS;
            plan.cards.push((
                PUSH_PAD_L + col as f32 * PUSH_CARD_STEP_X,
                y + row as f32 * PUSH_CARD_STEP_Y,
            ));
        }
        let rows = count.div_ceil(PUSH_CARD_COLS);
        y += rows as f32 * PUSH_CARD_STEP_Y + PUSH_GROUP_TAIL;
        plan.groups.push(PushGroupPlan { top, first, count });
    }
    plan.foot_y = y + PUSH_FOOT_GAP;
    plan.height = y + PUSH_BOTTOM_PAD;
    plan
}

fn push_group_slots(plan: PushGroupPlan) -> Vec<(f32, f32)> {
    let slots = (PUSH_CARD_COLS - plan.count % PUSH_CARD_COLS) % PUSH_CARD_COLS;
    if slots == 0 || plan.count == 0 {
        return Vec::new();
    }
    let rows = plan.count.div_ceil(PUSH_CARD_COLS);
    let first = PUSH_CARD_COLS - slots;
    let y = plan.top + PUSH_LEVEL_H + (rows as f32 - 1.0) * PUSH_CARD_STEP_Y;
    (0..slots)
        .map(|index| (PUSH_PAD_L + (first + index) as f32 * PUSH_CARD_STEP_X, y))
        .collect()
}

pub fn suggest_size(input: &SuggestInput) -> (i32, i32) {
    (SUGGEST_WIDTH, clamp_height(push_plan(input).height))
}

pub fn suggest_request(input: &SuggestInput) -> RenderRequest {
    let (width, height) = suggest_size(input);
    request_for(width, height)
}

pub fn suggest_card(input: SuggestInput) -> RenderCard {
    let request = suggest_request(&input);
    let label = format!(
        "{} {}x{}",
        CardKind::Suggest.label(),
        request.width,
        request.height
    );
    let cache_key = super::memo::fingerprint("suggest", &input);
    RenderCard::new(request, label, move |kit| paint_suggest(kit, &input)).with_cache_key(cache_key)
}

fn push_empty_slot(kit: &CanvasKit<'_>, x: f32, y: f32) {
    let box_ = area(x, y, PUSH_CARD_W, PUSH_CARD_H);
    sketch::clip_round(kit, box_, PUSH_CARD_R, || {
        sketch::fill_round(kit, box_, PUSH_CARD_R, fade(SLOT_BASE));
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

fn push_rank_arrow(hint: &SuggestHint) -> String {
    if !hint.has_score {
        return format!("NEW {PUSH_ARROW} B{}", hint.target_rank);
    }
    format!("#{} {PUSH_ARROW} B{}", hint.rank, hint.target_rank)
}

fn push_from_line(hint: &SuggestHint) -> String {
    if !hint.has_score {
        return String::from("NO RECORD");
    }
    let base = format!("from {}", thousands(hint.score));
    let judge = hint_note_text(hint);
    match (judge.is_empty(), hint.notes.short) {
        (true, true) => format!("{base} · 大P"),
        (true, false) => base,
        (false, true) => format!("{base} {PUSH_ARROW} {judge} · 大P"),
        (false, false) => format!("{base} {PUSH_ARROW} {judge}"),
    }
}

fn push_delta_text(hint: &SuggestHint) -> String {
    if !hint.has_score {
        return String::from("NEW");
    }
    format!("+{}", thousands(hint.delta.max(0)))
}

fn push_card(kit: &CanvasKit<'_>, x: f32, y: f32, hint: &SuggestHint) {
    let card = area(x, y, PUSH_CARD_W, PUSH_CARD_H);
    sketch::clip_round(kit, card, PUSH_CARD_R, || {
        kit.fill_rect(card, fade(PUSH_CARD_WHITE));
        kit.fill_rect(
            area(
                x,
                y + PUSH_CARD_SPLIT_Y,
                PUSH_CARD_ICE_W,
                PUSH_CARD_H - PUSH_CARD_SPLIT_Y,
            ),
            fade(PUSH_CARD_ICE),
        );
        sketch::fill_polygon(
            kit,
            &[
                (x + PUSH_CARD_ICE_W, y + PUSH_CARD_SPLIT_Y),
                (x + PUSH_CARD_W, y + PUSH_CARD_SPLIT_Y),
                (x + PUSH_CARD_W, y + PUSH_CARD_H),
                (x + PUSH_CARD_LEAN_X, y + PUSH_CARD_H),
            ],
            fade(PUSH_ACCENT_BG),
        );
        if hint.pick {
            kit.fill_rect(
                area(
                    x,
                    y + PUSH_CARD_SPLIT_Y,
                    PUSH_PICK_BAR_W,
                    PUSH_CARD_H - PUSH_CARD_SPLIT_Y,
                ),
                fade(PUSH_ACCENT),
            );
        }
    });
    let thumb = area(
        x + PUSH_THUMB_X,
        y + PUSH_THUMB_Y,
        PUSH_THUMB_SIZE,
        PUSH_THUMB_SIZE,
    );
    if kit
        .draw_asset_cover(&thumb_key(&hint.song_id), thumb, PUSH_THUMB_R)
        .is_err()
    {
        sketch::fill_round(kit, thumb, PUSH_THUMB_R, PUSH_CARD_ICE);
    }
    widgets::grade_badge(
        kit,
        x + PUSH_GRADE_CX,
        y + PUSH_GRADE_CY,
        PUSH_GRADE_SZ,
        hint.has_score.then(|| Grade::from_score(hint.score)),
    );
    let title_style = font_display(PUSH_TITLE_SZ, PUSH_INK);
    if !hint.title.is_empty() {
        let text = sketch::ellipsize(kit, &title_style, &hint.title, PUSH_TITLE_W);
        sketch::text(
            kit,
            &title_style,
            &text,
            x + PUSH_TITLE_COL_X,
            y + PUSH_CARD_TITLE_BASE,
            PUSH_INK,
        );
    }
    let cons = if hint.constant > 0.0 {
        format!("{:.1}", hint.constant)
    } else {
        String::from("—")
    };
    sketch::text(
        kit,
        &font_display(PUSH_META_SZ, PUSH_ACCENT),
        &cons,
        x + PUSH_CONST_TEXT_X,
        y + PUSH_META_BASE,
        PUSH_ACCENT,
    );
    let rank_base = y + PUSH_META_BASE;
    if hint.pick {
        sketch::diamond(
            kit,
            x + PUSH_RANK_TEXT_X + 2.5 * PUSH_K * K,
            rank_base - 3.5 * PUSH_K * K,
            2.6 * PUSH_K * K,
            PUSH_ACCENT,
        );
        sketch::text(
            kit,
            &font_display(PUSH_META_SZ, PUSH_INK_SUB),
            &push_rank_arrow(hint),
            x + PUSH_RANK_TEXT_X + 6.0 * PUSH_K * K,
            rank_base,
            PUSH_INK_SUB,
        );
    } else {
        sketch::text(
            kit,
            &font_display(PUSH_META_SZ, PUSH_INK_SUB),
            &format!("· {}", push_rank_arrow(hint)),
            x + PUSH_RANK_TEXT_X,
            rank_base,
            PUSH_INK_SUB,
        );
    }
    sketch::text(
        kit,
        &font_score(PUSH_TARGET_SZ, PUSH_INK),
        &thousands(hint.target),
        x + PUSH_CARD_PAD_X,
        y + PUSH_TARGET_BASE,
        PUSH_INK,
    );
    sketch::text(
        kit,
        &font_text(PUSH_FROM_SZ, PUSH_INK_SUB),
        &push_from_line(hint),
        x + PUSH_CARD_PAD_X,
        y + PUSH_FROM_BASE,
        PUSH_INK_SUB,
    );
    sketch::text_center(
        kit,
        &font_score(PUSH_DELTA_SZ, PUSH_ACCENT),
        &push_delta_text(hint),
        x + PUSH_DELTA_CX,
        y + PUSH_DELTA_BASE,
        PUSH_ACCENT,
    );
}

pub fn paint_suggest(kit: &CanvasKit<'_>, input: &SuggestInput) {
    let plan = push_plan(input);
    let width = SUGGEST_WIDTH as f32;
    let height = suggest_size(input).1 as f32;
    paper_backdrop(kit, width, height);
    let art = if !input.backdrop.is_empty() {
        format!("jackets/{}.jpg", input.backdrop)
    } else {
        input
            .board
            .all_hints()
            .first()
            .map(|hint| format!("jackets/{}.jpg", hint.song_id))
            .unwrap_or_default()
    };
    ledger_art_wash(kit, &art, width, height);
    kit.fill_rect(area(0.0, 0.0, width, height), PUSH_BG_MASK);
    let ptt_text = match (input.ptt_before, input.ptt_after) {
        (Some(before), Some(after)) => format!("PTT {before:.3} → {after:.3}"),
        _ => String::from("PTT —"),
    };
    let condition = if input.subtitle.trim().is_empty() {
        String::from("ALL CHARTS")
    } else {
        input.subtitle.trim().to_string()
    };
    let rest = format!(
        "·  {}  ·  已记录 {} / {}  ·  可推 {} 曲 (Top {})",
        condition, input.board.pool, input.board.pool, input.board.pushable, input.board.hint_count
    );
    sketch::text(
        kit,
        &font_display(PUSH_TITLE_SIZE, PUSH_INK),
        PUSH_TITLE,
        PUSH_PAD_L,
        PUSH_TITLE_BASE,
        PUSH_INK,
    );
    let ptt_style = font_display(PUSH_SUB_SIZE, PUSH_ACCENT);
    let ptt_w = sketch::text(
        kit,
        &ptt_style,
        &ptt_text,
        PUSH_PAD_L,
        PUSH_SUB_BASE,
        PUSH_ACCENT,
    );
    sketch::text(
        kit,
        &font_text(PUSH_SUB_SIZE, PUSH_INK_SUB),
        &rest,
        PUSH_PAD_L + ptt_w + PUSH_SUB_GAP,
        PUSH_SUB_BASE,
        PUSH_INK_SUB,
    );
    let mut flat: Vec<&SuggestHint> = Vec::new();
    for group in &input.board.groups {
        flat.extend(group.hints.iter());
    }
    for (index, group) in input.board.groups.iter().enumerate() {
        let Some(group_plan) = plan.groups.get(index) else {
            break;
        };
        kit.fill_rect(
            area(
                PUSH_LEVEL_BAR_X,
                group_plan.top.round(),
                PUSH_LEVEL_BAR_W,
                PUSH_LEVEL_BAR_H,
            ),
            PUSH_ACCENT,
        );
        let heading = group.heading();
        sketch::text(
            kit,
            &font_display(PUSH_LEVEL_LABEL_SIZE, PUSH_INK),
            &heading,
            PUSH_LEVEL_LABEL_X,
            group_plan.top + PUSH_LEVEL_BASE,
            PUSH_INK,
        );
        sketch::text(
            kit,
            &font_text(PUSH_LEVEL_COUNT_SIZE, PUSH_INK_SUB),
            &format!("× {}", group_plan.count),
            PUSH_LEVEL_COUNT_X,
            group_plan.top + PUSH_LEVEL_BASE,
            PUSH_INK_SUB,
        );
        if index == 0 {
            let legend = "曲线拟合首选主推";
            let style = font_text(PUSH_LEVEL_COUNT_SIZE, PUSH_INK_SUB);
            sketch::text_right(
                kit,
                &style,
                legend,
                PUSH_RIGHT,
                group_plan.top + PUSH_LEVEL_BASE,
                PUSH_INK_SUB,
            );
            let gap = 5.0 * PUSH_K * K;
            let diamond_r = 2.6 * PUSH_K * K;
            let mut x = PUSH_RIGHT - sketch::text_width(kit, &style, legend) - gap;
            let cy = group_plan.top + PUSH_LEVEL_BASE - PUSH_LEVEL_COUNT_SIZE * 0.34;
            sketch::diamond(kit, x - diamond_r, cy, diamond_r, PUSH_ACCENT);
            x -= 2.0 * diamond_r;
            x -= gap;
            kit.fill_rect(
                area(
                    x - PUSH_PICK_BAR_W,
                    cy - PUSH_LEVEL_BAR_H / 2.0,
                    PUSH_PICK_BAR_W,
                    PUSH_LEVEL_BAR_H,
                ),
                PUSH_ACCENT,
            );
        }
        for item in 0..group_plan.count {
            let index = group_plan.first + item;
            let Some((x, y)) = plan.cards.get(index) else {
                break;
            };
            let Some(hint) = flat.get(index) else {
                break;
            };
            push_card(kit, *x, *y, hint);
        }
        for (x, y) in push_group_slots(*group_plan) {
            push_empty_slot(kit, x, y);
        }
    }
    if plan.cards.is_empty() {
        sketch::text(
            kit,
            &font_text(14.0 * K, PUSH_INK_SUB),
            "没有能推的谱面了 —— 进 b50 的都已经 PM，或者再推也抬不动 PTT",
            PUSH_PAD_L,
            PUSH_GRID_TOP,
            PUSH_INK_SUB,
        );
    }
    sketch::text(
        kit,
        &font_display(PUSH_FOOT_SZ, PUSH_INK_SUB),
        FOOT_SIGN.to_uppercase().as_str(),
        PUSH_PAD_L,
        plan.foot_y,
        PUSH_INK_SUB,
    );
    player_block(kit, &input.badge, PUSH_RIGHT, compact_center_y());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChapExpand {
    #[default]
    All,
    Selected,
    Collapsed,
}

impl ChapExpand {
    pub const ALL: [ChapExpand; 3] = [ChapExpand::All, ChapExpand::Selected, ChapExpand::Collapsed];

    pub fn as_str(self) -> &'static str {
        match self {
            ChapExpand::All | ChapExpand::Selected => "逐曲明细",
            ChapExpand::Collapsed => "",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChapSong {
    pub song_id: String,
    pub title: String,
    pub difficulty: Difficulty,
    pub level: String,
    pub constant: Option<f64>,
    pub jacket: String,
    pub score: i64,
    pub grade: Option<Grade>,
    pub ptt: f64,
    pub played: bool,
    pub perfect: i64,
    pub shiny_perfect: i64,
    pub near: i64,
    pub miss: i64,
}

impl ChapSong {
    pub fn new(
        song_id: impl Into<String>,
        title: impl Into<String>,
        difficulty: Difficulty,
    ) -> Self {
        Self {
            song_id: song_id.into(),
            title: title.into(),
            difficulty,
            level: String::new(),
            constant: None,
            jacket: String::new(),
            score: 0,
            grade: None,
            ptt: 0.0,
            played: false,
            perfect: 0,
            shiny_perfect: 0,
            near: 0,
            miss: 0,
        }
    }

    pub fn with_level(mut self, level: impl Into<String>) -> Self {
        self.level = level.into();
        self
    }

    pub fn with_constant(mut self, constant: f64) -> Self {
        self.constant = Some(constant);
        self
    }

    pub fn with_jacket(mut self, key: impl Into<String>) -> Self {
        self.jacket = key.into();
        self
    }

    pub fn with_score(mut self, score: i64) -> Self {
        self.score = score;
        self
    }

    pub fn with_grade(mut self, grade: Grade) -> Self {
        self.grade = Some(grade);
        self
    }

    pub fn with_ptt(mut self, ptt: f64) -> Self {
        self.ptt = ptt;
        self
    }

    pub fn with_played(mut self, played: bool) -> Self {
        self.played = played;
        self
    }

    pub fn with_judgement(
        mut self,
        perfect: i64,
        shiny_perfect: i64,
        near: i64,
        miss: i64,
    ) -> Self {
        self.perfect = perfect;
        self.shiny_perfect = shiny_perfect;
        self.near = near;
        self.miss = miss;
        self
    }

    fn jacket_key(&self) -> String {
        if !self.jacket.is_empty() {
            self.jacket.clone()
        } else {
            thumb_key(&self.song_id)
        }
    }

    fn const_text(&self) -> String {
        match self.constant {
            Some(value) if value > 0.0 => format!("{value:.1}"),
            _ => String::from("—"),
        }
    }

    fn score_text(&self) -> String {
        if self.played {
            thousands(self.score)
        } else {
            String::from("0'000'000")
        }
    }

    fn ptt_text(&self) -> String {
        if !self.played {
            return String::from("0.000");
        }
        if self.constant.is_none() || self.ptt <= 0.0 {
            return String::from("—");
        }
        format!("{:.3}", self.ptt)
    }

    pub fn score_label(&self) -> String {
        self.score_text()
    }

    pub fn ptt_label(&self) -> String {
        self.ptt_text()
    }
}

#[derive(Debug, Clone)]
pub struct ChapPack {
    pub pack_id: String,
    pub name: String,
    pub song_count: usize,
    pub owned: usize,
    pub cleared: usize,
    pub full_combo: usize,
    pub pure_memory: usize,
    pub jackets: Vec<String>,
    pub charts: Vec<ChapSong>,
    pub section: String,
    pub avg_score: f64,
    pub avg_ptt: f64,
}

impl ChapPack {
    pub fn new(pack_id: impl Into<String>, name: impl Into<String>, song_count: usize) -> Self {
        Self {
            pack_id: pack_id.into(),
            name: name.into(),
            song_count,
            owned: 0,
            cleared: 0,
            full_combo: 0,
            pure_memory: 0,
            jackets: Vec::new(),
            charts: Vec::new(),
            section: String::new(),
            avg_score: 0.0,
            avg_ptt: 0.0,
        }
    }

    pub fn with_owned(mut self, owned: usize) -> Self {
        self.owned = owned;
        self
    }

    pub fn with_cleared(mut self, cleared: usize) -> Self {
        self.cleared = cleared;
        self
    }

    pub fn with_full_combo(mut self, full_combo: usize) -> Self {
        self.full_combo = full_combo;
        self
    }

    pub fn with_pure_memory(mut self, pure_memory: usize) -> Self {
        self.pure_memory = pure_memory;
        self
    }

    pub fn with_jacket(mut self, key: impl Into<String>) -> Self {
        let key = key.into();
        if !key.is_empty() {
            self.jackets.push(key);
        }
        self
    }

    pub fn with_chart(mut self, chart: ChapSong) -> Self {
        self.charts.push(chart);
        self
    }

    pub fn with_charts(mut self, charts: Vec<ChapSong>) -> Self {
        self.charts = charts;
        self
    }

    pub fn with_section(mut self, section: impl Into<String>) -> Self {
        self.section = section.into();
        self
    }

    pub fn with_average(mut self, avg_score: f64, avg_ptt: f64) -> Self {
        self.avg_score = avg_score.max(0.0);
        self.avg_ptt = avg_ptt.max(0.0);
        self
    }

    pub fn owned_count(&self) -> usize {
        self.owned.min(self.song_count)
    }

    pub fn clear_count(&self) -> usize {
        self.cleared.min(self.song_count)
    }

    pub fn pm_count(&self) -> usize {
        self.pure_memory.min(self.song_count)
    }

    pub fn fc_count(&self) -> usize {
        self.full_combo.min(self.song_count)
    }

    pub fn fail_count(&self) -> usize {
        self.owned_count().saturating_sub(self.clear_count())
    }

    pub fn missing_count(&self) -> usize {
        self.song_count.saturating_sub(self.owned_count())
    }

    fn pack_key(&self) -> String {
        if self.pack_id.is_empty() {
            self.jackets.first().cloned().unwrap_or_default()
        } else {
            format!("packs/{}.jpg", self.pack_id)
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChapTotals {
    pub played: usize,
    pub total: usize,
}

#[derive(Debug, Clone)]
pub struct ChapInput {
    pub title: String,
    pub subtitle: String,
    pub theme: Theme,
    pub badge: PlayerBadge,
    pub packs: Vec<ChapPack>,
    pub expand: ChapExpand,
    pub focus: Option<usize>,
    pub totals: Option<ChapTotals>,
    pub note: String,
}

impl ChapInput {
    pub fn new(badge: PlayerBadge) -> Self {
        Self {
            title: CHAP_TITLE.to_string(),
            subtitle: String::new(),
            theme: Theme::DEFAULT,
            badge,
            packs: Vec::new(),
            expand: ChapExpand::default(),
            focus: None,
            totals: None,
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

    pub fn with_pack(mut self, pack: ChapPack) -> Self {
        self.packs.push(pack);
        self
    }

    pub fn with_packs(mut self, packs: Vec<ChapPack>) -> Self {
        self.packs = packs;
        self
    }

    pub fn with_totals(mut self, played: usize, total: usize) -> Self {
        self.totals = Some(ChapTotals { played, total });
        self
    }

    pub fn with_expand(mut self, expand: ChapExpand) -> Self {
        self.expand = expand;
        self
    }

    pub fn with_focus(mut self, focus: usize) -> Self {
        self.focus = Some(focus);
        self
    }
}

impl Default for ChapInput {
    fn default() -> Self {
        Self::new(PlayerBadge::new("Afkeru")).with_subtitle("按曲包统计")
    }
}

pub fn chap_expanded(input: &ChapInput) -> Vec<bool> {
    match input.expand {
        ChapExpand::All => vec![true; input.packs.len()],
        ChapExpand::Collapsed => vec![false; input.packs.len()],
        ChapExpand::Selected => {
            let target = input
                .focus
                .filter(|index| *index < input.packs.len())
                .unwrap_or(0);
            (0..input.packs.len())
                .map(|index| index == target)
                .collect()
        }
    }
}

type GridPoint = (f32, f32);

fn chap_grid(
    top: f32,
    count: usize,
    cols: usize,
    step_x: f32,
    step_y: f32,
) -> (Vec<GridPoint>, Vec<GridPoint>, usize) {
    let mut cards = Vec::with_capacity(count);
    for index in 0..count {
        let col = index % cols;
        let row = index / cols;
        cards.push((CHAP_PAD_L + col as f32 * step_x, top + row as f32 * step_y));
    }
    let rows = count.div_ceil(cols);
    let mut slots = Vec::new();
    let extra = (cols - count % cols) % cols;
    if extra > 0 {
        let first = cols - extra;
        let y = top + (rows as f32 - 1.0) * step_y;
        for index in 0..extra {
            slots.push((CHAP_PAD_L + (first + index) as f32 * step_x, y));
        }
    }
    (cards, slots, rows)
}

#[derive(Debug, Clone, Default)]
struct ChapSectionPlan {
    name: String,
    note: String,
    top: f32,
    cards: Vec<(f32, f32)>,
    slots: Vec<(f32, f32)>,
    packs: Vec<usize>,
    charts: Vec<ChapSong>,
}

#[derive(Debug, Clone, Default)]
struct ChapPlan {
    sections: Vec<ChapSectionPlan>,
    foot_y: f32,
    height: f32,
}

fn chapter_groups(input: &ChapInput) -> Vec<(String, Vec<ChapSong>)> {
    let flags = chap_expanded(input);
    let mut order: Vec<String> = Vec::new();
    let mut buckets: Vec<(String, Vec<ChapSong>)> = Vec::new();
    for (index, pack) in input.packs.iter().enumerate() {
        if !flags.get(index).copied().unwrap_or(false) {
            continue;
        }
        for song in &pack.charts {
            let label = song.difficulty.as_str().to_string();
            if let Some(slot) = buckets.iter_mut().find(|(name, _)| *name == label) {
                slot.1.push(song.clone());
            } else {
                order.push(label.clone());
                buckets.push((label, vec![song.clone()]));
            }
        }
    }
    let rank = |label: &str| -> i32 {
        match label {
            "BYD" | "INS" => 5,
            "ETR" => 4,
            "FTR" => 3,
            "PRS" => 2,
            "PST" => 1,
            _ => 0,
        }
    };
    order.sort_by(|left, right| rank(right).cmp(&rank(left)).then_with(|| left.cmp(right)));
    for (label, songs) in &mut buckets {
        songs.sort_by(|left, right| {
            right
                .constant
                .unwrap_or(0.0)
                .partial_cmp(&left.constant.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let _ = label;
    }
    let mut out: Vec<(String, Vec<ChapSong>)> = Vec::new();
    for label in order {
        if let Some(position) = buckets.iter().position(|(name, _)| *name == label) {
            out.push(buckets.remove(position));
        }
    }
    out
}

fn chap_pack_groups(input: &ChapInput) -> Vec<(String, Vec<usize>)> {
    if input.focus.is_some() {
        return Vec::new();
    }
    let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
    for (index, pack) in input.packs.iter().enumerate() {
        let label = pack.section.trim().to_string();
        match groups.iter_mut().find(|(name, _)| *name == label) {
            Some((_, indices)) => indices.push(index),
            None => groups.push((label, vec![index])),
        }
    }
    groups
}

fn chap_section_note(input: &ChapInput, indices: &[usize], groups: usize) -> String {
    let (played, total) = match (input.totals, groups) {
        (Some(totals), 1) => (totals.played, totals.total),
        _ => indices
            .iter()
            .filter_map(|index| input.packs.get(*index))
            .fold((0usize, 0usize), |(played, total), pack| {
                (played + pack.owned_count(), total + pack.song_count)
            }),
    };
    format!("{} 只包 · 已玩 {played} / {total}", indices.len())
}

fn chap_plan(input: &ChapInput) -> ChapPlan {
    let mut plan = ChapPlan::default();
    let mut y = CHAP_GRID_TOP;
    let groups = chap_pack_groups(input);
    for (position, (label, indices)) in groups.iter().enumerate() {
        if position > 0 {
            y += CHAP_SECTION_GAP;
        }
        let (cards, slots, rows) = chap_grid(
            y + CHAP_HEAD_H,
            indices.len(),
            PACK_COLS,
            PACK_STEP_X,
            PACK_STEP_Y,
        );
        let name = match (label.is_empty(), input.title.trim().is_empty()) {
            (false, _) => label.clone(),
            (true, false) => input.title.clone(),
            (true, true) => CHAP_TITLE.to_string(),
        };
        plan.sections.push(ChapSectionPlan {
            name,
            note: chap_section_note(input, indices, groups.len()),
            top: y,
            cards,
            slots,
            packs: indices.clone(),
            charts: Vec::new(),
        });
        y += CHAP_HEAD_H + (rows as f32 - 1.0) * PACK_STEP_Y + PACK_H;
    }
    for (label, songs) in chapter_groups(input) {
        y += CHAP_SECTION_GAP;
        let (cards, slots, rows) = chap_grid(
            y + CHAP_HEAD_H,
            songs.len(),
            CHART_COLS,
            CHART_STEP_X,
            CHART_STEP_Y,
        );
        plan.sections.push(ChapSectionPlan {
            name: label,
            note: String::new(),
            top: y,
            cards,
            slots,
            packs: Vec::new(),
            charts: songs,
        });
        y += CHAP_HEAD_H + (rows as f32 - 1.0) * CHART_STEP_Y + CHART_H;
    }
    plan.foot_y = y + CHAP_FOOT_GAP;
    plan.height = plan.foot_y + CHAP_BOTTOM_PAD - CHAP_FOOT_GAP;
    plan
}

pub fn chap_size(input: &ChapInput) -> (i32, i32) {
    (CHAP_WIDTH, clamp_height(chap_plan(input).height))
}

pub fn chap_request(input: &ChapInput) -> RenderRequest {
    let (width, height) = chap_size(input);
    request_for(width, height).with_scale(1.5)
}

pub fn chap_card(input: ChapInput) -> RenderCard {
    let request = chap_request(&input);
    let label = format!(
        "{} {}x{}",
        CardKind::Chap.label(),
        request.width,
        request.height
    );
    let cache_key = super::memo::fingerprint("chap", &input);
    RenderCard::new(request, label, move |kit| paint_chap(kit, &input)).with_cache_key(cache_key)
}

pub fn chap_expand_summary(input: &ChapInput) -> String {
    let (played, total) = match input.totals {
        Some(totals) => (totals.played, totals.total),
        None => (
            input.packs.iter().map(|pack| pack.owned).sum(),
            input.packs.iter().map(|pack| pack.song_count).sum(),
        ),
    };
    format!("{} 包 · 已记录 {played}/{total}", input.packs.len())
}

fn pack_counts(pack: &ChapPack) -> (usize, usize, usize, usize, usize) {
    let total = pack.song_count;
    let pm = pack.pure_memory.min(total);
    let fc = pack.full_combo.min(total).max(pm);
    let cleared = pack.cleared.min(total).max(fc);
    let played = pack.owned.min(total).max(cleared);
    (cleared - fc, fc - pm, pm, played - cleared, total - played)
}

fn pack_count_ink(pack: &ChapPack) -> Color {
    if pack.song_count == 0 || pack.owned == 0 {
        return CHAP_INK_DIM;
    }
    if pack.owned >= pack.song_count {
        return CHAP_INK_MINT;
    }
    CHAP_INK_BLUE
}

fn pack_card(kit: &CanvasKit<'_>, x: f32, y: f32, pack: &ChapPack) {
    let card = area(x, y, PACK_W, PACK_H);
    sketch::fill_round(kit, card, PACK_R, fade(PUSH_CARD_WHITE));
    let cover = area(
        x + PACK_THUMB_X,
        y + PACK_THUMB_Y,
        PACK_THUMB_SIZE,
        PACK_THUMB_SIZE,
    );
    if kit
        .draw_asset_cover(&pack.pack_key(), cover, PACK_THUMB_R)
        .is_err()
    {
        sketch::fill_round(kit, cover, PACK_THUMB_R, PUSH_CARD_ICE);
    }
    let right = x + PACK_TEXT_X + PACK_TEXT_W;
    let denom = format!("/ {}", pack.song_count);
    let denom_style = font_display(PACK_DENOM_SIZE, CHAP_INK_DIM);
    let denom_w = sketch::text_right(
        kit,
        &denom_style,
        &denom,
        right,
        y + PACK_COUNT_BASE,
        CHAP_INK_DIM,
    );
    let num_style = font_number(PACK_COUNT_SIZE, pack_count_ink(pack));
    let num_text = pack.owned_count().to_string();
    let num_w = sketch::text_right(
        kit,
        &num_style,
        &num_text,
        right - denom_w,
        y + PACK_COUNT_BASE,
        pack_count_ink(pack),
    );
    let name_w = PACK_TEXT_W - denom_w - num_w - 12.0 * K;
    let name_style = font_display(PACK_NAME_SIZE, CHAP_INK_NAME);
    let name = sketch::ellipsize(kit, &name_style, &pack.name, name_w);
    sketch::text(
        kit,
        &name_style,
        &name,
        x + PACK_TEXT_X,
        y + PACK_NAME_BASE,
        CHAP_INK_NAME,
    );
    let (clear_only, fc_only, pm, fail, _) = pack_counts(pack);
    let items = [
        ("PM", pm, PUSH_BAR_PM),
        ("FR", fc_only, PUSH_BAR_FC),
        ("通关", clear_only, PUSH_BAR_CLEARED),
        ("失败", fail, PUSH_BAR_FAIL),
    ];
    let dot = (6.0 * CHAP_SCALE).round();
    let gap = 4.0 * CHAP_SCALE;
    let mut cx = x + PACK_TEXT_X;
    for (index, (label, count, ink)) in items.iter().enumerate() {
        let (dot_ink, text_ink) = if *count == 0 {
            (PUSH_INK_MUTED, PUSH_INK_MUTED)
        } else {
            (*ink, CHAP_INK_STAT)
        };
        if index > 0 {
            cx += 10.0 * CHAP_SCALE;
        }
        let cy = y + PACK_FAN_BASE - PACK_FAN_SIZE * 0.36;
        kit.fill_rect(
            area(cx.round(), (cy - dot / 2.0).round(), dot, dot),
            dot_ink,
        );
        let style = font_text(PACK_FAN_SIZE, text_ink);
        let text = format!("{label} {count}");
        sketch::text(
            kit,
            &style,
            &text,
            cx + dot + gap,
            y + PACK_FAN_BASE,
            text_ink,
        );
        cx = cx + dot + gap + sketch::text_width(kit, &style, &text);
    }
    if pack.song_count > 0 && pack.owned >= pack.song_count {
        sketch::text_right(
            kit,
            &font_display(PACK_MAX_SIZE, CHAP_INK_MAX),
            "100% MAX",
            right,
            y + PACK_FAN_BASE,
            CHAP_INK_MAX,
        );
    } else if pack.avg_score > 0.0 {
        let style = font_display(PACK_MAX_SIZE, CHAP_INK_DIM);
        let average = thousands(pack.avg_score.round() as i64);
        let text = match pack.avg_ptt > 0.0 {
            true => format!("均 {average} · PTT {:.2}", pack.avg_ptt),
            false => format!("均 {average}"),
        };
        let width = sketch::text_width(kit, &style, &text);
        if cx + 12.0 * CHAP_SCALE + width <= right {
            sketch::text_right(
                kit,
                &style,
                &text,
                right,
                y + PACK_FAN_BASE,
                CHAP_INK_DIM,
            );
        }
    }
    let bar = area(x + PACK_TEXT_X, y + PACK_BAR_Y, PACK_TEXT_W, PACK_BAR_H);
    stack(
        kit,
        bar,
        &[
            (clear_only as f32, PUSH_BAR_CLEARED),
            (fc_only as f32, PUSH_BAR_FC),
            (pm as f32, PUSH_BAR_PM),
            (fail as f32, PUSH_BAR_FAIL),
            (
                (pack.song_count.saturating_sub(pack.owned)) as f32,
                PUSH_BAR_TRACK,
            ),
        ],
    );
}
fn chapter_judge_text(song: &ChapSong) -> String {
    if !song.played {
        return String::from("P0 · F0 · L0");
    }
    let perfect = song.perfect.max(0);
    let mut pure = format!("P{perfect}");
    let shiny = song.shiny_perfect.max(0).min(perfect);
    if shiny > 0 {
        pure.push_str(&format!("(+{shiny})"));
    }
    format!("{pure} · F{} · L{}", song.near.max(0), song.miss.max(0))
}

fn chap_slot(
    kit: &CanvasKit<'_>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    ice: Option<(f32, f32)>,
) {
    let box_ = area(x, y, w, h);
    sketch::clip_round(kit, box_, radius, || {
        sketch::fill_round(kit, box_, radius, CHAP_SLOT_BG);
        hatch(kit, box_, 15, 3, CHAP_SLOT_INK);
        if let Some((ice_x, ice_lean)) = ice {
            sketch::fill_polygon(
                kit,
                &[
                    (x + ice_x, y + CHART_SPLIT_Y),
                    (x + w, y + CHART_SPLIT_Y),
                    (x + w, y + h),
                    (x + ice_x - ice_lean, y + h),
                ],
                CHAP_SLOT_ICE,
            );
        }
    });
}

fn chart_card(kit: &CanvasKit<'_>, x: f32, y: f32, song: &ChapSong) {
    let card = area(x, y, CHART_W, CHART_H);
    sketch::clip_round(kit, card, CHART_R, || {
        kit.fill_rect(card, fade(PUSH_CARD_WHITE));
        kit.fill_rect(
            area(x, y + CHART_SPLIT_Y, CHART_ICE_X, CHART_H - CHART_SPLIT_Y),
            fade(PUSH_CARD_ICE),
        );
        sketch::fill_polygon(
            kit,
            &[
                (x + CHART_ICE_X, y + CHART_SPLIT_Y),
                (x + CHART_W, y + CHART_SPLIT_Y),
                (x + CHART_W, y + CHART_H),
                (x + CHART_ICE_X - CHART_ICE_LEAN, y + CHART_H),
            ],
            fade(PUSH_ACCENT_BG),
        );
    });
    let thumb = area(
        x + CHART_THUMB_X,
        y + CHART_THUMB_Y,
        CHART_THUMB_SIZE,
        CHART_THUMB_SIZE,
    );
    if kit
        .draw_asset_cover(&song.jacket_key(), thumb, CHART_THUMB_R)
        .is_err()
    {
        sketch::fill_round(kit, thumb, CHART_THUMB_R, PUSH_CARD_ICE);
    }
    let title_style = font_display(CHART_TITLE_SIZE, PUSH_INK);
    if !song.title.is_empty() {
        let text = sketch::ellipsize(kit, &title_style, &song.title, CHART_TITLE_W);
        sketch::text(
            kit,
            &title_style,
            &text,
            x + CHART_TITLE_X,
            y + CHART_TITLE_BASE,
            PUSH_INK,
        );
    }
    let diff = song.difficulty.as_str().to_string();
    let meta_style = font_display(CHART_META_SIZE, PUSH_INK);
    let diff_w = sketch::text(
        kit,
        &meta_style,
        &diff,
        x + CHART_TITLE_X,
        y + CHART_META_BASE,
        PUSH_INK,
    );
    sketch::text(
        kit,
        &font_number(CHART_META_SIZE, PUSH_ACCENT),
        &format!("· {}", song.const_text()),
        x + CHART_TITLE_X + diff_w + 6.0 * K,
        y + CHART_META_BASE,
        PUSH_ACCENT,
    );
    let cx = x + CHART_RIGHT_CX;
    let cy = y + CHART_GRADE_CY;
    if !song.played {
        sketch::text_center(
            kit,
            &font_display(CHART_IDLE_SIZE, CHAP_INK_DIM),
            "未记录",
            cx,
            cy + CHART_IDLE_SIZE * 0.36,
            CHAP_INK_DIM,
        );
    } else {
        widgets::grade_badge(kit, cx, cy, CHART_GRADE_H, song.grade);
    }
    sketch::text(
        kit,
        &font_number(CHART_SCORE_SIZE, PUSH_INK),
        &song.score_text(),
        x + CHART_SCORE_X,
        y + CHART_SCORE_BASE,
        PUSH_INK,
    );
    let judge_style = font_text(CHART_JUDGE_SIZE, PUSH_INK_SUB);
    let judge = chapter_judge_text(song);
    let judge_w = CHART_ICE_X - CHART_SCORE_X * 2.0;
    let judge = sketch::ellipsize(kit, &judge_style, &judge, judge_w);
    sketch::text(
        kit,
        &judge_style,
        &judge,
        x + CHART_SCORE_X,
        y + CHART_JUDGE_BASE,
        PUSH_INK_SUB,
    );
    sketch::text_center(
        kit,
        &font_number(CHART_PTT_SIZE, PUSH_ACCENT),
        &song.ptt_text(),
        x + CHART_PTT_CX,
        y + CHART_PTT_BASE,
        PUSH_ACCENT,
    );
}

pub fn paint_chap(kit: &CanvasKit<'_>, input: &ChapInput) {
    let plan = chap_plan(input);
    let width = CHAP_WIDTH as f32;
    let height = chap_size(input).1 as f32;
    paper_backdrop(kit, width, height);
    let art = input
        .packs
        .first()
        .map(ChapPack::pack_key)
        .unwrap_or_default();
    ledger_art_wash(kit, &art, width, height);
    kit.fill_rect(area(0.0, 0.0, width, height), PUSH_BG_MASK);
    let head = if input.title.trim().is_empty() {
        CHAP_TITLE
    } else {
        input.title.trim()
    };
    let mut avail = CHAP_RIGHT - CHAP_PAD_L;
    let (plate_w, plate_x) = plate_box(kit, &input.badge, CHAP_RIGHT);
    if plate_w > 0.0 {
        avail = avail.min(plate_x - LEDGER_HEAD_BAR_GAP - CHAP_PAD_L);
    }
    micro_head(
        kit,
        MicroHead {
            title: head,
            note: &input.subtitle,
            x: CHAP_PAD_L,
            title_base: CHAP_TITLE_BASE,
            sub_base: CHAP_SUB_BASE,
            title_size: CHAP_TITLE_SIZE,
            sub_size: CHAP_SUB_SIZE,
            avail: avail.max(0.0),
        },
    );
    for section in &plan.sections {
        kit.fill_rect(
            area(CHAP_PAD_L, section.top.round(), CHAP_BAR_W, CHAP_BAR_H),
            PUSH_ACCENT,
        );
        sketch::text(
            kit,
            &font_display(CHAP_NAME_SIZE, PUSH_INK),
            &section.name,
            CHAP_NAME_X,
            section.top + CHAP_HEAD_BASE,
            PUSH_INK,
        );
        if !section.note.is_empty() {
            sketch::text_right(
                kit,
                &font_text(CHAP_FOOT_SIZE, PUSH_INK_SUB),
                &section.note,
                CHAP_RIGHT,
                section.top + CHAP_HEAD_BASE,
                PUSH_INK_SUB,
            );
        }
        for (index, (x, y)) in section.cards.iter().enumerate() {
            if let Some(pack) = section
                .packs
                .get(index)
                .and_then(|position| input.packs.get(*position))
            {
                pack_card(kit, *x, *y, pack);
            } else if let Some(song) = section.charts.get(index) {
                chart_card(kit, *x, *y, song);
            }
        }
        for (x, y) in &section.slots {
            if section.packs.is_empty() {
                chap_slot(
                    kit,
                    *x,
                    *y,
                    CHART_W,
                    CHART_H,
                    CHART_R,
                    Some((CHART_ICE_X, CHART_ICE_LEAN)),
                );
            } else {
                chap_slot(kit, *x, *y, PACK_W, PACK_H, PACK_R, None);
            }
        }
    }
    if !input.note.trim().is_empty() {
        sketch::text_right(
            kit,
            &font_text(CHAP_FOOT_SIZE, PUSH_INK_SUB),
            input.note.trim(),
            CHAP_RIGHT,
            plan.foot_y,
            PUSH_INK_SUB,
        );
    }
    sketch::text(
        kit,
        &font_display(CHAP_FOOT_SIZE, PUSH_INK_SUB),
        FOOT_SIGN.to_uppercase().as_str(),
        CHAP_PAD_L,
        plan.foot_y,
        PUSH_INK_SUB,
    );
    player::paint(
        kit,
        &block_of(&input.badge, CHAP_RIGHT, CHAP_PLAYER_CY).with_scale(CHAP_SCALE),
    );
}

#[derive(Debug, Clone)]
pub struct TrendPoint {
    pub label: String,
    pub value: f64,
    pub score: i64,
    pub date: String,
    pub time: String,
    pub minute: i64,
}

impl TrendPoint {
    pub fn new(label: impl Into<String>, value: f64) -> Self {
        Self {
            label: label.into(),
            value,
            score: 0,
            date: String::new(),
            time: String::new(),
            minute: 0,
        }
    }

    pub fn with_clock(
        mut self,
        date: impl Into<String>,
        time: impl Into<String>,
        minute: i64,
    ) -> Self {
        self.date = date.into();
        self.time = time.into();
        self.minute = minute;
        self
    }

    fn stamped(&self) -> bool {
        !self.date.trim().is_empty()
    }

    pub fn with_score(mut self, score: i64) -> Self {
        self.score = score;
        self
    }
}

#[derive(Debug, Clone)]
pub struct TrendInput {
    pub title: String,
    pub subtitle: String,
    pub theme: Theme,
    pub badge: PlayerBadge,
    pub points: Vec<TrendPoint>,
    pub note: String,
    pub backdrop: String,
    pub days: Option<u32>,
    pub rank: Option<i64>,
    pub rank_total: Option<i64>,
    pub peak: Option<f64>,
    pub recent7: Option<f64>,
    pub recent30: Option<f64>,
    pub records: Option<usize>,
    pub frame: Option<i32>,
    pub bounds: Vec<f64>,
}

impl TrendInput {
    pub fn new(badge: PlayerBadge) -> Self {
        Self {
            title: CardKind::Trend.label().to_string(),
            subtitle: String::new(),
            theme: Theme::DEFAULT,
            badge,
            points: Vec::new(),
            note: String::new(),
            backdrop: String::new(),
            days: None,
            rank: None,
            rank_total: None,
            peak: None,
            recent7: None,
            recent30: None,
            records: None,
            frame: None,
            bounds: Vec::new(),
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

    pub fn with_point(mut self, point: TrendPoint) -> Self {
        self.points.push(point);
        self
    }

    pub fn with_points(mut self, points: Vec<TrendPoint>) -> Self {
        self.points = points;
        self
    }

    pub fn with_days(mut self, days: u32) -> Self {
        if days > 0 {
            self.days = Some(days);
        }
        self
    }

    pub fn with_rank(mut self, rank: i64, total: i64) -> Self {
        if rank > 0 {
            self.rank = Some(rank);
        }
        if total > 0 {
            self.rank_total = Some(total);
        }
        self
    }

    pub fn with_curve_stats(
        mut self,
        peak: Option<f64>,
        recent7: Option<f64>,
        recent30: Option<f64>,
    ) -> Self {
        self.peak = peak.filter(|value| *value > 0.0);
        self.recent7 = recent7;
        self.recent30 = recent30;
        self
    }

    pub fn with_records(mut self, records: usize) -> Self {
        if records > 0 {
            self.records = Some(records);
        }
        self
    }

    pub fn with_frame(mut self, frame: i32) -> Self {
        if frame >= 0 {
            self.frame = Some(frame);
        }
        self
    }

    pub fn with_bounds(mut self, bounds: &[f64]) -> Self {
        self.bounds = bounds.to_vec();
        self
    }

    fn art_key(&self) -> String {
        if !self.backdrop.is_empty() {
            return format!("jackets/{}.jpg", self.backdrop);
        }
        let first = self.points.first().map(|p| p.label.as_str()).unwrap_or("");
        let last = self.points.last();
        let value = format!("{:.3}", last.map(|p| p.value).unwrap_or(0.0));
        let parts = [
            "trend",
            first,
            last.map(|p| p.label.as_str()).unwrap_or(""),
            value.as_str(),
        ];
        jacket_pool::ledger_art(&parts)
            .map(|id| format!("jackets/{id}.jpg"))
            .unwrap_or_default()
    }
}

impl Default for TrendInput {
    fn default() -> Self {
        Self::new(PlayerBadge::new("Afkeru")).with_subtitle("潜力值随时间的变化")
    }
}

#[derive(Debug, Clone, Default)]
struct CurveScale {
    lo: f64,
    hi: f64,
    data_lo: f64,
    step: f64,
    ticks: Vec<f64>,
    knee: bool,
    knee_ptt: f64,
}

fn curve_clean(value: f64) -> f64 {
    if value.abs() < 1e-9 {
        return 0.0;
    }
    (value * 1e6).round() / 1e6
}

fn curve_tick_step(lo: f64, hi: f64) -> f64 {
    let mut best = CURVE_TICK_LADDER[CURVE_TICK_LADDER.len() - 1];
    let mut best_score = f64::INFINITY;
    for step in CURVE_TICK_LADDER {
        let count = ((hi - (lo / step).ceil() * step) / step + 1e-9).floor() + 1.0;
        if (5.0..=7.0).contains(&count) {
            return step;
        }
        let score = (count - 6.0).abs();
        if score < best_score {
            best = step;
            best_score = score;
        }
    }
    best
}

fn curve_ticks(scale: &CurveScale) -> (Vec<f64>, f64) {
    if scale.hi <= scale.lo {
        return (Vec::new(), 0.0);
    }
    let split = scale.knee && scale.knee_ptt > scale.lo && scale.knee_ptt < scale.hi;
    let from = if split { scale.knee_ptt } else { scale.lo };
    let step = curve_tick_step(from, scale.hi);
    let mut out: Vec<f64> = Vec::with_capacity(8);
    let mut index = 0.0;
    loop {
        let value = curve_clean((from / step - 1e-9).ceil() * step + index * step);
        if value > scale.hi + 1e-9 {
            break;
        }
        out.push(value);
        index += 1.0;
    }
    if split {
        out.push(curve_clean(scale.knee_ptt));
        out.push(curve_clean(scale.data_lo));
    }
    out.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let mut ticks: Vec<f64> = Vec::with_capacity(out.len());
    for value in out {
        if let Some(last) = ticks.last() {
            if (value - last).abs() < 1e-9 {
                continue;
            }
        }
        ticks.push(value);
    }
    (ticks, step)
}

fn curve_tick_text(value: f64, _step: f64) -> String {
    format!("{:.3}", curve_clean(value))
}

fn curve_knee(points: &[TrendPoint]) -> Option<f64> {
    let count = points.len();
    if count < 5 {
        return None;
    }
    let mut lo = points[0].value;
    let mut hi = points[0].value;
    for point in points {
        lo = lo.min(point.value);
        hi = hi.max(point.value);
    }
    let from = count / 5;
    let mut recent = points[from].value;
    for point in &points[from..] {
        recent = recent.min(point.value);
    }
    if recent <= lo + 1e-9 {
        return None;
    }
    if hi - lo > 2.5 * (hi - recent).max(0.02) {
        return Some(recent);
    }
    None
}

fn build_curve_scale(points: &[TrendPoint]) -> CurveScale {
    if points.is_empty() {
        return CurveScale::default();
    }
    let mut lo = points[0].value;
    let mut hi = points[0].value;
    for point in points {
        lo = lo.min(point.value);
        hi = hi.max(point.value);
    }
    let mut scale = CurveScale {
        data_lo: lo,
        ..CurveScale::default()
    };
    let delta = hi - lo;
    if delta > 0.0 {
        let eff = delta.max(CURVE_MIN_SPAN);
        scale.lo = lo - CURVE_PAD_LOW * eff;
        scale.hi = hi + CURVE_PAD_HIGH * eff;
        if let Some(knee) = curve_knee(points) {
            scale.knee = true;
            scale.knee_ptt = knee;
            scale.hi = hi + CURVE_PAD_HIGH * (hi - knee).max(CURVE_MIN_SPAN);
            scale.lo = lo - CURVE_PAD_LOW * (knee - lo).max(CURVE_MIN_SPAN);
        }
    } else {
        scale.lo = lo - CURVE_FLAT_SPAN;
        scale.hi = lo + CURVE_FLAT_SPAN;
    }
    let (ticks, step) = curve_ticks(&scale);
    scale.ticks = ticks;
    scale.step = step;
    scale
}

#[derive(Debug, Clone, Default)]
struct TrendPlan {
    height: f32,
    last_bottom: f32,
    block: Block,
    plot: Rect,
}

fn trend_plan() -> TrendPlan {
    let grid_top = player_bottom(compact_center_y()) + COMPACT_GRID_GAP;
    let mut plan = TrendPlan::default();
    let mut flow = Flow::new(grid_top);
    let mut block = Block::default();
    let top = flow.open(&mut block);
    plan.plot = area(
        MARGIN_X,
        top + TREND_BLOCK_GAP,
        LEDGER_W - 2.0 * MARGIN_X,
        TREND_CURVE_H,
    );
    flow.advance(TREND_CURVE_H);
    plan.block = block;
    plan.last_bottom = flow.y;
    plan.height = plan.last_bottom + FOOTER_H;
    plan
}

pub fn trend_size(input: &TrendInput) -> (i32, i32) {
    let _ = input;
    (TREND_WIDTH, clamp_height(trend_plan().height))
}

pub fn trend_request(input: &TrendInput) -> RenderRequest {
    let (width, height) = trend_size(input);
    request_for(width, height).with_scale(1.5)
}

pub fn trend_card(input: TrendInput) -> RenderCard {
    let request = trend_request(&input);
    let label = format!(
        "{} {}x{}",
        CardKind::Trend.label(),
        request.width,
        request.height
    );
    let cache_key = super::memo::fingerprint("trend", &input);
    RenderCard::new(request, label, move |kit| paint_trend(kit, &input)).with_cache_key(cache_key)
}

const TREND_WARP_BASE: f32 = 1.0;
const TREND_WARP_K: f32 = 1.0;
const TREND_WARP_CAP_H: f32 = 48.0;
const TREND_ALL_LABELS: usize = 6;
const TREND_X_MAX_TICKS: usize = 8;
const TREND_X_MIN_GAP: f32 = 36.0 * K;
const TREND_STOP_MIN_RATIO: f32 = 0.17;
const TREND_GAP_FILL: f32 = 180.0 * K;
const TREND_SUB_SCALE: f32 = 0.82;
const TREND_MAJOR_JUMP: f64 = 0.015;
const TREND_AXIS_TEXT_DY: f32 = 18.0 * 0.75 * K;
const TREND_AXIS_SUB_DY: f32 = 29.0 * 0.75 * K;
const TREND_DROP_AXIS_GAP: f32 = 5.0 * K;
const TREND_X_EPSILON: f32 = 1e-4;

fn trend_xs(points: &[TrendPoint], left: f32, right: f32) -> Vec<f32> {
    let count = points.len();
    if count == 0 {
        return Vec::new();
    }
    let mut xs = vec![left; count];
    if count == 1 {
        return xs;
    }
    let width = right - left;
    let mut weights: Vec<f32> = Vec::with_capacity(count - 1);
    let mut sum = 0.0f32;
    for index in 1..count {
        let hours = ((points[index].minute - points[index - 1].minute) as f32 / 60.0)
            .clamp(0.0, TREND_WARP_CAP_H);
        let weight = TREND_WARP_BASE + TREND_WARP_K * (1.0 + hours).ln();
        weights.push(weight);
        sum += weight;
    }
    if sum <= 0.0 {
        for (index, x) in xs.iter_mut().enumerate() {
            *x = left + width * index as f32 / (count as f32 - 1.0);
        }
        xs[count - 1] = right;
        return xs;
    }
    let mut acc = 0.0f32;
    for index in 1..count {
        acc += weights[index - 1];
        xs[index] = left + width * acc / sum;
    }
    xs[count - 1] = right;
    xs
}

#[derive(Debug, Clone, PartialEq)]
struct TrendXLabel {
    text: String,
    sub: String,
    x: f32,
    left: f32,
    right: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct TrendStop {
    index: usize,
    x: f32,
    text: String,
    sub: String,
}

fn trend_clusters(points: &[TrendPoint]) -> Vec<(usize, usize)> {
    let mut out: Vec<(usize, usize)> = Vec::new();
    let mut index = 0usize;
    while index < points.len() {
        let mut end = index;
        while end + 1 < points.len() && points[end + 1].date == points[index].date {
            end += 1;
        }
        out.push((index, end));
        index = end + 1;
    }
    out
}

fn trend_cluster_of(clusters: &[(usize, usize)], index: usize) -> (usize, usize) {
    for cluster in clusters {
        if index >= cluster.0 && index <= cluster.1 {
            return *cluster;
        }
    }
    (index, index)
}

fn trend_is_local_peak(points: &[TrendPoint], index: usize) -> bool {
    let lo = index.saturating_sub(3);
    let hi = (index + 3).min(points.len() - 1);
    for at in lo..=hi {
        if points[at].value > points[index].value {
            return false;
        }
    }
    true
}

fn trend_cluster_weight(points: &[TrendPoint], cluster: (usize, usize)) -> f64 {
    let size = (cluster.1 - cluster.0 + 1) as f64;
    let mut weight = 1.0 + 3.0 * (size - 1.0);
    let mut lo = points[cluster.0].value;
    let mut hi = lo;
    for point in &points[cluster.0..=cluster.1] {
        lo = lo.min(point.value);
        hi = hi.max(point.value);
    }
    weight += 20.0 * (hi - lo).abs();
    if trend_is_local_peak(points, cluster.1) {
        weight += 5.0;
    }
    weight
}

fn trend_month_day(date: &str) -> String {
    match date.len() >= 10 {
        true => date[5..].to_string(),
        false => date.to_string(),
    }
}

fn trend_label_box(label: &mut TrendXLabel, measure: &impl Fn(&str, f32) -> f32) {
    let mut width = measure(label.text.as_str(), 1.0);
    let sub = measure(label.sub.as_str(), TREND_SUB_SCALE);
    if sub > width {
        width = sub;
    }
    label.left = label.x - width / 2.0;
    label.right = label.x + width / 2.0;
}

fn trend_label(
    points: &[TrendPoint],
    xs: &[f32],
    clusters: &[(usize, usize)],
    index: usize,
    measure: &impl Fn(&str, f32) -> f32,
) -> TrendXLabel {
    let cluster = trend_cluster_of(clusters, index);
    let point = &points[index];
    let sub = match cluster.1 > cluster.0 && index == cluster.0 {
        true => format!("{}~{}", points[cluster.0].time, points[cluster.1].time),
        false => point.time.clone(),
    };
    let mut label = TrendXLabel {
        text: point.date.clone(),
        sub,
        x: xs[index],
        left: 0.0,
        right: 0.0,
    };
    trend_label_box(&mut label, measure);
    label
}

fn trend_legacy_label(
    points: &[TrendPoint],
    xs: &[f32],
    index: usize,
    measure: &impl Fn(&str, f32) -> f32,
) -> TrendXLabel {
    let point = &points[index];
    let text = match point.label.trim().is_empty() {
        true => format!("#{}", index + 1),
        false => point.label.clone(),
    };
    let mut label = TrendXLabel {
        text,
        sub: String::new(),
        x: xs[index],
        left: 0.0,
        right: 0.0,
    };
    trend_label_box(&mut label, measure);
    label
}

fn trend_axis_labels(
    points: &[TrendPoint],
    xs: &[f32],
    single: bool,
    measure: &impl Fn(&str, f32) -> f32,
    left: f32,
    right: f32,
) -> Vec<TrendXLabel> {
    let count = points.len();
    if count == 0 {
        return Vec::new();
    }
    if !points.iter().all(TrendPoint::stamped) {
        return trend_label_indices(count)
            .into_iter()
            .map(|index| trend_legacy_label(points, xs, index, measure))
            .collect();
    }
    if single {
        let point = &points[0];
        let mut out = vec![
            TrendXLabel {
                text: point.date.clone(),
                sub: match point.time.is_empty() {
                    true => String::from("首次记录"),
                    false => format!("{} 首次记录", point.time),
                },
                x: left,
                left: 0.0,
                right: 0.0,
            },
            TrendXLabel {
                text: point.date.clone(),
                sub: String::from("当前状态"),
                x: right,
                left: 0.0,
                right: 0.0,
            },
        ];
        for label in out.iter_mut() {
            trend_label_box(label, measure);
        }
        return out;
    }
    if count <= TREND_ALL_LABELS {
        let clusters = trend_clusters(points);
        let mut out: Vec<TrendXLabel> = Vec::with_capacity(count);
        let mut previous = String::new();
        for index in 0..count {
            let mut label = trend_label(points, xs, &clusters, index, measure);
            if label.text == previous {
                label.text = String::new();
            }
            previous = points[index].date.clone();
            trend_label_box(&mut label, measure);
            out.push(label);
        }
        return out;
    }
    plan_trend_x_axis(points, xs, measure)
}

fn plan_trend_x_axis(
    points: &[TrendPoint],
    xs: &[f32],
    measure: &impl Fn(&str, f32) -> f32,
) -> Vec<TrendXLabel> {
    let count = points.len();
    let clusters = trend_clusters(points);
    let mut head = trend_label(points, xs, &clusters, 0, measure);
    let mut tail = trend_label(points, xs, &clusters, count - 1, measure);
    if head.text == tail.text {
        head.sub = points[0].time.clone();
        tail.text = String::new();
        tail.sub = points[count - 1].time.clone();
        trend_label_box(&mut head, measure);
        trend_label_box(&mut tail, measure);
    }
    let mut candidates: Vec<(usize, f64)> = Vec::new();
    for cluster in &clusters {
        if cluster.0 == 0 || cluster.1 == count - 1 {
            continue;
        }
        let mut index = cluster.0;
        for at in cluster.0..=cluster.1 {
            if points[at].value > points[index].value {
                index = at;
            }
        }
        candidates.push((index, trend_cluster_weight(points, *cluster)));
    }
    candidates.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(left.0.cmp(&right.0))
    });
    let fits = |label: &TrendXLabel, chosen: &[TrendXLabel], tail: &TrendXLabel| -> bool {
        for other in chosen {
            if label.left < other.right + TREND_X_MIN_GAP
                && other.left < label.right + TREND_X_MIN_GAP
            {
                return false;
            }
        }
        !(tail.left < label.right + TREND_X_MIN_GAP
            && label.left < tail.right + TREND_X_MIN_GAP)
    };
    let mut chosen: Vec<TrendXLabel> = vec![head];
    for (index, _) in &candidates {
        if chosen.len() >= TREND_X_MAX_TICKS - 1 {
            break;
        }
        let label = trend_label(points, xs, &clusters, *index, measure);
        if fits(&label, &chosen, &tail) {
            chosen.push(label);
        }
    }
    chosen.push(tail);
    chosen.sort_by(|left, right| {
        left.x
            .partial_cmp(&right.x)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for _ in 0..2 {
        let length = chosen.len();
        let mut grown: Vec<TrendXLabel> = Vec::with_capacity(length + 2);
        for position in 0..length {
            grown.push(chosen[position].clone());
            if position + 1 >= length || length >= TREND_X_MAX_TICKS {
                continue;
            }
            let anchor = &chosen[position];
            let next = &chosen[position + 1];
            if next.x - anchor.x <= TREND_GAP_FILL {
                continue;
            }
            let middle = (anchor.x + next.x) / 2.0;
            let mut best: Option<(f32, TrendXLabel)> = None;
            for (index, _) in &candidates {
                let label = trend_label(points, xs, &clusters, *index, measure);
                if label.x <= anchor.x || label.x >= next.x {
                    continue;
                }
                if label.left < anchor.right + TREND_X_MIN_GAP
                    || label.right > next.left - TREND_X_MIN_GAP
                {
                    continue;
                }
                let distance = (label.x - middle).abs();
                if best.as_ref().is_none_or(|(seen, _)| distance < *seen) {
                    best = Some((distance, label));
                }
            }
            if let Some((_, label)) = best {
                grown.push(label);
            }
        }
        chosen = grown;
    }
    chosen
}

fn trend_stops(
    points: &[TrendPoint],
    xs: &[f32],
    peak: usize,
    labels: &[TrendXLabel],
    measure: &impl Fn(&str, f32) -> f32,
    left: f32,
    right: f32,
) -> Vec<TrendStop> {
    let count = points.len();
    if count == 0 {
        return Vec::new();
    }
    let clusters = trend_clusters(points);
    let label_at = |index: usize| -> TrendXLabel {
        if let Some(found) = labels
            .iter()
            .find(|label| (label.x - xs[index]).abs() < TREND_X_EPSILON)
        {
            return found.clone();
        }
        match points[index].stamped() {
            true => trend_label(points, xs, &clusters, index, measure),
            false => trend_legacy_label(points, xs, index, measure),
        }
    };
    let mut candidates: Vec<(usize, TrendXLabel, f64)> = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for label in labels {
        let Some(index) = xs
            .iter()
            .position(|value| (value - label.x).abs() < TREND_X_EPSILON)
        else {
            continue;
        };
        if !seen.insert(index) {
            continue;
        }
        let score = match index > 0 {
            true => {
                let delta = (points[index].value - points[index - 1].value).abs();
                match delta >= TREND_MAJOR_JUMP {
                    true => delta + 1.0,
                    false => delta,
                }
            }
            false => 0.0,
        };
        candidates.push((index, label.clone(), score));
    }
    for index in [0usize, count - 1] {
        if seen.insert(index) {
            candidates.push((index, label_at(index), f64::INFINITY));
        }
    }
    if peak != 0 && peak != count - 1 && seen.insert(peak) {
        candidates.push((peak, label_at(peak), f64::INFINITY));
    }
    candidates.sort_by(|left, right| {
        right
            .2
            .partial_cmp(&left.2)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(left.0.cmp(&right.0))
    });
    let minimum = (right - left) * TREND_STOP_MIN_RATIO;
    let mut kept: Vec<(usize, TrendXLabel)> = Vec::new();
    let mut taken: Vec<f32> = Vec::new();
    for wanted in [0usize, count - 1] {
        if let Some(position) = candidates.iter().position(|item| item.0 == wanted) {
            let (index, label, _) = candidates[position].clone();
            taken.push(label.x);
            kept.push((index, label));
        }
    }
    for (index, label, _) in &candidates {
        if *index == 0 || *index == count - 1 {
            continue;
        }
        if taken.iter().any(|x| (x - label.x).abs() < minimum) {
            continue;
        }
        taken.push(label.x);
        kept.push((*index, label.clone()));
    }
    kept.sort_by(|left, right| {
        left.1
            .x
            .partial_cmp(&right.1.x)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut stops: Vec<TrendStop> = Vec::with_capacity(kept.len());
    let mut previous_year = String::new();
    for (index, label) in kept {
        let point = &points[index];
        let text = match point.stamped() {
            true => {
                let year = point.date[..4].to_string();
                let text = match year != previous_year {
                    true => point.date.clone(),
                    false => trend_month_day(&point.date),
                };
                previous_year = year;
                text
            }
            false => match point.label.trim().is_empty() {
                true => format!("#{}", index + 1),
                false => point.label.clone(),
            },
        };
        let stop = TrendStop {
            index,
            x: label.x,
            text,
            sub: label.sub,
        };
        if let Some(previous) = stops.last() {
            if previous.text == stop.text && previous.sub == stop.sub && previous.index != 0 {
                stops.pop();
            }
        }
        stops.push(stop);
    }
    stops
}


fn stroke_path(kit: &CanvasKit<'_>, points: &[(f32, f32)], width: f32, color: Color) {
    if points.len() < 2 {
        if let Some(point) = points.first() {
            kit.circle(*point, width / 2.0, color);
        }
        return;
    }
    let mut builder = PathBuilder::new();
    builder.move_to(points[0]);
    for point in &points[1..] {
        builder.line_to(*point);
    }
    let path = builder.detach();
    let mut paint = aurorix_render::skia_safe::Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(color.to_skia());
    paint.set_style(aurorix_render::skia_safe::paint::Style::Stroke);
    paint.set_stroke_width(width.max(0.5));
    paint.set_stroke_join(aurorix_render::skia_safe::paint::Join::Round);
    kit.canvas().draw_path(&path, &paint);
}

fn trend_wash_ramp(distance: f32, feather: f32) -> f32 {
    if feather <= 0.0 {
        return 1.0;
    }
    let t = (distance / feather).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn trend_wash_gradient(ink: Color, width: f32, feather: f32, vertical: f32) -> Gradient {
    let mut stops = Vec::with_capacity(6);
    for (offset, factor) in [
        (0.0, 0.0f32),
        (feather / 2.0 / width, 0.5),
        (feather / width, 1.0),
        (1.0 - feather / width, 1.0),
        (1.0 - feather / 2.0 / width, 0.5),
        (1.0, 0.0f32),
    ] {
        let alpha = ((ink.a as f32) * factor * vertical)
            .round()
            .clamp(0.0, 255.0) as u8;
        stops.push(GradientStop::new(
            offset,
            Color::new(ink.r, ink.g, ink.b, alpha),
        ));
    }
    Gradient::Linear(LinearGradient::horizontal(stops))
}

fn trend_wash(kit: &CanvasKit<'_>, rect: Rect, feather: f32, ink: Color) {
    let width = rect.width();
    let height = rect.height();
    if width <= 0.0 || height <= 0.0 || feather <= 0.0 || ink.a == 0 {
        return;
    }
    let feather_x = feather.min(width / 2.0);
    let feather_y = feather.min(height / 2.0);
    let mut y = 0.0;
    while y < height - 0.5 {
        let band = if y < feather_y {
            (y + 1.0).min(feather_y)
        } else if y >= height - feather_y {
            (y + 1.0).min(height)
        } else {
            height - feather_y
        };
        let band = band.max(y + 1.0).min(height);
        let center = (y + band) / 2.0;
        let vertical = trend_wash_ramp(center.min(height - center), feather_y);
        let gradient = trend_wash_gradient(ink, width, feather_x, vertical);
        let _ = kit.fill_gradient(
            area(rect.left(), rect.top() + y, width, band - y),
            0.0,
            &gradient,
        );
        y = band;
    }
}

fn trend_label_indices(count: usize) -> Vec<usize> {
    const MAX_LABELS: usize = 8;
    if count == 0 {
        return Vec::new();
    }
    if count <= MAX_LABELS {
        return (0..count).collect();
    }
    (0..MAX_LABELS)
        .map(|slot| slot * (count - 1) / (MAX_LABELS - 1))
        .collect()
}

fn signed_curve(value: f64) -> String {
    if value > 0.0 {
        format!("+{value:.3}")
    } else if value < 0.0 {
        format!("−{:.3}", -value)
    } else {
        String::from("±0.000")
    }
}

fn trend_stats_note(input: &TrendInput) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(peak) = input.peak {
        parts.push(format!("峰值 {peak:.3}"));
    }
    if let Some(value) = input.recent7 {
        parts.push(format!("近 7 天 {}", signed_curve(value)));
    }
    if let Some(value) = input.recent30 {
        parts.push(format!("近 30 天 {}", signed_curve(value)));
    }
    parts.join(" · ")
}

fn trend_tags(input: &TrendInput) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if let Some(rank) = input.rank {
        out.push(match input.rank_total {
            Some(total) => format!("全站 #{} / {}", thousands(rank), thousands(total)),
            None => format!("全站 #{}", thousands(rank)),
        });
    }
    if let Some(days) = input.days {
        out.push(match input.records {
            Some(records) => format!("{days} 天 · {records} 个记录点"),
            None => format!("{days} 天"),
        });
    } else if let Some(records) = input.records {
        out.push(format!("{records} 个记录点"));
    }
    out
}

fn trend_bound_line(kit: &CanvasKit<'_>, left: f32, right: f32, y: f32, current: bool) {
    if current {
        kit.fill_rect(area(left, y, (right - left).max(0.0), 1.5), CURVE_BOUND_CURRENT);
        return;
    }
    let mut x = left;
    while x < right {
        let end = (x + TREND_BOUND_ON).min(right);
        kit.fill_rect(area(x, y, end - x, 1.0), CURVE_BOUND);
        x += TREND_BOUND_ON + TREND_BOUND_OFF;
    }
}

pub fn paint_trend(kit: &CanvasKit<'_>, input: &TrendInput) {
    let plan = trend_plan();
    let width = TREND_WIDTH as f32;
    let height = trend_size(input).1 as f32;
    let right = LEDGER_W - MARGIN_X;
    paper_backdrop(kit, width, height);
    ledger_art_wash(kit, &input.art_key(), width, height);
    let frame = LedgerFrame {
        left: MARGIN_X,
        right,
        brand_right: false,
        no_brand: true,
        brand_rule_y: 0.0,
    };
    let (plate_w, plate_x) = plate_box(kit, &input.badge, right);
    let plate_left = if plate_w > 0.0 { plate_x } else { 0.0 };
    ledger_head(
        kit,
        &frame,
        "PLAYER TREND",
        trend_title_note(input).as_str(),
        KICKER_TREND,
        plate_left,
    );
    player::paint(
        kit,
        &block_of(&input.badge, right, compact_center_y()).with_rating(input.frame, None),
    );
    let stats_note = trend_stats_note(input);
    section_head(
        kit,
        MARGIN_X,
        right,
        plan.block.title_base,
        "潜力值变化",
        stats_note.as_str(),
        SECTION_RULE_STRONG,
    );
    if input.points.is_empty() {
        sketch::text(
            kit,
            &font_text(SECTION_SIZE * 0.62, INK_SUB),
            "还没有历史 —— 后端每次扫到 PTT 变化就记一条, 有变化这里就有曲线",
            plan.plot.left(),
            plan.plot.top() + 40.0 * 0.75 * K,
            INK_SUB,
        );
        ledger_foot_dated(kit, plan.last_bottom, MARGIN_X, right);
        return;
    }
    let left = plan.plot.left() + 32.0 * 0.75 * K;
    let right_edge = plan.plot.right() - PROFILE_AXIS_GUTTER;
    let top_y = plan.plot.top() + PROFILE_PLOT_TOP_PAD;
    let bottom_y = plan.plot.bottom() - TREND_PLOT_BOTTOM_PAD;
    trend_wash(
        kit,
        area(
            left - 14.0,
            top_y - 16.0,
            right_edge - left + 28.0,
            bottom_y - top_y + 28.0,
        ),
        TREND_WASH_FEATHER,
        CURVE_GLASS,
    );
    let scale = build_curve_scale(&input.points);
    let span = (scale.hi - scale.lo).max(1e-6);
    let y_of = |value: f64| -> f32 {
        let ratio = ((value - scale.lo) / span).clamp(0.0, 1.0) as f32;
        bottom_y - ratio * (bottom_y - top_y)
    };
    let (ticks, _) = curve_ticks(&scale);
    for tick in &ticks {
        let y = y_of(*tick);
        if y < top_y - 1.0 || y > bottom_y + 1.0 {
            continue;
        }
        let mut x = left - 4.0;
        while x < right_edge + 6.0 {
            let seg = (x + TREND_GRID_ON).min(right_edge + 6.0);
            kit.fill_rect(area(x, y, seg - x, 1.0), CURVE_GRID);
            x += TREND_GRID_ON + TREND_GRID_OFF;
        }
        let label = curve_tick_text(*tick, scale.step);
        sketch::text(
            kit,
            &font_text(TREND_AXIS_SIZE, INK_SUB),
            &label,
            right_edge + TREND_AXIS_LABEL_GAP,
            y + TREND_AXIS_SIZE * 0.36,
            INK_SUB,
        );
    }
    let frame_bound = input
        .frame
        .and_then(|frame| usize::try_from(frame).ok())
        .and_then(|index| input.bounds.get(index).copied());
    for bound in &input.bounds {
        if *bound <= scale.lo || *bound >= scale.hi {
            continue;
        }
        let y = y_of(*bound);
        if y < top_y - 1.0 || y > bottom_y + 1.0 {
            continue;
        }
        let current = frame_bound.is_some_and(|value| (value - *bound).abs() < 1e-9);
        trend_bound_line(kit, left, right_edge, y, current);
    }
    let single = input.points.len() == 1;
    let points: Vec<TrendPoint> = match single {
        true => vec![input.points[0].clone(), input.points[0].clone()],
        false => input.points.clone(),
    };
    let xs = trend_xs(&points, left, right_edge);
    let ys: Vec<f32> = points.iter().map(|point| y_of(point.value)).collect();
    let nodes: Vec<(f32, f32)> = xs.iter().copied().zip(ys.iter().copied()).collect();
    if nodes.len() >= 2 {
        let ink_at = |alpha: u8| with_alpha(CURVE_AREA, alpha);
        let area_grad = sketch::gradient_stops(&[
            (0.0, ink_at(TREND_AREA_TOP)),
            (TREND_AREA_MID_FRAC, ink_at(TREND_AREA_MID)),
            (1.0, ink_at(TREND_AREA_BOT)),
        ]);
        let x0 = nodes[0].0.max(left);
        let x1 = nodes[nodes.len() - 1].0.min(right_edge);
        let mut segment = 0usize;
        let mut x = x0;
        while x <= x1 {
            while segment + 2 < nodes.len() && nodes[segment + 1].0 < x {
                segment += 1;
            }
            let (ax, ay) = nodes[segment];
            let (bx, by) = nodes[(segment + 1).min(nodes.len() - 1)];
            let t = if (bx - ax).abs() < f32::EPSILON {
                0.0
            } else {
                ((x - ax) / (bx - ax)).clamp(0.0, 1.0)
            };
            let top = ay + (by - ay) * t;
            let height = bottom_y - top;
            if height > 0.5 {
                let _ = kit.fill_gradient(area(x, top, 1.0, height), 0.0, &area_grad);
            }
            x += 1.0;
        }
    }
    match nodes.len() {
        1 => {
            let y = nodes[0].1;
            let bar = TREND_STROKE_W.max(1.5);
            kit.fill_rect(
                area(left, y - bar / 2.0, (right_edge - left).max(0.0), bar),
                CURVE_INK,
            );
        }
        _ => stroke_path(kit, &nodes, TREND_STROKE_W, CURVE_INK),
    }
    kit.fill_rect(area(left, bottom_y, right_edge - left, 1.5), CURVE_AXIS);
    let count = nodes.len();
    let mut peak = 0usize;
    for index in 0..count {
        if points[index].value > points[peak].value {
            peak = index;
        }
    }
    let measure = |text: &str, scale: f32| -> f32 {
        sketch::text_width(kit, &font_text(TREND_AXIS_SIZE * scale, CURVE_AXIS_INK), text)
    };
    let axis_labels = trend_axis_labels(&points, &xs, single, &measure, left, right_edge);
    let stops = trend_stops(
        &points,
        &xs,
        peak,
        &axis_labels,
        &measure,
        left,
        right_edge,
    );
    for stop in &stops {
        let (x, y) = nodes[stop.index];
        let is_last = stop.index + 1 == count;
        let radius = if is_last { TREND_NOW_R } else { TREND_MAJOR_R };
        let mut guide_y = y + radius + 3.0;
        let guide_bottom = bottom_y - TREND_DROP_AXIS_GAP;
        while guide_y < guide_bottom {
            let segment_end = (guide_y + 7.0 * K).min(guide_bottom);
            kit.fill_rect(area(x - 0.5, guide_y, 1.0, segment_end - guide_y), CURVE_DROP);
            guide_y = segment_end + 7.0 * K;
        }
        sketch::diamond(kit, x, y, radius, CURVE_INK);
        sketch::diamond(
            kit,
            x,
            y,
            radius - TREND_RING_W,
            Color::new(0xFF, 0xFF, 0xFF, 0xFF),
        );
        if is_last {
            sketch::diamond(kit, x, y, radius - TREND_RING_W * 2.0, CURVE_INK);
            sketch::diamond(
                kit,
                x,
                y,
                radius - TREND_RING_W * 3.0,
                Color::new(0xFF, 0xFF, 0xFF, 0xFF),
            );
            sketch::diamond(kit, x, y, radius - TREND_RING_W * 4.0 - 1.0, CURVE_INK);
        }
    }
    let date_style = font_text(TREND_AXIS_SIZE, CURVE_AXIS_INK);
    let sub_style = font_text(TREND_AXIS_SIZE * TREND_SUB_SCALE, CURVE_AXIS_SUB);
    for stop in &stops {
        let x = nodes[stop.index].0;
        let half = measure(stop.text.as_str(), 1.0) / 2.0;
        let low = half + 2.0;
        let high = (width - half - 2.0).max(low);
        sketch::text_center(
            kit,
            &date_style,
            stop.text.as_str(),
            x.clamp(low, high),
            bottom_y + TREND_AXIS_TEXT_DY,
            CURVE_AXIS_INK,
        );
        if stop.sub.is_empty() {
            continue;
        }
        let sub_half = measure(stop.sub.as_str(), TREND_SUB_SCALE) / 2.0;
        let sub_low = sub_half + 2.0;
        let sub_high = (width - sub_half - 2.0).max(sub_low);
        sketch::text_center(
            kit,
            &sub_style,
            stop.sub.as_str(),
            x.clamp(sub_low, sub_high),
            bottom_y + TREND_AXIS_SUB_DY,
            CURVE_AXIS_SUB,
        );
    }
    trend_pills(
        kit,
        &nodes,
        &points,
        left,
        right_edge,
        top_y,
        bottom_y,
    );
    ledger_foot_dated(kit, plan.last_bottom, MARGIN_X, right);
}

fn trend_title_note(input: &TrendInput) -> String {
    trend_tags(input).join(" · ")
}

fn trend_curve_at(nodes: &[(f32, f32)], x: f32) -> Option<f32> {
    if nodes.len() < 2 {
        return nodes.first().map(|node| node.1);
    }
    for pair in nodes.windows(2) {
        let (ax, ay) = pair[0];
        let (bx, by) = pair[1];
        if x < ax || x > bx {
            continue;
        }
        if (bx - ax).abs() < f32::EPSILON {
            return Some(ay);
        }
        let t = (x - ax) / (bx - ax);
        return Some(ay + (by - ay) * t);
    }
    None
}

fn trend_box_hits_curve(nodes: &[(f32, f32)], box_: Rect) -> bool {
    let mut x = box_.left();
    while x < box_.right() {
        if let Some(y) = trend_curve_at(nodes, x) {
            if y >= box_.top() - 2.0 && y <= box_.bottom() + 2.0 {
                return true;
            }
        }
        x += 1.0;
    }
    false
}

struct TrendPill {
    text: String,
    node_x: f32,
    node_y: f32,
    anchor: u8,
    box_: Option<Rect>,
    priority: u8,
}

fn trend_pill_boxes(
    kit: &CanvasKit<'_>,
    nodes: &[(f32, f32)],
    points: &[TrendPoint],
    left: f32,
    right: f32,
    top_y: f32,
    bottom_y: f32,
) -> Vec<(String, Rect)> {
    if points.is_empty() || nodes.len() != points.len() {
        return Vec::new();
    }
    let style = font_display(SIZE_XS, CURVE_INK);
    let (pad_x, pad_y) = (6.0, 4.0);
    let ascent = SIZE_XS * 0.85;
    let descent = SIZE_XS * 0.15;
    let ptt = |value: f64| format!("{value:.3}");
    let last = points.len() - 1;
    let mut peak = 0usize;
    for (index, point) in points.iter().enumerate() {
        if point.value > points[peak].value {
            peak = index;
        }
    }
    let mut pills: Vec<TrendPill> = Vec::new();
    for (index, anchor, priority, text) in [
        (peak, 1u8, 1u8, format!("峰值 {}", ptt(points[peak].value))),
        (last, 2, 0, format!("现在 {}", ptt(points[last].value))),
        (0, 0, 2, ptt(points[0].value)),
    ] {
        if anchor == 1 && (peak == 0 || peak == last) {
            continue;
        }
        if anchor == 0 && (points[0].value - points[last].value).abs() < 1e-9 {
            continue;
        }
        pills.push(TrendPill {
            text,
            node_x: nodes[index].0,
            node_y: nodes[index].1,
            anchor,
            box_: None,
            priority,
        });
    }
    let height = ascent + descent + 2.0 * pad_y;
    for pill in &mut pills {
        let width = sketch::text_width(kit, &style, &pill.text) + 2.0 * pad_x;
        let mut box_ = match pill.anchor {
            2 => area(
                pill.node_x + 6.0 - width,
                pill.node_y - TREND_PILL_CLEAR_NOW - height,
                width,
                height,
            ),
            1 => area(
                pill.node_x - width / 2.0,
                pill.node_y - TREND_PILL_CLEAR - height,
                width,
                height,
            ),
            _ => area(
                pill.node_x - width / 2.0,
                pill.node_y + TREND_PILL_CLEAR,
                width,
                height,
            ),
        };
        if pill.anchor == 0 && box_.bottom() > bottom_y {
            box_ = area(
                pill.node_x - width - 6.0,
                pill.node_y - TREND_PILL_CLEAR - height,
                width,
                height,
            );
        }
        if box_.left() < left - 4.0 {
            let shift = left - 4.0 - box_.left();
            box_ = area(box_.left() + shift, box_.top(), width, height);
        }
        if box_.right() > right - 2.0 {
            let shift = right - 2.0 - box_.right();
            box_ = area(box_.left() + shift, box_.top(), width, height);
        }
        if box_.top() < top_y - 6.0 {
            let shift = top_y - 6.0 - box_.top();
            box_ = area(box_.left(), box_.top() + shift, width, height);
        }
        pill.box_ = Some(box_);
    }
    for pill in &mut pills {
        if let Some(box_) = pill.box_ {
            if trend_box_hits_curve(nodes, box_) {
                pill.box_ = None;
            }
        }
    }
    pills.sort_by_key(|pill| pill.priority);
    let mut placed: Vec<Rect> = Vec::new();
    for pill in &mut pills {
        let Some(box_) = pill.box_ else {
            continue;
        };
        let clash = placed.iter().any(|other| {
            box_.left() - TREND_PILL_GAP < other.right()
                && box_.right() + TREND_PILL_GAP > other.left()
                && box_.top() - TREND_PILL_GAP < other.bottom()
                && box_.bottom() + TREND_PILL_GAP > other.top()
        });
        if clash {
            pill.box_ = None;
            continue;
        }
        placed.push(box_);
    }
    let mut out: Vec<(String, Rect)> = Vec::new();
    for pill in &pills {
        if let Some(box_) = pill.box_ {
            out.push((pill.text.clone(), box_));
        }
    }
    out
}

fn trend_pills(
    kit: &CanvasKit<'_>,
    nodes: &[(f32, f32)],
    points: &[TrendPoint],
    left: f32,
    right: f32,
    top_y: f32,
    bottom_y: f32,
) {
    let style = font_display(SIZE_XS, CURVE_INK);
    let (pad_x, pad_y) = (6.0, 4.0);
    let descent = SIZE_XS * 0.15;
    for (text, box_) in trend_pill_boxes(kit, nodes, points, left, right, top_y, bottom_y) {
        kit.rrect_fill(
            area(
                box_.left() - 1.0,
                box_.top() - 1.0,
                box_.width() + 2.0,
                box_.height() + 2.0,
            ),
            5.0,
            CURVE_LABEL_EDGE,
        );
        kit.rrect_fill(box_, 4.0, CURVE_LABEL_PLATE);
        sketch::text(
            kit,
            &style,
            &text,
            box_.left() + pad_x,
            box_.bottom() - pad_y - descent,
            CURVE_INK,
        );
    }
}

#[derive(Debug, Clone)]
pub struct HisRecord {
    pub label: String,
    pub score: i64,
    pub delta: i64,
    pub ptt: f64,
    pub perfect: i64,
    pub normal_perfect: i64,
    pub far: i64,
    pub lost: i64,
    pub grade: Option<Grade>,
    pub clear_type: Option<i32>,
}

impl HisRecord {
    pub fn new(label: impl Into<String>, score: i64, delta: i64) -> Self {
        Self {
            label: label.into(),
            score,
            delta,
            ptt: 0.0,
            perfect: 0,
            normal_perfect: 0,
            far: 0,
            lost: 0,
            grade: None,
            clear_type: None,
        }
    }

    pub fn with_ptt(mut self, ptt: f64) -> Self {
        self.ptt = ptt;
        self
    }

    pub fn with_normal_perfect(mut self, normal: i64) -> Self {
        self.normal_perfect = normal;
        self
    }

    pub fn with_judgement(mut self, far: i64, lost: i64) -> Self {
        self.far = far;
        self.lost = lost;
        self
    }

    pub fn with_perfect(mut self, perfect: i64) -> Self {
        self.perfect = perfect;
        self
    }

    pub fn judge_text(&self) -> String {
        format!("P{} F{} L{}", self.perfect, self.far, self.lost)
    }

    pub fn delta_text(&self, first: bool) -> String {
        if first && self.delta == 0 {
            return String::from("起点");
        }
        if self.delta > 0 {
            format!("+{}", thousands(self.delta))
        } else if self.delta < 0 {
            format!("−{}", thousands(-self.delta))
        } else {
            String::from("±0")
        }
    }

    pub fn with_grade(mut self, grade: Grade) -> Self {
        self.grade = Some(grade);
        self
    }

    pub fn with_clear(mut self, clear_type: i32) -> Self {
        self.clear_type = Some(clear_type);
        self
    }

    pub fn grade_or_computed(&self) -> Grade {
        self.grade.unwrap_or_else(|| ptt::grade_of(self.score))
    }
}

#[derive(Debug, Clone)]
pub struct HisSection {
    pub difficulty: Difficulty,
    pub constant: f64,
    pub records: Vec<HisRecord>,
    pub truncated: bool,
}

impl HisSection {
    pub fn new(difficulty: Difficulty) -> Self {
        Self {
            difficulty,
            constant: 0.0,
            records: Vec::new(),
            truncated: false,
        }
    }

    pub fn with_constant(mut self, constant: f64) -> Self {
        self.constant = constant;
        self
    }

    pub fn with_records(mut self, records: Vec<HisRecord>) -> Self {
        self.records = records;
        self
    }

    pub fn with_truncated(mut self, truncated: bool) -> Self {
        self.truncated = truncated;
        self
    }

    pub fn best_score(&self) -> i64 {
        self.records
            .iter()
            .map(|record| record.score)
            .max()
            .unwrap_or(0)
    }

    fn heading(&self) -> String {
        match self.constant > 0.0 {
            true => format!("{} {:.3}", self.difficulty.as_str(), self.constant),
            false => self.difficulty.as_str().to_string(),
        }
    }

    fn note(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if self.best_score() > 0 {
            parts.push(format!("最好 {}", thousands(self.best_score())));
        }
        parts.push(format!("共 {} 次变化", self.records.len()));
        if self.truncated {
            parts.push(format!("只显示最近 {} 条", self.records.len()));
        }
        parts.join(" · ")
    }
}

#[derive(Debug, Clone)]
pub struct HisInput {
    pub title: String,
    pub subtitle: String,
    pub theme: Theme,
    pub badge: PlayerBadge,
    pub song_id: String,
    pub song_title: String,
    pub jacket: Option<String>,
    pub sections: Vec<HisSection>,
    pub note: String,
    pub omitted: String,
}

impl HisInput {
    pub fn new(song_id: impl Into<String>, song_title: impl Into<String>) -> Self {
        Self {
            title: String::from("Chart History"),
            subtitle: String::new(),
            theme: Theme::DEFAULT,
            badge: PlayerBadge::new("Afkeru"),
            song_id: song_id.into(),
            song_title: song_title.into(),
            jacket: None,
            sections: Vec::new(),
            note: String::new(),
            omitted: String::new(),
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

    pub fn with_badge(mut self, badge: PlayerBadge) -> Self {
        self.badge = badge;
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = note.into();
        self
    }

    pub fn with_omitted(mut self, omitted: impl Into<String>) -> Self {
        self.omitted = omitted.into();
        self
    }

    pub fn with_jacket(mut self, key: impl Into<String>) -> Self {
        let key = key.into();
        if !key.is_empty() {
            self.jacket = Some(key);
        }
        self
    }

    pub fn with_section(mut self, section: HisSection) -> Self {
        self.sections.push(section);
        self
    }

    pub fn with_sections(mut self, sections: Vec<HisSection>) -> Self {
        self.sections = sections;
        self
    }

    pub fn jacket_key(&self) -> String {
        match self.jacket.as_deref() {
            Some(key) => key.to_string(),
            None => thumb_key(&self.song_id),
        }
    }

    pub fn record_count(&self) -> usize {
        self.sections
            .iter()
            .map(|section| section.records.len())
            .sum()
    }

    pub fn best_score(&self) -> i64 {
        self.sections
            .iter()
            .map(HisSection::best_score)
            .max()
            .unwrap_or(0)
    }

    fn art_key(&self) -> String {
        if self.song_id.is_empty() {
            String::new()
        } else {
            self.jacket_key()
        }
    }
}

impl Default for HisInput {
    fn default() -> Self {
        Self::new("song", "曲目").with_subtitle("游玩历史")
    }
}

fn history_gap() -> f32 {
    let used: f32 = HISTORY_DATE_W + HISTORY_COLS.iter().sum::<f32>() * 0.75 * K;
    let gap = (LEDGER_W - 2.0 * MARGIN_X - used) / HISTORY_COLS.len() as f32;
    gap.max(12.0 * 0.75 * K)
}

fn history_col_right(index: usize) -> f32 {
    let mut x = LEDGER_W - MARGIN_X;
    let gap = history_gap();
    for col in HISTORY_COLS.iter().take(index) {
        x -= col * 0.75 * K + gap;
    }
    x
}

fn history_col_left(index: usize) -> f32 {
    if index >= HISTORY_COLS.len() {
        return 0.0;
    }
    history_col_right(index) - HISTORY_COLS[index] * 0.75 * K
}

fn history_cell_at(row: Rect, index: usize, size: f32) -> (f32, f32, bool) {
    let baseline = row.top() + row.height() / 2.0 + size * 0.36;
    match index {
        HISTORY_COL_DATE => (row.left(), baseline, false),
        HISTORY_COL_JUDGE => (history_col_left(index), baseline, false),
        _ => (history_col_right(index), baseline, true),
    }
}

const HISTORY_HEADS: [(usize, &str); 6] = [
    (HISTORY_COL_DATE, "日期"),
    (HISTORY_COL_SCORE, "分数"),
    (HISTORY_COL_GRADE, "评级"),
    (HISTORY_COL_JUDGE, "判定"),
    (HISTORY_COL_PTT, "单曲 PTT"),
    (HISTORY_COL_DELTA, "变化"),
];

#[derive(Debug, Clone, Default)]
struct HisSectionPlan {
    block: Block,
    head_base: f32,
    rows: Vec<Rect>,
}

#[derive(Debug, Clone, Default)]
struct HisPlan {
    height: f32,
    last_bottom: f32,
    sections: Vec<HisSectionPlan>,
}

fn his_plan(input: &HisInput) -> HisPlan {
    let grid_top = player_bottom(compact_center_y()) + COMPACT_GRID_GAP;
    let col_x = MARGIN_X;
    let col_w = LEDGER_W - 2.0 * MARGIN_X;
    let mut plan = HisPlan::default();
    let mut flow = Flow::new(grid_top);
    for section in &input.sections {
        let mut block = Block::default();
        let top = flow.open(&mut block);
        let head_base = top + HISTORY_HEAD_GAP;
        flow.advance(HISTORY_HEAD_GAP + HISTORY_HEAD_H);
        let rows = flow.rows(section.records.len(), HISTORY_ROW_H, 0.0, col_x, col_w);
        plan.sections.push(HisSectionPlan {
            block,
            head_base,
            rows,
        });
    }
    plan.last_bottom = flow.y;
    plan.height = plan.last_bottom + FOOTER_H;
    plan
}

pub fn his_size(input: &HisInput) -> (i32, i32) {
    (HIS_WIDTH, clamp_height(his_plan(input).height))
}

pub fn his_request(input: &HisInput) -> RenderRequest {
    let (width, height) = his_size(input);
    request_for(width, height).with_scale(1.5)
}

pub fn his_card(input: HisInput) -> RenderCard {
    let request = his_request(&input);
    let label = format!(
        "{} {}x{}",
        CardKind::His.label(),
        request.width,
        request.height
    );
    let cache_key = super::memo::fingerprint("his", &input);
    RenderCard::new(request, label, move |kit| paint_his(kit, &input)).with_cache_key(cache_key)
}

pub fn paint_his(kit: &CanvasKit<'_>, input: &HisInput) {
    let plan = his_plan(input);
    let width = HIS_WIDTH as f32;
    let height = his_size(input).1 as f32;
    let right = LEDGER_W - MARGIN_X;
    paper_backdrop(kit, width, height);
    ledger_art_wash(kit, &input.art_key(), width, height);
    let frame = LedgerFrame {
        left: MARGIN_X,
        right,
        brand_right: false,
        no_brand: true,
        brand_rule_y: 0.0,
    };
    let (plate_w, plate_x) = plate_box(kit, &input.badge, right);
    let plate_left = if plate_w > 0.0 { plate_x } else { 0.0 };
    ledger_head(
        kit,
        &frame,
        input.title.trim(),
        "",
        "",
        plate_left,
    );
    player_block(kit, &input.badge, right, compact_center_y());
    for (index, section) in input.sections.iter().enumerate() {
        let Some(planned) = plan.sections.get(index) else {
            break;
        };
        let heading = section.heading();
        let note = section.note();
        section_head(
            kit,
            MARGIN_X,
            right,
            planned.block.title_base,
            heading.as_str(),
            note.as_str(),
            SECTION_RULE_STRONG,
        );
        let row_area = area(MARGIN_X, planned.head_base, right - MARGIN_X, 0.0);
        let head_style = font_text(SIZE_XS, INK_HEAD);
        for (column, label) in HISTORY_HEADS {
            let (x, _, align_right) = history_cell_at(row_area, column, SIZE_XS);
            if align_right {
                sketch::text_right(kit, &head_style, label, x, planned.head_base, INK_HEAD);
            } else {
                sketch::text(kit, &head_style, label, x, planned.head_base, INK_HEAD);
            }
        }
        for (position, record) in section.records.iter().enumerate() {
            let Some(row) = planned.rows.get(position) else {
                break;
            };
            paint_his_row(kit, *row, record, position + 1 == section.records.len());
        }
    }
    ledger_foot_dated(kit, plan.last_bottom, MARGIN_X, right);
}

fn paint_his_row(kit: &CanvasKit<'_>, row: Rect, record: &HisRecord, first: bool) {
    let date_style = font_text(HISTORY_DATE_SIZE, INK_SUB);
    let date = sketch::ellipsize(kit, &date_style, &record.label, HISTORY_DATE_W);
    let (x, base, _) = history_cell_at(row, HISTORY_COL_DATE, HISTORY_DATE_SIZE);
    sketch::text(kit, &date_style, &date, x, base, INK_SUB);
    let (x, base, _) = history_cell_at(row, HISTORY_COL_SCORE, HISTORY_SCORE_SIZE);
    sketch::text_right(
        kit,
        &font_number(HISTORY_SCORE_SIZE, INK_STRONG),
        &thousands(record.score),
        x,
        base,
        INK_STRONG,
    );
    let (x, base, _) = history_cell_at(row, HISTORY_COL_GRADE, HISTORY_GRADE_SIZE);
    sketch::text_right(
        kit,
        &font_display(HISTORY_GRADE_SIZE, INK_STRONG),
        grade_label(record.grade_or_computed()),
        x,
        base,
        INK_STRONG,
    );
    let (x, base, _) = history_cell_at(row, HISTORY_COL_JUDGE, HISTORY_JUDGE_SIZE);
    let style = font_text(HISTORY_JUDGE_SIZE, INK_SUB);
    let mut cursor = x;
    let pure = match record.normal_perfect > 0 {
        true => format!("P{}(-{})", record.perfect, record.normal_perfect),
        false => format!("P{}", record.perfect),
    };
    for segment in [pure, format!("F{}", record.far), format!("L{}", record.lost)] {
        let text = format!("{segment}  ");
        cursor += sketch::text(kit, &style, &text, cursor, base, INK_SUB);
    }
    let (x, base, _) = history_cell_at(row, HISTORY_COL_PTT, HISTORY_PTT_SIZE);
    let ptt_text = if record.ptt > 0.0 {
        format!("{:.3}", record.ptt)
    } else {
        String::from("—")
    };
    sketch::text_right(
        kit,
        &font_number(HISTORY_PTT_SIZE, INK_SUB),
        &ptt_text,
        x,
        base,
        INK_SUB,
    );
    let (x, base, _) = history_cell_at(row, HISTORY_COL_DELTA, HISTORY_DELTA_SIZE);
    sketch::text_right(
        kit,
        &font_display(HISTORY_DELTA_SIZE, INK_SUB),
        &record.delta_text(first),
        x,
        base,
        INK_SUB,
    );
    kit.fill_rect(
        area(row.left(), row.bottom() - 1.0, row.width(), 1.0),
        ROW_RULE,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::harness::{decode, Harness};
    use crate::cards::Extent;

    fn case_114(input: StatsInput) -> (Vec<u8>, i32, i32) {
        let request = stats_request(&input).with_scale(1.0);
        let (width, height) = (request.width, request.height);
        let harness = Harness::new();
        let bytes = harness.render(request, |kit| paint_stats(kit, &input));
        (bytes, width, height)
    }

    fn case_254(index: usize) -> crate::domain::SuggestCandidate {
        crate::domain::SuggestCandidate {
            song_id: format!("song{index:02}"),
            title: format!("曲目 {index:02}"),
            difficulty: Difficulty::Future,
            constant: 10.0 + index as f64 * 0.1,
            note: 1000,
            score: 9_500_000 + index as i64 * 5_000,
            has_score: !index.is_multiple_of(4),
            far: (index % 3) as i64,
            lost: (index % 2) as i64,
            has_judgement: !index.is_multiple_of(4),
        }
    }

    fn case_255() -> SuggestInput {
        let candidates: Vec<crate::domain::SuggestCandidate> = (0..14).map(case_254).collect();
        SuggestInput::new(
            PlayerBadge::new("Afkeru").with_ptt(12.34),
            crate::domain::suggest::suggest(&candidates, 0.08, 4),
        )
    }

    fn case_256() -> StatsInput {
        StatsInput::new(PlayerBadge::new("Afkeru"))
            .with_title("9 - 15 区间成绩")
            .with_note("命中 42 条 · 已收录 512 张谱面成绩")
            .with_bands(vec![
                StatsBand::new("FTR")
                    .with_counts(120, 96)
                    .with_clear(88, 41, 12)
                    .with_avg(9_642_310.0),
                StatsBand::new("BYD")
                    .with_counts(40, 18)
                    .with_clear(15, 6, 1)
                    .with_avg(9_512_880.0),
            ])
            .with_consts(vec![
                StatsConst::new(9.0, 24, 24),
                StatsConst::new(9.1, 31, 28),
                StatsConst::new(9.2, 27, 19),
                StatsConst::new(9.3, 18, 9),
            ])
            .with_const_note("已截断 27 档")
            .with_grades(vec![
                StatsGrade::new("EX+", 3),
                StatsGrade::new("EX", 21),
                StatsGrade::new("AA", 44),
                StatsGrade::new("A", 12),
            ])
            .with_short_title("短板")
            .with_short(
                (0..6)
                    .map(|i| {
                        StatsRow::new(format!("song{i:02}"), format!("曲目 {i:02}"), "FTR")
                            .with_constant(10.4 + i as f64 * 0.1)
                            .with_score(9_720_000 - i as i64 * 9_000)
                            .with_grade("EX")
                            .with_need("EX+", 12_000 + i as i64 * 1_000)
                    })
                    .collect(),
            )
    }

    #[test]
    fn case_257() {
        let input = case_256();
        let (bytes, width, height) = case_114(input.clone());
        assert_eq!(width, STATS_WIDTH);
        assert_eq!(STATS_WIDTH, 1080);
        assert_eq!(height, stats_size(&input).1);
        assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
        let plan = stats_plan(&input);
        assert!(plan.bands.have && plan.consts.have && plan.grades.have && plan.short.have);
        assert!((plan.plot.height() - STATS_CONST_PLOT_H).abs() < 0.01);
        assert!(plan.plot.top() > plan.consts.body_top);
        assert!(plan.last_bottom > plan.plot.bottom());
        assert!(height > 0 && height <= MAX_HEIGHT);
        assert!(stats_size(&StatsInput::default()).1 <= height);
    }

    #[test]
    fn case_258() {
        let input = case_256();
        let (bytes, _, _) = case_114(input.clone());
        let pixels = decode(&bytes).expect("解码统计页");
        assert!(!pixels.blank());
        let plan = stats_plan(&input);
        let row = plan.band_rows[0];
        let bar = stats_band_bar(row);
        let mid = (bar.top() + bar.height() / 2.0) as u32;
        let mut colors: Vec<[u8; 4]> = Vec::new();
        let mut x = bar.left() as u32;
        while x < bar.right() as u32 {
            let pixel = pixels.at(x, mid);
            if !colors.contains(&pixel) {
                colors.push(pixel);
            }
            x += 2;
        }
        assert!(
            colors.len() >= 3,
            "五段横条要分色，实际 {} 种",
            colors.len()
        );
        assert!((bar.height() - STATS_BAND_BAR_H).abs() < 0.01);
    }

    #[test]
    fn case_259() {
        let input = case_256();
        let plan = stats_plan(&input);
        let (bytes, _, _) = case_114(input);
        let pixels = decode(&bytes).expect("解码");
        let ink = pixels.ink_ratio(
            plan.plot.left() as u32,
            plan.plot.top() as u32,
            plan.plot.width() as u32,
            plan.plot.height() as u32,
            12,
        );
        assert!(ink > 0.01, "柱区要有墨迹，实际 {ink}");
    }

    #[test]
    fn case_260() {
        let input = case_256();
        let plan = stats_plan(&input);
        assert_eq!(plan.short_rows.len(), input.short.len());
        let (bytes, _, _) = case_114(input);
        let pixels = decode(&bytes).expect("解码");
        assert!(!pixels.blank());
    }

    #[test]
    fn case_261() {
        let input = StatsInput::default();
        let (bytes, _, _) = case_114(input);
        let pixels = decode(&bytes).expect("解码");
        assert!(!pixels.blank());
    }

    #[test]
    fn case_262() {
        let input = case_255();
        let request = suggest_request(&input);
        assert_eq!(request.width, SUGGEST_WIDTH);
        assert_eq!(SUGGEST_WIDTH, 1400);
        let plan = push_plan(&input);
        assert!(!plan.cards.is_empty());
        assert!((PUSH_CARD_W - 253.0 * PUSH_K * K).abs() < 0.01);
        assert!((PUSH_CARD_H - 98.0 * PUSH_K * K).abs() < 0.01);
        let first = plan.groups[0];
        assert!((plan.cards[first.first].0 - PUSH_PAD_L).abs() < 0.01);
        assert!((plan.cards[first.first].1 - (first.top + PUSH_LEVEL_H)).abs() < 0.01);
        if plan.cards.len() > 1 {
            assert!(
                (plan.cards[first.first + 1].1 - plan.cards[first.first].1).abs() < 0.01,
                "同一行要并排"
            );
        }
        let harness = Harness::new();
        let bytes = harness.render(request, |kit| paint_suggest(kit, &input));
        assert!(!decode(&bytes).expect("解码").blank());
    }

    #[test]
    fn case_263() {
        let input = case_255();
        let request = suggest_request(&input);
        let harness = Harness::new();
        let bytes = harness.render(request, |kit| paint_suggest(kit, &input));
        let pixels = decode(&bytes).expect("解码");
        let plan = push_plan(&input);
        let (x, y) = plan.cards[0];
        let target = area(
            x + PUSH_CARD_PAD_X,
            y + PUSH_TARGET_BASE - PUSH_TARGET_SZ,
            120.0,
            PUSH_TARGET_SZ,
        );
        let ink = pixels.ink_ratio(
            target.left() as u32,
            target.top() as u32,
            target.width() as u32,
            target.height() as u32,
            2,
        );
        assert!(ink > 0.02, "目标分大字要有墨迹，实际 {ink}");
        let delta = area(
            x + PUSH_DELTA_CX - 40.0,
            y + PUSH_DELTA_BASE - PUSH_DELTA_SZ,
            80.0,
            PUSH_DELTA_SZ,
        );
        let ink = pixels.ink_ratio(
            delta.left() as u32,
            delta.top() as u32,
            delta.width() as u32,
            delta.height() as u32,
            2,
        );
        assert!(ink > 0.01, "需加分要有墨迹，实际 {ink}");
        let hint = &input.board.groups[0].hints[0];
        assert!(push_delta_text(hint).starts_with('+') || push_delta_text(hint) == "NEW");
        assert!(push_rank_arrow(hint).contains(PUSH_ARROW));
    }

    #[test]
    fn case_264() {
        let input = SuggestInput::default();
        let request = suggest_request(&input);
        let harness = Harness::new();
        let bytes = harness.render(request, |kit| paint_suggest(kit, &input));
        assert!(!decode(&bytes).expect("解码").blank());
    }

    #[test]
    fn case_265() {
        let input = ChapInput {
            title: String::from("曲包进度"),
            subtitle: String::from("按曲包统计"),
            theme: Theme::DEFAULT,
            badge: PlayerBadge::new("Afkeru"),
            packs: (0..3)
                .map(|index| {
                    ChapPack::new(format!("pack-{index}"), format!("曲包 {index}"), 6)
                        .with_owned(5)
                        .with_cleared(4)
                        .with_full_combo(3)
                        .with_pure_memory(2)
                        .with_charts(
                            (0..6)
                                .map(|item| {
                                    ChapSong::new(
                                        format!("song-{index}-{item}"),
                                        format!("曲目 {item}"),
                                        Difficulty::Future,
                                    )
                                    .with_constant(10.3)
                                    .with_score(9_800_000)
                                    .with_ptt(11.8)
                                    .with_played(item != 5)
                                })
                                .collect(),
                        )
                })
                .collect(),
            expand: ChapExpand::All,
            focus: None,
            totals: None,
            note: String::from("预览样张"),
        };
        let plan = chap_plan(&input);
        assert!((CHAP_W - 1350.0).abs() < 0.01);
        let pack_section = &plan.sections[0];
        assert!(!pack_section.packs.is_empty());
        assert_eq!(pack_section.cards.len(), 3);
        assert!((pack_section.cards[1].0 - (CHAP_PAD_L + PACK_STEP_X)).abs() < 0.01);
        assert!((pack_section.cards[0].0 - CHAP_PAD_L).abs() < 0.01);
        let chart_section = &plan.sections[1];
        assert_eq!(chart_section.name, "FTR");
        assert_eq!(chart_section.charts.len(), 18);
        assert!((chart_section.cards[1].0 - (CHAP_PAD_L + CHART_STEP_X)).abs() < 0.01);
        let request = chap_request(&input);
        assert_eq!(request.width, CHAP_WIDTH);
        let harness = Harness::new();
        let bytes = harness.render(request, |kit| paint_chap(kit, &input));
        assert!(!decode(&bytes).expect("解码").blank());
    }

    #[test]
    fn case_266() {
        let pack = ChapPack::new("pack", "曲包", 10)
            .with_owned(6)
            .with_cleared(4)
            .with_full_combo(3)
            .with_pure_memory(2);
        let (clear_only, fc_only, pm, fail, rest) = pack_counts(&pack);
        assert_eq!((clear_only, fc_only, pm, fail, rest), (1, 1, 2, 2, 4));
        assert_eq!(pack_count_ink(&pack), CHAP_INK_BLUE);
        let done = ChapPack::new("p", "包", 4).with_owned(4);
        assert_eq!(pack_count_ink(&done), CHAP_INK_MINT);
        let idle = ChapPack::new("p", "包", 4);
        assert_eq!(pack_count_ink(&idle), CHAP_INK_DIM);
        assert_eq!(pack.missing_count(), 4);
    }

    #[test]
    fn case_267() {
        let mut input = ChapInput::default().with_pack(
            ChapPack::new("a", "A", 2).with_chart(ChapSong::new("s", "S", Difficulty::Past)),
        );
        input.expand = ChapExpand::Collapsed;
        let plan = chap_plan(&input);
        assert_eq!(plan.sections.len(), 1, "收起时只剩包卡那一区");
        assert!(!plan.sections[0].packs.is_empty());
        assert!(chap_expand_summary(&input).contains("已记录 0/2"));
    }

    #[test]
    fn case_268() {
        assert_eq!(trend_label_indices(0), Vec::<usize>::new());
        assert_eq!(trend_label_indices(1), vec![0]);
        assert_eq!(trend_label_indices(6), (0..6).collect::<Vec<_>>());
        for count in [9, 14, 60, 200] {
            let labels = trend_label_indices(count);
            assert_eq!(labels.len(), 8);
            assert_eq!(labels.first(), Some(&0));
            assert_eq!(labels.last(), Some(&(count - 1)));
            assert!(labels.windows(2).all(|pair| pair[0] < pair[1]));
        }
    }

    fn trend_axis_measure(text: &str, scale: f32) -> f32 {
        text.chars().count() as f32 * 7.0 * scale
    }

    fn trend_axis_point(index: usize, date: &str, time: &str, minute: i64, value: f64) -> TrendPoint {
        TrendPoint::new(format!("#{index}"), value).with_clock(date, time, minute)
    }


    #[test]
    fn trend_axis_marks_real_events() {
        let mut points: Vec<TrendPoint> = Vec::new();
        for day in 0..30i64 {
            let date = format!("2026-03-{:02}", day + 1);
            for slot in 0..2i64 {
                let minute = day * 1440 + 600 + slot * 120;
                let time = format!("{:02}:00", 10 + slot * 2);
                let value = 12.0 + day as f64 * 0.01 + slot as f64 * 0.002;
                let index = points.len();
                points.push(trend_axis_point(
                    index,
                    date.as_str(),
                    time.as_str(),
                    minute,
                    value,
                ));
            }
        }
        let xs = trend_xs(&points, 24.0, 1040.0);
        assert_eq!(xs.first().copied(), Some(24.0));
        assert_eq!(xs.last().copied(), Some(1040.0));
        assert!(xs.windows(2).all(|pair| pair[1] > pair[0]), "事件轴必须严格递增");
        let labels = trend_axis_labels(&points, &xs, false, &trend_axis_measure, 24.0, 1040.0);
        assert!(labels.len() >= 2, "至少要有首末两条刻度");
        assert!(labels.len() <= TREND_X_MAX_TICKS, "刻度不能超过上限");
        for label in &labels {
            assert!(
                xs.iter().any(|x| (x - label.x).abs() < TREND_X_EPSILON),
                "刻度 {} 必须落在真实事件上",
                label.text
            );
            assert!(!label.sub.is_empty(), "刻度 {} 缺少时刻", label.text);
        }
        let mut packed = labels.clone();
        packed.sort_by(|left, right| {
            left.left
                .partial_cmp(&right.left)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for pair in packed.windows(2) {
            assert!(
                pair[1].left - pair[0].right >= TREND_X_MIN_GAP - 0.01,
                "相邻刻度空白不足: {} 与 {}",
                pair[0].text,
                pair[1].text
            );
        }
        let stops = trend_stops(&points, &xs, 59, &labels, &trend_axis_measure, 24.0, 1040.0);
        assert_eq!(stops.first().map(|stop| stop.index), Some(0));
        assert_eq!(
            stops.last().map(|stop| stop.index),
            Some(points.len() - 1)
        );
        for pair in stops.windows(2) {
            assert!(
                pair[0].text != pair[1].text || pair[0].sub != pair[1].sub,
                "横轴上不许出现两条一样的坐标标签"
            );
        }
        assert!(stops.len() < points.len(), "没入选的事件不该画锚点");
    }

    #[test]
    fn trend_axis_same_day_keeps_times() {
        let points: Vec<TrendPoint> = (0..5)
            .map(|index| {
                let minute = 14 * 60 + index as i64 * 10;
                trend_axis_point(
                    index as usize,
                    "2026-03-01",
                    format!("14:{:02}", index * 10).as_str(),
                    minute,
                    11.0 + index as f64 * 0.1,
                )
            })
            .collect();
        let xs = trend_xs(&points, 24.0, 1040.0);
        let labels = plan_trend_x_axis(&points, &xs, &trend_axis_measure);
        assert_eq!(labels.first().map(|label| label.text.as_str()), Some("2026-03-01"));
        assert_eq!(labels.first().map(|label| label.sub.as_str()), Some("14:00"));
        let tail = labels.last().expect("要有尾刻度");
        assert_eq!(tail.text, "", "同日时尾刻度不再重复日期");
        assert_eq!(tail.sub, "14:40", "同日时尾刻度只写结束时刻");
        let stops = trend_stops(&points, &xs, 4, &labels, &trend_axis_measure, 24.0, 1040.0);
        assert_eq!(stops.len(), 2, "同日只留首末两个锚点");
        assert_eq!(stops[0].text, "2026-03-01");
        assert_eq!(stops[1].text, "03-01", "尾锚点省略年份");
    }

    #[test]
    fn trend_axis_spreads_bursts_and_caps_idle() {
        let mut points = vec![trend_axis_point(0, "2026-03-01", "08:00", 0, 12.0)];
        for index in 1..=12i64 {
            points.push(trend_axis_point(
                index as usize,
                "2026-03-04",
                "20:00",
                72 * 60 + (index - 1) * 5,
                12.0 + index as f64 * 0.012,
            ));
        }
        points.push(trend_axis_point(
            13,
            "2026-03-06",
            "22:00",
            72 * 60 + 55 + 50 * 60,
            12.3,
        ));
        let xs = trend_xs(&points, 78.0, 978.0);
        let width = 978.0 - 78.0;
        let burst = xs[11] - xs[2];
        assert!(burst >= 30.0, "连推段要摊开，实得 {burst}px");
        let idle = xs[13] - xs[12];
        assert!(
            idle <= width * 0.3,
            "50 小时空档占了 {:.0}% 画布",
            idle / width * 100.0
        );
        let single = vec![trend_axis_point(0, "2026-03-01", "08:00", 0, 12.0)];
        assert_eq!(trend_xs(&single, 78.0, 978.0), vec![78.0]);
    }

    #[test]
    fn case_269() {
        let input = TrendInput::new(PlayerBadge::new("Afkeru")).with_points(
            (0..12)
                .map(|index| {
                    TrendPoint::new(format!("11-{:02}", index + 1), 12.0 + index as f64 * 0.04)
                })
                .collect(),
        );
        let plan = trend_plan();
        assert!((TREND_CURVE_H - 660.0 * 0.75 * K).abs() < 0.01);
        assert!((plan.plot.width() - (LEDGER_W - 2.0 * MARGIN_X)).abs() < 0.01);
        let request = trend_request(&input);
        assert_eq!(request.width, TREND_WIDTH);
        assert_eq!(TREND_WIDTH, 1080);
        let harness = Harness::new();
        let bytes = harness.render(request, |kit| paint_trend(kit, &input));
        let pixels = decode(&bytes).expect("解码趋势");
        let ink = pixels.ink_ratio(
            plan.plot.left() as u32,
            plan.plot.top() as u32,
            plan.plot.width() as u32,
            plan.plot.height() as u32,
            40,
        );
        assert!(ink > 0.01, "曲线区要有墨迹，实际 {ink}");
        let scale = build_curve_scale(&input.points);
        assert!(scale.hi > scale.lo);
        let xs = trend_xs(&input.points, 100.0, 900.0);
        assert_eq!(xs.first().copied(), Some(100.0));
        assert_eq!(xs.last().copied(), Some(900.0));
    }

    #[test]
    fn case_270() {
        let input = TrendInput::new(PlayerBadge::new("Afkeru")).with_points(
            (0..12)
                .map(|index| {
                    TrendPoint::new(format!("11-{:02}", index + 1), 12.0 + index as f64 * 0.04)
                })
                .collect(),
        );
        let plan = trend_plan();
        let left = plan.plot.left() + 32.0 * 0.75 * K;
        let right = plan.plot.right() - PROFILE_AXIS_GUTTER;
        let top_y = plan.plot.top() + PROFILE_PLOT_TOP_PAD;
        let bottom_y = plan.plot.bottom() - TREND_PLOT_BOTTOM_PAD;
        let boxes: std::cell::RefCell<Vec<(String, Rect)>> = std::cell::RefCell::new(Vec::new());
        let nodes: std::cell::RefCell<Vec<(f32, f32)>> = std::cell::RefCell::new(Vec::new());
        let harness = Harness::new();
        let _ = harness.render(trend_request(&input), |kit| {
            let xs = trend_xs(&input.points, left, right);
            let scale = build_curve_scale(&input.points);
            let span = (scale.hi - scale.lo).max(1e-6);
            let ys: Vec<f32> = input
                .points
                .iter()
                .map(|point| {
                    let ratio = ((point.value - scale.lo) / span).clamp(0.0, 1.0) as f32;
                    bottom_y - ratio * (bottom_y - top_y)
                })
                .collect();
            let joined: Vec<(f32, f32)> = xs.into_iter().zip(ys).collect();
            *boxes.borrow_mut() =
                trend_pill_boxes(kit, &joined, &input.points, left, right, top_y, bottom_y);
            *nodes.borrow_mut() = joined;
        });
        let boxes = boxes.into_inner();
        let nodes = nodes.into_inner();
        assert_eq!(boxes.len(), 2, "单边上涨时只有现在与起点两枚");
        let case_543 = |x: f32| -> Option<f32> {
            for pair in nodes.windows(2) {
                let (ax, ay) = pair[0];
                let (bx, by) = pair[1];
                if x >= ax && x <= bx && (bx - ax).abs() > f32::EPSILON {
                    return Some(ay + (by - ay) * (x - ax) / (bx - ax));
                }
            }
            None
        };
        for (text, box_) in &boxes {
            let mut x = box_.left();
            while x <= box_.right() {
                if let Some(y) = case_543(x) {
                    assert!(
                        y < box_.top() || y > box_.bottom(),
                        "胶囊「{text}」压住了曲线"
                    );
                }
                x += 1.0;
            }
        }
        for (index, left_box) in boxes.iter().enumerate() {
            for right_box in boxes.iter().skip(index + 1) {
                assert!(
                    !(left_box.1.left() < right_box.1.right()
                        && left_box.1.right() > right_box.1.left()
                        && left_box.1.top() < right_box.1.bottom()
                        && left_box.1.bottom() > right_box.1.top()),
                    "两枚胶囊不许叠在一起"
                );
            }
        }
    }

    #[test]
    fn case_271() {
        let single =
            TrendInput::new(PlayerBadge::new("Afkeru")).with_point(TrendPoint::new("01-01", 12.0));
        let harness = Harness::new();
        let bytes = harness.render(trend_request(&single), |kit| paint_trend(kit, &single));
        assert!(!decode(&bytes).expect("解码").blank());
        let empty = TrendInput::default();
        let bytes = harness.render(trend_request(&empty), |kit| paint_trend(kit, &empty));
        assert!(!decode(&bytes).expect("解码").blank());
    }

    #[test]
    fn case_272() {
        let input = HisInput::new("acheron", "Acheron")
            .with_badge(PlayerBadge::new("Afkeru"))
            .with_sections(vec![HisSection::new(Difficulty::Future)
                .with_constant(10.4)
                .with_records(
                    (0..8)
                        .map(|index| {
                            let delta = if index == 0 { 0 } else { 11_000 };
                            HisRecord::new(
                                format!("11-{:02} 12:34", index + 1),
                                9_800_000 + index * 11_000,
                                delta,
                            )
                            .with_perfect(1020)
                            .with_judgement(20 - index, index)
                            .with_ptt(12.1)
                            .with_grade(Grade::ExPlus)
                        })
                        .collect(),
                )]);
        let request = his_request(&input);
        assert_eq!(request.width, HIS_WIDTH);
        let plan = his_plan(&input);
        let rows = &plan.sections[0].rows;
        assert_eq!(rows.len(), 8);
        assert!((rows[1].top() - (rows[0].top() + HISTORY_ROW_H)).abs() < 0.01);
        let (x_date, _, right_date) = history_cell_at(rows[0], HISTORY_COL_DATE, HISTORY_DATE_SIZE);
        assert!(!right_date, "日期列左对齐");
        assert!((x_date - MARGIN_X).abs() < 0.01);
        let (x_score, _, right_score) =
            history_cell_at(rows[0], HISTORY_COL_SCORE, HISTORY_SCORE_SIZE);
        assert!(right_score, "数字列右对齐");
        assert!(x_score > history_col_left(HISTORY_COL_SCORE));
        assert!(history_col_right(HISTORY_COL_DELTA) <= LEDGER_W - MARGIN_X + 0.01);
        let records = &input.sections[0].records;
        assert_eq!(records[1].judge_text(), "P1020 F19 L1");
        assert_eq!(records[1].delta_text(false), "+11'000");
        assert_eq!(records[0].delta_text(true), "起点");
        let harness = Harness::new();
        let bytes = harness.render(request, |kit| paint_his(kit, &input));
        assert!(!decode(&bytes).expect("解码").blank());
    }

    #[test]
    fn case_273() {
        let input = HisInput::default();
        let harness = Harness::new();
        let bytes = harness.render(his_request(&input), |kit| paint_his(kit, &input));
        assert!(!decode(&bytes).expect("解码").blank());
    }

    #[test]
    fn case_274() {
        for height in [
            stats_size(&StatsInput::default()).1,
            suggest_size(&SuggestInput::default()).1,
            chap_size(&ChapInput::default()).1,
            trend_size(&TrendInput::default()).1,
            his_size(&HisInput::default()).1,
        ] {
            assert!(
                (MIN_HEIGHT..=MAX_HEIGHT).contains(&height),
                "高度越界：{height}"
            );
        }
        assert_eq!(CardKind::Trend.spec().height, Extent::dynamic(360, 12000));
    }

    #[test]
    fn case_275_chapter_title_stops_before_the_player_block() {
        let input = ChapInput {
            title: String::from("Absolute Reason And Then Some Extra Words"),
            subtitle: String::from("曲包「Absolute Reason」· 8 张谱面"),
            theme: Theme::DEFAULT,
            badge: PlayerBadge::new("Afkeru"),
            packs: vec![ChapPack::new("pack", "Absolute Reason", 4).with_chart(ChapSong::new(
                "song",
                "曲目",
                Difficulty::Future,
            ))],
            expand: ChapExpand::All,
            focus: Some(0),
            totals: None,
            note: String::from("预览样张"),
        };
        let harness = Harness::new();
        let bytes = harness.render(chap_request(&input), |kit| {
            let plate_x = plate_box(kit, &input.badge, CHAP_RIGHT).1;
            let avail = (plate_x - LEDGER_HEAD_BAR_GAP - CHAP_PAD_L).max(0.0);
            let style = font_display(CHAP_TITLE_SIZE, PUSH_INK);
            let drawn = sketch::ellipsize(kit, &style, &input.title, avail);
            assert!(drawn.ends_with('…'), "过长包名要带省略号: {drawn}");
            let advance = sketch::text_width(kit, &style, &drawn);
            assert!(
                advance <= avail + 1.0,
                "标题排布宽 {advance} 超过可用宽 {avail}, 会压到玩家块"
            );
            paint_chap(kit, &input);
        });
        assert!(!decode(&bytes).expect("解码").blank());
    }
}
