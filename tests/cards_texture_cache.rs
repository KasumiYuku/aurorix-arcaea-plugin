use aurorix_arcaea_plugin::cards::b50::{paint, request, B50Input};
use aurorix_arcaea_plugin::cards::harness::{image_size, Harness};
use aurorix_arcaea_plugin::cards::theme::Theme;
use aurorix_arcaea_plugin::cards::widgets::PlayerBadge;
use aurorix_arcaea_plugin::domain::BoardRow;
use aurorix_arcaea_plugin::model::{Difficulty, Grade};

const SLOTS: usize = 50;

fn row(index: usize, song: &str) -> BoardRow {
    BoardRow {
        rank: index + 1,
        song_id: song.to_string(),
        title: format!("Abstruse Dilemma {index:02}"),
        difficulty: Difficulty::Future,
        constant: 9.6,
        has_constant: true,
        score: 9_978_342,
        rating: 12.4,
        grade: Grade::Ex,
        far: 2,
        lost: 0,
        normal_perfect: 12,
        object_amount: 1100,
        clear_type: Some(3),
        has_judgement: true,
        band: None,
        span: 0.3,
        pressure: 0.0,
        play_at: None,
        mess: 0.0,
        over_delta: 0.0,
        has_over_level: false,
        over_level: None,
        over_band: None,
        over_mess: None,
        backend_pressure: None,
    }
}

#[test]
fn case_430() {
    let best: Vec<BoardRow> = (0..SLOTS)
        .map(|index| row(index, "abstrusedilemma"))
        .collect();
    let recent: Vec<BoardRow> = (0..10).map(|index| row(index, "abstrusedilemma")).collect();
    let input = B50Input::new(PlayerBadge::new("Afkeru").with_ptt(12.31), Theme::DEFAULT)
        .with_best(best)
        .with_recent(recent)
        .with_ptt(12.31)
        .with_note("五十格同图");
    let harness = Harness::new();
    let cold = harness.render(request(&input), |kit| paint(kit, &input));
    assert_eq!(
        image_size(&cold),
        Some((1820, 2134)),
        "B50 是 Go 的 1300x1524 按 1.4 放大后的 1820x2134"
    );
    let decoded_cold = harness.decodes();
    assert!(decoded_cold > 0, "首次渲染必须真的解码素材");
    assert!(
        decoded_cold < SLOTS as u64,
        "50 个格子共用一张缩略图, 解码次数必须远少于格子数, 实际 {decoded_cold}"
    );
    let warm = harness.render(request(&input), |kit| paint(kit, &input));
    assert_eq!(image_size(&warm), Some((1820, 2134)));
    let again = harness.decodes() - decoded_cold;
    assert_eq!(
        again, 0,
        "同一张缩略图画满 50 格后再画一次必须全部命中纹理缓存, 实际新增 {again}"
    );
    let before = harness.decodes();
    let third = harness.render(request(&input), |kit| paint(kit, &input));
    assert_eq!(image_size(&third), Some((1820, 2134)));
    assert_eq!(
        harness.decodes() - before,
        0,
        "反复渲染同一张榜面不允许再解码任何素材"
    );
}
