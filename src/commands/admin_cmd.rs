use aurorix_plugin::{Outgoing, PluginError};

use crate::backend::{ArcDataChange, ArcDataCounts, ArcDataSync, BackendError, UpstreamState};
use crate::commands::{reply, Ctx};

pub const ID_PREVIEW: usize = 8;

const DRY_WORDS: [&str; 4] = ["d", "-d", "dry", "试运行"];

pub fn dry_of(arg: &str) -> bool {
    arg.split(|value: char| value.is_whitespace() || value == ',')
        .map(str::to_lowercase)
        .any(|word| DRY_WORDS.contains(&word.as_str()))
}

const MAINT_USAGE: &str = "用法：/a maint on|off\non：闭闸 —— 所有要打 Arcaea 的能力都停（号池扫描、探针、国服授权与刷新），本地图鉴照常\noff：开闸 —— 立刻恢复常规调度\n不写参数：报当前状态";

pub enum Maint {
    Status,
    On,
    Off,
    Invalid,
}

pub fn switch_of(text: &str) -> Maint {
    match text.trim().to_lowercase().as_str() {
        "" | "status" => Maint::Status,
        "on" => Maint::On,
        "off" => Maint::Off,
        _ => Maint::Invalid,
    }
}

fn total_of(counts: &ArcDataCounts) -> i64 {
    counts.songs + counts.charts + counts.aliases
}

fn change_line(label: &str, change: &ArcDataChange) -> String {
    if change.ids.is_empty() {
        return String::new();
    }
    let shown: Vec<&str> = change
        .ids
        .iter()
        .take(ID_PREVIEW)
        .map(String::as_str)
        .collect();
    let total = change.total.max(change.ids.len() as i64);
    if total > shown.len() as i64 {
        format!("{label} {total} 条：{}…", shown.join("、"))
    } else {
        format!("{label} {} 条：{}", shown.len(), shown.join("、"))
    }
}

pub fn sync_summary(sync: &ArcDataSync, dry_run: bool) -> String {
    let prefix = if dry_run { "[试运行] " } else { "" };
    if !sync.changed {
        return format!("{prefix}曲库已是最新（歌曲·谱面·别名）");
    }
    let mut text = format!(
        "{prefix}曲库已更新：{}→{}（歌曲 {}→{} · 谱面 {}→{} · 别名 {}→{}）",
        total_of(&sync.before),
        total_of(&sync.after),
        sync.before.songs,
        sync.after.songs,
        sync.before.charts,
        sync.after.charts,
        sync.before.aliases,
        sync.after.aliases
    );
    for line in [
        change_line("新增", &sync.added),
        change_line("移除", &sync.removed),
    ] {
        if !line.is_empty() {
            text.push('\n');
            text.push_str(line.as_str());
        }
    }
    text
}

pub fn gate_text(state: &UpstreamState) -> String {
    format!(
        "上游当前：{}（原因：{}）",
        if state.closed { "已关闭" } else { "正常" },
        state.reason
    )
}

pub async fn updb(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    run_sync(ctx, dry_of(arg)).await
}

pub async fn updb_dry(ctx: &Ctx<'_>) -> Result<Option<Outgoing>, PluginError> {
    run_sync(ctx, true).await
}

const SYNC_ACK: &str = "曲库同步已开始：正在拉取歌曲·谱面·别名（大约一分钟），结果一到就发在这里";
const SYNC_ACK_DRY: &str = "曲库同步已开始（试运行，不落库）：正在比对歌曲·谱面·别名（大约一分钟），结果一到就发在这里";
const SYNC_SLOT: &str = "arcaea:updb:ack";

pub fn sync_ack(dry_run: bool) -> &'static str {
    match dry_run {
        true => SYNC_ACK_DRY,
        false => SYNC_ACK,
    }
}

fn sync_result(result: Result<ArcDataSync, BackendError>, dry_run: bool) -> Outgoing {
    match result {
        Ok(sync) => reply::plain(sync_summary(&sync, dry_run)),
        Err(error) => reply::fail(error.user_message()),
    }
}

async fn run_sync(ctx: &Ctx<'_>, dry_run: bool) -> Result<Option<Outgoing>, PluginError> {
    let Some(sender) = ctx.inv.opt_sender.clone() else {
        return Ok(Some(sync_result(
            ctx.backend.sync_catalog(dry_run).await,
            dry_run,
        )));
    };
    let backend = ctx.backend.clone();
    tokio::spawn(async move {
        let out = sync_result(backend.sync_catalog(dry_run).await, dry_run).supersede(SYNC_SLOT);
        let _ = sender.send(out).await;
    });
    Ok(Some(reply::notice(sync_ack(dry_run)).supersede(SYNC_SLOT)))
}

pub async fn maint(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    let (closed, reason) = match switch_of(arg) {
        Maint::Status => {
            return match ctx.backend.upstream_state().await {
                Ok(state) => Ok(Some(reply::plain(gate_text(&state)))),
                Err(error) => Ok(Some(reply::fail(error.user_message()))),
            };
        }
        Maint::On => (true, "官方维护"),
        Maint::Off => (false, ""),
        Maint::Invalid => return Ok(Some(reply::notice(MAINT_USAGE))),
    };
    match ctx.backend.set_upstream(closed, reason).await {
        Ok(state) => Ok(Some(reply::plain(gate_text(&state)))),
        Err(error) => Ok(Some(reply::fail(error.user_message()))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_077() {
        assert!(matches!(switch_of("on"), Maint::On));
        assert!(matches!(switch_of("ON"), Maint::On));
        assert!(matches!(switch_of(" on "), Maint::On));
        assert!(matches!(switch_of("off"), Maint::Off));
        assert!(matches!(switch_of("OFF"), Maint::Off));
        assert!(matches!(switch_of(""), Maint::Status));
        assert!(matches!(switch_of("status"), Maint::Status));
        assert!(matches!(switch_of("开服"), Maint::Invalid));
        assert!(matches!(switch_of("关闭"), Maint::Invalid));
        assert!(matches!(switch_of("维护中"), Maint::Invalid));
    }

    #[test]
    fn case_078() {
        assert!(dry_of("d"));
        assert!(dry_of("-d"));
        assert!(dry_of(" Dry "));
        assert!(dry_of("d,now"));
        assert!(!dry_of(""));
        assert!(!dry_of("go"));
        assert_eq!(sync_ack(true), SYNC_ACK_DRY);
        assert_eq!(sync_ack(false), SYNC_ACK);
        assert!(sync_ack(true).contains("试运行"));
        assert!(sync_ack(false).contains("曲库同步已开始"));
    }

    #[test]
    fn case_079() {
        let mut sync = ArcDataSync::default();
        assert_eq!(sync_summary(&sync, false), "曲库已是最新（歌曲·谱面·别名）");
        assert_eq!(
            sync_summary(&sync, true),
            "[试运行] 曲库已是最新（歌曲·谱面·别名）"
        );
        sync.changed = true;
        sync.before = ArcDataCounts {
            songs: 10,
            charts: 20,
            aliases: 5,
        };
        sync.after = ArcDataCounts {
            songs: 12,
            charts: 22,
            aliases: 5,
        };
        sync.added = ArcDataChange {
            total: 4,
            ids: vec!["a".to_string(), "b".to_string()],
            note: String::new(),
        };
        let text = sync_summary(&sync, true);
        assert!(text.starts_with("[试运行] 曲库已更新：35→39"), "实际：{text}");
        assert!(text.contains("歌曲 10→12"), "实际：{text}");
        assert!(text.contains("新增 4 条：a、b…"), "实际：{text}");
    }

    #[test]
    fn case_080() {
        let open = UpstreamState::default();
        assert_eq!(gate_text(&open), "上游当前：正常（原因：）");
        let closed = UpstreamState {
            closed: true,
            reason: "维护".to_string(),
            ..UpstreamState::default()
        };
        assert_eq!(gate_text(&closed), "上游当前：已关闭（原因：维护）");
    }
}
