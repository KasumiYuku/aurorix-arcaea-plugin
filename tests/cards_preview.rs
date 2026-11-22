use std::fs;

use aurorix_arcaea_plugin::cards::theme::Theme;
use aurorix_arcaea_plugin::cards::widgets::PlayerBadge;
use aurorix_arcaea_plugin::cards::{
    b50, diff, gallery, guess, harness, lists, score, search, select, song, stats,
};
use aurorix_arcaea_plugin::domain::board::BoardRow;
use aurorix_arcaea_plugin::domain::song::Chart;
use aurorix_arcaea_plugin::domain::suggest::{suggest, SuggestCandidate};
use aurorix_arcaea_plugin::{Difficulty, Grade, Region};
use chrono::{TimeZone, Utc};

const 曲绘: [&str; 8] = [
    "jacket_thumbs/acheron.jpg",
    "jacket_thumbs/prism_prs.jpg",
    "jacket_thumbs/conflict.jpg",
    "jacket_thumbs/aiueoon.jpg",
    "jacket_thumbs/alterego.jpg",
    "jacket_thumbs/ascent.jpg",
    "jacket_thumbs/antithese_byd.jpg",
    "jacket_thumbs/lastcelebration.jpg",
];

const 曲名: [&str; 8] = [
    "Acheron",
    "PRAGMATISM -RESURRECTION-",
    "Conflict",
    "AIUEON",
    "AlterAle",
    "Ascent",
    "Antithese",
    "Last Celebration",
];

fn case_425(index: usize) -> String {
    let key = 曲绘[index % 曲绘.len()];
    key.trim_start_matches("jacket_thumbs/")
        .trim_end_matches(".jpg")
        .to_string()
}

const SAMPLE_AVATAR: &str = "char_icons/阿米娅_amiya_102_icon.png";
const 立绘: &str = "characters/阿米娅_amiya_102.png";
const 横幅: &str = "banners/course_banner_01.png";
const 时间: &str = "2025-11-20 12:34";

fn case_426() -> PlayerBadge {
    PlayerBadge::new("Afkeru")
        .with_ptt(12.34)
        .with_star(6)
        .with_frame(Some(5))
        .with_subtitle("星野ゆめ")
        .with_avatar(SAMPLE_AVATAR)
        .with_banner(横幅)
        .with_character(立绘)
        .with_global_rank(Some(233))
}

fn case_427(index: usize, recent: bool) -> BoardRow {
    let difficulty = match index % 4 {
        0 => Difficulty::Future,
        1 => Difficulty::Beyond,
        2 => Difficulty::Present,
        _ => Difficulty::Eternal,
    };
    BoardRow {
        rank: index + 1,
        song_id: case_425(index),
        title: 曲名[index % 曲名.len()].to_string(),
        difficulty,
        constant: 10.1 + (index % 9) as f64 * 0.1,
        has_constant: true,
        score: 9_800_000 + (index as i64 % 20) * 9_000,
        rating: 11.2 + (index % 7) as f64 * 0.13,
        grade: Grade::ExPlus,
        far: (index as i64 % 5) * 2,
        lost: index as i64 % 3,
        normal_perfect: 18,
        object_amount: 1100 + index as i64 * 7,
        clear_type: if recent { Some(1) } else { Some(0) },
        has_judgement: true,
        band: None,
        span: 0.0,
        pressure: 0.0,
        play_at: Some(Utc.with_ymd_and_hms(2025, 11, 20, 12, 34, 56).unwrap()),
        mess: 0.0,
        over_delta: 0.0,
        has_over_level: false,
        over_level: None,
        over_band: None,
        over_mess: None,
        backend_pressure: None,
    }
}

fn case_254(index: usize) -> SuggestCandidate {
    SuggestCandidate::new(
        format!("song-{index:02}"),
        曲名[index % 曲名.len()],
        if index.is_multiple_of(2) {
            Difficulty::Future
        } else {
            Difficulty::Beyond
        },
        11.4 + (index % 6) as f64 * 0.1,
        1250 + index as i64 * 9,
    )
}

fn case_428() -> Vec<Chart> {
    vec![
        Chart::new(Difficulty::Past, "6", 6.5),
        Chart::new(Difficulty::Present, "8", 8.4),
        Chart::new(Difficulty::Future, "10", 10.4),
        Chart::new(Difficulty::Beyond, "11", 11.2),
    ]
}

#[test]
#[ignore]
fn case_429() {
    let dir = std::path::PathBuf::from("/tmp/arcaea_picture");
    fs::create_dir_all(&dir).expect("样张目录必须可创建");
    let harness = harness::Harness::new();
    let badge = case_426();

    let case_551 = |name: &str, bytes: Vec<u8>| {
        fs::write(dir.join(name), &bytes).expect("样张必须可写入");
        println!("{name} -> {} 字节", bytes.len());
    };

    let b50_input = b50::B50Input {
        badge: badge.clone(),
        theme: Theme::DEFAULT,
        board: None,
        best: (0..50).map(|i| case_427(i, false)).collect(),
        recent: (0..10).map(|i| case_427(i + 50, true)).collect(),
        ptt: Some(12.34),
        note: String::new(),
        generated_at: Some(时间.to_string()),
        rank_frame: Some(7),
        board_stat: None,
        anchor_upper: 0.0,
        over_count: None,
        cn: true,
        best10_avg: None,
        best50_avg: None,
        best_count: None,
        max_ptt: None,
    };
    case_551(
        "01-b50.png",
        harness.render(b50::request(&b50_input), |kit| b50::paint(kit, &b50_input)),
    );

    let score_input = score::ScoreInput {
        badge: badge.clone(),
        theme: Theme::DEFAULT,
        song_id: String::from("acheron"),
        title: String::from("Acheron"),
        artist: String::from("sasakure.UK"),
        difficulty: Difficulty::Future,
        constant: Some(10.4),
        level: String::new(),
        score: 9_912_345,
        grade: Grade::ExPlus,
        pure: 1043,
        far: 7,
        lost: 1,
        shiny_perfect: Some(886),
        normal_perfect: Some(157),
        rating: 12.6,
        ptt: Some(12.34),
        rating_frame: Some(7),
        ptt_delta: Some(0.03),
        previous_rating: Some(12.31),
        clear_type: Some(0),
        skill: String::from("回忆率上限提升 20%"),
        jacket: Some(曲绘[0].to_string()),
        play_at: Some(Utc.with_ymd_and_hms(2025, 11, 20, 12, 34, 56).unwrap()),
        note: String::new(),
        recollection: Some(86.0),
    };
    case_551(
        "02-score.png",
        harness.render(score::score_request(&score_input), |kit| {
            score::paint_score(kit, &score_input)
        }),
    );
    case_551(
        "03-recent.png",
        harness.render(score::recent_request(&score_input), |kit| {
            score::paint_recent(kit, &score_input)
        }),
    );

    let mut 暗色 = score_input.clone();
    暗色.theme = Theme::by_name("arknights");
    case_551(
        "20-score-arknights.png",
        harness.render(score::score_request(&暗色), |kit| {
            score::paint_score(kit, &暗色)
        }),
    );

    let song_input = song::SongInput {
        song_id: String::from("acheron"),
        title: String::from("Acheron"),
        title_ja: String::from("アケローン"),
        aliases: vec![String::from("黑河"), String::from("阿刻戎")],
        artist: String::from("sasakure.UK"),
        bpm: 197.0,
        bpm_label: None,
        release: String::from("2026-01-01"),
        duration: 162,
        pack_id: String::from("single"),
        pack_name: String::from("Memory Archive"),
        pack_name_zh: String::from("回忆档案馆"),
        jacket: Some(曲绘[0].to_string()),
        charts: case_428(),
        ratings: vec![(Difficulty::Future, Grade::ExPlus)],
        theme: Theme::DEFAULT,
        footer_note: String::new(),
        cells: Some(vec![
            (String::from("BPM"), String::from("197")),
            (String::from("时长"), String::from("2:42")),
            (String::from("物量"), String::from("1127")),
            (String::from("谱师"), String::from("Nightmare")),
        ]),
        boards: Some(
            [Difficulty::Past, Difficulty::Present, Difficulty::Future]
                .into_iter()
                .map(|difficulty| song::SongBoard {
                    difficulty,
                    rows: (0..3)
                        .map(|i| song::SongBoardRow {
                            name: format!("玩家 {i}"),
                            avatar: Some(SAMPLE_AVATAR.to_string()),
                            grade: String::from("EX+"),
                            score: 9_950_000 + i as i64 * 9_000,
                            rating: 12.4 + i as f64 * 0.1,
                            has_rating: true,
                            rating_frame: 7,
                            rating_accent: 1,
                            perfect: 1020,
                            normal_perfect: 90,
                            far: 12,
                            lost: 1,
                            judgement: true,
                        })
                        .collect(),
                })
                .collect(),
        ),
    };
    case_551(
        "04-song.png",
        harness.render(song::request(&song_input), |kit| {
            song::paint(kit, &song_input)
        }),
    );

    let select_input = select::SelectInput {
        song_id: String::from("acheron"),
        title: String::from("Acheron"),
        artist: String::from("sasakure.UK"),
        pack_id: String::from("single"),
        pack_name: String::from("Memory Archive"),
        pack_name_zh: String::from("回忆档案馆"),
        jacket: Some(曲绘[0].to_string()),
        backdrop: Some(String::from("background/background1.jpg")),
        charts: case_428(),
        selected: Difficulty::Future,
        score: Some(9_912_345),
        grade: Some(Grade::ExPlus),
        pack_count: 42,
        clear_count: 17,
        pure_count: 6,
        total_count: 68,
        retry_hint: String::from("输入 /a rand 重抽"),
        roll: String::from("seed-20251120"),
        theme: Theme::DEFAULT,
        footer_note: String::new(),
        rows: Some(
            (0..5)
                .map(|i| select::SelectRow {
                    title: 曲名[i].to_string(),
                    song_id: case_425(i),
                    artist: String::from("Various"),
                    difficulty: Difficulty::Future,
                    level: String::from("10"),
                    constant: 10.1 + i as f64 * 0.1,
                    has_constant: true,
                    selected: i == 2,
                    clear: 'P',
                    grade: String::from("EX+"),
                    bpm: format!("{}", 160 + i * 12),
                })
                .collect(),
        ),
        levels: Some(vec![
            String::from("6"),
            String::from("8"),
            String::from("10"),
            String::from("11"),
        ]),
        page_title: Some(String::from("随机选曲")),
        pack_title: Some(String::from("Memory Archive")),
        player: Some(select::SelectPlayer {
            name: String::from("Afkeru"),
            ptt: 12.34,
            rating_frame: 7,
            rating_accent: 1,
            global_rank: Some(1234),
            banner: Some(横幅.to_string()),
            avatar: Some(SAMPLE_AVATAR.to_string()),
        }),
    };
    case_551(
        "05-rand.png",
        harness.render(select::request(&select_input), |kit| {
            select::paint(kit, &select_input)
        }),
    );

    let search_input = search::SearchInput {
        query: String::from("arc"),
        hits: (0..9)
            .map(|i| search::SearchHit {
                song_id: case_425(i),
                title: 曲名[i % 曲名.len()].to_string(),
                artist: String::from("Various"),
                jacket: Some(曲绘[i % 曲绘.len()].to_string()),
                charts: case_428(),
                group: None,
                active: Some(Difficulty::Future),
            })
            .collect(),
        truncated: 0,
        theme: Theme::DEFAULT,
        footer_note: String::new(),
        head: Some(String::from("arc")),
        note: Some(String::from("按相关度排序")),
    };
    case_551(
        "06-search.png",
        harness.render(search::request(&search_input), |kit| {
            search::paint(kit, &search_input)
        }),
    );

    let guess_input = guess::GuessInput {
        stage: guess::GuessStage::Question,
        round: 3,
        song_id: String::from("acheron"),
        title: String::from("Acheron"),
        artist: String::from("sasakure.UK"),
        jacket: Some(曲绘[0].to_string()),
        hints: Vec::new(),
        answer: String::from("Acheron"),
        correct: true,
        cost: 3,
        theme: Theme::DEFAULT,
        footer_note: String::new(),
        seed: Some(20251120),
        level: Some(0.5),
        zoom: Some(0.4),
        interference: Some(0.5),
        variant: Some(String::from("wash")),
        code: Some(String::from("1A2B3C")),
    };
    case_551(
        "07-guess.png",
        harness.render(guess::request(&guess_input), |kit| {
            guess::paint(kit, &guess_input)
        }),
    );

    let list_input = lists::ListInput {
        title: String::from("定数 10 曲目清单"),
        subtitle: String::from("全部成绩 · 单曲 PTT降序 · 前 14 条"),
        theme: Theme::DEFAULT,
        backdrop: String::new(),
        badge: badge.clone(),
        entries: (0..14)
            .map(|i| lists::ListEntry {
                song_id: case_425(i),
                title: 曲名[i % 曲名.len()].to_string(),
                artist: String::from("Various"),
                difficulty: Difficulty::Future,
                level: String::from("10"),
                constant: 10.1 + i as f64 * 0.05,
                has_constant: true,
                score: 9_800_000 + i as i64 * 7_000,
                has_score: true,
                has_judgement: true,
                perfect: 1_000 - i as i64,
                shiny: 900,
                far: i as i64,
                lost: 0,
                loss: Some(120.0 + i as f64),
                grade: Some(Grade::ExPlus),
                clear_type: Some(0),
                jacket: Some(曲绘[i % 曲绘.len()].to_string()),
            })
            .collect(),
        note: String::new(),
    };
    case_551(
        "08-list.png",
        harness.render(lists::request(&list_input), |kit| {
            lists::paint(kit, &list_input)
        }),
    );

    let table_input = lists::TableInput {
        title: String::from("定数表"),
        subtitle: String::new(),
        library: 1899,
        theme: Theme::DEFAULT,
        badge: badge.clone(),
        mode: lists::TableMode::Charts,
        songs: Vec::new(),
        sheets: [9.0, 9.1, 9.2]
            .into_iter()
            .map(|base| lists::TableSheet {
                band: lists::table_band_label(base),
                charts: (0..7)
                    .map(|i| lists::TableChart {
                        song_id: case_425((base * 10.0) as usize + i),
                        title: 曲名[((base * 10.0) as usize + i) % 曲名.len()].to_string(),
                        difficulty: Difficulty::Future,
                        constant: base,
                        has_constant: true,
                        score: 9_700_000 + i as i64 * 12_000,
                        has_score: !i.is_multiple_of(3),
                        jacket: Some(曲绘[((base * 10.0) as usize + i) % 曲绘.len()].to_string()),
                    })
                    .collect(),
            })
            .collect(),
        note: String::new(),
    };
    case_551(
        "09-table.png",
        harness.render(lists::table_request(&table_input), |kit| {
            lists::paint_table(kit, &table_input)
        }),
    );

    let ahv_input = lists::AhvInput {
        title: String::from("成就表"),
        subtitle: String::from("定数 12.0"),
        theme: Theme::DEFAULT,
        badge: badge.clone(),
        records: (0..18)
            .map(|i| lists::AhvRecord {
                song_id: case_425(i),
                title: 曲名[i % 曲名.len()].to_string(),
                difficulty: Difficulty::Future,
                constant: 10.2 + (i % 5) as f64 * 0.1,
                has_constant: true,
                score: 9_800_000 + i as i64 * 6_000,
                score_max: 10_000_000,
                far: (i as i64 % 4) * 3,
                lost: i as i64 % 2,
                has_judgement: !i.is_multiple_of(5),
                clear_type: Some(i as i32 % 3),
                grade: Some(aurorix_arcaea_plugin::domain::ptt::grade_for(
                    9_800_000 + i as i64 * 6_000,
                    (i as i64 % 4) * 3,
                    i as i64 % 2,
                    !i.is_multiple_of(5),
                )),
                jacket: Some(曲绘[i % 曲绘.len()].to_string()),
                pure: Some(1020),
                shiny: Some(880),
                rating: Some(12.4),
                loss: Some(0.4),
            })
            .collect(),
        note: String::new(),
        generated_at: Some(时间.to_string()),
    };
    case_551(
        "10-ahv.png",
        harness.render(lists::ahv_request(&ahv_input), |kit| {
            lists::paint_ahv(kit, &ahv_input)
        }),
    );

    let stats_input = stats::StatsInput {
        title: String::from("9 - 15 区间成绩"),
        note: String::from("命中 42 条 · 已收录 512 张谱面成绩"),
        backdrop: String::new(),
        badge: badge.clone(),
        bands: vec![
            stats::StatsBand {
                name: String::from("FTR"),
                total: 120,
                played: 96,
                cleared: 88,
                full_combo: 41,
                pm: 12,
                avg_score: 9_642_310.0,
            },
            stats::StatsBand {
                name: String::from("BYD"),
                total: 40,
                played: 18,
                cleared: 15,
                full_combo: 6,
                pm: 1,
                avg_score: 9_512_880.0,
            },
        ],
        consts: (0..12)
            .map(|i| stats::StatsConst {
                constant: 9.0 + i as f64 * 0.1,
                total: 24 + i * 2,
                played: 18 + i,
            })
            .collect(),
        const_note: String::from("已截断 12 档"),
        grades: [("EX+", 3), ("EX", 21), ("AA", 44), ("A", 31), ("B", 9)]
            .into_iter()
            .map(|(name, count)| stats::StatsGrade {
                name: String::from(name),
                count,
            })
            .collect(),
        short_title: String::from("短板"),
        short: (0..8usize)
            .map(|i| stats::StatsRow {
                song_id: case_425(i),
                title: 曲名[i % 曲名.len()].to_string(),
                diff: String::from("FTR"),
                constant: 10.4 + i as f64 * 0.1,
                known: true,
                score: 9_720_000 - i as i64 * 9_000,
                grade: String::from("EX"),
                next: String::from("EX+"),
                need: 12_000 + i as i64 * 1_000,
            })
            .collect(),
        short_note: String::new(),
    };
    case_551(
        "11-stats.png",
        harness.render(stats::stats_request(&stats_input), |kit| {
            stats::paint_stats(kit, &stats_input)
        }),
    );

    let suggest_input = stats::SuggestInput::new(
        badge.clone(),
        suggest(
            &(0..14).map(case_254).collect::<Vec<SuggestCandidate>>(),
            0.08,
            4,
        ),
    );
    case_551(
        "12-suggest.png",
        harness.render(stats::suggest_request(&suggest_input), |kit| {
            stats::paint_suggest(kit, &suggest_input)
        }),
    );

    let chap_input = stats::ChapInput {
        title: String::from("曲包进度"),
        subtitle: String::from("按曲包统计"),
        theme: Theme::DEFAULT,
        badge: badge.clone(),
        packs: (0..3)
            .map(|p| stats::ChapPack {
                pack_id: format!("pack-{p}"),
                name: format!("曲包 {p}"),
                song_count: 6,
                owned: 5,
                cleared: 4,
                full_combo: 3,
                pure_memory: 2,
                jackets: vec![曲绘[p].to_string()],
                charts: (0..6)
                    .map(|i| stats::ChapSong {
                        song_id: case_425(p + i),
                        title: 曲名[(p + i) % 曲名.len()].to_string(),
                        difficulty: Difficulty::Future,
                        level: String::from("10"),
                        constant: Some(10.3),
                        jacket: 曲绘[(p + i) % 曲绘.len()].to_string(),
                        score: 9_800_000 + i as i64 * 8_000,
                        grade: Some(Grade::ExPlus),
                        ptt: 11.8,
                        played: i != 5,
                        perfect: 1200,
                        shiny_perfect: 1100,
                        near: 12,
                        miss: 0,
                    })
                    .collect(),
                section: String::new(),
                avg_score: 0.0,
                avg_ptt: 0.0,
            })
            .collect(),
        expand: stats::ChapExpand::All,
        totals: None,
        focus: None,
        note: String::new(),
    };
    case_551(
        "13-chap.png",
        harness.render(stats::chap_request(&chap_input), |kit| {
            stats::paint_chap(kit, &chap_input)
        }),
    );

    let trend_input = stats::TrendInput {
        title: String::from("潜力值趋势"),
        subtitle: String::from("11-01 → 11-12 · 12 天 · 12 个记录点"),
        theme: Theme::DEFAULT,
        backdrop: String::from("lastcelebration"),
        badge: badge.clone(),
        points: (0..12)
            .map(|i| {
                let day = match i {
                    0..=2 => 1,
                    3..=5 => 2,
                    other => other - 2,
                };
                let hour = 9 + (i % 3) * 3;
                let minute = (i * 7) % 60;
                stats::TrendPoint {
                    label: format!("2025-11-{day:02} {hour:02}:{minute:02}"),
                    value: 12.0 + i as f64 * 0.04,
                    score: 9_800_000 + i as i64 * 5_000,
                    date: format!("2025-11-{day:02}"),
                    time: format!("{hour:02}:{minute:02}"),
                    minute: (day as i64 * 24 + hour as i64) * 60 + minute as i64,
                }
            })
            .collect(),
        note: String::new(),
        days: Some(12),
        rank: Some(1234),
        rank_total: Some(56789),
        peak: Some(12.44),
        recent7: Some(0.12),
        recent30: Some(0.28),
        records: Some(12),
        frame: Some(5),
        bounds: vec![3.49, 6.99, 9.99, 10.99, 11.99, 12.49, 12.99, 13.5],
    };
    case_551(
        "14-trend.png",
        harness.render(stats::trend_request(&trend_input), |kit| {
            stats::paint_trend(kit, &trend_input)
        }),
    );

    let his_input = stats::HisInput {
        title: String::from("历史记录"),
        subtitle: String::from("Acheron · FTR"),
        theme: Theme::DEFAULT,
        badge: badge.clone(),
        song_id: String::from("acheron"),
        song_title: String::from("Acheron"),
        jacket: Some(曲绘[0].to_string()),
        sections: vec![
            stats::HisSection::new(Difficulty::Future)
                .with_constant(10.4)
                .with_records(
                    (0..8)
                        .map(|i| stats::HisRecord {
                            normal_perfect: 0,
                            label: format!("11-{:02} 12:34", i + 1),
                            score: 9_800_000 + i as i64 * 11_000,
                            delta: if i == 0 { 0 } else { 11_000 },
                            ptt: 12.1 + i as f64 * 0.02,
                            perfect: 1020 + i as i64,
                            far: 20 - i as i64,
                            lost: i as i64,
                            grade: Some(Grade::ExPlus),
                            clear_type: Some(0),
                        })
                        .collect(),
                ),
        ],
        note: String::new(),
        omitted: String::new(),
    };
    case_551(
        "15-his.png",
        harness.render(stats::his_request(&his_input), |kit| {
            stats::paint_his(kit, &his_input)
        }),
    );

    let me_input = gallery::MeInput {
        title: String::from("我的档案"),
        subtitle: String::from("绑定与成绩概览"),
        theme: Theme::DEFAULT,
        badge: badge.clone(),
        user_id: String::from("10001"),
        region: Region::Cn,
        joined: String::from("2024-03-01"),
        updated: String::from(时间),
        best_average: 12.61,
        best: (0..6)
            .map(|i| gallery::MeScore {
                title: 曲名[i].to_string(),
                difficulty: Difficulty::Future,
                constant: 10.4,
                score: 9_900_000 + i as i64 * 4_000,
                ptt: 12.7 - i as f64 * 0.03,
                grade: Some(Grade::ExPlus),
                jacket: Some(曲绘[i].to_string()),
            })
            .collect(),
        recent: (0..6)
            .map(|i| gallery::MeScore {
                title: 曲名[(i + 2) % 曲名.len()].to_string(),
                difficulty: Difficulty::Beyond,
                constant: 11.2,
                score: 9_850_000 + i as i64 * 3_000,
                ptt: 12.2,
                grade: Some(Grade::Ex),
                jacket: Some(曲绘[(i + 2) % 曲绘.len()].to_string()),
            })
            .collect(),
        note: String::new(),
        curve: Some(gallery::MeCurve {
            points: (0..30)
                .map(|i| gallery::MeCurvePoint {
                    at: 1_760_000_000 + i as i64 * 86_400,
                    ptt: 11.4 + i as f64 * 0.03,
                })
                .collect(),
            peak: 12.61,
            peak_at: Some(1_762_592_000),
            recent7: Some(0.12),
            recent30: Some(0.34),
            records30: Some(128),
            first: Some(1_710_000_000),
            last: Some(1_762_592_000),
        }),
        pm_small_p: 3487,
        bands: Some(
            [
                (Difficulty::Ins, 6, 5, 4, 2, 1),
                (Difficulty::Beyond, 30, 22, 12, 4, 1),
                (Difficulty::Eternal, 8, 6, 4, 2, 0),
                (Difficulty::Future, 120, 110, 88, 40, 12),
                (Difficulty::Present, 60, 58, 50, 30, 10),
                (Difficulty::Past, 42, 42, 40, 30, 12),
            ]
            .into_iter()
            .map(|(kind, total, played, cleared, fc, pm)| gallery::MeBand {
                kind,
                name: kind.as_str().to_string(),
                played,
                total,
                cleared,
                full_combo: fc,
                pm,
                ex_plus: cleared.saturating_sub(4),
                ex: cleared.saturating_sub(2),
                loss_played: f64::from(played) * 137.421,
                loss_total: f64::from(total) * 291.337,
            })
            .collect(),
        ),
        scatter: Some(gallery::MeScatter {
            bins: (0..6)
                .map(|i| gallery::MeBin {
                    constant: 9.0 + i as f64 * 0.6,
                    low: 9.0 + i as f64 * 0.6,
                    high: 9.6 + i as f64 * 0.6,
                    avg: 9.3 + i as f64 * 0.6,
                    count: 12 - i,
                })
                .collect(),
            max_ptt: 12.5,
            avg_ptt: Some(11.2),
            ability: Some(11.6),
        }),
        board: Some(gallery::MeBoard {
            total: 342,
            rows: (0..10)
                .map(|i| gallery::MeBoardRow {
                    rank: i + 1,
                    name: format!("玩家 {:02}", i + 1),
                    avatar: Some(SAMPLE_AVATAR.to_string()),
                    ptt: Some(12.9 - i as f64 * 0.06),
                    rating_frame: Some(7),
                    rating_accent: Some(1),
                    me: i == 3,
                })
                .collect(),
        }),
        changes: Some(
            (0..5)
                .map(|i| gallery::MeChange {
                    title: 曲名[i].to_string(),
                    diff: String::from("FTR"),
                    old_score: Some(9_800_000),
                    new_score: 9_850_000 + i as i64 * 5_000,
                    at: Some(1_762_592_000 - i as i64 * 86_400),
                    jacket: Some(曲绘[i].to_string()),
                })
                .collect(),
        ),
        rating_frame: Some(7),
        rating_accent: Some(1),
    };
    case_551(
        "16-me.png",
        harness.render(gallery::request(&me_input), |kit| {
            gallery::paint(kit, &me_input)
        }),
    );

    let char_input = gallery::CharInput {
        title: String::from("搭档一览"),
        subtitle: String::from("共 24 位"),
        theme: Theme::DEFAULT,
        cells: (0..12usize)
            .map(|i| gallery::CharCell {
                name: format!("搭档 {i}"),
                label: String::from("平衡型"),
                thumb: String::from("char_thumbs/71u_s1.png"),
                art: String::from(立绘),
                unlocked: !i.is_multiple_of(4),
            })
            .collect(),
        group: String::from("全部"),
        note: String::new(),
    };
    case_551(
        "17-char.png",
        harness.render(gallery::char_request(&char_input), |kit| {
            gallery::paint_char(kit, &char_input)
        }),
    );

    let banner_input = gallery::BannerInput {
        title: String::from("背景一览"),
        subtitle: String::from("课程背景"),
        theme: Theme::DEFAULT,
        cells: (0..8usize)
            .map(|i| gallery::BannerCell {
                key: format!("banners/course_banner_{:02}.png", i % 4 + 1),
                label: format!("课程背景 {}", i + 1),
                unlocked: !i.is_multiple_of(3),
            })
            .collect(),
        group: String::from("全部"),
        note: String::new(),
    };
    case_551(
        "18-banner.png",
        harness.render(gallery::banner_request(&banner_input), |kit| {
            gallery::paint_banner(kit, &banner_input)
        }),
    );

    let theme_input = gallery::ThemeInput {
        title: String::from("主题"),
        subtitle: String::from("当前 1 / 2"),
        theme: Theme::DEFAULT,
        themes: vec![
            gallery::ThemeEntry::new(Theme::DEFAULT.name, "默认初始主题")
                .with_preview(Theme::DEFAULT.backdrop)
                .with_active(true),
            gallery::ThemeEntry::new(Theme::by_name("arknights").name, "Arknights")
                .with_preview(Theme::by_name("arknights").backdrop),
        ],
        note: String::new(),
    };
    case_551(
        "19-theme.png",
        harness.render(gallery::theme_request(&theme_input), |kit| {
            gallery::paint_theme(kit, &theme_input)
        }),
    );

    let diff_rows = [
        ("acheron", "Acheron", Difficulty::Future, 10.9, Some(1234), 10_012_345, Some(12.61), Some(0.12)),
        ("prism_prs", "PRAGMATISM -RESURRECTION-", Difficulty::Beyond, 10.6, None, 9_980_000, Some(11.20), None),
        ("conflict", "Conflict", Difficulty::Present, 8.3, Some(-210), 9_876_543, Some(9.87), Some(-0.04)),
        ("fractureray", "Fracture Ray", Difficulty::Future, 10.8, Some(3400), 10_023_456, Some(12.74), Some(0.31)),
        ("grievouslady", "Grievous Lady", Difficulty::Future, 11.0, Some(0), 9_999_999, Some(12.30), None),
        ("tempestissimo", "Tempestissimo", Difficulty::Ins, 10.7, Some(-88), 9_950_000, Some(11.80), Some(-0.01)),
        ("saikyostronger", "最強 STRONGER", Difficulty::Future, 10.5, Some(15000), 9_871_234, Some(10.44), Some(0.52)),
        ("worldvanquisher", "World Vanquisher", Difficulty::Eternal, 8.7, None, 10_000_000, Some(11.02), None),
    ];
    let mut diff_input = diff::DiffInput::new(badge.clone(), 7)
        .with_summary(8, 8, Some(0.18))
        .with_created(时间.to_string());
    for (index, (id, title, difficulty, constant, delta, score, ptt, ptt_delta)) in
        diff_rows.into_iter().enumerate()
    {
        diff_input = diff_input.with_entry(
            diff::DiffEntry::new(id, title, difficulty)
                .with_constant(Some(constant))
                .with_score(score)
                .with_delta(delta)
                .with_rating(ptt)
                .with_rating_delta(ptt_delta)
                .with_loss(Some(4.3))
                .with_judgement(1050, 998, 12, 3)
                .with_at(Some(format!("0{}-14 21:0{index}", index + 1))),
        );
    }
    case_551(
        "20-diff.png",
        harness.render_card(&diff::diff_card(diff_input)),
    );

    let files = fs::read_dir(&dir).expect("样张目录必须可读").count();
    println!("共 {files} 张样张，目录 {}", dir.display());
}
