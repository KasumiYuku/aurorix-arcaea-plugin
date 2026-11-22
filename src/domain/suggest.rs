use std::collections::BTreeMap;

use crate::domain::ptt;
use crate::model::Difficulty;

pub const PICK_TITLE: &str = "首选主推";

pub const UNKNOWN_LEVEL_LABEL: &str = "LEVEL ?";

pub const LEVEL_PREFIX: &str = "LEVEL ";

pub const PICK_PER_PAGE: usize = 10;

pub const PICK_MIN: usize = 5;

pub const PICK_MAX: usize = 10;

pub const DEFAULT_BONUS_NEED: f64 = 0.02;

pub const PAGE_LIMIT: usize = 60;

pub const FAR_UNIT_SCORE: f64 = 5_000_000.0;

pub const LOST_UNIT_SCORE: f64 = 10_000_000.0;

pub const EX_SCORE: i64 = 9_800_000;

pub const EPS: f64 = 1e-9;

#[derive(Debug, Clone, PartialEq)]
pub struct SuggestCandidate {
    pub song_id: String,
    pub title: String,
    pub difficulty: Difficulty,
    pub constant: f64,
    pub note: i64,
    pub score: i64,
    pub has_score: bool,
    pub far: i64,
    pub lost: i64,
    pub has_judgement: bool,
}

impl SuggestCandidate {
    pub fn new(
        song_id: impl Into<String>,
        title: impl Into<String>,
        difficulty: Difficulty,
        constant: f64,
        note: i64,
    ) -> Self {
        Self {
            song_id: song_id.into(),
            title: title.into(),
            difficulty,
            constant,
            note,
            score: 0,
            has_score: false,
            far: 0,
            lost: 0,
            has_judgement: false,
        }
    }

    pub fn with_score(mut self, score: i64) -> Self {
        self.score = score;
        self.has_score = score > 0;
        self
    }

    pub fn with_judgement(mut self, far: i64, lost: i64) -> Self {
        self.far = far;
        self.lost = lost;
        self.has_judgement = true;
        self
    }

    pub fn rating(&self) -> f64 {
        if !self.has_score || self.constant <= 0.0 {
            return 0.0;
        }
        ptt::single_rating(self.constant, self.score)
    }

    pub fn level_floor(&self) -> Option<i64> {
        if self.constant <= 0.0 {
            return None;
        }
        Some(self.constant.floor() as i64)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NotePlan {
    pub known: bool,
    pub far_need: i64,
    pub lost_need: i64,
    pub far_room: i64,
    pub lost_room: i64,
    pub far_use: i64,
    pub lost_use: i64,
    pub short: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuggestHint {
    pub song_id: String,
    pub title: String,
    pub difficulty: Difficulty,
    pub constant: f64,
    pub note: i64,
    pub score: i64,
    pub has_score: bool,
    pub far: i64,
    pub lost: i64,
    pub has_judgement: bool,
    pub rating: f64,
    pub target_rating: f64,
    pub delta: i64,
    pub target: i64,
    pub rank: usize,
    pub target_rank: usize,
    pub pick: bool,
    pub notes: NotePlan,
}

impl SuggestHint {
    fn level_floor(&self) -> Option<i64> {
        if self.constant <= 0.0 {
            return None;
        }
        Some(self.constant.floor() as i64)
    }

    fn tie_break(left: &SuggestHint, right: &SuggestHint) -> std::cmp::Ordering {
        left.song_id
            .cmp(&right.song_id)
            .then_with(|| left.difficulty.cmp(&right.difficulty))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuggestGroup {
    pub label: String,
    pub title: Option<&'static str>,
    pub hints: Vec<SuggestHint>,
}

impl SuggestGroup {
    pub fn count(&self) -> usize {
        self.hints.len()
    }

    pub fn heading(&self) -> String {
        if let Some(title) = self.title {
            return title.to_string();
        }
        if self.label.is_empty() {
            return UNKNOWN_LEVEL_LABEL.to_string();
        }
        format!("{LEVEL_PREFIX}{}", self.label)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SuggestBoard {
    pub groups: Vec<SuggestGroup>,
    pub hint_count: usize,
    pub pick_count: usize,
    pub pool: usize,
    pub pushable: usize,
}

impl SuggestBoard {
    pub fn labels(&self) -> Vec<String> {
        self.groups.iter().map(SuggestGroup::heading).collect()
    }

    pub fn all_hints(&self) -> Vec<&SuggestHint> {
        self.groups
            .iter()
            .flat_map(|group| group.hints.iter())
            .collect()
    }
}

pub fn far_unit(note: i64) -> f64 {
    if note <= 0 {
        return 0.0;
    }
    FAR_UNIT_SCORE / note as f64
}

pub fn lost_unit(note: i64) -> f64 {
    if note <= 0 {
        return 0.0;
    }
    LOST_UNIT_SCORE / note as f64
}

pub fn ceil_div(value: f64, step: f64) -> i64 {
    if step <= 0.0 {
        return 0;
    }
    let steps = (value / step - EPS).ceil();
    if steps <= 0.0 {
        0
    } else {
        steps as i64
    }
}

pub fn note_plan(candidate: &SuggestCandidate, delta: i64) -> NotePlan {
    if candidate.note <= 0 || !candidate.has_score || delta <= 0 {
        return NotePlan::default();
    }
    let far_gain = far_unit(candidate.note);
    let lost_gain = lost_unit(candidate.note);
    let mut plan = NotePlan {
        known: true,
        far_room: candidate.far,
        lost_room: candidate.lost,
        far_need: ceil_div(delta as f64, far_gain),
        lost_need: ceil_div(delta as f64, lost_gain),
        ..NotePlan::default()
    };
    let mut remaining = delta as f64;
    if candidate.lost > 0 {
        plan.lost_use = candidate.lost.min(ceil_div(remaining, lost_gain));
        remaining -= plan.lost_use as f64 * lost_gain;
    }
    if remaining > 1e-6 && candidate.far > 0 {
        plan.far_use = candidate.far.min(ceil_div(remaining, far_gain));
        remaining -= plan.far_use as f64 * far_gain;
    }
    plan.short = remaining > 1e-6;
    plan
}

pub fn quantize_target(candidate: &SuggestCandidate, raw_target: i64, delta: i64) -> Option<i64> {
    if !candidate.has_score || candidate.note <= 0 || delta <= 0 {
        return None;
    }
    let plan = note_plan(candidate, delta);
    if plan.known && plan.short {
        return None;
    }
    if raw_target >= ptt::FULL_SCORE {
        return None;
    }
    let unit = far_unit(candidate.note);
    let mut fars = (delta as f64 / unit - EPS).ceil();
    if fars < 1.0 {
        fars = 1.0;
    }
    let step = (fars * unit - EPS).ceil();
    let quantized = candidate
        .score
        .saturating_add(step as i64)
        .min(ptt::FULL_SCORE);
    if quantized <= candidate.score {
        return None;
    }
    if quantized < raw_target {
        return None;
    }
    Some(quantized)
}

pub fn clamp_score(value: f64) -> i64 {
    let rounded = value.round();
    if rounded <= 0.0 {
        return 0;
    }
    if rounded >= ptt::FULL_SCORE as f64 {
        return ptt::FULL_SCORE;
    }
    rounded as i64
}

pub fn score_for_bonus(constant: f64, bonus: f64) -> i64 {
    if constant <= 0.0 {
        return 0;
    }
    if bonus >= ptt::MAX_BONUS - EPS {
        return ptt::FULL_SCORE;
    }
    if bonus > 1.2 + EPS {
        let score = 9_800_000.0 + (bonus - 1.2) * 200_000.0;
        if score >= ptt::EX_SCORE as f64 {
            return clamp_score(score);
        }
    }
    let score = 9_500_000.0 + (bonus - 0.2) * 300_000.0;
    if score >= ptt::CLEAR_SCORE as f64 {
        return clamp_score(score);
    }
    0
}

pub fn bonus_of(constant: f64, score: i64) -> f64 {
    ptt::single_rating(constant, score) - constant
}

pub fn pick_limit_for(count: usize) -> usize {
    (count / PICK_PER_PAGE).clamp(PICK_MIN, PICK_MAX).min(count)
}

fn build_hint(candidate: &SuggestCandidate, bonus_need: f64) -> Option<SuggestHint> {
    if candidate.constant <= 0.0 {
        return None;
    }
    let current = candidate.rating();
    let (target, delta, has_score) = if candidate.has_score {
        if candidate.score >= ptt::FULL_SCORE {
            return None;
        }
        let goal_bonus =
            (bonus_of(candidate.constant, candidate.score) + bonus_need).min(ptt::MAX_BONUS);
        let raw_target = score_for_bonus(candidate.constant, goal_bonus);
        let raw_delta = raw_target - candidate.score;
        if raw_delta <= 0 {
            return None;
        }
        let target = quantize_target(candidate, raw_target, raw_delta).unwrap_or(raw_target);
        let target = target.max(raw_target);
        (target, target - candidate.score, true)
    } else {
        let raw_target = EX_SCORE.max(score_for_bonus(candidate.constant, ptt::PASS_BONUS));
        (raw_target, raw_target, false)
    };
    if delta <= 0 {
        return None;
    }
    let notes = note_plan(candidate, delta);
    let target_rating = ptt::single_rating(candidate.constant, target);
    Some(SuggestHint {
        song_id: candidate.song_id.clone(),
        title: candidate.title.clone(),
        difficulty: candidate.difficulty,
        constant: candidate.constant,
        note: candidate.note,
        score: candidate.score,
        has_score,
        far: candidate.far,
        lost: candidate.lost,
        has_judgement: candidate.has_judgement,
        rating: current,
        target_rating,
        delta,
        target,
        rank: 0,
        target_rank: 0,
        pick: false,
        notes,
    })
}

fn assign_ranks(hints: &mut [SuggestHint]) {
    let mut current: Vec<(f64, usize)> = hints
        .iter()
        .enumerate()
        .map(|(index, hint)| (hint.rating, index))
        .collect();
    current.sort_by(|left, right| right.0.total_cmp(&left.0));
    for (position, (_, index)) in current.iter().enumerate() {
        hints[*index].rank = position + 1;
    }
    let mut targets: Vec<(f64, usize)> = hints
        .iter()
        .enumerate()
        .map(|(index, hint)| (hint.target_rating, index))
        .collect();
    targets.sort_by(|left, right| right.0.total_cmp(&left.0));
    for (position, (_, index)) in targets.iter().enumerate() {
        hints[*index].target_rank = position + 1;
    }
}

fn order_by_target_rank(left: &SuggestHint, right: &SuggestHint) -> std::cmp::Ordering {
    left.target_rank
        .cmp(&right.target_rank)
        .then_with(|| SuggestHint::tie_break(left, right))
}

fn order_in_level(left: &SuggestHint, right: &SuggestHint) -> std::cmp::Ordering {
    right
        .has_score
        .cmp(&left.has_score)
        .then_with(|| right.constant.total_cmp(&left.constant))
        .then_with(|| left.delta.cmp(&right.delta))
        .then_with(|| SuggestHint::tie_break(left, right))
}

pub fn suggest(candidates: &[SuggestCandidate], bonus_need: f64, limit: usize) -> SuggestBoard {
    let limit = if limit == 0 { PAGE_LIMIT } else { limit };
    let pool = candidates.len();
    let mut hints: Vec<SuggestHint> = candidates
        .iter()
        .filter_map(|candidate| build_hint(candidate, bonus_need))
        .collect();
    assign_ranks(&mut hints);
    let pushable = hints
        .iter()
        .filter(|hint| hint.has_score && hint.target > hint.score)
        .count();
    let mut board = SuggestBoard {
        groups: Vec::new(),
        hint_count: 0,
        pick_count: 0,
        pool,
        pushable,
    };
    let mut picked: Vec<usize> = (0..hints.len())
        .filter(|index| {
            let hint = &hints[*index];
            hint.has_score
                && hint.target_rating > hint.rating
                && !(hint.notes.known && hint.notes.short)
        })
        .collect();
    picked.sort_by(|left, right| order_by_target_rank(&hints[*left], &hints[*right]));
    picked.truncate(pick_limit_for(hints.len()).min(limit));
    for index in &picked {
        hints[*index].pick = true;
    }
    let picks: Vec<SuggestHint> = picked.iter().map(|index| hints[*index].clone()).collect();
    if !picks.is_empty() {
        board.pick_count = picks.len();
        board.hint_count = picks.len();
        board.groups.push(SuggestGroup {
            label: String::new(),
            title: Some(PICK_TITLE),
            hints: picks,
        });
    }
    let mut rest: Vec<SuggestHint> = hints.into_iter().filter(|hint| !hint.pick).collect();
    rest.sort_by(order_in_level);
    let mut buckets: BTreeMap<Option<i64>, Vec<SuggestHint>> = BTreeMap::new();
    for hint in rest {
        buckets.entry(hint.level_floor()).or_default().push(hint);
    }
    let mut known: Vec<i64> = buckets.keys().filter_map(|key| *key).collect();
    known.sort_unstable_by(|left, right| right.cmp(left));
    let mut ordered_keys: Vec<Option<i64>> = known.into_iter().map(Some).collect();
    if buckets.contains_key(&None) {
        ordered_keys.push(None);
    }
    let mut remaining = limit.saturating_sub(board.pick_count);
    for key in ordered_keys {
        let Some(group) = buckets.remove(&key) else {
            continue;
        };
        if remaining == 0 {
            break;
        }
        let take = group.len().min(remaining);
        let mut taken: Vec<SuggestHint> = group.into_iter().take(take).collect();
        taken.sort_by(order_in_level);
        remaining -= taken.len();
        board.hint_count += taken.len();
        board.groups.push(SuggestGroup {
            label: key.map(|level| level.to_string()).unwrap_or_default(),
            title: None,
            hints: taken,
        });
    }
    board
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(song: &str, constant: f64, note: i64, score: i64, far: i64) -> SuggestCandidate {
        SuggestCandidate::new(
            song,
            song.to_uppercase(),
            Difficulty::Future,
            constant,
            note,
        )
        .with_score(score)
        .with_judgement(far + 10, 0)
    }

    fn signature(board: &SuggestBoard) -> Vec<String> {
        let mut out: Vec<String> = board
            .groups
            .iter()
            .map(|group| format!("{}#{}", group.heading(), group.count()))
            .collect();
        for hint in board.all_hints() {
            out.push(format!(
                "{}|{}|{}|{}|{}|{}",
                hint.song_id,
                hint.difficulty.as_str(),
                hint.score,
                hint.target,
                hint.delta,
                hint.pick
            ));
        }
        out
    }

    #[test]
    fn case_295() {
        let chart = candidate("a", 10.0, 800, 9_800_000, 0);
        let goal = bonus_of(10.0, 9_800_000) + DEFAULT_BONUS_NEED;
        let raw = score_for_bonus(10.0, goal);
        assert_eq!(raw, 9_804_000, "单曲 PTT 涨 0.02 对应 4000 分");
        let delta = raw - chart.score;
        let quantized = quantize_target(&chart, raw, delta).expect("应当能量化");
        assert_eq!(
            quantized, 9_806_250,
            "800 物量一个 FAR 值 6250 分，向上取整"
        );
        assert!(quantized >= raw, "量化只能往上，不许比原目标还低");
        let plan = note_plan(&chart, quantized - chart.score);
        assert!(plan.known);
        assert_eq!(plan.far_need, 1, "差 6250 分正好是一个 FAR");
        assert_eq!(plan.lost_need, 1, "一个 Lost 值 12500 分");
    }

    #[test]
    fn case_296() {
        let sparse = candidate("a", 10.0, 400, 9_800_000, 0);
        let dense = candidate("b", 10.0, 1600, 9_800_000, 0);
        let sparse_plan = note_plan(&sparse, 10_000);
        let dense_plan = note_plan(&dense, 10_000);
        assert_eq!(sparse_plan.far_need, 1, "400 物量一个 FAR 值 12500 分");
        assert_eq!(dense_plan.far_need, 4, "1600 物量一个 FAR 只值 3125 分");
        assert!(dense_plan.far_need > sparse_plan.far_need);
    }

    #[test]
    fn case_297() {
        let chart = candidate("a", 10.0, 0, 9_800_000, 0);
        let raw = 9_804_000;
        assert_eq!(
            quantize_target(&chart, raw, 4000),
            None,
            "不知道物量就不量化"
        );
        let plan = note_plan(&chart, 4000);
        assert!(!plan.known);
        assert_eq!(plan.far_need, 0);
    }

    #[test]
    fn case_298() {
        let chart = candidate("a", 10.0, 1000, 9_700_000, 2);
        let raw = score_for_bonus(10.0, bonus_of(10.0, 9_700_000) + 0.5);
        let delta = raw - chart.score;
        let plan = note_plan(&chart, delta);
        assert!(plan.known);
        assert!(plan.short, "现有 12 个 FAR 不够补上这么大的分差");
        assert_eq!(quantize_target(&chart, raw, delta), None, "short 时不量化");
    }

    #[test]
    fn case_299() {
        let candidates = vec![
            candidate("high", 11.2, 1200, 9_850_000, 0),
            candidate("mid", 10.4, 1000, 9_850_000, 0),
            SuggestCandidate::new("fresh_high", "F", Difficulty::Future, 10.8, 1000),
            SuggestCandidate::new("fresh_low", "F", Difficulty::Future, 9.6, 900),
        ];
        let board = suggest(&candidates, DEFAULT_BONUS_NEED, PAGE_LIMIT);
        let labels = board.labels();
        assert_eq!(labels[0], PICK_TITLE, "第一层永远是首选主推");
        assert_eq!(labels[1], "LEVEL 10");
        assert_eq!(labels[2], "LEVEL 9");
        assert_eq!(board.pick_count, 2, "只有打过的谱面能进主推");
        assert_eq!(board.hint_count, 4);
        let ids: Vec<&str> = board
            .all_hints()
            .iter()
            .map(|hint| hint.song_id.as_str())
            .collect();
        for id in ["high", "mid", "fresh_high", "fresh_low"] {
            assert_eq!(
                ids.iter().filter(|seen| **seen == id).count(),
                1,
                "{id} 只能出现一次"
            );
        }
    }

    #[test]
    fn case_300() {
        let mut unknown = candidate("unknown", 0.0, 1000, 9_850_000, 0);
        unknown.constant = 0.0;
        let candidates = vec![
            candidate("known", 10.4, 1000, 9_850_000, 0),
            SuggestCandidate::new("fresh", "F", Difficulty::Future, 9.5, 900),
            unknown,
        ];
        let board = suggest(&candidates, DEFAULT_BONUS_NEED, PAGE_LIMIT);
        assert_eq!(board.pool, 3);
        assert_eq!(board.hint_count, 2, "没有定数的谱面算不出目标分，不进建议");
        assert_eq!(board.groups.len(), 2, "首选主推 + LEVEL 9");
        assert!(board
            .labels()
            .iter()
            .all(|label| label != UNKNOWN_LEVEL_LABEL));
        assert!(board.all_hints().iter().all(|hint| hint.constant > 0.0));
    }

    #[test]
    fn case_301() {
        assert_eq!(pick_limit_for(3), 3);
        assert_eq!(pick_limit_for(20), 5);
        assert_eq!(pick_limit_for(60), 6);
        assert_eq!(pick_limit_for(200), 10);
    }

    #[test]
    fn case_302() {
        let mut candidates: Vec<SuggestCandidate> = (0..30)
            .map(|index| {
                candidate(
                    &format!("p{index:02}"),
                    9.0 + index as f64 * 0.1,
                    1000,
                    9_700_000 + index * 1000,
                    0,
                )
            })
            .collect();
        for index in 0..30 {
            candidates.push(SuggestCandidate::new(
                format!("u{index:02}"),
                "U",
                Difficulty::Future,
                9.0 + index as f64 * 0.1,
                1000,
            ));
        }
        let board = suggest(&candidates, DEFAULT_BONUS_NEED, PAGE_LIMIT);
        assert!(board.pick_count <= PICK_MAX);
        let mut levels = 0;
        for group in &board.groups {
            if group.title.is_some() {
                continue;
            }
            levels += 1;
            let mut unplayed_seen = false;
            let mut last_played = f64::INFINITY;
            let mut last_unplayed = f64::INFINITY;
            for hint in &group.hints {
                if hint.has_score {
                    assert!(!unplayed_seen, "已游玩必须排在未游玩之前");
                    assert!(hint.constant <= last_played, "已游玩内部按定数降序");
                    last_played = hint.constant;
                } else {
                    unplayed_seen = true;
                    assert!(hint.constant <= last_unplayed, "未游玩内部按定数降序");
                    last_unplayed = hint.constant;
                }
            }
        }
        assert!(levels > 1, "至少要跨两个等级档");
    }

    #[test]
    fn case_303() {
        let candidates = vec![SuggestCandidate::new(
            "fresh",
            "F",
            Difficulty::Future,
            10.5,
            1000,
        )];
        let board = suggest(&candidates, DEFAULT_BONUS_NEED, PAGE_LIMIT);
        let hint = board.all_hints()[0];
        assert!(!hint.has_score);
        assert_eq!(hint.target, EX_SCORE, "开荒目标就是 EX");
        assert_eq!(hint.delta, EX_SCORE);
        assert!(!hint.pick, "没打过的谱面不拿主推记号");
        assert_eq!(hint.rank, 1);
    }

    #[test]
    fn case_304() {
        let candidates = vec![
            candidate("maxed", 10.0, 1000, ptt::FULL_SCORE, 0),
            candidate("open", 10.0, 1000, 9_900_000, 0),
        ];
        let board = suggest(&candidates, DEFAULT_BONUS_NEED, PAGE_LIMIT);
        assert_eq!(board.hint_count, 1);
        assert_eq!(board.all_hints()[0].song_id, "open");
    }

    #[test]
    fn case_305() {
        let build = || {
            let mut items = Vec::new();
            for index in 0..40 {
                let constant = 9.0 + (index % 22) as f64 * 0.1;
                let note = 800 + index as i64 * 10;
                let score = 9_700_000 + index as i64 * 1000;
                let far = (index % 5) as i64;
                items.push(
                    SuggestCandidate::new(
                        format!("song{index:02}"),
                        format!("Song {index:02}"),
                        Difficulty::ALL[index % 5],
                        constant,
                        note,
                    )
                    .with_score(score)
                    .with_judgement(far + 10, 0),
                );
            }
            items
        };
        let base = signature(&suggest(&build(), DEFAULT_BONUS_NEED, 40));
        let mut seed: u64 = 0x2545_F491_4F6C_DD1D;
        for round in 0..1000 {
            let mut items = build();
            let mut index = items.len();
            while index > 1 {
                seed = seed
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let swap_with = ((seed >> 33) as usize) % index;
                index -= 1;
                items.swap(index, swap_with);
            }
            let current = signature(&suggest(&items, DEFAULT_BONUS_NEED, 40));
            assert_eq!(
                current, base,
                "第 {round} 次打乱之后输出就变了，说明排序不是确定性的"
            );
        }
    }

    #[test]
    fn case_306() {
        let mut items: Vec<SuggestCandidate> = (0..12)
            .map(|index| {
                SuggestCandidate::new(
                    format!("song{index:02}"),
                    "T",
                    Difficulty::Future,
                    10.0,
                    1000,
                )
                .with_score(9_800_000 + index * 100)
                .with_judgement(10, 0)
            })
            .collect();
        let forward = signature(&suggest(&items, DEFAULT_BONUS_NEED, PAGE_LIMIT));
        items.reverse();
        let backward = signature(&suggest(&items, DEFAULT_BONUS_NEED, PAGE_LIMIT));
        assert_eq!(forward, backward);
    }

    #[test]
    fn case_307() {
        let candidates: Vec<SuggestCandidate> = (0..30)
            .map(|index| {
                SuggestCandidate::new(
                    format!("song{index:02}"),
                    "T",
                    Difficulty::Future,
                    10.0,
                    1000,
                )
                .with_score(9_800_000 + index * 100)
                .with_judgement(10, 0)
            })
            .collect();
        let board = suggest(&candidates, DEFAULT_BONUS_NEED, 10);
        assert_eq!(board.pool, 30);
        assert_eq!(board.pushable, 30, "可推条数说的是截断之前的总数");
        assert_eq!(board.hint_count, 10, "分页上限要生效");
        assert!(board.pick_count <= PICK_MAX);
    }

    #[test]
    fn case_308() {
        let board = suggest(&[], DEFAULT_BONUS_NEED, PAGE_LIMIT);
        assert!(board.groups.is_empty());
        assert_eq!(board.hint_count, 0);
        assert_eq!(board.pick_count, 0);
        assert_eq!(board.pool, 0);
        assert_eq!(board.pushable, 0);
    }

    #[test]
    fn case_309() {
        assert!((ptt::single_rating(10.0, 9_800_000) - 11.2).abs() < 1e-9);
        assert_eq!(score_for_bonus(10.0, ptt::MAX_BONUS), ptt::FULL_SCORE);
        assert_eq!(
            score_for_bonus(10.0, 0.0),
            9_440_000,
            "0 个完成度是通关线往下 0.2 档，也就是 944 万"
        );
        assert_eq!(
            score_for_bonus(10.0, -9.0),
            0,
            "连通关线都够不着的完成度不给建议"
        );
        assert_eq!(
            score_for_bonus(10.0, 0.2),
            9_500_000,
            "0.2 个完成度正好是 AA 线"
        );
    }
}
