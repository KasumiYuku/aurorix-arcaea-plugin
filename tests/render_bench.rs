use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use aurorix_arcaea_plugin::backend as dto;
use aurorix_arcaea_plugin::cards::theme::Theme;
use aurorix_arcaea_plugin::cards::{
    b50, harness, lists, score as score_cards, search as search_cards, song as song_cards, stats,
};
use aurorix_arcaea_plugin::commands::bridge;
use aurorix_arcaea_plugin::domain::suggest::{
    NotePlan, SuggestBoard, SuggestGroup, SuggestHint,
};
use aurorix_arcaea_plugin::model::{Prefs, Region};
use aurorix_plugin::RenderCard;
use aurorix_render::codec::{encode_rgba, EncodeOptions};
use aurorix_render::{CanvasKit, RenderFormat, RenderRequest};

const SAMPLES: usize = 5;

const DATA_ENV: &str = "ARC_BENCH_DATA";

const OUT_ENV: &str = "ARC_BENCH_OUT";

fn ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.partial_cmp(right).expect("耗时必须是数字"));
    sorted[sorted.len() / 2]
}

fn dir_of(key: &str) -> PathBuf {
    let raw = std::env::var(key).unwrap_or_else(|_| panic!("需要设置 {key}"));
    let path = PathBuf::from(raw);
    fs::create_dir_all(&path).expect("目录必须可创建");
    path
}

fn b50_of(dir: &Path) -> dto::B50 {
    let raw = fs::read_to_string(dir.join("b50.json")).expect("b50.json 必须存在");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("b50.json 可解析");
    let payload: dto::B50Payload =
        serde_json::from_value(value["data"].clone()).expect("b50 数据可解析");
    dto::B50::from_payload(payload, dto::QueryMeta::default())
}

fn hits_of(dir: &Path) -> Vec<dto::SongHit> {
    let raw = fs::read_to_string(dir.join("songs200.json")).expect("songs200.json 必须存在");
    let search: dto::SongSearch = serde_json::from_str(&raw).expect("曲库可解析");
    search.items
}

fn scores_of(dir: &Path) -> Vec<dto::ScoreRecord> {
    let raw = fs::read_to_string(dir.join("scores.json")).expect("scores.json 必须存在");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("scores.json 可解析");
    serde_json::from_value(value["data"].clone()).expect("成绩数据可解析")
}

fn level_of(record: &dto::ScoreRecord) -> String {
    if record.constant > 0.0 {
        format!("{:.1}", record.constant)
    } else {
        String::from("-")
    }
}

fn level_of_hit(hits: &[dto::SongHit], record: &dto::ScoreRecord) -> String {
    hits.iter()
        .find(|hit| hit.song.song_id == record.song_id)
        .and_then(|hit| {
            hit.song
                .charts
                .iter()
                .find(|chart| chart.rating_class == record.difficulty)
        })
        .map(|chart| chart.rating_display.clone())
        .unwrap_or_default()
}

fn suggest_board(records: &[dto::ScoreRecord]) -> SuggestBoard {
    let mut groups: Vec<SuggestGroup> = Vec::new();
    for (index, record) in records.iter().enumerate() {
        let label = format!("{:.1}", record.constant.floor());
        let difficulty = bridge::difficulty_of(record);
        let note = i64::from(record.object_amount).max(1);
        let hint = SuggestHint {
            song_id: record.song_id.clone(),
            title: record.title.clone(),
            difficulty,
            constant: record.constant,
            note,
            score: i64::from(record.score),
            has_score: record.score > 0,
            far: i64::from(record.far),
            lost: i64::from(record.lost),
            has_judgement: record.judged(),
            rating: record.rating,
            target_rating: record.rating + 0.02,
            delta: 4000,
            target: i64::from(record.score) + 4000,
            rank: index + 1,
            target_rank: index,
            pick: index % 7 == 0,
            notes: NotePlan {
                known: true,
                far_need: 12,
                lost_need: 0,
                far_room: 30,
                lost_room: 4,
                far_use: 22,
                lost_use: 1,
                short: index % 5 == 0,
            },
        };
        match groups.last_mut() {
            Some(group) if group.label == label => group.hints.push(hint),
            _ => groups.push(SuggestGroup {
                label,
                title: None,
                hints: vec![hint],
            }),
        }
    }
    let hint_count = groups.iter().map(SuggestGroup::count).sum();
    let pick_count = groups
        .iter()
        .flat_map(|group| group.hints.iter())
        .filter(|hint| hint.pick)
        .count();
    SuggestBoard {
        groups,
        hint_count,
        pick_count,
        pool: hint_count,
        pushable: pick_count,
    }
}

fn scenarios(dir: &Path) -> Vec<(String, RenderCard)> {
    let data = b50_of(dir);
    let hits = hits_of(dir);
    let records = scores_of(dir);
    let theme = Theme::DEFAULT;
    let prefs = Prefs::default();
    let badge = bridge::badge_of(&data.identity, &prefs);
    let note = String::from("与 10.4-10.9 对比 · 命中 200 条 · 画 200 行");
    let mut out: Vec<(String, RenderCard)> = Vec::new();
    out.push((
        String::from("b50-50"),
        b50::card(bridge::b50_input(Region::Cn, true, &data, &prefs, None)),
    ));
    let best = &records[0];
    let level = level_of_hit(&hits, best);
    out.push((
        String::from("score-1"),
        score_cards::score_card(bridge::score_input(
            best,
            &data.identity,
            &prefs,
            theme,
            level.as_str(),
        )),
    ));
    out.push((
        String::from("recent-1"),
        score_cards::recent_card(bridge::score_input(
            best,
            &data.identity,
            &prefs,
            theme,
            level.as_str(),
        )),
    ));
    out.push((
        String::from("song-1"),
        song_cards::card(bridge::song_input(&hits[0].song, theme)),
    ));
    out.push((
        String::from("search-200"),
        search_cards::card(bridge::search_input("200 条检索", &hits, theme, 200)),
    ));
    let entries: Vec<lists::ListEntry> = records
        .iter()
        .take(200)
        .map(|record| bridge::list_entry_of(record, level_of(record).as_str()))
        .collect();
    out.push((
        format!("list-{}", entries.len()),
        lists::card(bridge::list_input(
            badge.clone(),
            theme,
            "成绩清单",
            note.as_str(),
            note.as_str(),
            entries,
        )),
    ));
    let songs: Vec<dto::Song> = hits.iter().map(|hit| hit.song.clone()).collect();
    let table_songs = bridge::table_input(
        badge.clone(),
        theme,
        bridge::TableHead {
            title: "定数表",
            note: "只读曲库 · 最多画 200 首歌",
        },
        &songs,
        200,
        lists::TableMode::Songs,
        None,
    );
    out.push((
        format!("table-songs-{}", table_songs.songs.len()),
        lists::table_card(table_songs),
    ));
    let table_charts = bridge::table_input(
        badge.clone(),
        theme,
        bridge::TableHead {
            title: "定数表",
            note: "只读曲库 · 最多画 200 张谱面",
        },
        &songs,
        200,
        lists::TableMode::Charts,
        None,
    );
    out.push((
        format!("table-charts-{}", table_charts.sheets.len()),
        lists::table_card(table_charts),
    ));
    let ahv_records: Vec<lists::AhvRecord> = records
        .iter()
        .take(200)
        .map(bridge::ahv_record_of)
        .collect();
    out.push((
        format!("ahv-{}", ahv_records.len()),
        lists::ahv_card(bridge::ahv_input(
            badge.clone(),
            theme,
            "推分清单",
            note.as_str(),
            ahv_records,
            "2025-11-26 12:34",
        )),
    ));
    let board = suggest_board(&records[..200]);
    out.push((
        format!("suggest-{}", board.hint_count),
        stats::suggest_card(stats::SuggestInput::new(badge, board)),
    ));
    out
}

struct Row {
    label: String,
    width: i32,
    height: i32,
    format: String,
    bytes: usize,
    cold: f64,
    warm: f64,
    encode: f64,
    decodes: u64,
}

fn draw(
    harness: &harness::Harness,
    request: RenderRequest,
    card: &RenderCard,
) -> (Vec<u8>, f64) {
    let painter = Arc::clone(&card.painter);
    let started = Instant::now();
    let bytes = harness
        .context()
        .draw(request, |canvas, context| {
            let kit = CanvasKit::new(canvas, context);
            painter(&kit);
        })
        .expect("渲染失败");
    (bytes.to_vec(), ms(started))
}

fn pixels_of(harness: &harness::Harness, card: &RenderCard) -> harness::Pixels {
    let (png, _) = draw(
        harness,
        card.request
            .with_format(RenderFormat::Png)
            .with_quality(100),
        card,
    );
    harness::decode(&png).expect("PNG 必须可解码")
}

fn encode_once(pixels: &harness::Pixels, format: RenderFormat, options: &EncodeOptions) -> (usize, f64) {
    let started = Instant::now();
    let bytes = encode_rgba(
        format,
        &pixels.data,
        pixels.width as i32,
        pixels.height as i32,
        (pixels.width * 4) as usize,
        options,
    )
    .expect("编码必须成功");
    (bytes.len(), ms(started))
}

fn bench(label: &str, harness: &harness::Harness, card: &RenderCard) -> (Row, String) {
    let request = card.request;
    let before = harness.decodes();
    let (bytes, cold) = draw(harness, request, card);
    let mut warm: Vec<f64> = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let (_, took) = draw(harness, request, card);
        warm.push(took);
    }
    let decodes = harness.decodes() - before;
    let pixels = pixels_of(harness, card);
    let (prod_bytes, encode) = encode_once(
        &pixels,
        request.format,
        &EncodeOptions::new(request.quality),
    );
    let mut report: Vec<String> = Vec::new();
    for quality in [92, 90, 85, 80] {
        let (size, _) = encode_once(&pixels, RenderFormat::Jpeg, &EncodeOptions::new(quality));
        report.push(format!("q{quality} {}KB", size / 1024));
    }
    let (png_bytes, _) = encode_once(&pixels, RenderFormat::Png, &EncodeOptions::new(100));
    report.push(format!("png {}KB", png_bytes / 1024));
    report.push(format!(
        "产物 {}KB@q{}",
        prod_bytes / 1024,
        request.quality
    ));
    let (width, height) = request.physical_size();
    let row = Row {
        label: label.to_string(),
        width,
        height,
        format: format!("{} q{}", request.format.name(), request.quality),
        bytes: bytes.len(),
        cold,
        warm: median(&warm),
        encode,
        decodes,
    };
    (row, report.join(" | "))
}

#[test]
#[ignore]
fn render_bench() {
    let dir = dir_of(DATA_ENV);
    let out = dir_of(OUT_ENV);
    let harness = harness::Harness::new();
    let started = Instant::now();
    let report = aurorix_arcaea_plugin::warmup(harness.assets(), harness.context());
    println!(
        "预热 {:.0}ms 字体 {} 模板 {} 贴图 {} 缺失 {}",
        ms(started),
        report.fonts,
        report.templates,
        report.textures,
        report.missing
    );
    let cards = scenarios(&dir);
    let mut rows: Vec<Row> = Vec::new();
    println!(
        "{:<20} {:>11} {:>9} {:>7} {:>8} {:>8} {:>7} {:>7} {:>5}",
        "卡片", "分辨率", "格式", "体积", "冷", "暖", "编码", "绘制", "解码"
    );
    for (label, card) in &cards {
        let (row, report) = bench(label, &harness, card);
        let (bytes, _) = draw(&harness, card.request, card);
        fs::write(out.join(format!("{label}.jpg")), &bytes).expect("出图必须可写入");
        println!(
            "{:<20} {:>11} {:>9} {:>6}KB {:>6.0}ms {:>6.0}ms {:>5.0}ms {:>5.0}ms {:>5}",
            row.label,
            format!("{}x{}", row.width, row.height),
            row.format,
            row.bytes / 1024,
            row.cold,
            row.warm,
            row.encode,
            (row.warm - row.encode).max(0.0),
            row.decodes
        );
        println!("{:<20} {}", "", report);
        rows.push(row);
    }
    println!();
    println!(
        "{:<20} {:>10} {:>10} {:>9} {:>7} {:>10}",
        "汇总", "冷", "暖", "编码", "体积KB", "分辨率"
    );
    for row in &rows {
        println!(
            "{:<20} {:>9.0}ms {:>9.0}ms {:>8.0}ms {:>7} {:>10}",
            row.label,
            row.cold,
            row.warm,
            row.encode,
            row.bytes / 1024,
            format!("{}x{}", row.width, row.height)
        );
    }
    println!();
    println!(
        "{:<24} {:>11} {:>9} {:>9} {:>8}",
        "倍率对照", "分辨率", "暖中位", "体积KB", "编码ms"
    );
    for (label, card) in &cards {
        if card.request.scale <= 1.0 {
            continue;
        }
        let mut scaled = card.clone();
        scaled.request = card.request.with_scale(1.0);
        let (row, report) = bench(label, &harness, &scaled);
        let (bytes, _) = draw(&harness, scaled.request, &scaled);
        fs::write(out.join(format!("{label}-scale10.jpg")), &bytes).expect("出图必须可写入");
        println!(
            "{:<24} {:>11} {:>7.0}ms {:>9} {:>7.0}",
            format!("{}@1.0", row.label),
            format!("{}x{}", row.width, row.height),
            row.warm,
            row.bytes / 1024,
            row.encode
        );
        println!("{:<24} {}", "", report);
    }
}

#[test]
#[ignore]
fn build_bench() {
    let dir = dir_of(DATA_ENV);
    let started = Instant::now();
    let _ = aurorix_arcaea_plugin::cards::harness::Harness::new();
    let prepare = ms(started);
    println!("Harness::new {prepare:.1}ms");
    for round in 0..3 {
        let started = Instant::now();
        let cards = scenarios(&dir);
        let took = ms(started);
        let mut parts: Vec<String> = Vec::new();
        for (label, card) in &cards {
            let (width, height) = card.request.physical_size();
            parts.push(format!("{label} {width}x{height}"));
        }
        println!("构建#{round} {took:.1}ms 共 {} 张 ({})", cards.len(), parts.join(", "));
    }
}
