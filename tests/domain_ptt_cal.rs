use aurorix_arcaea_plugin::domain::ptt;
use aurorix_arcaea_plugin::domain::{grade_asset_for_label, grade_asset_for_score};
use aurorix_arcaea_plugin::model::Grade;

fn close(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-6
}

#[test]
fn case_442() {
    let result = ptt::parse_cal_args(&["10.5", "9950000"]).expect("合法参数应当换算成功");
    assert!(
        close(result.rating, 12.45),
        "10.5 定数 995 万的单曲 PTT 是 12.45，实际 {}",
        result.rating
    );
    assert!(close(result.achievement, 99.5), "达成率是 99.5%");
    assert_eq!(result.grade, Grade::ExPlus);
    assert_eq!(result.grade_label, "EX+");
    assert_eq!(result.next_grade_label, "", "995 万之上只有满分");
    assert_eq!(result.next_grade_need, 50_000);
    assert!(close(result.max_rating, 12.7));
}

#[test]
fn case_443() {
    let result = ptt::cal(3.0, 0);
    assert_eq!(result.rating, 0.0, "算不出来时要夹到 0，不许出现负数");
    assert_eq!(result.grade, Grade::D);
    assert_eq!(result.next_grade_label, "C");
    assert_eq!(result.next_grade_need, 8_600_000);
}

#[test]
fn case_444() {
    let best = vec![12.5; 30];
    let recent = vec![11.0; 10];
    let expected = (12.5 * 30.0 + 11.0 * 10.0) / 40.0;
    assert!(
        close(ptt::total_ptt(&best, &recent), expected),
        "总 PTT 应为 {}，实际 {}",
        expected,
        ptt::total_ptt(&best, &recent)
    );
    assert_eq!(
        ptt::total_ptt(&[], &[]),
        0.0,
        "一条成绩都没有时是 0，不是 NaN"
    );
    assert_eq!(ptt::best30(&[]), None);
    assert_eq!(ptt::recent10(&[]), None);
    assert_eq!(ptt::b50_weighted_ptt(&[]), 0.0);
}

#[test]
fn case_445() {
    let ratings = vec![12.0, 11.0];
    assert!(close(
        ptt::best30(&ratings).expect("两条也算得出来"),
        23.0 / 30.0
    ));
    assert!(close(
        ptt::recent10(&ratings).expect("两条也算得出来"),
        23.0 / 10.0
    ));
}

#[test]
fn case_446() {
    let cases = [
        (0, "D"),
        (8_600_000, "C"),
        (8_900_000, "B"),
        (9_200_000, "A"),
        (9_500_000, "AA"),
        (9_800_000, "EX"),
        (9_900_000, "EX+"),
        (10_000_000, "EX+"),
    ];
    for (score, label) in cases {
        assert_eq!(
            ptt::score_grade_label(score),
            label,
            "分数 {score} 的评级应当是 {label}"
        );
    }
    assert_eq!(
        ptt::grade_of(9_500_000),
        Grade::Aa,
        "9.5 百万是独立的 AA 档"
    );
    assert_eq!(ptt::grade_of(9_900_000), Grade::ExPlus);
    assert_eq!(ptt::grade_of(9_800_000), Grade::Ex);
    assert_eq!(ptt::grade_of(8_900_000), Grade::B);
    assert_eq!(ptt::grade_of(8_600_000), Grade::C);
    assert_eq!(ptt::grade_of(0), Grade::D);
}

#[test]
fn case_447() {
    assert_eq!(grade_asset_for_score(9_550_000), "grade/AA.png");
    assert_eq!(grade_asset_for_score(9_250_000), "grade/A.png");
    assert_eq!(grade_asset_for_score(9_800_000), "grade/EX.png");
    assert_eq!(grade_asset_for_score(9_950_000), "grade/EX+.png");
    assert_eq!(grade_asset_for_label("PM"), Some("grade/EX+.png"));
    assert_eq!(grade_asset_for_label("FR"), Some("grade/EX+.png"));
    assert_eq!(grade_asset_for_label("不认识"), None);
}

#[test]
fn case_448() {
    assert_eq!(ptt::format_rating(12.982_715), "12.9827");
    assert_eq!(ptt::format_rating(13.199_995), "13.1999");
    assert_eq!(ptt::format_rating(11.2), "11.2000");
}
