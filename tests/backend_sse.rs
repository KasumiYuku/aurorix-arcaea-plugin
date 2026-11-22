use std::time::Duration;

use aurorix_arcaea_plugin::backend::{
    events_from_envelope_body, parse_sse, B50Event, B50Mapper, ProgressSource, SseEvent, SseItem,
    SseParser,
};

fn case_405(items: &[SseItem]) -> Vec<&SseEvent> {
    items
        .iter()
        .filter_map(|item| match item {
            SseItem::Event(event) => Some(event),
            _ => None,
        })
        .collect()
}

fn case_406(input: &str, 切点: &[usize]) -> Vec<SseItem> {
    let bytes = input.as_bytes();
    let mut parser = SseParser::new();
    let mut items = Vec::new();
    let mut start = 0;
    for cut in 切点 {
        items.extend(parser.feed(&bytes[start..*cut]));
        start = *cut;
    }
    items.extend(parser.feed(&bytes[start..]));
    items.extend(parser.finish());
    items
}

#[test]
fn case_407() {
    let mut parser = SseParser::new();
    let mut items = parser.feed(b"event: prog");
    items.extend(parser.feed(b"ress\ndata: {\"done\":1"));
    items.extend(parser.feed(b",\"total\":50}\n\n"));
    items.extend(parser.finish());
    let events = case_405(&items);
    assert_eq!(events.len(), 1, "跨三个包的事件只能派发一次");
    assert_eq!(events[0].name, "progress", "事件名要跨包拼回");
    assert_eq!(
        events[0].data, "{\"done\":1,\"total\":50}",
        "数据要跨包拼回"
    );
}

#[test]
fn case_408() {
    let input = "event: progress\r\ndata: {\"done\":1}\n\n: keepalive\r\n\r\nevent: done\ndata: {\"name\":\"玩家\"}\n\n";
    let whole = parse_sse(input.as_bytes());
    for cut in 1..input.len() {
        let split = case_406(input, &[cut]);
        assert_eq!(split, whole, "在字节 {cut} 处切开后解析结果必须与整块一致");
    }
}

#[test]
fn case_409() {
    let items = parse_sse(b"event: progress\ndata: first\ndata: second\ndata: third\n\n");
    let events = case_405(&items);
    assert_eq!(events.len(), 1, "多行 data 属于同一个事件");
    assert_eq!(
        events[0].data, "first\nsecond\nthird",
        "多行 data 要按行拼成一条"
    );
}

#[test]
fn case_410() {
    let items = parse_sse(b": keepalive\n\n: ping\n\nevent: progress\ndata: 1\n\n");
    let events = case_405(&items);
    assert_eq!(events.len(), 1, "注释行之后只应有一个事件");
    assert_eq!(events[0].data, "1", "事件数据要完整");
    let comments = items
        .iter()
        .filter(|item| matches!(item, SseItem::Comment(_)))
        .count();
    assert_eq!(comments, 2, "两条注释要各自记下来");
}

#[test]
fn case_411() {
    let items = parse_sse(b"event: progress\r\ndata: 1\r\n\r\nevent: done\ndata: 2\n\n");
    let events = case_405(&items);
    assert_eq!(events.len(), 2, "两种行尾都要认");
    assert_eq!(events[0].name, "progress", "事件名不能带回车");
    assert_eq!(events[0].data, "1", "数据不能带回车");
    assert_eq!(events[1].name, "done", "第二个事件名要对");
}

#[test]
fn case_412() {
    let mut parser = SseParser::new();
    let mut items = parser.feed(b"event: done\ndata: {\"name\":\"a\"}");
    assert!(case_405(&items).is_empty(), "没有空行时先不派发");
    items.extend(parser.finish());
    let events = case_405(&items);
    assert_eq!(events.len(), 1, "收尾时必须补派发最后一个事件");
    assert_eq!(events[0].data, "{\"name\":\"a\"}", "数据要完整");
    assert!(parser.finish().is_empty(), "重复收尾不能再吐事件");
}

#[test]
fn case_373() {
    let items = parse_sse(b"retry: 2500\n\n");
    assert_eq!(items, vec![SseItem::Retry(2500)], "retry 要换算成毫秒");
}

#[test]
fn case_413() {
    let items = parse_sse(b"\n\nevent: progress\n\n: ping\n\n");
    assert!(case_405(&items).is_empty(), "没有 data 的事件不能派发");
}

#[test]
fn case_414() {
    let items = parse_sse(b"id: 7\nevent: progress\ndata: 1\n\n");
    let events = case_405(&items);
    assert_eq!(events[0].id.as_deref(), Some("7"), "id 要跟随事件带出");
}

#[test]
fn case_415() {
    let mut parser = SseParser::new();
    let mut mapper = B50Mapper::new();
    let mut items = parser
        .feed(b"event: progress\ndata: {\"done\":3,\"total\":12,\"eta_remaining_seconds\":45}\n\n");
    items.extend(parser.finish());
    let events: Vec<B50Event> = items
        .into_iter()
        .filter_map(|item| mapper.map(item))
        .map(|item| item.expect("进度事件不该是错误"))
        .collect();
    assert_eq!(events.len(), 1, "只应产生一个进度事件");
    match &events[0] {
        B50Event::Progress {
            done,
            total,
            percent,
            eta,
            ..
        } => {
            assert_eq!(*done, 3, "已完成数要对上");
            assert_eq!(*total, 12, "总量要对上");
            assert_eq!(*percent, Some(25.0), "百分比要算出来");
            assert_eq!(eta.remaining, Some(45), "剩余时间要对上");
        }
        other => panic!("应当是进度事件, 实际: {other:?}"),
    }
    assert_eq!(mapper.total(), 12, "映射器要记住总量");
}

#[test]
fn case_416() {
    let mut mapper = B50Mapper::new();
    let cached = mapper
        .map(SseItem::Event(SseEvent {
            name: "cached".to_string(),
            data: "{\"cached_at\":\"2026-01-02T03:04:05Z\",\"total\":50}".to_string(),
            id: None,
        }))
        .expect("缓存事件要映射")
        .expect("缓存事件不该是错误");
    match cached {
        B50Event::Started {
            source,
            cached,
            cached_at,
            total,
        } => {
            assert_eq!(source, ProgressSource::Cache, "缓存命中要标成 cache");
            assert!(cached, "缓存命中要带 cached 标记");
            assert!(cached_at.is_some(), "缓存时间要解析出来");
            assert_eq!(total, Some(50), "总量要带出来");
        }
        other => panic!("应当是开始事件, 实际: {other:?}"),
    }
    let live = mapper
        .map(SseItem::Event(SseEvent {
            name: "live".to_string(),
            data: String::new(),
            id: None,
        }))
        .expect("实时事件要映射")
        .expect("实时事件不该是错误");
    assert_eq!(live.source(), ProgressSource::Live, "上游实时要标成 live");
    let waiting = mapper
        .map(SseItem::Event(SseEvent {
            name: "retry".to_string(),
            data: "{\"retry_after_seconds\":8}".to_string(),
            id: None,
        }))
        .expect("重试事件要映射")
        .expect("重试事件不该是错误");
    match waiting {
        B50Event::Retry { delay, source, .. } => {
            assert_eq!(delay, Duration::from_secs(8), "重试间隔要按秒还原");
            assert_eq!(source, ProgressSource::RetryWait, "重试等待要单独区分");
        }
        other => panic!("应当是重试事件, 实际: {other:?}"),
    }
}

#[test]
fn case_417() {
    let mut mapper = B50Mapper::new();
    assert!(
        mapper
            .map(SseItem::Event(SseEvent {
                name: "whatever".to_string(),
                data: "not json".to_string(),
                id: None,
            }))
            .is_none(),
        "看不懂的事件不能打断流"
    );
    assert!(
        mapper
            .map(SseItem::Comment("keepalive".to_string()))
            .is_none(),
        "注释不参与 B50 映射"
    );
}

#[test]
fn case_418() {
    let body = r#"{"data":{"name":"玩家","current_ptt":12.5,"records":[{"song_id":"x","difficulty":2,"score":9900000,"rank":"EX+"}]},"source":"arcaea","cached":true,"cached_at":"2026-01-02T03:04:05Z","progress":{"done":50,"total":50},"eta_seconds":12}"#;
    let events: Vec<B50Event> = events_from_envelope_body(body.as_bytes())
        .into_iter()
        .map(|item| item.expect("回退解析不该报错"))
        .collect();
    assert!(
        matches!(events[0], B50Event::Started { cached: true, .. }),
        "首个事件要说清来源"
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, B50Event::Progress { total: 50, .. })),
        "要给出进度"
    );
    let completed = events
        .iter()
        .find_map(|event| match event {
            B50Event::Completed { b50, .. } => Some(b50),
            _ => None,
        })
        .expect("必须收到完成事件");
    assert_eq!(completed.records.len(), 1, "成绩条数要对上");
    assert_eq!(
        completed.records[0].grade().map(|grade| grade.as_str()),
        Some("EX+"),
        "评级要能解析"
    );
    assert!(completed.meta.cached, "元信息要保留缓存标记");
}

#[test]
fn case_419() {
    let events = events_from_envelope_body(
        r#"{"status":"failed","error":"上游没连上","task_id":"t1"}"#.as_bytes(),
    );
    let failure = events
        .into_iter()
        .find_map(Result::err)
        .expect("失败信封必须变成错误");
    assert!(
        failure.to_string().contains("上游没连上"),
        "错误正文要透传, 实际: {failure}"
    );
}

#[test]
fn case_420() {
    let events: Vec<B50Event> = events_from_envelope_body(
        r#"{"task_id":"t1","status":"running","progress":{"done":7,"total":50},"eta_seconds":30}"#
            .as_bytes(),
    )
    .into_iter()
    .map(|item| item.expect("等待中不是错误"))
    .collect();
    let pending = events
        .iter()
        .find_map(|event| match event {
            B50Event::Pending { task, eta } => Some((task, eta)),
            _ => None,
        })
        .expect("应当给出等待事件");
    assert_eq!(pending.0.done, 7, "任务进度要对上");
    assert_eq!(pending.0.total, 50, "任务总量要对上");
    assert_eq!(pending.1.seconds, Some(30), "ETA 要对上");
}
