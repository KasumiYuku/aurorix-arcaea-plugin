use std::collections::{BTreeMap, HashMap};

use aurorix_plugin::{Outgoing, Part, PluginError, RenderCard, ReplyKind};

use crate::backend as dto;
use crate::backend::{ScoreSource, SongQuery};
use crate::cards::diff as diff_cards;
use crate::cards::lists as list_cards;
use crate::cards::stats as stats_cards;
use crate::cards::theme::Theme;
use crate::cards::widgets::PlayerBadge;
use crate::commands::bridge;
use crate::commands::{
    beijing, bound_account, hints, local_time, one, prefs_of, reply, root_of,
    split_difficulty, Ctx,
};
use crate::domain::{self, filter};
use crate::model::{diff_display_weight, Difficulty, Grade, Region};

const LIBRARY_LIMIT: u32 = 2000;
const LIST_DEFAULT: usize = 150;
const LIST_MAX: usize = 200;
const TABLE_DEFAULT: usize = 200;
const TABLE_MAX: usize = 200;
const AHV_DEFAULT: usize = 200;
const AHV_MAX: usize = 200;
const STATS_DEFAULT: usize = 6;
const STATS_MAX: usize = 20;
const SUGGEST_DEFAULT: usize = 200;
const SUGGEST_MAX: usize = 250;
const HIS_DEFAULT: usize = 200;
const HIS_MAX: usize = 200;
const TREND_DEFAULT_DAYS: u32 = 365;
const TREND_ALL_DAYS: u32 = 1825;
const TREND_MAX_DAYS: u32 = 1825;
const TREND_LIMIT: u32 = 20000;
const DIFF_DEFAULT_DAYS: u32 = 7;
const DIFF_MAX_DAYS: u32 = 365;
const DIFF_LIMIT: u32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Order {
    Score,
    Ptt,
    Constant,
    Date,
}

impl Order {
    fn label(self) -> &'static str {
        match self {
            Order::Score => "分数",
            Order::Ptt => "单曲 PTT",
            Order::Constant => "定数",
            Order::Date => "时间",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Band {
    Pm,
    Fr,
    Grade(Grade),
}

impl Band {
    fn label(self) -> String {
        match self {
            Band::Pm => String::from("PM"),
            Band::Fr => String::from("FR"),
            Band::Grade(grade) => grade.as_str().to_string(),
        }
    }
}

fn words_of(arg: &str) -> Vec<String> {
    arg.split_whitespace().map(str::to_string).collect()
}

fn count_of(token: &str) -> Option<usize> {
    let digits = token.strip_prefix(['n', 'N'])?;
    if digits.is_empty() || !digits.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }
    digits.parse::<usize>().ok().map(|value| value.max(1))
}

fn take_count(words: &mut Vec<String>, default: usize, max: usize) -> usize {
    if let Some(position) = words.iter().position(|word| count_of(word).is_some()) {
        let value = count_of(words[position].as_str()).unwrap_or(default);
        words.remove(position);
        return value.clamp(1, max);
    }
    default
}

fn order_of(token: &str) -> Option<(Order, bool)> {
    let (body, descending) = match token.strip_prefix('-') {
        Some(rest) => (rest, true),
        None => (token, false),
    };
    let order = match body.trim().to_ascii_lowercase().as_str() {
        "score" | "分" => Order::Score,
        "ptt" | "rks" => Order::Ptt,
        "const" | "定数" => Order::Constant,
        "date" | "时间" | "最近" => Order::Date,
        _ => return None,
    };
    Some((order, descending))
}

fn band_of(token: &str) -> Option<Band> {
    let lowered = token.trim().to_ascii_lowercase();
    match lowered.as_str() {
        "pm" => Some(Band::Pm),
        "fr" => Some(Band::Fr),
        "ex+" => Some(Band::Grade(Grade::ExPlus)),
        "ex" => Some(Band::Grade(Grade::Ex)),
        _ => Grade::parse(lowered.as_str()).map(Band::Grade),
    }
}

fn band_hit(band: Band, record: &dto::ScoreRecord) -> bool {
    match band {
        Band::Pm => record.judged() && record.far == 0 && record.lost == 0,
        Band::Fr => record.judged() && record.lost == 0,
        Band::Grade(grade) => grade_key(record) == grade,
    }
}

fn ptt_of(record: &dto::ScoreRecord) -> f64 {
    if record.rating > 0.0 {
        record.rating
    } else {
        domain::single_rating(record.constant, i64::from(record.score))
    }
}

fn table_mode_of(filter: Option<&domain::Filter>) -> list_cards::TableMode {
    match filter {
        Some(filter) if filter.has_constant() || filter.has_level() || !filter.diffs.is_empty() => {
            list_cards::TableMode::Charts
        }
        _ => list_cards::TableMode::Songs,
    }
}

fn compare_rows(
    order: Order,
    descending: bool,
    left: &dto::ScoreRecord,
    right: &dto::ScoreRecord,
) -> std::cmp::Ordering {
    let primary = match order {
        Order::Score => i64::from(left.score).cmp(&i64::from(right.score)),
        Order::Ptt => ptt_of(left).total_cmp(&ptt_of(right)),
        Order::Constant => left.constant.total_cmp(&right.constant),
        Order::Date => left.time_played.cmp(&right.time_played),
    };
    let primary = if descending { primary.reverse() } else { primary };
    diff_display_weight(bridge::difficulty_of(left))
        .cmp(&diff_display_weight(bridge::difficulty_of(right)))
        .then(primary)
        .then_with(|| tie_break(order, left, right))
        .then_with(|| left.title.to_lowercase().cmp(&right.title.to_lowercase()))
        .then_with(|| left.song_id.cmp(&right.song_id))
}

fn tie_break(order: Order, left: &dto::ScoreRecord, right: &dto::ScoreRecord) -> std::cmp::Ordering {
    let score = i64::from(right.score).cmp(&i64::from(left.score));
    let constant = right.constant.total_cmp(&left.constant);
    let ptt = ptt_of(right).total_cmp(&ptt_of(left));
    let clear = right
        .clear_type
        .unwrap_or(0)
        .cmp(&left.clear_type.unwrap_or(0));
    match order {
        Order::Ptt => score.then(constant).then(clear),
        Order::Score => constant.then(ptt).then(clear),
        Order::Constant => score.then(clear),
        Order::Date => score,
    }
}

fn day_key(at: chrono::DateTime<chrono::Utc>) -> String {
    at.with_timezone(&beijing())
        .format("%Y-%m-%d")
        .to_string()
}

fn day_label(at: chrono::DateTime<chrono::Utc>) -> String {
    at.with_timezone(&beijing()).format("%m-%d").to_string()
}

fn clock_of(at: chrono::DateTime<chrono::Utc>) -> (String, String, i64) {
    let local = at.with_timezone(&beijing());
    (
        local.format("%Y-%m-%d").to_string(),
        local.format("%H:%M").to_string(),
        local.timestamp() / 60,
    )
}

fn day_minute(day: &str) -> i64 {
    chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d")
        .ok()
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .map(|naive| naive.and_utc().timestamp() / 60)
        .unwrap_or(0)
}

fn grade_key(record: &dto::ScoreRecord) -> Grade {
    record
        .grade()
        .unwrap_or_else(|| domain::grade_of(i64::from(record.score)))
}

fn record_loss_text(record: &dto::ScoreRecord) -> String {
    match bridge::record_loss(record) {
        Some(loss) => format!("{loss:.1}"),
        None => String::from("—"),
    }
}

fn image_outgoing(id: &str, caption: String, fallback: String, card: RenderCard) -> Outgoing {
    let parts = match caption.trim().is_empty() {
        true => Vec::new(),
        false => vec![Part::Markdown(caption)],
    };
    let outgoing = Outgoing {
        parts,
        ..Outgoing::new(ReplyKind::Handler, String::new())
    };
    let outgoing = match fallback.trim().is_empty() {
        true => outgoing.card(card),
        false => outgoing.card_or(card, fallback),
    };
    hints::finish(id, outgoing)
}

fn caption_of(id: &str) -> String {
    reply::fill(id, &reply::args())
}

async fn identity_of(
    ctx: &Ctx<'_>,
    repo: &crate::repo::ArcaeaRepo,
    uid: Option<i64>,
    fallback: &str,
) -> (dto::PlayerIdentity, crate::model::Prefs, Theme) {
    let prefs = prefs_of(ctx, repo).await;
    let theme = bridge::theme_of(&prefs);
    let mut identity = dto::PlayerIdentity {
        name: fallback.to_string(),
        ..Default::default()
    };
    if let Some(uid) = uid {
        if let Ok(Some(player)) = ctx.backend.player(ctx.region, uid).await {
            identity = player.identity.named(fallback);
        }
    }
    (identity, prefs, theme)
}

async fn player_of(
    ctx: &Ctx<'_>,
    repo: &crate::repo::ArcaeaRepo,
    uid: Option<i64>,
    fallback: &str,
) -> (PlayerBadge, Theme) {
    let (identity, prefs, theme) = identity_of(ctx, repo, uid, fallback).await;
    (bridge::badge_of(&identity, &prefs), theme)
}

async fn bound_uid(
    ctx: &Ctx<'_>,
    repo: &crate::repo::ArcaeaRepo,
) -> Result<(i64, String), Box<Outgoing>> {
    let account = bound_account(ctx, repo).await?;
    match account.uid() {
        Some(uid) => Ok((uid, account.name.clone())),
        None => Err(Box::new(reply::unbound(ctx.region))),
    }
}

async fn library_of(
    ctx: &Ctx<'_>,
    filter: Option<&domain::Filter>,
) -> Result<Vec<dto::Song>, Box<Outgoing>> {
    let text = filter.map(|filter| filter.text.clone()).unwrap_or_default();
    let mut query = SongQuery::new(text).with_limit(LIBRARY_LIMIT);
    if let Some(filter) = filter {
        let dsl = filter.to_dsl();
        if !dsl.is_empty() {
            query = query.with_filter(dsl);
        }
    }
    match ctx.backend.songs(ctx.region, query).await {
        Ok(search) => Ok(search.items.into_iter().map(|hit| hit.song).collect()),
        Err(error) => Err(Box::new(reply::fail(error.user_message()))),
    }
}

fn list_usage(ctx: &Ctx<'_>) -> String {
    let root = root_of(ctx);
    format!(
        "用法：{root} list [条件…] [排序] [评级] [n条数]\n  n<数字>　取多少条，默认 {LIST_DEFAULT}，最多 {LIST_MAX}\n  排序键　score / ptt / const / date，前缀 - 表示降序，不写默认 -const\n  评级　　pm / fr / ex / ex+ / aa / a / b / c / d\n  条件　　与 {root} search 相同：定数区间、等级、难度、谱师、曲名"
    )
}

fn table_usage(ctx: &Ctx<'_>) -> String {
    let root = root_of(ctx);
    format!(
        "用法：{root} table [条件…] [n条数]\n  n<数字>　画多少行，默认 {TABLE_DEFAULT}，最多 {TABLE_MAX}\n  条件　　与 {root} search 相同：等级 9+、定数 10.0-10.9、难度 ftr…\n  筛定数/等级/难度时一行一张谱面；只筛曲名/曲包时一行一首曲，卡片下缘排出各档定数\n  只读曲库，不看成绩"
    )
}

fn ahv_usage(ctx: &Ctx<'_>) -> String {
    let root = root_of(ctx);
    format!(
        "用法：{root} ahv <定数或等级> [条件…] [n条数]\n  n<数字>　画多少张谱面，默认 {AHV_DEFAULT}，最多 {AHV_MAX}\n  定数或等级至少写一个，例如 {root} ahv 10.0-10.9 或 {root} ahv 11"
    )
}

fn stats_usage(ctx: &Ctx<'_>) -> String {
    let root = root_of(ctx);
    format!(
        "用法：{root} stats <定数区间> [d] [c] [noclear] [nofc] [nopm] [n条数]\n  d / diff / 难度　多画一列按难度拆分的完成度\n  c / const / 定数　多画一列按定数拆分的完成度\n  noclear / 未通关　只看还没通关的\n  nofc / 未fc　　　只看还没 FC 的\n  nopm / 未pm　　　只看还没 PM 的\n  n<数字>　　　　　待补清单条数，默认 {STATS_DEFAULT}，最多 {STATS_MAX}"
    )
}

fn suggest_usage(ctx: &Ctx<'_>) -> String {
    let root = root_of(ctx);
    format!("用法：{root} suggest [条件…] [n条数]\n  n<数字>　取多少条建议，默认 {SUGGEST_DEFAULT}，最多 {SUGGEST_MAX}")
}

fn chap_usage(ctx: &Ctx<'_>) -> String {
    let root = root_of(ctx);
    format!(
        "用法：{root} chap [曲包名] [难度…]\n  例如 {root} chap 主线 ftr、{root} chap 黑域 byd"
    )
}

fn trend_usage(ctx: &Ctx<'_>) -> String {
    let root = root_of(ctx);
    format!(
        "用法：{root} trend [天数|all]\n  天数　1 到 {TREND_MAX_DAYS}，默认 {TREND_DEFAULT_DAYS}\n  all / 全部 / alltime　一次画完 {TREND_ALL_DAYS} 天"
    )
}

fn his_usage(ctx: &Ctx<'_>) -> String {
    let root = root_of(ctx);
    format!(
        "用法：{root} his <曲目> [难度] [n条数]\n  难度　　不写就查这首曲的全部难度\n  n<数字>　每档看多少条，默认 {HIS_DEFAULT}，最多 {HIS_MAX}\n  例如 {root} his 骨折 ftr n20"
    )
}

struct ListRow<'a> {
    record: &'a dto::ScoreRecord,
    level: String,
}

pub async fn list(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    let usage = list_usage(ctx);
    let mut words = words_of(arg);
    let limit = take_count(&mut words, LIST_DEFAULT, LIST_MAX);
    let mut order = (Order::Constant, true);
    let mut band: Option<Band> = None;
    let mut rest: Vec<String> = Vec::new();
    for word in words {
        if let Some(parsed) = order_of(word.as_str()) {
            order = parsed;
            continue;
        }
        if band.is_none() {
            if let Some(parsed) = band_of(word.as_str()) {
                band = Some(parsed);
                continue;
            }
        }
        rest.push(word);
    }
    let filter = match domain::Filter::parse(rest.join(" ").as_str()) {
        Ok(filter) => filter,
        Err(_) => Some(domain::Filter::text_only(rest.join(" "))),
    };
    let Some(repo) = ctx.repo() else {
        return Ok(Some(reply::fail("插件数据库还没准备好")));
    };
    let (uid, account_name) = match bound_uid(ctx, &repo).await {
        Ok(found) => found,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    let scores = match ctx
        .backend
        .all_scores(ctx.region, uid, ScoreSource::Local)
        .await
    {
        Ok(scores) => scores,
        Err(error) => return Ok(Some(reply::fail(error.user_message()))),
    };
    let library = match library_of(ctx, None).await {
        Ok(library) => library,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    let mut index: HashMap<&str, usize> = HashMap::new();
    for (position, song) in library.iter().enumerate() {
        index.insert(song.song_id.as_str(), position);
    }
    let mut rows: Vec<ListRow<'_>> = Vec::new();
    for record in &scores.records {
        let song = index
            .get(record.song_id.as_str())
            .map(|position| &library[*position]);
        let probe = bridge::probe_of(song, record);
        if let Some(filter) = filter.as_ref() {
            if !filter.matches(&probe) {
                continue;
            }
        }
        if let Some(wanted) = band {
            if !band_hit(wanted, record) {
                continue;
            }
        }
        rows.push(ListRow {
            record,
            level: probe.level,
        });
    }
    rows.sort_by(|left, right| compare_rows(order.0, order.1, left.record, right.record));
    let matched = rows.len();
    rows.truncate(limit);
    if rows.is_empty() {
        return Ok(Some(reply::notice(format!(
            "没有符合条件的成绩。\n\n{usage}"
        ))));
    }
    let (badge, theme) = player_of(ctx, &repo, Some(uid), account_name.as_str()).await;
    let head = filter
        .as_ref()
        .map(domain::Filter::describe)
        .unwrap_or_else(|| String::from("全部成绩"));
    let direction = match order.1 {
        true => "降序",
        false => "升序",
    };
    let mut conditions: Vec<String> = Vec::new();
    conditions.push(format!("排序 {}{direction}", order.0.label()));
    if let Some(band) = band {
        conditions.push(format!("评级 {}", band.label()));
    }
    let note = format!(
        "{} · 命中 {matched} 条 · 画 {} 行",
        conditions.join(" · "),
        rows.len()
    );
    let subtitle = match head.as_str() {
        "全部成绩" => note.clone(),
        other => format!("{other} · {note}"),
    };
    let entries = rows
        .iter()
        .map(|row| bridge::list_entry_of(row.record, row.level.as_str()))
        .collect();
    let input = bridge::list_input(
        badge.clone(),
        theme,
        "成绩清单",
        subtitle.as_str(),
        note.as_str(),
        entries,
    );
    let mut text_args = reply::args();
    reply::put(&mut text_args, "name", badge.name.clone());
    reply::put(&mut text_args, "known", library.len().to_string());
    reply::put(&mut text_args, "matched", matched.to_string());
    reply::put(&mut text_args, "head", head.clone());
    reply::put(&mut text_args, "note", note.clone());
    for (position, row) in rows.iter().enumerate() {
        let record = row.record;
        let score = i64::from(record.score);
        reply::row(
            &mut text_args,
            "rows",
            &[
                ("index", (position + 1).to_string()),
                ("title", record.title.clone()),
                ("difficulty", bridge::difficulty_of(record).as_str().to_string()),
                ("constant", one(record.constant)),
                ("score", score.to_string()),
                ("loss", record_loss_text(record)),
                ("grade", grade_key(record).as_str().to_string()),
            ],
        );
    }
    let text = reply::fill("query/query.list", &text_args);
    let id = "query/query.list.caption";
    let caption = caption_of(id);
    Ok(Some(image_outgoing(id, caption, text, list_cards::card(input))))
}

pub async fn table(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    let usage = table_usage(ctx);
    let mut words = words_of(arg);
    let limit = take_count(&mut words, TABLE_DEFAULT, TABLE_MAX);
    let filter = match domain::Filter::parse(words.join(" ").as_str()) {
        Ok(filter) => filter,
        Err(_) => Some(domain::Filter::text_only(words.join(" "))),
    };
    let Some(repo) = ctx.repo() else {
        return Ok(Some(reply::fail("插件数据库还没准备好")));
    };
    let songs = match library_of(ctx, filter.as_ref()).await {
        Ok(songs) => songs,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    let account = repo
        .get_account(ctx.region, &ctx.inv.user)
        .await
        .ok()
        .flatten();
    let uid = account.as_ref().and_then(|account| account.uid());
    let fallback = account
        .as_ref()
        .map(|account| account.name.clone())
        .unwrap_or_else(|| String::from("Arcaea"));
    let (badge, theme) = player_of(ctx, &repo, uid, fallback.as_str()).await;
    let head = filter
        .as_ref()
        .map(domain::Filter::describe)
        .unwrap_or_else(|| String::from("全部谱面"));
    let mode = table_mode_of(filter.as_ref());
    let note = match mode {
        list_cards::TableMode::Songs => format!("只读曲库 · 最多画 {limit} 首歌"),
        list_cards::TableMode::Charts => format!("只读曲库 · 最多画 {limit} 张谱面"),
    };
    let input = bridge::table_input(
        badge.clone(),
        theme,
        bridge::TableHead {
            title: "定数表",
            note: note.as_str(),
        },
        &songs,
        limit,
        mode,
        filter.as_ref(),
    );
    if input.sheets.is_empty() && input.songs.is_empty() {
        return Ok(Some(reply::notice(format!(
            "没有符合条件的谱面（{head}）\n\n{usage}"
        ))));
    }
    let mut text_args = reply::args();
    reply::put(&mut text_args, "head", head.clone());
    reply::put(&mut text_args, "note", note.clone());
    reply::put(
        &mut text_args,
        "caption",
        String::from(match mode {
            list_cards::TableMode::Songs => "按最高定数从高到低",
            list_cards::TableMode::Charts => "按等级从高到低",
        }),
    );
    if mode == list_cards::TableMode::Songs {
        for song in &input.songs {
            let cells: Vec<String> = song
                .known_cells()
                .iter()
                .map(|cell| format!("{} {:.1}", cell.difficulty.as_str(), cell.constant))
                .collect();
            reply::row(
                &mut text_args,
                "rows",
                &[("title", song.title.clone()), ("cells", cells.join("；"))],
            );
        }
    } else {
        for sheet in &input.sheets {
            let cells: Vec<String> = sheet
                .charts
                .iter()
                .map(|chart| {
                    format!(
                        "{} {} {:.1}",
                        chart.difficulty.as_str(),
                        chart.title,
                        chart.constant
                    )
                })
                .collect();
            reply::row(
                &mut text_args,
                "rows",
                &[
                    ("title", sheet.band.clone()),
                    ("cells", cells.join("；")),
                ],
            );
        }
    }
    let text = reply::fill("query/query.table", &text_args);
    let id = "query/query.table.caption";
    let caption = caption_of(id);
    Ok(Some(image_outgoing(id, caption, text, list_cards::table_card(input))))
}

pub async fn ahv(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    let usage = ahv_usage(ctx);
    let mut words = words_of(arg);
    let limit = take_count(&mut words, AHV_DEFAULT, AHV_MAX);
    let filter = match domain::Filter::parse(words.join(" ").as_str()) {
        Ok(Some(filter)) => filter,
        Ok(None) | Err(_) => domain::Filter::text_only(words.join(" ")),
    };
    if !filter.has_constant() && !filter.has_level() {
        return Ok(Some(reply::notice(format!(
            "定数或等级至少要写一个。\n\n{usage}"
        ))));
    }
    let Some(repo) = ctx.repo() else {
        return Ok(Some(reply::fail("插件数据库还没准备好")));
    };
    let (uid, account_name) = match bound_uid(ctx, &repo).await {
        Ok(found) => found,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    let scores = match ctx
        .backend
        .all_scores(ctx.region, uid, ScoreSource::Local)
        .await
    {
        Ok(scores) => scores,
        Err(error) => return Ok(Some(reply::fail(error.user_message()))),
    };
    let library = match library_of(ctx, None).await {
        Ok(library) => library,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    let mut by_key: HashMap<(String, i32), &dto::ScoreRecord> = HashMap::new();
    for record in &scores.records {
        by_key.insert((record.song_id.clone(), record.difficulty), record);
    }
    let mut hits: Vec<AhvHit<'_>> = Vec::new();
    for song in &library {
        for chart in &song.charts {
            let Some(difficulty) = chart.difficulty_kind() else {
                continue;
            };
            if !filter.matches(&bridge::probe_of_chart(song, chart, difficulty)) {
                continue;
            }
            let record = by_key
                .get(&(song.song_id.clone(), i32::from(difficulty.as_upstream())))
                .copied();
            hits.push(AhvHit {
                song,
                chart,
                difficulty,
                record,
            });
        }
    }
    hits.sort_by(ahv_hit_order);
    let matched = hits.len();
    let played = hits.iter().filter(|hit| hit.played()).count();
    let shown = ahv_cut(&hits, limit);
    if shown == 0 {
        return Ok(Some(reply::notice(format!(
            "没有符合条件的谱面。\n\n{usage}"
        ))));
    }
    let (badge, theme) = player_of(ctx, &repo, Some(uid), account_name.as_str()).await;
    let head = filter.describe();
    let stamp = local_time(chrono::Utc::now());
    let note = match matched - shown {
        0 => format!("命中 {matched} 张 · 有记录 {played} · 按定数 0.1 分档"),
        cut => format!("命中 {matched} 张 · 有记录 {played} · 画 {shown} 张（截 {cut} 张）"),
    };
    let records = hits[..shown]
        .iter()
        .map(|hit| match hit.record {
            Some(record) => bridge::ahv_record_of(record),
            None => bridge::ahv_record_of_chart(hit.song, hit.chart, hit.difficulty),
        })
        .collect();
    let input = bridge::ahv_input(
        badge,
        theme,
        "成就表",
        note.as_str(),
        records,
        stamp.as_str(),
    );
    let mut text_args = reply::args();
    reply::put(&mut text_args, "head", head.clone());
    reply::put(&mut text_args, "note", note.clone());
    for (position, hit) in hits[..shown].iter().enumerate() {
        let line = match hit.record {
            Some(record) => format!(
                "{}. {} {} {:.1} · {} · {} · P{} F{} L{} · LS {}",
                position + 1,
                record.title,
                hit.difficulty.as_str(),
                hit.constant(),
                record.score,
                grade_key(record).as_str(),
                record.perfect,
                record.far,
                record.lost,
                match bridge::record_loss(record) {
                    Some(loss) => format!("{loss:.1}"),
                    None => String::from("—"),
                }
            ),
            None => format!(
                "{}. {} {} {:.1} · 未游玩",
                position + 1,
                hit.song.title,
                hit.difficulty.as_str(),
                hit.constant()
            ),
        };
        reply::row(&mut text_args, "lines", &[("text", line)]);
    }
    let text = reply::fill("query/query.ahv", &text_args);
    let id = "query/query.ahv.caption";
    let caption = caption_of(id);
    Ok(Some(image_outgoing(id, caption, text, list_cards::ahv_card(input))))
}

struct AhvHit<'a> {
    song: &'a dto::Song,
    chart: &'a dto::Chart,
    difficulty: Difficulty,
    record: Option<&'a dto::ScoreRecord>,
}

impl AhvHit<'_> {
    fn played(&self) -> bool {
        self.record.is_some_and(|record| record.score > 0)
    }

    fn constant(&self) -> f64 {
        self.chart.constant.unwrap_or(0.0)
    }
}

fn ahv_hit_order(left: &AhvHit<'_>, right: &AhvHit<'_>) -> std::cmp::Ordering {
    list_cards::table_band_key(right.constant())
        .cmp(&list_cards::table_band_key(left.constant()))
        .then_with(|| right.played().cmp(&left.played()))
        .then_with(|| match (left.record, right.record) {
            (Some(a), Some(b)) => b.score.cmp(&a.score),
            _ => std::cmp::Ordering::Equal,
        })
        .then_with(|| diff_display_weight(right.difficulty).cmp(&diff_display_weight(left.difficulty)))
        .then_with(|| {
            left.song
                .title
                .to_lowercase()
                .cmp(&right.song.title.to_lowercase())
        })
        .then_with(|| left.song.song_id.cmp(&right.song.song_id))
}

fn ahv_cut(hits: &[AhvHit<'_>], limit: usize) -> usize {
    if hits.len() <= limit {
        return hits.len();
    }
    let mut shown = 0;
    let mut index = 0;
    while index < hits.len() {
        let band = list_cards::table_band_key(hits[index].constant());
        let mut end = index;
        while end < hits.len() && list_cards::table_band_key(hits[end].constant()) == band {
            end += 1;
        }
        if end > limit {
            return shown;
        }
        shown = end;
        index = end;
    }
    shown
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatsDetail {
    Difficulty,
    Constant,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatsView {
    All,
    Uncleared,
    Unfc,
    Unpm,
}

impl StatsView {
    fn as_str(self) -> &'static str {
        match self {
            StatsView::All => "all",
            StatsView::Uncleared => "uncleared",
            StatsView::Unfc => "unfc",
            StatsView::Unpm => "unpm",
        }
    }
}

fn stats_sections(
    detail: StatsDetail,
    data: &dto::PlayerStats,
) -> (&[dto::StatsBand], &[dto::StatsConst]) {
    let bands: &[dto::StatsBand] = if detail == StatsDetail::Constant {
        &[]
    } else {
        data.bands.as_slice()
    };
    let consts: &[dto::StatsConst] = if detail == StatsDetail::Difficulty {
        &[]
    } else {
        data.consts.as_slice()
    };
    (bands, consts)
}

pub(crate) fn no_scores_notice(region: Region) -> String {
    if region.is_cn() {
        String::from(
            "Tips: 国服不会自动上传成绩，每次查分前请先在游戏内「添加好友」页点击「云端同步」，再发送 `/ca update`，以免出现曲目缺失或成绩未更新的情况",
        )
    } else {
        String::from(
            "还没有可用的成绩数据：先发一次 /a b50，或稍后再试 —— 后端正在按定数从高到低补扫",
        )
    }
}

fn stats_rate(part: i64, whole: i64) -> String {
    if whole <= 0 {
        return String::from("—");
    }
    format!("{}%", (part as f64 * 100.0 / whole as f64).round() as i64)
}

fn stats_avg(played: i64, avg: f64) -> String {
    if played <= 0 {
        return String::from("—");
    }
    format!("{:.0}", (avg + 0.5).floor())
}

fn stats_rating(played: i64, avg: f64) -> String {
    if played <= 0 {
        return String::from("—");
    }
    domain::format_rating(avg)
}

fn stats_short_text(view: StatsView, count: usize) -> (String, String) {
    match view {
        StatsView::Uncleared => (
            format!("待通关 · 最接近通关线的 {count} 首"),
            String::from("这一段全部通关了"),
        ),
        StatsView::Unfc => (
            format!("待 FC · 分数最高的 {count} 首"),
            String::from("这一段全部 FC 了"),
        ),
        StatsView::Unpm => (
            format!("待 PM · 分数最高的 {count} 首"),
            String::from("这一段全部满分了"),
        ),
        StatsView::All => (
            format!("短板 · 分数最低的 {count} 首"),
            String::from("这一段还没有成绩"),
        ),
    }
}

pub async fn stats(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    let usage = stats_usage(ctx);
    let mut words = words_of(arg);
    let limit = take_count(&mut words, STATS_DEFAULT, STATS_MAX);
    let mut detail = StatsDetail::Both;
    let mut view = StatsView::All;
    let mut rest: Vec<String> = Vec::new();
    for word in words {
        match word.to_ascii_lowercase().as_str() {
            "d" | "diff" | "难度" => {
                detail = StatsDetail::Difficulty;
                continue;
            }
            "c" | "const" | "定数" => {
                detail = StatsDetail::Constant;
                continue;
            }
            "noclear" | "未通关" => {
                view = StatsView::Uncleared;
                continue;
            }
            "nofc" | "未fc" => {
                view = StatsView::Unfc;
                continue;
            }
            "nopm" | "未pm" => {
                view = StatsView::Unpm;
                continue;
            }
            _ => {}
        }
        rest.push(word);
    }
    let filter = match domain::Filter::parse(rest.join(" ").as_str()) {
        Ok(Some(filter)) => filter,
        Ok(None) | Err(_) => domain::Filter::text_only(rest.join(" ")),
    };
    if !filter.has_constant() {
        return Ok(Some(reply::notice(format!(
            "统计必须先写一个定数区间，例如 10.0-10.9。\n\n{usage}"
        ))));
    }
    if !filter.levels.is_empty() {
        return Ok(Some(reply::notice(format!(
            "统计只按定数区间取数，等级请换成定数写法。\n\n{usage}"
        ))));
    }
    if !filter.text.is_empty() {
        return Ok(Some(reply::notice(format!(
            "统计只按定数区间取数，不要带曲名。\n\n{usage}"
        ))));
    }
    let Some(repo) = ctx.repo() else {
        return Ok(Some(reply::fail("插件数据库还没准备好")));
    };
    let (uid, account_name) = match bound_uid(ctx, &repo).await {
        Ok(found) => found,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    let span = filter.constant;
    let from = if span.min_set {
        filter::number(span.min)
    } else {
        String::from("0")
    };
    let to = if span.max_set {
        filter::number(span.max)
    } else {
        String::new()
    };
    let mut query = dto::StatsQuery::new(from, view.as_str()).with_short(limit as u32);
    if !to.is_empty() {
        query = query.with_to(to);
    }
    if !filter.diffs.is_empty() {
        query = query.with_diff(
            filter
                .diffs
                .iter()
                .map(|difficulty| difficulty.as_upstream().to_string())
                .collect(),
        );
    }
    let data = match ctx.backend.stats(ctx.region, uid, query).await {
        Ok(data) => data,
        Err(error) => return Ok(Some(reply::fail(error.user_message()))),
    };
    let head = span.count_text();
    if data.known <= 0 {
        return Ok(Some(reply::notice(no_scores_notice(ctx.region))));
    }
    if data.total <= 0 {
        return Ok(Some(reply::notice(format!("曲库里这一段没有谱面（{head}）"))));
    }
    let short = data.short.as_deref().unwrap_or(&[]);
    let (bands, consts) = stats_sections(detail, &data);
    let (badge, _) = player_of(ctx, &repo, Some(uid), account_name.as_str()).await;
    let title = format!("区间统计 · {head}");
    let note = format!("定数 {head} · 共 {} 张 · 已玩 {} 张", data.total, data.played);
    let (short_title, short_note) = stats_short_text(view, limit);
    let input = bridge::stats_input(
        badge,
        bridge::StatsArgs {
            title: title.as_str(),
            note: note.as_str(),
            data: &data,
            bands,
            consts,
            short,
            short_title: short_title.as_str(),
            short_note: short_note.as_str(),
        },
    );
    let mut text_args = reply::args();
    reply::put(&mut text_args, "head", head.clone());
    reply::put(
        &mut text_args,
        "stats",
        format!(
            "谱面 {} · 已玩 {} · 覆盖率 {} · 通关率 {} · FC 率 {} · PM 率 {} · 均分 {} · 均单曲 PTT {}",
            data.total,
            data.played,
            stats_rate(data.played, data.total),
            stats_rate(data.cleared, data.played),
            stats_rate(data.full_combo, data.played),
            stats_rate(data.pm, data.played),
            stats_avg(data.played, data.avg_score),
            stats_rating(data.played, data.avg_rating),
        ),
    );
    reply::put(&mut text_args, "note", note.clone());
    reply::put(&mut text_args, "shortTitle", short_title.clone());
    for band in &data.bands {
        let name = band
            .difficulty_kind()
            .map(|difficulty| difficulty.as_str().to_string())
            .unwrap_or_else(|| band.difficulty.to_string());
        reply::row(
            &mut text_args,
            "bands",
            &[
                ("name", name),
                ("played", band.played.to_string()),
                ("total", band.total.to_string()),
                ("cleared", band.cleared.to_string()),
                ("fc", band.full_combo.to_string()),
                ("pm", band.pm.to_string()),
                ("avg", format!("{:.0}", band.avg_score)),
            ],
        );
    }
    for row in short {
        let difficulty = Difficulty::from_upstream(i64::from(row.difficulty))
            .map(|difficulty| difficulty.as_str().to_string())
            .unwrap_or_else(|| row.difficulty.to_string());
        reply::row(
            &mut text_args,
            "short",
            &[
                ("title", row.title.clone()),
                ("difficulty", difficulty),
                ("constant", one(row.constant.unwrap_or(0.0))),
                ("score", row.score.to_string()),
                ("grade", row.grade.clone()),
                ("need", format!("{} {}", row.next_rank, row.next_need)),
            ],
        );
    }
    let text = reply::fill("query/query.stats", &text_args);
    let id = "query/query.stats.caption";
    let caption = caption_of(id);
    Ok(Some(image_outgoing(id, caption, text, stats_cards::stats_card(input))))
}

pub async fn suggest(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    let usage = suggest_usage(ctx);
    let mut words = words_of(arg);
    let limit = take_count(&mut words, SUGGEST_DEFAULT, SUGGEST_MAX);
    let filter = match domain::Filter::parse(words.join(" ").as_str()) {
        Ok(filter) => filter,
        Err(error) => return Ok(Some(reply::notice(format!("{error}\n\n{usage}")))),
    };
    let Some(repo) = ctx.repo() else {
        return Ok(Some(reply::fail("插件数据库还没准备好")));
    };
    let (uid, account_name) = match bound_uid(ctx, &repo).await {
        Ok(found) => found,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    let mut query = dto::SuggestQuery::new().with_limit(limit as u32);
    if let Some(filter) = filter.as_ref() {
        let dsl = filter.to_dsl();
        if !dsl.is_empty() {
            query = query.with_filter(dsl);
        }
        if !filter.text.is_empty() {
            query = query.with_text(filter.text.clone());
        }
    }
    let data = match ctx.backend.suggest(ctx.region, uid, query).await {
        Ok(data) => data,
        Err(error) => return Ok(Some(reply::fail(error.user_message()))),
    };
    if data.known <= 0 {
        let notice = if ctx.region.is_cn() {
            format!("{}，然后回来看能推哪首", no_scores_notice(ctx.region))
        } else {
            String::from("还没有成绩记录 —— 先用 /a b50 同步一次，再来看能推哪首")
        };
        return Ok(Some(reply::notice(notice)));
    }
    if !data.enough {
        return Ok(Some(reply::notice(format!(
            "PTT 要 {} 条成绩才算得出来，现在只有 {} 条",
            domain::ptt::B50_WINDOW, data.known
        ))));
    }
    if data.hints.is_empty() {
        let condition = filter
            .as_ref()
            .map(domain::Filter::describe)
            .unwrap_or_default();
        return Ok(Some(reply::notice(if condition.is_empty() {
            String::from("没有能推的谱面了 —— 进 b50 的都已经 PM，或者再推也抬不动 PTT")
        } else {
            format!("没有符合 {condition} 的谱面能推 —— 条件放宽些，或者不写条件看全部")
        })));
    }
    let (badge, theme) = player_of(ctx, &repo, Some(uid), account_name.as_str()).await;
    let head = format!(
        "已记录 {} 条 · PTT {:.3} → {:.3}",
        data.known, data.displayed, data.target
    );
    let note = if data.enough {
        format!("命中 {} 条建议", data.hints.len())
    } else {
        format!("只凑到 {} 条建议", data.hints.len())
    };
    let board = bridge::suggest_board_of(&data);
    let input = bridge::suggest_input(badge, theme, "推分建议", note.as_str(), &data);
    let mut text_args = reply::args();
    reply::put(&mut text_args, "head", head.clone());
    reply::put(&mut text_args, "note", note.clone());
    reply::note(
        &mut text_args,
        format!("主推 {} 条 · 共 {} 条建议", board.pick_count, board.hint_count),
    );
    for (position, hint) in board.all_hints().iter().enumerate() {
        let pick = if hint.pick { "★" } else { "" };
        let difficulty = hint.difficulty.as_str().to_string();
        reply::row(
            &mut text_args,
            "hints",
            &[
                ("seq", (position + 1).to_string()),
                ("title", hint.title.clone()),
                ("diff", difficulty),
                ("constant", one(hint.constant)),
                ("scoreText", hint.score.to_string()),
                ("deltaText", format!("+{}", hint.delta)),
                ("rank", hint.rank.to_string()),
                ("pick", pick.to_string()),
            ],
        );
    }
    let text = reply::fill("query/query.suggest", &text_args);
    let id = "query/query.suggest.caption";
    let caption = caption_of(id);
    Ok(Some(image_outgoing(id, caption, text, stats_cards::suggest_card(input))))
}

pub async fn chap(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    let usage = chap_usage(ctx);
    let mut difficulties: Vec<Difficulty> = Vec::new();
    let mut names: Vec<String> = Vec::new();
    for word in words_of(arg) {
        match Difficulty::parse(word.as_str()) {
            Some(difficulty) => {
                if !difficulties.contains(&difficulty) {
                    difficulties.push(difficulty);
                }
            }
            None => names.push(word),
        }
    }
    difficulties.sort();
    let query_text = names.join(" ");
    let label = difficulties
        .iter()
        .map(|difficulty| difficulty.as_str())
        .collect::<Vec<_>>()
        .join("/");
    let Some(repo) = ctx.repo() else {
        return Ok(Some(reply::fail("插件数据库还没准备好")));
    };
    let (uid, account_name) = match bound_uid(ctx, &repo).await {
        Ok(found) => found,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    let query = dto::ChapterQuery::new()
        .with_text(query_text.clone())
        .with_rating_class(
            difficulties
                .iter()
                .map(|difficulty| difficulty.as_upstream().to_string())
                .collect(),
        );
    let data = match ctx.backend.chapters(ctx.region, uid, query).await {
        Ok(data) => data,
        Err(error) => return Ok(Some(reply::fail(error.user_message()))),
    };
    if data.matched.len() > 1 {
        let names: Vec<String> = data
            .matched
            .iter()
            .map(|hit| hit.name.clone())
            .collect();
        return Ok(Some(reply::notice(format!(
            "命中多个曲包：{}\n说得再具体一点再发一次",
            names.join("、")
        ))));
    }
    if data.detail.is_none() && data.matched.is_empty() && !query_text.is_empty() {
        return Ok(Some(reply::notice(format!(
            "没找到曲包「{query_text}」\n{usage}"
        ))));
    }
    if let Some(detail) = data.detail.as_ref() {
        if !difficulties.is_empty() && detail.charts.is_empty() {
            return Ok(Some(reply::notice(format!(
                "「{}」没有 {label} 谱面",
                detail.pack.name
            ))));
        }
    }
    let (badge, theme) = player_of(ctx, &repo, Some(uid), account_name.as_str()).await;
    let title = match data.detail.as_ref() {
        Some(detail) if !detail.pack.name.trim().is_empty() => detail.pack.name.trim().to_string(),
        Some(_) => String::from("曲包完成度"),
        None => String::new(),
    };
    let note = if query_text.is_empty() {
        format!(
            "共 {} 个曲包 · {} 张谱面",
            data.totals.packs, data.totals.total
        )
    } else {
        format!(
            "曲包「{query_text}」· {} 张谱面",
            data.totals.total.max(1)
        )
    };
    let note = match label.is_empty() {
        true => note,
        false => format!("仅 {label} · {note}"),
    };
    let packs = if data.detail.is_some() {
        data.totals.packs.max(1)
    } else {
        data.totals.packs
    };
    let subtitle = match label.is_empty() {
        true => format!("命中 {packs} 个曲包"),
        false => format!("命中 {packs} 个曲包 · 仅 {label}"),
    };
    let input = bridge::chap_input(
        badge,
        theme,
        title.as_str(),
        subtitle.as_str(),
        note.as_str(),
        &data,
    );
    if input.packs.is_empty() {
        return Ok(Some(reply::notice(format!(
            "没有符合条件的曲包（{query_text}）\n{usage}"
        ))));
    }
    let head = match title.trim().is_empty() {
        true => String::from("曲包完成度"),
        false => title.clone(),
    };
    let mut text_args = reply::args();
    reply::put(&mut text_args, "title", head);
    reply::put(&mut text_args, "note", note.clone());
    for pack in &input.packs {
        reply::row(
            &mut text_args,
            "rows",
            &[(
                "text",
                format!(
                    "{} · {}/{} 已打 · 通关 {} · FC {} · PM {}",
                    pack.name, pack.owned, pack.song_count, pack.cleared, pack.full_combo, pack.pure_memory
                ),
            )],
        );
    }
    let text = reply::fill("query/query.chapter", &text_args);
    let id = "query/query.chapter.caption";
    let caption = caption_of(id);
    Ok(Some(image_outgoing(id, caption, text, stats_cards::chap_card(input))))
}

fn signed_ptt(value: f64) -> String {
    if value > 0.0 {
        format!("+{value:.2}")
    } else if value < 0.0 {
        format!("−{:.2}", -value)
    } else {
        format!("{value:.2}")
    }
}

fn trend_change(events: &[dto::RatingEvent], days: i64) -> Option<f64> {
    let last = events.last()?;
    if events.len() < 2 {
        return None;
    }
    let base = last.at - chrono::Duration::days(days);
    let point = events.iter().rev().find(|event| event.at <= base)?;
    let slack = (days / 2).max(7);
    if last.at - point.at > chrono::Duration::days(days + slack) {
        return None;
    }
    Some(last.rating - point.rating)
}

pub async fn trend(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    let usage = trend_usage(ctx);
    let text = arg.trim();
    let days = if text.is_empty() {
        TREND_DEFAULT_DAYS
    } else {
        let lowered = text.to_ascii_lowercase();
        if lowered == "all" || lowered == "全部" || lowered == "alltime" {
            TREND_ALL_DAYS
        } else {
            match text.parse::<u32>() {
                Ok(value) if (1..=TREND_MAX_DAYS).contains(&value) => value,
                _ => {
                    return Ok(Some(reply::notice(format!(
                        "天数只能写 1 到 {TREND_MAX_DAYS}，或者 all。\n\n{usage}"
                    ))))
                }
            }
        }
    };
    let Some(repo) = ctx.repo() else {
        return Ok(Some(reply::fail("插件数据库还没准备好")));
    };
    let (uid, account_name) = match bound_uid(ctx, &repo).await {
        Ok(found) => found,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    let events = match ctx
        .backend
        .rating_events(ctx.region, uid, Some(days), Some(TREND_LIMIT))
        .await
    {
        Ok(events) => events,
        Err(error) => return Ok(Some(reply::fail(error.user_message()))),
    };
    let profile = match ctx
        .backend
        .profile(ctx.region, uid, dto::ProfileQuery::trend(days))
        .await
    {
        Ok(profile) => profile,
        Err(error) => return Ok(Some(reply::fail(error.user_message()))),
    };
    let mut counts: BTreeMap<String, i64> = BTreeMap::new();
    for point in &profile.curve {
        counts.insert(point.day.clone(), point.records);
    }
    let mut points: Vec<stats_cards::TrendPoint> = Vec::new();
    for event in &events {
        let count = counts.get(&day_key(event.at)).copied().unwrap_or(0);
        let clock = clock_of(event.at);
        points.push(
            stats_cards::TrendPoint::new(day_label(event.at), event.rating)
                .with_score(count)
                .with_clock(clock.0, clock.1, clock.2),
        );
    }
    if points.is_empty() {
        for point in &profile.curve {
            if let Some(rating) = point.rating.filter(|value| *value > 0.0) {
                points.push(
                    stats_cards::TrendPoint::new(point.day.clone(), rating)
                        .with_score(point.records)
                        .with_clock(point.day.clone(), String::new(), day_minute(point.day.as_str())),
                );
            }
        }
    }
    if points.is_empty() {
        return Ok(Some(reply::notice(
            "还没有 PTT 历史 —— 日结每天记一次，明天再来看看",
        )));
    }
    let name = if profile.name().is_empty() {
        account_name
    } else {
        profile.name().to_string()
    };
    let (mut badge, theme) = player_of(ctx, &repo, Some(uid), name.as_str()).await;
    if let Some(player) = profile.player.as_ref() {
        if !player.name.is_empty() {
            badge.name = player.name.clone();
        }
        badge.ptt = player.rating.filter(|value| *value > 0.0);
    }
    let now = points.last().map(|point| point.value).unwrap_or(0.0);
    let peak = points
        .iter()
        .map(|point| point.value)
        .fold(f64::MIN, f64::max);
    let recent7 = trend_change(&events, 7);
    let recent30 = trend_change(&events, 30);
    let rank = profile
        .rank
        .as_ref()
        .filter(|rank| rank.listed())
        .map(|rank| rank.rank.to_string())
        .unwrap_or_else(|| String::from("—"));
    let head = format!("{} 天 · {} 个记录点", days, points.len());
    let mut parts: Vec<String> = vec![format!("峰值 {peak:.2}")];
    if let Some(value) = recent7 {
        parts.push(format!("近 7 天 {}", signed_ptt(value)));
    }
    if let Some(value) = recent30 {
        parts.push(format!("近 30 天 {}", signed_ptt(value)));
    }
    let note = parts.join(" · ");
    let first_day = points.first().map(|point| point.label.clone()).unwrap_or_default();
    let last_day = points.last().map(|point| point.label.clone()).unwrap_or_default();
    let head_line = format!("{first_day} → {last_day}");
    let mut input = bridge::trend_input(badge, theme, "潜力值趋势", head_line.as_str(), points.clone())
        .with_note(note.as_str())
        .with_days(days)
        .with_records(points.len())
        .with_curve_stats(Some(peak), recent7, recent30)
        .with_bounds(&domain::rating_frame_bounds());
    if let Some(found) = profile.rank.as_ref().filter(|rank| rank.listed()) {
        input = input.with_rank(found.rank, found.total);
    }
    if let Some(frame) = profile.player.as_ref().and_then(|player| player.rating_frame) {
        input = input.with_frame(frame);
    }
    let mut text_args = reply::args();
    reply::put(&mut text_args, "name", name.clone());
    reply::put(&mut text_args, "head", head.clone());
    reply::put(&mut text_args, "note", note.clone());
    reply::put(&mut text_args, "now", format!("{now:.2}"));
    reply::put(&mut text_args, "peak", format!("{peak:.2}"));
    reply::put(
        &mut text_args,
        "recent7",
        recent7.map_or_else(|| String::from("—"), signed_ptt),
    );
    reply::put(
        &mut text_args,
        "recent30",
        recent30.map_or_else(|| String::from("—"), signed_ptt),
    );
    reply::put(&mut text_args, "rank", rank.clone());
    for point in &points {
        reply::row(
            &mut text_args,
            "points",
            &[
                ("day", point.label.clone()),
                ("ptt", format!("{:.2}", point.value)),
            ],
        );
    }
    let text = reply::fill("query/query.trend", &text_args);
    let id = "query/query.trend.caption";
    let caption = caption_of(id);
    Ok(Some(image_outgoing(id, caption, text, stats_cards::trend_card(input))))
}

pub async fn his(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    let usage = his_usage(ctx);
    let mut words = words_of(arg);
    let limit = take_count(&mut words, HIS_DEFAULT, HIS_MAX);
    let rest = words.join(" ");
    if rest.trim().is_empty() {
        return Ok(Some(reply::notice(usage)));
    }
    let (query, difficulty) = split_difficulty(rest.as_str());
    let query = query.as_str();
    let Some(repo) = ctx.repo() else {
        return Ok(Some(reply::fail("插件数据库还没准备好")));
    };
    let (uid, account_name) = match bound_uid(ctx, &repo).await {
        Ok(found) => found,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    let search = match ctx
        .backend
        .songs(
            ctx.region,
            SongQuery::new(query).with_limit(crate::backend::DEFAULT_SEARCH_LIMIT),
        )
        .await
    {
        Ok(search) => search,
        Err(error) => return Ok(Some(reply::fail(error.user_message()))),
    };
    let library: Vec<domain::Song> = search
        .items
        .iter()
        .map(|hit| bridge::song_of(&hit.song))
        .collect();
    let Some(found) = domain::resolve(&library, query) else {
        return Ok(Some(reply::notice(format!(
            "没有找到匹配「{query}」的曲目，可以试试曲目 ID、别名或歌名"
        ))));
    };
    let song = &found.best.song;
    let wanted: Vec<Difficulty> = match difficulty {
        Some(one) => match song.charts.iter().any(|chart| chart.difficulty == one) {
            true => vec![one],
            false => {
                return Ok(Some(reply::notice(format!(
                    "《{}》没有 {} 难度",
                    song.title,
                    one.as_str()
                ))))
            }
        },
        None => song.charts.iter().map(|chart| chart.difficulty).collect(),
    };
    let asked = futures_util::future::join_all(wanted.iter().map(|difficulty| {
        ctx.backend.history(
            ctx.region,
            uid,
            song.id.as_str(),
            *difficulty,
            Some(limit as u32),
        )
    }))
    .await;
    let mut sections: Vec<(Difficulty, Vec<dto::SongHistoryRow>)> = Vec::new();
    for (difficulty, rows) in wanted.into_iter().zip(asked) {
        let rows = match rows {
            Ok(rows) => rows,
            Err(error) => return Ok(Some(reply::fail(error.user_message()))),
        };
        if rows.is_empty() {
            continue;
        }
        sections.push((difficulty, rows));
    }
    if sections.is_empty() {
        return Ok(Some(reply::notice(format!(
            "《{}》还没有游玩历史记录",
            song.title
        ))));
    }
    let (badge, theme) = player_of(ctx, &repo, Some(uid), account_name.as_str()).await;
    let input = bridge::his_input(badge, theme, song, &sections);
    let mut note = format!(
        "{} · {} 档 · {} 条记录",
        song.title,
        input.sections.len(),
        input.record_count()
    );
    if !input.omitted.is_empty() {
        note.push_str(" · ");
        note.push_str(&input.omitted);
    }
    let input = input.with_note(note.as_str());
    let mut text_args = reply::args();
    reply::put(&mut text_args, "title", song.title.clone());
    reply::put(&mut text_args, "note", note.clone());
    for (difficulty, rows) in &sections {
        let constant = song
            .charts
            .iter()
            .find(|chart| chart.difficulty == *difficulty)
            .map(|chart| chart.constant)
            .unwrap_or(0.0);
        let head = match constant > 0.0 {
            true => format!("{} {:.1}", difficulty.as_str(), constant),
            false => difficulty.as_str().to_string(),
        };
        let body: Vec<String> = rows
            .iter()
            .map(|row| {
                let delta = if row.has_old {
                    i64::from(row.new_score) - i64::from(row.old_score)
                } else {
                    0
                };
                let at = row
                    .at
                    .map(local_time)
                    .unwrap_or_else(|| String::from("时间未知"));
                format!(
                    "{at} · {} · 变化 {:+} · P{} F{} L{}",
                    row.new_score, delta, row.new_perfect, row.new_near, row.new_miss
                )
            })
            .collect();
        reply::row(
            &mut text_args,
            "sections",
            &[("head", head), ("body", body.join("\n"))],
        );
    }
    let text = reply::fill("query/query.his", &text_args);
    let id = "query/query.his.caption";
    let caption = caption_of(id);
    Ok(Some(image_outgoing(id, caption, text, stats_cards::his_card(input))))
}

fn diff_usage(ctx: &Ctx<'_>) -> String {
    let root = root_of(ctx);
    format!(
        "用法：{root} diff [天数]\n  天数　1 到 {DIFF_MAX_DAYS}，写成 7 / 7d / 7天 都行，不写就是最近 {DIFF_DEFAULT_DAYS} 天\n  例如 {root} diff 30 / {root} diff 30d"
    )
}

fn diff_days(arg: &str) -> Result<u32, String> {
    let text = arg.trim();
    if text.is_empty() {
        return Ok(DIFF_DEFAULT_DAYS);
    }
    let body = text
        .strip_suffix('天')
        .or_else(|| text.strip_suffix('日'))
        .or_else(|| text.strip_suffix('d'))
        .or_else(|| text.strip_suffix('D'))
        .unwrap_or(text)
        .trim();
    let days = body
        .parse::<u32>()
        .map_err(|_| String::from("天数只能写数字，比如 7 / 7d / 7天"))?;
    if days == 0 || days > DIFF_MAX_DAYS {
        return Err(format!("天数只能写 1 到 {DIFF_MAX_DAYS}"));
    }
    Ok(days)
}

fn diff_clock(at: chrono::DateTime<chrono::Utc>) -> String {
    at.with_timezone(&beijing())
        .format("%m/%d %H:%M")
        .to_string()
}

fn diff_summary(rows: &[dto::SongHistoryRow]) -> (usize, usize, Option<f64>) {
    let mut songs: Vec<&str> = rows.iter().map(|row| row.song_id.as_str()).collect();
    songs.sort_unstable();
    songs.dedup();
    let latest = rows.iter().find_map(|row| row.account_rating);
    let earliest = rows.iter().rev().find_map(|row| row.account_rating);
    (
        songs.len(),
        rows.len(),
        latest.zip(earliest).map(|(new, old)| new - old),
    )
}

fn diff_subtitle(days: u32, rows: &[dto::SongHistoryRow]) -> String {
    let (songs, changes, delta) = diff_summary(rows);
    let mut parts: Vec<String> = vec![
        format!("近 {days} 天"),
        format!("{songs} 首"),
        format!("{changes} 条变化"),
    ];
    if let Some(delta) = delta {
        parts.push(format!("PTT {}", signed_ptt(delta)));
    }
    parts.join(" · ")
}

fn diff_rating(row: &dto::SongHistoryRow) -> Option<f64> {
    row.new_rating.or_else(|| {
        row.constant
            .filter(|value| *value > 0.0)
            .map(|constant| crate::domain::ptt::single_rating(constant, i64::from(row.new_score)))
    })
}

fn diff_entry_of(row: &dto::SongHistoryRow) -> Option<diff_cards::DiffEntry> {
    let difficulty = row.difficulty_kind()?;
    let delta = row
        .has_old
        .then(|| i64::from(row.new_score) - i64::from(row.old_score));
    let new_rating = diff_rating(row);
    let old_rating = row.has_old.then(|| {
        row.old_rating.or_else(|| {
            row.constant.filter(|value| *value > 0.0).map(|constant| {
                crate::domain::ptt::single_rating(constant, i64::from(row.old_score))
            })
        })
    }).flatten();
    Some(
        diff_cards::DiffEntry::new(row.song_id.clone(), row.title.clone(), difficulty)
            .with_constant(row.constant)
            .with_score(i64::from(row.new_score))
            .with_delta(delta)
            .with_rating(new_rating)
            .with_rating_delta(old_rating.zip(new_rating).map(|(old, new)| new - old))
            .with_loss(row.lose_score)
            .with_judgement(
                i64::from(row.new_perfect),
                i64::from(row.new_shiny_perfect),
                i64::from(row.new_near),
                i64::from(row.new_miss),
            )
            .with_at(row.at.map(diff_clock)),
    )
}

pub async fn diff(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    let usage = diff_usage(ctx);
    let days = match diff_days(arg) {
        Ok(days) => days,
        Err(reason) => return Ok(Some(reply::notice(format!("{reason}\n\n{usage}")))),
    };
    let Some(repo) = ctx.repo() else {
        return Ok(Some(reply::fail("插件数据库还没准备好")));
    };
    let (uid, account_name) = match bound_uid(ctx, &repo).await {
        Ok(found) => found,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    let rows = match ctx
        .backend
        .diff(ctx.region, uid, days, Some(DIFF_LIMIT))
        .await
    {
        Ok(rows) => rows,
        Err(error) => return Ok(Some(reply::fail(error.user_message()))),
    };
    if rows.is_empty() {
        return Ok(Some(reply::notice(format!(
            "最近 {days} 天没有成绩变化"
        ))));
    }
    let mut latest: std::collections::HashMap<(&str, i32), &dto::SongHistoryRow> =
        std::collections::HashMap::new();
    for row in &rows {
        let key = (row.song_id.as_str(), row.difficulty);
        let newer = latest.get(&key).is_none_or(|seen| row.at > seen.at);
        if newer {
            latest.insert(key, row);
        }
    }
    let mut picked: Vec<&dto::SongHistoryRow> = latest.into_values().collect();
    picked.sort_by_key(|row| std::cmp::Reverse(row.at));
    let merged: Vec<dto::SongHistoryRow> = picked.iter().map(|row| (*row).clone()).collect();
    let subtitle = diff_subtitle(days, &merged);
    let (songs, changes, ptt_delta) = diff_summary(&merged);
    let entries: Vec<diff_cards::DiffEntry> = picked.iter().filter_map(|row| diff_entry_of(row)).collect();
    let (mut identity, prefs, _theme) =
        identity_of(ctx, &repo, Some(uid), account_name.as_str()).await;
    if let Some(rating) = rows.iter().find_map(|row| row.account_rating) {
        identity.current_ptt = Some(rating).filter(|value| *value > 0.0);
        identity.rating_frame = None;
        identity.rating_accent = None;
    }
    let badge = bridge::badge_of(&identity, &prefs);
    let input = diff_cards::DiffInput::new(badge, days)
        .with_summary(songs, changes, ptt_delta)
        .with_created(local_time(chrono::Utc::now()))
        .with_entries(entries);
    let mut text_args = reply::args();
    reply::put(&mut text_args, "days", days.to_string());
    reply::put(&mut text_args, "head", subtitle.clone());
    for row in &merged {
        reply::row(
            &mut text_args,
            "rows",
            &[
                (
                    "clock",
                    row.at.map(local_time).unwrap_or_else(|| String::from("时间未知")),
                ),
                ("title", row.title.clone()),
                (
                    "difficulty",
                    row.difficulty_kind()
                        .map(|kind| kind.as_str().to_string())
                        .unwrap_or_default(),
                ),
                (
                    "constant",
                    row.constant
                        .map_or_else(String::new, |value| format!("{value:.1}")),
                ),
                ("score", row.new_score.to_string()),
                (
                    "delta",
                    match row.has_old {
                        true => format!("{:+}", i64::from(row.new_score) - i64::from(row.old_score)),
                        false => String::from("首次收录"),
                    },
                ),
                ("perfect", row.new_perfect.to_string()),
                ("far", row.new_near.to_string()),
                ("lost", row.new_miss.to_string()),
                (
                    "rating",
                    diff_rating(row)
                        .map_or_else(|| String::from("—"), |value| format!("{value:.2}")),
                ),
                (
                    "loss",
                    row.lose_score
                        .map_or_else(|| String::from("—"), |value| format!("{value:.1}")),
                ),
            ],
        );
    }
    let text = reply::fill("query/query.diff", &text_args);
    let id = "query/query.diff.caption";
    let caption = caption_of(id);
    Ok(Some(image_outgoing(
        id,
        caption,
        text,
        diff_cards::diff_card(input),
    )))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::backend::{ArcBackendNext, BackendConfig};
    use crate::commands::{plugin, Services};

    fn tree() -> aurorix_plugin::PluginCommands {
        let backend = ArcBackendNext::new(BackendConfig::new("http://127.0.0.1:9"))
            .expect("后端配置要能构建");
        plugin(Services::new(Arc::new(backend)))
    }

    fn names_of(root: &aurorix_plugin::CommandSpec) -> Vec<&str> {
        root.subcommands
            .iter()
            .map(|spec| spec.name.as_str())
            .collect()
    }

    fn spec_of<'a>(
        root: &'a aurorix_plugin::CommandSpec,
        name: &str,
    ) -> &'a aurorix_plugin::CommandSpec {
        root.subcommands
            .iter()
            .find(|spec| spec.name == name)
            .unwrap_or_else(|| panic!("{name} 必须挂在根上"))
    }

    #[test]
    fn case_052() {
        let commands = tree();
        for wanted in [
            "list", "table", "ahv", "stats", "suggest", "chap", "trend", "his", "diff",
        ] {
            for root in &commands.commands {
                assert!(
                    names_of(root).contains(&wanted),
                    "{} 上必须有 {wanted}",
                    root.name
                );
            }
        }
        assert_eq!(commands.commands[0].subcommands.len(), 37);
        assert_eq!(commands.commands[1].subcommands.len(), 39);
    }

    #[test]
    fn case_053() {
        let commands = tree();
        let root = &commands.commands[0];
        let expected: [(&str, [&str; 5]); 9] = [
            ("list", ["清单", "筛选", "", "", ""]),
            ("table", ["定数表", "const", "rating", "", ""]),
            ("ahv", ["成就", "achievement", "comp", "", ""]),
            ("stats", ["统计", "区间", "", "", ""]),
            ("suggest", ["push", "up", "推分", "推分建议", ""]),
            ("chap", ["章节", "曲包", "pack", "", ""]),
            ("trend", ["趋势", "ptt", "", "", ""]),
            ("his", ["历史", "record", "", "", ""]),
            ("diff", ["变化", "变动", "", "", ""]),
        ];
        for (name, aliases) in expected {
            let spec = spec_of(root, name);
            for alias in aliases {
                if alias.is_empty() {
                    continue;
                }
                assert!(
                    spec.aliases.iter().any(|value| value == alias),
                    "{name} 缺别名 {alias}"
                );
            }
        }
    }

    #[test]
    fn case_054() {
        let mut words = vec![String::from("ftr"), String::from("n50")];
        assert_eq!(take_count(&mut words, LIST_DEFAULT, LIST_MAX), 50);
        assert_eq!(words, vec![String::from("ftr")]);
        let mut zero = vec![String::from("n0")];
        assert_eq!(take_count(&mut zero, LIST_DEFAULT, LIST_MAX), 1);
        assert!(zero.is_empty());
        let mut huge = vec![String::from("N9999")];
        assert_eq!(take_count(&mut huge, LIST_DEFAULT, LIST_MAX), LIST_MAX);
        let mut none = vec![String::from("noclear"), String::from("note")];
        assert_eq!(take_count(&mut none, LIST_DEFAULT, LIST_MAX), LIST_DEFAULT);
        assert_eq!(none.len(), 2, "不是 n<数字> 的词不能被吃掉");
    }

    #[test]
    fn case_055() {
        assert_eq!(order_of("score"), Some((Order::Score, false)));
        assert_eq!(order_of("分"), Some((Order::Score, false)));
        assert_eq!(order_of("-ptt"), Some((Order::Ptt, true)));
        assert_eq!(order_of("rks"), Some((Order::Ptt, false)));
        assert_eq!(order_of("const"), Some((Order::Constant, false)));
        assert_eq!(order_of("定数"), Some((Order::Constant, false)));
        assert_eq!(order_of("最近"), Some((Order::Date, false)));
        assert_eq!(order_of("delete"), None);
    }

    #[test]
    fn case_056() {
        assert_eq!(band_of("pm"), Some(Band::Pm));
        assert_eq!(band_of("FR"), Some(Band::Fr));
        assert_eq!(band_of("ex+"), Some(Band::Grade(Grade::ExPlus)));
        assert_eq!(band_of("EX"), Some(Band::Grade(Grade::Ex)));
        assert_eq!(band_of("aa"), Some(Band::Grade(Grade::Aa)));
        assert_eq!(band_of("d"), Some(Band::Grade(Grade::D)));
        assert_eq!(band_of("s"), None);
    }

    #[test]
    fn case_057() {
        let mut record = dto::ScoreRecord {
            score: 10_000_000,
            ..dto::ScoreRecord::default()
        };
        record.perfect = 1_000;
        record.clear_type = Some(3);
        assert!(band_hit(Band::Pm, &record));
        assert!(band_hit(Band::Fr, &record));
        assert!(band_hit(Band::Grade(Grade::ExPlus), &record));
        record.score = 9_850_000;
        record.far = 3;
        record.clear_type = None;
        assert!(!band_hit(Band::Pm, &record));
        assert!(band_hit(Band::Fr, &record));
        record.lost = 1;
        assert!(!band_hit(Band::Fr, &record));
        assert!(band_hit(Band::Grade(Grade::Ex), &record));
    }

    #[test]
    fn case_058() {
        let mut record = dto::ScoreRecord {
            constant: 10.0,
            ..dto::ScoreRecord::default()
        };
        record.score = 9_800_000;
        assert!((ptt_of(&record) - 11.2).abs() < 1e-9);
        record.rating = 12.34;
        assert!((ptt_of(&record) - 12.34).abs() < 1e-9);
    }

    #[test]
    fn case_059() {
        for wanted in [1u32, 365, TREND_ALL_DAYS] {
            assert!((1..=TREND_MAX_DAYS).contains(&wanted));
        }
        assert!(!(1..=TREND_MAX_DAYS).contains(&(TREND_MAX_DAYS + 1)));
    }

    fn record(score: i32, constant: f64) -> dto::ScoreRecord {
        let mut record = dto::ScoreRecord {
            song_id: String::from("abstrusedilemma"),
            ..dto::ScoreRecord::default()
        };
        record.title = String::from("Abstruse Dilemma");
        record.artist = String::from("ああああ");
        record.difficulty = 2;
        record.difficulty_label = String::from("10+");
        record.score = score;
        record.constant = constant;
        record.rank = String::from("EX");
        record.perfect = 1_300;
        record.far = 3;
        record.lost = 0;
        record.object_amount = 1_303;
        record.clear_type = Some(1);
        record
    }

    fn catalog() -> Vec<dto::Song> {
        let mut song = dto::Song {
            song_id: String::from("abstrusedilemma"),
            ..dto::Song::default()
        };
        song.title = String::from("Abstruse Dilemma");
        song.artist = String::from("ああああ");
        song.bpm = String::from("200");
        song.set = String::from("pack1");
        song.pack_name = String::from("Pack");
        let mut chart = dto::Chart {
            rating_class: 2,
            ..dto::Chart::default()
        };
        chart.rating_display = String::from("10+");
        chart.constant = Some(10.7);
        chart.note = Some(1_303);
        chart.chart_designer = String::from("Nitro");
        song.charts = vec![chart];
        vec![song]
    }

    fn badge() -> PlayerBadge {
        let identity = dto::PlayerIdentity {
            name: String::from("Afkeru"),
            current_ptt: Some(12.3),
            rating_frame: Some(4),
            rating_accent: Some(4),
            ..dto::PlayerIdentity::default()
        };
        bridge::badge_of(&identity, &crate::model::Prefs::default())
    }

    fn fits(kind: crate::cards::CardKind, width: i32, height: i32) -> bool {
        crate::cards::spec_of(kind).accepts(width, height)
    }

    #[test]
    fn case_060() {
        let catalog = catalog();
        let score = record(9_978_342, 10.7);
        let entry = bridge::list_entry_of(&score, "10+");
        let list_input = bridge::list_input(
            badge(),
            Theme::DEFAULT,
            "成绩清单",
            "只画一行",
            "只画一行",
            vec![entry],
        );
        let list = list_cards::request(&list_input);
        assert!(fits(crate::cards::CardKind::List, list.width, list.height));

        let table_input = bridge::table_input(
            badge(),
            Theme::DEFAULT,
            bridge::TableHead {
                title: "定数表",
                note: "只画一张",
            },
            &catalog,
            TABLE_MAX,
            list_cards::TableMode::Charts,
            None,
        );
        let table = list_cards::table_request(&table_input);
        assert!(fits(crate::cards::CardKind::Table, table.width, table.height));

        let ahv_input = bridge::ahv_input(
            badge(),
            Theme::DEFAULT,
            "成就表",
            "只画一张",
            vec![bridge::ahv_record_of(&score)],
            "2026-01-01 00:00",
        );
        let ahv = list_cards::ahv_request(&ahv_input);
        assert!(fits(crate::cards::CardKind::Ahv, ahv.width, ahv.height));

        let mut stats_data = dto::PlayerStats {
            total: 1,
            ..dto::PlayerStats::default()
        };
        stats_data.played = 1;
        stats_data.bands = vec![dto::StatsBand {
            difficulty: 2,
            total: 1,
            played: 1,
            ..dto::StatsBand::default()
        }];
        stats_data.consts = vec![dto::StatsConst {
            constant: 10.7,
            total: 1,
            played: 1,
        }];
        stats_data.grades = vec![dto::StatsGrade {
            grade: String::from("EX"),
            count: 1,
        }];
        stats_data.short = Some(vec![dto::StatsShort {
            clear_type: Some(3),
            song_id: String::from("abstrusedilemma"),
            title: String::from("Abstruse Dilemma"),
            difficulty: 2,
            constant: Some(10.7),
            has_constant: true,
            score: 9_978_342,
            grade: String::from("EX"),
            next_rank: String::from("EX+"),
            next_need: 12_000,
        }]);
        let stats_input = bridge::stats_input(
            badge(),
            bridge::StatsArgs {
                title: "区间统计",
                note: "只画一行",
                data: &stats_data,
                bands: &stats_data.bands,
                consts: &stats_data.consts,
                short: stats_data.short.as_deref().unwrap_or(&[]),
                short_title: "待补清单",
                short_note: "",
            },
        );
        let stats = stats_cards::stats_request(&stats_input);
        assert!(fits(crate::cards::CardKind::Stats, stats.width, stats.height));

        let mut hint = dto::SuggestHint {
            song_id: String::from("abstrusedilemma"),
            ..dto::SuggestHint::default()
        };
        hint.title = String::from("Abstruse Dilemma");
        hint.difficulty = 2;
        hint.constant = 10.7;
        hint.score = 9_978_342;
        hint.has_score = true;
        hint.target = 9_990_000.0;
        hint.delta = 11_658.0;
        hint.rating = 12.3;
        hint.target_rating = 12.4;
        hint.rank = 1;
        hint.target_rank = 1;
        hint.pick = true;
        let mut suggest_data = dto::PlayerSuggest {
            target: 12.5,
            ..dto::PlayerSuggest::default()
        };
        suggest_data.pool = 1;
        suggest_data.pushable = 1;
        suggest_data.hints = vec![hint];
        let suggest_input = bridge::suggest_input(
            badge(),
            Theme::DEFAULT,
            "推分建议",
            "只画一张",
            &suggest_data,
        );
        let suggest = stats_cards::suggest_request(&suggest_input);
        assert!(fits(
            crate::cards::CardKind::Suggest,
            suggest.width,
            suggest.height
        ));

        let mut chapter = dto::ChapterPack {
            pack_id: String::from("pack1"),
            ..dto::ChapterPack::default()
        };
        chapter.name = String::from("Pack");
        chapter.total = 1;
        chapter.played = 1;
        let mut chap_data = dto::PlayerChapters::default();
        chap_data.totals.packs = 1;
        chap_data.totals.total = 1;
        chap_data.sections = vec![dto::ChapterSection {
            id: String::from("main"),
            name: String::from("主线"),
            packs: vec![chapter],
        }];
        let chap_input =
            bridge::chap_input(
                badge(),
                Theme::DEFAULT,
                "曲包一览",
                "只有一个曲包",
                "只有一包",
                &chap_data,
            );
        let chap = stats_cards::chap_request(&chap_input);
        assert!(fits(crate::cards::CardKind::Chap, chap.width, chap.height));

        let point = stats_cards::TrendPoint::new("01-01", 12.3).with_score(4);
        let trend_input =
            bridge::trend_input(badge(), Theme::DEFAULT, "潜力值趋势", "只有一个点", vec![point]);
        let trend = stats_cards::trend_request(&trend_input);
        assert!(fits(crate::cards::CardKind::Trend, trend.width, trend.height));

        let library: Vec<domain::Song> = catalog
            .iter()
            .map(bridge::song_of)
            .collect();
        let rows = vec![dto::SongHistoryRow {
            difficulty: 2,
            has_old: true,
            old_score: 9_900_000,
            new_score: 9_978_342,
            new_perfect: 1_300,
            new_miss: 0,
            ..dto::SongHistoryRow::default()
        }];
        let sections = vec![(Difficulty::Future, rows)];
        let his_input =
            bridge::his_input(badge(), Theme::DEFAULT, &library[0], &sections).with_note("一条记录");
        let his = stats_cards::his_request(&his_input);
        assert!(fits(crate::cards::CardKind::His, his.width, his.height));
        assert_eq!((list.width, list.height), (1188, 360));
        assert_eq!((table.width, table.height), (1409, 549));
        assert_eq!((ahv.width, ahv.height), (1600, 559));
        assert_eq!((stats.width, stats.height), (1080, 852));
        assert_eq!((suggest.width, suggest.height), (1400, 405));
        assert_eq!((chap.width, chap.height), (1350, 417));
        assert_eq!((trend.width, trend.height), (1080, 747));
        assert_eq!((his.width, his.height), (1080, 395));
    }

    #[test]
    fn case_061() {
        use crate::commands::normalize;
        use crate::model::Region;

        assert_eq!(
            normalize::route(Region::Intl, "table11"),
            Some(("table", String::from("11")))
        );
        assert_eq!(
            normalize::route(Region::Intl, "推分ftr"),
            Some(("suggest", String::from("ftr")))
        );
        assert_eq!(
            normalize::route(Region::Cn, "ctrend365"),
            Some(("trend", String::from("365")))
        );
        assert_eq!(
            normalize::route(Region::Intl, "his骨折"),
            Some(("his", String::from("骨折")))
        );
        assert_eq!(normalize::route(Region::Intl, "list"), None);
    }
    #[test]
    fn case_062() {
        assert_eq!(
            stats_short_text(StatsView::Uncleared, 6),
            (
                String::from("待通关 · 最接近通关线的 6 首"),
                String::from("这一段全部通关了")
            )
        );
        assert_eq!(
            stats_short_text(StatsView::Unfc, 6),
            (
                String::from("待 FC · 分数最高的 6 首"),
                String::from("这一段全部 FC 了")
            )
        );
        assert_eq!(
            stats_short_text(StatsView::Unpm, 6),
            (
                String::from("待 PM · 分数最高的 6 首"),
                String::from("这一段全部满分了")
            )
        );
        assert_eq!(
            stats_short_text(StatsView::All, 6),
            (
                String::from("短板 · 分数最低的 6 首"),
                String::from("这一段还没有成绩")
            )
        );
    }

    #[test]
    fn case_063() {
        let mut data = dto::PlayerStats {
            bands: vec![dto::StatsBand {
                difficulty: 2,
                total: 10,
                ..dto::StatsBand::default()
            }],
            ..dto::PlayerStats::default()
        };
        data.consts = vec![dto::StatsConst {
            constant: 10.7,
            total: 3,
            played: 1,
        }];
        let (bands, consts) = stats_sections(StatsDetail::Both, &data);
        assert_eq!((bands.len(), consts.len()), (1, 1));
        let (bands, consts) = stats_sections(StatsDetail::Difficulty, &data);
        assert_eq!(bands.len(), 1, "d 要画难度段");
        assert!(consts.is_empty(), "d 不画定数段");
        let (bands, consts) = stats_sections(StatsDetail::Constant, &data);
        assert!(bands.is_empty(), "c 不画难度段");
        assert_eq!(consts.len(), 1, "c 要画定数段");
    }

    #[test]
    fn case_064() {
        let late = chrono::DateTime::parse_from_rfc3339("2026-03-01T23:30:00Z")
            .expect("时间要能解析")
            .with_timezone(&chrono::Utc);
        assert_eq!(day_key(late), "2026-03-02", "UTC 深夜在东八区已经跨天");
        assert_eq!(day_label(late), "03-02");
        let edge = chrono::DateTime::parse_from_rfc3339("2026-03-01T15:59:00Z")
            .expect("时间要能解析")
            .with_timezone(&chrono::Utc);
        assert_eq!(day_key(edge), "2026-03-01", "东八区 23:59 还在当天");
        assert_eq!(day_label(edge), "03-01");
        let morning = chrono::DateTime::parse_from_rfc3339("2026-03-01T00:30:00Z")
            .expect("时间要能解析")
            .with_timezone(&chrono::Utc);
        assert_eq!(day_key(morning), "2026-03-01");
    }

    #[test]
    fn case_065() {
        assert_eq!(StatsView::All.as_str(), "all");
        assert_eq!(StatsView::Uncleared.as_str(), "uncleared");
        assert_eq!(StatsView::Unfc.as_str(), "unfc");
        assert_eq!(StatsView::Unpm.as_str(), "unpm");
    }

    fn case_066(rank: i32, score: i32, constant: f64, rating: f64) -> dto::ScoreRecord {
        let mut record = record(score, constant);
        record.song_id = format!("song{rank}{score}");
        record.title = format!("曲 {rank} {score}");
        record.difficulty = rank;
        record.rating = rating;
        record
    }

    #[test]
    fn case_067() {
        let beyond = case_066(3, 9_000_000, 10.0, 10.0);
        let past = case_066(0, 10_000_000, 5.0, 12.0);
        assert_eq!(
            compare_rows(Order::Ptt, true, &beyond, &past),
            std::cmp::Ordering::Less,
            "超越档的低分要排在过去档的高分之前"
        );
        assert_eq!(
            compare_rows(Order::Score, true, &beyond, &past),
            std::cmp::Ordering::Less,
            "换主键也不改难度带的先后"
        );
        let low = case_066(2, 9_000_000, 10.0, 11.0);
        let high = case_066(2, 9_000_000, 10.0, 12.0);
        assert_eq!(
            compare_rows(Order::Ptt, true, &low, &high),
            std::cmp::Ordering::Greater,
            "同一带里按单曲 PTT 降序"
        );
        assert_eq!(
            compare_rows(Order::Ptt, false, &low, &high),
            std::cmp::Ordering::Less,
            "带内主键的方向跟着排序词走"
        );
    }

    #[test]
    fn case_068() {
        let base = case_066(2, 9_800_000, 10.0, 11.0);
        let mut higher_score = base.clone();
        higher_score.score = 9_900_000;
        assert_eq!(
            compare_rows(Order::Ptt, true, &base, &higher_score),
            std::cmp::Ordering::Greater,
            "PTT 打平时分数高的在前"
        );
        let mut higher_const = higher_score.clone();
        higher_const.constant = 10.5;
        assert_eq!(
            compare_rows(Order::Ptt, true, &higher_const, &higher_score),
            std::cmp::Ordering::Less,
            "分数也打平时定数高的在前"
        );
        let mut better_clear = higher_score.clone();
        better_clear.clear_type = Some(3);
        assert_eq!(
            compare_rows(Order::Score, true, &higher_score, &better_clear),
            std::cmp::Ordering::Greater,
            "Score 主键下平局先比定数，再比 PTT，最后比通关类型"
        );
        let mut zeta = higher_score.clone();
        zeta.title = String::from("Zeta");
        let mut alpha = higher_score.clone();
        alpha.title = String::from("alpha");
        assert_eq!(
            compare_rows(Order::Score, true, &zeta, &alpha),
            std::cmp::Ordering::Greater,
            "全打平时按曲名忽略大小写 A→Z"
        );
    }

    #[test]
    fn case_069() {
        let 全空 = domain::Filter::parse("").expect("空条件要能解析");
        assert_eq!(table_mode_of(全空.as_ref()), list_cards::TableMode::Songs);
        assert_eq!(table_mode_of(None), list_cards::TableMode::Songs);
        let 定数 = domain::Filter::parse("10-12").expect("定数区间要能解析");
        assert_eq!(table_mode_of(定数.as_ref()), list_cards::TableMode::Charts);
        let 等级 = domain::Filter::parse("9+").expect("等级要能解析");
        assert_eq!(table_mode_of(等级.as_ref()), list_cards::TableMode::Charts);
        let 难度 = domain::Filter::parse("ftr").expect("难度要能解析");
        assert_eq!(table_mode_of(难度.as_ref()), list_cards::TableMode::Charts);
        let 曲名 = domain::Filter::parse("骨折").expect("曲名要能解析");
        assert_eq!(table_mode_of(曲名.as_ref()), list_cards::TableMode::Songs);
    }

    #[test]
    fn case_070() {
        let catalog = catalog();
        let input = bridge::table_input(
            badge(),
            Theme::DEFAULT,
            bridge::TableHead {
                title: "定数表",
                note: "只读曲库",
            },
            &catalog,
            TABLE_MAX,
            list_cards::TableMode::Songs,
            None,
        );
        assert_eq!(input.mode, list_cards::TableMode::Songs);
        assert!(input.sheets.is_empty(), "曲目模式不分档");
        assert_eq!(input.songs.len(), catalog.len());
        let song = &input.songs[0];
        assert_eq!(song.song_id, "abstrusedilemma");
        let known = song.known_cells();
        assert_eq!(known.len(), 1, "夹具只有一张 FTR 谱面");
        assert_eq!(known[0].difficulty, Difficulty::Future);
        assert_eq!(known[0].constant, 10.7);
        let request = list_cards::table_request(&input);
        assert!(fits(crate::cards::CardKind::Table, request.width, request.height));
    }

}
