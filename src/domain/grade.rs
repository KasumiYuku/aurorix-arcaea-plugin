use crate::model::{Difficulty, Grade};

pub const ASSET_ROOT: &str = "assets";

pub const GRADE_DIR: &str = "grade";

pub const DIFFICULTY_DIR: &str = "difficulty";

pub const CLEAR_TYPE_DIR: &str = "clear_type";

pub const GRADE_EX_PLUS_ASSET: &str = "grade/EX+.png";

pub const GRADE_AA_ASSET: &str = "grade/AA.png";

pub const DIFFICULTY_ASSET: [(Difficulty, &str); 5] = [
    (Difficulty::Past, "difficulty/max-recall-past.png"),
    (Difficulty::Present, "difficulty/max-recall-present.png"),
    (Difficulty::Future, "difficulty/max-recall-future.png"),
    (Difficulty::Beyond, "difficulty/max-recall-beyond.png"),
    (Difficulty::Eternal, "difficulty/max-recall-eternal.png"),
];

pub const DIFFICULTY_INSCRIBED_ASSET: &str = "difficulty/max-recall-inscribed.png";

pub const CLEAR_FAIL_ASSET: &str = "clear_type/clear_fail.png";

pub const CLEAR_NORMAL_ASSET: &str = "clear_type/clear_normal.png";

pub const CLEAR_FULL_ASSET: &str = "clear_type/clear_full.png";

pub const CLEAR_PURE_ASSET: &str = "clear_type/clear_pure.png";

pub const RATING_FRAME_BOUNDS: [f64; 8] = [3.5, 7.0, 10.0, 11.0, 12.0, 12.5, 13.0, 13.5];

pub fn rating_frame_bounds() -> Vec<f64> {
    RATING_FRAME_BOUNDS.to_vec()
}

pub fn rating_frame_of(ptt: f64) -> u8 {
    for (index, bound) in RATING_FRAME_BOUNDS.iter().enumerate() {
        if ptt < *bound {
            return index as u8;
        }
    }
    RATING_FRAME_BOUNDS.len() as u8
}

pub fn grade_asset(grade: Grade) -> &'static str {
    match grade {
        Grade::Pm | Grade::Fr | Grade::ExPlus => GRADE_EX_PLUS_ASSET,
        Grade::Ex => "grade/EX.png",
        Grade::Aa => GRADE_AA_ASSET,
        Grade::A => "grade/A.png",
        Grade::B => "grade/B.png",
        Grade::C => "grade/C.png",
        Grade::D => "grade/D.png",
    }
}

pub fn grade_assets() -> Vec<(Grade, &'static str)> {
    Grade::ALL
        .iter()
        .map(|grade| (*grade, grade_asset(*grade)))
        .collect()
}

pub fn grade_asset_for_label(label: &str) -> Option<&'static str> {
    Grade::parse(label).map(grade_asset)
}

pub fn grade_asset_for_score(score: i64) -> &'static str {
    grade_asset(Grade::from_score(score))
}

pub fn grade_select_asset(grade: Grade) -> &'static str {
    match grade {
        Grade::Pm | Grade::Fr | Grade::ExPlus => "songselect/grade/explus.png",
        Grade::Ex => "songselect/grade/ex.png",
        Grade::Aa => "songselect/grade/aa.png",
        Grade::A => "songselect/grade/a.png",
        Grade::B => "songselect/grade/b.png",
        Grade::C => "songselect/grade/c.png",
        Grade::D => "songselect/grade/d.png",
    }
}

pub fn grade_select_asset_for_label(label: &str) -> &'static str {
    Grade::parse(label).map(grade_select_asset).unwrap_or_default()
}

pub fn grade_file_name(grade: Grade) -> &'static str {
    grade_asset(grade).rsplit('/').next().unwrap_or_default()
}

pub fn grade_file_name_for_label(label: &str) -> &'static str {
    Grade::parse(label).map(grade_file_name).unwrap_or_default()
}

pub fn difficulty_asset(difficulty: Difficulty) -> &'static str {
    DIFFICULTY_ASSET
        .iter()
        .find(|(candidate, _)| *candidate == difficulty)
        .map(|(_, path)| *path)
        .unwrap_or("difficulty/max-recall-future.png")
}

pub fn clear_asset(clear_type: i32) -> &'static str {
    match clear_type {
        0 => CLEAR_FAIL_ASSET,
        2 => CLEAR_FULL_ASSET,
        3 | 6 => CLEAR_PURE_ASSET,
        _ => CLEAR_NORMAL_ASSET,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grade_asset_covers_every_grade() {
        assert_eq!(Grade::ALL.len(), 9, "评级枚举一共九档");
        assert_eq!(grade_assets().len(), 9, "九档必须一一对应");
        let mut seen: Vec<Grade> = grade_assets().into_iter().map(|(grade, _)| grade).collect();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), 9, "同一档不许重复出现");
    }

    #[test]
    fn judgement_grades_borrow_ex_plus() {
        assert_eq!(grade_asset(Grade::Pm), GRADE_EX_PLUS_ASSET);
        assert_eq!(grade_asset(Grade::Fr), GRADE_EX_PLUS_ASSET);
        assert_eq!(grade_asset(Grade::ExPlus), GRADE_EX_PLUS_ASSET);
        assert_eq!(grade_asset(Grade::Ex), "grade/EX.png");
        assert_eq!(grade_asset(Grade::Aa), GRADE_AA_ASSET);
        assert_eq!(grade_asset(Grade::A), "grade/A.png");
        assert_eq!(grade_asset(Grade::B), "grade/B.png");
        assert_eq!(grade_asset(Grade::C), "grade/C.png");
        assert_eq!(grade_asset(Grade::D), "grade/D.png");
    }

    #[test]
    fn only_d_grade_uses_d_asset() {
        for (grade, path) in grade_assets() {
            if grade == Grade::D {
                assert_eq!(path, "grade/D.png");
                continue;
            }
            assert_ne!(
                path, "grade/D.png",
                "{grade} 不许掉到 D 的素材上, 评级与素材必须一一对应"
            );
        }
    }

    #[test]
    fn grade_asset_files_exist() {
        let root = crate::assets_root();
        if !root.is_dir() {
            return;
        }
        for (grade, path) in grade_assets() {
            assert!(
                root.join(path).is_file(),
                "评级 {grade} 的素材 {path} 不在素材仓库里"
            );
            assert_eq!(GRADE_DIR, path.split('/').next().unwrap_or_default());
        }
    }

    #[test]
    fn grade_select_asset_files_exist() {
        let root = crate::assets_root();
        if !root.is_dir() {
            return;
        }
        for grade in Grade::ALL {
            let path = grade_select_asset(grade);
            assert!(root.join(path).is_file(), "选曲页评级素材 {path} 缺失");
        }
    }

    #[test]
    fn score_grade_asset_only_uses_score_bands() {
        for score in [0, 8_600_000, 8_900_000, 9_200_000, 9_500_000, 9_800_000, 9_900_000] {
            let grade = Grade::from_score(score);
            assert!(
                Grade::SCORE.contains(&grade),
                "分数 {score} 只许出七档分数评级"
            );
            assert_eq!(grade_asset_for_score(score), grade_asset(grade));
        }
    }

    #[test]
    fn grade_asset_for_score_matches_thresholds() {
        assert_eq!(grade_asset_for_score(9_550_000), GRADE_AA_ASSET);
        assert_eq!(grade_asset_for_score(9_200_000), "grade/A.png");
        assert_eq!(grade_asset_for_score(9_800_000), "grade/EX.png");
        assert_eq!(grade_asset_for_score(9_899_999), "grade/EX.png");
        assert_eq!(grade_asset_for_score(9_900_000), GRADE_EX_PLUS_ASSET);
        assert_eq!(grade_asset_for_score(10_000_000), GRADE_EX_PLUS_ASSET);
        assert_eq!(grade_asset_for_score(8_600_000), "grade/C.png");
        assert_eq!(grade_asset_for_score(8_900_000), "grade/B.png");
        assert_eq!(grade_asset_for_score(8_000_000), "grade/D.png");
    }

    #[test]
    fn grade_asset_for_label_matches_enum() {
        for grade in Grade::ALL {
            assert_eq!(
                grade_asset_for_label(grade.as_str()),
                Some(grade_asset(grade)),
                "标签 {} 必须换回同一张图",
                grade.as_str()
            );
            assert_eq!(
                grade_select_asset_for_label(grade.as_str()),
                grade_select_asset(grade)
            );
        }
        assert_eq!(grade_asset_for_label("pm"), Some(GRADE_EX_PLUS_ASSET));
        assert_eq!(grade_asset_for_label("fr"), Some(GRADE_EX_PLUS_ASSET));
        assert_eq!(grade_asset_for_label("aa"), Some(GRADE_AA_ASSET));
        assert_eq!(grade_asset_for_label("乱写"), None, "认不出的评级不许瞎猜");
        assert_eq!(grade_select_asset_for_label("乱写"), "");
    }

    #[test]
    fn difficulty_assets_are_complete() {
        assert_eq!(
            difficulty_asset(Difficulty::Past),
            "difficulty/max-recall-past.png"
        );
        assert_eq!(
            difficulty_asset(Difficulty::Present),
            "difficulty/max-recall-present.png"
        );
        assert_eq!(
            difficulty_asset(Difficulty::Future),
            "difficulty/max-recall-future.png"
        );
        assert_eq!(
            difficulty_asset(Difficulty::Beyond),
            "difficulty/max-recall-beyond.png"
        );
        assert_eq!(
            difficulty_asset(Difficulty::Eternal),
            "difficulty/max-recall-eternal.png"
        );
        let root = crate::assets_root();
        if root.is_dir() {
            for (difficulty, path) in DIFFICULTY_ASSET {
                assert!(
                    root.join(path).is_file(),
                    "难度 {difficulty} 的素材 {path} 不在素材仓库里"
                );
            }
        }
    }

    #[test]
    fn clear_assets_follow_clear_type() {
        assert_eq!(clear_asset(0), "clear_type/clear_fail.png");
        assert_eq!(clear_asset(1), "clear_type/clear_normal.png");
        assert_eq!(clear_asset(2), "clear_type/clear_full.png");
        assert_eq!(clear_asset(3), "clear_type/clear_pure.png");
        assert_eq!(clear_asset(6), "clear_type/clear_pure.png");
        assert_eq!(
            clear_asset(99),
            "clear_type/clear_normal.png",
            "未知的通关类型退回普通"
        );
    }

    #[test]
    fn rating_frame_uses_half_open_bands() {
        for (ptt, want) in [
            (0.0, 0u8),
            (3.499, 0),
            (3.5, 1),
            (6.999, 1),
            (7.0, 2),
            (10.999, 3),
            (11.0, 4),
            (11.993, 4),
            (11.999, 4),
            (12.0, 5),
            (12.49, 5),
            (12.5, 6),
            (12.999, 6),
            (13.0, 7),
            (13.499, 7),
            (13.5, 8),
            (14.5, 8),
        ] {
            assert_eq!(rating_frame_of(ptt), want, "PTT {ptt} 的段位框");
        }
        assert_eq!(RATING_FRAME_BOUNDS.len(), 8, "八条分界给九档");
    }
}
