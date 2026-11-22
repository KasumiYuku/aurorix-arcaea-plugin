use aurorix_plugin::{Outgoing, PluginError, RenderCard};

use crate::backend::{Play, ScoreRecord};
use crate::cards::score::{recent_card, score_card};
use crate::commands::bridge;
use crate::commands::{
    ack_after, bound_account, buttons, freshness_split, local_time, one, pending_outgoing,
    prefs_of, reply, repo_of, root_of, split_difficulty, stale_notes, Ctx,
};
use crate::domain::{self, ptt, Song};
use crate::model::{diff_order_weight, Difficulty};

fn missing_record(error: &crate::backend::BackendError) -> bool {
    error.status() == Some(404) || error.code() == Some("not_found")
}

pub fn error_line(error: &crate::backend::BackendError) -> String {
    error.user_message()
}
pub async fn recent(ctx: &Ctx<'_>) -> Result<Option<Outgoing>, PluginError> {
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
    let inv = ctx.inv.clone();
    let backend = ctx.backend.clone();
    let region = ctx.region;
    let repo = std::sync::Arc::new(repo);
    let run_repo = std::sync::Arc::clone(&repo);
    ack_after(ctx, "正在查询最近一次游玩，完成后会 @ 你", async move {
        let ctx = Ctx::new(&inv, region, &backend);
        recent_once(&ctx, &run_repo, uid).await
    })
    .await
}

async fn recent_once(
    ctx: &Ctx<'_>,
    repo: &std::sync::Arc<crate::repo::ArcaeaRepo>,
    uid: i64,
) -> Outgoing {
    match ctx.backend.recent(ctx.region, uid).await {
        Ok(play) => {
            if play.record.is_none() && play.meta.awaiting_task() {
                if let Some(sender) = ctx.inv.opt_sender.clone() {
                    let inv = ctx.inv.clone();
                    let backend = ctx.backend.clone();
                    let region = ctx.region;
                    let repo = std::sync::Arc::clone(repo);
                    let meta = play.meta.clone();
                    let task_id = meta.task_id.clone();
                    tokio::spawn(async move {
                        let ctx = Ctx::new(&inv, region, &backend);
                        let out = finish_recent(&ctx, repo, task_id).await;
                        let _ = sender.send(out).await;
                    });
                    return pending_outgoing(&meta);
                }
                return pending_outgoing(&play.meta);
            }
            let Some(record) = play.record.clone() else {
                return reply::notice("还没有可查的游玩记录");
            };
            if !play.found {
                return reply::notice("还没有可查的游玩记录");
            }
            let rank =
                crate::commands::bridge::world_rank(ctx.backend, ctx.region, uid).await;
            recent_rendered(ctx, repo, &play, &record, rank).await
        }
        Err(error) => reply::fail(error_line(&error)),
    }
}

pub async fn single(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    let (query, wanted) = split_difficulty(arg);
    let query = query.as_str();
    if query.trim().is_empty() {
        return Ok(Some(reply::notice(format!(
            "用法：{} score 曲目 [难度]\n曲目支持 ID、别名、歌名与模糊输入，难度可填 PST/PRS/FTR/BYD/ETR/INS，默认 FTR",
            root_of(ctx)
        ))));
    }
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
    let found = match bridge::resolve_song(ctx.backend, ctx.region, query).await {
        Ok(Some(found)) => found,
        Ok(None) => return Ok(Some(no_song(query))),
        Err(error) if missing_record(&error) => return Ok(Some(no_song(query))),
        Err(error) => return Ok(Some(reply::fail(error_line(&error)))),
    };
    if found.matched.needs_pick(query) {
        return Ok(Some(reply::pick(
            root_of(ctx),
            "score",
            query,
            &found.matched,
        )));
    }
    let matched = match wanted {
        Some(difficulty) => found.matched.prefer_chart(difficulty),
        None => found.matched,
    };
    let song = &matched.best.song;
    if let Some(difficulty) = wanted.filter(|difficulty| song.chart(*difficulty).is_none()) {
        return Ok(Some(
            missing_score(
                ctx,
                uid,
                &matched,
                format!("《{}》没有 {} 难度", song.title, difficulty.as_str()),
            )
            .await,
        ));
    }
    let played = match played_difficulties(ctx, uid, &song.id).await {
        Ok(played) => played,
        Err(error) => return Ok(Some(reply::fail(error_line(&error)))),
    };
    let difficulty = match wanted {
        Some(difficulty) => difficulty,
        None => match played.iter().copied().max_by_key(|kind| diff_order_weight(*kind)) {
            Some(difficulty) => difficulty,
            None => return Ok(Some(no_plays(song))),
        },
    };
    let play = match ctx
        .backend
        .score(ctx.region, uid, &song.id, difficulty, true)
        .await
    {
        Ok(play) => play,
        Err(error) if missing_record(&error) => {
            return Ok(Some(
                missing_score(
                    ctx,
                    uid,
                    &matched,
                    format!(
                        "《{}》的 {} 谱面还没有你的成绩",
                        song.title,
                        difficulty.as_str()
                    ),
                )
                .await,
            ))
        }
        Err(error) => return Ok(Some(reply::fail(error_line(&error)))),
    };
    let Some(record) = play.record.clone() else {
        if play.meta.awaiting_task() {
            let meta = play.meta.clone();
            return Ok(Some(
                pending_with_task(ctx, repo, song.clone(), difficulty, &meta).await,
            ));
        }
        return Ok(Some(
            missing_score(
                ctx,
                uid,
                &matched,
                format!(
                    "《{}》的 {} 谱面还没有你的成绩",
                    song.title,
                    difficulty.as_str()
                ),
            )
            .await,
        ));
    };
    let mut notes = stale_notes(&play.meta);
    notes.extend(matched.notes_brief());
    let rank = match play.identity.global_rank {
        Some(rank) => Some(rank),
        None => bridge::world_rank(ctx.backend, ctx.region, uid).await,
    };
    Ok(Some(
        rendered(
            ctx,
            &repo,
            song,
            difficulty,
            &play,
            &record,
            ScoreExtras { notes, rank },
        )
        .await,
    ))
}

struct ScoreExtras {
    notes: Vec<String>,
    rank: Option<i32>,
}

async fn rendered(
    ctx: &Ctx<'_>,
    repo: &crate::repo::ArcaeaRepo,
    song: &Song,
    difficulty: Difficulty,
    play: &Play,
    record: &ScoreRecord,
    extras: ScoreExtras,
) -> Outgoing {
    let fresh = freshness_split(&play.meta);
    let prefs = prefs_of(ctx, repo).await;
    let theme = bridge::theme_of(&prefs);
    let level = song
        .chart(difficulty)
        .map(|chart| chart.level.clone())
        .unwrap_or_default();
    let mut identity = play.identity.clone();
    if identity.global_rank.is_none() {
        identity.global_rank = extras.rank;
    }
    let input = bridge::score_input(record, &identity, &prefs, theme, &level);
    let outgoing = play_outgoing(
        record,
        "单曲成绩",
        (&fresh.0, &fresh.1),
        &extras.notes,
        bridge::record_loss(record),
        score_card(input),
    );
    reply::board(
        outgoing,
        buttons::score_board(ctx.region, &song.id, difficulty),
    )
}

async fn pending_with_task(
    ctx: &Ctx<'_>,
    repo: crate::repo::ArcaeaRepo,
    song: Song,
    difficulty: Difficulty,
    meta: &crate::backend::QueryMeta,
) -> Outgoing {
    let Some(sender) = ctx.inv.opt_sender.clone() else {
        return pending_outgoing(meta);
    };
    let inv = ctx.inv.clone();
    let backend = ctx.backend.clone();
    let region = ctx.region;
    let repo = std::sync::Arc::new(repo);
    let task_id = meta.task_id.clone();
    tokio::spawn(async move {
        let ctx = Ctx::new(&inv, region, &backend);
        let out = finish_score(&ctx, repo, song, difficulty, task_id).await;
        let _ = sender.send(out).await;
    });
    pending_outgoing(meta)
}

async fn finish_score(
    ctx: &Ctx<'_>,
    repo: std::sync::Arc<crate::repo::ArcaeaRepo>,
    song: Song,
    difficulty: Difficulty,
    task_id: String,
) -> Outgoing {
    for _ in 0..20 {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        match ctx.backend.play_task(ctx.region, &task_id, Some(15)).await {
            Ok(play) if play.meta.awaiting_task() => continue,
            Ok(play) => match play.record.clone() {
                Some(record) => {
                    let notes = stale_notes(&play.meta);
                    return rendered(
                        ctx,
                        &repo,
                        &song,
                        difficulty,
                        &play,
                        &record,
                        ScoreExtras {
                            notes,
                            rank: play.identity.global_rank,
                        },
                    )
                    .await;
                }
                None => {
                    return reply::fail(format!(
                        "《{}》的 {} 谱面还没有你的成绩",
                        song.title,
                        difficulty.as_str()
                    ))
                }
            },
            Err(error) => return reply::fail(error_line(&error)),
        }
    }
    reply::fail("后端仍在查询，稍后重新发一次指令就能拿到结果")
}

async fn recent_rendered(
    ctx: &Ctx<'_>,
    repo: &crate::repo::ArcaeaRepo,
    play: &crate::backend::RecentPlay,
    record: &ScoreRecord,
    rank: Option<i32>,
) -> Outgoing {
    let fresh = freshness_split(&play.meta);
    let prefs = prefs_of(ctx, repo).await;
    let theme = bridge::theme_of(&prefs);
    let level = match ctx.backend.song(ctx.region, &record.song_id).await {
        Ok(song) => song
            .charts
            .iter()
            .find(|chart| chart.rating_class == record.difficulty)
            .map(|chart| chart.rating_display.clone())
            .unwrap_or_default(),
        Err(_) => String::new(),
    };
    let mut identity = play.identity.clone();
    if identity.global_rank.is_none() {
        identity.global_rank = rank;
    }
    let input = bridge::score_input(record, &identity, &prefs, theme, &level);
    let notes = stale_notes(&play.meta);
    let loss = match ctx.region.is_cn() {
        true => bridge::record_loss(record),
        false => None,
    };
    let outgoing = play_outgoing(
        record,
        "最近一次游玩",
        (&fresh.0, &fresh.1),
        &notes,
        loss,
        recent_card(input),
    );
    let difficulty = bridge::difficulty_of(record);
    reply::board(
        outgoing,
        buttons::recent_board(ctx.region, &record.song_id, difficulty),
    )
}

async fn finish_recent(
    ctx: &Ctx<'_>,
    repo: std::sync::Arc<crate::repo::ArcaeaRepo>,
    task_id: String,
) -> Outgoing {
    for _ in 0..20 {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        match ctx.backend.recent_task(ctx.region, &task_id, Some(15)).await {
            Ok(play) if play.meta.awaiting_task() => continue,
            Ok(play) => match play.record.clone() {
                Some(record) if play.found => {
                    return recent_rendered(ctx, &repo, &play, &record, None).await;
                }
                _ => return reply::notice("还没有可查的游玩记录"),
            },
            Err(error) => return reply::fail(error_line(&error)),
        }
    }
    reply::fail("后端仍在查询，稍后重新发一次指令就能拿到结果")
}

pub fn play_outgoing(
    record: &ScoreRecord,
    heading: &str,
    fresh: (&str, &str),
    notes: &[String],
    loss: Option<f64>,
    card: RenderCard,
) -> Outgoing {
    let difficulty = bridge::difficulty_of(record);
    let grade = bridge::grade_of(record);
    let rating = if record.rating > 0.0 {
        record.rating
    } else {
        ptt::single_rating(record.constant, i64::from(record.score))
    };
    let mut args = reply::args();
    reply::put(&mut args, "heading", heading);
    reply::put(&mut args, "title", record.title.clone());
    reply::put(&mut args, "difficulty", difficulty.as_str());
    reply::put(&mut args, "constant", one(record.constant));
    reply::put(&mut args, "score", record.score.to_string());
    reply::put(&mut args, "grade", grade.as_str());
    reply::put(&mut args, "rating", domain::format_rating(rating));
    reply::put(&mut args, "perfect", record.perfect.to_string());
    reply::put(&mut args, "far", record.far.to_string());
    reply::put(&mut args, "lost", record.lost.to_string());
    reply::put(&mut args, "played_at", played_at(record));
    let mut stamp = String::new();
    if !fresh.0.is_empty() {
        stamp.push_str(&format!("> {}\n", fresh.0));
    }
    if let Some(value) = loss {
        stamp.push_str(&format!("> LOSS SCORE: {value:.4}\n"));
    }
    if !fresh.1.is_empty() {
        stamp.push_str(&format!("> {}\n", fresh.1));
    }
    reply::put(&mut args, "stamp", stamp);
    reply::put(&mut args, "freshness", crate::commands::freshness_join(fresh));
    for note in notes {
        reply::note(&mut args, note.clone());
    }
    let full = reply::fill("score/score.play", &args);
    reply::card_or(reply::text("score/score.play.fresh", &args), card, full)
}

fn played_at(record: &ScoreRecord) -> String {
    match bridge::play_at(record.time_played) {
        Some(at) => local_time(at),
        None => "未知时间".to_string(),
    }
}

async fn played_difficulties(
    ctx: &Ctx<'_>,
    uid: i64,
    song_id: &str,
) -> Result<Vec<Difficulty>, crate::backend::BackendError> {
    let scores = ctx
        .backend
        .all_scores(ctx.region, uid, crate::backend::ScoreSource::Local)
        .await?;
    let mut played: Vec<Difficulty> = scores
        .records
        .iter()
        .filter(|record| record.song_id == song_id)
        .filter_map(|record| Difficulty::from_upstream(i64::from(record.difficulty)))
        .collect();
    played.sort_by_key(|difficulty| diff_order_weight(*difficulty));
    played.dedup();
    Ok(played)
}

fn no_song(query: &str) -> Outgoing {
    reply::notice(format!(
        "没有找到匹配「{query}」的曲目，可以试试曲目 ID、别名或歌名"
    ))
}

fn no_plays(song: &Song) -> Outgoing {
    reply::notice(format!("《{}》还没有你的游玩成绩", song.title))
}

async fn missing_score(
    ctx: &Ctx<'_>,
    uid: i64,
    matched: &domain::SongMatch,
    reason: String,
) -> Outgoing {
    let song = &matched.best.song;
    let mut args = reply::args();
    reply::put(&mut args, "title", song.title.clone());
    reply::put(&mut args, "reason", reason);
    let played = played_difficulties(ctx, uid, &song.id)
        .await
        .unwrap_or_default();
    for difficulty in played {
        let constant = match song.chart(difficulty) {
            Some(chart) if chart.has_constant => one(chart.constant),
            Some(_) => String::from("未收录"),
            None => String::from("未收录"),
        };
        reply::row(
            &mut args,
            "rows",
            &[
                (
                    "label",
                    format!("{} 定数 {}", difficulty.as_str(), constant),
                ),
                (
                    "command",
                    format!(
                        "{} score {} {}",
                        root_of(ctx),
                        song.id,
                        difficulty.as_str()
                    ),
                ),
            ],
        );
    }
    reply::text("score/score.play.none", &args)
}
