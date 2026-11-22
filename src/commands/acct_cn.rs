use aurorix_plugin::{Outgoing, PluginError};

use crate::commands::{bound_account, buttons, local_time, reply, repo_of, Ctx};
use crate::repo::ArcaeaRepo;

pub async fn login(ctx: &Ctx<'_>) -> Result<Option<Outgoing>, PluginError> {
    let repo = match repo_of(ctx).await {
        Ok(repo) => repo,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    match repo.bind_gate().await {
        Ok(true) => {}
        Ok(false) => return Ok(Some(reply::notice("当前不开放国服新绑定，请联系管理员"))),
        Err(error) => return Ok(Some(reply::fail(error.to_string()))),
    }
    match repo.get_account(ctx.region, &ctx.inv.user).await {
        Ok(Some(account)) if account.uid().is_some() => {
            let mut args = reply::args();
            reply::put(&mut args, "name", account.name.clone());
            reply::put(&mut args, "fc", account.friend_code.clone());
            return Ok(Some(reply::board(
                reply::text("account/account.cn.bound", &args),
                buttons::rebind_board(ctx.region),
            )));
        }
        Ok(_) => {}
        Err(error) => return Ok(Some(reply::fail(error.to_string()))),
    }
    let device = match repo.take_device_id(&ctx.inv.user).await {
        Ok(device) => device,
        Err(error) => return Ok(Some(reply::fail(error.to_string()))),
    };
    let start = match ctx.backend.cn_login_start(device.as_str(), None).await {
        Ok(start) => start,
        Err(error) => return Ok(Some(reply::fail(error.user_message()))),
    };
    let mut args = reply::args();
    reply::put(&mut args, "url", start.url.clone());
    let expires = match start.expires_at {
        Some(at) => format!("{} 前", local_time(at)),
        None => "5 分钟".to_string(),
    };
    reply::put(&mut args, "expires", expires);
    reply::put(&mut args, "hint", "扫码后在手机上确认授权");
    spawn_login_wait(ctx, &start.ticket_id, start.interval());
    Ok(Some(reply::board(
        reply::text("account/account.cn.login.ticket", &args),
        buttons::login_board(ctx.region),
    )))
}

pub async fn relogin(ctx: &Ctx<'_>) -> Result<Option<Outgoing>, PluginError> {
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
    match ctx.backend.cn_renew(uid).await {
        Ok(renew) => {
            if renew.renewed {
                return Ok(Some(reply::notice("国服登录已续期")));
            }
            if renew.need_scan {
                return Ok(Some(reply::board(
                    reply::notice(format!(
                        "需要重新扫码，请发送 {} login",
                        crate::commands::root_of(ctx)
                    )),
                    buttons::relogin_board(ctx.region),
                )));
            }
            if renew.cooldown_seconds > 0 {
                return Ok(Some(reply::notice(format!(
                    "续期太频繁，请 {} 秒后再试",
                    renew.cooldown_seconds
                ))));
            }
            let reason = if renew.reason.is_empty() {
                "续期没有成功，稍后再试".to_string()
            } else {
                renew.reason.clone()
            };
            Ok(Some(reply::notice(reason)))
        }
        Err(error) => Ok(Some(reply::fail(error.user_message()))),
    }
}

pub async fn update(ctx: &Ctx<'_>) -> Result<Option<Outgoing>, PluginError> {
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
    match ctx.backend.cn_refresh(uid).await {
        Ok(refresh) => {
            if refresh.need_relogin {
                return Ok(Some(reply::notice(format!(
                    "国服登录已过期，请发送 {} relogin",
                    crate::commands::root_of(ctx)
                ))));
            }
            if refresh.cooldown_seconds > 0 {
                return Ok(Some(reply::notice(format!(
                    "同步太频繁，请 {} 秒后再试",
                    refresh.cooldown_seconds
                ))));
            }
            if refresh.score_count > 0 {
                return Ok(Some(reply::notice(format!(
                    "云存档同步完成：{} 条有更新，库里已收录 {} 条",
                    refresh.score_count, refresh.known_count
                ))));
            }
            if refresh.known_count == 0 {
                return Ok(Some(reply::notice(format!(
                    "云存档里没有找到任何成绩。\n{}",
                    super::analysis::no_scores_notice(ctx.region)
                ))));
            }
            Ok(Some(reply::notice(format!(
                "云存档没有新成绩（库里已收录 {} 条）。\n{}",
                refresh.known_count,
                super::analysis::no_scores_notice(ctx.region)
            ))))
        }
        Err(error) => Ok(Some(reply::fail(error.user_message()))),
    }
}

pub async fn nbind(ctx: &Ctx<'_>) -> Result<Option<Outgoing>, PluginError> {
    let repo = match repo_of(ctx).await {
        Ok(repo) => repo,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    match repo.set_bind_gate(false).await {
        Ok(()) => Ok(Some(reply::notice(
            "已关闭国服新绑定：之后的 /ca login 会被挡下，已绑定的用户不受影响",
        ))),
        Err(error) => Ok(Some(reply::fail(error.to_string()))),
    }
}

pub async fn ybind(ctx: &Ctx<'_>) -> Result<Option<Outgoing>, PluginError> {
    let repo = match repo_of(ctx).await {
        Ok(repo) => repo,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    match repo.set_bind_gate(true).await {
        Ok(()) => Ok(Some(reply::notice(
            "已开放国服新绑定：现在可以用 /ca login 绑定国服账号",
        ))),
        Err(error) => Ok(Some(reply::fail(error.to_string()))),
    }
}

fn spawn_login_wait(ctx: &Ctx<'_>, ticket_id: &str, interval: std::time::Duration) {
    let Some(db) = ctx.inv.plugin_db() else {
        return;
    };
    let backend = ctx.backend.clone();
    let user = ctx.inv.user.clone();
    let sender = ctx.inv.opt_sender.clone();
    let ticket = ticket_id.to_string();
    tokio::spawn(async move {
        wait_login(backend, db, user, sender, ticket, interval).await;
    });
}

async fn wait_login(
    backend: crate::backend::ArcBackendNext,
    db: std::sync::Arc<aurorix_plugin::PluginDb>,
    user: String,
    sender: Option<aurorix_plugin::InvocationSender>,
    ticket_id: String,
    interval: std::time::Duration,
) {
    let Ok(wait) = backend
        .cn_login_wait(&ticket_id, interval, std::time::Duration::from_secs(240))
        .await
    else {
        return;
    };
    if !wait.poll.status.ready() {
        return;
    }
    let Some(player) = wait.poll.player else {
        return;
    };
    let repo = ArcaeaRepo::from_db(db);
    let mut account = crate::model::Account::new(crate::model::Region::Cn, user, player.arcaea_uid);
    account.friend_code = player.friend_code;
    account.name = player.name;
    if repo.put_account(&account).await.is_err() {
        return;
    }
    let (records, hint) = match backend.cn_refresh(player.arcaea_uid).await {
        Ok(refresh) if refresh.need_relogin => (
            refresh.known_count.max(0).to_string(),
            String::from("登录凭证要重新授权，稍后发 /ca relogin 再试"),
        ),
        Ok(refresh) if refresh.score_count > 0 => (
            refresh.known_count.max(0).to_string(),
            format!("本次从云存档更新 {} 条成绩", refresh.score_count),
        ),
        Ok(refresh) if refresh.known_count == 0 => (
            String::from("0"),
            String::from("云存档里没有找到任何成绩，稍后发 /ca update 再试"),
        ),
        Ok(refresh) => (
            refresh.known_count.to_string(),
            format!("云存档没有新成绩（库里已收录 {} 条）", refresh.known_count),
        ),
        Err(_) => (
            String::from("0"),
            String::from("这次没拉到云存档，稍后发 /ca update 再试"),
        ),
    };
    let Some(sender) = sender else {
        return;
    };
    let mut args = reply::args();
    reply::put(&mut args, "name", account.name);
    reply::put(&mut args, "fc", account.friend_code);
    reply::put(&mut args, "records", records);
    reply::put(
        &mut args,
        "sync_tip",
        super::analysis::no_scores_notice(crate::model::Region::Cn),
    );
    reply::put(&mut args, "hint", hint);
    let _ = sender
        .send(reply::board(
            reply::text("account/account.cn.login.ok", &args),
            super::buttons::best_board(crate::model::Region::Cn),
        ))
        .await;
}
