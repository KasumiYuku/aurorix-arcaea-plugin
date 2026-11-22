use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use aurorix_core::Scene;
use aurorix_plugin::Invocation;
use aurorix_plugin::{Outgoing, PluginError, RenderCard};
use aurorix_template::Args;

use crate::backend::{ArcBackendNext, Song, SongQuery, DEFAULT_SEARCH_LIMIT};
use crate::cards::guess::{GuessInput, GuessStage};
use crate::cards::theme::Theme;
use crate::commands::{bridge, buttons, prefs_of, reply, Ctx};
use crate::model::Region;

pub const STEP_INTERVAL: Duration = Duration::from_secs(15);

pub const LIFETIME: Duration = Duration::from_secs(240);

pub const HINT_COOLDOWN: Duration = Duration::from_secs(20);

pub const PICK_COOLDOWN: Duration = Duration::from_secs(30);

pub const DRAW_ATTEMPTS: u32 = 4;

const HINT_SECTIONS: usize = 3;

const BUSY: &str = "这一局还没结束，先答题或者发 /a guess stop 收局";

const NO_ROUND: &str = "现在没有正在进行的猜曲绘";

const NO_ROUND_START: &str = "现在没有正在进行的猜曲绘，先发 /a guess 开一局";

const WRONG: &str = "不是这个，再想想";

const COOLING: &str = "提示还在冷却，稍等一会";

const CARD_SLOT: &str = "arcaea:guess:card";

const SAID_SLOT: &str = "arcaea:guess:said";

#[derive(Debug, Clone)]
struct Round {
    song_id: String,
    title: String,
    artist: String,
    jacket: String,
    pack: String,
    released: String,
    bpm: String,
    level: String,
    hints: Vec<String>,
    round: u32,
    started: Instant,
    next_step: Instant,
    hint_at: Instant,
    answered: bool,
    seed: i64,
}

enum State {
    None,
    Expired(Round),
    Active(Round),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Hint,
    Stop,
}

static ROUNDS: OnceLock<Mutex<HashMap<String, Round>>> = OnceLock::new();

static PICKS: OnceLock<Mutex<HashMap<String, Instant>>> = OnceLock::new();

fn rounds() -> &'static Mutex<HashMap<String, Round>> {
    ROUNDS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn picks() -> &'static Mutex<HashMap<String, Instant>> {
    PICKS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn state_key(ctx: &Ctx<'_>) -> String {
    match ctx.inv.scene {
        Scene::Group | Scene::Channel => format!("g:{}", ctx.inv.target.id),
        Scene::C2c | Scene::Dm => format!("u:{}", ctx.inv.user),
    }
}

fn step_of(text: &str) -> Option<Step> {
    match text.trim().to_lowercase().as_str() {
        "hint" | "提示" | "线索" => Some(Step::Hint),
        "stop" | "end" | "quit" | "over" | "结束" | "收局" | "不玩了" | "停止" => {
            Some(Step::Stop)
        }
        _ => None,
    }
}

fn lapsed(round: &Round) -> bool {
    round.started.elapsed() > LIFETIME
}

fn inspect(key: &str) -> State {
    let Ok(mut map) = rounds().lock() else {
        return State::None;
    };
    let Some(round) = map.get(key) else {
        return State::None;
    };
    if !round.answered && !lapsed(round) {
        return State::Active(round.clone());
    }
    let Some(round) = map.remove(key) else {
        return State::None;
    };
    if round.answered {
        State::None
    } else {
        State::Expired(round)
    }
}

fn finish(key: &str) -> Option<Round> {
    let mut map = rounds().lock().ok()?;
    let mut round = map.remove(key)?;
    round.answered = true;
    Some(round)
}

fn last_pick(key: &str) -> Option<Instant> {
    picks().lock().ok()?.get(key).copied()
}

fn remember_pick(key: &str) {
    if let Ok(mut map) = picks().lock() {
        map.insert(key.to_string(), Instant::now());
    }
}

fn seed_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos() as i64)
        .unwrap_or_default()
}

fn step_span(round: &Round) -> f64 {
    round
        .next_step
        .saturating_duration_since(round.started)
        .as_secs_f64()
        .max(1.0)
}

fn reveal_level(round: &Round) -> f64 {
    (round.started.elapsed().as_secs_f64() / step_span(round)).clamp(0.0, 1.0)
}

fn pack_label(song: &Song) -> String {
    if !song.pack_name_zh.trim().is_empty() {
        return song.pack_name_zh.trim().to_string();
    }
    song.pack_name.trim().to_string()
}

fn release_label(release_ts: i64) -> String {
    if release_ts <= 0 {
        return String::new();
    }
    match chrono::DateTime::from_timestamp(release_ts, 0) {
        Some(at) => at
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d")
            .to_string(),
        None => String::new(),
    }
}

fn level_label(song: &Song) -> String {
    song.charts
        .iter()
        .filter_map(|chart| chart.constant)
        .filter(|constant| *constant > 0.0)
        .fold(None::<f64>, |best, constant| match best {
            Some(value) if value >= constant => Some(value),
            _ => Some(constant),
        })
        .map(|constant| format!("{constant:.1}"))
        .unwrap_or_default()
}

fn hint_tiers(round: &Round) -> Vec<String> {
    let mut tiers: Vec<String> = Vec::new();
    let mut first: Vec<String> = Vec::new();
    if !round.pack.is_empty() {
        first.push(format!("曲包：{}", round.pack));
    }
    if !round.released.is_empty() {
        first.push(format!("上架：{}", round.released));
    }
    if !first.is_empty() {
        tiers.push(first.join(" · "));
    }
    let mut second: Vec<String> = Vec::new();
    if !round.bpm.is_empty() {
        second.push(format!("BPM：{}", round.bpm));
    }
    if !round.level.is_empty() {
        second.push(format!("最高定数：{}", round.level));
    }
    if !second.is_empty() {
        tiers.push(second.join(" · "));
    }
    if !round.artist.trim().is_empty() {
        tiers.push(format!("曲师：{}", round.artist.trim()));
    }
    tiers.truncate(HINT_SECTIONS);
    tiers
}

fn next_hint(round: &Round) -> Option<String> {
    hint_tiers(round).into_iter().nth(round.hints.len())
}

fn same_song(song: &Song, round: &Round) -> bool {
    song.song_id == round.song_id || song.title.eq_ignore_ascii_case(round.title.as_str())
}

fn input_of(round: &Round, stage: GuessStage, theme: Theme) -> GuessInput {
    let mut song = crate::domain::Song::new(round.song_id.clone(), round.title.clone());
    song.artist = round.artist.clone();
    let hints: Vec<&str> = round.hints.iter().map(String::as_str).collect();
    GuessInput::of(&song, stage)
        .with_jacket(round.jacket.clone())
        .with_seed(round.seed)
        .with_round(round.round)
        .with_theme(theme)
        .with_hints(hints.as_slice())
        .with_level(reveal_level(round))
}

fn caption_of(id: &str, fallback: &str, args: &Args) -> Outgoing {
    reply::filled(id, fallback, args)
}

fn over_caption(round: &Round, head: &str) -> Outgoing {
    let mut args = reply::args();
    reply::put(&mut args, "head", head);
    reply::put(&mut args, "title", round.title.clone());
    reply::put(&mut args, "hints", round.hints.join("\n"));
    reply::filled("guess/guess.over", head, &args)
}

async fn theme_of(ctx: &Ctx<'_>) -> Theme {
    match ctx.repo() {
        Some(repo) => bridge::theme_of(&prefs_of(ctx, &repo).await),
        None => Theme::DEFAULT,
    }
}

async fn draw(ctx: &Ctx<'_>) -> Result<Song, Box<Outgoing>> {
    let mut failure: Option<String> = None;
    for _ in 0..DRAW_ATTEMPTS {
        let query = SongQuery::new("").with_sort("random").with_limit(1);
        match ctx.backend.songs(ctx.region, query).await {
            Ok(search) => {
                if let Some(hit) = search.items.into_iter().next() {
                    return Ok(hit.song);
                }
            }
            Err(error) => failure = Some(error.to_string()),
        }
    }
    Err(Box::new(match failure {
        Some(message) => reply::fail(message),
        None => reply::fail("曲库里没抽到合适的题目，稍后再试"),
    }))
}

async fn reveal(ctx: &Ctx<'_>, round: &Round, answer: &str, correct: bool) -> RenderCard {
    let theme = theme_of(ctx).await;
    let mut input = input_of(round, GuessStage::Answer, theme)
        .with_answer(answer, correct)
        .with_level(1.0);
    input.cost = round.hints.len() as u32;
    crate::cards::guess::card(input)
}

pub async fn guess(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    let key = state_key(ctx);
    let active = match inspect(&key) {
        State::None => None,
        State::Expired(round) => {
            let card = reveal(ctx, &round, round.title.as_str(), false).await;
            return Ok(Some(
                reply::board(
                    reply::card(over_caption(&round, "时间到"), card),
                    buttons::quiz_again_board(ctx.region, &round.title),
                )
                .supersede(CARD_SLOT),
            ));
        }
        State::Active(round) => Some(round),
    };
    let text = arg.trim();
    if let Some(step) = step_of(text) {
        let Some(round) = active else {
            let message = if step == Step::Hint {
                NO_ROUND_START
            } else {
                NO_ROUND
            };
            return Ok(Some(reply::notice(message).supersede(SAID_SLOT)));
        };
        return match step {
            Step::Hint => hint_step(ctx, &key, round).await,
            Step::Stop => Ok(Some(stop_step(ctx, &key, round).await)),
        };
    }
    if text.is_empty() {
        return match active {
            Some(_) => Ok(Some(reply::notice(BUSY).supersede(SAID_SLOT))),
            None => open(ctx, &key).await,
        };
    }
    let Some(round) = active else {
        return Ok(Some(reply::notice(NO_ROUND_START).supersede(SAID_SLOT)));
    };
    answer_step(ctx, &key, round, text).await
}

fn peek_round(key: &str) -> Option<Round> {
    rounds().lock().ok()?.get(key).cloned()
}

fn bump_step(key: &str) {
    if let Ok(mut map) = rounds().lock() {
        if let Some(round) = map.get_mut(key) {
            round.next_step += STEP_INTERVAL;
        }
    }
}

pub fn spawn_ticker(inv: Invocation, region: Region, backend: ArcBackendNext, key: String) {
    let Some(sink) = inv.push_sink().cloned() else {
        return;
    };
    let target = inv.target.clone();
    tokio::spawn(async move {
        let ctx = Ctx::new(&inv, region, &backend);
        loop {
            let Some(round) = peek_round(&key) else {
                return;
            };
            let wait = round
                .next_step
                .saturating_duration_since(Instant::now())
                .max(Duration::from_secs(1));
            tokio::time::sleep(wait).await;
            let Some(round) = peek_round(&key) else {
                return;
            };
            if lapsed(&round) {
                let Some(round) = finish(&key) else {
                    return;
                };
                let card = reveal(&ctx, &round, round.title.as_str(), false).await;
                let out = reply::board(
                    reply::card(over_caption(&round, "时间到"), card),
                    buttons::quiz_again_board(region, &round.title),
                )
                .supersede(CARD_SLOT);
                let _ = sink.push(target.clone(), out).await;
                return;
            }
            bump_step(&key);
            let left = LIFETIME.saturating_sub(round.started.elapsed()).as_secs();
            let theme = theme_of(&ctx).await;
            let card = crate::cards::guess::card(input_of(&round, GuessStage::Question, theme));
            let mut args = reply::args();
            reply::put(&mut args, "step", format!("还剩 {left} 秒"));
            reply::put(&mut args, "hints", round.hints.join("\n"));
            let caption = reply::filled("guess/guess.step", "再看一眼这张曲绘", &args);
            let out = reply::board(reply::card(caption, card), buttons::quiz_board(region))
                .supersede(CARD_SLOT);
            if sink.push(target.clone(), out).await.is_err() {
                return;
            }
        }
    });
}

async fn open(ctx: &Ctx<'_>, key: &str) -> Result<Option<Outgoing>, PluginError> {
    if let Some(at) = last_pick(key) {
        let waited = at.elapsed();
        if waited < PICK_COOLDOWN {
            let seconds = (PICK_COOLDOWN - waited).as_secs() + 1;
            return Ok(Some(
                reply::notice(format!("刚抽过题，{seconds} 秒后再开新一局")).supersede(SAID_SLOT),
            ));
        }
    }
    let song = match draw(ctx).await {
        Ok(song) => song,
        Err(outgoing) => return Ok(Some(*outgoing)),
    };
    let now = Instant::now();
    let round = Round {
        song_id: song.song_id.clone(),
        title: song.title.clone(),
        artist: song.artist.clone(),
        jacket: format!("jackets/{}.jpg", song.song_id),
        pack: pack_label(&song),
        released: release_label(song.release_ts),
        bpm: song.bpm.trim().to_string(),
        level: level_label(&song),
        hints: Vec::new(),
        round: 1,
        started: now,
        next_step: now + STEP_INTERVAL,
        hint_at: now,
        answered: false,
        seed: seed_now(),
    };
    if let Ok(mut map) = rounds().lock() {
        map.insert(key.to_string(), round.clone());
    }
    remember_pick(key);
    spawn_ticker(
        ctx.inv.clone(),
        ctx.region,
        ctx.backend.clone(),
        key.to_string(),
    );
    let theme = theme_of(ctx).await;
    let card = crate::cards::guess::card(input_of(&round, GuessStage::Question, theme));
    let caption = caption_of(
        "guess/guess.start",
        "猜曲绘开始了：看这张受损的曲绘，猜出它是哪首歌",
        &reply::args(),
    );
    Ok(Some(
        reply::board(
            reply::card(caption, card),
            buttons::quiz_board(ctx.region),
        )
        .supersede(CARD_SLOT),
    ))
}

async fn answer_step(
    ctx: &Ctx<'_>,
    key: &str,
    round: Round,
    text: &str,
) -> Result<Option<Outgoing>, PluginError> {
    let query = SongQuery::new(text).with_limit(DEFAULT_SEARCH_LIMIT);
    let search = match ctx.backend.songs(ctx.region, query).await {
        Ok(search) => search,
        Err(error) => return Ok(Some(reply::fail(error.user_message()))),
    };
    if !search
        .items
        .iter()
        .any(|hit| same_song(&hit.song, &round))
    {
        return Ok(Some(reply::notice(WRONG).supersede(SAID_SLOT)));
    }
    let round = finish(key).unwrap_or(round);
    let card = reveal(ctx, &round, text, true).await;
    Ok(Some(
        reply::board(
            reply::card(over_caption(&round, "🎉 恭喜回答正确"), card),
            buttons::quiz_again_board(ctx.region, &round.title),
        )
        .supersede(CARD_SLOT),
    ))
}

async fn hint_step(
    ctx: &Ctx<'_>,
    key: &str,
    mut round: Round,
) -> Result<Option<Outgoing>, PluginError> {
    if round.hint_at.elapsed() < HINT_COOLDOWN {
        return Ok(Some(reply::notice(COOLING).supersede(SAID_SLOT)));
    }
    let Some(hint) = next_hint(&round) else {
        return Ok(Some(reply::notice("这首歌的提示已经给完了").supersede(SAID_SLOT)));
    };
    round.hints.push(hint);
    round.hint_at = Instant::now();
    let theme = theme_of(ctx).await;
    let card = crate::cards::guess::card(input_of(&round, GuessStage::Question, theme));
    let mut args = reply::args();
    reply::put(&mut args, "hints", round.hints.join("\n"));
    let caption = caption_of("guess/guess.hint", "提示", &args);
    if let Ok(mut map) = rounds().lock() {
        if map.contains_key(key) {
            map.insert(key.to_string(), round);
        }
    }
    Ok(Some(
        reply::board(
            reply::card(caption, card),
            buttons::quiz_board(ctx.region),
        )
        .supersede(CARD_SLOT),
    ))
}

async fn stop_step(ctx: &Ctx<'_>, key: &str, round: Round) -> Outgoing {
    let round = finish(key).unwrap_or(round);
    let card = reveal(ctx, &round, round.title.as_str(), false).await;
    reply::board(
        reply::card(over_caption(&round, "这一局结束了"), card),
        buttons::quiz_again_board(ctx.region, &round.title),
    )
    .supersede(CARD_SLOT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{ArcBackendNext, BackendConfig, Song as DtoSong};
    use crate::model::Region;
    use aurorix_core::Target;
    use aurorix_plugin::Invocation;

    fn backend() -> &'static ArcBackendNext {
        static BACKEND: OnceLock<ArcBackendNext> = OnceLock::new();
        BACKEND.get_or_init(|| {
            ArcBackendNext::new(BackendConfig::new("http://127.0.0.1:9")).expect("后端配置要能构建")
        })
    }

    fn round_at(started: Instant) -> Round {
        Round {
            song_id: "fractureray".to_string(),
            title: "Fracture Ray".to_string(),
            artist: "sakuzyo".to_string(),
            jacket: "jackets/fractureray.jpg".to_string(),
            pack: "Absolute Reason".to_string(),
            released: "2019-01-01".to_string(),
            bpm: "160".to_string(),
            level: "10.8".to_string(),
            hints: Vec::new(),
            round: 1,
            started,
            next_step: started + STEP_INTERVAL,
            hint_at: started,
            answered: false,
            seed: 7,
        }
    }

    fn key_at(scene: Scene, target: &str, user: &str) -> String {
        let inv = Invocation {
            scene,
            target: Target::new(scene, target),
            user: user.to_string(),
            ..Invocation::default()
        };
        let ctx = Ctx::new(&inv, Region::Intl, backend());
        state_key(&ctx)
    }

    #[test]
    fn case_043() {
        assert_eq!(key_at(Scene::Group, "g1", "u1"), "g:g1");
        assert_eq!(key_at(Scene::Channel, "c9", "u1"), "g:c9");
        assert_eq!(key_at(Scene::C2c, "u1", "u5"), "u:u5");
        assert_eq!(key_at(Scene::Dm, "u1", "u7"), "u:u7");
    }

    #[test]
    fn case_044() {
        assert!(!lapsed(&round_at(Instant::now())));
        let old = Instant::now() - LIFETIME - Duration::from_secs(1);
        assert!(lapsed(&round_at(old)), "超过生命周期必须判成过期");
    }

    #[test]
    fn case_045() {
        for text in ["hint", " 提示 ", "线索"] {
            assert_eq!(step_of(text), Some(Step::Hint), "{text} 必须能要提示");
        }
        for text in [
            "stop",
            "end",
            "quit",
            "over",
            "结束",
            "收局",
            "不玩了",
            "停止",
        ] {
            assert_eq!(step_of(text), Some(Step::Stop), "{text} 必须能收局");
        }
        assert_eq!(step_of("fracture ray"), None);
        assert_eq!(step_of(""), None);
    }

    #[test]
    fn case_046() {
        let bare = DtoSong {
            artist: "sakuzyo".to_string(),
            ..DtoSong::default()
        };
        assert_eq!(pack_label(&bare), "");
        assert_eq!(release_label(0), "");
        assert_eq!(level_label(&bare), "");
        let mut round = round_at(Instant::now());
        round.pack = String::new();
        round.released = String::new();
        round.bpm = String::new();
        round.level = String::new();
        assert_eq!(hint_tiers(&round), vec!["曲师：sakuzyo".to_string()]);
        round.hints.push("曲师：sakuzyo".to_string());
        assert_eq!(next_hint(&round), None, "三档给完就没了");
        let full = round_at(Instant::now());
        assert_eq!(
            next_hint(&full),
            Some("曲包：Absolute Reason · 上架：2019-01-01".to_string())
        );
    }

    #[test]
    fn case_047() {
        let round = round_at(Instant::now());
        let by_id = DtoSong {
            song_id: "fractureray".to_string(),
            ..DtoSong::default()
        };
        assert!(same_song(&by_id, &round));
        let by_title = DtoSong {
            song_id: "other".to_string(),
            title: "fracture ray".to_string(),
            ..DtoSong::default()
        };
        assert!(same_song(&by_title, &round));
        let other = DtoSong {
            song_id: "sheriruth".to_string(),
            title: "Sheriruth".to_string(),
            ..DtoSong::default()
        };
        assert!(!same_song(&other, &round));
    }

    #[test]
    fn case_048() {
        let (width, height) = GuessStage::Question.size();
        assert_eq!((width, height), (739, 1006));
        let spec = crate::cards::spec_of(crate::cards::CardKind::Guess);
        assert!(spec.accepts(width, height), "{width}x{height} 不合规格");
    }
}
