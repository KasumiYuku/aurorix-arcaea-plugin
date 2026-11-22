use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use aurorix_arcaea_plugin::model::{Account, Prefs, Region, Skin};
use aurorix_arcaea_plugin::repo::errors;
use aurorix_arcaea_plugin::repo::{ArcaeaRepo, SnapshotRecord};
use aurorix_db::{MigrationRegistry, PluginDb, PluginDbRegistry, RegistryConfig};
use chrono::{TimeZone, Utc};
use serde_json::json;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;

static SEQ: AtomicU64 = AtomicU64::new(0);

fn base_dsn() -> Option<String> {
    std::env::var("AURORIX_TEST_DATABASE")
        .ok()
        .filter(|dsn| !dsn.is_empty())
}

struct Sandbox {
    admin: PgPool,
    schema: String,
}

impl Sandbox {
    async fn new() -> Option<Self> {
        let dsn = base_dsn()?;
        let admin = PgPoolOptions::new()
            .max_connections(2)
            .connect(&dsn)
            .await
            .ok()?;
        let schema = format!(
            "arc_repo_a_{}_{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::SeqCst)
        );
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&admin)
            .await
            .ok()?;
        Some(Self { admin, schema })
    }

    async fn pool(&self, max_connections: u32) -> PgPool {
        let dsn = base_dsn().expect("测试库连接串");
        let options = PgConnectOptions::from_str(&dsn)
            .expect("解析连接串")
            .options([("search_path", self.schema.as_str())]);
        PgPoolOptions::new()
            .max_connections(max_connections)
            .connect_with(options)
            .await
            .expect("连接沙箱")
    }

    async fn drop_schema(&self) {
        let _ = sqlx::query(&format!("DROP SCHEMA IF EXISTS {} CASCADE", self.schema))
            .execute(&self.admin)
            .await;
    }
}

async fn sandbox_or_skip() -> Option<Sandbox> {
    match Sandbox::new().await {
        Some(sandbox) => Some(sandbox),
        None => {
            eprintln!("跳过需要 PostgreSQL 的用例: 未设置 AURORIX_TEST_DATABASE");
            None
        }
    }
}

async fn open_repo(sandbox: &Sandbox, permits: usize) -> (PgPool, ArcaeaRepo) {
    let pool = sandbox.pool(8).await;
    let registry = PluginDbRegistry::with_migrations(
        pool.clone(),
        RegistryConfig::new(permits, 0),
        Arc::new(MigrationRegistry::postgres(pool.clone())),
    );
    let db: Arc<PluginDb> = registry.plugin("arcaea").expect("插件句柄");
    ArcaeaRepo::open(Arc::clone(&db))
        .await
        .expect("迁移应当成功");
    (pool, ArcaeaRepo::from_db(db))
}

fn fixed_time() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 10, 10, 1, 34)
        .single()
        .expect("合法时间")
}

fn account(region: Region, user_id: &str, uid: i64) -> Account {
    let mut account = Account::new(region, user_id, uid);
    account.bound_at = fixed_time();
    account
}

#[tokio::test]
async fn case_501() {
    let Some(sandbox) = sandbox_or_skip().await else {
        return;
    };
    let (_pool, repo) = open_repo(&sandbox, 4).await;
    let mut binding = account(Region::Intl, "u1", 23784);
    binding.friend_code = "431343680".to_string();
    binding.name = "光".to_string();
    repo.put_account(&binding).await.expect("绑定应当成功");

    let loaded = repo
        .get_account(Region::Intl, "u1")
        .await
        .expect("读账号")
        .expect("绑定后必须能读到");
    assert_eq!(loaded, binding, "读回的账号应当与写入的一字不差");
    assert_eq!(loaded.uid(), Some(23784));

    assert!(
        repo.delete_account(Region::Intl, "u1")
            .await
            .expect("解绑应当成功"),
        "解绑应当真的删掉一行"
    );
    assert!(
        repo.get_account(Region::Intl, "u1")
            .await
            .expect("再次读取")
            .is_none(),
        "解绑之后不该再读到账号"
    );
    assert!(
        !repo
            .delete_account(Region::Intl, "u1")
            .await
            .expect("重复解绑"),
        "重复解绑应当返回 false"
    );
    sandbox.drop_schema().await;
}

#[tokio::test]
async fn case_502() {
    let Some(sandbox) = sandbox_or_skip().await else {
        return;
    };
    let (_pool, repo) = open_repo(&sandbox, 4).await;
    repo.put_account(&account(Region::Intl, "a", 23784))
        .await
        .expect("第一个人绑定应当成功");

    let error = repo
        .put_account(&account(Region::Intl, "b", 23784))
        .await
        .expect_err("同一个 uid 的第二个人必须被拒绝");
    assert!(
        errors::is_uid_taken(&error),
        "错误必须能被识别为账号已被占用, 实际为 {error}"
    );
    assert_eq!(error.to_string(), "这个账号已经绑在别人名下了");

    let mut again = account(Region::Intl, "a", 23784);
    again.name = "改名了".to_string();
    repo.put_account(&again).await.expect("本人改绑应当成功");
    assert_eq!(
        repo.get_account(Region::Intl, "a")
            .await
            .expect("读账号")
            .expect("账号仍在")
            .name,
        "改名了"
    );

    repo.put_account(&account(Region::Cn, "b", 23784))
        .await
        .expect("国服与国际服是两套 uid 空间");
    assert_eq!(
        repo.get_account(Region::Cn, "b")
            .await
            .expect("读国服账号")
            .expect("国服账号在")
            .uid(),
        Some(23784)
    );
    sandbox.drop_schema().await;
}

#[tokio::test]
async fn case_503() {
    let Some(sandbox) = sandbox_or_skip().await else {
        return;
    };
    let (_pool, repo) = open_repo(&sandbox, 4).await;
    let mut pending = account(Region::Intl, "u1", 0);
    pending.friend_code = "431343680".to_string();
    repo.put_account(&pending).await.expect("先落成待解析行");
    let stored = repo
        .get_account(Region::Intl, "u1")
        .await
        .expect("读账号")
        .expect("待解析行在");
    assert!(stored.is_pending(), "还没有 uid 时应当算待解析");
    assert!(stored.needs_friend_code_resolve());

    let resolved = repo
        .resolve_pending(Region::Intl, "u1", 23784, "光")
        .await
        .expect("补 uid 应当成功");
    assert_eq!(resolved.uid(), Some(23784));
    assert_eq!(resolved.name, "光");
    assert_eq!(resolved.friend_code, "431343680", "补 uid 不该丢好友码");
    assert_eq!(resolved.bound_at, fixed_time(), "补 uid 不该改绑定时间");

    let error = repo
        .resolve_pending(Region::Intl, "u1", 999, "别人")
        .await
        .expect_err("已经没有待解析的行了");
    assert!(
        errors::is_pending_missing(&error),
        "必须报找不到待解析记录, 实际为 {error}"
    );
    sandbox.drop_schema().await;
}

#[tokio::test]
async fn case_504() {
    let Some(sandbox) = sandbox_or_skip().await else {
        return;
    };
    let (_pool, repo) = open_repo(&sandbox, 4).await;
    repo.put_account(&account(Region::Intl, "owner", 23784))
        .await
        .expect("先让别人占住 uid");
    let mut pending = account(Region::Intl, "u1", 0);
    pending.friend_code = "431343680".to_string();
    repo.put_account(&pending).await.expect("落成待解析行");

    let error = repo
        .resolve_pending(Region::Intl, "u1", 23784, "光")
        .await
        .expect_err("uid 已被别人占用");
    assert!(
        errors::is_uid_taken(&error),
        "补 uid 的冲突必须是同一种「已被占用」错误, 实际为 {error}"
    );
    assert_eq!(error.to_string(), "这个账号已经绑在别人名下了");
    assert!(
        repo.get_account(Region::Intl, "u1")
            .await
            .expect("读账号")
            .expect("行还在")
            .is_pending(),
        "失败的补 uid 不该改动原行"
    );
    sandbox.drop_schema().await;
}

#[tokio::test]
async fn case_505() {
    let Some(sandbox) = sandbox_or_skip().await else {
        return;
    };
    let (pool, repo) = open_repo(&sandbox, 4).await;
    let (first, second, third, fourth) = tokio::join!(
        repo.take_device_id("u1"),
        repo.take_device_id("u1"),
        repo.take_device_id("u1"),
        repo.take_device_id("u1")
    );
    let first = first.expect("第一个设备码");
    let second = second.expect("第二个设备码");
    let third = third.expect("第三个设备码");
    let fourth = fourth.expect("第四个设备码");
    assert_eq!(first, second, "并发取设备码不该产出两台设备");
    assert_eq!(first, third, "并发取设备码不该产出两台设备");
    assert_eq!(first, fourth, "并发取设备码不该产出两台设备");
    assert_eq!(first.len(), 16, "设备码必须是 16 位, 实际为 {first}");
    assert!(
        first
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
        "设备码必须是小写十六进制, 实际为 {first}"
    );
    let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM arcaea_cn_device")
        .fetch_one(&pool)
        .await
        .expect("统计设备行");
    assert_eq!(rows, 1, "一个用户只该有一行设备记录");
    let stored: String = sqlx::query_scalar("SELECT device_id FROM arcaea_cn_device")
        .fetch_one(&pool)
        .await
        .expect("读设备码");
    assert_eq!(stored, first, "库里的设备码必须就是返回给调用方的那个");
    assert_eq!(
        repo.take_device_id("u1").await.expect("再取一次"),
        first,
        "已经存在的设备码必须稳定返回"
    );
    sandbox.drop_schema().await;
}

#[tokio::test]
async fn case_506() {
    let Some(sandbox) = sandbox_or_skip().await else {
        return;
    };
    let (pool, repo) = open_repo(&sandbox, 4).await;
    assert!(
        repo.unbound_at("u2").await.expect("读解绑时刻").is_none(),
        "从没绑过的用户不该有解绑时刻"
    );
    let device = repo.take_device_id("u2").await.expect("取设备码");
    repo.mark_unbound("u2").await.expect("记解绑时刻");
    let moment = repo
        .unbound_at("u2")
        .await
        .expect("读解绑时刻")
        .expect("刚解绑的用户必须有解绑时刻");
    assert!(
        moment.timestamp_millis() > 0,
        "解绑时刻必须是真实时间, 实际为 {moment}"
    );
    let stored: String =
        sqlx::query_scalar("SELECT device_id FROM arcaea_cn_device WHERE user_id = 'u2'")
            .fetch_one(&pool)
            .await
            .expect("读设备码");
    assert_eq!(stored, device, "解绑之后设备码必须保留");
    assert_eq!(
        repo.take_device_id("u2").await.expect("再取设备码"),
        device,
        "解绑之后仍要复用原设备码"
    );
    sandbox.drop_schema().await;
}

#[tokio::test]
async fn case_507() {
    let Some(sandbox) = sandbox_or_skip().await else {
        return;
    };
    let (_pool, repo) = open_repo(&sandbox, 4).await;
    assert!(
        repo.bind_gate().await.expect("读开关"),
        "读不到开关时必须默认开放"
    );
    repo.set_bind_gate(false).await.expect("关闭开关");
    assert!(
        !repo.bind_gate().await.expect("读开关"),
        "关掉之后必须读回关闭"
    );
    repo.set_bind_gate(true).await.expect("重新打开");
    assert!(
        repo.bind_gate().await.expect("读开关"),
        "重新打开后应当开放"
    );
    sandbox.drop_schema().await;
}

#[tokio::test]
async fn case_508() {
    let Some(sandbox) = sandbox_or_skip().await else {
        return;
    };
    let (_pool, repo) = open_repo(&sandbox, 4).await;
    assert!(
        repo.prefs(Region::Intl, "u1")
            .await
            .expect("读偏好")
            .is_empty(),
        "没写过偏好时应当是空偏好"
    );
    assert!(
        !repo.skin(Region::Intl, "u1").await.expect("读皮肤").known(),
        "没观测过皮肤时应当是未知皮肤"
    );

    let prefs = Prefs {
        char: "char_12.png".to_string(),
        banner: "banner_12.png".to_string(),
        theme: "glacier".to_string(),
    };
    repo.set_prefs(Region::Intl, "u1", &prefs)
        .await
        .expect("写偏好");
    assert_eq!(repo.prefs(Region::Intl, "u1").await.expect("读偏好"), prefs);

    let mut skin = Skin::unknown();
    skin.char_id = 12;
    skin.uncapped = true;
    skin.banner = "seen_12.png".to_string();
    skin.seen_at = Some(fixed_time());
    repo.remember_skin(Region::Intl, "u1", &skin)
        .await
        .expect("写观测皮肤");
    assert_eq!(repo.skin(Region::Intl, "u1").await.expect("读皮肤"), skin);
    assert_eq!(
        repo.prefs(Region::Intl, "u1").await.expect("读偏好"),
        prefs,
        "写观测皮肤不许改动自选偏好"
    );

    let updated = Prefs {
        char: String::new(),
        banner: String::new(),
        theme: "arknights".to_string(),
    };
    repo.set_prefs(Region::Intl, "u1", &updated)
        .await
        .expect("改偏好");
    assert_eq!(
        repo.skin(Region::Intl, "u1").await.expect("读皮肤"),
        skin,
        "改自选偏好不许改动观测皮肤"
    );
    assert_eq!(
        repo.prefs(Region::Cn, "u1").await.expect("读国服偏好"),
        Prefs::default(),
        "两个区域的偏好互不相干"
    );
    sandbox.drop_schema().await;
}

#[tokio::test]
async fn case_509() {
    let Some(sandbox) = sandbox_or_skip().await else {
        return;
    };
    let (_pool, repo) = open_repo(&sandbox, 4).await;
    assert!(
        repo.load_snapshot(Region::Intl, "u1")
            .await
            .expect("读快照")
            .is_none(),
        "没存过快照时应当读不到"
    );
    let payload = json!({
        "best": [{"song": "grievous lady", "score": 10002000}],
        "meta": {"rank": 1, "nested": {"deep": [1, 2, 3]}},
    });
    let snapshot = SnapshotRecord {
        region: Region::Intl,
        user_id: "u1".to_string(),
        payload: payload.clone(),
        arcaea_uid: Some(23784),
        friend_code: "431343680".to_string(),
        captured_at: fixed_time(),
    };
    repo.save_snapshot(&snapshot).await.expect("存快照");
    let loaded = repo
        .load_snapshot(Region::Intl, "u1")
        .await
        .expect("读快照")
        .expect("快照在");
    assert_eq!(loaded, snapshot, "快照应当原样读回");

    let newer = SnapshotRecord {
        captured_at: Utc
            .with_ymd_and_hms(2026, 11, 11, 0, 0, 0)
            .single()
            .unwrap(),
        payload: json!({"best": []}),
        ..snapshot
    };
    repo.save_snapshot(&newer).await.expect("覆盖快照");
    assert_eq!(
        repo.load_snapshot(Region::Intl, "u1")
            .await
            .expect("读快照")
            .expect("快照在"),
        newer,
        "同一个用户只保留最后一份快照"
    );
    sandbox.drop_schema().await;
}
