use crate::model::Grade;

pub const FULL_SCORE: i64 = 10_000_000;
pub const EX_SCORE: i64 = 9_800_000;
pub const EX_PLUS_SCORE: i64 = 9_900_000;
pub const CLEAR_SCORE: i64 = 7_000_000;
pub const CLEAR_BONUS: f64 = 0.2;
pub const MAX_BONUS: f64 = 2.2;
pub const PASS_BONUS: f64 = 1.2;
pub const B30_SIZE: usize = 30;
pub const R10_SIZE: usize = 10;
pub const PTT_DIVISOR: f64 = 40.0;
pub const B50_WINDOW: usize = 50;
pub const B50_DIVISOR: f64 = 60.0;
pub const EPS: f64 = 1e-9;

pub use crate::model::GRADE_THRESHOLDS;

pub fn round9(value: f64) -> f64 {
    (value * 1e9).round() / 1e9
}

pub fn single_rating(constant: f64, score: i64) -> f64 {
    let rating = if score >= FULL_SCORE {
        constant + 2.0
    } else if score > EX_SCORE {
        constant + 1.0 + (score - EX_SCORE) as f64 / 200_000.0
    } else {
        (constant + (score - 9_500_000) as f64 / 300_000.0).max(0.0)
    };
    let rating = if score >= CLEAR_SCORE {
        rating + CLEAR_BONUS
    } else {
        rating
    };
    round9(rating)
}

pub fn max_rating(constant: f64) -> f64 {
    round9(constant + MAX_BONUS)
}

pub fn sorted_desc(values: &[f64]) -> Vec<f64> {
    let mut out = values.to_vec();
    out.sort_by(|left, right| right.total_cmp(left));
    out
}

pub fn best_average(values: &[f64], window: usize) -> Option<f64> {
    if values.is_empty() || window == 0 {
        return None;
    }
    let ordered = sorted_desc(values);
    let taken = ordered.len().min(window);
    let sum: f64 = ordered[..taken].iter().sum();
    Some(sum / window as f64)
}

pub fn best30(values: &[f64]) -> Option<f64> {
    best_average(values, B30_SIZE)
}

pub fn recent10(values: &[f64]) -> Option<f64> {
    best_average(values, R10_SIZE)
}

pub fn best_window_sum(values: &[f64], window: usize) -> f64 {
    let ordered = sorted_desc(values);
    ordered.iter().take(window).sum()
}

pub fn total_ptt(best: &[f64], recent: &[f64]) -> f64 {
    let best_sum = best_window_sum(best, B30_SIZE);
    let recent_sum = best_window_sum(recent, R10_SIZE);
    if best.is_empty() && recent.is_empty() {
        return 0.0;
    }
    round9((best_sum + recent_sum) / PTT_DIVISOR)
}

pub fn b50_weighted_ptt(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let ordered = sorted_desc(values);
    let mut sum = 0.0;
    for (index, rating) in ordered.iter().take(B50_WINDOW).enumerate() {
        if index < R10_SIZE {
            sum += 2.0 * rating;
        } else {
            sum += rating;
        }
    }
    round9(sum / B50_DIVISOR)
}

pub fn score_grade_label(score: i64) -> &'static str {
    Grade::from_score(score).as_str()
}

pub fn grade_of(score: i64) -> Grade {
    Grade::from_score(score)
}

pub fn grade_for(score: i64, far: i64, lost: i64, judged: bool) -> Grade {
    if judged {
        if lost == 0 && far == 0 {
            return Grade::Pm;
        }
        if lost == 0 && far > 0 {
            return Grade::Fr;
        }
    }
    grade_of(score)
}

pub fn next_grade(score: i64) -> Option<(&'static str, i64)> {
    if score >= FULL_SCORE {
        return None;
    }
    for (grade, min) in GRADE_THRESHOLDS {
        if score < min {
            return Some((grade.as_str(), min - score));
        }
    }
    Some(("", FULL_SCORE - score))
}

pub fn achievement(score: i64) -> f64 {
    let percent = score as f64 / FULL_SCORE as f64 * 100.0;
    (percent * 1e4).round() / 1e4
}

pub fn format_rating(value: f64) -> String {
    let mul = 1e4;
    let epsilon = value.abs() * mul * f64::EPSILON * 8.0 + 1e-9;
    format!("{:.4}", (value * mul + epsilon).floor() / mul)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalInput {
    pub constant: f64,
    pub score: i64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalResult {
    pub constant: f64,
    pub score: i64,
    pub rating: f64,
    pub max_rating: f64,
    pub achievement: f64,
    pub grade: Grade,
    pub grade_label: &'static str,
    pub next_grade_label: &'static str,
    pub next_grade_need: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CalError {
    #[error("用法：/a cal <定数> <分数>，例如 /a cal 10.5 9950000")]
    参数不足,
    #[error("定数写法看不懂：{0}（写成 10.5 这样的小数）")]
    定数(String),
    #[error("分数写法看不懂：{0}（写成 9950000 这样的整数）")]
    分数(String),
}

fn strip_separators(text: &str) -> String {
    text.chars()
        .filter(|character| *character != ',' && *character != '_' && *character != '\'')
        .collect()
}

pub fn parse_constant(text: &str) -> Result<f64, CalError> {
    let cleaned = strip_separators(text.trim());
    match cleaned.parse::<f64>() {
        Ok(value) if value.is_finite() && value > 0.0 => Ok(value),
        _ => Err(CalError::定数(text.to_string())),
    }
}

pub fn parse_score(text: &str) -> Result<i64, CalError> {
    let cleaned = strip_separators(text.trim());
    match cleaned.parse::<i64>() {
        Ok(value) if (0..=FULL_SCORE).contains(&value) => Ok(value),
        _ => Err(CalError::分数(text.to_string())),
    }
}

pub fn parse_cal_args(args: &[&str]) -> Result<CalResult, CalError> {
    let words: Vec<&str> = args
        .iter()
        .map(|word| word.trim())
        .filter(|word| !word.is_empty())
        .collect();
    if words.len() < 2 {
        return Err(CalError::参数不足);
    }
    let constant = parse_constant(words[0])?;
    let score = parse_score(words[1])?;
    Ok(cal(constant, score))
}

pub fn cal(constant: f64, score: i64) -> CalResult {
    let (next_grade_label, next_grade_need) = next_grade(score).unwrap_or(("", 0));
    CalResult {
        constant,
        score,
        rating: single_rating(constant, score),
        max_rating: max_rating(constant),
        achievement: achievement(score),
        grade: grade_of(score),
        grade_label: score_grade_label(score),
        next_grade_label,
        next_grade_need,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(left: f64, right: f64) -> bool {
        (left - right).abs() <= EPS
    }

    #[test]
    fn case_326() {
        assert!(
            close(single_rating(10.5, FULL_SCORE), 12.7),
            "满分应当是 10.5 + 2 + 0.2 = 12.7，实际 {}",
            single_rating(10.5, FULL_SCORE)
        );
        assert!(close(single_rating(0.0, FULL_SCORE), MAX_BONUS));
    }

    #[test]
    fn case_327() {
        assert!(
            close(single_rating(10.0, EX_SCORE), 11.2),
            "正好 EX 只拿 1.0 加成"
        );
        assert!(close(single_rating(10.0, 9_900_000), 11.7));
        assert!(close(single_rating(10.0, 9_999_999), 12.199_995));
    }

    #[test]
    fn case_328() {
        assert!(close(single_rating(10.0, 9_500_000), 10.2));
        assert!(close(single_rating(10.0, 7_000_000), 1.866_666_667));
        assert_eq!(
            single_rating(3.0, 0),
            0.0,
            "低到算不出来时必须夹到 0，不能是负数"
        );
        assert_eq!(single_rating(3.0, 6_999_999), 0.0, "通关线以下没有通关加成");
    }

    #[test]
    fn case_329() {
        let without = single_rating(9.0, CLEAR_SCORE - 1);
        let with = single_rating(9.0, CLEAR_SCORE);
        assert!(
            close(with - without, CLEAR_BONUS + 1.0 / 300_000.0),
            "通关线两侧差的是 0.2 加成加一分分数，实际 {}",
            with - without
        );
    }

    #[test]
    fn case_330() {
        let best = vec![12.0, 11.0, 10.0];
        assert!(
            close(best30(&best).unwrap(), 33.0 / 30.0),
            "不足 30 条也只除以 30"
        );
        let recent = vec![12.0];
        assert!(
            close(recent10(&recent).unwrap(), 1.2),
            "不足 10 条也只除以 10"
        );
    }

    #[test]
    fn case_331() {
        assert_eq!(best30(&[]), None);
        assert_eq!(recent10(&[]), None);
        assert_eq!(total_ptt(&[], &[]), 0.0, "一条成绩都没有时总 PTT 是 0");
        assert_eq!(b50_weighted_ptt(&[]), 0.0);
    }

    #[test]
    fn case_332() {
        let best: Vec<f64> = (0..30).map(|index| 12.0 - index as f64 * 0.1).collect();
        let recent: Vec<f64> = (0..10).map(|index| 10.0 - index as f64 * 0.1).collect();
        let expected = (best.iter().sum::<f64>() + recent.iter().sum::<f64>()) / 40.0;
        assert!(
            close(total_ptt(&best, &recent), expected),
            "30:10 合成应等于 (b30 和 + r10 和) / 40，实际 {}",
            total_ptt(&best, &recent)
        );
        let full = vec![12.0; 30];
        let full_recent = vec![12.0; 10];
        assert!(close(total_ptt(&full, &full_recent), 12.0));
    }

    #[test]
    fn case_333() {
        let recent = vec![11.0; 10];
        assert!(
            close(total_ptt(&[], &recent), 11.0 / 4.0),
            "只有 R10 时只除以 40"
        );
    }

    #[test]
    fn case_334() {
        let ratings: Vec<f64> = (0..50).map(|index| 12.0 - index as f64 * 0.01).collect();
        let mut expected = 0.0;
        for (index, rating) in ratings.iter().enumerate() {
            expected += if index < 10 { 2.0 * rating } else { *rating };
        }
        assert!(close(b50_weighted_ptt(&ratings), expected / 60.0));
    }

    #[test]
    fn case_335() {
        assert_eq!(score_grade_label(0), "D");
        assert_eq!(score_grade_label(8_599_999), "D");
        assert_eq!(score_grade_label(8_600_000), "C");
        assert_eq!(score_grade_label(8_900_000), "B");
        assert_eq!(score_grade_label(9_200_000), "A");
        assert_eq!(score_grade_label(9_500_000), "AA");
        assert_eq!(score_grade_label(9_800_000), "EX");
        assert_eq!(score_grade_label(9_900_000), "EX+");
        assert_eq!(score_grade_label(FULL_SCORE), "EX+");
    }

    #[test]
    fn case_336() {
        assert_eq!(grade_of(9_500_000), Grade::Aa, "9.5 百万是独立的 AA 档");
        assert_eq!(grade_of(9_200_000), Grade::A);
        assert_eq!(grade_of(8_900_000), Grade::B);
        assert_eq!(grade_of(8_600_000), Grade::C);
        assert_eq!(grade_of(0), Grade::D);
        assert_eq!(grade_of(9_900_000), Grade::ExPlus);
        assert_eq!(grade_of(9_800_000), Grade::Ex);
        for (grade, min) in GRADE_THRESHOLDS {
            assert_eq!(grade_of(min), grade, "阈值 {min} 必须正好是 {grade}");
        }
        let covered: Vec<Grade> = GRADE_THRESHOLDS.iter().map(|(grade, _)| *grade).collect();
        for grade in Grade::SCORE {
            assert!(covered.contains(&grade), "评级 {grade} 必须有阈值覆盖");
        }
        for grade in [Grade::Pm, Grade::Fr] {
            assert!(!covered.contains(&grade), "{grade} 是判定档, 不该进阈值表");
        }
    }

    #[test]
    fn case_337() {
        assert_eq!(
            GRADE_THRESHOLDS,
            [
                (Grade::D, 0),
                (Grade::C, 8_600_000),
                (Grade::B, 8_900_000),
                (Grade::A, 9_200_000),
                (Grade::Aa, 9_500_000),
                (Grade::Ex, 9_800_000),
                (Grade::ExPlus, 9_900_000),
            ],
            "阈值必须由低到高排列，覆盖七档分数评级"
        );
    }

    #[test]
    fn case_338() {
        let cases: [(i64, &str, &str); 7] = [
            (0, "D", "D"),
            (8_600_000, "C", "D"),
            (8_900_000, "B", "C"),
            (9_200_000, "A", "B"),
            (9_500_000, "AA", "A"),
            (9_800_000, "EX", "AA"),
            (9_900_000, "EX+", "EX"),
        ];
        for (min, at, below) in cases {
            assert_eq!(
                score_grade_label(min),
                at,
                "阈值 {min} 这一分应当正好是 {at}"
            );
            assert_eq!(
                score_grade_label(min + 1),
                at,
                "阈值 {min} 高一分必须还是 {at}"
            );
            assert_eq!(
                grade_of(min),
                Grade::SCORE
                    .into_iter()
                    .find(|grade| grade.as_str() == at)
                    .expect("标签必须能换回枚举"),
                "分数 {min} 的枚举必须与标签 {at} 一致"
            );
            if min > 0 {
                assert_eq!(
                    score_grade_label(min - 1),
                    below,
                    "阈值 {min} 低一分必须掉回 {below}"
                );
                assert_eq!(
                    grade_of(min - 1),
                    Grade::SCORE
                        .into_iter()
                        .find(|grade| grade.as_str() == below)
                        .expect("低一档的标签也必须能换回枚举"),
                    "阈值 {min} 低一分的枚举必须与 {below} 一致"
                );
            }
        }
        assert_eq!(score_grade_label(FULL_SCORE), "EX+");
        assert_eq!(grade_of(FULL_SCORE), Grade::ExPlus);
    }

    #[test]
    fn case_339() {
        assert_eq!(next_grade(9_850_000), Some(("EX+", 50_000)));
        assert_eq!(
            next_grade(9_250_000),
            Some(("AA", 250_000)),
            "A 档的下一档是 AA"
        );
        assert_eq!(next_grade(9_499_999), Some(("AA", 1)), "差一分到 AA");
        assert_eq!(next_grade(FULL_SCORE), None, "满分没有下一档");
        assert_eq!(
            next_grade(9_950_000),
            Some(("", 50_000)),
            "只差满分时下一档是满分"
        );
    }

    #[test]
    fn case_340() {
        assert!(close(achievement(FULL_SCORE), 100.0));
        assert!(close(achievement(9_870_000), 98.7));
        assert!(close(achievement(0), 0.0));
    }

    #[test]
    fn case_341() {
        let result = parse_cal_args(&["10.5", "9950000"]).expect("这两个参数都合法");
        assert!(close(result.constant, 10.5));
        assert_eq!(result.score, 9_950_000);
        assert!(close(result.rating, single_rating(10.5, 9_950_000)));
        assert!(close(result.achievement, 99.5));
        assert_eq!(result.grade_label, "EX+", "995 万是 EX+");
        assert_eq!(result.grade, Grade::ExPlus);
        assert!(close(result.max_rating, 12.7));
    }

    #[test]
    fn case_342() {
        let result = parse_cal_args(&["11", "9,980,000"]).expect("千分位应当被认出来");
        assert_eq!(result.score, 9_980_000);
        assert_eq!(
            parse_cal_args(&["11"]),
            Err(CalError::参数不足),
            "只给一个参数要报用法"
        );
        assert!(matches!(
            parse_cal_args(&["abc", "9980000"]),
            Err(CalError::定数(_))
        ));
        assert!(matches!(
            parse_cal_args(&["11", "10million"]),
            Err(CalError::分数(_))
        ));
        assert!(
            matches!(parse_cal_args(&["11", "20000000"]), Err(CalError::分数(_))),
            "超过满分的分数要拒绝"
        );
    }

    #[test]
    fn case_343() {
        assert_eq!(format_rating(12.982_715), "12.9827");
        assert_eq!(
            format_rating(11.2),
            "11.2000",
            "浮点尾差不能把 11.2 截成 11.1999"
        );
        assert_eq!(format_rating(0.0), "0.0000");
    }
}
