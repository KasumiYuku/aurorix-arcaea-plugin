use std::cell::Cell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

use aurorix_render::{Color, FontChain, FontWeight, TextStyle};

use crate::model::{Difficulty, Grade};

pub const FONT_TEXT: &str = "L2-Regular";

pub const FONT_DISPLAY: &str = "L2-Semibold";

pub const FONT_LIGHT: &str = "L2-Light";

pub const FONT_NUMBER: &str = "Exo-Semibold";

pub const FONT_LATIN: &str = "Exo-Medium";

pub const FONT_SCORE: &str = "GeosansLight";

pub const FONT_FALLBACK: &str = "NotoSansCJK-Regular#2";

pub const FONT_TITLE: &str = FONT_NUMBER;

pub const FONT_NARROW: &str = FONT_TEXT;

pub const FONT_NARROW_BOLD: &str = FONT_DISPLAY;

pub const FONT_BODY: &str = FONT_TEXT;

pub const FONT_BOLD: &str = FONT_DISPLAY;

pub const FONT_CJK: &str = FONT_FALLBACK;

pub const FONT_SOFT: &str = FONT_SCORE;

pub const DISPLAY_INK_SCALE: f32 = 0.93;

pub const CARD_RADIUS: f32 = 18.0;

pub const ROW_RADIUS: f32 = 12.0;

pub const SLOT_RADIUS: f32 = 10.0;

pub const THEME_DIR: &str = "themes";

pub const THEME_MANIFEST: &str = "theme.json";

pub const OFFICIAL_LABEL: &str = "默认主题";

pub const OFFICIAL_BACKDROP: &str = "background/background1.jpg";

pub const OFFICIAL_COUNT_X: f32 = 700.0;

pub const OFFICIAL_BADGE_PURE: Badge = Badge {
    w: 249.0,
    h: 45.0,
    cx: 0.0,
};

pub const OFFICIAL_BADGE_FAR: Badge = Badge {
    w: 248.0,
    h: 43.0,
    cx: 0.0,
};

pub const OFFICIAL_BADGE_LOST: Badge = Badge {
    w: 248.0,
    h: 43.0,
    cx: 0.0,
};

pub const OFFICIAL_PANEL_INK: RectF = RectF {
    x0: 20.0,
    x1: 261.0,
    y0: 156.0,
    y1: 209.0,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemePage {
    Score,
}

impl ThemePage {
    pub const ALL: [ThemePage; 1] = [ThemePage::Score];

    pub fn name(self) -> &'static str {
        match self {
            ThemePage::Score => "score",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ThemePage::Score => "单曲成绩页",
        }
    }

    pub fn parse(value: &str) -> Option<ThemePage> {
        let wanted = value.trim().to_ascii_lowercase();
        ThemePage::ALL
            .into_iter()
            .find(|page| page.name() == wanted)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RectF {
    pub x0: f32,
    pub x1: f32,
    pub y0: f32,
    pub y1: f32,
}

impl RectF {
    pub fn is_zero(self) -> bool {
        self == RectF::default()
    }

    pub fn tuple(self) -> (f64, f64, f64, f64) {
        (
            f64::from(self.x0),
            f64::from(self.x1),
            f64::from(self.y0),
            f64::from(self.y1),
        )
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Badge {
    pub w: f32,
    pub h: f32,
    pub cx: f32,
}

impl Badge {
    pub fn is_zero(self) -> bool {
        self.w <= 0.0 || self.h <= 0.0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CountStyle {
    pub x: f32,
    pub dy: f32,
    pub right: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JudgeStyle {
    pub label_from_art: bool,
    pub badge_pure: Badge,
    pub badge_far: Badge,
    pub badge_lost: Badge,
    pub ink: RectF,
    pub count: CountStyle,
    pub label_right: f32,
}

impl JudgeStyle {
    pub const OFFICIAL: JudgeStyle = JudgeStyle {
        label_from_art: false,
        badge_pure: OFFICIAL_BADGE_PURE,
        badge_far: OFFICIAL_BADGE_FAR,
        badge_lost: OFFICIAL_BADGE_LOST,
        ink: RectF {
            x0: 0.0,
            x1: 0.0,
            y0: 0.0,
            y1: 0.0,
        },
        count: CountStyle {
            x: 0.0,
            dy: 0.0,
            right: 0.0,
        },
        label_right: 0.0,
    };

    pub fn badge(&self, key: &str) -> Badge {
        let declared = match key {
            "far" => self.badge_far,
            "lost" => self.badge_lost,
            _ => self.badge_pure,
        };
        if declared.is_zero() {
            official_badge(key)
        } else {
            declared
        }
    }

    pub fn count_x(&self) -> f32 {
        match self.count.x > 0.0 {
            true => self.count.x,
            false => OFFICIAL_COUNT_X,
        }
    }

    pub fn count_dy(&self) -> f32 {
        self.count.dy
    }

    pub fn count_right(&self) -> Option<f32> {
        (self.count.right > 0.0).then_some(self.count.right)
    }

    pub fn label_right(&self, fallback: f64) -> f64 {
        match self.label_right > 0.0 {
            true => f64::from(self.label_right),
            false => fallback,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanelStyle {
    pub ink: RectF,
    pub word_dx: f32,
}

impl PanelStyle {
    pub const OFFICIAL: PanelStyle = PanelStyle {
        ink: RectF {
            x0: 0.0,
            x1: 0.0,
            y0: 0.0,
            y1: 0.0,
        },
        word_dx: 0.0,
    };

    pub fn has_custom_ink(self) -> bool {
        !self.ink.is_zero()
    }

    pub fn ink(self) -> (f64, f64, f64, f64) {
        match self.has_custom_ink() {
            true => self.ink.tuple(),
            false => OFFICIAL_PANEL_INK.tuple(),
        }
    }

    pub fn word_dx(self) -> f64 {
        f64::from(self.word_dx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScoreStyle {
    pub judge: JudgeStyle,
    pub panel: PanelStyle,
}

impl ScoreStyle {
    pub const OFFICIAL: ScoreStyle = ScoreStyle {
        judge: JudgeStyle::OFFICIAL,
        panel: PanelStyle::OFFICIAL,
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub panel: Color,
    pub panel_alt: Color,
    pub slot: Color,
    pub stripe: Color,
    pub text: Color,
    pub muted: Color,
    pub accent: Color,
    pub divider: Color,
}

impl Palette {
    pub const OFFICIAL: Palette = Palette {
        panel: Color::new(18, 22, 38, 214),
        panel_alt: Color::new(30, 36, 58, 196),
        slot: Color::new(28, 33, 52, 168),
        stripe: Color::new(255, 255, 255, 18),
        text: Color::rgb(240, 244, 255),
        muted: Color::rgb(158, 168, 196),
        accent: Color::rgb(120, 208, 255),
        divider: Color::new(255, 255, 255, 26),
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    pub name: &'static str,
    pub label: &'static str,
    pub pages: &'static [ThemePage],
    pub backdrop: &'static str,
    pub backdrop_fallback: &'static str,
    pub panel: Color,
    pub panel_alt: Color,
    pub slot: Color,
    pub stripe: Color,
    pub text: Color,
    pub muted: Color,
    pub accent: Color,
    pub divider: Color,
    pub score: ScoreStyle,
    pub font_display: &'static str,
    pub font_text: &'static str,
}

impl Theme {
    pub const DEFAULT: Theme = Theme {
        name: "default",
        label: OFFICIAL_LABEL,
        pages: &[],
        backdrop: OFFICIAL_BACKDROP,
        backdrop_fallback: OFFICIAL_BACKDROP,
        panel: Palette::OFFICIAL.panel,
        panel_alt: Palette::OFFICIAL.panel_alt,
        slot: Palette::OFFICIAL.slot,
        stripe: Palette::OFFICIAL.stripe,
        text: Palette::OFFICIAL.text,
        muted: Palette::OFFICIAL.muted,
        accent: Palette::OFFICIAL.accent,
        divider: Palette::OFFICIAL.divider,
        score: ScoreStyle::OFFICIAL,
        font_display: "",
        font_text: "",
    };

    pub fn all() -> Vec<Theme> {
        with_registry(|registry| registry.list.clone()).unwrap_or_default()
    }

    pub fn by_name(name: &str) -> Theme {
        let wanted = name.trim();
        if wanted.is_empty() {
            return Theme::DEFAULT;
        }
        with_registry(|registry| registry.find(wanted))
            .flatten()
            .unwrap_or(Theme::DEFAULT)
    }

    pub fn is_official(self) -> bool {
        self.name == Theme::DEFAULT.name
    }

    pub fn covers(self, page: ThemePage) -> bool {
        self.pages.is_empty() || self.pages.contains(&page)
    }

    pub fn asset(self, page: ThemePage, rel: &str) -> String {
        match self.is_official() || !self.covers(page) {
            true => rel.to_string(),
            false => format!("{THEME_DIR}/{}/{rel}", self.name),
        }
    }

    pub fn family(self, family: &'static str) -> &'static str {
        let declared = match family == FONT_TEXT {
            true => self.font_text,
            false => match family == FONT_DISPLAY {
                true => self.font_display,
                false => "",
            },
        };
        match declared.is_empty() {
            true => family,
            false => declared,
        }
    }

    pub fn row_fill(self, index: usize) -> Color {
        if index.is_multiple_of(2) {
            self.panel_alt
        } else {
            self.panel
        }
    }
}

fn official_badge(key: &str) -> Badge {
    match key {
        "far" => OFFICIAL_BADGE_FAR,
        "lost" => OFFICIAL_BADGE_LOST,
        _ => OFFICIAL_BADGE_PURE,
    }
}

struct Registry {
    list: Vec<Theme>,
    by_name: HashMap<String, Theme>,
}

impl Registry {
    fn load(root: &Path) -> Registry {
        let mut list = Vec::new();
        let mut by_name = HashMap::new();
        let mut names: Vec<String> = std::fs::read_dir(root.join(THEME_DIR))
            .map(|entries| {
                entries
                    .flatten()
                    .filter(|entry| entry.path().is_dir())
                    .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        for name in names {
            let Some(theme) = load_theme(root, name.as_str()) else {
                continue;
            };
            by_name.insert(name.to_ascii_lowercase(), theme);
            for alias in aliases_of(root, name.as_str()) {
                by_name.insert(alias.to_ascii_lowercase(), theme);
            }
            by_name
                .entry(theme.label.to_ascii_lowercase())
                .or_insert(theme);
            list.push(theme);
        }
        Registry { list, by_name }
    }

    fn find(&self, name: &str) -> Option<Theme> {
        self.by_name.get(&name.to_ascii_lowercase()).copied()
    }
}

static REGISTRY: LazyLock<Mutex<Option<Registry>>> = LazyLock::new(|| Mutex::new(None));

fn with_registry<R>(body: impl FnOnce(&Registry) -> R) -> Option<R> {
    let mut guard = REGISTRY
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if guard.is_none() {
        *guard = Some(Registry::load(&crate::assets_root()));
    }
    guard.as_ref().map(body)
}

pub fn reload() {
    let mut guard = REGISTRY
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *guard = None;
}

fn load_theme(root: &Path, name: &str) -> Option<Theme> {
    let dir = root.join(THEME_DIR).join(name);
    let body = std::fs::read_to_string(dir.join(THEME_MANIFEST)).ok();
    let manifest = body
        .as_deref()
        .and_then(|raw| serde_json::from_str::<Manifest>(raw).ok())
        .unwrap_or_default();
    let mut pages: Vec<ThemePage> = manifest
        .pages
        .iter()
        .filter_map(|page| ThemePage::parse(page.as_str()))
        .collect();
    pages.dedup();
    let pages: Vec<ThemePage> = match pages.is_empty() {
        true => ThemePage::ALL.to_vec(),
        false => pages,
    };
    let pages: &'static [ThemePage] = Box::leak(pages.into_boxed_slice());
    let label = match manifest.label.trim().is_empty() {
        true => name.to_string(),
        false => manifest.label.trim().to_string(),
    };
    let covers_score = pages.is_empty() || pages.contains(&ThemePage::Score);
    let themed_backdrop = format!("{THEME_DIR}/{name}/background/score.png");
    let backdrop = match covers_score && dir.join("background/score.png").is_file() {
        true => leak(themed_backdrop),
        false => OFFICIAL_BACKDROP,
    };
    let palette = manifest.palette.unwrap_or_default().resolve();
    let score = manifest
        .score
        .map(ManifestScore::resolve)
        .unwrap_or(ScoreStyle::OFFICIAL);
    let fonts = manifest.fonts.unwrap_or_default();
    let font_display = register_theme_font(dir.as_path(), name, "display", fonts.display.as_str());
    let font_text = register_theme_font(dir.as_path(), name, "text", fonts.text.as_str());
    Some(Theme {
        name: leak(name.to_string()),
        label: leak(label),
        pages,
        backdrop,
        backdrop_fallback: OFFICIAL_BACKDROP,
        panel: palette.panel,
        panel_alt: palette.panel_alt,
        slot: palette.slot,
        stripe: palette.stripe,
        text: palette.text,
        muted: palette.muted,
        accent: palette.accent,
        divider: palette.divider,
        score,
        font_display,
        font_text,
    })
}

fn register_theme_font(dir: &Path, name: &str, role: &str, decl: &str) -> &'static str {
    let decl = decl.trim();
    if decl.is_empty() {
        return "";
    }
    let path = match decl.contains('/') {
        true => dir.join(decl),
        false => dir.join("fonts").join(decl),
    };
    let Ok(bytes) = std::fs::read(path) else {
        return "";
    };
    let alias = format!("theme.{name}.{role}");
    match aurorix_render::register_font_family(alias.as_str(), bytes, None) {
        Ok(()) => leak(alias),
        Err(_) => "",
    }
}

fn aliases_of(root: &Path, name: &str) -> Vec<String> {
    let path = root.join(THEME_DIR).join(name).join(THEME_MANIFEST);
    let Ok(body) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(body.as_str()) else {
        return Vec::new();
    };
    value
        .get("aliases")
        .and_then(|item| item.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn leak(value: String) -> &'static str {
    if value.is_empty() {
        return "";
    }
    Box::leak(value.into_boxed_str())
}

pub fn check(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    if root.join("skins").is_dir() {
        out.push(String::from(
            "skins/ 是改名前的目录, 现在是 themes/ (清单也从 skin.json 改名 theme.json)",
        ));
    }
    let mut names: Vec<String> = std::fs::read_dir(root.join(THEME_DIR))
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| entry.path().is_dir())
                .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    for name in names {
        out.extend(check_theme(root, name.as_str()));
    }
    out
}

fn check_theme(root: &Path, name: &str) -> Vec<String> {
    let dir = root.join(THEME_DIR).join(name);
    let at = format!("{THEME_DIR}/{name}");
    let files = count_files(&dir);
    if files == 0 {
        return vec![format!("{at} (主题目录是空的)")];
    }
    if files == 1 && dir.join(THEME_MANIFEST).is_file() {
        return vec![format!("{at} (只有清单, 没有一张素材)")];
    }
    let path = dir.join(THEME_MANIFEST);
    if !path.is_file() {
        return vec![format!("{at}/{THEME_MANIFEST} (没有清单: 这套主题只换素材, 改不了判定几何)")];
    }
    let Ok(body) = std::fs::read_to_string(&path) else {
        return vec![format!("{at}/{THEME_MANIFEST} (读不出来)")];
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(body.as_str()) else {
        return vec![format!("{at}/{THEME_MANIFEST} (清单不是合法 JSON)")];
    };
    let mut out = Vec::new();
    for key in unknown_fields(&value, &MANIFEST_KEYS) {
        out.push(format!("{at}/{THEME_MANIFEST} (多写的字段 \"{key}\")"));
    }
    if let Some(pages) = value.get("pages").and_then(|item| item.as_array()) {
        for page in pages.iter().filter_map(|item| item.as_str()) {
            if ThemePage::parse(page).is_none() {
                out.push(format!("{at}/{THEME_MANIFEST} (pages 里的 \"{page}\" 不是认得的页名)"));
            }
        }
    }
    if let Some(fonts) = value.get("fonts").and_then(|item| item.as_object()) {
        for key in unknown_fields(&serde_json::Value::Object(fonts.clone()), &FONT_KEYS) {
            out.push(format!("{at}/{THEME_MANIFEST} (fonts 里多写的字段 \"{key}\")"));
        }
        for (key, decl) in fonts {
            let Some(decl) = decl.as_str().filter(|item| !item.trim().is_empty()) else {
                continue;
            };
            let rel = match decl.contains('/') {
                true => decl.to_string(),
                false => format!("{THEME_DIR}/{name}/fonts/{decl}"),
            };
            if !root.join(rel.as_str()).is_file() {
                out.push(format!("{at}/{THEME_MANIFEST} (fonts.{key} 指向的 {rel} 不存在)"));
            }
        }
    }
    if let Some(score) = value.get("score").and_then(|item| item.as_object()) {
        for key in unknown_fields(&serde_json::Value::Object(score.clone()), &SCORE_KEYS) {
            out.push(format!("{at}/{THEME_MANIFEST} (score 里多写的字段 \"{key}\")"));
        }
        if let Some(judge) = score.get("judge").and_then(|item| item.as_object()) {
            for key in unknown_fields(&serde_json::Value::Object(judge.clone()), &JUDGE_KEYS) {
                out.push(format!("{at}/{THEME_MANIFEST} (score.judge 里多写的字段 \"{key}\")"));
            }
            if let Some(badge) = judge.get("badge").and_then(|item| item.as_object()) {
                for key in unknown_fields(&serde_json::Value::Object(badge.clone()), &BADGE_NAMES) {
                    out.push(format!(
                        "{at}/{THEME_MANIFEST} (score.judge.badge 里的 \"{key}\" 不是判定名: pure/far/lost)"
                    ));
                }
            }
        }
        if let Some(panel) = score.get("panel").and_then(|item| item.as_object()) {
            for key in unknown_fields(&serde_json::Value::Object(panel.clone()), &PANEL_KEYS) {
                out.push(format!("{at}/{THEME_MANIFEST} (score.panel 里多写的字段 \"{key}\")"));
            }
        }
    }
    if let Some(palette) = value.get("palette").and_then(|item| item.as_object()) {
        for key in unknown_fields(&serde_json::Value::Object(palette.clone()), &PALETTE_KEYS) {
            out.push(format!("{at}/{THEME_MANIFEST} (palette 里多写的字段 \"{key}\")"));
        }
        for (key, value) in palette {
            if value.is_null() {
                continue;
            }
            let text = value.as_str().unwrap_or_default();
            if parse_color(text).is_none() {
                out.push(format!(
                    "{at}/{THEME_MANIFEST} (palette.{key} 的 \"{text}\" 不是 #RRGGBB 或 #RRGGBBAA)"
                ));
            }
        }
    }
    out
}

const MANIFEST_KEYS: [&str; 6] = ["label", "aliases", "pages", "fonts", "score", "palette"];

const FONT_KEYS: [&str; 2] = ["display", "text"];

const SCORE_KEYS: [&str; 2] = ["judge", "panel"];

const JUDGE_KEYS: [&str; 5] = ["labelFromArt", "badge", "ink", "count", "labelRight"];

const BADGE_NAMES: [&str; 3] = ["pure", "far", "lost"];

const PANEL_KEYS: [&str; 2] = ["ink", "wordDX"];

const PALETTE_KEYS: [&str; 8] = [
    "panel",
    "panelAlt",
    "slot",
    "stripe",
    "text",
    "muted",
    "accent",
    "divider",
];

fn unknown_fields(value: &serde_json::Value, allowed: &[&str]) -> Vec<String> {
    let Some(map) = value.as_object() else {
        return Vec::new();
    };
    map.keys()
        .filter(|key| !allowed.iter().any(|item| item.eq_ignore_ascii_case(key)))
        .cloned()
        .collect()
}

fn count_files(dir: &Path) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| match entry.path().is_dir() {
            true => count_files(entry.path().as_path()),
            false => 1,
        })
        .sum()
}

#[derive(Debug, Default, serde::Deserialize)]
struct Manifest {
    #[serde(default)]
    label: String,
    #[serde(default)]
    pages: Vec<String>,
    #[serde(default)]
    fonts: Option<ManifestFonts>,
    #[serde(default)]
    score: Option<ManifestScore>,
    #[serde(default)]
    palette: Option<ManifestPalette>,
}

#[derive(Debug, Default, serde::Deserialize)]
struct ManifestFonts {
    #[serde(default)]
    display: String,
    #[serde(default)]
    text: String,
}

#[derive(Debug, Default, serde::Deserialize)]
struct ManifestScore {
    #[serde(default)]
    judge: Option<ManifestJudge>,
    #[serde(default)]
    panel: Option<ManifestPanel>,
}

impl ManifestScore {
    fn resolve(self) -> ScoreStyle {
        let judge = self.judge.unwrap_or_default();
        let panel = self.panel.unwrap_or_default();
        ScoreStyle {
            judge: JudgeStyle {
                label_from_art: judge.label_from_art,
                badge_pure: judge.badge("pure"),
                badge_far: judge.badge("far"),
                badge_lost: judge.badge("lost"),
                ink: judge.ink.unwrap_or_default().into(),
                count: CountStyle {
                    x: judge.count.x,
                    dy: judge.count.dy,
                    right: judge.count.right,
                },
                label_right: judge.label_right,
            },
            panel: PanelStyle {
                ink: panel.ink.unwrap_or_default().into(),
                word_dx: panel.word_dx,
            },
        }
    }
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestJudge {
    #[serde(default)]
    label_from_art: bool,
    #[serde(default)]
    badge: HashMap<String, ManifestBadge>,
    #[serde(default)]
    ink: Option<ManifestRect>,
    #[serde(default)]
    count: ManifestCount,
    #[serde(default)]
    label_right: f32,
}

impl ManifestJudge {
    fn badge(&self, key: &str) -> Badge {
        self.badge
            .get(key)
            .map(|badge| Badge {
                w: badge.w,
                h: badge.h,
                cx: badge.cx,
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Default, serde::Deserialize)]
struct ManifestBadge {
    #[serde(default)]
    w: f32,
    #[serde(default)]
    h: f32,
    #[serde(default)]
    cx: f32,
}

#[derive(Debug, Default, serde::Deserialize)]
struct ManifestCount {
    #[serde(default)]
    x: f32,
    #[serde(default)]
    dy: f32,
    #[serde(default)]
    right: f32,
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestPanel {
    #[serde(default)]
    ink: Option<ManifestRect>,
    #[serde(default, rename = "wordDX")]
    word_dx: f32,
}

#[derive(Debug, Default, serde::Deserialize)]
struct ManifestRect {
    #[serde(default)]
    x0: f32,
    #[serde(default)]
    x1: f32,
    #[serde(default)]
    y0: f32,
    #[serde(default)]
    y1: f32,
}

impl From<ManifestRect> for RectF {
    fn from(value: ManifestRect) -> RectF {
        RectF {
            x0: value.x0,
            x1: value.x1,
            y0: value.y0,
            y1: value.y1,
        }
    }
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestPalette {
    #[serde(default)]
    panel: Option<String>,
    #[serde(default)]
    panel_alt: Option<String>,
    #[serde(default)]
    slot: Option<String>,
    #[serde(default)]
    stripe: Option<String>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    muted: Option<String>,
    #[serde(default)]
    accent: Option<String>,
    #[serde(default)]
    divider: Option<String>,
}

impl ManifestPalette {
    fn resolve(self) -> Palette {
        let official = Palette::OFFICIAL;
        Palette {
            panel: parse_color(self.panel.as_deref().unwrap_or_default()).unwrap_or(official.panel),
            panel_alt: parse_color(self.panel_alt.as_deref().unwrap_or_default())
                .unwrap_or(official.panel_alt),
            slot: parse_color(self.slot.as_deref().unwrap_or_default()).unwrap_or(official.slot),
            stripe: parse_color(self.stripe.as_deref().unwrap_or_default())
                .unwrap_or(official.stripe),
            text: parse_color(self.text.as_deref().unwrap_or_default()).unwrap_or(official.text),
            muted: parse_color(self.muted.as_deref().unwrap_or_default()).unwrap_or(official.muted),
            accent: parse_color(self.accent.as_deref().unwrap_or_default())
                .unwrap_or(official.accent),
            divider: parse_color(self.divider.as_deref().unwrap_or_default())
                .unwrap_or(official.divider),
        }
    }
}

pub fn parse_color(value: &str) -> Option<Color> {
    let text = value.trim().trim_start_matches('#');
    if !text.chars().all(|item| item.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |index: usize| -> Option<u8> {
        text.get(index..index + 2)
            .and_then(|pair| u8::from_str_radix(pair, 16).ok())
    };
    match text.len() {
        6 => Some(Color::new(byte(0)?, byte(2)?, byte(4)?, 255)),
        8 => Some(Color::new(byte(0)?, byte(2)?, byte(4)?, byte(6)?)),
        _ => None,
    }
}

thread_local! {
    static CURRENT: Cell<Option<Theme>> = const { Cell::new(None) };
}

pub struct ThemeScope(Option<Theme>);

impl Drop for ThemeScope {
    fn drop(&mut self) {
        CURRENT.with(|cell| cell.set(self.0));
    }
}

pub fn enter(theme: Theme) -> ThemeScope {
    let previous = CURRENT.with(|cell| cell.replace(Some(theme)));
    ThemeScope(previous)
}

pub fn current() -> Theme {
    CURRENT.with(|cell| cell.get()).unwrap_or(Theme::DEFAULT)
}

pub fn family(name: &'static str) -> &'static str {
    current().family(name)
}

pub fn registry_root() -> PathBuf {
    crate::assets_root().join(THEME_DIR)
}

pub fn difficulty_color(difficulty: Difficulty) -> Color {
    match difficulty {
        Difficulty::Past => Color::rgb(83, 181, 226),
        Difficulty::Present => Color::rgb(90, 208, 122),
        Difficulty::Future => Color::rgb(193, 97, 226),
        Difficulty::Beyond => Color::rgb(226, 85, 92),
        Difficulty::Eternal => Color::rgb(154, 167, 196),
        Difficulty::Ins => Color::rgb(116, 136, 215),
    }
}

pub fn difficulty_deep(difficulty: Difficulty) -> Color {
    let base = difficulty_color(difficulty);
    Color::new(
        (base.r as u16 * 62 / 100) as u8,
        (base.g as u16 * 62 / 100) as u8,
        (base.b as u16 * 62 / 100) as u8,
        255,
    )
}

pub fn grade_color(grade: Grade) -> Color {
    match grade {
        Grade::Pm => Color::rgb(255, 226, 120),
        Grade::Ex => Color::rgb(226, 178, 84),
        Grade::Aa => Color::rgb(154, 216, 240),
        Grade::A => Color::rgb(127, 209, 138),
        Grade::B => Color::rgb(143, 168, 216),
        Grade::C => Color::rgb(201, 168, 143),
        Grade::D => Color::rgb(176, 176, 176),
        Grade::Fr => Color::rgb(190, 160, 255),
        Grade::ExPlus => Color::rgb(242, 201, 91),
    }
}

pub fn clear_color(clear_type: Option<i32>) -> Color {
    match clear_type.unwrap_or(-1) {
        3 => Color::rgb(255, 224, 130),
        2 => Color::rgb(160, 230, 200),
        1 => Color::rgb(150, 190, 255),
        0 => Color::rgb(200, 200, 210),
        _ => Color::rgb(150, 132, 132),
    }
}

pub fn chain() -> FontChain {
    FontChain::new(FONT_TEXT).with_fallbacks([
        FONT_FALLBACK,
        FONT_DISPLAY,
        FONT_NUMBER,
        FONT_LATIN,
        FONT_LIGHT,
        FONT_SCORE,
    ])
}

pub fn style(family: &str, size: u32, color: Color) -> TextStyle {
    TextStyle::new(family, size).with_color(color)
}

pub fn title_style(size: u32, color: Color) -> TextStyle {
    style(FONT_TITLE, size, color).with_weight(FontWeight::SemiBold)
}

pub fn body_style(size: u32, color: Color) -> TextStyle {
    style(FONT_BODY, size, color)
}

pub fn bold_style(size: u32, color: Color) -> TextStyle {
    style(FONT_BOLD, size, color).with_weight(FontWeight::Bold)
}

pub fn number_style(size: u32, color: Color) -> TextStyle {
    style(FONT_NUMBER, size, color)
}

pub fn narrow_style(size: u32, color: Color) -> TextStyle {
    style(FONT_NARROW, size, color)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_091() {
        assert_eq!(Theme::by_name("arknights").name, "arknights");
        assert_eq!(Theme::by_name("明日方舟").name, "arknights");
        assert_eq!(Theme::by_name("glacier").name, "default");
        assert_eq!(Theme::by_name("  DEFAULT ").name, "default");
    }

    #[test]
    fn case_092() {
        let arknights = Theme::by_name("arknights");
        assert!(arknights.backdrop.starts_with("themes/arknights/"));
        assert!(Theme::DEFAULT.backdrop.starts_with("background/"));
        assert_ne!(arknights.panel, Theme::DEFAULT.panel);
    }

    #[test]
    fn case_093() {
        let mut seen = Vec::new();
        for difficulty in Difficulty::ALL {
            let color = difficulty_color(difficulty);
            assert!(!seen.contains(&color), "{difficulty} 的颜色和别的难度撞了");
            let deep = difficulty_deep(difficulty);
            assert!(deep.r <= color.r && deep.g <= color.g && deep.b <= color.b);
            seen.push(color);
        }
    }

    #[test]
    fn case_094() {
        let official = ScoreStyle::OFFICIAL;
        assert_eq!(official.judge.count_x(), OFFICIAL_COUNT_X);
        assert_eq!(official.judge.badge("pure").w, OFFICIAL_BADGE_PURE.w);
        assert_eq!(official.judge.badge("far").h, OFFICIAL_BADGE_FAR.h);
        assert!(!official.panel.has_custom_ink());
        assert_eq!(official.panel.ink(), OFFICIAL_PANEL_INK.tuple());
        assert_eq!(official.panel.word_dx(), 0.0);
        assert_eq!(official.judge.label_right(629.0), 629.0);
    }

    #[test]
    fn case_095() {
        assert_eq!(parse_color("#112233"), Some(Color::new(17, 34, 51, 255)));
        assert_eq!(parse_color("11223344"), Some(Color::new(17, 34, 51, 68)));
        assert_eq!(parse_color("#12345"), None);
        assert_eq!(parse_color("zzzzzz"), None);
        assert_eq!(parse_color(""), None);
    }

    #[test]
    fn case_097() {
        let theme = Theme::by_name("arknights");
        assert_eq!(theme.panel, Color::new(14, 15, 17, 226));
        assert_eq!(theme.accent, Color::rgb(214, 60, 60));
        assert!(theme.score.judge.label_from_art);
        assert_eq!(theme.score.judge.count_x(), 709.0);
        assert_eq!(theme.score.judge.count_dy(), -3.0);
        assert_eq!(theme.score.judge.count_right(), Some(746.0));
        assert!(theme.score.panel.has_custom_ink());
        assert_eq!(theme.score.panel.ink(), (46.0, 232.0, 170.0, 198.0));
        assert_eq!(theme.score.panel.word_dx(), 12.0);
        assert_eq!(theme.score.judge.badge("pure").w, 367.0);
        assert_eq!(theme.score.judge.badge("far").h, 56.0);
    }

    #[test]
    fn case_098() {
        let root = std::env::temp_dir().join(format!("aurorix-theme-check-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("themes/broken")).expect("建目录");
        std::fs::write(root.join("themes/broken/theme.json"), "{\"lable\":\"x\"}").expect("写清单");
        let issues = check(root.as_path());
        assert!(
            issues.iter().any(|item| item.contains("只有清单")),
            "{issues:?}"
        );
        std::fs::write(root.join("themes/broken/a.png"), b"x").expect("写素材");
        let issues = check(root.as_path());
        assert!(
            issues.iter().any(|item| item.contains("多写的字段 \"lable\"")),
            "{issues:?}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn case_099() {
        let root = std::env::temp_dir().join(format!("aurorix-font-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("themes/probe/fonts")).expect("建目录");
        let font = std::fs::read_dir(crate::assets_root().join("fonts"))
            .expect("读字体目录")
            .flatten()
            .map(|entry| entry.path())
            .find(|path| {
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("ttf") || ext.eq_ignore_ascii_case("otf"))
                    .unwrap_or(false)
            })
            .expect("素材里要有一支字体");
        std::fs::copy(font.as_path(), root.join("themes/probe/fonts/probe.ttf")).expect("复制字体");
        std::fs::write(
            root.join("themes/probe/theme.json"),
            "{\"label\":\"Probe\",\"pages\":[\"score\"],\"fonts\":{\"display\":\"fonts/probe.ttf\",\"text\":\"probe.ttf\"}}",
        )
        .expect("写清单");
        let theme = load_theme(root.as_path(), "probe").expect("装载主题");
        assert_eq!(theme.font_display, "theme.probe.display");
        assert_eq!(theme.font_text, "theme.probe.text");
        assert!(aurorix_render::has_font_family("theme.probe.display"));
        {
            let _guard = enter(theme);
            assert_eq!(family(FONT_TEXT), "theme.probe.text");
            assert_eq!(family(FONT_DISPLAY), "theme.probe.display");
            assert_eq!(family(FONT_NUMBER), FONT_NUMBER);
        }
        assert_eq!(family(FONT_TEXT), FONT_TEXT);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn case_096() {
        let theme = Theme::DEFAULT;
        assert!(theme.is_official());
        assert!(theme.covers(ThemePage::Score));
        assert_eq!(theme.asset(ThemePage::Score, "result/judge/pure.png"), "result/judge/pure.png");
        let themed = Theme {
            name: "probe",
            pages: &[ThemePage::Score],
            ..Theme::DEFAULT
        };
        assert_eq!(
            themed.asset(ThemePage::Score, "result/judge/pure.png"),
            "themes/probe/result/judge/pure.png"
        );
        let limited = Theme {
            name: "probe",
            pages: &[],
            ..Theme::DEFAULT
        };
        assert!(limited.covers(ThemePage::Score));
    }
}
