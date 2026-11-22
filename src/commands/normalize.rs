use std::sync::LazyLock;

use crate::model::Region;

const RAW: [(&str, &str); 29] = [
    ("bind", "bind"),
    ("绑定", "bind"),
    ("char", "char"),
    ("立绘", "char"),
    ("角色", "char"),
    ("banner", "banner"),
    ("横幅", "banner"),
    ("名片", "banner"),
    ("score", "score"),
    ("单曲", "score"),
    ("送", "score"),
    ("song", "song"),
    ("曲目", "song"),
    ("详情", "song"),
    ("info", "song"),
    ("alias", "alias"),
    ("别名", "alias"),
    ("ill", "ill"),
    ("曲绘", "ill"),
    ("看曲绘", "ill"),
    ("查看曲绘", "ill"),
    ("play", "play"),
    ("听", "play"),
    ("点歌", "play"),
    ("播放", "play"),
    ("music", "play"),
    ("rand", "rand"),
    ("随机", "rand"),
    ("随机曲目", "rand"),
];

const RAW_TAIL: [(&str, &str); 6] = [
    ("search", "search"),
    ("检索", "search"),
    ("搜索", "search"),
    ("chart", "chart"),
    ("谱面", "chart"),
    ("预览", "chart"),
];

const RAW_LAST: [(&str, &str); 2] = [("cal", "cal"), ("计算", "cal")];

const RAW_EXTRA: [(&str, &str); 11] = [
    ("theme", "theme"),
    ("主题", "theme"),
    ("guess", "guess"),
    ("guesssong", "guess"),
    ("猜曲绘", "guess"),
    ("猜歌", "guess"),
    ("updb", "updb"),
    ("更新数据库", "updb"),
    ("更新曲库", "updb"),
    ("maint", "maint"),
    ("维护", "maint"),
];

const RAW_ANALYSIS: [(&str, &str); 32] = [
    ("list", "list"),
    ("清单", "list"),
    ("筛选", "list"),
    ("table", "table"),
    ("定数表", "table"),
    ("const", "table"),
    ("rating", "table"),
    ("ahv", "ahv"),
    ("成就", "ahv"),
    ("achievement", "ahv"),
    ("comp", "ahv"),
    ("stats", "stats"),
    ("统计", "stats"),
    ("区间", "stats"),
    ("suggest", "suggest"),
    ("push", "suggest"),
    ("up", "suggest"),
    ("推分", "suggest"),
    ("推分建议", "suggest"),
    ("chap", "chap"),
    ("章节", "chap"),
    ("曲包", "chap"),
    ("pack", "chap"),
    ("trend", "trend"),
    ("趋势", "trend"),
    ("ptt", "trend"),
    ("his", "his"),
    ("历史", "his"),
    ("record", "his"),
    ("diff", "diff"),
    ("变化", "diff"),
    ("变动", "diff"),
];

static KEYS: LazyLock<Vec<(&'static str, &'static str)>> = LazyLock::new(|| {
    let mut keys: Vec<(&'static str, &'static str)> = Vec::with_capacity(77);
    keys.extend(RAW);
    keys.extend(RAW_TAIL);
    keys.extend(RAW_LAST);
    keys.extend(RAW_ANALYSIS);
    keys.extend(RAW_EXTRA);
    keys.sort_by_key(|entry| std::cmp::Reverse(entry.0.len()));
    keys
});

pub fn is_help(text: &str) -> bool {
    let text = text.trim();
    text == "help" || text == "帮助"
}

pub fn route(region: Region, text: &str) -> Option<(&'static str, String)> {
    if let Some(hit) = scan(text) {
        return Some(hit);
    }
    if !region.is_cn() {
        return None;
    }
    if text == "l" {
        return Some(("cal", String::new()));
    }
    if let Some(rest) = text.strip_prefix("l ") {
        return Some(("cal", rest.trim().to_string()));
    }
    if text.starts_with('c') {
        if let Some(rest) = text.get(1..) {
            return scan(rest);
        }
    }
    None
}

fn scan(text: &str) -> Option<(&'static str, String)> {
    for &(key, name) in KEYS.iter() {
        let Some(rest) = text.strip_prefix(key) else {
            continue;
        };
        if rest.is_empty() || rest.contains(' ') || rest.contains('\t') {
            continue;
        }
        return Some((name, rest.to_string()));
    }
    None
}
