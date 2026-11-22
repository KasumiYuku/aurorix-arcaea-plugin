use aurorix_template::{Registry, TemplateKind};

pub const MARKDOWN: [(&str, &str); 59] = [
    ("account.bind.already", include_str!("../templates/markdown/account/account.bind.already.md")),
    ("account.bind.error", include_str!("../templates/markdown/account/account.bind.error.md")),
    ("account.bind.ok", include_str!("../templates/markdown/account/account.bind.ok.md")),
    ("account.cn.bound", include_str!("../templates/markdown/account/account.cn.bound.md")),
    ("account.cn.login.ok", include_str!("../templates/markdown/account/account.cn.login.ok.md")),
    ("account.cn.login.ticket", include_str!("../templates/markdown/account/account.cn.login.ticket.md")),
    ("account.cn.unbound", include_str!("../templates/markdown/account/account.cn.unbound.md")),
    ("account.me", include_str!("../templates/markdown/account/account.me.md")),
    ("account.me.caption", include_str!("../templates/markdown/account/account.me.caption.md")),
    ("account.pref.caption", include_str!("../templates/markdown/account/account.pref.caption.md")),
    ("account.unbind.confirm", include_str!("../templates/markdown/account/account.unbind.confirm.md")),
    ("account.unbind.ok", include_str!("../templates/markdown/account/account.unbind.ok.md")),
    ("ack", include_str!("../templates/markdown/shared/ack.md")),
    ("calc.ptt", include_str!("../templates/markdown/calc/calc.ptt.md")),
    ("chart.none", include_str!("../templates/markdown/chart/chart.none.md")),
    ("chart.preview", include_str!("../templates/markdown/chart/chart.preview.md")),
    ("fail", include_str!("../templates/markdown/shared/fail.md")),
    ("guess.hint", include_str!("../templates/markdown/guess/guess.hint.md")),
    ("guess.over", include_str!("../templates/markdown/guess/guess.over.md")),
    ("guess.start", include_str!("../templates/markdown/guess/guess.start.md")),
    ("guess.step", include_str!("../templates/markdown/guess/guess.step.md")),
    ("guess.step.plain", include_str!("../templates/markdown/guess/guess.step.plain.md")),
    ("help", include_str!("../templates/markdown/help.md")),
    ("help.cn", include_str!("../templates/markdown/help.cn.md")),
    ("hints.line", include_str!("../templates/markdown/hints/line.md")),
    ("notice", include_str!("../templates/markdown/shared/notice.md")),
    ("pick", include_str!("../templates/markdown/shared/pick.md")),
    ("query.ahv", include_str!("../templates/markdown/query/query.ahv.md")),
    ("query.ahv.caption", include_str!("../templates/markdown/query/query.ahv.caption.md")),
    ("query.chapter", include_str!("../templates/markdown/query/query.chapter.md")),
    ("query.chapter.caption", include_str!("../templates/markdown/query/query.chapter.caption.md")),
    ("query.his", include_str!("../templates/markdown/query/query.his.md")),
    ("query.his.caption", include_str!("../templates/markdown/query/query.his.caption.md")),
    ("query.list", include_str!("../templates/markdown/query/query.list.md")),
    ("query.list.caption", include_str!("../templates/markdown/query/query.list.caption.md")),
    ("query.stats", include_str!("../templates/markdown/query/query.stats.md")),
    ("query.stats.caption", include_str!("../templates/markdown/query/query.stats.caption.md")),
    ("query.suggest", include_str!("../templates/markdown/query/query.suggest.md")),
    ("query.suggest.caption", include_str!("../templates/markdown/query/query.suggest.caption.md")),
    ("query.table", include_str!("../templates/markdown/query/query.table.md")),
    ("query.table.caption", include_str!("../templates/markdown/query/query.table.caption.md")),
    ("query.trend", include_str!("../templates/markdown/query/query.trend.md")),
    ("query.trend.caption", include_str!("../templates/markdown/query/query.trend.caption.md")),
    ("score.b50", include_str!("../templates/markdown/score/score.b50.md")),
    ("score.b50.fresh", include_str!("../templates/markdown/score/score.b50.fresh.md")),
    ("score.pending", include_str!("../templates/markdown/score/score.pending.md")),
    ("score.play", include_str!("../templates/markdown/score/score.play.md")),
    ("score.play.fresh", include_str!("../templates/markdown/score/score.play.fresh.md")),
    ("score.play.none", include_str!("../templates/markdown/score/score.play.none.md")),
    ("song.alias", include_str!("../templates/markdown/song/song.alias.md")),
    ("song.card", include_str!("../templates/markdown/song/song.card.md")),
    ("song.ill", include_str!("../templates/markdown/song/song.ill.md")),
    ("song.none", include_str!("../templates/markdown/song/song.none.md")),
    ("song.play", include_str!("../templates/markdown/song/song.play.md")),
    ("song.profile.caption", include_str!("../templates/markdown/song/song.profile.caption.md")),
    ("song.search.caption", include_str!("../templates/markdown/song/song.search.caption.md")),
    ("theme.switch", include_str!("../templates/markdown/theme/theme.switch.md")),
    ("unbound", include_str!("../templates/markdown/shared/unbound.md")),
    ("warn", include_str!("../templates/markdown/shared/warn.md")),
];

pub fn register_templates(registry: &Registry) -> usize {
    let mut registered = 0;
    for (id, body) in MARKDOWN {
        registry.register(TemplateKind::Markdown, crate::PLUGIN_ID, id, body);
        registered += 1;
    }
    registered
}

pub fn template_ids() -> Vec<&'static str> {
    MARKDOWN.iter().map(|(id, _)| *id).collect()
}
