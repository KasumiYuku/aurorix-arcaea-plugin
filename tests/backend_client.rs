use std::time::Duration;

use aurorix_arcaea_plugin::backend::{
    dedupe_key, encode_segment, normalize_base_url, ArcBackendNext, BackendConfig, BackendError,
    CnLoginStatus, ScoreSource,
};
use aurorix_arcaea_plugin::model::{Difficulty, Region};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
struct 收到请求 {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: String,
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
    delay: Duration,
}

impl 假回应 {
    fn json(body: &str) -> Self {
        Self {
            status: 200,
            content_type: "application/json".to_string(),
            body: body.to_string(),
            delay: Duration::ZERO,
        }
    }

    fn problem(status: u16, body: &str) -> Self {
        Self {
            status,
            content_type: "application/problem+json".to_string(),
            body: body.to_string(),
            delay: Duration::ZERO,
        }
    }

    fn sse(body: &str) -> Self {
        Self {
            status: 200,
            content_type: "text/event-stream".to_string(),
            body: body.to_string(),
            delay: Duration::ZERO,
        }
    }

    fn case_474(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }
}

fn case_450(status: u16) -> &'static str {
    match status {
        200 => "OK",
        401 => "Unauthorized",
        404 => "Not Found",
        409 => "Conflict",
        422 => "Unprocessable Entity",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
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
        body: String::from_utf8_lossy(&body).to_string(),
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
            if !response.delay.is_zero() {
                tokio::time::sleep(response.delay).await;
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

fn case_454(base: &str, timeout: Duration) -> ArcBackendNext {
    let config = BackendConfig::new(base)
        .token("dev-token")
        .timeout(timeout)
        .retries(0, Duration::from_millis(10));
    ArcBackendNext::new(config).expect("客户端要能构建")
}

fn case_475(base: &str) -> ArcBackendNext {
    let config = BackendConfig::new(base)
        .token("dev-token")
        .admin_token("admin-token")
        .timeout(Duration::from_secs(2))
        .retries(0, Duration::from_millis(10));
    ArcBackendNext::new(config).expect("客户端要能构建")
}

#[tokio::test]
async fn case_476() {
    let (base, mut 收到) = case_453(vec![假回应::json(
        r#"{"name":"某位玩家","friend_code":"123456789","rating":12.34,"rating_frame":3,"rating_accent":2}"#,
    )])
    .await;
    let client = case_454(&base, Duration::from_secs(2));
    let player = client
        .player(Region::Intl, 123456)
        .await
        .expect("玩家档案应当解析成功")
        .expect("应当返回一份档案");
    assert_eq!(player.identity.name, "某位玩家", "名字要与响应一致");
    assert_eq!(player.identity.current_ptt, Some(12.34), "PTT 要解析出来");
    assert_eq!(player.identity.rating_frame, Some(3), "段位框要解析出来");
    let request = 收到.recv().await.expect("假后端应当收到一个请求");
    assert_eq!(request.method, "GET", "方法必须是 GET");
    assert_eq!(request.path, "/v1/players/123456", "路径要带玩家号");
    assert_eq!(
        request.case_449("authorization"),
        Some("Bearer dev-token"),
        "必须带 Bearer 令牌"
    );
}

#[tokio::test]
async fn case_477() {
    let (base, mut 收到) = case_453(vec![假回应::json(
        r#"{"data":{"name":"国服玩家","records":[]},"source":"arcaea"}"#,
    )])
    .await;
    let client = case_454(&base, Duration::from_secs(2));
    let b50 = client
        .b50(Region::Cn, 777, false)
        .await
        .expect("B50 要能解析");
    assert_eq!(b50.identity.name, "国服玩家", "名字要对上");
    let request = 收到.recv().await.expect("假后端应当收到一个请求");
    assert_eq!(request.path, "/v1/cn/players/777/b50", "国服要加 /cn 前缀");
}

#[tokio::test]
async fn case_478() {
    let (base, _收到) = case_453(vec![假回应::problem(
        404,
        r#"{"code":"not_found","detail":"查无此人"}"#,
    )])
    .await;
    let client = case_454(&base, Duration::from_secs(2));
    let player = client
        .player(Region::Intl, 404404)
        .await
        .expect("404 不该变成错误");
    assert!(player.is_none(), "查无此人要返回 None");
}

#[tokio::test]
async fn case_479() {
    let long_detail = "错".repeat(400);
    let body = format!(r#"{{"code":"internal","detail":"{long_detail}"}}"#);
    let (base, _收到) = case_453(vec![假回应::problem(500, &body)]).await;
    let client = case_454(&base, Duration::from_secs(2));
    let error = client
        .b50(Region::Intl, 1, false)
        .await
        .expect_err("500 必须变成错误");
    match &error {
        BackendError::Http {
            status, code, body, ..
        } => {
            assert_eq!(*status, 500, "状态码要保留");
            assert_eq!(code.as_deref(), Some("internal"), "问题码要保留");
            assert!(body.len() <= 512, "错误正文必须截断到 512 字节以内");
        }
        other => panic!("应当是 HTTP 状态错误, 实际: {other:?}"),
    }
    assert!(
        error.to_string().contains("HTTP 500"),
        "错误正文要写清状态码, 实际: {error}"
    );
}

#[tokio::test]
async fn case_480() {
    let (base, _收到) = case_453(vec![假回应::json("{不是 JSON")]).await;
    let client = case_454(&base, Duration::from_secs(2));
    let error = client
        .player(Region::Intl, 9)
        .await
        .expect_err("畸形 JSON 必须报错");
    assert!(
        matches!(error, BackendError::Decode { .. }),
        "应当是解码错误, 实际: {error:?}"
    );
    assert!(
        error.to_string().contains("解析失败"),
        "错误要说明解析失败, 实际: {error}"
    );
}

#[tokio::test]
async fn case_481() {
    let (base, _收到) = case_453(vec![假回应::json("{}").case_474(Duration::from_millis(800))]).await;
    let client = case_454(&base, Duration::from_millis(120));
    let error = client
        .player(Region::Intl, 5)
        .await
        .expect_err("超时必须报错");
    assert!(
        matches!(error, BackendError::Timeout { .. }),
        "应当是超时错误, 实际: {error:?}"
    );
}

#[tokio::test]
async fn case_482() {
    let (base, mut 收到) = case_453(vec![假回应::json(
        r#"{"char":"char_12.png","banner":"banner_1.jpg"}"#,
    )])
    .await;
    let client = case_454(&base, Duration::from_secs(2));
    let prefs = client
        .set_prefs(Region::Intl, 7, Some("char_12.png"), None)
        .await
        .expect("写偏好要成功");
    assert_eq!(prefs.char, "char_12.png", "回执要解析");
    let request = 收到.recv().await.expect("假后端应当收到一个请求");
    assert_eq!(request.method, "PUT", "方法必须是 PUT");
    assert_eq!(request.path, "/v1/players/7/prefs", "路径要对");
    assert_eq!(
        request.body, r#"{"char":"char_12.png"}"#,
        "请求体必须只带要改的那一件"
    );
    assert_eq!(
        request.case_449("content-type"),
        Some("application/json"),
        "必须声明 JSON 请求体"
    );
}

#[tokio::test]
async fn case_483() {
    let (base, _收到) = case_453(vec![假回应::problem(
        429,
        r#"{"code":"rate_limited","detail":"查得太频繁了"}"#,
    )])
    .await;
    let client = case_454(&base, Duration::from_secs(2));
    let error = client
        .recent(Region::Intl, 3)
        .await
        .expect_err("限流必须报错");
    assert!(
        error.is_rate_limited(),
        "限流错误必须可识别, 实际: {error:?}"
    );
    assert!(
        error.to_string().contains("查得太频繁了"),
        "限流原因要透传, 实际: {error}"
    );
}

#[tokio::test]
async fn case_484() {
    let (base, _收到) = case_453(vec![假回应::problem(
        401,
        r#"{"code":"unauthenticated","detail":"令牌已失效"}"#,
    )])
    .await;
    let client = case_454(&base, Duration::from_secs(2));
    let error = client
        .player(Region::Intl, 3)
        .await
        .expect_err("401 必须报错");
    assert!(
        error.is_credential_invalid(),
        "凭证失效必须可识别, 实际: {error:?}"
    );
}

#[tokio::test]
async fn case_485() {
    let body = r#"{"data":{"name":"玩家","current_ptt":12.5,"best50":12.1,"best10":12.0,"max_ptt":13.0,"count":50,"records":[{"song_id":"test","difficulty":3,"score":9900000,"rating":12.0,"rank":"EX+","clear_type":0,"perfect":980,"far":12,"lost":3}]},"source":"arcaea","cached":true,"cached_at":"2026-01-02T03:04:05Z","eta_seconds":12}"#;
    let (base, _收到) = case_453(vec![假回应::json(body)]).await;
    let client = case_454(&base, Duration::from_secs(2));
    let b50 = client
        .b50(Region::Intl, 1, false)
        .await
        .expect("B50 要能解析");
    assert!(b50.meta.cached, "缓存标记要保留");
    assert!(b50.meta.cached_at.is_some(), "缓存时间要解析");
    assert_eq!(b50.meta.eta.seconds, Some(12), "ETA 要解析");
    assert_eq!(b50.records.len(), 1, "成绩条数要对上");
    let record = &b50.records[0];
    assert_eq!(
        record.difficulty_kind(),
        Some(Difficulty::Beyond),
        "难度编号要能换成难度"
    );
    assert_eq!(record.clear_type, Some(0), "Track Lost 要是有效值 0");
    assert!(record.judged(), "没有判定字段时按分数推断");
}

#[tokio::test]
async fn case_486() {
    let body = concat!(
        "event: cached\ndata: {\"cached_at\":\"2026-01-02T03:04:05Z\",\"total\":50}\n\n",
        "event: progress\ndata: {\"done\":25,\"total\":50,\"eta_remaining_seconds\":20}\n\n",
        "event: done\ndata: {\"data\":{\"name\":\"流式玩家\",\"records\":[{\"song_id\":\"x\",\"difficulty\":2,\"score\":9900000}]}}\n\n"
    );
    let (base, _收到) = case_453(vec![假回应::sse(body)]).await;
    let client = case_454(&base, Duration::from_secs(2));
    let b50 = client
        .b50_collected(Region::Intl, 1, false)
        .await
        .expect("SSE 流要能收敛成 B50");
    assert_eq!(b50.identity.name, "流式玩家", "流式完成事件要解析出名字");
    assert_eq!(b50.records.len(), 1, "流式完成事件要带上成绩");
}

#[tokio::test]
async fn case_487() {
    let body = r#"{"data":{"name":"普通玩家","records":[]},"source":"arcaea","cached":false}"#;
    let (base, _收到) = case_453(vec![假回应::json(body)]).await;
    let client = case_454(&base, Duration::from_secs(2));
    let b50 = client
        .b50_collected(Region::Intl, 1, false)
        .await
        .expect("普通响应也要能收敛成 B50");
    assert_eq!(b50.identity.name, "普通玩家", "普通响应要解析出名字");
    assert!(!b50.meta.cached, "这次不是缓存命中");
}

#[tokio::test]
async fn case_488() {
    let body = r#"{"data":[{"song_id":"a","difficulty":0,"score":10000000,"rank":"PM"},{"song_id":"b","difficulty":4,"score":9500000,"rank":"EX"}],"source":"local","cached":true}"#;
    let (base, mut 收到) = case_453(vec![假回应::json(body)]).await;
    let client = case_454(&base, Duration::from_secs(2));
    let scores = client
        .all_scores(Region::Intl, 88, ScoreSource::Probe)
        .await
        .expect("全量成绩要能解析");
    assert_eq!(scores.records.len(), 2, "两条成绩都要解析");
    assert_eq!(
        scores.records[1].difficulty_kind(),
        Some(Difficulty::Eternal),
        "难度要对上"
    );
    let request = 收到.recv().await.expect("假后端应当收到一个请求");
    assert_eq!(
        request.path, "/v1/players/88/scores?source=probe",
        "来源要进查询串"
    );
}

#[tokio::test]
async fn case_489() {
    let body = r#"{"dry_run":true,"changed":true,"before":{"songs":100,"charts":300,"aliases":10},"after":{"songs":101,"charts":302,"aliases":11},"added":{"total":1,"ids":["new-song"],"note":"新增"},"removed":{"total":0,"ids":[],"note":""}}"#;
    let (base, mut 收到) = case_453(vec![假回应::json(body)]).await;
    let client = case_475(&base);
    let sync = client.sync_catalog(true).await.expect("曲库同步要能解析");
    assert!(sync.dry_run, "dry 标记要保留");
    assert!(sync.changed, "变更标记要保留");
    assert_eq!(sync.song_delta(), 1, "歌曲增量要对");
    assert_eq!(sync.chart_delta(), 2, "谱面增量要对");
    assert_eq!(
        sync.added.ids,
        vec!["new-song".to_string()],
        "新增 id 要带出"
    );
    let request = 收到.recv().await.expect("假后端应当收到一个请求");
    assert_eq!(request.method, "POST", "同步是 POST");
    assert_eq!(
        request.path, "/v1/admin/arcdata/sync?dry=1",
        "dry 要进查询串"
    );
    assert_eq!(
        request.case_449("authorization"),
        Some("Bearer admin-token"),
        "曲库同步必须用管理令牌"
    );
}

#[tokio::test]
async fn case_490() {
    let start = r#"{"ticket_id":"t-1","url":"device-page","user_code":"ABCD","interval_seconds":0,"reused":false,"device_id":"dev-9"}"#;
    let poll = r#"{"status":"ready","player":{"arcaea_uid":9,"name":"国服玩家","friend_code":"998877","device_id":"dev-9"}}"#;
    let (base, mut 收到) = case_453(vec![假回应::json(start), 假回应::json(poll)]).await;
    let client = case_454(&base, Duration::from_secs(2));
    let login = client
        .cn_login_start("dev-9", Some(9))
        .await
        .expect("授权申请要成功");
    assert_eq!(login.ticket_id, "t-1", "票据号要解析");
    assert_eq!(
        login.interval(),
        Duration::from_secs(5),
        "缺省轮询间隔要回落到 5 秒"
    );
    let request = 收到.recv().await.expect("假后端应当收到一个请求");
    assert_eq!(request.path, "/v1/cn/login/start", "路径要对");
    assert_eq!(
        request.body, r#"{"arcaea_uid":9,"device_id":"dev-9"}"#,
        "已绑定账号要把 uid 一起发过去"
    );
    let status = client.cn_login_poll("t-1").await.expect("轮询要成功");
    assert_eq!(status.status, CnLoginStatus::Ready, "状态要解析成 ready");
    assert_eq!(
        status.player.as_ref().map(|player| player.name.as_str()),
        Some("国服玩家"),
        "就绪时要带出档案"
    );
}

#[tokio::test]
async fn case_491() {
    let (base, _收到) = case_453(Vec::new()).await;
    let client = case_454(&base, Duration::from_secs(2));
    let error = client
        .player(Region::Intl, 0)
        .await
        .expect_err("uid 为 0 必须拒绝");
    assert!(
        matches!(error, BackendError::InvalidInput { .. }),
        "应当是参数错误, 实际: {error:?}"
    );
}

#[tokio::test]
async fn case_492() {
    let first = dedupe_key(Region::Intl, 1001, "  B50   -f ");
    let second = dedupe_key(Region::Intl, 1001, "b50 -f");
    assert_eq!(first, second, "大小写与空白要归一化");
    assert_eq!(first, "arc:intl:1001:b50 -f", "键的形状要固定");
    assert_ne!(
        first,
        dedupe_key(Region::Cn, 1001, "b50 -f"),
        "不同区域的同一条指令不能共用一个键"
    );
}

#[test]
fn case_383() {
    let 分隔 = ["/", "/"].concat();
    let 合法 = format!("https:{分隔}arc.example.com");
    let base = normalize_base_url(&合法).expect("合法地址要能归一化");
    assert!(base.ends_with('/'), "base 必须以斜杠结尾");
    assert_eq!(base, format!("{合法}/"), "只补一个斜杠");
    let 带查询串 = format!("http:{分隔}host?a=1");
    assert!(
        normalize_base_url(&带查询串).is_err(),
        "带查询串的地址必须拒绝"
    );
    assert!(
        normalize_base_url("ftp:/x").is_err(),
        "非 http 协议必须拒绝"
    );
    assert!(normalize_base_url("   ").is_err(), "空地址必须拒绝");
}

#[test]
fn case_493() {
    let segment = encode_segment("a/b c").expect("非空片段可以编码");
    assert_eq!(segment, "a%2Fb%20c", "斜杠必须转义, 防止路径注入");
    assert!(encode_segment("   ").is_err(), "空白片段必须拒绝");
    assert_eq!(
        encode_segment("song-1.id_2").expect("安全字符也要能编码"),
        "song-1.id_2",
        "安全字符保持原样"
    );
}
