use aurorix_arcaea_plugin::domain::board::{
    band_of, select_board, select_fr50, select_mx50, select_nm50, select_ol50, select_pm50,
    select_x50, BandCounts, BoardInput, BoardKind, BoardRecord, OverBand, BOARD_TOP, OVER_FLOOR,
};
use aurorix_arcaea_plugin::domain::FULL_SCORE;
use aurorix_arcaea_plugin::model::Difficulty;

fn scored(
    song: &str,
    difficulty: Difficulty,
    constant: f64,
    score: i64,
    far: i64,
    lost: i64,
) -> BoardRecord {
    BoardRecord::new(song, song, difficulty, constant)
        .with_score(score)
        .with_judgement(far, lost, 1000)
}

#[test]
fn case_397() {
    let input = BoardInput::new(vec![
        scored("perfect", Difficulty::Future, 10.0, FULL_SCORE, 0, 0),
        scored("full_recall", Difficulty::Future, 10.0, 9_999_000, 3, 0),
        scored("one_lost", Difficulty::Future, 10.0, 9_998_000, 0, 1),
        scored("one_far", Difficulty::Future, 10.0, 9_997_000, 1, 0),
        scored("messy", Difficulty::Future, 10.0, 9_900_000, 5, 3),
    ]);
    assert_eq!(select_pm50(&input).rows.len(), 1, "pm50 只收 0 far 0 lost");
    assert_eq!(
        select_fr50(&input).rows.len(),
        3,
        "fr50 收 PM 与所有 0 lost 的局"
    );
    assert_eq!(
        select_x50(&input).rows.len(),
        2,
        "x50 收 F + L ≤ 2（Far 与 Lost 直接相加），PM 归 pm50"
    );
    assert_eq!(select_nm50(&input).rows.len(), 4, "nm50 收所有没 PM 的");
    assert_eq!(select_mx50(&input).rows.len(), 1, "mx50 只收 1000 万");
    assert_eq!(
        select_board(&input, BoardKind::Ol50, BOARD_TOP).rows.len(),
        5,
        "ol50 的候选门槛只有分数下限"
    );
}

#[test]
fn case_398() {
    assert_eq!(BoardKind::ALL.len(), 6);
    let names: Vec<&str> = BoardKind::ALL.iter().map(|kind| kind.as_str()).collect();
    assert_eq!(names, vec!["pm50", "fr50", "x50", "nm50", "mx50", "ol50"]);
    assert_eq!(OVER_FLOOR, 7_000_000);
    assert_eq!(BOARD_TOP, 50);
}

#[test]
fn case_399() {
    assert_eq!(
        band_of(0.3, 9_900_000),
        Some(OverBand::Held),
        "越线且 EX 是攻克"
    );
    assert_eq!(
        band_of(0.3, 9_790_000),
        Some(OverBand::Mild),
        "越线未 EX 且只差一口气是小越"
    );
    assert_eq!(
        band_of(0.9, 9_450_000),
        Some(OverBand::Heavy),
        "跨度 0.8 以上是大越"
    );
    assert_eq!(
        band_of(0.3, 9_150_000),
        Some(OverBand::Disaster),
        "掉出 AA 是打不动"
    );
    assert_eq!(
        band_of(1.2, 9_340_000),
        Some(OverBand::Disaster),
        "跨度 1.0 以上且不到 935 万也是打不动"
    );
    assert_eq!(
        band_of(-0.2, 9_700_000),
        Some(OverBand::Underperform),
        "线内 0.5 又没 EX 是翻车"
    );
    assert_eq!(
        band_of(-0.2, 9_820_000),
        Some(OverBand::Normal),
        "线内拿下 EX 是达标"
    );
    assert_eq!(
        band_of(0.3, 9_899_999),
        Some(OverBand::Normal),
        "越级上 EX+ 之下到 EX 之间也是达标"
    );
    assert_eq!(
        band_of(0.3, 9_800_000),
        Some(OverBand::Normal),
        "越级上 EX 记达标，不许掉进大越或打不动"
    );
    assert_eq!(
        band_of(-1.2, 9_400_000),
        None,
        "线下一大截的低级谱根本不进榜"
    );
}

#[test]
fn case_400() {
    let mut records = Vec::new();
    for (song, band, span, score) in [
        ("disaster", OverBand::Disaster, 1.6, 9_000_000),
        ("heavy", OverBand::Heavy, 0.9, 9_400_000),
        ("mild", OverBand::Mild, 0.3, 9_790_000),
        ("held", OverBand::Held, 0.4, 9_900_000),
        ("underperform", OverBand::Underperform, -0.2, 9_700_000),
        ("normal", OverBand::Normal, 0.2, 9_850_000),
    ] {
        let mut record = scored(song, Difficulty::Future, 10.0, score, 0, 0);
        record.over_level = Some(span);
        record.over_band = Some(band);
        record.has_over_level = true;
        records.push(record);
    }
    let input = BoardInput::new(records).with_curve(true).with_anchor(10.0);
    let board = select_ol50(&input);
    let counts: BandCounts = board.band_counts;
    assert_eq!(counts.disaster, 1);
    assert_eq!(counts.heavy, 1);
    assert_eq!(counts.mild, 1);
    assert_eq!(counts.held, 1);
    assert_eq!(counts.underperform, 1);
    assert_eq!(counts.normal, 1);
    assert_eq!(counts.over_count(), 4, "只有攻克/小越/大越/打不动算越级");
    assert_eq!(board.over_count, 4);
    assert_eq!(counts.total(), 6);
    let order: Vec<&str> = board.rows.iter().map(|row| row.song_id.as_str()).collect();
    assert_eq!(
        order,
        vec![
            "disaster",
            "heavy",
            "mild",
            "held",
            "underperform",
            "normal"
        ],
        "行顺序原样照抄后端"
    );
    assert_eq!(board.rows[2].span, 0.3, "越级跨度直接用 over_level");
    assert_eq!(board.rows[2].band, Some(OverBand::Mild));
    assert!(board.rows.iter().all(|row| row.has_over_level));
}

#[test]
fn case_401() {
    let mut records = Vec::new();
    for song in ["bbb", "aaa", "ccc"] {
        for difficulty in [Difficulty::Future, Difficulty::Past] {
            records.push(scored(song, difficulty, 10.0, FULL_SCORE, 0, 0));
        }
    }
    let input = BoardInput::new(records);
    let board = select_pm50(&input);
    let keys: Vec<(String, Difficulty)> = board
        .rows
        .iter()
        .map(|row| (row.song_id.clone(), row.difficulty))
        .collect();
    let expected: Vec<(String, Difficulty)> = vec![
        ("aaa".to_string(), Difficulty::Past),
        ("aaa".to_string(), Difficulty::Future),
        ("bbb".to_string(), Difficulty::Past),
        ("bbb".to_string(), Difficulty::Future),
        ("ccc".to_string(), Difficulty::Past),
        ("ccc".to_string(), Difficulty::Future),
    ];
    assert_eq!(keys, expected, "分数完全并列时结果也必须唯一");
    let ranks: Vec<usize> = board.rows.iter().map(|row| row.rank).collect();
    assert_eq!(ranks, vec![1, 2, 3, 4, 5, 6], "名次从 1 开始连续");
}

#[test]
fn case_402() {
    let mut broken = scored("broken", Difficulty::Future, 11.5, 7_500_000, 0, 40);
    broken.backend_pressure = Some(100.0);
    let input = BoardInput::new(vec![
        scored("near", Difficulty::Future, 10.0, 9_790_000, 1, 0),
        broken,
    ]);
    let board = select_ol50(&input);
    assert_eq!(board.rows[0].song_id, "near", "不再按压力重排");
    assert!(board.rows.iter().all(|row| row.band.is_none()));
    assert!(board.rows.iter().all(|row| row.span == 0.0));
    assert_eq!(board.rows[1].pressure, 100.0, "压力取后端下发的值");
    assert_eq!(board.band_counts.total(), 0);
}

#[test]
fn case_403() {
    let input = BoardInput::new(vec![
        scored("abandon", Difficulty::Future, 12.0, 6_000_000, 0, 100),
        scored("kept", Difficulty::Future, 12.0, 7_000_001, 0, 100),
    ])
    .with_anchor(10.0);
    let board = select_ol50(&input);
    assert_eq!(board.known, 2);
    assert_eq!(board.matched, 1, "700 万是候选池的硬门槛");
    assert_eq!(board.rows[0].song_id, "kept");
}

#[test]
fn case_404() {
    let input = BoardInput::default();
    for kind in BoardKind::ALL {
        let board = select_board(&input, kind, BOARD_TOP);
        assert!(board.rows.is_empty(), "{kind} 在空输入下不该有行");
        assert_eq!(board.matched, 0);
        assert_eq!(board.known, 0);
        assert_eq!(board.band_counts.total(), 0);
    }
}

#[test]
fn case_405() {
    assert_eq!(
        band_of(0.3, 9_900_000),
        Some(OverBand::Held),
        "越级跨度达标又打 EX+ 是攻克"
    );
    assert_eq!(
        band_of(0.3, 9_800_000),
        Some(OverBand::Normal),
        "越级跨度达标只打 EX 是达标"
    );
    assert_eq!(
        band_of(0.3, 9_899_999),
        Some(OverBand::Normal),
        "EX 到 EX+ 之间还是达标"
    );
    assert_eq!(
        band_of(0.01, 9_960_000),
        Some(OverBand::Normal),
        "跨度不到 0.05 的 EX+ 是达标, 不是攻克"
    );
    assert_eq!(
        band_of(0.0, 9_900_000),
        Some(OverBand::Normal),
        "跨度缺省 0 的 EX+ 同样是达标"
    );
    assert_eq!(
        band_of(0.01, 9_820_000),
        Some(OverBand::Normal),
        "跨度不到 0.05 打 EX 仍是达标"
    );
    assert_eq!(
        band_of(0.01, 9_780_000),
        Some(OverBand::Underperform),
        "跨度不到 0.05 又没 EX 仍是翻车"
    );
    assert_eq!(
        band_of(-0.9, 9_300_000),
        None,
        "线下一大截又没 EX 的照旧不进榜"
    );

    let mut held = scored("held", Difficulty::Future, 11.0, 9_940_000, 1, 0);
    held.over_level = Some(0.4);
    held.over_band = Some(OverBand::Held);
    held.has_over_level = true;
    let mut near = scored("near", Difficulty::Future, 11.0, 9_940_000, 1, 0);
    near.over_level = Some(0.01);
    near.has_over_level = true;
    let board = select_ol50(
        &BoardInput::new(vec![held, near])
            .with_curve(true)
            .with_anchor(11.0),
    );
    assert_eq!(board.rows[0].band_word(), "攻克", "越线又 EX+ 的格写攻克");
    assert_eq!(
        board.rows[1].band_word(),
        "达标",
        "线内的 EX+ 不许被本地复算改写成攻克"
    );
    assert_eq!(board.band_counts.held, 1);
    assert_eq!(board.band_counts.normal, 1);
}
