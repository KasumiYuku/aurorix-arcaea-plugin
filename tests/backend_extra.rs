use std::time::Duration;

use aurorix_arcaea_plugin::backend::{
    parse_time, ArcBackendNext, BackendConfig, BackendError, ChapterQuery, ProfileQuery,
    StatsQuery, SuggestQuery,
};
use aurorix_arcaea_plugin::model::{Difficulty, Region};
use chrono::SecondsFormat;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
struct 收到请求 {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
}

impl 收到请求 {
    fn case_449(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }
}

#[derive(Debug, Clone)]
struct 假回应 {
    status: u16,
    content_type: String,
    body: String,
}

impl 假回应 {
    fn json(body: &str) -> Self {
        Self {
            status: 200,
            content_type: "application/json".to_string(),
            body: body.to_string(),
        }
    }

    fn problem(status: u16, body: &str) -> Self {
        Self {
            status,
            content_type: "application/problem+json".to_string(),
            body: body.to_string(),
        }
    }
}

fn case_450(status: u16) -> &'static str {
    match status {
        200 => "OK",
        404 => "Not Found",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "Unknown",
    }
}

fn case_451(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|窗口| 窗口 == needle)
}

async fn case_452(socket: &mut tokio::net::TcpStream) -> Option<收到请求> {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 1024];
    let header_end = loop {
        let read = socket.read(&mut chunk).await.ok()?;
        if read == 0 {
            return None;
        }
        buffer.extend_from_slice(&chunk[..read]);
        if let Some(position) = case_451(&buffer, b"\r\n\r\n") {
            break position + 4;
        }
        if buffer.len() > 65536 {
            return None;
        }
    };
    let head = String::from_utf8_lossy(&buffer[..header_end]).to_string();
    let mut lines = head.split("\r\n");
    let request_line = lines.next()?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next()?.to_string();
    let path = parts.next()?.to_string();
    let mut headers = Vec::new();
    let mut content_length = 0usize;
    for line in lines {
        if line.is_empty() {
            continue;
        }
        if let Some((name, value)) = line.split_once(':') {
            let value = value.trim().to_string();
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value.parse().unwrap_or(0);
            }
            headers.push((name.to_ascii_lowercase(), value));
        }
    }
    let mut body = buffer[header_end..].to_vec();
    while body.len() < content_length {
        let read = socket.read(&mut chunk).await.ok()?;
        if read == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..read]);
    }
    Some(收到请求 {
        method,
        path,
        headers,
    })
}

async fn case_453(
    回应: Vec<假回应>
) -> (String, mpsc::UnboundedReceiver<收到请求>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .await
        .expect("假后端要能监听本地端口");
    let address = listener.local_addr().expect("假后端要能拿到监听地址");
    let (sender, receiver) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        for response in 回应 {
            let mut socket = match listener.accept().await {
                Ok((socket, _)) => socket,
                Err(_) => break,
            };
            if let Some(request) = case_452(&mut socket).await {
                let _ = sender.send(request);
            }
            let head = format!(
                "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                response.status,
                case_450(response.status),
                response.content_type,
                response.body.len()
            );
            let _ = socket.write_all(head.as_bytes()).await;
            let _ = socket.write_all(response.body.as_bytes()).await;
            let _ = socket.flush().await;
            let _ = socket.shutdown().await;
        }
    });
    (format!("{}{}{}{}/", "http:", "/", "/", address), receiver)
}

fn case_454(base: &str) -> ArcBackendNext {
    let config = BackendConfig::new(base)
        .token("dev-token")
        .timeout(Duration::from_secs(2))
        .retries(0, Duration::from_millis(10));
    ArcBackendNext::new(config).expect("客户端要能构建")
}

fn case_455(value: &chrono::DateTime<chrono::Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Secs, true)
}

#[test]
fn case_456() {
    let parsed = parse_time("2026-11-20 12:34:56").expect("无时区串必须能解析");
    assert_eq!(
        case_455(&parsed),
        "2026-11-20T04:34:56Z",
        "国服时间是东八区，12:34:56 应当是 UTC 04:34:56"
    );
    assert_ne!(
        case_455(&parsed),
        "2026-11-20T12:34:56Z",
        "无偏移串不能当 UTC"
    );
    let naive_utc = parse_time("2026-11-20T12:34:56").expect("ISO 无偏移也要能解析");
    assert_eq!(
        case_455(&naive_utc),
        "2026-11-20T04:34:56Z",
        "T 分隔的无偏移串同样按东八区"
    );
}

#[test]
fn case_457() {
    let zulu = parse_time("2026-11-20T12:34:56Z").expect("Z 结尾必须能解析");
    assert_eq!(case_455(&zulu), "2026-11-20T12:34:56Z", "Z 就是 UTC");
    let cn = parse_time("2026-11-20T12:34:56+08:00").expect("+08:00 必须能解析");
    assert_eq!(case_455(&cn), "2026-11-20T04:34:56Z", "+08:00 要换算回 UTC");
    let utc = parse_time("2026-11-20T12:34:56+00:00").expect("显式零偏移必须能解析");
    assert_eq!(case_455(&utc), "2026-11-20T12:34:56Z");
    assert_ne!(case_455(&zulu), case_455(&cn), "两种写法不是同一时刻");
}

#[test]
fn case_458() {
    let parsed = parse_time("2026-11-20").expect("纯日期必须能解析");
    assert_eq!(
        case_455(&parsed),
        "2026-11-19T16:00:00Z",
        "东八区 00:00 是 UTC 前一天 16:00"
    );
    let slashed = parse_time("2026/11/20").expect("斜杠日期也要能解析");
    assert_eq!(case_455(&slashed), case_455(&parsed));
}

#[test]
fn case_459() {
    let parsed = parse_time("2026-11-20 12:34:56.789").expect("小数秒必须能解析");
    let text = parsed.to_rfc3339_opts(SecondsFormat::Millis, true);
    assert_eq!(text, "2026-11-20T04:34:56.789Z");
}

#[test]
fn case_460() {
    for text in [
        "",
        "   ",
        "昨天",
        "2026-13-40",
        "2026-11-20 25:61:61",
        "not a time",
    ] {
        assert!(parse_time(text).is_none(), "{text:?} 应当解析失败");
    }
}

#[tokio::test]
async fn case_461() {
    let body = r#"{"bound":true,"player":{"arcaea_uid":7,"name":"档案玩家","friend_code":"123","rating":12.5,"rating_frame":3,"join_date":"2020-01-02"},"rank":{"rank":88,"total":9000,"rating":12.5},"board":{"total":1,"rows":[{"rank":1,"name":"档案玩家","rating":12.5,"uid":"7","me":true,"character":12,"banner":"b.png"}]},"curve":[{"day":"2026-11-20","rating":12.3,"records":2}],"changes":[{"song_id":"s1","difficulty":2,"has_old":true,"old_score":9800000,"new_score":9900000,"at":"2026-11-20 12:34:56"}]}"#;
    let (base, mut 收到) = case_453(vec![假回应::json(body)]).await;
    let client = case_454(&base);
    let profile = client
        .profile(Region::Intl, 7, ProfileQuery::new())
        .await
        .expect("档案要能解析");
    assert!(profile.bound, "绑定标记要保留");
    assert_eq!(profile.name(), "档案玩家");
    assert_eq!(profile.rank.as_ref().map(|rank| rank.rank), Some(88));
    assert!(profile.rank.as_ref().is_some_and(|rank| rank.listed()));
    assert_eq!(profile.curve.len(), 1, "曲线点要解析");
    let points = profile.points();
    assert!(points[0].0.is_some(), "曲线日期也要能解析");
    assert_eq!(points[0].1, Some(12.3));
    let change = &profile.changes[0];
    assert_eq!(change.difficulty_kind(), Some(Difficulty::Future));
    assert_eq!(change.new_score, 9_900_000);
    assert_eq!(
        change.at.map(|at| case_455(&at)),
        Some("2026-11-20T04:34:56Z".to_string()),
        "变动时间按东八区换算"
    );
    let board = profile.board.as_ref().expect("榜段要解析");
    assert!(board.rows[0].is_me, "me 字段要映射到 is_me");
    assert!(
        board.rows[0].identity.skin.known(),
        "角色要能换出皮肤"
    );
    let request = 收到.recv().await.expect("假后端应当收到一个请求");
    assert_eq!(request.method, "GET");
    assert_eq!(
        request.path, "/v1/players/7/profile?days=365&changes=8&board=12&around=2",
        "四个参数都必须下发"
    );
    assert_eq!(request.case_449("authorization"), Some("Bearer dev-token"));
}

#[tokio::test]
async fn case_462() {
    let body = r#"{"kind":"pm50","matched":1,"known":2,"total":3,"has_curve":true,"anchor":12.1,"anchor_upper":12.5,"anchor_source":"b50","band_counts":{"90":1},"updated_at":"2026-11-20T12:34:56Z","avg_best10":12.2,"avg_best50":11.9,"best_count":1,"over_count":0,"mean_gap":0.3,"mean_pressure":1.2,"rows":[{"song_id":"s1","title":"曲子","difficulty":3,"score":10000000,"constant":10.5,"rating":12.7,"board_rank":1,"has_judgement":false,"clear_type":3,"over_level":0.5,"pressure":1.1}]}"#;
    let (base, mut 收到) = case_453(vec![假回应::json(body)]).await;
    let client = case_454(&base);
    let board = client
        .boards(Region::Cn, 9, "pm50", Some(50))
        .await
        .expect("榜面要能解析");
    assert_eq!(board.kind, "pm50");
    assert_eq!(board.rows.len(), 1);
    assert_eq!(board.rows[0].difficulty_kind(), Some(Difficulty::Beyond));
    assert_eq!(board.rows[0].board_rank, 1);
    assert_eq!(board.band_counts.get("90"), Some(&1));
    let request = 收到.recv().await.expect("假后端应当收到一个请求");
    assert_eq!(
        request.path, "/v1/cn/players/9/boards/pm50?limit=50",
        "国服要带前缀，kind 与 limit 都要进路径"
    );
}

#[tokio::test]
async fn case_463() {
    let (base, mut 收到) = case_453(Vec::new()).await;
    let client = case_454(&base);
    let board = client
        .boards(Region::Intl, 9, "  ", None)
        .await
        .expect("空类别不是错误");
    assert_eq!(board.total, 0);
    assert!(board.rows.is_empty());
    assert!(收到.try_recv().is_err(), "空类别不许真的发请求出去");
}

#[tokio::test]
async fn case_464() {
    let body = r#"{"total":100,"played":60,"cleared":50,"full_combo":10,"pm":5,"avg_score":9700000.5,"avg_rating":11.2,"bands":[{"difficulty":2,"total":40,"played":30,"cleared":25,"full_combo":3,"pm":1,"avg_score":9600000.0}],"consts":[{"constant":10.5,"total":4,"played":2}],"grades":[{"grade":"EX+","count":3},{"grade":"AA","count":2}],"short":[{"song_id":"s2","title":"短表","difficulty":2,"constant":9.5,"has_constant":true,"score":9500000,"grade":"AA","next_rank":"EX","next_need":300000}],"known":90,"updated_at":"2026-11-20 08:00:00"}"#;
    let (base, mut 收到) = case_453(vec![假回应::json(body)]).await;
    let client = case_454(&base);
    let query = StatsQuery::new("2026-01-01", "all")
        .with_to("+Inf")
        .with_diff(vec!["0".to_string(), "2".to_string()])
        .with_short(10);
    let stats = client
        .stats(Region::Intl, 21, query)
        .await
        .expect("统计要能解析");
    assert_eq!(stats.total, 100);
    assert_eq!(stats.avg_score, 9_700_000.5);
    assert_eq!(stats.bands[0].difficulty_kind(), Some(Difficulty::Future));
    assert_eq!(stats.grades[1].grade_kind(), Some(arcaea_grade_aa()));
    assert_eq!(stats.short.as_deref().unwrap_or(&[])[0].next_rank, "EX");
    let request = 收到.recv().await.expect("假后端应当收到一个请求");
    assert_eq!(
        request.path, "/v1/players/21/stats?from=2026-01-01&diff=0%2C2&view=all&short=10",
        "to=+Inf 不许下发，diff 逗号要转义"
    );
}

fn arcaea_grade_aa() -> aurorix_arcaea_plugin::model::Grade {
    aurorix_arcaea_plugin::model::Grade::Aa
}

#[tokio::test]
async fn case_465() {
    let body = r#"{"displayed":12.34,"target":12.5,"enough":true,"pool":3,"maxed":3,"thin":false,"pushable":1,"has_ability":true,"anchor":12.1,"pick_count":1,"known":50,"updated_at":"2026-11-20T00:00:00Z","hints":[{"song_id":"s3","title":"推分","artist":"a","difficulty":2,"constant":10.5,"note":1234,"score":9800000,"has_score":true,"far":10,"lost":1,"has_judgement":true,"delta":0.3,"target":12.5,"rating":12.1,"target_rank":9,"rank":7,"counted":true,"gap":0.4,"reach":true,"pick":true,"expected_score":12.2,"headroom":0.1,"notes":{"known":true,"far_need":5,"lost_need":2,"far_room":3,"lost_room":1,"far_use":5,"lost_use":2,"short":false}}]}"#;
    let (base, mut 收到) = case_453(vec![假回应::json(body)]).await;
    let client = case_454(&base);
    let query = SuggestQuery::new()
        .with_limit(8)
        .with_filter("diff:2")
        .with_text("a b");
    let suggest = client
        .suggest(Region::Intl, 33, query)
        .await
        .expect("建议要能解析");
    assert_eq!(suggest.displayed, 12.34);
    assert!(suggest.enough);
    assert_eq!(suggest.hints.len(), 1);
    assert!(suggest.hints[0].notes.known);
    assert_eq!(suggest.hints[0].notes.far_need, 5);
    assert_eq!(suggest.hints[0].difficulty_kind(), Some(Difficulty::Future));
    let request = 收到.recv().await.expect("假后端应当收到一个请求");
    assert_eq!(
        request.path, "/v1/players/33/suggest?limit=8&filter=diff%3A2&q=a%20b",
        "filter 与 q 必须转义"
    );
}

#[tokio::test]
async fn case_466() {
    let body = r#"{"query":"主线","empty":false,"updated_at":"2026-11-20 09:00:00","totals":{"packs":2,"total":10,"played":6,"cleared":5,"full_combo":1,"pm":0},"sections":[{"id":"main","name":"主线","packs":[{"pack_id":"p1","name":"包一","section":"main","total":5,"played":4,"cleared":3,"full_combo":1,"pm":0,"avg_score":9700000.0,"avg_ptt":11.0}]}],"unfiled":{"pack_id":"none","name":"未归类"},"detail":{"pack":{"pack_id":"p1","name":"包一"},"charts":[{"song_id":"s4","title":"曲四","rating_class":2,"constant":9.6,"played":true,"score":9600000,"grade":"AA","ptt":10.5,"perfect":900,"shiny_perfect":800,"near":40,"miss":2}]},"matched":[{"pack_id":"p1","name":"包一"}]}"#;
    let (base, mut 收到) = case_453(vec![假回应::json(body)]).await;
    let client = case_454(&base);
    let chapters = client
        .chapters(
            Region::Cn,
            44,
            ChapterQuery::new()
                .with_text("主线")
                .with_rating_class(vec!["2".to_string()]),
        )
        .await
        .expect("曲包要能解析");
    assert_eq!(chapters.query, "主线");
    assert_eq!(chapters.totals.packs, 2);
    assert_eq!(chapters.sections[0].packs[0].pack_id, "p1");
    assert!(chapters.unfiled.is_some(), "未归类包要解析");
    let detail = chapters.detail.as_ref().expect("明细要解析");
    assert_eq!(detail.charts[0].difficulty_kind(), Some(Difficulty::Future));
    assert_eq!(detail.charts[0].constant, Some(9.6));
    assert_eq!(chapters.matched[0].pack_id, "p1");
    let request = 收到.recv().await.expect("假后端应当收到一个请求");
    assert_eq!(
        request.path, "/v1/cn/players/44/chapters?q=%E4%B8%BB%E7%BA%BF&rating_class=2",
        "中文查询要转义"
    );
}

#[tokio::test]
async fn case_467() {
    let body = r#"[{"at":"2026-11-20 12:00:00","rating":12.1},{"at":"2026-11-21 12:00:00","rating":null},{"at":"不是时间","rating":12.2},{"at":"2026-11-22","rating":0}]"#;
    let (base, mut 收到) = case_453(vec![假回应::json(body)]).await;
    let client = case_454(&base);
    let events = client
        .rating_events(Region::Intl, 55, Some(30), Some(100))
        .await
        .expect("流水要能解析");
    assert_eq!(events.len(), 1, "空值与坏时间行都要丢掉");
    assert_eq!(events[0].rating, 12.1);
    assert_eq!(case_455(&events[0].at), "2026-11-20T04:00:00Z");
    let request = 收到.recv().await.expect("假后端应当收到一个请求");
    assert_eq!(
        request.path,
        "/v1/players/55/rating-events?days=30&limit=100"
    );
}

#[tokio::test]
async fn case_468() {
    let rank = r#"{"rank":12,"total":9000,"rating":12.4}"#;
    let eta = r#"{"seconds":90}"#;
    let (base, mut 收到) = case_453(vec![假回应::json(rank), 假回应::json(eta)]).await;
    let client = case_454(&base);
    let rank = client.rank(Region::Intl, 66).await.expect("名次要能解析");
    assert_eq!(rank.rank, 12);
    assert_eq!(rank.total, 9000);
    assert_eq!(rank.rating, Some(12.4));
    let first = 收到.recv().await.expect("假后端应当收到名次请求");
    assert_eq!(first.path, "/v1/players/66/rank");
    let eta = client.eta(Region::Cn, 66).await.expect("估时要能解析");
    assert_eq!(eta.seconds, 90);
    let second = 收到.recv().await.expect("假后端应当收到估时请求");
    assert_eq!(second.path, "/v1/cn/players/66/eta");
}

#[tokio::test]
async fn case_469() {
    let not_available = r#"{"code":"unavailable","detail":"上游连不上"}"#;
    let play = r#"{"data":{"found":true,"record":{"song_id":"song/x","title":"单曲","difficulty":2,"score":9800000,"rank":"EX"},"name":"玩家","current_ptt":12.0,"rating_frame":1}}"#;
    let (base, mut 收到) = case_453(vec![
        假回应::problem(503, not_available),
        假回应::json(play),
    ])
    .await;
    let client = case_454(&base);
    let play = client
        .score(Region::Intl, 77, "song/x", Difficulty::Future, false)
        .await
        .expect("兜底重试要成功");
    assert!(play.found);
    assert_eq!(play.identity.name, "玩家");
    assert_eq!(
        play.record.as_ref().map(|record| record.score),
        Some(9_800_000)
    );
    let first = 收到.recv().await.expect("假后端应当收到第一次请求");
    assert_eq!(
        first.path, "/v1/players/77/scores/song%2Fx/2",
        "歌曲 id 必须转义，实时查询不带 local"
    );
    let second = 收到.recv().await.expect("假后端应当收到兜底请求");
    assert_eq!(
        second.path, "/v1/players/77/scores/song%2Fx/2?local=true",
        "兜底重试必须带 local=true"
    );
}

#[tokio::test]
async fn case_470() {
    let body = r#"[{"difficulty":3,"has_old":true,"old_score":9500000,"old_perfect":900,"old_near":40,"old_miss":2,"old_clear_type":1,"old_rating":10.2,"new_score":9800000,"new_perfect":950,"new_shiny_perfect":900,"new_near":20,"new_miss":1,"new_clear_type":3,"new_rating":11.1,"account_rating":12.0,"at":"2026-11-20 18:00:00"}]"#;
    let (base, mut 收到) = case_453(vec![假回应::json(body)]).await;
    let client = case_454(&base);
    let rows = client
        .history(Region::Intl, 88, "s1", Difficulty::Beyond, Some(5))
        .await
        .expect("变迁史要能解析");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].difficulty_kind(), Some(Difficulty::Beyond));
    assert!(rows[0].improved(), "分数变高要算进步");
    assert_eq!(rows[0].new_clear_type, Some(3));
    assert_eq!(
        rows[0].at.map(|at| case_455(&at)),
        Some("2026-11-20T10:00:00Z".to_string())
    );
    let request = 收到.recv().await.expect("假后端应当收到一个请求");
    assert_eq!(
        request.path, "/v1/players/88/scores/s1/3/history?limit=5",
        "难度用上游编号，limit 进查询串"
    );
}

#[tokio::test]
async fn case_471() {
    let (base, _收到) = case_453(vec![假回应::json("[]")]).await;
    let client = case_454(&base);
    let rows = client
        .history(Region::Intl, 99, "s1", Difficulty::Past, None)
        .await
        .expect("空数组不是错误");
    assert!(rows.is_empty());
}

#[tokio::test]
async fn case_472() {
    let (base, _收到) = case_453(vec![假回应::problem(
        404,
        r#"{"code":"not_found","detail":"查无此人"}"#,
    )])
    .await;
    let client = case_454(&base);
    let error = client
        .rank(Region::Intl, 123)
        .await
        .expect_err("404 必须报错");
    assert_eq!(error.status(), Some(404));
    assert!(error.is_not_found());

    let (base, _收到) = case_453(vec![假回应::problem(
        500,
        r#"{"code":"internal","detail":"炸了"}"#,
    )])
    .await;
    let client = case_454(&base);
    let error = client
        .stats(Region::Intl, 123, StatsQuery::new("2026-01-01", "all"))
        .await
        .expect_err("500 必须报错");
    match error {
        BackendError::Http { status, code, .. } => {
            assert_eq!(status, 500);
            assert_eq!(code.as_deref(), Some("internal"));
        }
        other => panic!("应当是 HTTP 错误，实际 {other:?}"),
    }
}

#[tokio::test]
async fn case_473() {
    let (base, _收到) = case_453(vec![假回应::json("{不是 JSON")]).await;
    let client = case_454(&base);
    let error = client
        .profile(Region::Intl, 1, ProfileQuery::trend(30))
        .await
        .expect_err("畸形 JSON 必须报错");
    assert!(
        matches!(error, BackendError::Decode { .. }),
        "应当是解码错误，实际 {error:?}"
    );
    assert!(error.to_string().contains("解析失败"));
}
