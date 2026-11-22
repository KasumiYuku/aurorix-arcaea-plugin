use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

use aurorix_core::{RoleRequired, Scene};
use aurorix_plugin::{CommandSpec, Outgoing, Part};
use aurorix_template::Arg;

use crate::model::Region;

pub const PER_MESSAGE: usize = 4;

pub const LABELS: [(&str, &str); 32] = [
    ("me", "我的档案"),
    ("char", "立绘一览"),
    ("banner", "横幅一览"),
    ("b50", "BEST 50"),
    ("b50 last", "上次榜单"),
    ("recent", "最近游玩"),
    ("score", "单曲成绩"),
    ("chart", "谱面预览"),
    ("song", "单曲图鉴"),
    ("alias", "曲目别名"),
    ("ill", "单曲曲绘"),
    ("play", "ARC点歌"),
    ("theme", "成绩主题"),
    ("rand", "随机一曲"),
    ("search", "曲目检索"),
    ("cal", "PTT计算"),
    ("guess", "曲绘猜歌"),
    ("list", "成绩筛选"),
    ("table", "查定数表"),
    ("ahv", "查成就表"),
    ("stats", "区间统计"),
    ("suggest", "推分建议"),
    ("chap", "曲包完成"),
    ("trend", "PTT趋势"),
    ("his", "游玩历史"),
    ("diff", "成绩变化"),
    ("pm50", "查PM50"),
    ("fr50", "查FR50"),
    ("x50", "性歌50"),
    ("nm50", "近满50"),
    ("mx50", "理论50"),
    ("ol50", "越级50"),
];

pub const CN_ONLY: [(&str, &str); 2] = [("login", "国服登录"), ("update", "更新存档")];

pub const ARGS: [&str; 23] = [
    "bind",
    "char",
    "banner",
    "cal",
    "chart",
    "list",
    "table",
    "ahv",
    "stats",
    "suggest",
    "chap",
    "trend",
    "his",
    "diff",
    "score",
    "song",
    "alias",
    "ill",
    "play",
    "rand",
    "search",
    "theme",
    "maint",
];

pub const CLICKS: [(&str, &str); 6] = [
    ("score.best50", "b50"),
    ("score.recent", "recent"),
    ("common.menu", ""),
    ("account.unbind", "unbind"),
    ("account.unbind.confirm", "unbind"),
    ("account.login.cancel", "login"),
];

pub const HIDDEN: [&str; 6] = ["bind", "unbind", "login", "relogin", "confirm", "cancel"];

pub const PINNED: [&str; 1] = ["update"];

pub const NO_HINT: [&str; 11] = [
    "fail",
    "warn",
    "notice",
    "unbound",
    "account.cn.unbound",
    "ack",
    "help",
    "help.cn",
    "account.pref.caption",
    "pick",
    "song.play",
];

pub const NO_MENTION: [&str; 3] = ["help", "help.cn", "song.play"];

pub const NO_FOOTER: [&str; 26] = [
    "help",
    "help.cn",
    "fail",
    "warn",
    "notice",
    "unbound",
    "ack",
    "score.pending",
    "guess.start",
    "guess.step",
    "guess.step.plain",
    "guess.hint",
    "guess.over",
    "song.ill",
    "song.play",
    "account.unbind.confirm",
    "account.bind.ok",
    "account.bind.already",
    "account.bind.error",
    "account.unbind.ok",
    "account.cn.login.ticket",
    "account.cn.login.ok",
    "account.cn.bound",
    "song.none",
    "chart.none",
    "score.play.none",
];

const INTL_ROOT: &str = "/a";

const CN_ROOT: &str = "/ca";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hint {
    pub label: &'static str,
    pub command: String,
}

struct Tip {
    path: String,
    label: &'static str,
    command: String,
}

pub struct Board {
    root: &'static str,
    pool: Vec<Tip>,
    pinned: Vec<Tip>,
    index: HashMap<String, String>,
    cursor: AtomicUsize,
}

impl Board {
    fn build(root: &'static str, specs: &[CommandSpec]) -> Result<Self, String> {
        let mut board = Self {
            root,
            pool: Vec::new(),
            pinned: Vec::new(),
            index: HashMap::new(),
            cursor: AtomicUsize::new(0),
        };
        board.walk(specs, "")?;
        let mut pool = Vec::with_capacity(board.pool.len());
        let mut pinned = Vec::new();
        for tip in std::mem::take(&mut board.pool) {
            if PINNED.contains(&tip.path.as_str()) {
                pinned.push(tip);
            } else {
                pool.push(tip);
            }
        }
        shuffle(&mut pool);
        board.pool = pool;
        board.pinned = pinned;
        Ok(board)
    }

    fn walk(&mut self, specs: &[CommandSpec], parent: &str) -> Result<(), String> {
        for spec in specs {
            let path = if parent.is_empty() {
                spec.name.clone()
            } else {
                format!("{parent} {}", spec.name)
            };
            if spec.handler.is_some() && member(spec) && !HIDDEN.contains(&path.as_str()) {
                let label = label_of(self.root, &path)?;
                let mut command = format!("{} {}", self.root, path);
                if spec.args.is_some() || ARGS.contains(&path.as_str()) {
                    command.push(' ');
                }
                self.index.insert(normalize(&command), path.clone());
                for alias in &spec.aliases {
                    self.index
                        .insert(normalize(&format!("{} {}", self.root, alias)), path.clone());
                }
                self.pool.push(Tip {
                    path: path.clone(),
                    label,
                    command,
                });
            }
            self.walk(&spec.subcommands, &path)?;
        }
        Ok(())
    }

    pub(crate) fn pool_len(&self) -> usize {
        self.pool.len()
    }

    pub(crate) fn pinned_len(&self) -> usize {
        self.pinned.len()
    }

    #[cfg(test)]
    pub(crate) fn paths(&self) -> Vec<&str> {
        self.pool
            .iter()
            .chain(self.pinned.iter())
            .map(|tip| tip.path.as_str())
            .collect()
    }

    pub fn pick(&self, skipped_commands: &[String], skipped_ids: &[String]) -> Vec<Hint> {
        let total = self.pool.len();
        if total == 0 {
            return Vec::new();
        }
        let mut skip: Vec<&str> = Vec::with_capacity(skipped_commands.len());
        for command in skipped_commands {
            if let Some(path) = self.index.get(normalize(command).as_str()) {
                skip.push(path.as_str());
            }
        }
        for id in skipped_ids {
            let Some((_, path)) = CLICKS.iter().find(|(click, _)| click == id) else {
                continue;
            };
            if path.is_empty() || skip.contains(path) {
                continue;
            }
            skip.push(path);
        }
        let mut picked: Vec<Hint> = Vec::with_capacity(PER_MESSAGE);
        for tip in &self.pinned {
            if skip.contains(&tip.path.as_str()) {
                continue;
            }
            picked.push(Hint {
                label: tip.label,
                command: tip.command.clone(),
            });
        }
        let start = self.cursor.fetch_add(PER_MESSAGE, Ordering::SeqCst) % total;
        for offset in 0..total {
            if picked.len() == PER_MESSAGE {
                break;
            }
            let candidate = &self.pool[(start + offset) % total];
            if skip.contains(&candidate.path.as_str()) {
                continue;
            }
            picked.push(Hint {
                label: candidate.label,
                command: candidate.command.clone(),
            });
        }
        picked
    }
}

pub struct Boards {
    intl: Board,
    cn: Board,
}

impl Boards {
    fn board(&self, region: Region) -> &Board {
        if region.is_cn() {
            &self.cn
        } else {
            &self.intl
        }
    }

    #[cfg(test)]
    pub(crate) fn intl(&self) -> &Board {
        &self.intl
    }

    #[cfg(test)]
    pub(crate) fn cn(&self) -> &Board {
        &self.cn
    }
}

static BOARDS: OnceLock<Boards> = OnceLock::new();

static REGISTERED: OnceLock<()> = OnceLock::new();

pub fn build(specs: &[CommandSpec]) -> Result<Boards, String> {
    let mut intl: Option<Board> = None;
    let mut cn: Option<Board> = None;
    for spec in specs {
        match spec.name.as_str() {
            "a" => intl = Some(Board::build(INTL_ROOT, &spec.subcommands)?),
            "ca" => cn = Some(Board::build(CN_ROOT, &spec.subcommands)?),
            other => return Err(format!("提示行不认识指令根 {other}")),
        }
    }
    Ok(Boards {
        intl: intl.ok_or_else(|| "提示行缺少国际服指令根 /a".to_string())?,
        cn: cn.ok_or_else(|| "提示行缺少国服指令根 /ca".to_string())?,
    })
}

pub fn install(specs: &[CommandSpec]) -> Result<usize, String> {
    let boards = build(specs)?;
    let total = boards.intl.pool_len() + boards.intl.pinned_len();
    let total = total + boards.cn.pool_len() + boards.cn.pinned_len();
    let _ = BOARDS.set(boards);
    Ok(total)
}

pub struct Scope {
    pub scene: Scene,
    pub region: Region,
    pending: RefCell<Option<String>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Snapshot {
    pub scene: Scene,
    pub region: Region,
}

tokio::task_local! {
    static SCOPE: Scope;
}

pub fn scope<F>(scene: Scene, region: Region, future: F) -> impl Future<Output = F::Output>
where
    F: Future,
{
    SCOPE.scope(
        Scope {
            scene,
            region,
            pending: RefCell::new(None),
        },
        future,
    )
}

pub fn current_scope() -> Option<Snapshot> {
    SCOPE
        .try_with(|scope| Snapshot {
            scene: scope.scene,
            region: scope.region,
        })
        .ok()
}

fn pending_set(line: String) {
    let _ = SCOPE.try_with(|scope| {
        *scope.pending.borrow_mut() = Some(line);
    });
}

fn pending_take() -> Option<String> {
    SCOPE
        .try_with(|scope| scope.pending.borrow_mut().take())
        .ok()
        .flatten()
}

pub fn short_id(template_id: &str) -> &str {
    match template_id.rsplit_once('/') {
        Some((_, short)) => short,
        None => template_id,
    }
}

pub fn skipped(template_id: &str) -> bool {
    NO_HINT.contains(&short_id(template_id))
}

pub fn whitelisted(template_id: &str) -> bool {
    NO_FOOTER.contains(&short_id(template_id))
}

fn member(spec: &CommandSpec) -> bool {
    matches!(spec.role, None | Some(RoleRequired::Member))
}

fn label_of(root: &str, path: &str) -> Result<&'static str, String> {
    if let Some((_, label)) = LABELS.iter().find(|(name, _)| *name == path) {
        return Ok(label);
    }
    if root != INTL_ROOT {
        if let Some((_, label)) = CN_ONLY.iter().find(|(name, _)| *name == path) {
            return Ok(label);
        }
    }
    Err(format!(
        "指令 {root} {path} 没有四字提示名：在 src/commands/hints.rs 的 LABELS 里补一条"
    ))
}

fn normalize(command: &str) -> String {
    let mut fields = command.split_whitespace();
    match (fields.next(), fields.next()) {
        (Some(head), Some(tail)) => format!("{head} {tail}"),
        _ => command.trim().to_string(),
    }
}

fn shuffle(pool: &mut [Tip]) {
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    for index in (1..pool.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let pick = (state % (index as u64 + 1)) as usize;
        pool.swap(index, pick);
    }
}

fn ensure_templates() {
    REGISTERED.get_or_init(|| {
        crate::templates::register_templates(aurorix_template::Registry::global());
    });
}

pub fn render(hints: &[Hint]) -> String {
    if hints.is_empty() {
        return String::new();
    }
    ensure_templates();
    let mut tips: Vec<Arg> = Vec::with_capacity(hints.len());
    for hint in hints {
        let mut item = BTreeMap::new();
        item.insert("label".to_string(), Arg::Str(format!("[{}]", hint.label)));
        item.insert("command".to_string(), Arg::Str(hint.command.clone()));
        tips.push(Arg::Map(item));
    }
    let mut args = aurorix_template::Args::new();
    args.insert("tips".to_string(), Arg::List(tips));
    aurorix_template::fill_for(crate::PLUGIN_ID, "hints.line", &args)
        .map(|line| line.trim().to_string())
        .unwrap_or_default()
}

pub fn line_for(region: Region, skipped_commands: &[String], skipped_ids: &[String]) -> String {
    let Some(boards) = BOARDS.get() else {
        return String::new();
    };
    render(&boards.board(region).pick(skipped_commands, skipped_ids))
}

pub fn finish(template_id: &str, outgoing: Outgoing) -> Outgoing {
    let outgoing = if whitelisted(template_id) {
        outgoing.no_footer()
    } else {
        outgoing
    };
    let outgoing = if NO_MENTION.contains(&short_id(template_id)) {
        outgoing
    } else {
        mention(outgoing)
    };
    if skipped(template_id) {
        return outgoing;
    }
    if matches!(current_scope().map(|scope| scope.scene), Some(Scene::Channel)) {
        return outgoing;
    }
    attach(outgoing)
}

pub fn mention(outgoing: Outgoing) -> Outgoing {
    if outgoing.at {
        return outgoing;
    }
    let Some(scope) = current_scope() else {
        return outgoing;
    };
    if !matches!(scope.scene, Scene::Group | Scene::Channel) {
        return outgoing;
    }
    Outgoing { at: true, ..outgoing }
}

fn attach(outgoing: Outgoing) -> Outgoing {
    let region = current_scope()
        .map(|scope| scope.region)
        .unwrap_or(Region::Intl);
    let line = line_for(region, &[], &[]);
    if line.is_empty() {
        return outgoing;
    }
    pending_set(line.clone());
    let separator = match outgoing.body().trim().is_empty() {
        true => "",
        false => "\n\n",
    };
    outgoing.markdown(format!("{separator}{line}"))
}

const HINT_PREFIX: &str = "---\n\n> **🔗";

pub fn relocate(mut outgoing: Outgoing) -> Outgoing {
    let Some(index) = outgoing
        .parts
        .iter()
        .position(|part| matches!(part, Part::Markdown(text) if is_hint_line(text)))
    else {
        return outgoing;
    };
    if index + 1 == outgoing.parts.len() {
        return outgoing;
    }
    let hint = outgoing.parts.remove(index);
    outgoing.parts.push(hint);
    outgoing
}

fn is_hint_line(text: &str) -> bool {
    text.trim_start().starts_with(HINT_PREFIX)
}

pub fn land(mut outgoing: Outgoing, keyboard: Option<&aurorix_buttons::Keyboard>) -> Outgoing {
    let Some(previous) = pending_take() else {
        return outgoing;
    };
    let Some(scope) = current_scope() else {
        return outgoing;
    };
    let mut skipped_commands: Vec<String> = Vec::new();
    let mut skipped_ids: Vec<String> = Vec::new();
    if let Some(keyboard) = keyboard {
        for row in &keyboard.rows {
            for button in &row.buttons {
                if button.action.action_type == aurorix_buttons::ButtonAction::Command {
                    if let Some(message) = button.action.msg.as_deref() {
                        if !message.trim().is_empty() {
                            skipped_commands.push(message.to_string());
                        }
                    }
                    continue;
                }
                skipped_ids.push(button.id.clone());
            }
        }
    }
    let next = line_for(scope.region, &skipped_commands, &skipped_ids);
    if next == previous {
        return outgoing;
    }
    let (head, separator) = {
        let Some(aurorix_plugin::Part::Markdown(last)) = outgoing.parts.last() else {
            return outgoing;
        };
        let separator = match last.starts_with("\n\n") {
            true => "\n\n",
            false => "",
        };
        if last == &previous {
            (String::new(), separator)
        } else {
            let Some(head) = last.strip_suffix(&format!("\n\n{previous}")) else {
                return outgoing;
            };
            (head.trim_end().to_string(), separator)
        }
    };
    let tail = match (head.is_empty(), next.is_empty()) {
        (true, true) => String::new(),
        (true, false) => format!("{separator}{next}"),
        (false, true) => head,
        (false, false) => format!("{head}\n\n{next}"),
    };
    if tail.is_empty() {
        outgoing.parts.pop();
    } else if let Some(aurorix_plugin::Part::Markdown(last)) = outgoing.parts.last_mut() {
        *last = tail;
    }
    outgoing
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurorix_plugin::ReplyKind;

    fn hint_of(outgoing: &Outgoing) -> String {
        match outgoing.parts.last() {
            Some(aurorix_plugin::Part::Markdown(text)) => text.clone(),
            _ => String::new(),
        }
    }

    fn leaf(name: &str) -> CommandSpec {
        CommandSpec::new(name).on_sync(|_| Ok(None))
    }

    fn admin(name: &str) -> CommandSpec {
        CommandSpec::new(name)
            .role(RoleRequired::Admin)
            .on_sync(|_| Ok(None))
    }

    fn clone_spec(spec: &CommandSpec) -> CommandSpec {
        let mut out = CommandSpec::new(spec.name.clone());
        out.aliases = spec.aliases.clone();
        out.role = spec.role;
        out.args = spec.args.clone();
        out.handler = spec.handler.clone();
        out.subcommands = spec.subcommands.iter().map(clone_spec).collect();
        out
    }

    fn roots() -> Vec<CommandSpec> {
        let mut intl: Vec<CommandSpec> = LABELS
            .iter()
            .map(|(path, _)| {
                if *path == "b50" {
                    leaf("b50").subcommand(leaf("last"))
                } else {
                    leaf(path)
                }
            })
            .filter(|spec| spec.name != "b50 last")
            .collect();
        intl.push(leaf("bind"));
        intl.push(leaf("unbind"));
        intl.push(admin("updb"));
        let mut cn: Vec<CommandSpec> = intl
            .iter()
            .filter(|spec| spec.name != "updb")
            .map(clone_spec)
            .map(|spec| {
                if spec.name == "b50" {
                    leaf("b50").subcommand(admin("-f"))
                } else {
                    spec
                }
            })
            .collect();
        cn.push(leaf("login"));
        cn.push(leaf("update"));
        cn.push(leaf("relogin"));
        vec![
            CommandSpec::new("a").subcommands(intl),
            CommandSpec::new("ca").subcommands(cn),
        ]
    }

    #[test]
    fn case_016() {
        let boards = build(&roots()).expect("四字名必须齐全");
        assert_eq!(boards.intl().pool_len(), 32);
        assert_eq!(boards.intl().pinned_len(), 0);
        assert_eq!(boards.cn().pool_len(), 31);
        assert_eq!(boards.cn().pinned_len(), 1);
        assert!(boards.cn().paths().contains(&"update"));
        assert!(boards.intl().paths().contains(&"b50 last"));
        assert!(!boards.intl().paths().contains(&"bind"));
        assert!(!boards.intl().paths().contains(&"updb"));
    }

    #[test]
    fn case_017() {
        let specs = vec![CommandSpec::new("a").subcommand(leaf("nope"))];
        let error = build(&specs).err().expect("缺名必须报错");
        assert!(error.contains("/a nope"), "错误信息要指出是哪条指令：{error}");
    }

    #[test]
    fn case_018() {
        let specs = [CommandSpec::new("a").subcommands(vec![CommandSpec::new("score")
            .with_args(aurorix_plugin::ArgsSpec::new())
            .on_sync(|_| Ok(None))])];
        let board = Board::build("/a", &specs[0].subcommands).expect("四字名必须齐全");
        let picked = board.pick(&[], &[]);
        assert_eq!(picked[0].command, "/a score ");
        assert_eq!(picked[0].label, "单曲成绩");
    }

    #[test]
    fn case_019() {
        let boards = build(&roots()).expect("四字名必须齐全");
        let cn = boards.cn();
        let picked = cn.pick(&[], &[]);
        assert_eq!(picked.len(), PER_MESSAGE);
        assert_eq!(picked[0].command, "/ca update");
        let update_count = picked
            .iter()
            .filter(|hint| hint.command == "/ca update")
            .count();
        assert_eq!(update_count, 1, "常驻提示只出现一次");
    }

    #[test]
    fn case_020() {
        let boards = build(&roots()).expect("四字名必须齐全");
        let intl = boards.intl();
        let first = intl.pick(&[], &[]);
        let second = intl.pick(&[], &[]);
        assert_eq!(first.len(), PER_MESSAGE);
        assert_eq!(second.len(), PER_MESSAGE);
        assert_ne!(first[0].command, second[0].command, "两批必须错开");
        let mut seen: Vec<String> = Vec::new();
        for _ in 0..8 {
            for hint in intl.pick(&[], &[]) {
                if !seen.contains(&hint.command) {
                    seen.push(hint.command);
                }
            }
        }
        assert_eq!(seen.len(), 32, "轮换要让每条指令都露一次脸");
    }

    #[test]
    fn case_021() {
        let boards = build(&roots()).expect("四字名必须齐全");
        let intl = boards.intl();
        let blocked = intl.pick(&[], &[])[0].command.clone();
        let picked = intl.pick(std::slice::from_ref(&blocked), &[]);
        assert_eq!(picked.len(), PER_MESSAGE);
        assert!(
            !picked.iter().any(|hint| hint.command == blocked),
            "已经出现过的指令不该再被推荐：{blocked}"
        );
    }

    #[tokio::test]
    async fn case_022() {
        let group = scope(Scene::Group, Region::Intl, async {
            (
                finish("shared/notice", Outgoing::new(ReplyKind::Handler, "x")).at,
                finish("help", Outgoing::new(ReplyKind::Handler, "x")).at,
            )
        })
        .await;
        assert!(group.0, "群聊里结果要艾特触发者");
        assert!(!group.1, "帮助页艾特到人头上是噪音");
        let single = scope(Scene::C2c, Region::Intl, async {
            finish("shared/notice", Outgoing::new(ReplyKind::Handler, "x")).at
        })
        .await;
        assert!(!single, "单聊里艾特没有意义");
    }

    #[test]
    fn case_023() {
        assert_eq!(NO_FOOTER.len(), 26);
        assert_eq!(NO_HINT.len(), 11);
        let outgoing = finish(
            "score/score.play.none",
            Outgoing::new(ReplyKind::Handler, "x"),
        );
        assert!(outgoing.no_footer);
        assert!(skipped("shared/fail"));
        assert!(skipped("account/account.cn.unbound"));
        assert!(skipped("account/account.pref.caption"));
        assert!(skipped("shared/pick"));
        assert!(!skipped("song/song.ill"));
        assert!(whitelisted("song/song.ill"));
    }

    #[tokio::test]
    async fn case_024() {
        let _ = BOARDS.set(build(&roots()).expect("四字名必须齐全"));
        let outgoing = scope(Scene::Group, Region::Intl, async {
            finish(
                "score/score.b50",
                Outgoing::new(ReplyKind::Handler, "正文"),
            )
        })
        .await;
        assert!(outgoing.body().starts_with("正文"), "正文必须排在提示行前面");
        let hint = hint_of(&outgoing);
        let hint = hint.trim_start();
        let lines: Vec<&str> = hint.lines().collect();
        assert_eq!(lines[0], "---");
        assert_eq!(lines[1], "");
        assert!(
            lines[2].starts_with("> **🔗[欢迎大家点击这里帮我们一起补齐别名]"),
            "抬头不对：{}",
            lines[2]
        );
        assert!(lines[3].starts_with(">  📌 "), "提示行不对：{}", lines[3]);
        assert!(
            lines[3].contains("qqbot-cmd-input"),
            "提示必须渲染成可点指令：{}",
            lines[3]
        );
        assert_eq!(lines.len(), 4, "整行只该有这么几行：{lines:?}");
    }

    #[tokio::test]
    async fn case_025() {
        let _ = BOARDS.set(build(&roots()).expect("四字名必须齐全"));
        let channel = scope(Scene::Channel, Region::Intl, async {
            finish(
                "score/score.b50",
                Outgoing::new(ReplyKind::Handler, "正文"),
            )
        })
        .await;
        assert_eq!(channel.body(), "正文");
        let group = scope(Scene::Group, Region::Intl, async {
            finish(
                "score/score.b50",
                Outgoing::new(ReplyKind::Handler, "正文"),
            )
        })
        .await;
        let hint = hint_of(&group);
        assert!(hint.contains("---"));
        assert!(hint.contains("%2Fa%20"), "国际服提示行必须填 /a 原文");
        let cn = scope(Scene::Group, Region::Cn, async {
            finish(
                "score/score.b50",
                Outgoing::new(ReplyKind::Handler, "正文"),
            )
        })
        .await;
        let hint = hint_of(&cn);
        assert!(
            hint.contains("%2Fca%20"),
            "国服提示行必须填 /ca 原文：{hint}"
        );
    }

    fn keyboard_of(commands: &[String]) -> aurorix_buttons::Keyboard {
        let mut buttons: Vec<aurorix_buttons::Button> = commands
            .iter()
            .enumerate()
            .map(|(index, command)| aurorix_buttons::Button {
                id: format!("b{index}"),
                render_data: aurorix_buttons::RenderData::default(),
                action: aurorix_buttons::Action {
                    action_type: aurorix_buttons::ButtonAction::Command,
                    msg: Some(command.clone()),
                    ..aurorix_buttons::Action::default()
                },
            })
            .collect();
        buttons.push(aurorix_buttons::Button {
            id: String::from("score.best50"),
            render_data: aurorix_buttons::RenderData::default(),
            action: aurorix_buttons::Action {
                action_type: aurorix_buttons::ButtonAction::Callback,
                callback_data: Some(String::from("50")),
                ..aurorix_buttons::Action::default()
            },
        });
        aurorix_buttons::Keyboard {
            rows: vec![aurorix_buttons::ButtonRow { buttons }],
            ..aurorix_buttons::Keyboard::default()
        }
    }

    fn pool_commands() -> Vec<String> {
        let boards = build(&roots()).expect("四字名必须齐全");
        boards
            .intl()
            .paths()
            .iter()
            .map(|path| {
                if ARGS.contains(path) {
                    format!("/a {path} ")
                } else {
                    format!("/a {path}")
                }
            })
            .collect()
    }

    #[tokio::test]
    async fn case_026() {
        let _ = BOARDS.set(build(&roots()).expect("四字名必须齐全"));
        let keyboard = keyboard_of(&pool_commands());
        let outgoing = scope(Scene::Group, Region::Intl, async move {
            let text = finish(
                "score/score.b50",
                Outgoing::new(ReplyKind::Handler, "正文"),
            );
            assert!(hint_of(&text).contains("📌"), "正文先贴上提示行");
            land(text, Some(&keyboard))
        })
        .await;
        assert_eq!(outgoing.body(), "正文");
        assert!(
            hint_of(&outgoing).is_empty(),
            "整池都被当页摆出来时提示行要整行撤掉"
        );
    }

    #[tokio::test]
    async fn case_027() {
        let _ = BOARDS.set(build(&roots()).expect("四字名必须齐全"));
        let taken = pool_commands();
        let keyboard = keyboard_of(&taken[..4]);
        let outgoing = scope(Scene::Group, Region::Intl, async move {
            land(
                finish(
                    "score/score.b50",
                    Outgoing::new(ReplyKind::Handler, "正文"),
                ),
                Some(&keyboard),
            )
        })
        .await;
        let hint = hint_of(&outgoing);
        assert!(hint.starts_with("\n\n"), "提示行与前文之间必须留一个空行：{hint:?}");
        assert!(hint.trim_start().starts_with("---"), "提示行必须还在");
        assert!(hint.contains("📌"));
    }

    #[tokio::test]
    async fn case_028() {
        let _ = BOARDS.set(build(&roots()).expect("四字名必须齐全"));
        let outgoing = scope(Scene::Group, Region::Intl, async {
            finish("", Outgoing::new(ReplyKind::Handler, String::new()))
        })
        .await;
        let hint = hint_of(&outgoing);
        assert!(
            hint.starts_with("---"),
            "正文为空时整段就是提示行本身：{hint:?}"
        );
    }

    #[tokio::test]
    async fn case_029() {
        let _ = BOARDS.set(build(&roots()).expect("四字名必须齐全"));
        let keyboard = keyboard_of(&pool_commands());
        let outgoing = scope(Scene::Group, Region::Intl, async move {
            land(
                finish("", Outgoing::new(ReplyKind::Handler, String::new())),
                Some(&keyboard),
            )
        })
        .await;
        assert!(hint_of(&outgoing).is_empty(), "整池都摆出来时提示行要整行撤掉");
    }

    #[tokio::test]
    async fn land_keeps_a_blank_line_before_the_rule() {
        let _ = BOARDS.set(build(&roots()).expect("四字名必须齐全"));
        let keyboard = keyboard_of(&pool_commands()[..4]);
        let outgoing = scope(Scene::Group, Region::Intl, async move {
            land(
                finish(
                    "score/score.b50",
                    Outgoing::new(ReplyKind::Handler, "正文"),
                ),
                Some(&keyboard),
            )
        })
        .await;
        let body = outgoing.body();
        assert!(
            body.contains("正文\n\n---\n\n"),
            "分割线前面必须空一行, 否则 markdown 会把正文当成标题：{body:?}"
        );
    }

    #[test]
    fn case_030() {
        let mut outgoing = Outgoing::new(ReplyKind::Handler, "正文");
        outgoing.parts.push(Part::Markdown(String::from(
            "---\n\n> **🔗[欢迎大家点击这里帮我们一起补齐别名](https://arc.afkeru.icu)**\n>  📌 tip",
        )));
        outgoing
            .parts
            .push(Part::Image(aurorix_plugin::ImageSource::Url {
                url: String::from("https://example.com/chart.png"),
                summary: String::from("曲名 FUTURE"),
            }));
        let moved = relocate(outgoing);
        assert!(
            matches!(moved.parts.last(), Some(Part::Markdown(text)) if is_hint_line(text)),
            "提示行必须排在图片之后"
        );
    }

    #[test]
    fn case_031() {
        assert_eq!(normalize("/a score 骨折光"), "/a score");
        assert_eq!(normalize("  /a  b50   last "), "/a b50");
        assert_eq!(normalize("/a"), "/a");
    }
}
