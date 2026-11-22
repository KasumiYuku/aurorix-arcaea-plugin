use std::collections::{BTreeMap, HashMap};

use aurorix_plugin::{Outgoing, Part, PluginError};
use chrono::Utc;

use crate::backend::{
    ProfileBoard, ProfileChange, ProfilePoint, ProfileQuery, QueryMeta, ScoreRecord, ScoreSource,
    StatsBand, StatsQuery,
};
use crate::cards::gallery::{
    BannerCell, BannerInput, CharCell, CharInput, MeBand, MeBin, MeBoard, MeBoardRow, MeChange,
    MeCurve, MeCurvePoint, MeScore, MeScatter,
};
use crate::commands::{
    ack_outgoing, bound_account, buttons, local_time, prefs_of, reply, repo_of, root_of, Ctx,
};
use crate::domain::ptt;
use crate::model::{Account, Prefs, Region};

enum Which {
    Char,
    Banner,
}

pub async fn bind(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    let code: String = arg
        .chars()
        .filter(|value| !value.is_whitespace())
        .collect();
    if code.len() != 9 || !code.chars().all(|value| value.is_ascii_digit()) {
        return Ok(Some(reply::board(
            reply::notice(format!(
                "好友码是 9 位数字，例如 {} bind 431343680",
                root_of(ctx)
            )),
            buttons::bind_board(ctx.region),
        )));
    }
    let repo = match repo_of(ctx).await {
        Ok(repo) => repo,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    match repo.get_account(ctx.region, &ctx.inv.user).await {
        Ok(Some(account)) if account.uid().is_some() => {
            return Ok(Some(bind_already(ctx.region, &account)))
        }
        Ok(_) => {}
        Err(error) => return Ok(Some(reply::fail(error.to_string()))),
    }
    if let Some(sender) = ctx.inv.opt_sender.clone() {
        let _ = sender
            .send(ack_outgoing("正在绑定国际服账号，结果出来后我会 @ 你"))
            .await;
    }
    match ctx.backend.bind(&code).await {
        Ok(binding) => {
            let mut account = Account::new(ctx.region, ctx.inv.user.clone(), binding.arcaea_uid);
            account.friend_code = binding.friend_code.clone();
            account.name = binding.name.clone();
            match repo.put_account(&account).await {
                Ok(()) => Ok(Some(bind_ok(ctx.region, &account))),
                Err(error) if error.is_uid_taken() => {
                    Ok(Some(bind_error("该账号已被其他用户绑定")))
                }
                Err(error) => Ok(Some(bind_error(error.to_string()))),
            }
        }
        Err(error) => Ok(Some(bind_error(error.user_message()))),
    }
}

pub async fn unbind(ctx: &Ctx<'_>) -> Result<Option<Outgoing>, PluginError> {
    let repo = match repo_of(ctx).await {
        Ok(repo) => repo,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    let account = match bound_account(ctx, &repo).await {
        Ok(account) => account,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    let expires = Utc::now() + chrono::Duration::minutes(10);
    if let Err(error) = repo.put_ticket(ctx.region, &ctx.inv.user, expires).await {
        return Ok(Some(reply::fail(error.to_string())));
    }
    let mut args = reply::args();
    reply::put(&mut args, "name", account.name.clone());
    reply::put(&mut args, "fc", account.friend_code.clone());
    reply::put(&mut args, "minutes", "10");
    reply::put(&mut args, "confirm_cmd", format!("{} confirm", root_of(ctx)));
    reply::put(&mut args, "cancel_cmd", format!("{} cancel", root_of(ctx)));
    let ticket = Utc::now().timestamp().to_string();
    Ok(Some(reply::board(
        reply::text("account/account.unbind.confirm", &args),
        buttons::unbind_confirm_board(ctx.region, &ticket),
    )))
}

pub async fn confirm(ctx: &Ctx<'_>) -> Result<Option<Outgoing>, PluginError> {
    let repo = match repo_of(ctx).await {
        Ok(repo) => repo,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    let fresh = match repo.take_ticket(ctx.region, &ctx.inv.user).await {
        Ok(fresh) => fresh,
        Err(error) => return Ok(Some(reply::fail(error.to_string()))),
    };
    if !fresh {
        return Ok(Some(reply::notice("没有等你确认的解绑（10 分钟内有效）")));
    }
    match repo.delete_account(ctx.region, &ctx.inv.user).await {
        Ok(_) => Ok(Some(reply::text(
            "account/account.unbind.ok",
            &reply::args(),
        ))),
        Err(error) => Ok(Some(reply::fail(error.to_string()))),
    }
}

pub async fn cancel(ctx: &Ctx<'_>) -> Result<Option<Outgoing>, PluginError> {
    let repo = match repo_of(ctx).await {
        Ok(repo) => repo,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    match repo.clear_ticket(ctx.region, &ctx.inv.user).await {
        Ok(true) => Ok(Some(reply::notice("已取消解绑，账号仍绑定着"))),
        Ok(false) => Ok(Some(reply::notice("没有等你确认的解绑（10 分钟内有效）"))),
        Err(error) => Ok(Some(reply::fail(error.to_string()))),
    }
}

pub async fn me(ctx: &Ctx<'_>) -> Result<Option<Outgoing>, PluginError> {
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
    let player = match ctx.backend.player(ctx.region, uid).await {
        Ok(Some(player)) => player,
        Ok(None) => return Ok(Some(reply::notice("后端没有这个账号的资料"))),
        Err(error) => return Ok(Some(reply::fail(error.user_message()))),
    };
    let prefs = prefs_of(ctx, &repo).await;
    let identity = player.identity.named(account.name.as_str());
    let friend_code = if player.friend_code.is_empty() {
        account.friend_code.clone()
    } else {
        player.friend_code.clone()
    };
    let mut badge = crate::commands::bridge::badge_of(&identity, &prefs);
    badge.subtitle = format!(
        "{} · 好友码 {}",
        if ctx.region.is_cn() { "国服" } else { "国际服" },
        friend_code
    );
    let mut missing: Vec<&'static str> = Vec::new();
    let profile = match ctx
        .backend
        .profile(ctx.region, uid, ProfileQuery::new())
        .await
    {
        Ok(profile) => profile,
        Err(_) => {
            missing.push("成长曲线与全站排行");
            Default::default()
        }
    };
    let stats = match ctx
        .backend
        .stats(ctx.region, uid, StatsQuery::new("0", "all"))
        .await
    {
        Ok(stats) => stats,
        Err(_) => {
            missing.push("完成度");
            Default::default()
        }
    };
    let scores = match ctx
        .backend
        .all_scores(ctx.region, uid, ScoreSource::Local)
        .await
    {
        Ok(scores) => scores,
        Err(_) => {
            missing.push("单曲 PTT 分布");
            Default::default()
        }
    };
    let best = local_best(&scores.records);
    let ratings: Vec<f64> = scores
        .records
        .iter()
        .map(|record| record.rating)
        .filter(|rating| *rating > 0.0)
        .collect();
    let best_average = ptt::best_average(&ratings, 10)
        .filter(|value| *value > 0.0)
        .or(identity.ptt())
        .unwrap_or(0.0);
    let titles: HashMap<&str, &str> = scores
        .records
        .iter()
        .map(|record| (record.song_id.as_str(), record.title.as_str()))
        .collect();
    let updated = data_time(&scores.meta, &profile.curve);
    let mut input = crate::commands::bridge::me_input(
        badge,
        crate::commands::bridge::theme_of(&prefs),
        crate::commands::bridge::MeArgs {
            user_id: &ctx.inv.user,
            region: ctx.region,
            joined: &local_time(account.bound_at),
            updated: updated.as_str(),
            best_average,
            best,
            rating_frame: crate::commands::bridge::frame_of_identity(&identity),
            rating_accent: identity
                .accent()
                .or_else(|| crate::commands::bridge::frame_of_identity(&identity)),
        },
    );
    if let Some(curve) = me_curve(&profile.curve) {
        input = input.with_curve(curve);
    }
    let bands = me_bands(&stats.bands);
    if !bands.is_empty() {
        input = input.with_bands(bands);
    }
    input = input.with_pm_small_p(stats.pm_small_p);
    if let Some(scatter) = me_scatter(&scores.records) {
        input = input.with_scatter(scatter);
    }
    if let Some(board) = profile.board.as_ref().and_then(|board| me_board(board, &prefs)) {
        input = input.with_board(board);
    }
    let changes = me_changes(&profile.changes, &titles);
    if !changes.is_empty() {
        input = input.with_changes(changes);
    }
    let mut caption = reply::text("account/account.me.caption", &reply::args());
    if !missing.is_empty() {
        let note = format!("\n\n> 这次没取到：{}", missing.join("、"));
        if let Some(Part::Markdown(body)) = caption.parts.first_mut() {
            body.push_str(&note);
        }
    }
    Ok(Some(reply::board(
        reply::card(caption, crate::cards::gallery::card(input)),
        buttons::me_board(ctx.region),
    )))
}

fn local_best(records: &[ScoreRecord]) -> Vec<MeScore> {
    let mut rows: Vec<&ScoreRecord> = records
        .iter()
        .filter(|record| record.rating > 0.0)
        .collect();
    rows.sort_by(|left, right| right.rating.total_cmp(&left.rating));
    rows.into_iter()
        .take(10)
        .map(crate::commands::bridge::me_score_of)
        .collect()
}

fn data_time(scores: &QueryMeta, curve: &[ProfilePoint]) -> String {
    if let Some(at) = scores.cached_at {
        return local_time(at);
    }
    curve
        .iter()
        .rev()
        .find_map(|point| crate::backend::parse_time(point.day.as_str()))
        .map(local_time)
        .unwrap_or_else(|| String::from("未记录"))
}

fn me_curve(points: &[ProfilePoint]) -> Option<MeCurve> {
    let mut samples: Vec<(i64, f64, i64)> = points
        .iter()
        .filter_map(|point| {
            let at = crate::backend::parse_time(point.day.as_str())?;
            let rating = point.rating.filter(|value| *value > 0.0)?;
            Some((at.timestamp(), rating, point.records))
        })
        .collect();
    if samples.is_empty() {
        return None;
    }
    samples.sort_by_key(|(at, _, _)| *at);
    let last_at = samples[samples.len() - 1].0;
    let last_rating = samples[samples.len() - 1].1;
    let peak = samples
        .iter()
        .map(|(_, rating, _)| *rating)
        .fold(f64::MIN, f64::max);
    let peak_at = samples
        .iter()
        .find(|(_, rating, _)| *rating == peak)
        .map(|(at, _, _)| *at);
    let delta = |days: i64| -> Option<f64> {
        let cutoff = last_at - days * 86_400;
        let base = samples
            .iter()
            .rev()
            .find(|(at, _, _)| *at <= cutoff)
            .map(|(_, rating, _)| *rating)?;
        Some(last_rating - base)
    };
    let cutoff = last_at - 30 * 86_400;
    let records30: i64 = samples
        .iter()
        .filter(|(at, _, _)| *at >= cutoff)
        .map(|(_, _, records)| *records)
        .sum();
    Some(MeCurve {
        points: samples
            .iter()
            .map(|(at, rating, _)| MeCurvePoint {
                at: *at,
                ptt: *rating,
            })
            .collect(),
        peak,
        peak_at,
        recent7: delta(7),
        recent30: delta(30),
        records30: Some(count_of(records30)),
        first: samples.first().map(|(at, _, _)| *at),
        last: Some(last_at),
    })
}

fn me_bands(bands: &[StatsBand]) -> Vec<MeBand> {
    let mut rows: Vec<(u8, MeBand)> = bands
        .iter()
        .filter_map(|band| {
            let difficulty = band.difficulty_kind()?;
            let mut row = MeBand::new(difficulty);
            row.played = count_of(band.played);
            row.total = count_of(band.total);
            row.cleared = count_of(band.cleared);
            row.full_combo = count_of(band.full_combo);
            row.pm = count_of(band.pm);
            row.ex_plus = count_of(band.ex_plus);
            row.ex = count_of(band.ex);
            row.loss_played = band.loss_played.max(0.0);
            row.loss_total = band.loss_total.max(0.0);
            Some((crate::model::diff_display_weight(difficulty), row))
        })
        .collect();
    rows.sort_by_key(|(weight, _)| *weight);
    rows.into_iter().map(|(_, band)| band).collect()
}

fn me_scatter(records: &[ScoreRecord]) -> Option<MeScatter> {
    let mut bins: BTreeMap<i64, (f64, f64, f64, i32)> = BTreeMap::new();
    for record in records {
        if record.constant <= 0.0 {
            continue;
        }
        let rating = if record.rating > 0.0 {
            record.rating
        } else {
            ptt::single_rating(record.constant, i64::from(record.score))
        };
        if rating <= 0.0 {
            continue;
        }
        let key = (record.constant * 10.0).round() as i64;
        let entry = bins.entry(key).or_insert((rating, rating, 0.0, 0));
        entry.0 = entry.0.min(rating);
        entry.1 = entry.1.max(rating);
        entry.2 += rating;
        entry.3 += 1;
    }
    if bins.is_empty() {
        return None;
    }
    let mut sum = 0.0;
    let mut hits = 0;
    let bins: Vec<MeBin> = bins
        .into_iter()
        .map(|(key, (low, high, total, count))| {
            sum += total;
            hits += count;
            MeBin {
                constant: key as f64 / 10.0,
                low,
                high,
                avg: total / f64::from(count),
                count,
            }
        })
        .collect();
    let max_ptt = bins.iter().map(|bin| bin.high).fold(0.0_f64, f64::max);
    Some(MeScatter {
        bins,
        max_ptt,
        avg_ptt: (hits > 0).then(|| sum / f64::from(hits)),
        ability: None,
    })
}

fn me_board(board: &ProfileBoard, prefs: &crate::model::Prefs) -> Option<MeBoard> {
    if board.rows.is_empty() {
        return None;
    }
    Some(MeBoard {
        total: count_of(board.total),
        rows: board
            .rows
            .iter()
            .map(|row| MeBoardRow {
                rank: count_of(row.rank),
                name: row.identity.name.clone(),
                avatar: crate::commands::bridge::badge_of(&row.identity, prefs).avatar,
                ptt: row.identity.ptt(),
                rating_frame: row.identity.frame(),
                rating_accent: row.identity.accent(),
                me: row.is_me,
            })
            .collect(),
    })
}

fn me_changes(changes: &[ProfileChange], titles: &HashMap<&str, &str>) -> Vec<MeChange> {
    changes
        .iter()
        .take(6)
        .map(|change| MeChange {
            title: titles
                .get(change.song_id.as_str())
                .map(|title| title.to_string())
                .unwrap_or_default(),
            diff: change
                .difficulty_kind()
                .map(|difficulty| difficulty.as_str().to_string())
                .unwrap_or_default(),
            old_score: change.has_old.then_some(i64::from(change.old_score)),
            new_score: i64::from(change.new_score),
            at: change.at.map(|at| at.timestamp()),
            jacket: change
                .difficulty_kind()
                .and_then(|difficulty| {
                    crate::commands::bridge::jacket_of_chart(&change.song_id, difficulty)
                })
                .or_else(|| {
                    (!change.song_id.trim().is_empty())
                        .then(|| crate::commands::bridge::jacket_key(&change.song_id))
                }),
        })
        .collect()
}

fn count_of(value: i64) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX).max(0)
}

pub async fn char(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    set_pref(ctx, arg, Which::Char).await
}

pub async fn banner(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    set_pref(ctx, arg, Which::Banner).await
}

async fn set_pref(
    ctx: &Ctx<'_>,
    arg: &str,
    which: Which,
) -> Result<Option<Outgoing>, PluginError> {
    let (command, directory, label) = match which {
        Which::Char => ("char", "characters", "角色立绘"),
        Which::Banner => ("banner", "banners", "资料卡横幅"),
    };
    let names = asset_names(directory);
    let repo = repo_of(ctx).await.ok();
    let prefs = match repo.as_ref() {
        Some(repo) => prefs_of(ctx, repo).await,
        None => Prefs::default(),
    };
    let current = match which {
        Which::Char => prefs.char.clone(),
        Which::Banner => prefs.banner.clone(),
    };
    if arg.trim().is_empty() {
        return Ok(Some(gallery_outgoing(
            ctx.region, &names, &current, label, command, which, &prefs,
        )));
    }
    let Ok(index) = arg.trim().parse::<usize>() else {
        return Ok(Some(reply::notice(format!(
            "用法：{} {command} <编号>（0 跟随游戏；不带编号看一览图）",
            root_of(ctx)
        ))));
    };
    if index > names.len() {
        return Ok(Some(reply::notice(format!(
            "没有第 {index} 个，{label}一共 {} 个（1~{}）",
            names.len(),
            names.len()
        ))));
    }
    let Some(repo) = repo else {
        return Ok(Some(reply::fail("插件数据库还没准备好")));
    };
    let account = match bound_account(ctx, &repo).await {
        Ok(account) => account,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    let Some(uid) = account.uid() else {
        return Ok(Some(reply::unbound(ctx.region)));
    };
    let value = if index == 0 {
        String::new()
    } else {
        names[index - 1].clone()
    };
    let updated = match which {
        Which::Char => {
            ctx.backend
                .set_prefs(ctx.region, uid, Some(value.as_str()), None)
                .await
        }
        Which::Banner => {
            ctx.backend
                .set_prefs(ctx.region, uid, None, Some(value.as_str()))
                .await
        }
    };
    if let Err(error) = updated {
        return Ok(Some(reply::fail(error.user_message())));
    }
    let mut next = match repo.prefs(ctx.region, &ctx.inv.user).await {
        Ok(prefs) => prefs,
        Err(error) => return Ok(Some(reply::fail(error.to_string()))),
    };
    match which {
        Which::Char => next.char = value.clone(),
        Which::Banner => next.banner = value.clone(),
    }
    if let Err(error) = repo.set_prefs(ctx.region, &ctx.inv.user, &next).await {
        return Ok(Some(reply::fail(error.to_string())));
    }
    let message = if index == 0 {
        format!("已改回跟随游戏里的{label}")
    } else {
        format!("已把{label}换成第 {index} 个 · {}", stem_of(&value))
    };
    Ok(Some(reply::notice(message)))
}

fn gallery_outgoing(
    region: Region,
    names: &[String],
    current: &str,
    label: &str,
    command: &str,
    which: Which,
    prefs: &Prefs,
) -> Outgoing {
    if names.is_empty() {
        return reply::notice(format!("素材目录里还没有{label}，先检查一下 assets"));
    }
    let theme = crate::commands::bridge::theme_of(prefs);
    let mut args = reply::args();
    reply::put(&mut args, "title", format!("{label}一览图"));
    reply::put(&mut args, "text", current_text(names, current, label));
    let caption = reply::text("account/account.pref.caption", &args);
    let guide = format!("共 {} 个 · {command} <编号> 切换，0 跟随游戏", names.len());
    let card = match which {
        Which::Char => {
            let cells = names
                .iter()
                .map(|name| CharCell::new(stem_of(name)))
                .collect();
            let input = CharInput::new()
                .with_title(format!("{label}一览"))
                .with_theme(theme)
                .with_note(guide)
                .with_cells(cells);
            crate::cards::gallery::char_card(input)
        }
        Which::Banner => {
            let cells = names
                .iter()
                .map(|name| BannerCell::new(format!("banners/{name}")))
                .collect();
            let input = BannerInput::new()
                .with_title(format!("{label}一览"))
                .with_theme(theme)
                .with_note(guide)
                .with_cells(cells);
            crate::cards::gallery::banner_card(input)
        }
    };
    reply::board(reply::card(caption, card), buttons::pref_board(region, command))
}

fn current_text(names: &[String], current: &str, label: &str) -> String {
    if current.is_empty() {
        return format!("当前：跟随游戏 · 共 {} 个", names.len());
    }
    match names.iter().position(|name| name == current) {
        Some(index) => format!(
            "当前：第 {} 个（{}） · 共 {} 个",
            index + 1,
            stem_of(current),
            names.len()
        ),
        None => format!(
            "当前：跟随游戏（你选的{label}已不在素材里） · 共 {} 个",
            names.len()
        ),
    }
}

fn stem_of(name: &str) -> String {
    match name.rsplit_once('.') {
        Some((stem, _)) => stem.to_string(),
        None => name.to_string(),
    }
}

pub fn asset_names(directory: &str) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(crate::assets_root().join(directory))
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| entry.path().is_file())
                .filter(|entry| {
                    entry
                        .path()
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
                })
                .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

fn bind_ok(region: Region, account: &Account) -> Outgoing {
    let mut args = reply::args();
    reply::put(&mut args, "name", account.name.clone());
    reply::put(&mut args, "fc", account.friend_code.clone());
    reply::board(
        reply::text("account/account.bind.ok", &args),
        buttons::bind_ok_board(region),
    )
}

fn bind_already(region: Region, account: &Account) -> Outgoing {
    let mut args = reply::args();
    reply::put(&mut args, "name", account.name.clone());
    reply::put(&mut args, "fc", account.friend_code.clone());
    reply::board(
        reply::text("account/account.bind.already", &args),
        buttons::unbind_board(region),
    )
}

fn bind_error(reason: impl Into<String>) -> Outgoing {
    let mut args = reply::args();
    reply::put(&mut args, "reason", reason);
    reply::text("account/account.bind.error", &args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_049() {
        let chars = asset_names("characters");
        assert!(!chars.is_empty(), "characters 目录必须扫得出素材");
        for name in &chars {
            assert!(
                name.ends_with(".png"),
                "后端 prefs 存的是完整文件名，{name} 少了扩展名"
            );
            assert!(!name.contains('/'), "清单里不能出现目录段：{name}");
        }
        assert!(
            chars.windows(2).all(|pair| pair[0] <= pair[1]),
            "编号靠这份顺序，必须排好序"
        );
        let banners = asset_names("banners");
        assert!(!banners.is_empty(), "banners 目录必须扫得出素材");
        assert!(
            banners.iter().all(|name| name.ends_with(".png")),
            "横幅清单同样是完整文件名"
        );
    }

    #[test]
    fn case_050() {
        let names = asset_names("characters");
        let picked = names.first().expect("至少一个立绘").clone();
        assert!(picked.ends_with(".png"), "落库的是完整文件名");
        assert!(!stem_of(&picked).contains('.'), "去后缀之后不该还有扩展名");
        assert_eq!(
            asset_names("characters")[0],
            picked,
            "编号映射必须跨调用稳定"
        );
    }

    #[test]
    fn case_051() {
        let names = vec!["a.png".to_string(), "b.png".to_string()];
        assert_eq!(current_text(&names, "", "角色立绘"), "当前：跟随游戏 · 共 2 个");
        assert_eq!(
            current_text(&names, "b.png", "角色立绘"),
            "当前：第 2 个（b） · 共 2 个"
        );
        assert!(
            current_text(&names, "gone.png", "角色立绘").contains("已不在素材里"),
            "素材没了要说清楚，而不是显示第 0 个"
        );
    }
}
