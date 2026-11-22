use std::collections::HashMap;

use std::sync::Arc;

use aurorix_plugin::{Outgoing, PluginError};
use chrono::Utc;

use crate::backend::{B50Payload, PlayerBoard, B50};
use crate::commands::bridge;
use crate::commands::{
    ago, best_notes, bound_account, buttons, freshness, pending_outgoing, prefs_of, reply, repo_of,
    root_of, Ctx,
};
use crate::domain::{self, BandCounts, BoardInput, BoardKind, OverBand};
use crate::repo::SnapshotRecord;

pub fn describe(kind: BoardKind) -> &'static str {
    match kind {
        BoardKind::Pm50 => "PM 50：已 Pure Memory —— 0 Far 0 Lost（理论值也在里面）",
        BoardKind::Fr50 => "全连 50（FR）：0 Lost，已 PM 的也在里面",
        BoardKind::X50 => {
            "差一点 50：只有一点点没接住 —— F + L ≤ 2（Far 与 Lost 直接相加）"
        }
        BoardKind::Nm50 => "近 PM 50：还没 PM 里离 PM 最近的 50 条（差一点的也在里面）",
        BoardKind::Mx50 => "理论值 50：10'000'000（全大 P）",
        BoardKind::Ol50 => "越级 50：每一局和你在这档定数上的常态比，按越级量排序",
    }
}

fn empty_text(kind: BoardKind) -> &'static str {
    match kind {
        BoardKind::Fr50 => {
            "还没有全连的成绩：打一首 0 Lost 的歌, 它就会出现在这里（PM 的也算全连）"
        }
        BoardKind::X50 => {
            "没有差一点的成绩：F + L ≤ 2（Far 与 Lost 直接相加）的那几局会出现在这里"
        }
        BoardKind::Nm50 => "本地成绩已经全是 PM 了，没有「近 PM」可看",
        BoardKind::Mx50 => "还没有理论值：打到 10'000'000（全大 P）的谱面会出现在这里",
        BoardKind::Ol50 => "还没有成绩，可以先查询一下 Best50 或者等待后台自动按照定数从高到低补扫",
        BoardKind::Pm50 => "还没有 PM 的成绩：打一首 0 Far 0 Lost 的歌, 它就会出现在这里",
    }
}

fn share(part: i64, whole: i64) -> String {
    if whole <= 0 {
        return String::from("—");
    }
    format!("{:.0}%", part as f64 / whole as f64 * 100.0)
}

fn board_stat(kind: BoardKind, board: &PlayerBoard, picked: &domain::Board) -> (String, String) {
    match kind {
        BoardKind::Fr50 => (String::from("FR 覆盖率"), share(board.matched, board.known)),
        BoardKind::X50 => (String::from("F + L ≤ 2"), board.matched.to_string()),
        BoardKind::Nm50 => (
            String::from("平均差 PTT"),
            format!("{:.2}", picked.mean_gap.unwrap_or(0.0)),
        ),
        BoardKind::Mx50 => (String::from("理论值谱面"), board.matched.to_string()),
        BoardKind::Ol50 if board.has_curve => (
            String::from("能力线"),
            format!("{:.2}", board.anchor.unwrap_or(0.0)),
        ),
        BoardKind::Ol50 => (
            String::from("平均压力"),
            format!("{:.0}", board.mean_pressure),
        ),
        BoardKind::Pm50 => (String::from("PM 覆盖率"), share(board.matched, board.known)),
    }
}

fn band_counts_of(counts: &HashMap<String, i64>) -> BandCounts {
    let mut out = BandCounts::default();
    for (key, value) in counts {
        let Some(band) = OverBand::parse(key) else {
            continue;
        };
        for _ in 0..(*value).max(0) {
            out.add(band);
        }
    }
    out
}

pub async fn b50(ctx: &Ctx<'_>, _arg: &str) -> Result<Option<Outgoing>, PluginError> {
    fetch(ctx, false).await
}

pub async fn b50_force(ctx: &Ctx<'_>) -> Result<Option<Outgoing>, PluginError> {
    fetch(ctx, true).await
}

async fn fetch(ctx: &Ctx<'_>, refresh: bool) -> Result<Option<Outgoing>, PluginError> {
    let repo = match repo_of(ctx).await {
        Ok(repo) => repo,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    let account = match bound_account(ctx, &repo).await {
        Ok(account) => account,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    let Some(uid) = account.uid() else {
        return Ok(Some(reply::unbound(ctx.region)));
    };
    match ctx.backend.b50(ctx.region, uid, refresh).await {
        Ok(mut b50) => {
            if b50.meta.awaiting_task() {
                let Some(sender) = ctx.inv.opt_sender.clone() else {
                    return Ok(Some(pending_outgoing(&b50.meta)));
                };
                let inv = ctx.inv.clone();
                let backend = ctx.backend.clone();
                let region = ctx.region;
                let repo = Arc::new(repo);
                let task_id = b50.meta.task_id.clone();
                tokio::spawn(async move {
                    let repo = Arc::clone(&repo);
                    let ctx = Ctx::new(&inv, region, &backend);
                    let out = finish_b50(ctx, repo, account, task_id).await;
                    let _ = sender.send(out).await;
                });
                return Ok(Some(pending_outgoing(&b50.meta)));
            }
            if b50.identity.global_rank.is_none() {
                b50.identity.global_rank = world_rank(ctx, Some(uid)).await;
            }
            remember(ctx, &repo, &account, &b50).await;
            let prefs = prefs_of(ctx, &repo).await;
            let freshness = freshness(&b50.meta);
            Ok(Some(b50_outgoing(ctx, &b50, &freshness, &prefs, None)))
        }
        Err(error) => Ok(Some(reply::fail(error.user_message()))),
    }
}

async fn finish_b50(
    ctx: Ctx<'_>,
    repo: Arc<crate::repo::ArcaeaRepo>,
    account: crate::model::Account,
    task_id: String,
) -> Outgoing {
    for _ in 0..20 {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        match ctx.backend.b50_task(ctx.region, &task_id, Some(15)).await {
            Ok(b50) if b50.meta.awaiting_task() => continue,
            Ok(b50) => {
                remember(&ctx, &repo, &account, &b50).await;
                let prefs = prefs_of(&ctx, &repo).await;
                let freshness = freshness(&b50.meta);
                return b50_outgoing(&ctx, &b50, &freshness, &prefs, None);
            }
            Err(error) => return reply::fail(error.user_message()),
        }
    }
    reply::fail("后端仍在扫描，请稍后重试")
}

async fn remember(
    ctx: &Ctx<'_>,
    repo: &crate::repo::ArcaeaRepo,
    account: &crate::model::Account,
    b50: &B50,
) {
    let Ok(payload) = serde_json::to_value(b50.payload()) else {
        return;
    };
    let snapshot = SnapshotRecord {
        region: ctx.region,
        user_id: ctx.inv.user.clone(),
        payload,
        arcaea_uid: account.uid(),
        friend_code: account.friend_code.clone(),
        captured_at: Utc::now(),
    };
    let _ = repo.save_snapshot(&snapshot).await;
}

pub async fn b50_last(ctx: &Ctx<'_>) -> Result<Option<Outgoing>, PluginError> {
    let repo = match repo_of(ctx).await {
        Ok(repo) => repo,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    if let Err(outgoing) = bound_account(ctx, &repo).await {
        return Ok(Some(*outgoing));
    }
    let snapshot = match repo.load_snapshot(ctx.region, &ctx.inv.user).await {
        Ok(snapshot) => snapshot,
        Err(error) => return Ok(Some(reply::fail(error.to_string()))),
    };
    let Some(snapshot) = snapshot else {
        return Ok(Some(reply::notice(format!(
            "还没有可重放的 Best 50，先发一次 {} b50 查一份，之后就能用 {} b50 last 看回来",
            root_of(ctx),
            root_of(ctx)
        ))));
    };
    let Ok(payload) = serde_json::from_value::<B50Payload>(snapshot.payload.clone()) else {
        return Ok(Some(reply::notice(format!(
            "上一次的 Best 50 快照读不出来，先发一次 {} b50 重查一份",
            root_of(ctx)
        ))));
    };
    let meta = payload.meta.clone().unwrap_or(crate::backend::QueryMeta {
        cached: true,
        cached_at: Some(snapshot.captured_at),
        ..crate::backend::QueryMeta::default()
    });
    let b50 = B50::from_payload(payload, meta);
    let freshness = format!("上一次查询 · {}", ago(snapshot.captured_at));
    let prefs = prefs_of(ctx, &repo).await;
    Ok(Some(b50_outgoing(ctx, &b50, &freshness, &prefs, None)))
}

pub async fn rank50(ctx: &Ctx<'_>, kind: BoardKind) -> Result<Option<Outgoing>, PluginError> {
    let repo = match repo_of(ctx).await {
        Ok(repo) => repo,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    let account = match bound_account(ctx, &repo).await {
        Ok(account) => account,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    let Some(uid) = account.uid() else {
        return Ok(Some(reply::unbound(ctx.region)));
    };
    let board = match ctx
        .backend
        .boards(ctx.region, uid, kind.as_str(), Some(50))
        .await
    {
        Ok(board) => board,
        Err(error) => return Ok(Some(reply::fail(error.user_message()))),
    };
    if board.rows.is_empty() {
        return Ok(Some(reply::notice(empty_text(kind))));
    }
    let records = board
        .rows
        .iter()
        .map(bridge::board_record_of)
        .collect::<Vec<_>>();
    let mut source = BoardInput::new(records).with_curve(board.has_curve);
    if let Some(anchor) = board.anchor {
        source = source.with_anchor(anchor);
    }
    if let Some(upper) = board.anchor_upper {
        source = source.with_upper(upper);
    }
    source = source
        .with_counts(
            band_counts_of(&board.band_counts),
            board.over_count.max(0) as usize,
        )
        .with_pressure(board.mean_pressure)
        .with_averages(
            board.avg_best10,
            board.avg_best50,
            board.best_count.max(0) as usize,
        )
        .with_mean_gap(board.mean_gap);
    let picked = domain::select_board(&source, kind, domain::BOARD_TOP);
    let shown = picked.rows.len();
    if shown == 0 {
        return Ok(Some(reply::notice(empty_text(kind))));
    }
    let prefs = prefs_of(ctx, &repo).await;
    let identity = board.identity.named(account.name.as_str());
    let stat = board_stat(kind, &board, &picked);
    let averages = (
        picked.avg_best10,
        picked.avg_best50,
        picked.best_count.unwrap_or(0),
    );
    let input = bridge::identity_input(ctx.region, ctx.region.is_cn(), &identity, &prefs, Some(kind))
        .with_best(picked.rows)
        .with_board_stat(stat.0, stat.1)
        .with_averages(averages.0, averages.1, averages.2)
        .with_anchor_upper(board.anchor_upper.unwrap_or(0.0))
        .with_over_count(board.over_count.max(0) as usize);
    let mut args = reply::args();
    if kind == BoardKind::Ol50 {
        let label = if board.calibrated {
            "谱面体感定数(全服校准，非官标)"
        } else {
            "谱面官方定数"
        };
        reply::note(
            &mut args,
            format!("能力线为全量成绩估算稳拿EX的定数；越级量={label}−能力线"),
        );
        reply::note(
            &mut args,
            "越级上 EX+ 记「攻克」，上 EX 记达标，未上 EX 按跨度与掉分记「小越/大越/打不动」；在能力线附近但失误记「翻车」",
        );
    }
    Ok(Some(reply::board(
        reply::card(
            reply::text("query/query.list.caption", &args),
            crate::cards::b50::card(input),
        ),
        buttons::rank_board(ctx.region).or_else(|| buttons::rank_board_fallback(ctx.region)),
    )))
}

fn b50_outgoing(
    ctx: &Ctx<'_>,
    b50: &B50,
    freshness: &str,
    prefs: &crate::model::Prefs,
    board: Option<BoardKind>,
) -> Outgoing {
    let input = bridge::b50_input(ctx.region, ctx.region.is_cn(), b50, prefs, board);
    let mut args = reply::args();
    reply::put(&mut args, "name", b50.identity.name.clone());
    reply::put(&mut args, "ptt", format!("{:.4}", b50.identity.ptt().unwrap_or(0.0)));
    reply::put(&mut args, "best50", format!("{:.4}", b50.best50));
    reply::put(&mut args, "best10", format!("{:.4}", b50.best10));
    reply::put(&mut args, "max_ptt", format!("{:.4}", b50.max_ptt));
    reply::put(&mut args, "count", b50.count.to_string());
    reply::put(&mut args, "freshness", freshness);
    for (index, record) in b50.records.iter().enumerate() {
        let judge = match record.judged() {
            true => format!("{}(-{})", record.perfect, record.normal_perfect),
            false => String::from("—"),
        };
        reply::row(
            &mut args,
            "records",
            &[
                ("index", (index + 1).to_string()),
                ("title", record.title.clone()),
                ("grade", bridge::grade_of(record).as_str().to_string()),
                (
                    "difficulty",
                    bridge::difficulty_of(record).as_str().to_string(),
                ),
                ("constant", format!("{:.1}", record.constant)),
                ("score", record.score.to_string()),
                ("rating", format!("{:.4}", record.rating)),
                ("judge", judge),
            ],
        );
    }
    for note in best_notes(&b50.meta, ctx.region) {
        reply::note(&mut args, note);
    }
    let full = reply::fill("score/score.b50", &args);
    reply::board(
        reply::card_or(
            reply::text("score/score.b50.fresh", &args),
            crate::cards::b50::card(input),
            full,
        ),
        buttons::best_board(ctx.region),
    )
}

async fn world_rank(ctx: &Ctx<'_>, uid: Option<i64>) -> Option<i32> {
    let uid = uid?;
    match ctx.backend.player(ctx.region, uid).await {
        Ok(Some(player)) => player.identity.global_rank,
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{QueryMeta, ScoreRecord};

    fn sample() -> B50 {
        let mut record = ScoreRecord {
            song_id: "abstrusedilemma".to_string(),
            ..ScoreRecord::default()
        };
        record.title = "Abstruse Dilemma".to_string();
        record.difficulty = 2;
        record.score = 9_978_342;
        record.constant = 10.7;
        record.rating = 12.3;
        record.rank = "EX+".to_string();
        record.perfect = 1_300;
        record.far = 3;
        record.lost = 0;
        record.object_amount = 1_303;
        record.clear_type = Some(1);
        record.time_played = 1_700_000_000;
        let mut b50 = B50::default();
        b50.identity.name = "Afkeru".to_string();
        b50.identity.current_ptt = Some(12.31);
        b50.identity.rating_frame = Some(4);
        b50.identity.rating_accent = Some(2);
        b50.best50 = 12.0;
        b50.best10 = 12.6;
        b50.max_ptt = 12.9;
        b50.count = 1;
        b50.records = vec![record];
        b50.identity.skin.char_pref = "对立_tairitsu_1.png".to_string();
        b50.identity.skin.banner = "online_banner_2026_10".to_string();
        b50.identity.skin.banner_pref = "course_banner_01.png".to_string();
        b50
    }

    #[test]
    fn case_071() {
        let origin = sample();
        let value = serde_json::to_value(origin.payload()).expect("快照载荷要能序列化");
        let payload: B50Payload = serde_json::from_value(value).expect("快照载荷要能反序列化");
        let restored = B50::from_payload(payload, QueryMeta::default());
        assert_eq!(restored.identity.name, origin.identity.name);
        assert_eq!(restored.identity.current_ptt, origin.identity.current_ptt);
        assert_eq!(restored.identity.rating_frame, origin.identity.rating_frame);
        assert_eq!(
            restored.identity.rating_accent,
            origin.identity.rating_accent
        );
        assert_eq!(restored.best50, origin.best50);
        assert_eq!(restored.best10, origin.best10);
        assert_eq!(restored.max_ptt, origin.max_ptt);
        assert_eq!(
            restored.identity.skin, origin.identity.skin,
            "自选素材要跟着快照一起回来"
        );
        assert_eq!(restored.records, origin.records, "成绩行要一模一样");
        assert_eq!(restored.count, restored.records.len() as i32);
    }

    #[test]
    fn case_072() {
        let board = PlayerBoard {
            matched: 296,
            known: 1603,
            ..PlayerBoard::default()
        };
        let picked = domain::Board {
            kind: BoardKind::X50,
            matched: 50,
            known: 1603,
            anchor: None,
            rows: Vec::new(),
            band_counts: BandCounts::default(),
            over_count: 0,
            avg_best10: None,
            avg_best50: None,
            best_count: None,
            mean_gap: None,
        };
        let stat = board_stat(BoardKind::X50, &board, &picked);
        assert_eq!(stat.0, "F + L ≤ 2");
        assert_eq!(
            stat.1, "296",
            "第三格要数全部符合条件的谱面, 不是榜上那 50 行"
        );
    }
}

