use std::sync::Arc;
use std::sync::OnceLock;

use aurorix_buttons::{style, Builder, ButtonStyle, Keyboard};
use aurorix_plugin::{
    CallbackBook, CallbackFuture, CallbackHandler, CallbackRequest, Invocation, Outgoing, PluginDb,
};

use crate::backend::ArcBackendNext;
use crate::commands::{acct_intl, boards, help, hints, reply, score, Ctx};
use crate::model::Region;

pub const QUIZ_ROOT: &str = "/a";

pub fn scope(region: Region) -> &'static str {
    match region.is_cn() {
        true => "cn.arcaea",
        false => "arcaea",
    }
}

pub fn root(region: Region) -> &'static str {
    match region.is_cn() {
        true => "/ca",
        false => "/a",
    }
}

fn opts() -> Vec<aurorix_buttons::Opt> {
    vec![style(ButtonStyle::GraySolid)]
}

fn board(region: Region) -> Builder {
    Builder::new().scope(scope(region))
}

fn finish(builder: Builder) -> Option<Keyboard> {
    let keyboard = builder.build()?;
    match keyboard.validate() {
        Ok(()) => Some(keyboard),
        Err(_) => None,
    }
}

fn quiz_id(region: Region, suffix: &str) -> String {
    format!("{}.{suffix}", scope(region))
}

fn line(region: Region, rest: &str) -> String {
    format!("{} {rest}", root(region))
}

fn with_arg(base: String, arg: &str) -> String {
    let arg = arg.trim();
    match arg.is_empty() {
        true => base,
        false => format!("{base} {arg}"),
    }
}

fn update_button(builder: Builder, region: Region) -> Builder {
    match region.is_cn() {
        true => builder.command(
            "account.update",
            "更新存档",
            &line(region, "update"),
            opts(),
        ),
        false => builder,
    }
}

pub fn help_board(region: Region) -> Option<Keyboard> {
    let builder = match region.is_cn() {
        true => board(region).command("account.login", "登录账号", &line(region, "login"), opts()),
        false => board(region).command("account.bind", "绑定账号", &line(region, "bind "), opts()),
    };
    finish(builder.callback("score.best50", "Best 50", "50", opts()))
}

pub fn bind_board(region: Region) -> Option<Keyboard> {
    match region.is_cn() {
        true => finish(board(region).command(
            "account.login",
            "登录账号",
            &line(region, "login"),
            opts(),
        )),
        false => finish(board(region).command(
            "account.bind",
            "绑定账号",
            &line(region, "bind "),
            opts(),
        )),
    }
}

pub fn login_board(region: Region) -> Option<Keyboard> {
    finish(board(region).callback("account.login.cancel", "取消登录", "cancel", opts()))
}

pub fn relogin_board(region: Region) -> Option<Keyboard> {
    finish(board(region).command(
        "account.relogin",
        "重新登录",
        &line(region, "relogin"),
        opts(),
    ))
}

pub fn rebind_board(region: Region) -> Option<Keyboard> {
    match region.is_cn() {
        true => finish(board(region).command(
            "account.login",
            "重新登录",
            &line(region, "login"),
            opts(),
        )),
        false => finish(board(region).command(
            "account.bind",
            "重新绑定",
            &line(region, "bind "),
            opts(),
        )),
    }
}

pub fn unbind_board(region: Region) -> Option<Keyboard> {
    finish(board(region).callback("account.unbind", "解除绑定", "start", opts()))
}

pub fn unbind_confirm_board(region: Region, ticket: &str) -> Option<Keyboard> {
    let data = match ticket.trim().is_empty() {
        true => "confirm",
        false => ticket.trim(),
    };
    finish(board(region).callback("account.unbind.confirm", "确认解绑", data, opts()))
}

pub fn me_board(region: Region) -> Option<Keyboard> {
    finish(
        board(region)
            .callback("score.best50", "Best 50", "50", opts())
            .callback("account.unbind", "解除绑定", "start", opts())
            .callback("common.menu", "Arc菜单", "menu", opts()),
    )
}

pub fn bind_ok_board(region: Region) -> Option<Keyboard> {
    finish(
        board(region)
            .callback("score.best50", "Best 50", "50", opts())
            .callback("score.recent", "最近游玩", "recent", opts())
            .command(
                "score.single",
                "单曲成绩",
                &line(region, "score "),
                opts(),
            )
            .callback("common.menu", "Arc菜单", "menu", opts()),
    )
}

pub fn best_board(region: Region) -> Option<Keyboard> {
    let builder = update_button(board(region), region);
    finish(
        builder
            .callback("score.best50", "Best 50", "50", opts())
            .callback("score.recent", "最近游玩", "recent", opts())
            .callback("common.menu", "Arc菜单", "menu", opts()),
    )
}

pub fn rank_board(region: Region) -> Option<Keyboard> {
    match region.is_cn() {
        true => finish(
            update_button(board(region), region)
                .callback("common.menu", "Arc菜单", "menu", opts()),
        ),
        false => None,
    }
}

pub fn rank_board_fallback(region: Region) -> Option<Keyboard> {
    finish(
        update_button(board(region), region)
            .callback("common.menu", "Arc菜单", "menu", opts()),
    )
}

fn slot(song_id: &str, difficulty: crate::Difficulty) -> String {
    let song_id = song_id.trim();
    match song_id.is_empty() {
        true => String::new(),
        false => format!("{song_id} {}", difficulty.as_str()),
    }
}

pub fn recent_board(
    region: Region,
    song_id: &str,
    difficulty: crate::Difficulty,
) -> Option<Keyboard> {
    let reference = slot(song_id, difficulty);
    finish(
        board(region)
            .callback("score.recent", "最近游玩", "recent", opts())
            .callback("score.best50", "Best 50", "50", opts())
            .command(
                "song.detail",
                "该曲图鉴",
                &with_arg(line(region, "song"), &reference),
                opts(),
            )
            .callback("common.menu", "Arc菜单", "menu", opts()),
    )
}

pub fn score_board(
    region: Region,
    song_id: &str,
    difficulty: crate::Difficulty,
) -> Option<Keyboard> {
    let play = slot(song_id, difficulty);
    finish(
        board(region)
            .command(
                "score.single",
                "该曲成绩",
                &with_arg(line(region, "score"), &play),
                opts(),
            )
            .command(
                "song.detail",
                "该曲图鉴",
                &with_arg(line(region, "song"), song_id),
                opts(),
            )
            .command(
                "song.play",
                "播放音乐",
                &with_arg(line(region, "play"), song_id),
                opts(),
            )
            .callback("common.menu", "Arc菜单", "menu", opts()),
    )
}

pub fn chart_board(
    region: Region,
    reference: &str,
    _difficulty: crate::Difficulty,
) -> Option<Keyboard> {
    finish(
        board(region)
            .command(
                "song.detail",
                "该曲图鉴",
                &with_arg(line(region, "song"), reference),
                opts(),
            )
            .callback("common.menu", "Arc菜单", "menu", opts()),
    )
}

pub fn search_board(region: Region, cond: &str) -> Option<Keyboard> {
    finish(
        board(region)
            .command(
                "song.search",
                "检索歌曲",
                &with_arg(line(region, "search"), cond),
                opts(),
            )
            .command(
                "song.rand.again",
                "随机曲目",
                &with_arg(line(region, "rand"), cond),
                opts(),
            )
            .callback("common.menu", "Arc菜单", "menu", opts()),
    )
}

fn pref_labels(kind: &str) -> (&'static str, &'static str) {
    match kind {
        "char" => ("换立绘", "看横幅"),
        "banner" => ("换横幅", "看立绘"),
        _ => ("点击换", "换"),
    }
}

fn pref_other(kind: &str) -> String {
    match kind {
        "char" => "banner".to_string(),
        "banner" => "char".to_string(),
        other => other.to_string(),
    }
}

pub fn pref_board(region: Region, kind: &str) -> Option<Keyboard> {
    let kind = kind.trim();
    let (pick, other) = pref_labels(kind);
    let next = pref_other(kind);
    finish(
        board(region)
            .command(
                "account.pref.pick",
                pick,
                &line(region, &format!("{kind} ")),
                opts(),
            )
            .command(
                "account.pref.other",
                other,
                &line(region, &format!("{next} ")),
                opts(),
            )
            .callback("common.menu", "Arc菜单", "menu", opts()),
    )
}

pub fn rand_board(region: Region, cond: &str, title: &str) -> Option<Keyboard> {
    finish(
        board(region)
            .command(
                "song.rand.again",
                "随机曲目",
                &with_arg(line(region, "rand"), cond),
                opts(),
            )
            .command(
                "song.detail",
                "该曲图鉴",
                &with_arg(line(region, "song"), title),
                opts(),
            ),
    )
}

pub fn detail_board(
    region: Region,
    song_id: &str,
    difficulty: crate::Difficulty,
) -> Option<Keyboard> {
    let play = slot(song_id, difficulty);
    finish(
        board(region)
            .command(
                "score.single",
                "该曲成绩",
                &with_arg(line(region, "score"), &play),
                opts(),
            )
            .command(
                "song.detail",
                "该曲图鉴",
                &with_arg(line(region, "song"), song_id),
                opts(),
            )
            .command(
                "song.play",
                "播放音乐",
                &with_arg(line(region, "play"), song_id),
                opts(),
            ),
    )
}

pub fn quiz_board(region: Region) -> Option<Keyboard> {
    finish(
        board(region)
            .command(
                &quiz_id(region, "try"),
                "再答一次",
                &format!("{QUIZ_ROOT} guess "),
                opts(),
            )
            .command(
                &quiz_id(region, "hint"),
                "提示",
                &format!("{QUIZ_ROOT} guess 提示"),
                opts(),
            )
            .command(
                &quiz_id(region, "stop"),
                "收局",
                &format!("{QUIZ_ROOT} guess stop"),
                opts(),
            ),
    )
}

pub fn quiz_again_board(region: Region, title: &str) -> Option<Keyboard> {
    let title = title.trim();
    finish(
        board(region)
            .command(
                &quiz_id(region, "again"),
                "再来一局",
                &format!("{QUIZ_ROOT} guess"),
                opts(),
            )
            .row()
            .command(
                &quiz_id(region, "over0"),
                "该曲图鉴",
                &format!("{QUIZ_ROOT} song {title}"),
                opts(),
            )
            .command(
                &quiz_id(region, "over1"),
                "该曲成绩",
                &format!("{QUIZ_ROOT} score {title}"),
                opts(),
            )
            .command(
                &quiz_id(region, "over2"),
                "播放音乐",
                &format!("{QUIZ_ROOT} play {title}"),
                opts(),
            ),
    )
}

pub fn switch_diff_button(region: Region) -> Option<aurorix_buttons::Button> {
    let keyboard = finish(board(region).command(
        "song.switch_diff",
        "切换难度",
        &line(region, "song "),
        opts(),
    ))?;
    keyboard.flatten().into_iter().next()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Best50,
    Recent,
    Menu,
    Unbind,
    UnbindConfirm,
    LoginCancel,
}

const CALLBACKS: [(&str, Action); 6] = [
    ("score.best50", Action::Best50),
    ("score.recent", Action::Recent),
    ("common.menu", Action::Menu),
    ("account.unbind", Action::Unbind),
    ("account.unbind.confirm", Action::UnbindConfirm),
    ("account.login.cancel", Action::LoginCancel),
];

pub fn callback_ids(region: Region) -> Vec<String> {
    CALLBACKS
        .iter()
        .map(|(suffix, _)| format!("{}.{suffix}", scope(region)))
        .collect()
}

pub fn install_callbacks(backend: Arc<ArcBackendNext>, db: Arc<PluginDb>) -> usize {
    static ONCE: OnceLock<()> = OnceLock::new();
    if ONCE.set(()).is_err() {
        return 0;
    }
    let book = CallbackBook::global();
    let mut registered = 0;
    for region in crate::REGIONS {
        for (suffix, action) in CALLBACKS {
            let id = format!("{}.{suffix}", scope(region));
            let handler = handler_for(action, region, Arc::clone(&backend), Arc::clone(&db));
            if book.register_scoped(id, crate::PLUGIN_ID, handler) {
                registered += 1;
            }
        }
    }
    registered
}

fn handler_for(
    action: Action,
    region: Region,
    backend: Arc<ArcBackendNext>,
    db: Arc<PluginDb>,
) -> CallbackHandler {
    Arc::new(move |request: CallbackRequest| {
        let backend = Arc::clone(&backend);
        let db = Arc::clone(&db);
        let future: CallbackFuture = Box::pin(async move {
            let scene = request.scene;
            let invocation = invocation_of(&request, &db);
            let ctx = Ctx::new(&invocation, region, &backend);
            let outcome = hints::scope(scene, region, async {
                match action {
                    Action::Best50 => boards::b50(&ctx, "").await,
                    Action::Recent => score::recent(&ctx).await,
                    Action::Menu => Ok(Some(help::root(&ctx))),
                    Action::Unbind => acct_intl::unbind(&ctx).await,
                    Action::UnbindConfirm => acct_intl::confirm(&ctx).await,
                    Action::LoginCancel => Ok(Some(cancelled(region))),
                }
            })
            .await;
            match outcome {
                Ok(outgoing) => outgoing,
                Err(error) => Some(reply::fail(error.to_string())),
            }
        });
        future
    })
}

fn cancelled(region: Region) -> Outgoing {
    reply::notice(format!(
        "已取消登录，需要时再发 {} login 重新扫码",
        root(region)
    ))
}

fn callback_invocation(request: &CallbackRequest) -> Invocation {
    let invocation = Invocation {
        plugin: crate::PLUGIN_ID.to_string(),
        scene: request.scene,
        target: request.target.clone(),
        user: request.user.clone(),
        ..Invocation::default()
    };
    match request.sender.clone() {
        Some(sender) => invocation.with_sender(sender),
        None => invocation,
    }
}

fn invocation_of(request: &CallbackRequest, db: &Arc<PluginDb>) -> Invocation {
    callback_invocation(request).with_plugin_db(Arc::clone(db))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Difficulty;

    #[test]
    fn callback_invocation_keeps_sender_for_pending_followup() {
        use aurorix_core::{Scene, Target};
        use aurorix_plugin::{InvocationSender, Outgoing};

        let sender = InvocationSender::new(|_outgoing: Outgoing| Box::pin(async { Ok(()) }));
        let request = CallbackRequest {
            button_id: String::from("arcaea.score.best50"),
            data: String::from("50"),
            plugin: crate::PLUGIN_ID.to_string(),
            user: String::from("user"),
            scene: Scene::Group,
            target: Target::new(Scene::Group, "group"),
            sender: Some(sender),
            interaction_id: String::from("interaction"),
        };
        assert!(
            callback_invocation(&request).has_sender(),
            "按钮交互必须保留中途发送通道，否则 pending 任务完成后无处推送结果"
        );

        let without_sender = CallbackRequest {
            sender: None,
            ..request
        };
        assert!(!callback_invocation(&without_sender).has_sender());
    }

    fn ids(keyboard: &Keyboard) -> Vec<String> {
        keyboard
            .flatten()
            .into_iter()
            .map(|button| button.id)
            .collect()
    }

    fn payloads(keyboard: &Keyboard) -> Vec<String> {
        keyboard
            .flatten()
            .into_iter()
            .filter_map(|button| button.action.msg)
            .collect()
    }

    fn every_board(region: Region) -> Vec<Keyboard> {
        let future = Difficulty::Future;
        [
            help_board(region),
            bind_board(region),
            login_board(region),
            relogin_board(region),
            rebind_board(region),
            unbind_board(region),
            unbind_confirm_board(region, "1700000000"),
            me_board(region),
            bind_ok_board(region),
            best_board(region),
            rank_board(region),
            rank_board_fallback(region),
            recent_board(region, "fractureray", future),
            score_board(region, "fractureray", future),
            chart_board(region, "fractureray", future),
            search_board(region, "定数>=10"),
            pref_board(region, "char"),
            rand_board(region, "定数>=10", "Fracture Ray"),
            detail_board(region, "fractureray", future),
            quiz_board(region),
            quiz_again_board(region, "Fracture Ray"),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    #[test]
    fn case_035() {
        for region in crate::REGIONS {
            let boards = every_board(region);
            assert!(boards.len() >= 20, "两块服都要有键盘，实得 {}", boards.len());
            for board in boards {
                assert!(board.validate().is_ok(), "{region:?} 键盘不合规");
                assert!(board.rows.len() <= 5, "{region:?} 超过五行");
                for row in &board.rows {
                    assert!(row.buttons.len() <= 5, "{region:?} 一行超过五颗");
                }
            }
        }
    }

    #[test]
    fn case_036() {
        let intl = Region::Intl;
        let cn = Region::Cn;
        let counts = |board: Option<Keyboard>| -> Vec<usize> {
            board
                .expect("键盘必须存在")
                .rows
                .iter()
                .map(|row| row.buttons.len())
                .collect()
        };
        assert_eq!(counts(help_board(intl)), vec![2]);
        assert_eq!(counts(help_board(cn)), vec![2]);
        assert_eq!(counts(bind_board(intl)), vec![1]);
        assert_eq!(counts(bind_board(cn)), vec![1]);
        assert_eq!(counts(login_board(cn)), vec![1]);
        assert_eq!(counts(relogin_board(cn)), vec![1]);
        assert_eq!(counts(rebind_board(intl)), vec![1]);
        assert_eq!(counts(rebind_board(cn)), vec![1]);
        assert_eq!(counts(unbind_board(intl)), vec![1]);
        assert_eq!(counts(unbind_confirm_board(intl, "1700000000")), vec![1]);
        let unbind = unbind_board(intl).expect("解绑棋盘");
        assert_eq!(unbind.rows[0].buttons[0].id, "arcaea.account.unbind");
        assert_eq!(unbind.rows[0].buttons[0].render_data.label, "解除绑定");
        let confirm = unbind_confirm_board(intl, "1700000000").expect("确认解绑棋盘");
        assert_eq!(
            confirm.rows[0].buttons[0].id,
            "arcaea.account.unbind.confirm"
        );
        assert_eq!(
            confirm.rows[0].buttons[0].action.callback_data.as_deref(),
            Some("1700000000"),
            "确认解绑要带上票号"
        );
        assert_eq!(counts(me_board(intl)), vec![3]);
        assert_eq!(counts(bind_ok_board(intl)), vec![4]);
        assert_eq!(counts(best_board(intl)), vec![3]);
        assert_eq!(counts(best_board(cn)), vec![4]);
        assert!(rank_board(intl).is_none(), "国际服没有榜单棋盘");
        assert_eq!(counts(rank_board(cn)), vec![2]);
        assert_eq!(counts(rank_board_fallback(intl)), vec![1]);
        assert_eq!(counts(rank_board_fallback(cn)), vec![2]);
        assert_eq!(
            counts(recent_board(intl, "fractureray", Difficulty::Future)),
            vec![4]
        );
        assert_eq!(
            counts(score_board(intl, "fractureray", Difficulty::Future)),
            vec![4]
        );
        assert_eq!(
            counts(chart_board(intl, "fractureray", Difficulty::Future)),
            vec![2]
        );
        assert_eq!(counts(search_board(intl, "定数>=10")), vec![3]);
        assert_eq!(counts(pref_board(intl, "char")), vec![3]);
        assert_eq!(
            counts(rand_board(intl, "定数>=10", "Fracture Ray")),
            vec![2]
        );
        assert_eq!(
            counts(detail_board(intl, "fractureray", Difficulty::Future)),
            vec![3]
        );
        assert_eq!(counts(quiz_board(intl)), vec![3]);
        assert_eq!(counts(quiz_again_board(intl, "Fracture Ray")), vec![1, 3]);
    }

    #[test]
    fn case_037() {
        let cn = best_board(Region::Cn).expect("国服 Best 50 棋盘");
        for id in ids(&cn) {
            assert!(id.starts_with("cn.arcaea."), "{id} 少了国服前缀");
        }
        assert!(payloads(&cn).contains(&String::from("/ca update")));
        let intl = best_board(Region::Intl).expect("国际服 Best 50 棋盘");
        for id in ids(&intl) {
            assert!(id.starts_with("arcaea."), "{id} 少了国际服前缀");
        }
        assert!(
            payloads(&intl).iter().all(|msg| !msg.starts_with("/ca")),
            "国际服不该出现国服前缀"
        );
    }

    #[test]
    fn case_038() {
        for region in crate::REGIONS {
            for board in [
                quiz_board(region).expect("猜曲绘棋盘"),
                quiz_again_board(region, "Fracture Ray").expect("结算棋盘"),
            ] {
                for button in board.flatten() {
                    assert_eq!(
                        button.action.action_type,
                        aurorix_buttons::ButtonAction::Command,
                        "猜曲绘按钮都是指令填充"
                    );
                    let msg = button.action.msg.clone().unwrap_or_default();
                    assert!(msg.starts_with("/a "), "{msg} 必须写死 /a 前缀");
                }
            }
        }
    }

    #[test]
    fn case_039() {
        let board = help_board(Region::Intl).expect("帮助棋盘");
        let button = &board.rows[0].buttons[0];
        assert_eq!(button.id, "arcaea.account.bind");
        assert_eq!(
            button.action.action_type,
            aurorix_buttons::ButtonAction::Command
        );
        assert_eq!(button.action.msg.as_deref(), Some("/a bind "));
        assert_eq!(button.render_data.label, "绑定账号");
        let cn = help_board(Region::Cn).expect("国服帮助棋盘");
        assert_eq!(cn.rows[0].buttons[0].id, "cn.arcaea.account.login");
        assert_eq!(cn.rows[0].buttons[0].action.msg.as_deref(), Some("/ca login"));
        assert_eq!(
            cn.rows[0].buttons[1].action.action_type,
            aurorix_buttons::ButtonAction::Callback
        );
        assert_eq!(
            board.rows[0].buttons[1].action.action_type,
            aurorix_buttons::ButtonAction::Callback
        );
        assert_eq!(button.render_data.style, ButtonStyle::GraySolid);
    }

    #[test]
    fn case_040() {
        for region in crate::REGIONS {
            let button = switch_diff_button(region).expect("切换难度按钮要能构建");
            assert_eq!(
                button.id,
                format!("{}.song.switch_diff", scope(region)),
                "id 要按服前缀命名"
            );
            assert_eq!(
                button.action.action_type,
                aurorix_buttons::ButtonAction::Command
            );
            assert_eq!(button.render_data.label, "切换难度");
            let msg = button.action.msg.clone().unwrap_or_default();
            assert_eq!(msg, format!("{} song ", root(region)));
            for board in every_board(region) {
                assert!(
                    !board.has_button(&button.id),
                    "原版生产未用，不该挂到任何键盘"
                );
            }
        }
    }

    #[test]
    fn case_041() {
        let expected = [
            "score.best50",
            "score.recent",
            "common.menu",
            "song.detail",
            "song.play",
            "score.single",
            "song.search",
            "song.rand.again",
            "song.switch_diff",
            "account.bind",
            "account.unbind",
            "account.update",
            "account.unbind.confirm",
            "account.login",
            "account.login.cancel",
            "account.relogin",
            "account.pref.pick",
            "account.pref.other",
            "try",
            "hint",
            "stop",
            "again",
            "over0",
            "over1",
            "over2",
        ];
        assert_eq!(expected.len(), 25);
        let mut seen: Vec<String> = Vec::new();
        for region in crate::REGIONS {
            let prefix = format!("{}.", scope(region));
            for board in every_board(region) {
                for button in board.flatten() {
                    seen.push(button.id.trim_start_matches(&prefix).trim_start_matches('.').to_string());
                }
            }
            if let Some(button) = switch_diff_button(region) {
                seen.push(button.id.trim_start_matches(&prefix).trim_start_matches('.').to_string());
            }
        }
        for suffix in expected {
            assert!(seen.iter().any(|item| item == suffix), "{suffix} 没有接上");
        }
        assert!(
            seen.iter().filter(|item| *item == "account.update").count() >= 2,
            "国服棋盘上要有更新存档按钮"
        );
    }

    #[test]
    fn case_042() {
        for region in crate::REGIONS {
            let allowed = callback_ids(region);
            for board in every_board(region) {
                for button in board.flatten() {
                    if button.action.action_type == aurorix_buttons::ButtonAction::Callback {
                        assert!(
                            allowed.contains(&button.id),
                            "{} 没有对应的 handler",
                            button.id
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn case_043() {
        let board = login_board(Region::Cn).expect("登录页");
        assert_eq!(board.rows[0].buttons.len(), 1);
        assert_eq!(board.rows[0].buttons[0].id, "cn.arcaea.account.login.cancel");
        let best = best_board(Region::Cn).expect("国服 Best 50");
        assert_eq!(best.rows[0].buttons[0].id, "cn.arcaea.account.update");
        assert_eq!(
            best.rows[0].buttons[0].action.msg.as_deref(),
            Some("/ca update")
        );
        let rank = rank_board(Region::Cn).expect("国服榜单");
        assert_eq!(rank.rows[0].buttons.len(), 2);
        assert_eq!(rank.rows[0].buttons[1].id, "cn.arcaea.common.menu");
    }
}
