use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use aurorix_arcaea_plugin::model::Region;
use aurorix_arcaea_plugin::repo::legacy::{self, LegacyImportReport, LEGACY_OWNER_TABLE};
use aurorix_arcaea_plugin::repo::ArcaeaRepo;
use aurorix_db::{MigrationRegistry, PluginDb, PluginDbRegistry, RegistryConfig};
use chrono::{DateTime, Utc};
use serde_json::Value;
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
            "arc_repo_l_{}_{}",
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

async fn open_db(sandbox: &Sandbox, permits: usize) -> (PgPool, Arc<PluginDb>) {
    let pool = sandbox.pool(8).await;
    let registry = PluginDbRegistry::with_migrations(
        pool.clone(),
        RegistryConfig::new(permits, 0),
        Arc::new(MigrationRegistry::postgres(pool.clone())),
    );
    let db = registry.plugin("arcaea").expect("插件句柄");
    ArcaeaRepo::open(Arc::clone(&db))
        .await
        .expect("迁移应当成功");
    (pool, db)
}

async fn create_kv_table(pool: &PgPool) {
    sqlx::query(aurorix_db::schema::KV_DATA)
        .execute(pool)
        .await
        .expect("建 kv_data");
}

async fn create_owner_table(pool: &PgPool) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS arcaea_binding_owner (
            region      TEXT NOT NULL,
            arcaea_uid  BIGINT NOT NULL,
            user_id     TEXT NOT NULL,
            bound_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
            PRIMARY KEY (region, arcaea_uid)
        )",
    )
    .execute(pool)
    .await
    .expect("建老占用表");
}

async fn put_kv(pool: &PgPool, key: &str, value: &str) {
    sqlx::query(
        "INSERT INTO kv_data (scope, namespace, key, value) VALUES ('plugin', 'arcaea', $1, $2)",
    )
    .bind(key)
    .bind(value.as_bytes())
    .execute(pool)
    .await
    .expect("写老 kv 行");
}

async fn put_owner(pool: &PgPool, region: &str, uid: i64, user_id: &str, bound_at: &str) {
    sqlx::query(&format!(
        "INSERT INTO {LEGACY_OWNER_TABLE} (region, arcaea_uid, user_id, bound_at) VALUES ($1, $2, $3, $4)"
    ))
    .bind(region)
    .bind(uid)
    .bind(user_id)
    .bind(DateTime::parse_from_rfc3339(bound_at).expect("合法时间").with_timezone(&Utc))
    .execute(pool)
    .await
    .expect("写老占用行");
}

async fn kv_rows(pool: &PgPool) -> Vec<(String, String)> {
    sqlx::query_as("SELECT key, encode(value, 'escape') FROM kv_data ORDER BY key")
        .fetch_all(pool)
        .await
        .expect("读老 kv 行")
}

async fn owner_rows(pool: &PgPool) -> Vec<(String, i64, String, String)> {
    sqlx::query_as(&format!(
        "SELECT region, arcaea_uid, user_id, bound_at::text FROM {LEGACY_OWNER_TABLE} ORDER BY region, arcaea_uid"
    ))
    .fetch_all(pool)
    .await
    .expect("读老占用行")
}

async fn new_table_counts(pool: &PgPool) -> Vec<i64> {
    let mut counts = Vec::new();
    for table in [
        "arcaea_account",
        "arcaea_cn_device",
        "arcaea_setting",
        "arcaea_pref",
        "arcaea_skin",
        "arcaea_snapshot",
        "arcaea_legacy_import",
    ] {
        let count: i64 = sqlx::query_scalar(&format!("SELECT count(*) FROM {table}"))
            .fetch_one(pool)
            .await
            .expect("统计新表");
        counts.push(count);
    }
    counts
}

fn seconds(text: &str) -> i64 {
    DateTime::parse_from_rfc3339(text)
        .expect("合法时间")
        .timestamp()
}

const LAST_B50: &str = r#"{"At":"2026-11-10T18:02:00.5+08:00","ArcaeaUID":23784,"FriendCode":123456789,"Best":[{"SongID":"grievouslady","Score":10002000,"Rank":1,"Meta":{"Extra":{"Deep":[1,2,3]}}}],"Meta":{"Rank":1,"Note":"整份原样"}}"#;

async fn seed_legacy(pool: &PgPool) {
    create_kv_table(pool).await;
    create_owner_table(pool).await;
    put_kv(
        pool,
        "acct:intl:u1",
        r#"{"ArcaeaUID":23784,"FriendCode":"","Name":"","BoundAt":"2026-11-10T18:01:34.179706925+08:00"}"#,
    )
    .await;
    put_kv(
        pool,
        "acct:cn:u2",
        r#"{"ArcaeaUID":0,"FriendCode":431343680,"Name":"国服玩家","BoundAt":"0001-01-01T00:00:00Z"}"#,
    )
    .await;
    put_kv(
        pool,
        "acct:u3",
        r#"{"FriendCode":"123456789","UID":4242,"Token":"绝不该被导入的令牌"}"#,
    )
    .await;
    put_kv(pool, "acct:intl:broken", "这不是 JSON").await;
    put_kv(pool, "cn:device:u2", r#""0123456789abcdef""#).await;
    put_kv(pool, "cn:device:u7", r#""NOT-HEX""#).await;
    put_kv(pool, "cn:unbound:u2", r#""2026-11-11T03:04:05.123456789Z""#).await;
    put_kv(pool, "cn:bind_open", "false").await;
    put_kv(pool, "last:b50:intl:u1", LAST_B50).await;
    put_kv(
        pool,
        "skin:u1",
        r#"{"ID":12,"Uncapped":true,"Override":false,"Banner":"banner_12","SeenAt":"2026-11-10T18:03:00Z","PrefChar":"char_12.png","PrefBanner":"banner_12.png"}"#,
    )
    .await;
    put_kv(
        pool,
        "skin:u9",
        r#"{"ID":-1,"Uncapped":false,"Override":false,"Banner":"","SeenAt":"0001-01-01T00:00:00Z","PrefChar":"","PrefBanner":""}"#,
    )
    .await;
    put_kv(pool, "theme:u1", r#"{"name":"glacier"}"#).await;
    put_kv(pool, "theme:u3", r#"{"name":"arknights"}"#).await;
    put_kv(pool, "未知:键", r#"{"x":1}"#).await;
    put_owner(pool, "intl", 23784, "u1", "2026-11-10T18:01:34Z").await;
    put_owner(pool, "cn", 55566, "u4", "2026-11-12T00:00:00Z").await;
    put_owner(pool, "intl", 777, "u3", "2026-11-09T00:00:00Z").await;
}

#[tokio::test]
async fn case_421() {
    let Some(sandbox) = sandbox_or_skip().await else {
        return;
    };
    let (pool, db) = open_db(&sandbox, 4).await;
    seed_legacy(&pool).await;
    let kv_before = kv_rows(&pool).await;
    let owners_before = owner_rows(&pool).await;

    let report = legacy::import_once(&db).await.expect("首次导入应当成功");
    assert!(!report.already_imported, "第一次导入不该被台账挡住");
    assert_eq!(
        report,
        LegacyImportReport {
            accounts: 3,
            devices: 2,
            prefs: 1,
            skins: 2,
            themes: 2,
            snapshots: 1,
            gate: 1,
            owners: 2,
            skipped: 3,
            ignored: 1,
            already_imported: false,
        },
        "首次导入的统计不符合预期"
    );

    let accounts: Vec<(String, String, Option<i64>, String, String, i64)> = sqlx::query_as(
        "SELECT region, user_id, arcaea_uid, friend_code, name, trunc(extract(epoch from bound_at))::bigint
            FROM arcaea_account ORDER BY region, user_id",
    )
    .fetch_all(&pool)
    .await
    .expect("读新账号表");
    assert_eq!(accounts.len(), 4, "四个老账号应当全部落到新表");
    assert_eq!(accounts[0].0, "cn");
    assert_eq!(accounts[0].1, "u2");
    assert_eq!(accounts[0].2, None, "老记录 uid 为 0 必须落成待解析");
    assert_eq!(
        accounts[0].3, "431343680",
        "数字型 FriendCode 必须转成字符串"
    );
    assert_eq!(accounts[0].4, "国服玩家");
    assert!(
        accounts[0].5 > 946_684_800,
        "零值 BoundAt 绝不能进新表, 实际落成 {}",
        accounts[0].5
    );
    assert_eq!(accounts[1].0, "cn");
    assert_eq!(accounts[1].1, "u4");
    assert_eq!(accounts[1].2, Some(55566), "老占用表里的账号要进来");
    assert_eq!(accounts[1].5, seconds("2026-11-12T00:00:00Z"));
    assert_eq!(accounts[2].0, "intl");
    assert_eq!(accounts[2].1, "u1");
    assert_eq!(accounts[2].2, Some(23784));
    assert_eq!(accounts[2].5, seconds("2026-11-10T18:01:34+08:00"));
    assert_eq!(accounts[3].0, "intl");
    assert_eq!(accounts[3].1, "u3", "无区域段的老键必须算国际服");
    assert_eq!(accounts[3].2, Some(777), "老占用表应当补齐待解析的 uid");
    assert_eq!(accounts[3].3, "123456789");
    assert!(
        accounts[3].5 > 946_684_800,
        "缺 BoundAt 时必须补成真实时间, 实际落成 {}",
        accounts[3].5
    );

    let devices: Vec<(String, Option<String>, Option<i64>)> = sqlx::query_as(
        "SELECT user_id, device_id, trunc(extract(epoch from last_unbound_at))::bigint
            FROM arcaea_cn_device ORDER BY user_id",
    )
    .fetch_all(&pool)
    .await
    .expect("读设备表");
    assert_eq!(devices.len(), 1, "合法设备码才该落到设备表");
    assert_eq!(devices[0].0, "u2");
    assert_eq!(devices[0].1.as_deref(), Some("0123456789abcdef"));
    assert_eq!(
        devices[0].2,
        Some(seconds("2026-11-11T03:04:05Z")),
        "解绑时刻必须按秒对齐"
    );
    let bad_device: i64 =
        sqlx::query_scalar("SELECT count(*) FROM arcaea_cn_device WHERE user_id = 'u7'")
            .fetch_one(&pool)
            .await
            .expect("统计非法设备");
    assert_eq!(bad_device, 0, "非法设备码绝不该写进新表");

    let gate: bool =
        sqlx::query_scalar("SELECT value FROM arcaea_setting WHERE name = 'cn_bind_gate'")
            .fetch_one(&pool)
            .await
            .expect("读绑定开关");
    assert!(!gate, "老开关写了 false 就必须读回 false");

    let prefs: Vec<(String, String, String, String, String)> = sqlx::query_as(
        "SELECT region, user_id, pref_char, pref_banner, theme FROM arcaea_pref ORDER BY user_id",
    )
    .fetch_all(&pool)
    .await
    .expect("读偏好表");
    assert_eq!(
        prefs,
        vec![
            (
                "intl".to_string(),
                "u1".to_string(),
                "char_12.png".to_string(),
                "banner_12.png".to_string(),
                "glacier".to_string()
            ),
            (
                "intl".to_string(),
                "u3".to_string(),
                String::new(),
                String::new(),
                "arknights".to_string()
            ),
        ],
        "皮肤里的自选与主题必须各自落到偏好表"
    );

    let skins: Vec<(String, i32, bool, bool, String, Option<i64>)> = sqlx::query_as(
        "SELECT user_id, char_id, uncapped, override_form, banner, trunc(extract(epoch from seen_at))::bigint
            FROM arcaea_skin ORDER BY user_id",
    )
    .fetch_all(&pool)
    .await
    .expect("读皮肤表");
    assert_eq!(skins.len(), 2);
    assert_eq!(skins[0].0, "u1");
    assert_eq!(skins[0].1, 12);
    assert!(skins[0].2, "Uncapped 应当保留");
    assert!(!skins[0].3);
    assert_eq!(skins[0].4, "banner_12");
    assert_eq!(skins[0].5, Some(seconds("2026-11-10T18:03:00Z")));
    assert_eq!(skins[1].0, "u9");
    assert_eq!(skins[1].1, -1, "ID 小于 0 必须落成未知皮肤");
    assert_eq!(skins[1].5, None, "零值 SeenAt 绝不能进新表");

    let snapshot: (Value, Option<i64>, String, i64) = sqlx::query_as(
        "SELECT payload, arcaea_uid, friend_code, trunc(extract(epoch from captured_at))::bigint
            FROM arcaea_snapshot WHERE region = 'intl' AND user_id = 'u1'",
    )
    .fetch_one(&pool)
    .await
    .expect("读快照");
    assert_eq!(
        snapshot.0,
        serde_json::from_str::<Value>(LAST_B50).expect("老快照 JSON"),
        "快照 payload 必须整份原样进库"
    );
    assert_eq!(snapshot.1, Some(23784));
    assert_eq!(
        snapshot.2, "123456789",
        "快照里的数字型好友码也要转成字符串"
    );
    assert_eq!(snapshot.3, seconds("2026-11-10T18:02:00+08:00"));

    let zero_rows: i64 = sqlx::query_scalar(
        "SELECT (SELECT count(*) FROM arcaea_account WHERE bound_at < '2000-01-01')
              + (SELECT count(*) FROM arcaea_skin WHERE seen_at IS NOT NULL AND seen_at < '2000-01-01')
              + (SELECT count(*) FROM arcaea_snapshot WHERE captured_at < '2000-01-01')",
    )
    .fetch_one(&pool)
    .await
    .expect("统计零值时间");
    assert_eq!(zero_rows, 0, "任何零值时间都不许进新表");

    let after_first = new_table_counts(&pool).await;

    let second = legacy::import_once(&db).await.expect("第二次导入应当成功");
    assert!(second.already_imported, "第二次导入必须被台账挡住");
    assert_eq!(second.total(), 0, "第二次导入不该多插任何数据");
    assert_eq!(second.ignored, 0, "台账挡住的运行不该再碰到任何老行");
    assert_eq!(second.skipped, 0, "台账挡住的运行不该再统计跳过");
    assert_eq!(
        new_table_counts(&pool).await,
        after_first,
        "第二次导入不该改变任何新表的行数"
    );
    assert_eq!(kv_rows(&pool).await, kv_before, "老 kv 数据必须一字未变");
    assert_eq!(
        owner_rows(&pool).await,
        owners_before,
        "老占用表必须一字未变"
    );

    let third = legacy::import_once(&db).await.expect("第三次导入应当成功");
    assert!(third.already_imported);
    assert_eq!(third.total(), 0);
    sandbox.drop_schema().await;
}

#[tokio::test]
async fn case_422() {
    let Some(sandbox) = sandbox_or_skip().await else {
        return;
    };
    let (pool, db) = open_db(&sandbox, 4).await;
    create_kv_table(&pool).await;
    put_kv(
        &pool,
        "acct:intl:u5",
        r#"{"ArcaeaUID":31337,"FriendCode":"999888777","Name":"老玩家","BoundAt":"2026-11-10T18:01:34Z"}"#,
    )
    .await;
    let repo = ArcaeaRepo::from_db(Arc::clone(&db));

    let account = repo
        .get_account(Region::Intl, "u5")
        .await
        .expect("读账号")
        .expect("首次未命中应当触发老数据导入");
    assert_eq!(account.uid(), Some(31337));
    assert_eq!(account.friend_code, "999888777");
    assert_eq!(account.name, "老玩家");
    assert_eq!(
        repo.get_account(Region::Intl, "不存在的人")
            .await
            .expect("读账号"),
        None,
        "没有老数据的人必须读不到"
    );
    let ledger: i64 = sqlx::query_scalar("SELECT count(*) FROM arcaea_legacy_import")
        .fetch_one(&pool)
        .await
        .expect("读台账");
    assert_eq!(ledger, 1, "一次性导入只能留下一条台账");
    sandbox.drop_schema().await;
}

#[tokio::test]
async fn case_423() {
    let Some(sandbox) = sandbox_or_skip().await else {
        return;
    };
    let (pool, db) = open_db(&sandbox, 4).await;
    create_kv_table(&pool).await;

    let report = legacy::import_once(&db).await.expect("空导入应当成功");
    assert_eq!(report.total(), 0);
    assert_eq!(report.skipped, 0);
    assert!(!report.already_imported);
    let stored: i64 = sqlx::query_scalar("SELECT count(*) FROM arcaea_legacy_import")
        .fetch_one(&pool)
        .await
        .expect("读台账");
    assert_eq!(stored, 1, "空导入也要写台账, 保证只跑一次");
    let again = legacy::import_once(&db).await.expect("再次导入");
    assert!(again.already_imported);
    sandbox.drop_schema().await;
}

#[tokio::test]
async fn case_424() {
    let Some(sandbox) = sandbox_or_skip().await else {
        return;
    };
    let (pool, db) = open_db(&sandbox, 4).await;
    let report = legacy::import_once(&db).await.expect("缺表也应当成功");
    assert_eq!(report.total(), 0, "没有老表就没有可导入的数据");
    assert!(report.is_empty(), "没有老表时统计应当全是零");
    let kv_table: Option<String> = sqlx::query_scalar("SELECT to_regclass('kv_data')::text")
        .fetch_one(&pool)
        .await
        .expect("查表存在性");
    assert!(
        kv_table.is_none(),
        "老 kv 表本来就不该存在, 实际为 {kv_table:?}"
    );
    sandbox.drop_schema().await;
}
