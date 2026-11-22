use aurorix_arcaea_plugin::backend::{
    B50Payload, PlayPayload, Player, PlayerBoard, PlayerIdentity, ProfileBoardRow, RecentPayload,
};
use aurorix_arcaea_plugin::cards::avatar::resolved_frame;
use aurorix_arcaea_plugin::commands::bridge;
use aurorix_arcaea_plugin::model::{Prefs, Region};

const CHARACTER: i32 = 102;

fn json(text: &str) -> serde_json::Value {
    serde_json::from_str(text).expect("样例 JSON 必须合法")
}

fn prefs(char: &str, banner: &str) -> Prefs {
    Prefs {
        char: char.to_string(),
        banner: banner.to_string(),
        theme: String::new(),
    }
}

fn b50_payload() -> B50Payload {
    let raw = json(
        r#"{"name":"wyndbz","current_ptt":12.993,"rating_frame":6,"rating_accent":6,
            "global_rank":22,"best50":12.44,"best10":12.80,"max_ptt":12.99,"count":50,
            "character":102,"char_uncapped":false,"char_override":false,
            "banner":"online_banner_2026_10","records":[]}"#,
    );
    serde_json::from_value(raw).expect("b50 载荷要能解析")
}

fn board_payload(kind: &str) -> PlayerBoard {
    let raw = json(&format!(
        r#"{{"kind":"{kind}","matched":1177,"known":1603,"total":1899,
            "has_curve":false,"anchor":0,"anchor_sd":0,"anchor_plays":0,"anchor_upper":0,
            "anchor_source":"","calibrated":false,"band_counts":{{}},"avg_best10":12.80,
            "avg_best50":12.44,"best_count":50,"over_count":0,"mean_gap":0.07,
            "mean_pressure":0,"name":"wyndbz","current_ptt":12.993,"rating_frame":6,
            "rating_accent":6,"global_rank":22,"character":102,"banner":"online_banner_2026_10",
            "rows":[{{"song_id":"heavenlycaress","title":"Heavenly caress","difficulty":3,
            "score":9995761,"constant":10.9,"rating":13.078805,"rating_frame":7,"rank":"FR"}}]}}"#
    ));
    serde_json::from_value(raw).expect("榜面载荷要能解析")
}

fn assert_identity(identity: &PlayerIdentity, label: &str) {
    assert!(
        identity.ptt().is_some(),
        "{label}: 后端给了 PTT, 身份里不能丢"
    );
    assert!(
        identity.frame().is_some(),
        "{label}: 后端给了 PTT, 段位框必须跟着到"
    );
    assert!(
        identity.rank().is_some(),
        "{label}: 后端给了世界排名, 身份里不能丢"
    );
    assert_eq!(
        identity.skin.character,
        Some(CHARACTER),
        "{label}: 游戏内搭档不能被丢掉"
    );
}

fn assert_badge(identity: &PlayerIdentity, label: &str) {
    let badge = bridge::badge_of(identity, &Prefs::default());
    assert_eq!(badge.ptt, Some(12.993), "{label}: 头像上的 PTT");
    assert_eq!(badge.frame, Some(6), "{label}: 段位框档位");
    assert_eq!(badge.global_rank, Some(22), "{label}: 排名框");
    assert!(
        badge
            .character
            .as_deref()
            .is_some_and(|key| key.starts_with("characters/")),
        "{label}: 立绘要能换出素材, got {:?}",
        badge.character
    );
    assert!(
        badge.avatar.as_deref().is_some_and(|key| key.ends_with("_icon.png")),
        "{label}: 头像图标要对上角色"
    );
}

#[test]
fn b50_payload_carries_identity() {
    let payload = b50_payload();
    assert_identity(&payload.identity, "b50");
    assert_badge(&payload.identity, "b50");
}

#[test]
fn every_board_kind_carries_identity() {
    for kind in ["pm50", "fr50", "x50", "nm50", "mx50", "ol50"] {
        let board = board_payload(kind);
        assert_identity(&board.identity, kind);
        assert_badge(&board.identity, kind);
        assert_eq!(board.rows.len(), 1, "{kind}: 行数据照常解析");
        assert!(
            (board.rows[0].rating - 13.078_805).abs() < 1e-9,
            "{kind}: 行里的 rating 是单曲 PTT, 不是玩家 PTT"
        );
        assert_eq!(
            board.rows[0].rating_frame, 7,
            "{kind}: 行里的 rating_frame 是单曲星框, 与页头身份各说各的"
        );
    }
}

#[test]
fn score_and_recent_carry_identity() {
    let raw = json(
        r#"{"found":true,"name":"wyndbz","current_ptt":12.993,"rating_frame":6,
            "rating_accent":6,"global_rank":22,"character":102,
            "banner":"online_banner_2026_10",
            "record":{"song_id":"heavenlycaress","title":"Heavenly caress","difficulty":3,
            "score":9995761,"constant":10.9,"rating":13.078805,"rating_frame":7}}"#,
    );
    let play: PlayPayload = serde_json::from_value(raw.clone()).expect("单曲载荷要能解析");
    assert_identity(&play.identity, "score");
    assert_badge(&play.identity, "score");
    let recent: RecentPayload = serde_json::from_value(raw).expect("最近游玩载荷要能解析");
    assert_identity(&recent.identity, "recent");
    assert_badge(&recent.identity, "recent");
}

#[test]
fn player_payload_keeps_character() {
    let raw = json(
        r#"{"arcaea_uid":7115537,"name":"wyndbz","friend_code":"235029093",
            "rating":12.993,"rating_frame":6,"rating_accent":6,"global_rank":22,
            "character":102,"banner":"online_banner_2026_10"}"#,
    );
    let player: Player = serde_json::from_value(raw).expect("档案要能解析");
    assert_identity(&player.identity, "player");
    assert_badge(&player.identity, "player");
}

#[test]
fn profile_board_row_carries_identity() {
    let raw = json(
        r#"{"rank":1,"arcaea_uid":10508363,"name":"Reincarnat","uid":"","rating":13.448,
            "rating_frame":7,"rating_accent":7,"character":38}"#,
    );
    let row: ProfileBoardRow = serde_json::from_value(raw).expect("档案榜行要能解析");
    assert_eq!(row.identity.name, "Reincarnat");
    assert_eq!(row.identity.ptt(), Some(13.448));
    assert_eq!(row.identity.frame(), Some(7));
    assert_eq!(row.identity.skin.character, Some(38));
}

#[test]
fn prefs_override_wins_over_backend_skin() {
    let mut identity = b50_payload().identity;
    let backend_only = bridge::badge_of(&identity, &Prefs::default());
    assert!(
        backend_only
            .banner
            .as_deref()
            .is_some_and(|key| key.contains("online_banner_2026_10")),
        "没有自选时跟随后端横幅, got {:?}",
        backend_only.banner
    );

    identity.skin.char_pref = String::new();
    let chosen = prefs("对立_tairitsu_1.png", "course_banner_01.png");
    let badge = bridge::badge_of(&identity, &chosen);
    assert_eq!(
        badge.character.as_deref(),
        Some("characters/对立_tairitsu_1.png"),
        "用户 /a char 设的立绘优先于游戏内"
    );
    assert_eq!(
        badge.banner.as_deref(),
        Some("banners/course_banner_01.png"),
        "用户 /a banner 设的横幅优先于游戏内"
    );
    assert_eq!(
        badge.ptt,
        Some(12.993),
        "自选素材只影响素材, 不该动 PTT"
    );
    assert_eq!(badge.frame, Some(6), "自选素材只影响素材, 不该动段位框");
}

#[test]
fn empty_prefs_keeps_backend_character() {
    let identity = b50_payload().identity;
    let badge = bridge::badge_of(&identity, &prefs("", ""));
    assert!(
        badge
            .character
            .as_deref()
            .is_some_and(|key| key.contains("_102")),
        "没设自选时必须回到游戏内搭档, got {:?}",
        badge.character
    );
}

#[test]
fn frame_follows_ptt_when_backend_omits_it() {
    let raw = json(r#"{"name":"wyndbz","current_ptt":13.6,"character":102}"#);
    let identity: PlayerIdentity = serde_json::from_value(raw).expect("身份要能解析");
    assert_eq!(identity.frame(), None, "后端没给档位");
    let badge = bridge::badge_of(&identity, &Prefs::default());
    let want = i32::from(aurorix_arcaea_plugin::domain::rating_frame_of(13.6));
    assert_eq!(
        badge.frame,
        Some(want),
        "有 PTT 就该算出档位, 不能因为缺档位画横杠"
    );
    assert_eq!(
        resolved_frame(Some(13.6), None),
        Some(want),
        "渲染层的兜底与页头同一条规则"
    );
}

#[test]
fn unknown_ptt_stays_a_dash() {
    let raw = json(r#"{"name":"wyndbz","character":102}"#);
    let identity: PlayerIdentity = serde_json::from_value(raw).expect("身份要能解析");
    let badge = bridge::badge_of(&identity, &Prefs::default());
    assert_eq!(badge.ptt, None);
    assert_eq!(badge.frame, None);
    assert_eq!(badge.global_rank, None);
    assert_eq!(resolved_frame(None, None), None);
    assert_eq!(resolved_frame(Some(0.0), None), None, "0 不是有效 PTT");
}

#[test]
fn ptt_override_rederives_frame() {
    let mut identity = b50_payload().identity;
    identity.current_ptt = Some(13.57);
    identity.rating_frame = None;
    identity.rating_accent = None;
    let badge = bridge::badge_of(&identity, &Prefs::default());
    assert_eq!(badge.ptt, Some(13.57), "/a diff 用行上的 PTT");
    assert_eq!(
        badge.frame,
        Some(i32::from(aurorix_arcaea_plugin::domain::rating_frame_of(
            13.57
        ))),
        "换了 PTT 就必须重算档位, 不能留着旧档位配新 PTT"
    );
    assert_eq!(
        badge.global_rank,
        Some(22),
        "换 PTT 不该把世界排名弄丢"
    );
}

#[test]
fn board_card_input_carries_identity() {
    let board = board_payload("x50");
    let input = bridge::identity_input(
        Region::Intl,
        false,
        &board.identity,
        &Prefs::default(),
        None,
    );
    assert_eq!(input.badge.ptt, Some(12.993), "榜面页头要有 PTT");
    assert_eq!(input.badge.frame, Some(6), "榜面页头要有段位框");
    assert_eq!(input.badge.global_rank, Some(22), "榜面页头要有排名");
    assert!(
        input.badge.character.is_some(),
        "榜面页头要有立绘, 不然只能画默认搭档"
    );
    assert_eq!(input.ptt, Some(12.993), "B50Input.ptt 也要拿到");
    assert_eq!(input.rank_frame, Some(6), "B50Input.rank_frame 也要拿到");
}
