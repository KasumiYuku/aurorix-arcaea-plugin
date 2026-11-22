pub mod acct_cn;
pub mod acct_intl;
pub mod admin_cmd;
pub mod analysis;
pub mod boards;
pub mod bridge;
pub mod buttons;
pub mod guess_cmd;
pub mod help;
pub mod hints;
pub mod normalize;
pub mod reply;
pub mod score;
pub mod song;
pub mod theme_cmd;

use std::sync::Arc;

use aurorix_core::RoleRequired;
use aurorix_plugin::{
    async_handler, CommandSpec, Invocation, Outgoing, PluginCommands, PluginError,
};
use aurorix_plugin::PluginConfigField;

use crate::backend::ArcBackendNext;
use crate::domain::BoardKind;
use crate::model::Region;

pub struct Services {
    pub backend: Arc<ArcBackendNext>,
}

impl Services {
    pub fn new(backend: Arc<ArcBackendNext>) -> Arc<Self> {
        Arc::new(Self { backend })
    }
}

pub struct Ctx<'a> {
    pub inv: &'a Invocation,
    pub region: Region,
    pub backend: &'a ArcBackendNext,
}

impl<'a> Ctx<'a> {
    pub fn new(inv: &'a Invocation, region: Region, backend: &'a ArcBackendNext) -> Self {
        Self {
            inv,
            region,
            backend,
        }
    }

    pub fn repo(&self) -> Option<crate::repo::ArcaeaRepo> {
        self.inv.plugin_db().map(crate::repo::ArcaeaRepo::from_db)
    }
}

macro_rules! on_arg {
    ($services:expr, $region:expr, $run:path) => {{
        let services = Arc::clone($services);
        move |inv: Invocation| {
            let services = Arc::clone(&services);
            let scene = inv.scene;
            hints::scope(scene, $region, async move {
                let ctx = Ctx::new(&inv, $region, &services.backend);
                $run(&ctx, inv.text.as_str()).await
            })
        }
    }};
}

macro_rules! on_ctx {
    ($services:expr, $region:expr, $run:path) => {{
        let services = Arc::clone($services);
        move |inv: Invocation| {
            let services = Arc::clone(&services);
            let scene = inv.scene;
            hints::scope(scene, $region, async move {
                let ctx = Ctx::new(&inv, $region, &services.backend);
                $run(&ctx).await
            })
        }
    }};
}

macro_rules! on_kind {
    ($services:expr, $region:expr, $run:path, $kind:expr) => {{
        let services = Arc::clone($services);
        move |inv: Invocation| {
            let services = Arc::clone(&services);
            let scene = inv.scene;
            hints::scope(scene, $region, async move {
                let ctx = Ctx::new(&inv, $region, &services.backend);
                $run(&ctx, $kind).await
            })
        }
    }};
}

macro_rules! sticky_fallback {
    ($services:expr, $region:expr) => {{
        let services = Arc::clone($services);
        async_handler(move |inv: Invocation| {
            let services = Arc::clone(&services);
            let scene = inv.scene;
            hints::scope(scene, $region, async move {
                let ctx = Ctx::new(&inv, $region, &services.backend);
                root_fallback(&services, &ctx).await
            })
        })
    }};
}
pub fn config_fields() -> Vec<PluginConfigField> {
    vec![
        PluginConfigField::text("backend_url", "后端地址")
            .default_value("http://127.0.0.1:2388")
            .description("ArcBackendNext 地址, 保存后重启生效"),
        PluginConfigField::password("backend_token", "后端令牌"),
        PluginConfigField::password("admin_token", "管理令牌"),
        PluginConfigField::number("timeout_seconds", "请求超时(秒)").default_value(45),
        PluginConfigField::boolean("webp_enabled", "图片转 WebP").default_value(true),
        PluginConfigField::text("assets_root", "素材目录").description("留空使用 assets/arcaea"),
    ]
}

pub fn plugin(services: Arc<Services>) -> PluginCommands {
    PluginCommands::new(crate::PLUGIN_ID, "Arcaea 音游助手")
        .describe("Arcaea 成绩查询、曲目检索、账号绑定与图片渲染")
        .command(intl_root(&services))
        .command(cn_root(&services))
        .with_config(config_fields())
}

fn intl_root(services: &Arc<Services>) -> CommandSpec {
    CommandSpec::new("a")
        .alias("arcaea")
        .describe("Arcaea 国际服指令总入口")
        .fallback(sticky_fallback!(services, Region::Intl))
        .subcommand(
            CommandSpec::new("bind")
                .alias("绑定")
                .describe("用好友码绑定 Arcaea 账号")
                .on(on_arg!(services, Region::Intl, acct_intl::bind)),
        )
        .subcommands(account_commands(services, Region::Intl))
        .subcommands(shared_commands(services, Region::Intl))
        .subcommands(admin_commands(services))
}

fn cn_root(services: &Arc<Services>) -> CommandSpec {
    CommandSpec::new("ca")
        .alias("ac")
        .describe("Arcaea 国服指令总入口")
        .fallback(sticky_fallback!(services, Region::Cn))
        .subcommands(account_commands(services, Region::Cn))
        .subcommands(shared_commands(services, Region::Cn))
        .subcommands(cn_login_commands(services))
}

fn account_commands(services: &Arc<Services>, region: Region) -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("unbind")
            .alias("解绑")
            .describe("解除当前账号绑定（需要二次确认）")
            .on(on_ctx!(services, region, acct_intl::unbind)),
        CommandSpec::new("confirm")
            .alias("确认")
            .alias("确认解绑")
            .describe("确认这次解绑")
            .on(on_ctx!(services, region, acct_intl::confirm)),
        CommandSpec::new("cancel")
            .alias("取消")
            .alias("取消解绑")
            .describe("取消这次解绑")
            .on(on_ctx!(services, region, acct_intl::cancel)),
        CommandSpec::new("me")
            .alias("我的")
            .describe("查看自己的账号绑定信息")
            .on(on_ctx!(services, region, acct_intl::me)),
        CommandSpec::new("char")
            .alias("立绘")
            .alias("角色")
            .describe("查看或切换卡面立绘")
            .on(on_arg!(services, region, acct_intl::char)),
        CommandSpec::new("banner")
            .alias("横幅")
            .alias("名片")
            .describe("查看或切换名片横幅")
            .on(on_arg!(services, region, acct_intl::banner)),
    ]
}

fn shared_commands(services: &Arc<Services>, region: Region) -> Vec<CommandSpec> {
    let mut commands = vec![
        CommandSpec::new("b50")
            .alias("B50")
            .describe("查询 Best 50 成绩")
            .on(on_arg!(services, region, boards::b50))
            .subcommands(b50_subs(services, region)),
        CommandSpec::new("recent")
            .alias("最近")
            .alias("cent")
            .describe("查看最近一次游玩成绩")
            .on(on_ctx!(services, region, score::recent)),
        CommandSpec::new("score")
            .alias("单曲")
            .alias("送")
            .describe("查询指定曲目的成绩")
            .on(on_arg!(services, region, score::single)),
        CommandSpec::new("song")
            .alias("曲目")
            .alias("详情")
            .alias("info")
            .describe("查看曲目信息")
            .on(on_arg!(services, region, song::song)),
        CommandSpec::new("alias")
            .alias("别名")
            .describe("查看曲目别名")
            .on(on_arg!(services, region, song::alias)),
        CommandSpec::new("ill")
            .alias("曲绘")
            .alias("看曲绘")
            .alias("查看曲绘")
            .describe("查看曲目曲绘")
            .on(on_arg!(services, region, song::ill)),
        CommandSpec::new("play")
            .alias("听")
            .alias("点歌")
            .alias("播放")
            .alias("music")
            .describe("试听曲目音频")
            .on(on_arg!(services, region, song::play)),
        CommandSpec::new("rand")
            .alias("随机")
            .alias("随机曲目")
            .describe("随机抽取一首曲目")
            .on(on_arg!(services, region, song::rand)),
        CommandSpec::new("search")
            .alias("检索")
            .alias("搜索")
            .describe("按条件检索曲目")
            .on(on_arg!(services, region, song::search)),
        CommandSpec::new("chart")
            .alias("谱面")
            .alias("预览")
            .describe("预览谱面信息")
            .on(on_arg!(services, region, song::chart)),
        CommandSpec::new("cal")
            .alias("计算")
            .describe("按定数与分数计算单曲 PTT")
            .on(on_arg!(services, region, song::cal)),
        CommandSpec::new("theme")
            .alias("主题")
            .describe("查看或切换出图主题")
            .on(on_arg!(services, region, theme_cmd::theme)),
        CommandSpec::new("guess")
            .alias("猜曲绘")
            .alias("猜歌")
            .alias("guesssong")
            .describe("猜曲绘小游戏")
            .on(on_arg!(services, region, guess_cmd::guess)),
    ];
    commands.extend(rank_commands(services, region));
    commands.extend(analysis_commands(services, region));
    commands
}

fn analysis_commands(services: &Arc<Services>, region: Region) -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("list")
            .alias("清单")
            .alias("筛选")
            .describe("按条件筛选成绩清单")
            .on(on_arg!(services, region, analysis::list)),
        CommandSpec::new("table")
            .alias("定数表")
            .alias("const")
            .alias("rating")
            .describe("按等级列出曲库定数表")
            .on(on_arg!(services, region, analysis::table)),
        CommandSpec::new("ahv")
            .alias("成就")
            .alias("achievement")
            .alias("comp")
            .describe("按定数分档查看谱面完成度")
            .on(on_arg!(services, region, analysis::ahv)),
        CommandSpec::new("stats")
            .alias("统计")
            .alias("区间")
            .describe("按定数区间统计完成度")
            .on(on_arg!(services, region, analysis::stats)),
        CommandSpec::new("suggest")
            .alias("push")
            .alias("up")
            .alias("推分")
            .alias("推分建议")
            .describe("按能力曲线给出推分建议")
            .on(on_arg!(services, region, analysis::suggest)),
        CommandSpec::new("chap")
            .alias("章节")
            .alias("曲包")
            .alias("pack")
            .describe("查看曲包完成度")
            .on(on_arg!(services, region, analysis::chap)),
        CommandSpec::new("trend")
            .alias("趋势")
            .alias("ptt")
            .describe("查看潜力值变化趋势")
            .on(on_arg!(services, region, analysis::trend)),
        CommandSpec::new("diff")
            .alias("变化")
            .alias("变动")
            .describe("查看最近成绩有变化的曲目")
            .on(on_arg!(services, region, analysis::diff)),
        CommandSpec::new("his")
            .alias("历史")
            .alias("record")
            .describe("查看单曲成绩变迁")
            .on(on_arg!(services, region, analysis::his)),
    ]
}

fn b50_subs(services: &Arc<Services>, region: Region) -> Vec<CommandSpec> {
    let mut subs = Vec::new();
    if !region.is_cn() {
        subs.push(
            CommandSpec::new("last")
                .describe("重放上一次的 Best 50 结果")
                .on(on_ctx!(services, region, boards::b50_last)),
        );
    }
    subs.push(
        CommandSpec::new("-f")
            .describe("强制刷新 Best 50")
            .role(RoleRequired::Admin)
            .on(on_ctx!(services, region, boards::b50_force)),
    );
    subs
}

fn rank_commands(services: &Arc<Services>, region: Region) -> Vec<CommandSpec> {
    let table: [(&str, BoardKind, [&str; 4]); 6] = [
        ("pm50", BoardKind::Pm50, ["p50", "完美50", "PM50", ""]),
        ("fr50", BoardKind::Fr50, ["全连50", "FR50", "", ""]),
        ("x50", BoardKind::X50, ["差一点50", "1lost50", "性50", "性30"]),
        ("nm50", BoardKind::Nm50, ["近PM50", "近满50", "", ""]),
        ("mx50", BoardKind::Mx50, ["理论值50", "MAX50", "", ""]),
        ("ol50", BoardKind::Ol50, ["越级50", "", "", ""]),
    ];
    let mut commands = Vec::new();
    for (name, kind, aliases) in table {
        let mut spec = CommandSpec::new(name).describe(boards::describe(kind));
        for alias in aliases {
            if !alias.is_empty() {
                spec = spec.alias(alias);
            }
        }
        spec = spec.on(on_kind!(services, region, boards::rank50, kind));
        if name == "x50" {
            spec = spec.alias("x30");
        }
        commands.push(spec);
    }
    commands
}

fn cn_login_commands(services: &Arc<Services>) -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("login")
            .alias("登录")
            .alias("bind")
            .alias("绑定")
            .describe("扫码登录国服账号")
            .on(on_ctx!(services, Region::Cn, acct_cn::login)),
        CommandSpec::new("relogin")
            .alias("重新登录")
            .alias("重登")
            .describe("国服登录续期")
            .on(on_ctx!(services, Region::Cn, acct_cn::relogin)),
        CommandSpec::new("update")
            .alias("同步")
            .describe("从国服云存档同步成绩（8 分钟内可更新 2 次；还没同步到成绩时第一次不必等）")
            .on(on_ctx!(services, Region::Cn, acct_cn::update)),
        CommandSpec::new("nbind")
            .describe("关闭国服新绑定（群主/管理员）")
            .role(RoleRequired::Admin)
            .on(on_ctx!(services, Region::Cn, acct_cn::nbind)),
        CommandSpec::new("ybind")
            .describe("开启国服新绑定（群主/管理员）")
            .role(RoleRequired::Admin)
            .on(on_ctx!(services, Region::Cn, acct_cn::ybind)),
    ]
}

fn admin_commands(services: &Arc<Services>) -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("updb")
            .alias("更新数据库")
            .alias("更新曲库")
            .describe("同步曲库数据（管理员）")
            .role(RoleRequired::Admin)
            .on(on_arg!(services, Region::Intl, admin_cmd::updb))
            .subcommand(
                CommandSpec::new("-d")
                    .describe("试运行：只比对不落库")
                    .role(RoleRequired::Admin)
                    .on(on_ctx!(services, Region::Intl, admin_cmd::updb_dry)),
            ),
        CommandSpec::new("maint")
            .alias("维护")
            .describe("查看或切换上游维护闸（管理员）")
            .role(RoleRequired::Admin)
            .on(on_arg!(services, Region::Intl, admin_cmd::maint)),
    ]
}

async fn root_fallback(
    services: &Arc<Services>,
    ctx: &Ctx<'_>,
) -> Result<Option<Outgoing>, PluginError> {
    let text = ctx.inv.text.trim().to_string();
    if text.is_empty() || normalize::is_help(&text) {
        return Ok(Some(help::root(ctx)));
    }
    if let Some((sub, arg)) = normalize::route(ctx.region, &text) {
        return dispatch(services, ctx, sub, &arg).await;
    }
    Ok(None)
}

async fn dispatch(
    services: &Arc<Services>,
    ctx: &Ctx<'_>,
    sub: &str,
    arg: &str,
) -> Result<Option<Outgoing>, PluginError> {
    let _ = services;
    match sub {
        "bind" => acct_intl::bind(ctx, arg).await,
        "unbind" => acct_intl::unbind(ctx).await,
        "confirm" => acct_intl::confirm(ctx).await,
        "cancel" => acct_intl::cancel(ctx).await,
        "me" => acct_intl::me(ctx).await,
        "char" => acct_intl::char(ctx, arg).await,
        "banner" => acct_intl::banner(ctx, arg).await,
        "b50" => boards::b50(ctx, arg).await,
        "recent" => score::recent(ctx).await,
        "score" => score::single(ctx, arg).await,
        "pm50" => boards::rank50(ctx, BoardKind::Pm50).await,
        "fr50" => boards::rank50(ctx, BoardKind::Fr50).await,
        "x50" => boards::rank50(ctx, BoardKind::X50).await,
        "nm50" => boards::rank50(ctx, BoardKind::Nm50).await,
        "mx50" => boards::rank50(ctx, BoardKind::Mx50).await,
        "ol50" => boards::rank50(ctx, BoardKind::Ol50).await,
        "song" => song::song(ctx, arg).await,
        "alias" => song::alias(ctx, arg).await,
        "ill" => song::ill(ctx, arg).await,
        "play" => song::play(ctx, arg).await,
        "rand" => song::rand(ctx, arg).await,
        "search" => song::search(ctx, arg).await,
        "chart" => song::chart(ctx, arg).await,
        "cal" => song::cal(ctx, arg).await,
        "list" => analysis::list(ctx, arg).await,
        "table" => analysis::table(ctx, arg).await,
        "ahv" => analysis::ahv(ctx, arg).await,
        "stats" => analysis::stats(ctx, arg).await,
        "suggest" => analysis::suggest(ctx, arg).await,
        "chap" => analysis::chap(ctx, arg).await,
        "trend" => analysis::trend(ctx, arg).await,
        "his" => analysis::his(ctx, arg).await,
        "diff" => analysis::diff(ctx, arg).await,
        "theme" => theme_cmd::theme(ctx, arg).await,
        "guess" => guess_cmd::guess(ctx, arg).await,
        "updb" | "maint" if ctx.region.is_cn() => Ok(Some(reply::notice(ADMIN_INTL_ONLY))),
        "updb" => admin_cmd::updb(ctx, arg).await,
        "maint" => admin_cmd::maint(ctx, arg).await,
        "login" => acct_cn::login(ctx).await,
        "relogin" => acct_cn::relogin(ctx).await,
        "update" => acct_cn::update(ctx).await,
        "nbind" => acct_cn::nbind(ctx).await,
        "ybind" => acct_cn::ybind(ctx).await,
        _ => Ok(None),
    }
}

pub const ADMIN_INTL_ONLY: &str = "曲库同步与维护闸只在国际服 /a 下提供，国服 /ca 没有这两条指令";

pub fn root_of(ctx: &Ctx<'_>) -> &'static str {
    if ctx.region.is_cn() {
        "/ca"
    } else {
        "/a"
    }
}

pub const BEIJING_OFFSET_SECONDS: i32 = 8 * 3600;

pub fn beijing() -> chrono::FixedOffset {
    chrono::FixedOffset::east_opt(BEIJING_OFFSET_SECONDS).expect("东八区偏移必须合法")
}

pub fn beijing_day_start(at: chrono::DateTime<chrono::Utc>) -> chrono::DateTime<chrono::Utc> {
    let offset = i64::from(BEIJING_OFFSET_SECONDS);
    let shifted = at.timestamp() + offset;
    let day = shifted.div_euclid(86_400) * 86_400;
    chrono::DateTime::from_timestamp(day - offset, 0).unwrap_or(at)
}

pub fn local_time(at: chrono::DateTime<chrono::Utc>) -> String {
    at.with_timezone(&beijing())
        .format("%Y-%m-%d %H:%M")
        .to_string()
}

pub fn two(value: f64) -> String {
    format!("{value:.2}")
}

pub fn one(value: f64) -> String {
    format!("{value:.1}")
}

pub async fn repo_of(ctx: &Ctx<'_>) -> Result<crate::repo::ArcaeaRepo, Box<Outgoing>> {
    match ctx.repo() {
        Some(repo) => Ok(repo),
        None => Err(Box::new(reply::fail("插件数据库还没准备好"))),
    }
}

pub async fn prefs_of(ctx: &Ctx<'_>, repo: &crate::repo::ArcaeaRepo) -> crate::model::Prefs {
    repo.prefs(ctx.region, &ctx.inv.user)
        .await
        .unwrap_or_default()
}

pub async fn bound_account(
    ctx: &Ctx<'_>,
    repo: &crate::repo::ArcaeaRepo,
) -> Result<crate::model::Account, Box<Outgoing>> {
    match repo.get_account(ctx.region, &ctx.inv.user).await {
        Ok(Some(account)) if account.uid().is_some() => Ok(account),
        Ok(_) => Err(Box::new(reply::unbound(ctx.region))),
        Err(error) => Err(Box::new(reply::fail(error.to_string()))),
    }
}

const REASON_COOLDOWN: &str = "cooldown";
const REASON_NO_NEW_PLAY: &str = "no_new_play";

pub fn freshness(meta: &crate::backend::QueryMeta) -> String {
    let (head, tail) = freshness_split(meta);
    join_parts(&[&head, &tail])
}

pub fn freshness_join(fresh: (&str, &str)) -> String {
    join_parts(&[fresh.0, fresh.1])
}

pub fn freshness_split(meta: &crate::backend::QueryMeta) -> (String, String) {
    if scanned_now(meta) {
        return (
            String::from("刚扫完"),
            join_parts(&[&spent_part(meta), &at_part(meta)]),
        );
    }
    match meta.kind.to_ascii_lowercase().as_str() {
        "live" => return (String::from("实时获取"), spent_part(meta)),
        "degraded" => {
            return match is_partial(&meta.reason) {
                true => (
                    reason_line(&meta.reason),
                    join_parts(&[&at_part(meta), &retry_part(meta)]),
                ),
                false => (
                    String::new(),
                    join_parts(&[&at_part(meta), &retry_part(meta)]),
                ),
            };
        }
        _ => {}
    }
    if meta.reason == REASON_NO_NEW_PLAY {
        return (since_part(meta), spent_part(meta));
    }
    let why = match meta.reason == REASON_COOLDOWN {
        true => String::new(),
        false => reason_line(&meta.reason),
    };
    (
        why,
        join_parts(&[&at_part(meta), &spent_part(meta), &retry_part(meta)]),
    )
}

pub fn stale_notes(meta: &crate::backend::QueryMeta) -> Vec<String> {
    if !meta.kind.eq_ignore_ascii_case("degraded") {
        return Vec::new();
    }
    if is_partial(&meta.reason) {
        if !meta.msg.is_empty() {
            return vec![meta.msg.clone()];
        }
        return vec![String::from(
            "这份数据可能不全：这次扫描提前收尾，缺的谱面没算进来",
        )];
    }
    let mut why = reason_line(&meta.reason);
    if why.is_empty() && !meta.reason.is_empty() {
        why = format!("原因码 {}", meta.reason);
    }
    if why.is_empty() {
        return vec![String::from("显示的是上一次存下的记录")];
    }
    vec![format!("{why}，显示的是上一次存下的记录")]
}

pub fn best_notes(meta: &crate::backend::QueryMeta, region: Region) -> Vec<String> {
    let mut notes = stale_notes(meta);
    if region.is_cn() {
        notes.push(String::from(
            "✨ 若出现曲目缺失或成绩落后的情况，请先在游戏「添加好友」页点击「云端同步」，再发送 /ca update",
        ));
        notes.push(String::from("⚠️ 请勿将此图上传至官方频道以及群聊"));
    }
    notes
}

fn scanned_now(meta: &crate::backend::QueryMeta) -> bool {
    !meta.cached
        && meta
            .task
            .as_ref()
            .is_some_and(crate::backend::TaskProgress::completed)
}

fn is_partial(reason: &str) -> bool {
    matches!(reason, "partial" | "scan_partial")
}

fn join_parts(parts: &[&str]) -> String {
    let kept: Vec<&str> = parts
        .iter()
        .copied()
        .filter(|part| !part.is_empty())
        .collect();
    match kept.is_empty() {
        true => String::from("数据是之前扫描的"),
        false => kept.join(" · "),
    }
}

fn at_part(meta: &crate::backend::QueryMeta) -> String {
    match meta.cached_at {
        Some(at) => format!("Created At {}", day_hm(at)),
        None => String::new(),
    }
}

fn since_part(meta: &crate::backend::QueryMeta) -> String {
    match meta.cached_at {
        Some(at) => format!("自 {} 起没有新游玩", day_hm(at)),
        None => String::from("没有新游玩"),
    }
}

fn spent_part(meta: &crate::backend::QueryMeta) -> String {
    match meta.elapsed_ms > 0 {
        true => format!("用时 {}", duration_fine(meta.elapsed_ms)),
        false => String::new(),
    }
}

fn retry_part(meta: &crate::backend::QueryMeta) -> String {
    if let Some(seconds) = meta.retry_after_seconds.filter(|seconds| *seconds > 0) {
        return format!("{}后可重试", duration_secs(seconds as i64));
    }
    match meta.cooldown_seconds > 0 {
        true => format!("{}后可刷新", duration_secs(meta.cooldown_seconds)),
        false => String::new(),
    }
}

fn duration_fine(millis: i64) -> String {
    match millis < 1000 {
        true => format!("{:.1} 秒", millis as f64 / 1000.0),
        false => duration_secs(millis / 1000),
    }
}

fn duration_secs(seconds: i64) -> String {
    match seconds {
        seconds if seconds <= 0 => String::from("0 秒"),
        seconds if seconds >= 60 && seconds % 60 == 0 => format!("{} 分钟", seconds / 60),
        seconds if seconds >= 60 => format!("{} 分 {} 秒", seconds / 60, seconds % 60),
        seconds => format!("{seconds} 秒"),
    }
}

fn reason_line(reason: &str) -> String {
    let line = match reason {
        "cooldown" => "冷却期内不重复扫描",
        "no_new_play" => "自上次扫描以来没有新游玩",
        "incremental" => "已按最近游玩增量更新",
        "scan_running" => "扫描还在进行",
        "scan_queued" => "扫描已排队",
        "upstream_empty" => "上游这次没给最近游玩",
        "upstream_timeout" | "soft_deadline" => "上游这次没在预算内回应",
        "upstream_error" => "上游这次没连上",
        "partial" | "scan_partial" => "这次扫描没能扫全",
        "upstream_unavailable" => "上游暂时不可用（服务器维护或客户端已更新）",
        "queue_full" => "同时在查的人太多，这次没能重新扫描",
        "local_only" => "本次只查了本地库",
        _ => "",
    };
    line.to_string()
}

fn day_hm(at: chrono::DateTime<chrono::Utc>) -> String {
    use chrono::Datelike;

    let local = at.with_timezone(&beijing());
    let now = chrono::Utc::now().with_timezone(&beijing());
    match (now.date_naive() - local.date_naive()).num_days() {
        0 => local.format("%H:%M").to_string(),
        1 => format!("昨天 {}", local.format("%H:%M")),
        2 => format!("前天 {}", local.format("%H:%M")),
        _ => match local.year() != now.year() {
            true => local.format("%Y-%m-%d %H:%M").to_string(),
            false => local.format("%m-%d %H:%M").to_string(),
        },
    }
}
pub fn ago(at: chrono::DateTime<chrono::Utc>) -> String {
    let delta = chrono::Utc::now() - at;
    match delta {
        delta if delta < chrono::Duration::minutes(1) => String::from("刚刚"),
        delta if delta < chrono::Duration::hours(1) => format!("{} 分钟前", delta.num_minutes()),
        delta if delta < chrono::Duration::hours(24) => format!("{} 小时前", delta.num_hours()),
        _ => at
            .with_timezone(&beijing())
            .format("%m-%d %H:%M")
            .to_string(),
    }
}

pub fn ack_outgoing(hint: &str) -> Outgoing {
    let mut args = reply::args();
    reply::put(&mut args, "hint", hint);
    reply::text("score/score.pending", &args).with_guide()
}
pub const ACK_AFTER: std::time::Duration = std::time::Duration::from_millis(1200);

pub async fn ack_after<F>(
    ctx: &Ctx<'_>,
    hint: &str,
    run: F,
) -> Result<Option<Outgoing>, PluginError>
where
    F: std::future::Future<Output = Outgoing> + Send + 'static,
{
    let (tx, mut rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let _ = tx.send(run.await);
    });
    match tokio::time::timeout(ACK_AFTER, &mut rx).await {
        Ok(Ok(outgoing)) => Ok(Some(outgoing)),
        Ok(Err(_)) => Ok(Some(reply::fail("查询中断了，请重新发送指令"))),
        Err(_) => match ctx.inv.opt_sender.clone() {
            Some(sender) => {
                tokio::spawn(async move {
                    if let Ok(outgoing) = rx.await {
                        let _ = sender.send(outgoing).await;
                    }
                });
                Ok(Some(ack_outgoing(hint)))
            }
            None => Ok(Some(match rx.await {
                Ok(outgoing) => outgoing,
                Err(_) => reply::fail("查询中断了，请重新发送指令"),
            })),
        },
    }
}

pub fn pending_outgoing(meta: &crate::backend::QueryMeta) -> Outgoing {
    let mut args = reply::args();
    reply::put(&mut args, "hint", waiting_hint(meta));
    reply::text("score/score.pending", &args).with_guide()
}

fn waiting_hint(meta: &crate::backend::QueryMeta) -> String {
    let mut parts = vec![String::from("正在扫描中")];
    let eta = eta_hint(meta);
    if !eta.is_empty() {
        parts.push(eta);
    }
    parts.push(String::from("，完成后会 @ 提醒您"));
    parts.join("")
}

fn eta_hint(meta: &crate::backend::QueryMeta) -> String {
    let eta = &meta.eta;
    if let Some(seconds) = eta
        .remaining
        .or(eta.seconds)
        .or(eta.high)
        .filter(|seconds| *seconds > 0)
    {
        return format!("，预计查询用时 {}", eta_clock(seconds));
    }
    match eta_from_msg(&meta.msg) {
        Some(seconds) => format!("，预计查询用时 {}", eta_clock(seconds)),
        None => String::new(),
    }
}

fn eta_from_msg(msg: &str) -> Option<u32> {
    let rest = &msg[msg.find("预计")?..];
    let digits: String = rest
        .chars()
        .skip_while(|ch| !ch.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

fn eta_clock(seconds: u32) -> String {
    match seconds >= 60 {
        true => format!("{} 分 {} 秒", seconds / 60, seconds % 60),
        false => format!("{seconds} 秒"),
    }
}
pub fn split_difficulty(arg: &str) -> (String, Option<crate::model::Difficulty>) {
    let mut words: Vec<&str> = arg.split_whitespace().collect();
    let Some(last) = words.pop() else {
        return (String::new(), None);
    };
    if let Some(difficulty) = crate::model::Difficulty::parse(last) {
        return (words.join(" "), Some(difficulty));
    }
    match glued_difficulty(last) {
        Some((prefix, difficulty)) => {
            words.push(prefix);
            (words.join(" "), Some(difficulty))
        }
        None => {
            words.push(last);
            (words.join(" "), None)
        }
    }
}

fn glued_difficulty(token: &str) -> Option<(&str, crate::model::Difficulty)> {
    use crate::model::Difficulty;
    const SUFFIXES: [(&str, Difficulty); 5] = [
        ("pst", Difficulty::Past),
        ("prs", Difficulty::Present),
        ("ftr", Difficulty::Future),
        ("byd", Difficulty::Beyond),
        ("etr", Difficulty::Eternal),
    ];
    let cut = token.len().checked_sub(3)?;
    if cut == 0 || !token.is_char_boundary(cut) {
        return None;
    }
    let tail = &token[cut..];
    if !tail.is_ascii() {
        return None;
    }
    let (_, difficulty) = SUFFIXES
        .into_iter()
        .find(|(suffix, _)| tail.eq_ignore_ascii_case(suffix))?;
    Some((&token[..cut], difficulty))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::BackendConfig;

    fn tree() -> PluginCommands {
        let backend = ArcBackendNext::new(BackendConfig::new("http://127.0.0.1:9"))
            .expect("后端配置要能构建");
        plugin(Services::new(Arc::new(backend)))
    }
    #[test]
    fn case_081() {
        let commands = tree();
        assert_eq!(commands.id, crate::PLUGIN_ID);
        assert_eq!(commands.commands.len(), 2);
        let intl = &commands.commands[0];
        assert_eq!(intl.name, "a");
        assert_eq!(intl.aliases, vec!["arcaea".to_string()]);
        assert!(intl.fallback.is_some(), "国际服根必须有粘合兜底");
        let cn = &commands.commands[1];
        assert_eq!(cn.name, "ca");
        assert_eq!(cn.aliases, vec!["ac".to_string()]);
        assert!(cn.fallback.is_some(), "国服根必须有粘合兜底");
    }

    #[test]
    fn case_082() {
        let commands = tree();
        let intl = &commands.commands[0];
        assert_eq!(intl.subcommands.len(), 37);
        let names: Vec<&str> = intl
            .subcommands
            .iter()
            .map(|spec| spec.name.as_str())
            .collect();
        for wanted in [
            "bind", "unbind", "confirm", "cancel", "me", "char", "banner", "b50", "recent",
            "score", "pm50", "fr50", "x50", "nm50", "mx50", "ol50", "song", "alias", "ill", "play",
            "rand", "search", "chart", "cal", "list", "table", "ahv", "stats", "suggest", "chap",
            "trend", "his", "diff", "theme", "guess", "updb", "maint",
        ] {
            assert!(names.contains(&wanted), "{wanted} 必须挂在 /a 上");
        }
        let cn = &commands.commands[1];
        assert_eq!(cn.subcommands.len(), 39);
        let cn_names: Vec<&str> = cn
            .subcommands
            .iter()
            .map(|spec| spec.name.as_str())
            .collect();
        for wanted in ["login", "relogin", "update", "nbind", "ybind"] {
            assert!(cn_names.contains(&wanted), "{wanted} 必须挂在 /ca 上");
        }
        assert!(!cn_names.contains(&"bind"), "国服根不挂独立的 bind");
        assert!(cn_names.contains(&"unbind"), "国服根保留 unbind");
        assert!(cn_names.contains(&"theme"), "主题指令两棵根都要挂");
        assert!(cn_names.contains(&"guess"), "猜曲绘两棵根都要挂");
        assert!(!cn_names.contains(&"updb"), "运维指令只挂国际服");
        assert!(!cn_names.contains(&"maint"), "运维指令只挂国际服");
    }

    #[test]
    fn case_083() {
        let commands = tree();
        let intl = &commands.commands[0];
        let b50 = intl
            .subcommands
            .iter()
            .find(|spec| spec.name == "b50")
            .expect("b50 必须存在");
        assert_eq!(b50.subcommands.len(), 2);
        let force = b50
            .subcommands
            .iter()
            .find(|spec| spec.name == "-f")
            .expect("要有 -f 子指令");
        assert_eq!(force.role, Some(RoleRequired::Admin));
        let last = b50
            .subcommands
            .iter()
            .find(|spec| spec.name == "last")
            .expect("要有 last 子指令");
        assert_eq!(last.role, None);
        let cn = &commands.commands[1];
        let cn_b50 = cn
            .subcommands
            .iter()
            .find(|spec| spec.name == "b50")
            .expect("国服也有 b50");
        assert_eq!(cn_b50.subcommands.len(), 1, "国服不挂 b50 last");
        assert_eq!(cn_b50.subcommands[0].name, "-f");
        let updb = intl
            .subcommands
            .iter()
            .find(|spec| spec.name == "updb")
            .expect("要有 updb");
        assert_eq!(
            updb.subcommands[0].name, "-d",
            "试运行子指令要与原版同写 -d"
        );
    }

    #[test]
    fn case_084() {
        assert_eq!(
            normalize::route(Region::Intl, "chart白雪"),
            Some(("chart", "白雪".to_string()))
        );
        assert_eq!(
            normalize::route(Region::Intl, "song Fracture Ray"),
            None,
            "剩余部分带空格就不算粘合"
        );
        assert_eq!(
            normalize::route(Region::Intl, "song"),
            None,
            "没参数不算粘合"
        );
        assert_eq!(
            normalize::route(Region::Cn, "cchart白雪"),
            Some(("chart", "白雪".to_string()))
        );
        assert_eq!(
            normalize::route(Region::Cn, "achart白雪"),
            None,
            "国际服根前缀不该被剥掉"
        );
        assert_eq!(
            normalize::route(Region::Cn, "l"),
            Some(("cal", String::new()))
        );
        assert_eq!(
            normalize::route(Region::Cn, "l 10.5 9950000"),
            Some(("cal", "10.5 9950000".to_string()))
        );
        assert!(normalize::is_help(" 帮助 "));
        assert!(normalize::is_help("help"));
        assert!(!normalize::is_help("help me"));
    }

    #[test]
    fn case_085() {
        let commands = tree();
        let boards = hints::build(&commands.commands).expect("四字名必须齐全");
        assert_eq!(boards.intl().pool_len(), 32);
        assert_eq!(
            boards.intl().pinned_len(),
            0,
            "国际服没有 update，不该有常驻项"
        );
        assert_eq!(boards.cn().pool_len(), 31);
        assert_eq!(boards.cn().pinned_len(), 1, "国服常驻项只有 update");
        let total = hints::install(&commands.commands).expect("装配必须成功");
        assert_eq!(total, 64);
    }

    fn invocation(text: &str) -> Invocation {
        Invocation {
            text: text.to_string(),
            scene: aurorix_core::Scene::Group,
            user: "u1".to_string(),
            ..Invocation::default()
        }
    }

    #[tokio::test]
    async fn case_086() {
        let backend = ArcBackendNext::new(BackendConfig::new("http://127.0.0.1:9"))
            .expect("后端配置要能构建");
        let services = Services::new(Arc::new(backend));
        let inv = invocation("l 10.5 9950000");
        let ctx = Ctx::new(&inv, Region::Cn, &services.backend);
        let outgoing = root_fallback(&services, &ctx)
            .await
            .expect("正常路径不该返回内部错误")
            .expect("cal 粘合后必须有回复");
        assert!(
            outgoing.body().contains("单曲 PTT") && outgoing.body().contains("9950000"),
            "cal 粘合后应当渲染单曲 PTT 模板，实际：{}",
            outgoing.body()
        );
    }

    #[tokio::test]
    async fn case_087() {
        let backend = ArcBackendNext::new(BackendConfig::new("http://127.0.0.1:9"))
            .expect("后端配置要能构建");
        let services = Services::new(Arc::new(backend));
        let inv = invocation("song白雪");
        let ctx = Ctx::new(&inv, Region::Intl, &services.backend);
        let outgoing = root_fallback(&services, &ctx)
            .await
            .expect("没有数据库也只该返回提示")
            .expect("song 粘合后必须有回复");
        assert!(
            outgoing.body().contains("出了些小问题"),
            "后端连不上时要走失败模板，实际：{}",
            outgoing.body()
        );
    }

    #[tokio::test]
    async fn case_088() {
        let backend = ArcBackendNext::new(BackendConfig::new("http://127.0.0.1:9"))
            .expect("后端配置要能构建");
        let services = Services::new(Arc::new(backend));
        let inv = invocation(" 帮助 ");
        let ctx = Ctx::new(&inv, Region::Cn, &services.backend);
        let outgoing = root_fallback(&services, &ctx)
            .await
            .expect("帮助不该返回内部错误")
            .expect("帮助必须有回复");
        assert!(
            outgoing.body().contains("%2Fca"),
            "国服帮助要写 /ca 指令，实际：{}",
            outgoing.body()
        );
    }

    #[test]
    fn case_089() {
        assert_eq!(
            split_difficulty("fractureray ftr"),
            (
                String::from("fractureray"),
                Some(crate::model::Difficulty::Future)
            )
        );
        assert_eq!(
            split_difficulty("fracture ray"),
            (String::from("fracture ray"), None)
        );
        assert_eq!(
            split_difficulty("byd"),
            (String::new(), Some(crate::model::Difficulty::Beyond))
        );
        assert_eq!(
            split_difficulty("dfftr"),
            (String::from("df"), Some(crate::model::Difficulty::Future))
        );
        assert_eq!(
            split_difficulty("fracture rayftr"),
            (String::from("fracture ray"), Some(crate::model::Difficulty::Future))
        );
        assert_eq!(
            split_difficulty("骨折光 超越"),
            (String::from("骨折光"), Some(crate::model::Difficulty::Beyond))
        );
        assert_eq!(split_difficulty("Twins"), (String::from("Twins"), None));
    }
    #[test]
    fn case_090() {
        let at = |text: &str| {
            chrono::DateTime::parse_from_rfc3339(text)
                .expect("时间要能解析")
                .with_timezone(&chrono::Utc)
        };
        assert_eq!(
            beijing_day_start(at("2026-03-01T15:59:00Z")).to_rfc3339(),
            "2026-02-28T16:00:00+00:00"
        );
        assert_eq!(
            beijing_day_start(at("2026-03-01T16:00:00Z")).to_rfc3339(),
            "2026-03-01T16:00:00+00:00"
        );
        assert_eq!(
            local_time(at("2026-03-01T16:30:00Z")),
            "2026-03-02 00:30",
            "同一时刻在北京时间已经跨天"
        );
    }
}
