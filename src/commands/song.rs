use aurorix_plugin::{Outgoing, PluginError};

use crate::backend as dto;
use crate::backend::SongQuery;
use crate::cards::select::{SelectPlayer, SelectRow};
use crate::commands::bridge;
use crate::commands::{buttons, one, prefs_of, reply, repo_of, root_of, split_difficulty, Ctx};
use crate::domain;
use crate::domain::SongMatch;
use crate::model::Difficulty;

pub const SONG_BOARD_LIMIT: u32 = 5;

pub async fn song(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    let (query, wanted) = split_difficulty(arg);
    let query = query.as_str();
    if query.trim().is_empty() {
        return Ok(Some(reply::notice(format!(
            "用法：{} song <曲目>",
            root_of(ctx)
        ))));
    }
    let song = match resolve_for(ctx, arg, query, wanted).await {
        Resolved::Song(_, song) => *song,
        Resolved::Pick(matched) => {
            return Ok(Some(reply::pick(root_of(ctx), "song", query, &matched)))
        }
        Resolved::Missing => return Ok(Some(reply::notice("没有找到这个曲目"))),
        Resolved::Failed(error) => return Ok(Some(reply::fail(error))),
    };
    let theme = theme_of(ctx).await;
    let difficulty = wanted
        .or_else(|| preferred_chart(&song))
        .unwrap_or(Difficulty::Future);
    let id = song.song_id.clone();
    let mut input = bridge::song_input(&song, theme);
    let mut boards = Vec::new();
    let mut levels: Vec<Difficulty> = song
        .charts
        .iter()
        .filter_map(|chart| Difficulty::from_upstream(i64::from(chart.rating_class)))
        .collect();
    levels.sort_by_key(|level| std::cmp::Reverse(crate::model::diff_order_weight(*level)));
    let song_id = id.as_str();
    let fetched = futures_util::future::join_all(levels.into_iter().map(|level| async move {
        ctx.backend
            .song_boards(ctx.region, song_id, level, SONG_BOARD_LIMIT)
            .await
            .ok()
            .map(|board| bridge::song_boards(&board, level))
    }))
    .await;
    for rows in fetched.into_iter().flatten() {
        boards.extend(rows);
    }
    if !boards.is_empty() {
        input = input.with_boards(boards);
    }
    Ok(Some(reply::board(
        reply::card(
            reply::text("song/song.profile.caption", &reply::args()),
            crate::cards::song::card(input),
        ),
        buttons::detail_board(ctx.region, &id, difficulty),
    )))
}

pub async fn alias(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    let (query, _) = split_difficulty(arg);
    let query = query.as_str();
    if query.trim().is_empty() {
        return Ok(Some(reply::notice(format!(
            "用法：{} alias <曲目>",
            root_of(ctx)
        ))));
    }
    let found = match bridge::resolve_song(ctx.backend, ctx.region, query).await {
        Ok(Some(found)) => found,
        Ok(None) => return Ok(Some(reply::notice("没有找到这个曲目"))),
        Err(error) => return Ok(Some(reply::fail(error.user_message()))),
    };
    let song = &found.matched.best.song;
    if song.aliases.is_empty() {
        return Ok(Some(reply::notice("这首曲目还没有收录别名")));
    }
    let mut args = reply::args();
    reply::put(&mut args, "title", song.title.clone());
    reply::put(&mut args, "aliases", song.aliases.join(" / "));
    for note in found.matched.notes(query) {
        reply::note(&mut args, note);
    }
    Ok(Some(reply::text("song/song.alias", &args)))
}

pub async fn ill(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    let (query, wanted) = split_difficulty(arg);
    let query = query.as_str();
    let (matched, song) = match resolve_for(ctx, arg, query, wanted).await {
        Resolved::Song(matched, song) => (matched, *song),
        Resolved::Pick(matched) => {
            return Ok(Some(reply::pick(root_of(ctx), "ill", query, &matched)))
        }
        Resolved::Missing => return Ok(Some(reply::notice("没有找到这个曲目"))),
        Resolved::Failed(error) => return Ok(Some(reply::fail(error))),
    };
    let difficulty = wanted
        .or_else(|| preferred_chart(&song))
        .unwrap_or(Difficulty::Future);
    if let Some(missing) = missing_difficulty(&song, difficulty, wanted.is_some()) {
        return Ok(Some(missing));
    }
    let url = cover_url(&song, difficulty, wanted.is_some());
    if url.is_empty() {
        return Ok(Some(reply::notice(format!(
            "《{}》还没有收录曲绘",
            song.title
        ))));
    }
    let mut args = reply::args();
    reply::put(&mut args, "title", song.title.clone());
    for note in matched.notes(query) {
        reply::note(&mut args, note);
    }
    Ok(Some(reply::image(
        reply::text("song/song.ill", &args),
        url,
        song.title.clone(),
    )))
}

pub async fn play(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    let (query, wanted) = split_difficulty(arg);
    let query = query.as_str();
    let song = match resolve_for(ctx, arg, query, wanted).await {
        Resolved::Song(_, song) => *song,
        Resolved::Pick(matched) => {
            return Ok(Some(reply::pick(root_of(ctx), "play", query, &matched)))
        }
        Resolved::Missing => return Ok(Some(reply::notice("没有找到这个曲目"))),
        Resolved::Failed(error) => return Ok(Some(reply::fail(error))),
    };
    let difficulty = wanted
        .or_else(|| preferred_chart(&song))
        .unwrap_or(Difficulty::Future);
    if let Some(missing) = missing_difficulty(&song, difficulty, wanted.is_some()) {
        return Ok(Some(missing));
    }
    let url = audio_url(&song, difficulty, wanted.is_some());
    if url.is_empty() {
        return Ok(Some(reply::notice(format!(
            "《{}》还没有收录音频",
            song.title
        ))));
    }
    let mut args = reply::args();
    reply::put(&mut args, "title", song.title.clone());
    reply::put(&mut args, "artist", song.artist.clone());
    reply::put(
        &mut args,
        "cover",
        cover_url(&song, difficulty, wanted.is_some()),
    );
    Ok(Some(reply::text("song/song.play", &args).voice(url)))
}

pub async fn rand(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    let (raw, wanted) = split_difficulty(arg);
    let raw = raw.as_str();
    let parsed = match domain::Filter::parse(raw) {
        Ok(parsed) => parsed,
        Err(_) => Some(domain::Filter::text_only(raw)),
    };
    let text = parsed
        .as_ref()
        .map(|filter| filter.text.clone())
        .unwrap_or_default();
    let dsl = parsed
        .as_ref()
        .map(domain::Filter::to_dsl)
        .unwrap_or_default();
    let mut query = SongQuery::new(text)
        .with_sort("random")
        .with_limit(1);
    if !dsl.is_empty() {
        query = query.with_filter(dsl);
    }
    let result = match ctx.backend.songs(ctx.region, query).await {
        Ok(result) => result,
        Err(error) => return Ok(Some(reply::fail(error.user_message()))),
    };
    let Some(hit) = result.items.first() else {
        let condition = parsed
            .as_ref()
            .map(domain::Filter::describe)
            .unwrap_or_default();
        let condition = if condition.is_empty() {
            "曲库为空".to_string()
        } else {
            condition
        };
        return Ok(Some(reply::notice(format!(
            "没有符合条件的曲目（{condition}），可以放宽一点条件再试"
        ))));
    };
    let picked = hit.song.clone();
    let Some(difficulty) = wanted.or_else(|| preferred_chart(&picked)) else {
        return Ok(Some(reply::notice("这首曲目还没有录入谱面")));
    };
    let rows = nearby_rows(ctx, &picked, difficulty).await;
    let player = rand_player(ctx).await;
    let theme = theme_of(ctx).await;
    let input = bridge::select_input(
        ctx.region,
        &picked,
        difficulty,
        theme,
        player,
        rows,
        None,
    );
    Ok(Some(reply::board(
        reply::card(reply::bare(), crate::cards::select::card(input)),
        buttons::rand_board(ctx.region, raw, &picked.title),
    )))
}

pub async fn search(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    let query = arg.trim();
    if query.is_empty() {
        return Ok(Some(reply::notice(format!(
            "用法：{} search <关键词或条件>",
            root_of(ctx)
        ))));
    }
    let parsed = match domain::Filter::parse(query) {
        Ok(parsed) => parsed,
        Err(_) => Some(domain::Filter::text_only(query)),
    };
    let text = parsed
        .as_ref()
        .map(|filter| filter.text.clone())
        .unwrap_or_else(|| query.to_string());
    let dsl = parsed
        .as_ref()
        .map(domain::Filter::to_dsl)
        .unwrap_or_default();
    let mut song_query = SongQuery::new(text.clone());
    if !dsl.is_empty() {
        song_query = song_query.with_filter(dsl);
    }
    let mut result = match ctx.backend.songs(ctx.region, song_query).await {
        Ok(result) => result,
        Err(error) => return Ok(Some(reply::fail(error.user_message()))),
    };
    let mut retried = false;
    if result.items.is_empty() && text != query {
        if let Ok(alt) = ctx.backend.songs(ctx.region, SongQuery::new(query)).await {
            if !alt.items.is_empty() {
                result = alt;
                retried = true;
            }
        }
    }
    if result.items.is_empty() {
        return Ok(Some(reply::notice(format!(
            "没有符合条件的曲目（{}），可以放宽一点条件再试",
            query
        ))));
    }
    let theme = theme_of(ctx).await;
    let limit = result.items.len();
    let mut input = bridge::search_input(query, &result.items, theme, limit);
    if retried {
        input = input.with_note("没有命中任何条件，已把整条输入当作曲名重新检索了一次");
    }
    Ok(Some(reply::board(
        reply::card(
            reply::text("song/song.search.caption", &reply::args()),
            crate::cards::search::card(input),
        ),
        buttons::search_board(ctx.region, query),
    )))
}

pub async fn chart(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    let (query, wanted) = split_difficulty(arg);
    let query = query.as_str();
    let (matched, song) = match resolve_for(ctx, arg, query, wanted).await {
        Resolved::Song(matched, song) => (matched, *song),
        Resolved::Pick(matched) => {
            return Ok(Some(reply::pick(root_of(ctx), "chart", query, &matched)))
        }
        Resolved::Missing => return Ok(Some(reply::notice("没有找到这个曲目"))),
        Resolved::Failed(error) => return Ok(Some(reply::fail(error))),
    };
    let difficulty = wanted
        .or_else(|| preferred_chart(&song))
        .unwrap_or(Difficulty::Future);
    if let Some(missing) = missing_difficulty(&song, difficulty, wanted.is_some()) {
        return Ok(Some(missing));
    }
    let Some(chart) = song.chart(difficulty) else {
        return Ok(Some(reply::notice(format!(
            "《{}》还没有收录谱面",
            song.title
        ))));
    };
    let mut args = reply::args();
    reply::put(&mut args, "title", song.title.clone());
    reply::put(&mut args, "chart_designer", chart.chart_designer.clone());
    reply::put(&mut args, "difficulty", difficulty.as_str());
    let constant = match chart.constant {
        Some(value) if value > 0.0 => one(value),
        _ => "-".to_string(),
    };
    reply::put(&mut args, "constant", constant);
    let has_preview = chart.chart_url.starts_with("http://") || chart.chart_url.starts_with("https://");
    for note in matched.notes(query) {
        reply::note(&mut args, note);
    }
    if !has_preview {
        reply::note(&mut args, "这张谱面还没有收录预览图");
    }
    let outgoing = reply::text("chart/chart.preview", &args);
    let outgoing = if has_preview {
        reply::image(
            outgoing,
            chart.chart_url.clone(),
            format!("{} {}", song.title, difficulty.as_str()),
        )
    } else {
        outgoing
    };
    Ok(Some(reply::board(
        outgoing,
        buttons::chart_board(ctx.region, query, difficulty),
    )))
}

pub async fn cal(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    let words: Vec<&str> = arg.split_whitespace().collect();
    let Ok(result) = domain::parse_cal_args(&words) else {
        return Ok(Some(reply::notice(format!(
            "用法：{} cal <定数> <分数>",
            root_of(ctx)
        ))));
    };
    let mut args = reply::args();
    reply::put(&mut args, "constant", one(result.constant));
    reply::put(&mut args, "score", result.score.to_string());
    reply::put(&mut args, "rating", domain::format_rating(result.rating));
    Ok(Some(reply::text("calc/calc.ptt", &args)))
}

enum Resolved {
    Missing,
    Failed(String),
    Pick(SongMatch),
    Song(SongMatch, Box<dto::Song>),
}

async fn resolve_for(
    ctx: &Ctx<'_>,
    full: &str,
    query: &str,
    wanted: Option<Difficulty>,
) -> Resolved {
    let found = match bridge::resolve_song_full(ctx.backend, ctx.region, query, full).await {
        Ok(Some(found)) => found,
        Ok(None) => return Resolved::Missing,
        Err(error) => return Resolved::Failed(error.to_string()),
    };
    if found.matched.needs_pick(query) {
        return Resolved::Pick(found.matched);
    }
    let matched = match wanted {
        Some(difficulty) => found.matched.prefer_chart(difficulty),
        None => found.matched,
    };
    let best = matched.best.song.id.clone();
    match found.songs.into_iter().find(|song| song.song_id == best) {
        Some(song) => Resolved::Song(matched, Box::new(song)),
        None => Resolved::Missing,
    }
}

fn missing_difficulty(song: &dto::Song, difficulty: Difficulty, pinned: bool) -> Option<Outgoing> {
    if !pinned || song.chart(difficulty).is_some() {
        return None;
    }
    let mut args = reply::args();
    reply::put(&mut args, "title", song.title.clone());
    reply::put(
        &mut args,
        "reason",
        format!("《{}》没有 {} 难度", song.title, difficulty.as_str()),
    );
    for chart in &song.charts {
        let label = chart
            .difficulty_kind()
            .map(|value| value.as_str())
            .unwrap_or(chart.difficulty.as_str());
        let constant = match chart.constant {
            Some(value) if value > 0.0 => one(value),
            _ => "-".to_string(),
        };
        reply::row(
            &mut args,
            "charts",
            &[("label", label.to_string()), ("constant", constant)],
        );
    }
    Some(reply::text("chart/chart.none", &args))
}

fn cover_url(song: &dto::Song, difficulty: Difficulty, pinned: bool) -> String {
    if pinned {
        if let Some(chart) = song.chart(difficulty) {
            if !chart.ill_url.is_empty() {
                return chart.ill_url.clone();
            }
        }
    }
    if let Some(chart) = song.chart(Difficulty::Future) {
        if !chart.ill_url.is_empty() {
            return chart.ill_url.clone();
        }
    }
    song.charts
        .iter()
        .map(|chart| chart.ill_url.clone())
        .find(|url| !url.is_empty())
        .unwrap_or_default()
}

fn audio_url(song: &dto::Song, difficulty: Difficulty, pinned: bool) -> String {
    if pinned {
        if let Some(chart) = song.chart(difficulty) {
            if !chart.audio_url.is_empty() {
                return chart.audio_url.clone();
            }
        }
    }
    if let Some(chart) = song.chart(Difficulty::Future) {
        if !chart.audio_url.is_empty() {
            return chart.audio_url.clone();
        }
    }
    if !song.audio_url.is_empty() {
        return song.audio_url.clone();
    }
    song.charts
        .iter()
        .map(|chart| chart.audio_url.clone())
        .find(|url| !url.is_empty())
        .unwrap_or_default()
}

async fn theme_of(ctx: &Ctx<'_>) -> crate::cards::theme::Theme {
    match repo_of(ctx).await {
        Ok(repo) => bridge::theme_of(&prefs_of(ctx, &repo).await),
        Err(_) => crate::cards::theme::Theme::DEFAULT,
    }
}

fn preferred_chart(song: &dto::Song) -> Option<Difficulty> {
    let has = |wanted: Difficulty| {
        song.charts
            .iter()
            .any(|chart| chart.difficulty_kind() == Some(wanted))
    };
    if has(Difficulty::Future) {
        return Some(Difficulty::Future);
    }
    for wanted in [
        Difficulty::Eternal,
        Difficulty::Beyond,
        Difficulty::Present,
        Difficulty::Past,
    ] {
        if has(wanted) {
            return Some(wanted);
        }
    }
    song.charts
        .first()
        .and_then(|chart| chart.difficulty_kind())
}

fn constant_of(song: &dto::Song, difficulty: Difficulty) -> f64 {
    song.chart(difficulty)
        .and_then(|chart| chart.constant)
        .unwrap_or(0.0)
}

fn select_row(song: &dto::Song, difficulty: Difficulty, selected: bool) -> SelectRow {
    let chart = song.chart(difficulty);
    let level = chart
        .map(|chart| chart.rating_display.clone())
        .unwrap_or_default();
    let constant = chart.and_then(|chart| chart.constant).unwrap_or(0.0);
    SelectRow::new(song.title.clone(), difficulty)
        .with_song_id(song.song_id.clone())
        .with_artist(song.artist_display(difficulty))
        .with_level(level, constant)
        .with_selected(selected)
        .with_bpm(song.bpm_display(difficulty))
}

async fn nearby_rows(ctx: &Ctx<'_>, picked: &dto::Song, difficulty: Difficulty) -> Vec<SelectRow> {
    let constant = constant_of(picked, difficulty);
    let mut pool: Vec<dto::Song> = Vec::new();
    if constant > 0.0 {
        let filter = format!(
            "constant>={:.1} && constant<={:.1}",
            (constant - 0.3).max(0.0),
            constant + 0.3
        );
        match ctx
            .backend
            .songs(
                ctx.region,
                SongQuery::new("").with_filter(filter).with_limit(24),
            )
            .await
        {
            Ok(result) => pool = result.items.into_iter().map(|hit| hit.song).collect(),
            Err(_) => pool.clear(),
        }
    }
    if pool.is_empty() {
        if let Ok(result) = ctx
            .backend
            .songs(ctx.region, SongQuery::new("").with_limit(60))
            .await
        {
            pool = result.items.into_iter().map(|hit| hit.song).collect();
        }
    }
    pool.retain(|song| song.song_id != picked.song_id);
    pool.sort_by(|left, right| {
        let left_gap = (constant_of(left, difficulty) - constant).abs();
        let right_gap = (constant_of(right, difficulty) - constant).abs();
        left_gap
            .total_cmp(&right_gap)
            .then_with(|| left.song_id.cmp(&right.song_id))
    });
    pool.truncate(8);
    let middle = pool.len() / 2;
    pool.insert(middle, picked.clone());
    pool.iter()
        .map(|song| select_row(song, difficulty, song.song_id == picked.song_id))
        .collect()
}

async fn rand_player(ctx: &Ctx<'_>) -> Option<SelectPlayer> {
    let repo = repo_of(ctx).await.ok()?;
    let account = repo
        .get_account(ctx.region, &ctx.inv.user)
        .await
        .ok()
        .flatten()?;
    let uid = account.uid()?;
    let player = ctx.backend.player(ctx.region, uid).await.ok().flatten()?;
    let prefs = crate::commands::prefs_of(ctx, &repo).await;
    let identity = player.identity.named(account.name.as_str());
    Some(bridge::select_player_of(&identity, &prefs))
}
