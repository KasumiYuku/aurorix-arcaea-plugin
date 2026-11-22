use crate::model::Difficulty;

pub const NEAR_BAND: f64 = 0.05;

pub const ALTERNATIVES_SHOWN: usize = 5;

pub const PICK_ROWS: usize = 10;

pub const ALTERNATIVES_LIMIT: usize = PICK_ROWS - 1;

pub const EXACT: f64 = 1.0;

pub const PREFIX_TITLE: f64 = 0.92;

pub const PREFIX_ALIAS: f64 = 0.9;

pub const PREFIX_ID: f64 = 0.88;

pub const CONTAINS_TITLE: f64 = 0.85;

pub const CONTAINS_ALIAS: f64 = 0.83;

pub const CONTAINS_ARTIST: f64 = 0.7;

pub const FUZZY_BASE: f64 = 0.55;

pub const FUZZY_SPAN: f64 = 0.2;

pub fn normalize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        let mapped = match character {
            '\u{3000}' => ' ',
            '\u{FF01}'..='\u{FF5E}' => {
                let code = character as u32 - 0xFF01 + 0x21;
                char::from_u32(code).unwrap_or(character)
            }
            other => other,
        };
        if mapped.is_whitespace() {
            continue;
        }
        for lowered in mapped.to_lowercase() {
            out.push(lowered);
        }
    }
    out
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chart {
    pub difficulty: Difficulty,
    pub level: String,
    pub constant: f64,
    pub has_constant: bool,
    pub designer: String,
    pub note: i64,
    pub chart_url: String,
    pub ill_url: String,
    pub audio_url: String,
}

impl Chart {
    pub fn new(difficulty: Difficulty, level: impl Into<String>, constant: f64) -> Self {
        Self {
            difficulty,
            level: level.into(),
            constant,
            has_constant: constant > 0.0,
            designer: String::new(),
            note: 0,
            chart_url: String::new(),
            ill_url: String::new(),
            audio_url: String::new(),
        }
    }

    pub fn with_designer(mut self, designer: impl Into<String>) -> Self {
        self.designer = designer.into();
        self
    }

    pub fn with_note(mut self, note: i64) -> Self {
        self.note = note;
        self
    }

    pub fn with_assets(
        mut self,
        chart_url: impl Into<String>,
        ill_url: impl Into<String>,
        audio_url: impl Into<String>,
    ) -> Self {
        self.chart_url = chart_url.into();
        self.ill_url = ill_url.into();
        self.audio_url = audio_url.into();
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Song {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub bpm: f64,
    pub pack_id: String,
    pub pack_name: String,
    pub pack_name_zh: String,
    pub pack_aliases: Vec<String>,
    pub aliases: Vec<String>,
    pub charts: Vec<Chart>,
}

impl Song {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            artist: String::new(),
            bpm: 0.0,
            pack_id: String::new(),
            pack_name: String::new(),
            pack_name_zh: String::new(),
            pack_aliases: Vec::new(),
            aliases: Vec::new(),
            charts: Vec::new(),
        }
    }

    pub fn with_artist(mut self, artist: impl Into<String>) -> Self {
        self.artist = artist.into();
        self
    }

    pub fn with_bpm(mut self, bpm: f64) -> Self {
        self.bpm = bpm;
        self
    }

    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    pub fn with_aliases(mut self, aliases: &[&str]) -> Self {
        self.aliases
            .extend(aliases.iter().map(|alias| alias.to_string()));
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

    pub fn with_pack_alias(mut self, alias: impl Into<String>) -> Self {
        self.pack_aliases.push(alias.into());
        self
    }

    pub fn with_chart(mut self, chart: Chart) -> Self {
        self.charts.push(chart);
        self
    }

    pub fn chart(&self, difficulty: Difficulty) -> Option<&Chart> {
        self.charts
            .iter()
            .find(|chart| chart.difficulty == difficulty)
            .or_else(|| {
                self.charts
                    .iter()
                    .find(|chart| chart.difficulty.rank() == difficulty.rank())
            })
    }

    pub fn exact(&self, query: &str) -> bool {
        let wanted = normalize(query);
        if wanted.is_empty() {
            return false;
        }
        if wanted == normalize(&self.id) || wanted == normalize(&self.title) {
            return true;
        }
        self.aliases.iter().any(|alias| wanted == normalize(alias))
    }

    pub fn strong_hit(&self, query: &str) -> bool {
        let wanted = normalize(query);
        if wanted.is_empty() {
            return false;
        }
        if self.exact(query) {
            return true;
        }
        if normalize(&self.title).starts_with(&wanted) {
            return true;
        }
        self.aliases
            .iter()
            .any(|alias| normalize(alias).starts_with(&wanted))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchKind {
    Id,
    Alias,
    Title,
    Artist,
    Fuzzy,
}

impl MatchKind {
    pub fn as_str(self) -> &'static str {
        match self {
            MatchKind::Id => "id",
            MatchKind::Alias => "alias",
            MatchKind::Title => "title",
            MatchKind::Artist => "artist",
            MatchKind::Fuzzy => "fuzzy",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "id" => MatchKind::Id,
            "alias" => MatchKind::Alias,
            "title" => MatchKind::Title,
            "artist" => MatchKind::Artist,
            _ => MatchKind::Fuzzy,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SongHit {
    pub song: Song,
    pub relevance: f64,
    pub matched_by: MatchKind,
}

impl SongHit {
    pub fn exact(&self, query: &str) -> bool {
        matches!(
            self.matched_by,
            MatchKind::Id | MatchKind::Alias | MatchKind::Title
        ) && self.song.exact(query)
    }

    pub fn label(&self) -> String {
        format!("{}（{}）", self.song.title, self.song.id)
    }
}

fn contains_ratio(haystack: &str, needle: &str) -> f64 {
    if needle.is_empty() {
        return 0.0;
    }
    let haystack_length = haystack.chars().count();
    let needle_length = needle.chars().count();
    if haystack_length == 0 || needle_length > haystack_length {
        return 0.0;
    }
    needle_length as f64 / haystack_length as f64
}

fn is_subsequence(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    let mut remaining = needle.chars();
    let mut wanted = remaining.next();
    for character in haystack.chars() {
        match wanted {
            None => return true,
            Some(expected) if expected == character => {
                wanted = remaining.next();
            }
            Some(_) => {}
        }
    }
    wanted.is_none()
}

fn best_of(
    current: Option<(f64, MatchKind)>,
    candidate: (f64, MatchKind),
) -> Option<(f64, MatchKind)> {
    match current {
        None => Some(candidate),
        Some((score, kind)) => {
            if candidate.0 > score || (candidate.0 == score && candidate.1 < kind) {
                Some(candidate)
            } else {
                Some((score, kind))
            }
        }
    }
}

pub fn score_song(song: &Song, query: &str) -> Option<(f64, MatchKind)> {
    let wanted = normalize(query);
    if wanted.is_empty() {
        return None;
    }
    let id = normalize(&song.id);
    let title = normalize(&song.title);
    let artist = normalize(&song.artist);
    if id == wanted {
        return Some((EXACT, MatchKind::Id));
    }
    if title == wanted {
        return Some((EXACT, MatchKind::Title));
    }
    if song.aliases.iter().any(|alias| normalize(alias) == wanted) {
        return Some((EXACT, MatchKind::Alias));
    }
    let mut best: Option<(f64, MatchKind)> = None;
    if title.starts_with(&wanted) {
        best = best_of(best, (PREFIX_TITLE, MatchKind::Title));
    }
    if song
        .aliases
        .iter()
        .any(|alias| normalize(alias).starts_with(&wanted))
    {
        best = best_of(best, (PREFIX_ALIAS, MatchKind::Alias));
    }
    if id.starts_with(&wanted) {
        best = best_of(best, (PREFIX_ID, MatchKind::Id));
    }
    if title.contains(&wanted) {
        best = best_of(best, (CONTAINS_TITLE, MatchKind::Title));
    }
    if song
        .aliases
        .iter()
        .any(|alias| normalize(alias).contains(&wanted))
    {
        best = best_of(best, (CONTAINS_ALIAS, MatchKind::Alias));
    }
    if !artist.is_empty() && artist.contains(&wanted) {
        best = best_of(best, (CONTAINS_ARTIST, MatchKind::Artist));
    }
    if is_subsequence(&title, &wanted) {
        let score = FUZZY_BASE + FUZZY_SPAN * contains_ratio(&title, &wanted);
        best = best_of(best, (score, MatchKind::Fuzzy));
    }
    if song
        .aliases
        .iter()
        .any(|alias| is_subsequence(&normalize(alias), &wanted))
    {
        let score = FUZZY_BASE + FUZZY_SPAN * contains_ratio(&title, &wanted);
        best = best_of(best, (score, MatchKind::Alias));
    }
    best
}

pub fn search(songs: &[Song], query: &str) -> Vec<SongHit> {
    let mut hits: Vec<SongHit> = songs
        .iter()
        .filter_map(|song| {
            score_song(song, query).map(|(relevance, matched_by)| SongHit {
                song: song.clone(),
                relevance,
                matched_by,
            })
        })
        .collect();
    hits.sort_by(|left, right| {
        right
            .relevance
            .total_cmp(&left.relevance)
            .then_with(|| left.song.id.cmp(&right.song.id))
            .then_with(|| left.song.title.cmp(&right.song.title))
    });
    hits
}

#[derive(Debug, Clone, PartialEq)]
pub struct SongMatch {
    pub best: SongHit,
    pub exact: bool,
    pub alternatives: Vec<SongHit>,
    pub total: usize,
}

impl SongMatch {
    pub fn needs_pick(&self, query: &str) -> bool {
        if self.alternatives.is_empty() {
            return false;
        }
        let mut exact_hits = usize::from(self.best.exact(query));
        for hit in &self.alternatives {
            if hit.exact(query) {
                exact_hits += 1;
            }
        }
        if exact_hits > 0 {
            return exact_hits > 1;
        }
        let mut strong_hits = usize::from(self.best.song.strong_hit(query));
        for hit in &self.alternatives {
            if hit.song.strong_hit(query) {
                strong_hits += 1;
            }
        }
        strong_hits != 1
    }

    pub fn candidates_line(&self) -> String {
        if self.alternatives.is_empty() {
            return String::new();
        }
        let mut text = format!("贴合度接近的还有 {} 首：", self.alternatives.len());
        for (index, hit) in self.alternatives.iter().enumerate() {
            if index >= ALTERNATIVES_SHOWN {
                text.push('…');
                break;
            }
            if index > 0 {
                text.push('、');
            }
            text.push_str(&hit.label());
        }
        let rest = self.total as i64 - 1 - self.alternatives.len() as i64;
        if rest > 0 {
            text.push_str(&format!("。后端共命中 {} 首，其余相关度更低", self.total));
        }
        text.push_str("。都不是你要的那首，直接用曲目 ID 重发");
        text
    }

    pub fn notes(&self, query: &str) -> Vec<String> {
        if self.exact {
            return Vec::new();
        }
        let mut notes = vec![format!(
            "咱把 「{}」 理解为 **{}**",
            query, self.best.song.title
        )];
        let line = self.candidates_line();
        if !line.is_empty() {
            notes.push(line);
        }
        notes
    }

    pub fn notes_brief(&self) -> Vec<String> {
        if self.exact {
            return Vec::new();
        }
        let line = self.candidates_line();
        if line.is_empty() {
            Vec::new()
        } else {
            vec![line]
        }
    }

    pub fn prefer_chart(self, difficulty: Difficulty) -> Self {
        if self.exact {
            return self;
        }
        if self.best.song.chart(difficulty).is_some() {
            return self;
        }
        let mut replacement: Option<usize> = None;
        for (index, hit) in self.alternatives.iter().enumerate() {
            if hit.song.chart(difficulty).is_some() {
                replacement = Some(index);
                break;
            }
        }
        let Some(index) = replacement else {
            return self;
        };
        let mut alternatives = self.alternatives;
        let promoted = alternatives.remove(index);
        alternatives.insert(0, self.best);
        Self {
            best: promoted,
            exact: self.exact,
            alternatives,
            total: self.total,
        }
    }
}

pub fn resolve(songs: &[Song], query: &str) -> Option<SongMatch> {
    let hits = search(songs, query);
    let total = hits.len();
    resolve_hits(hits, total, query)
}

pub fn resolve_hits(hits: Vec<SongHit>, total: usize, query: &str) -> Option<SongMatch> {
    let mut rest = hits.into_iter();
    let best = rest.next()?;
    let mut matched = SongMatch {
        exact: best.exact(query),
        best,
        alternatives: Vec::new(),
        total,
    };
    for hit in rest {
        if hit.relevance < matched.best.relevance - NEAR_BAND {
            break;
        }
        if matched.alternatives.len() >= ALTERNATIVES_LIMIT {
            break;
        }
        matched.alternatives.push(hit);
    }
    Some(matched)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn library() -> Vec<Song> {
        vec![
            Song::new("fractureray", "Fracture Ray")
                .with_artist("Sakuzyo")
                .with_bpm(160.0)
                .with_alias("骨折光")
                .with_alias("骨折")
                .with_chart(Chart::new(Difficulty::Future, "10+", 11.2))
                .with_chart(Chart::new(Difficulty::Beyond, "11", 11.7)),
            Song::new("fractureray_at", "Fracture Ray (Arcaea TanoC)")
                .with_artist("Sakuzyo")
                .with_chart(Chart::new(Difficulty::Future, "9+", 9.8)),
            Song::new("lumia", "Lumia")
                .with_artist("Sky_delta")
                .with_chart(Chart::new(Difficulty::Present, "7", 7.5)),
        ]
    }

    #[test]
    fn case_344() {
        let songs = library();
        assert!(songs[0].exact("fractureray"));
        assert!(
            songs[0].exact("Fracture Ray"),
            "大小写与空格不该影响精确匹配"
        );
        assert!(songs[0].exact("骨折光"));
        assert!(songs[0].exact("F R A C T U R E R A Y"), "空格被归一化掉");
        assert!(!songs[1].exact("Fracture Ray"), "前缀不算逐字命中");
        assert!(songs[1].strong_hit("Fracture Ray"), "前缀算强命中");
        assert!(!songs[1].exact("骨折"), "别名不跨曲目复用");
    }

    #[test]
    fn case_345() {
        let songs = vec![Song::new("a", "Ａｒｃａｎａ　Ｅｄｅｎ")];
        let hit = resolve(&songs, "arcana eden").expect("全角曲名应当能被半角输入命中");
        assert!(hit.exact, "归一化之后算逐字命中");
        assert_eq!(hit.best.song.id, "a");
    }

    #[test]
    fn case_346() {
        let songs = library();
        let hit = resolve(&songs, "frcray").expect("漏了字母也应当能模糊命中");
        assert_eq!(hit.best.song.id, "fractureray");
        assert!(!hit.exact, "模糊命中不是逐字命中");
        assert_eq!(
            hit.best.matched_by,
            MatchKind::Fuzzy,
            "跳字命中的途径是 fuzzy"
        );
    }

    #[test]
    fn case_347() {
        let songs = vec![
            Song::new("renegade_a", "Renegade").with_artist("A"),
            Song::new("renegade_b", "Renegade").with_artist("B"),
        ];
        let hit = resolve(&songs, "Renegade").expect("两首同名曲都能命中");
        assert_eq!(hit.total, 2);
        assert_eq!(hit.alternatives.len(), 1);
        assert!(hit.needs_pick("Renegade"), "逐字命中多首时必须问");
        assert!(
            hit.exact,
            "exact 只说明首选叫这个名字，判要不要问走 needs_pick"
        );
    }

    #[test]
    fn case_348() {
        let songs = vec![
            Song::new("renegade", "Renegade").with_artist("A"),
            Song::new("renegade_ark", "Renegade").with_artist("B"),
        ];
        let hit = resolve(&songs, "renegade").expect("两首都能命中");
        assert_eq!(hit.best.song.id, "renegade", "id 逐字命中排第一");
        assert!(hit.needs_pick("Renegade"), "打曲名时两首都逐字命中，必须问");
        assert!(
            hit.needs_pick("renegade"),
            "后端顺序为准: 曲目 ID 同时是同名曲的歌名时两首都逐字命中, 仍要问"
        );
    }

    #[test]
    fn case_349() {
        let hits = vec![
            SongHit {
                song: Song::new("brandnewworld", "Brand New World"),
                relevance: 0.82,
                matched_by: MatchKind::Fuzzy,
            },
            SongHit {
                song: Song::new("brandnew", "Brand New"),
                relevance: 0.79,
                matched_by: MatchKind::Fuzzy,
            },
            SongHit {
                song: Song::new("other", "Other"),
                relevance: 0.4,
                matched_by: MatchKind::Fuzzy,
            },
        ];
        let matched = resolve_hits(hits, 3, "brans new world").expect("有候选就该有结论");
        assert_eq!(matched.best.song.id, "brandnewworld", "首选沿用后端第一");
        assert_eq!(matched.alternatives.len(), 1, "只收近似带内的候选");
        assert_eq!(matched.alternatives[0].song.id, "brandnew");
        assert!(!matched.exact, "模糊命中不是精确命中");
        assert!(matched.needs_pick("brans new world"), "拼错时要让用户确认");
    }

    #[test]
    fn case_350() {
        let forward = vec![Song::new("a", "Renegade"), Song::new("b", "Renegade")];
        let backward = vec![Song::new("b", "Renegade"), Song::new("a", "Renegade")];
        let left = resolve(&forward, "Renegade").expect("能有结果");
        let right = resolve(&backward, "Renegade").expect("能有结果");
        assert_eq!(left.best.song.id, "a", "并列时按曲目 id 决胜");
        assert_eq!(left.best.song.id, right.best.song.id);
        assert_eq!(left.total, right.total);
    }

    #[test]
    fn case_351() {
        let songs: Vec<Song> = (0..9)
            .map(|index| Song::new(format!("song{index}"), format!("Conflict {index}")))
            .collect();
        let hit = resolve(&songs, "Conflict").expect("九首都命中");
        assert_eq!(hit.total, 9);
        assert_eq!(hit.alternatives.len(), 8, "候选条数先按近似带收集");
        let line = hit.candidates_line();
        assert!(line.starts_with("贴合度接近的还有 8 首："), "实际：{line}");
        assert_eq!(
            line.matches('（').count(),
            ALTERNATIVES_SHOWN,
            "最多只列 5 首"
        );
        assert!(line.contains('…'), "超过 5 首要写省略号：{line}");
        assert!(
            line.ends_with("。都不是你要的那首，直接用曲目 ID 重发"),
            "实际：{line}"
        );
        assert!(!line.contains("后端共命中"), "候选没收满 20 条时不必提总数");
    }

    #[test]
    fn case_352() {
        let songs: Vec<Song> = (0..40)
            .map(|index| Song::new(format!("song{index:02}"), format!("Conflict {index:02}")))
            .collect();
        let hit = resolve(&songs, "Conflict").expect("都命中");
        assert_eq!(hit.total, 40);
        assert_eq!(
            hit.alternatives.len(),
            ALTERNATIVES_LIMIT,
            "候选列表必须有上限"
        );
        let line = hit.candidates_line();
        assert!(
            line.contains("后端共命中 40 首，其余相关度更低"),
            "截断后要说清总数：{line}"
        );
    }

    #[test]
    fn case_353() {
        let songs = library();
        let hit = resolve(&songs, "Fracture Ray").expect("精确命中");
        assert!(hit.exact);
        assert!(hit.notes("Fracture Ray").is_empty(), "精确命中没有补充说明");
        let fuzzy = resolve(&songs, "骨折光").expect("别名命中");
        assert!(fuzzy.exact);
        let partial = resolve(&songs, "fractur").expect("前缀命中");
        assert!(!partial.exact);
        let notes = partial.notes("fractur");
        assert_eq!(notes.len(), 2, "先复述理解结果，再列候选");
        assert!(notes[0].contains("fractur"));
        assert!(notes[0].contains("Fracture Ray"));
    }

    #[test]
    fn case_354() {
        let songs = library();
        let hit = resolve(&songs, "fractur").expect("前缀命中两首");
        assert!(!hit.exact);
        let has_future = hit.clone().prefer_chart(Difficulty::Future);
        assert_eq!(has_future.best.song.id, "fractureray");
        let exact = resolve(&songs, "Fracture Ray (Arcaea TanoC)").expect("精确命中第二首");
        let kept = exact.prefer_chart(Difficulty::Beyond);
        assert_eq!(
            kept.best.song.id, "fractureray_at",
            "用户已经报出准确曲名，不该被换成别的曲子"
        );
    }

    #[test]
    fn case_355() {
        let songs = library();
        assert!(resolve(&songs, "zzzzzz").is_none());
        assert!(search(&songs, "").is_empty());
    }

    #[test]
    fn case_356() {
        let songs = library();
        let mut reversed = songs.clone();
        reversed.reverse();
        let left = search(&songs, "fractur");
        let right = search(&reversed, "fractur");
        let left_ids: Vec<&str> = left.iter().map(|hit| hit.song.id.as_str()).collect();
        let right_ids: Vec<&str> = right.iter().map(|hit| hit.song.id.as_str()).collect();
        assert_eq!(left_ids, right_ids, "检索结果不许取决于曲库输入顺序");
    }

    #[test]
    fn case_357() {
        let songs: Vec<Song> = (0..40)
            .map(|index| Song::new(format!("song{index:02}"), format!("Conflict {index:02}")))
            .collect();
        let hit = resolve(&songs, "Conflict").expect("都命中");
        let rows = 1 + hit.alternatives.len();
        assert_eq!(rows, PICK_ROWS, "候选行数 = 1 条最优 + 其余, 上限 {PICK_ROWS}");
    }
}
