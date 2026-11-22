use aurorix_arcaea_plugin::backend::ScoreRecord;
use aurorix_arcaea_plugin::commands::bridge;
use aurorix_arcaea_plugin::domain::grade_asset_for_score;
use aurorix_arcaea_plugin::Grade;

fn record(score: i32, far: i32, lost: i32, rank: &str) -> ScoreRecord {
    ScoreRecord {
        score,
        far,
        lost,
        rank: rank.to_string(),
        has_judgement: Some(true),
        ..ScoreRecord::default()
    }
}

#[test]
fn score_badge_follows_score_not_rank() {
    let cases = [
        (9_850_000, 3, 0, "FR", "grade/EX.png"),
        (9_950_000, 3, 0, "FR", "grade/EX+.png"),
        (10_000_000, 0, 0, "PM", "grade/EX+.png"),
        (9_650_000, 1, 0, "FR", "grade/AA.png"),
        (9_200_000, 0, 9, "A", "grade/A.png"),
        (8_900_000, 0, 9, "B", "grade/B.png"),
        (8_600_000, 0, 9, "C", "grade/C.png"),
        (8_000_000, 0, 9, "D", "grade/D.png"),
    ];
    for (score, far, lost, rank, want) in cases {
        let item = record(score, far, lost, rank);
        let got = grade_asset_for_score(i64::from(score));
        assert_eq!(got, want, "{score} 分（{rank}）的分数徽章");
        assert_eq!(bridge::grade_of(&item).as_str(), score_label(want));
    }
}

#[test]
fn achievement_keeps_judgement_bands() {
    for (score, far, lost, rank, want) in [
        (9_850_000, 3, 0, "FR", Grade::Fr),
        (9_950_000, 3, 0, "FR", Grade::Fr),
        (10_000_000, 0, 0, "PM", Grade::Pm),
        (9_950_000, 3, 1, "EX+", Grade::ExPlus),
        (9_650_000, 0, 0, "AA", Grade::Aa),
    ] {
        let item = record(score, far, lost, rank);
        assert_eq!(
            bridge::achievement_of(&item),
            want,
            "{score} 分（{rank}）的成就等级"
        );
    }
}

#[test]
fn achievement_falls_back_to_judgement_when_backend_silent() {
    assert_eq!(
        bridge::achievement_of(&record(9_850_000, 3, 0, "")),
        Grade::Fr,
        "后端没发 rank 时, 零 lost 且有 far 就是 FR"
    );
    assert_eq!(
        bridge::achievement_of(&record(10_000_000, 0, 0, "")),
        Grade::Pm
    );
    assert_eq!(
        bridge::achievement_of(&record(9_000_000, 4, 2, "")),
        Grade::B,
        "有 lost 时退回分数档"
    );
}

#[test]
fn only_d_grade_draws_d_asset() {
    for grade in Grade::SCORE {
        let score = match grade {
            Grade::ExPlus => 9_950_000,
            Grade::Ex => 9_850_000,
            Grade::Aa => 9_600_000,
            Grade::A => 9_300_000,
            Grade::B => 8_950_000,
            Grade::C => 8_700_000,
            _ => 8_000_000,
        };
        let path = grade_asset_for_score(score);
        if grade == Grade::D {
            assert_eq!(path, "grade/D.png");
        } else {
            assert_ne!(path, "grade/D.png", "{grade} 不许掉到 D 的素材上");
        }
    }
}

#[test]
fn ahv_record_carries_real_achievement() {
    for (score, far, lost, rank, want) in [
        (10_000_000, 0, 0, "PM", "PM"),
        (9_950_000, 3, 0, "FR", "FR"),
        (9_950_000, 0, 2, "EX+", "EX+"),
        (9_300_000, 0, 4, "A", "A"),
    ] {
        let item = record(score, far, lost, rank);
        let ahv = bridge::ahv_record_of(&item);
        assert_eq!(
            ahv.grade.map(Grade::as_str),
            Some(want),
            "{score} 分（{rank}）的折角标签必须是真实评级"
        );
    }
}

fn score_label(asset: &str) -> &'static str {
    match asset {
        "grade/EX+.png" => "EX+",
        "grade/EX.png" => "EX",
        "grade/AA.png" => "AA",
        "grade/A.png" => "A",
        "grade/B.png" => "B",
        "grade/C.png" => "C",
        "grade/D.png" => "D",
        other => panic!("未知素材 {other}"),
    }
}
