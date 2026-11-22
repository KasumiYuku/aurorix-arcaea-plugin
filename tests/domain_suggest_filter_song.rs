use aurorix_arcaea_plugin::domain::filter::{ChartProbe, Filter, Level, MIXED_HINT};
use aurorix_arcaea_plugin::domain::song::{
    resolve, search, Chart, MatchKind, Song, PICK_ROWS,
};
use aurorix_arcaea_plugin::domain::suggest::{
    suggest, SuggestBoard, SuggestCandidate, DEFAULT_BONUS_NEED, PICK_TITLE,
};
use aurorix_arcaea_plugin::model::Difficulty;

fn parse(raw: &str) -> Filter {
    Filter::parse(raw)
        .unwrap_or_else(|error| panic!("{raw} 应当能解析，实际报错：{error}"))
        .unwrap_or_else(|| panic!("{raw} 应当解析出条件"))
}

#[test]
fn case_385() {
    assert_eq!(parse("9").levels, vec![Level::new(9, false)]);
    assert_eq!(parse("9+").levels, vec![Level::new(9, true)]);
    assert!(parse("10.0-10.9").constant.contains(10.5));
    assert!(parse(">=10.5").constant.contains(10.5));
    assert_eq!(parse("FTR").diffs, vec![Difficulty::Future]);
    assert_eq!(parse("永恒").diffs, vec![Difficulty::Eternal]);
    assert!(parse("bpm180-200").bpm.contains(190.0));
    assert!(parse("物量1000").note.contains(1000.0));
    assert_eq!(parse("title:Fracture Ray").text, "Fracture Ray");
    assert_eq!(parse("designer:nitro").designer, "nitro");
}

#[test]
fn case_386() {
    for raw in ["9 10.0-10.9", "10.0-10.9 9", "8+-9", "9+ 11.0"] {
        let error = Filter::parse(raw).expect_err(&format!("{raw} 必须被拒绝"));
        assert_eq!(error.to_string(), MIXED_HINT, "{raw} 的错误文案不对");
    }
}

#[test]
fn case_387() {
    let filter = parse("9+ FTR 谱师 nitro");
    assert_eq!(filter.describe(), "等级 9+ · FTR · 谱师 nitro");
    let dsl = filter.to_dsl();
    assert!(dsl.contains("rating_display=\"9+\""));
    assert!(dsl.contains("rating_class=2"));
    assert!(dsl.contains("difficulty_label=\"FTR\""));
    assert!(dsl.contains("designer~\"nitro\""));
    let ranged = parse("10.0-10.9");
    let ranged_dsl = ranged.to_dsl();
    assert!(ranged_dsl.contains("constant>=10"), "实际：{ranged_dsl}");
    assert!(ranged_dsl.contains("constant<=10.9"), "实际：{ranged_dsl}");
    assert!(ranged_dsl.contains(" and "));
}

#[test]
fn case_388() {
    let probe = ChartProbe::new("Fracture Ray", Difficulty::Future, 11.2)
        .with_level("10+")
        .with_bpm(160.0)
        .with_note(1300)
        .with_designer("Toaster");
    assert!(parse("FTR 10.0-11.5 bpm160 物量1000+ 谱师 toaster").matches(&probe));
    assert!(!parse("BYD").matches(&probe));
    assert!(!parse("12.0-12.9").matches(&probe));
    assert!(!parse("9+").matches(&probe), "等级逐字比，10+ 不等于 9+");
}

fn library() -> Vec<Song> {
    vec![
        Song::new("fractureray", "Fracture Ray")
            .with_artist("Sakuzyo")
            .with_alias("骨折光")
            .with_chart(Chart::new(Difficulty::Future, "10+", 11.2))
            .with_chart(Chart::new(Difficulty::Beyond, "11", 11.7)),
        Song::new("fractureray_at", "Fracture Ray (Arcaea TanoC)")
            .with_artist("Sakuzyo")
            .with_chart(Chart::new(Difficulty::Future, "9+", 9.8)),
        Song::new("lumia", "Lumia").with_artist("Sky_delta"),
    ]
}

#[test]
fn case_389() {
    let songs = library();
    let exact = resolve(&songs, "Fracture Ray").expect("精确命中");
    assert!(exact.exact);
    assert_eq!(exact.best.song.id, "fractureray");
    let alias = resolve(&songs, "骨折光").expect("别名命中");
    assert!(alias.exact);
    assert_eq!(alias.best.matched_by, MatchKind::Alias);
    let fuzzy = resolve(&songs, "frcray").expect("跳字也应当命中");
    assert!(!fuzzy.exact);
    assert_eq!(fuzzy.best.matched_by, MatchKind::Fuzzy);
    let normalized = resolve(&songs, "  fracture   ray ").expect("空格不该影响命中");
    assert!(normalized.exact);
}

#[test]
fn case_390() {
    let songs: Vec<Song> = (0..30)
        .map(|index| {
            Song::new(
                format!("conflict{index:02}"),
                format!("Conflict {index:02}"),
            )
        })
        .collect();
    let hit = resolve(&songs, "Conflict").expect("三十首都命中");
    assert_eq!(hit.total, 30);
    assert_eq!(
        hit.alternatives.len() + 1,
        PICK_ROWS,
        "候选列表最多 {PICK_ROWS} 行, 最优那条也在里面"
    );
    let line = hit.candidates_line();
    assert!(
        line.contains("后端共命中 30 首，其余相关度更低"),
        "实际：{line}"
    );
    assert!(line.ends_with("。都不是你要的那首，直接用曲目 ID 重发"));
    assert_eq!(search(&songs, "Conflict").len(), 30);
}

#[test]
fn case_391() {
    let songs = library();
    let mut reversed = songs.clone();
    reversed.reverse();
    let left: Vec<String> = search(&songs, "fractur")
        .into_iter()
        .map(|hit| hit.song.id)
        .collect();
    let right: Vec<String> = search(&reversed, "fractur")
        .into_iter()
        .map(|hit| hit.song.id)
        .collect();
    assert_eq!(left, right);
}

fn played(song: &str, constant: f64, note: i64, score: i64) -> SuggestCandidate {
    SuggestCandidate::new(
        song,
        format!("曲目 {song}"),
        Difficulty::Future,
        constant,
        note,
    )
    .with_score(score)
    .with_judgement(10, 0)
}

fn signature(board: &SuggestBoard) -> Vec<String> {
    let mut out: Vec<String> = board
        .groups
        .iter()
        .map(|group| format!("{}#{}", group.heading(), group.count()))
        .collect();
    for hint in board.all_hints() {
        out.push(format!(
            "{}|{}|{}|{}|{}|{}",
            hint.song_id,
            hint.difficulty.as_str(),
            hint.score,
            hint.target,
            hint.delta,
            hint.pick
        ));
    }
    out
}

fn fixture() -> Vec<SuggestCandidate> {
    let mut items = Vec::new();
    for index in 0..50 {
        let constant = 9.0 + (index % 22) as f64 * 0.1;
        items.push(played(
            &format!("song{index:02}"),
            constant,
            800 + index * 10,
            9_700_000 + index * 2000,
        ));
    }
    items.push(
        SuggestCandidate::new("fresh", "新曲", Difficulty::Present, 10.8, 1200)
            .with_judgement(0, 0),
    );
    items
}

#[test]
fn case_392() {
    let base = signature(&suggest(&fixture(), DEFAULT_BONUS_NEED, 40));
    let mut seed: u64 = 0x2545_F491_4F6C_DD1D;
    for round in 0..1000 {
        let mut items = fixture();
        let mut index = items.len();
        while index > 1 {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let swap_with = ((seed >> 33) as usize) % index;
            index -= 1;
            items.swap(index, swap_with);
        }
        let current = signature(&suggest(&items, DEFAULT_BONUS_NEED, 40));
        assert_eq!(
            current, base,
            "第 {round} 次打乱之后输出就变了，说明首选主推或分组排序不确定"
        );
    }
}

#[test]
fn case_393() {
    let items = vec![
        played("high", 11.4, 1200, 9_850_000),
        played("mid", 10.4, 1000, 9_850_000),
        SuggestCandidate::new("fresh_high", "新曲", Difficulty::Future, 10.8, 1200),
        SuggestCandidate::new("fresh_low", "新曲", Difficulty::Future, 9.4, 900),
    ];
    let board = suggest(&items, DEFAULT_BONUS_NEED, 60);
    let labels = board.labels();
    assert_eq!(labels[0], PICK_TITLE, "第一层是首选主推");
    assert_eq!(labels[1], "LEVEL 10");
    assert_eq!(labels[2], "LEVEL 9");
    assert_eq!(board.pick_count, 2, "只有打过的谱面能进主推");
    assert_eq!(board.hint_count, 4);
    let ids: Vec<&str> = board
        .all_hints()
        .iter()
        .map(|hint| hint.song_id.as_str())
        .collect();
    for id in ["high", "mid", "fresh_high", "fresh_low"] {
        assert_eq!(
            ids.iter().filter(|seen| **seen == id).count(),
            1,
            "{id} 只能出现一次"
        );
    }
    assert!(board
        .all_hints()
        .iter()
        .all(|hint| hint.target > hint.score));
}

#[test]
fn case_394() {
    let single = vec![played("one", 10.0, 800, 9_800_000)];
    let board = suggest(&single, DEFAULT_BONUS_NEED, 60);
    let hint = board.all_hints()[0];
    assert_eq!(hint.score, 9_800_000);
    assert_eq!(hint.target, 9_806_250, "800 物量一个 FAR 值 6250 分");
    assert_eq!(hint.delta, 6_250);
    assert!(hint.notes.known);
    assert_eq!(hint.notes.far_need, 1);
    assert_eq!(hint.notes.lost_need, 1);
    assert!(hint.pick);
}

#[test]
fn case_395() {
    let items = vec![
        played("known", 10.4, 1000, 9_850_000),
        SuggestCandidate::new("unknown", "未知定数", Difficulty::Future, 0.0, 1000)
            .with_score(9_850_000)
            .with_judgement(10, 0),
    ];
    let board = suggest(&items, DEFAULT_BONUS_NEED, 60);
    assert_eq!(board.pool, 2, "候选池仍然报全部条数");
    assert_eq!(board.hint_count, 1, "算不出目标分的不进建议");
    assert!(
        board.all_hints().iter().all(|hint| hint.song_id == "known"),
        "没有定数的谱面不该出现在建议里"
    );
}

#[test]
fn case_396() {
    let board = suggest(&[], DEFAULT_BONUS_NEED, 60);
    assert!(board.groups.is_empty());
    assert_eq!(board.hint_count, 0);
    assert_eq!(board.pick_count, 0);
    assert_eq!(board.pool, 0);
}
