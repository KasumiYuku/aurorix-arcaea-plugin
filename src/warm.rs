use std::sync::Arc;

use aurorix_plugin::PluginAssets;
use aurorix_render::RenderContext;

pub const CORE_TEXTURES: [&str; 58] = [
    "avatar/char_icon_border.png",
    "avatar/rating/rating_off.png",
    "avatar/unknown_icon.png",
    "background/score.png",
    "banners/course_banner_01.png",
    "banners/online_banner_2026_10.png",
    "char_icons/unknown_icon.png",
    "characters/71u_s1.png",
    "clear_type/clear_fail.png",
    "clear_type/clear_full.png",
    "clear_type/clear_normal.png",
    "clear_type/clear_pure.png",
    "difficulty/max-recall-beyond.png",
    "difficulty/max-recall-eternal.png",
    "difficulty/max-recall-future.png",
    "difficulty/max-recall-inscribed.png",
    "difficulty/max-recall-past.png",
    "difficulty/max-recall-present.png",
    "grade/A.png",
    "grade/AA.png",
    "grade/B.png",
    "grade/C.png",
    "grade/D.png",
    "grade/EX+.png",
    "grade/EX.png",
    "logo/arcaea_logo.png",
    "result/button/back.png",
    "result/judge/pure.png",
    "result/topbar/top_bar_bg.png",
    "songselect/cell/song_cell_right.png",
    "songselect/cell/song_cell_right_beyond.png",
    "songselect/cell/song_cell_right_beyond_selected.png",
    "songselect/cell/song_cell_right_selected.png",
    "songselect/cell/song_cell_selected_piece.png",
    "songselect/cell/song_cell_selected_piece_beyond.png",
    "songselect/cell/start.png",
    "songselect/diff/difficulty_selector_0.png",
    "songselect/diff/difficulty_selector_0_selected.png",
    "songselect/diff/difficulty_selector_1.png",
    "songselect/diff/difficulty_selector_1_selected.png",
    "songselect/diff/difficulty_selector_2.png",
    "songselect/diff/difficulty_selector_2_selected.png",
    "songselect/diff/difficulty_selector_3-1.png",
    "songselect/diff/difficulty_selector_3-1_selected.png",
    "songselect/diff/difficulty_selector_3.png",
    "songselect/diff/difficulty_selector_3_selected.png",
    "songselect/diff/difficulty_selector_4.png",
    "songselect/diff/difficulty_selector_4_selected.png",
    "songselect/diff/song_score_tab.png",
    "songselect/grade/a.png",
    "songselect/grade/aa.png",
    "songselect/grade/b.png",
    "songselect/grade/c.png",
    "songselect/grade/d.png",
    "songselect/grade/ex.png",
    "songselect/grade/explus.png",
    "songselect/misc/main_button.png",
    "songselect/pack/song_currentpack.png",
];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Warmup {
    pub fonts: usize,
    pub templates: usize,
    pub textures: usize,
    pub missing: usize,
}

pub fn warmup(assets: &PluginAssets, context: &RenderContext) -> Warmup {
    let registry = Arc::new(aurorix_template::Registry::new());
    let report = assets.bootstrap(&registry);
    let templates = crate::templates::register_templates(&registry);
    let mut textures = 0;
    let mut missing = 0;
    for key in CORE_TEXTURES {
        match context.asset(key) {
            Ok(_) => textures += 1,
            Err(_) => missing += 1,
        }
    }
    Warmup {
        fonts: report.fonts.len(),
        templates,
        textures,
        missing,
    }
}
