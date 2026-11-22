use std::fmt;

use crate::model::Difficulty;

pub const SPAN_EPS: f64 = 1e-9;

pub const MIXED_HINT: &str =
    "等级和定数不能混写：等级用空格分开（8+ 9），定数用小数区间（8.0-9.9）";

pub fn fold_symbols(text: &str) -> String {
    text.chars()
        .map(|character| match character {
            '＞' | '﹥' => '>',
            '＜' | '﹤' => '<',
            '＝' => '=',
            '≧' | '⩾' => '≥',
            '≦' | '⩽' => '≤',
            '＋' => '+',
            '－' | '−' | '﹣' | '‑' | '–' | '—' | '―' => '-',
            '~' | '～' | '〜' | '∼' => '-',
            '．' => '.',
            '０' => '0',
            '１' => '1',
            '２' => '2',
            '３' => '3',
            '４' => '4',
            '５' => '5',
            '６' => '6',
            '７' => '7',
            '８' => '8',
            '９' => '9',
            '：' => ':',
            other => other,
        })
        .collect()
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Span {
    pub min: f64,
    pub max: f64,
    pub min_set: bool,
    pub max_set: bool,
    pub min_open: bool,
    pub max_open: bool,
}

impl Span {
    pub fn exactly(value: f64) -> Self {
        Self {
            min: value,
            max: value,
            min_set: true,
            max_set: true,
            min_open: false,
            max_open: false,
        }
    }

    pub fn range(min: f64, max: f64) -> Self {
        Self {
            min,
            max,
            min_set: true,
            max_set: true,
            min_open: false,
            max_open: false,
        }
    }

    pub fn at_least(value: f64) -> Self {
        Self {
            min: value,
            min_set: true,
            ..Self::default()
        }
    }

    pub fn at_most(value: f64) -> Self {
        Self {
            max: value,
            max_set: true,
            ..Self::default()
        }
    }

    pub fn band(low: f64) -> Self {
        Self::range(low, low + 0.9)
    }

    pub fn on(&self) -> bool {
        self.min_set || self.max_set
    }

    pub fn contains(&self, value: f64) -> bool {
        if self.min_set
            && (value < self.min - SPAN_EPS
                || (self.min_open && (value - self.min).abs() <= SPAN_EPS))
        {
            return false;
        }
        if self.max_set
            && (value > self.max + SPAN_EPS
                || (self.max_open && (value - self.max).abs() <= SPAN_EPS))
        {
            return false;
        }
        true
    }

    pub fn text(&self) -> String {
        if self.min_set && self.max_set && !self.min_open && !self.max_open {
            if (self.min - self.max).abs() <= SPAN_EPS {
                return trim_dot_zero(format!("{:.1}", self.min));
            }
            return format!("{:.1}-{:.1}", self.min, self.max);
        }
        let lower = if self.min_set {
            format!(
                "{}{}",
                symbol(self.min_open, ">", "≥"),
                trim_dot_zero(format!("{:.1}", self.min))
            )
        } else {
            String::new()
        };
        let upper = if self.max_set {
            format!(
                "{}{}",
                symbol(self.max_open, "<", "≤"),
                trim_dot_zero(format!("{:.1}", self.max))
            )
        } else {
            String::new()
        };
        format!("{lower} {upper}").trim().to_string()
    }

    pub fn count_text(&self) -> String {
        if self.min_set && self.max_set && !self.min_open && !self.max_open {
            if (self.min - self.max).abs() <= SPAN_EPS {
                return number(self.min);
            }
            return format!("{}-{}", number(self.min), number(self.max));
        }
        let lower = if self.min_set {
            format!("{}{}", symbol(self.min_open, ">", "≥"), number(self.min))
        } else {
            String::new()
        };
        let upper = if self.max_set {
            format!("{}{}", symbol(self.max_open, "<", "≤"), number(self.max))
        } else {
            String::new()
        };
        format!("{lower} {upper}").trim().to_string()
    }

    pub fn dsl(&self, field: &str) -> String {
        let mut parts = Vec::new();
        if self.min_set {
            parts.push(format!(
                "{field}{}{}",
                symbol(self.min_open, ">", ">="),
                number(self.min)
            ));
        }
        if self.max_set {
            parts.push(format!(
                "{field}{}{}",
                symbol(self.max_open, "<", "<="),
                number(self.max)
            ));
        }
        parts.join(" and ")
    }
}

fn symbol<'a>(open: bool, strict: &'a str, loose: &'a str) -> &'a str {
    if open {
        strict
    } else {
        loose
    }
}

fn trim_dot_zero(text: String) -> String {
    text.strip_suffix(".0").map(str::to_string).unwrap_or(text)
}

pub fn number(value: f64) -> String {
    if (value - value.round()).abs() <= SPAN_EPS && value.abs() <= i64::MAX as f64 {
        return format!("{}", value.round() as i64);
    }
    let mut text = format!("{value}");
    if text.contains('.') {
        while text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
    }
    text
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Level {
    pub int: u8,
    pub plus: bool,
}

impl Level {
    pub const MIN: u8 = 1;

    pub const MAX: u8 = 12;

    pub fn new(int: u8, plus: bool) -> Self {
        Self { int, plus }
    }

    pub fn parse(text: &str) -> Option<Self> {
        let folded = fold_symbols(text.trim());
        let plus = folded.ends_with('+');
        let digits = folded.strip_suffix('+').unwrap_or(&folded);
        if digits.is_empty() || digits.len() > 2 {
            return None;
        }
        if !digits.chars().all(|character| character.is_ascii_digit()) {
            return None;
        }
        let value: u8 = digits.parse().ok()?;
        if !(Level::MIN..=Level::MAX).contains(&value) {
            return None;
        }
        Some(Level::new(value, plus))
    }

    pub fn label(self) -> String {
        if self.plus {
            format!("{}+", self.int)
        } else {
            format!("{}", self.int)
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.label())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Filter {
    pub text: String,
    pub constant: Span,
    pub levels: Vec<Level>,
    pub diffs: Vec<Difficulty>,
    pub bpm: Span,
    pub note: Span,
    pub designer: String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FilterError {
    #[error("{}", MIXED_HINT)]
    等级与定数混写,
    #[error("同一个条件给了两次：{0}")]
    条件重复(&'static str),
    #[error("条件看不懂：{0}")]
    无法解析(String),
    #[error("关键字后面没有值：{0}")]
    缺值(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NumField {
    Constant,
    Bpm,
    Note,
}

impl NumField {
    fn label(self) -> &'static str {
        match self {
            NumField::Constant => "定数",
            NumField::Bpm => "bpm",
            NumField::Note => "物量",
        }
    }
}

const NUM_KEYWORDS: [(NumField, &[&str]); 3] = [
    (NumField::Bpm, &["bpm", "曲速", "速度"]),
    (NumField::Note, &["note", "combo", "cmb", "物量", "连击"]),
    (
        NumField::Constant,
        &[
            "constant",
            "const",
            "dif",
            "difficulty",
            "定数",
            "定级",
            "难度",
        ],
    ),
];

const TEXT_KEYWORDS: [&str; 4] = ["title", "曲名", "曲", "name"];

const DESIGNER_KEYWORDS: [&str; 5] = ["designer", "chart_designer", "谱师", "作曲", "chart"];

fn diff_name(text: &str) -> Option<Difficulty> {
    let lowered = text.trim().to_ascii_lowercase();
    let named: [(Difficulty, &[&str]); 6] = [
        (Difficulty::Past, &["pst", "past", "过去"]),
        (Difficulty::Present, &["prs", "present", "现在"]),
        (Difficulty::Future, &["ftr", "future", "未来"]),
        (Difficulty::Beyond, &["byd", "beyond", "超越"]),
        (Difficulty::Eternal, &["etr", "eternal", "永恒"]),
        (Difficulty::Ins, &["ins", "inscribed", "铭刻"]),
    ];
    for (difficulty, names) in named {
        if names.iter().any(|name| *name == lowered) {
            return Some(difficulty);
        }
    }
    None
}

fn glued_diff(token: &str) -> Option<(&str, Difficulty)> {
    let named: [(Difficulty, &[&str]); 6] = [
        (Difficulty::Past, &["pst", "past", "过去"]),
        (Difficulty::Present, &["prs", "present", "现在"]),
        (Difficulty::Future, &["ftr", "future", "未来"]),
        (Difficulty::Beyond, &["byd", "beyond", "超越"]),
        (Difficulty::Eternal, &["etr", "eternal", "永恒"]),
        (Difficulty::Ins, &["ins", "inscribed", "铭刻"]),
    ];
    for (difficulty, names) in named {
        for name in names {
            let Some(cut) = token.len().checked_sub(name.len()) else {
                continue;
            };
            if cut == 0 || !token.is_char_boundary(cut) {
                continue;
            }
            let tail = &token[cut..];
            let hit = match name.is_ascii() {
                true => tail.eq_ignore_ascii_case(name),
                false => tail == *name,
            };
            if hit {
                return Some((&token[..cut], difficulty));
            }
        }
    }
    None
}

pub fn parse_span(raw: &str) -> Option<Span> {
    let text = fold_symbols(raw.trim());
    if text.is_empty() {
        return None;
    }
    for (prefix, apply) in [
        (">=", Span::at_least as fn(f64) -> Span),
        ("≥", Span::at_least),
        (">", |value: f64| Span {
            min: value,
            min_set: true,
            min_open: true,
            ..Span::default()
        }),
        ("<=", Span::at_most),
        ("≤", Span::at_most),
        ("<", |value: f64| Span {
            max: value,
            max_set: true,
            max_open: true,
            ..Span::default()
        }),
        ("=", Span::exactly),
    ] {
        if let Some(rest) = text.strip_prefix(prefix) {
            let value: f64 = rest.trim().parse().ok()?;
            return Some(apply(value));
        }
    }
    if let Some(lower) = text.strip_suffix('+') {
        let value: f64 = lower.trim().parse().ok()?;
        return Some(Span::at_least(value));
    }
    match text.split_once('-') {
        None => {
            let value: f64 = text.parse().ok()?;
            Some(Span::exactly(value))
        }
        Some((head, tail)) if tail.trim().is_empty() => {
            let value: f64 = head.trim().parse().ok()?;
            Some(Span::at_most(value))
        }
        Some((head, tail)) => {
            let mut span = Span::default();
            if !head.trim().is_empty() {
                span.min = head.trim().parse().ok()?;
                span.min_set = true;
            }
            if !tail.trim().is_empty() {
                span.max = tail.trim().parse().ok()?;
                span.max_set = true;
            }
            if span.on() {
                Some(span)
            } else {
                None
            }
        }
    }
}

fn is_mixed_token(token: &str) -> bool {
    let folded = fold_symbols(token.trim());
    if folded.len() < 3 || !folded.contains('+') || !folded.contains('-') {
        return false;
    }
    folded.chars().all(|character| {
        character.is_ascii_digit() || character == '.' || character == '+' || character == '-'
    })
}

fn bare_integer(token: &str) -> Option<f64> {
    let folded = fold_symbols(token);
    if folded.contains(['.', '-', '+', '=', '<', '>', '~']) {
        return None;
    }
    folded.parse::<f64>().ok()
}

fn glued(token: &str) -> Option<(NumField, String)> {
    let lowered = fold_symbols(token).to_ascii_lowercase();
    for (field, keys) in NUM_KEYWORDS {
        for key in keys {
            if let Some(rest) = lowered.strip_prefix(key) {
                let rest = rest.trim_start_matches([':', '：', '=', ' ']);
                if !rest.is_empty() && parse_span(rest).is_some() {
                    return Some((field, rest.to_string()));
                }
            }
        }
    }
    for (field, keys) in NUM_KEYWORDS {
        for key in keys {
            if let Some(rest) = lowered.strip_suffix(key) {
                if !rest.is_empty() && rest.parse::<f64>().is_ok() {
                    return Some((field, rest.to_string()));
                }
            }
        }
    }
    None
}

fn standalone(token: &str) -> Option<Result<NumField, ()>> {
    let lowered = fold_symbols(token).to_ascii_lowercase();
    for key in TEXT_KEYWORDS {
        if lowered == key {
            return Some(Err(()));
        }
    }
    for (field, keys) in NUM_KEYWORDS {
        for key in keys.iter().copied() {
            if lowered == key {
                return Some(Ok(field));
            }
        }
    }
    None
}

fn cut_designer(token: &str) -> Option<String> {
    let lowered = fold_symbols(token).to_ascii_lowercase();
    for key in DESIGNER_KEYWORDS {
        if let Some(rest) = lowered.strip_prefix(key) {
            let offset = token.len() - rest.len();
            let tail = token.get(offset..).unwrap_or("");
            return Some(tail.trim_start_matches([':', '：', '=', ' ']).to_string());
        }
    }
    None
}

fn cut_title(token: &str) -> Option<String> {
    let lowered = fold_symbols(token).to_ascii_lowercase();
    for key in TEXT_KEYWORDS {
        if lowered == key {
            return Some(String::new());
        }
        if let Some(rest) = lowered.strip_prefix(key) {
            if rest.starts_with(':') {
                let offset = token.len() - rest.len();
                return Some(
                    token
                        .get(offset..)
                        .unwrap_or("")
                        .trim_start_matches([':', '：'])
                        .to_string(),
                );
            }
        }
    }
    None
}

fn starts_condition(token: &str) -> bool {
    diff_name(token).is_some()
        || Level::parse(token).is_some()
        || parse_span(token).is_some()
        || matches!(standalone(token), Some(Ok(_)))
        || glued(token).is_some()
        || cut_designer(token).is_some()
}

impl Filter {
    pub fn parse(raw: &str) -> Result<Option<Filter>, FilterError> {
        for token in raw.split_whitespace() {
            if is_mixed_token(token) {
                return Err(FilterError::等级与定数混写);
            }
        }
        let fields: Vec<&str> = raw.split_whitespace().collect();
        if fields.is_empty() {
            return Ok(None);
        }
        let mut filter = Filter::default();
        let mut texts: Vec<String> = Vec::new();
        let mut seen: Vec<NumField> = Vec::new();
        let mut index = 0usize;
        while index < fields.len() {
            let token = fields[index];
            if let Some((field, value)) = glued(token) {
                Self::apply_num(&mut filter, &mut seen, field, &value)?;
                index += 1;
                continue;
            }
            if let Some(rest) = cut_title(token) {
                index += 1;
                if !rest.is_empty() {
                    texts.push(rest);
                }
                while index < fields.len() && diff_name(fields[index]).is_none() {
                    texts.push(fields[index].to_string());
                    index += 1;
                }
                continue;
            }
            if let Some(kind) = standalone(token) {
                index += 1;
                let field = match kind {
                    Ok(field) => field,
                    Err(()) => return Err(FilterError::无法解析(token.to_string())),
                };
                let mut values: Vec<String> = Vec::new();
                if index < fields.len() {
                    values.push(fields[index].to_string());
                    index += 1;
                    while index < fields.len()
                        && values
                            .last()
                            .map(|value| value.ends_with(['-', '+', '<', '>', '=']))
                            .unwrap_or(false)
                    {
                        values.push(fields[index].to_string());
                        index += 1;
                    }
                }
                if values.is_empty() {
                    return Err(FilterError::缺值(field.label().to_string()));
                }
                Self::apply_num(&mut filter, &mut seen, field, &values.join(""))?;
                continue;
            }
            if let Some(rest) = cut_designer(token) {
                index += 1;
                let mut values: Vec<String> = Vec::new();
                if !rest.is_empty() {
                    values.push(rest);
                }
                while index < fields.len() && !starts_condition(fields[index]) {
                    values.push(fields[index].to_string());
                    index += 1;
                }
                if values.is_empty() {
                    return Err(FilterError::缺值("谱师".to_string()));
                }
                filter.designer = values.join(" ");
                continue;
            }
            if let Some((prefix, difficulty)) = glued_diff(token) {
                index += 1;
                if !prefix.is_empty() {
                    texts.push(prefix.to_string());
                }
                if !filter.diffs.contains(&difficulty) {
                    filter.diffs.push(difficulty);
                }
                continue;
            }
            if let Some(difficulty) = diff_name(token) {
                if !filter.diffs.contains(&difficulty) {
                    filter.diffs.push(difficulty);
                }
                index += 1;
                continue;
            }
            if let Some(level) = Level::parse(token) {
                if !filter.levels.contains(&level) {
                    filter.levels.push(level);
                }
                index += 1;
                continue;
            }
            if let Some(span) = parse_span(token) {
                if seen.contains(&NumField::Constant) {
                    return Err(FilterError::条件重复("定数"));
                }
                filter.constant = span;
                seen.push(NumField::Constant);
                index += 1;
                continue;
            }
            texts.push(token.to_string());
            index += 1;
        }
        filter.text = texts.join(" ");
        if !filter.levels.is_empty() && filter.constant.on() {
            return Err(FilterError::等级与定数混写);
        }
        filter
            .diffs
            .sort_by_key(|difficulty| display_rank(*difficulty));
        filter
            .levels
            .sort_by_key(|level| (level.int, u8::from(level.plus)));
        if filter.is_empty() {
            return Ok(None);
        }
        Ok(Some(filter))
    }

    pub fn text_only(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Self::default()
        }
    }

    pub fn keeps_chart(&self, difficulty: Difficulty, constant: f64) -> bool {
        if !self.diffs.is_empty() && !self.diffs.contains(&difficulty) {
            return false;
        }
        if !self.levels.is_empty() {
            let tenth = (constant * 10.0).round() as i64;
            let whole = (tenth / 10).clamp(0, i64::from(Level::MAX)) as u8;
            if !self.levels.contains(&Level::new(whole, tenth % 10 >= 7)) {
                return false;
            }
        }
        !self.constant.on() || self.constant.contains(constant)
    }

    fn apply_num(
        filter: &mut Filter,
        seen: &mut Vec<NumField>,
        field: NumField,
        value: &str,
    ) -> Result<(), FilterError> {
        if seen.contains(&field) {
            return Err(FilterError::条件重复(field.label()));
        }
        if field == NumField::Constant {
            if let Some(low) = bare_integer(value) {
                filter.constant = Span::band(low);
                seen.push(field);
                return Ok(());
            }
        }
        let span = parse_span(value).ok_or_else(|| FilterError::无法解析(value.to_string()))?;
        match field {
            NumField::Constant => filter.constant = span,
            NumField::Bpm => filter.bpm = span,
            NumField::Note => filter.note = span,
        }
        seen.push(field);
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
            && !self.constant.on()
            && self.levels.is_empty()
            && self.diffs.is_empty()
            && !self.bpm.on()
            && !self.note.on()
            && self.designer.is_empty()
    }

    pub fn has_level(&self) -> bool {
        !self.levels.is_empty()
    }

    pub fn has_constant(&self) -> bool {
        self.constant.on()
    }

    pub fn describe(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if !self.levels.is_empty() {
            let labels: Vec<String> = self.levels.iter().map(|level| level.label()).collect();
            parts.push(format!("等级 {}", labels.join("/")));
        }
        if self.constant.on() {
            parts.push(format!("定数 {}", self.constant.text()));
        }
        if !self.diffs.is_empty() {
            let codes: Vec<&str> = self
                .diffs
                .iter()
                .map(|difficulty| difficulty.as_str())
                .collect();
            parts.push(codes.join("/"));
        }
        if self.bpm.on() {
            parts.push(format!("BPM {}", self.bpm.count_text()));
        }
        if self.note.on() {
            parts.push(format!("物量 {}", self.note.count_text()));
        }
        if !self.designer.is_empty() {
            parts.push(format!("谱师 {}", self.designer));
        }
        if !self.text.is_empty() {
            parts.push(format!("「{}」", self.text));
        }
        parts.join(" · ")
    }

    pub fn to_dsl(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if self.constant.on() {
            parts.push(self.constant.dsl("constant"));
        }
        if self.bpm.on() {
            parts.push(self.bpm.dsl("bpm"));
        }
        if self.note.on() {
            parts.push(self.note.dsl("note"));
        }
        if self.levels.len() == 1 {
            parts.push(format!("rating_display=\"{}\"", self.levels[0].label()));
        } else if self.levels.len() > 1 {
            let labels: Vec<String> = self
                .levels
                .iter()
                .map(|level| format!("\"{}\"", level.label()))
                .collect();
            parts.push(format!("rating_display IN ({})", labels.join(",")));
        }
        if !self.diffs.is_empty() {
            let slots: Vec<String> = self
                .diffs
                .iter()
                .map(|difficulty| difficulty.as_upstream().to_string())
                .collect();
            let labels: Vec<String> = self
                .diffs
                .iter()
                .map(|difficulty| format!("\"{}\"", difficulty.as_str()))
                .collect();
            if slots.len() == 1 {
                parts.push(format!("rating_class={}", slots[0]));
                parts.push(format!("difficulty_label={}", labels[0]));
            } else {
                parts.push(format!("rating_class IN ({})", slots.join(",")));
                parts.push(format!("difficulty_label IN ({})", labels.join(",")));
            }
        }
        if !self.designer.is_empty() {
            parts.push(format!("designer~{}", quote(&self.designer)));
        }
        parts.join(" and ")
    }

    pub fn matches(&self, probe: &ChartProbe) -> bool {
        for word in self.text.to_lowercase().split_whitespace() {
            if !probe.text_hit(word) {
                return false;
            }
        }
        if self.constant.on() && (!probe.has_constant || !self.constant.contains(probe.constant)) {
            return false;
        }
        if !self.diffs.is_empty() && !self.diffs.contains(&probe.difficulty) {
            return false;
        }
        if !self.levels.is_empty() {
            match Level::parse(&probe.level) {
                Some(level) if self.levels.contains(&level) => {}
                _ => return false,
            }
        }
        if self.bpm.on() && (probe.bpm <= 0.0 || !self.bpm.contains(probe.bpm)) {
            return false;
        }
        if self.note.on() && (probe.note <= 0 || !self.note.contains(probe.note as f64)) {
            return false;
        }
        self.designer.is_empty()
            || probe
                .designer
                .to_lowercase()
                .contains(&self.designer.to_lowercase())
    }
}

fn quote(text: &str) -> String {
    let cleaned: String = text
        .chars()
        .filter(|character| *character != '"' && *character != '\'')
        .collect();
    format!("\"{cleaned}\"")
}

fn display_rank(difficulty: Difficulty) -> u8 {
    match difficulty {
        Difficulty::Past => 0,
        Difficulty::Present => 1,
        Difficulty::Future => 2,
        Difficulty::Eternal => 3,
        Difficulty::Beyond => 4,
        Difficulty::Ins => 5,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChartProbe {
    pub title: String,
    pub difficulty: Difficulty,
    pub level: String,
    pub constant: f64,
    pub has_constant: bool,
    pub bpm: f64,
    pub note: i64,
    pub designer: String,
    pub pack_id: String,
    pub pack_name: String,
    pub pack_name_zh: String,
    pub pack_aliases: Vec<String>,
}

impl ChartProbe {
    pub fn new(title: impl Into<String>, difficulty: Difficulty, constant: f64) -> Self {
        Self {
            title: title.into(),
            difficulty,
            level: String::new(),
            constant,
            has_constant: constant > 0.0,
            bpm: 0.0,
            note: 0,
            designer: String::new(),
            pack_id: String::new(),
            pack_name: String::new(),
            pack_name_zh: String::new(),
            pack_aliases: Vec::new(),
        }
    }

    pub fn with_level(mut self, level: impl Into<String>) -> Self {
        self.level = level.into();
        self
    }

    pub fn with_bpm(mut self, bpm: f64) -> Self {
        self.bpm = bpm;
        self
    }

    pub fn with_note(mut self, note: i64) -> Self {
        self.note = note;
        self
    }

    pub fn with_designer(mut self, designer: impl Into<String>) -> Self {
        self.designer = designer.into();
        self
    }

    pub fn with_pack(
        mut self,
        id: impl Into<String>,
        name: impl Into<String>,
        name_zh: impl Into<String>,
    ) -> Self {
        self.pack_id = id.into();
        self.pack_name = name.into();
        self.pack_name_zh = name_zh.into();
        self
    }

    fn text_hit(&self, word: &str) -> bool {
        if self.title.to_lowercase().contains(word) {
            return true;
        }
        for field in [&self.pack_id, &self.pack_name, &self.pack_name_zh] {
            if !field.is_empty() && field.to_lowercase().contains(word) {
                return true;
            }
        }
        self.pack_aliases
            .iter()
            .any(|alias| alias.to_lowercase().contains(word))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(raw: &str) -> Filter {
        Filter::parse(raw)
            .unwrap_or_else(|error| panic!("{raw} 应当能解析，实际报错：{error}"))
            .unwrap_or_else(|| panic!("{raw} 应当解析出条件"))
    }

    #[test]
    fn case_283() {
        let filter = parse("9");
        assert_eq!(filter.levels, vec![Level::new(9, false)]);
        assert!(!filter.constant.on(), "裸整数不许被读成定数");
        let plus = parse("9+");
        assert_eq!(plus.levels, vec![Level::new(9, true)]);
        assert_eq!(plus.describe(), "等级 9+");
    }

    #[test]
    fn case_284() {
        let exact = parse("11.0");
        assert!(exact.constant.contains(11.0));
        assert!(!exact.constant.contains(11.1), "11.0 是精确值不是整档");
        let range = parse("10.0-10.9");
        assert!(range.constant.contains(10.0));
        assert!(range.constant.contains(10.9));
        assert!(!range.constant.contains(11.0));
        let at_least = parse(">=10.5");
        assert!(at_least.constant.contains(10.5));
        assert!(!at_least.constant.contains(10.4));
        let strict = parse(">10.5");
        assert!(!strict.constant.contains(10.5), "大于号是开区间");
        assert!(strict.constant.contains(10.6));
        let at_most = parse("10-");
        assert!(at_most.constant.contains(9.9));
        assert!(!at_most.constant.contains(10.1));
        let band = parse("定数11");
        assert!(band.constant.contains(11.0));
        assert!(band.constant.contains(11.9));
        assert!(!band.constant.contains(12.0));
    }

    #[test]
    fn case_285() {
        let filter = parse("＞＝１０.５");
        assert!(filter.constant.contains(10.5));
        assert!(!filter.constant.contains(10.4));
        let square = parse("１０＋");
        assert_eq!(square.levels, vec![Level::new(10, true)]);
        assert_eq!(parse("１１").levels, vec![Level::new(11, false)]);
    }

    #[test]
    fn case_286() {
        let error = Filter::parse("9 10.0-10.9").expect_err("等级与定数同时出现必须拒绝");
        assert_eq!(error.to_string(), MIXED_HINT, "错误文案要与提示一致");
        let reversed = Filter::parse("10.0-10.9 9").expect_err("写反了也要拒绝");
        assert_eq!(reversed.to_string(), MIXED_HINT);
        let glued = Filter::parse("8+-9").expect_err("8+-9 是两套写法撞在一起");
        assert_eq!(glued.to_string(), MIXED_HINT);
        assert!(Filter::parse("8+ 9").is_ok(), "两个等级写在一起是合法的");
        assert!(Filter::parse("8.0-9.9").is_ok(), "只写定数也是合法的");
    }

    #[test]
    fn case_287() {
        let filter = parse("PST PRS FTR BYD ETR");
        assert_eq!(filter.diffs.len(), 5);
        assert_eq!(filter.describe(), "PST/PRS/FTR/ETR/BYD");
        assert_eq!(parse("永恒").diffs, vec![Difficulty::Eternal]);
        assert_eq!(parse("超越").diffs, vec![Difficulty::Beyond]);
        assert!(
            parse("3").levels == vec![Level::new(3, false)],
            "纯数字是等级，不是 BYD 的槽位号"
        );
    }

    #[test]
    fn case_288() {
        let glued = parse("bpm180-200");
        assert!(glued.bpm.contains(180.0));
        assert!(glued.bpm.contains(200.0));
        let suffixed = parse("180bpm");
        assert!(suffixed.bpm.contains(180.0));
        let spaced = parse("物量 1000");
        assert!(spaced.note.contains(1000.0));
        assert_eq!(spaced.describe(), "物量 1000");
        assert_eq!(parse("note1000+").describe(), "物量 ≥1000");
    }

    #[test]
    fn case_289() {
        let error = Filter::parse("bpm180 bpm200").expect_err("重复给 bpm 要拒绝");
        assert!(error.to_string().contains("bpm"), "错误要点名重复的条件");
        let duplicated = Filter::parse("11.0 11.5").expect_err("两个定数条件也是重复");
        assert!(duplicated.to_string().contains("定数"));
    }

    #[test]
    fn case_290() {
        let filter = parse("title:Fracture Ray FTR");
        assert_eq!(filter.text, "Fracture Ray");
        assert_eq!(filter.diffs, vec![Difficulty::Future]);
        let designer = parse("谱师 nitro 11+");
        assert_eq!(designer.designer, "nitro");
        assert_eq!(
            designer.levels,
            vec![Level::new(11, true)],
            "11+ 不能被吃进谱师名"
        );
        let spaced = parse("designer Toaster 12.0");
        assert_eq!(spaced.designer, "Toaster");
        assert!(spaced.constant.contains(12.0));
        let inline = parse("谱师nitro");
        assert_eq!(inline.designer, "nitro");
    }

    #[test]
    fn case_291() {
        let filter = parse("9+ FTR bpm180 note1000 光 谱师 nitro");
        let text = filter.describe();
        assert!(text.starts_with("等级 9+"), "等级写在最前，实际：{text}");
        assert!(text.contains("FTR"), "实际：{text}");
        assert!(text.contains("BPM 180"), "实际：{text}");
        assert!(text.contains("物量 1000"), "实际：{text}");
        assert!(text.contains("谱师 nitro"), "实际：{text}");
        assert!(text.ends_with("「光」"), "实际：{text}");
        assert_eq!(parse("10.0+").describe(), "定数 ≥10");
    }

    #[test]
    fn case_292() {
        assert_eq!(Filter::parse(""), Ok(None));
        assert_eq!(Filter::parse("   "), Ok(None));
        let filter = parse("FTR");
        assert!(!filter.is_empty());
    }

    #[test]
    fn case_293() {
        let probe = ChartProbe::new("Fracture Ray", Difficulty::Future, 11.2)
            .with_level("10+")
            .with_bpm(160.0)
            .with_note(1300)
            .with_designer("Toaster")
            .with_pack("rei", "Luminous Sky", "光与影");
        assert!(parse("FTR 10.0-11.5").matches(&probe));
        assert!(parse("fracture").matches(&probe));
        assert!(parse("bpm160").matches(&probe));
        assert!(parse("物量1000+").matches(&probe));
        assert!(parse("谱师 toaster").matches(&probe));
        assert!(parse("10+").matches(&probe), "等级逐字比");
        assert!(!parse("9+").matches(&probe));
        assert!(!parse("BYD").matches(&probe));
        assert!(!parse("12.0-12.9").matches(&probe));
        assert!(parse("lum").matches(&probe), "曲包名也能命中文本条件");
        assert!(parse("11.0-12.0").matches(&probe), "区间两端都含");
        assert!(
            !parse("11.3-12.0").matches(&probe),
            "定数是精确到一位小数的区间"
        );
    }

    #[test]
    fn case_294() {
        let filter = parse("9+ FTR 谱师 nitro");
        let dsl = filter.to_dsl();
        assert!(dsl.contains("rating_display=\"9+\""), "实际：{dsl}");
        assert!(dsl.contains("rating_class=2"), "实际：{dsl}");
        assert!(dsl.contains("difficulty_label=\"FTR\""), "实际：{dsl}");
        assert!(dsl.contains("designer~\"nitro\""), "实际：{dsl}");
        assert!(filter.to_dsl().contains(" and "));
        let ranged = parse("10.0-10.9");
        let ranged_dsl = ranged.to_dsl();
        assert!(ranged_dsl.contains("constant>=10"), "实际：{ranged_dsl}");
        assert!(ranged_dsl.contains("constant<=10.9"), "实际：{ranged_dsl}");
    }
}
