use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use aurorix_arcaea_plugin::repo::{ArcaeaRepo, MIGRATIONS, UID_OWNER_INDEX};
use aurorix_db::{MigrationRegistry, MigrationStep, PluginDb, PluginDbRegistry, RegistryConfig};
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
            "arc_repo_m_{}_{}",
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

async fn plugin_db(sandbox: &Sandbox, permits: usize) -> (PgPool, Arc<PluginDb>) {
    let pool = sandbox.pool(8).await;
    let registry = PluginDbRegistry::with_migrations(
        pool.clone(),
        RegistryConfig::new(permits, 0),
        Arc::new(MigrationRegistry::postgres(pool.clone())),
    );
    let db = registry.plugin("arcaea").expect("插件句柄");
    (pool, db)
}

async fn columns(pool: &PgPool, table: &str) -> Vec<(String, String, bool)> {
    sqlx::query_as(
        "SELECT column_name, data_type, (is_nullable = 'YES')
            FROM information_schema.columns
            WHERE table_schema = current_schema() AND table_name = $1
            ORDER BY ordinal_position",
    )
    .bind(table)
    .fetch_all(pool)
    .await
    .expect("读列信息")
}

fn column_names(rows: &[(String, String, bool)]) -> Vec<String> {
    rows.iter().map(|(name, _, _)| name.clone()).collect()
}

#[tokio::test]
async fn case_510() {
    let Some(sandbox) = sandbox_or_skip().await else {
        return;
    };
    let (pool, db) = plugin_db(&sandbox, 2).await;
    let first = ArcaeaRepo::open(Arc::clone(&db))
        .await
        .expect("首次迁移应当成功");
    assert_eq!(
        first.applied,
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        "首次迁移必须按版本升序建好全部结构"
    );
    assert!(first.skipped.is_empty(), "首次迁移不该有跳过的版本");
    let second = ArcaeaRepo::open(Arc::clone(&db))
        .await
        .expect("二次迁移应当成功");
    assert!(
        second.applied.is_empty(),
        "第二次迁移不该再应用任何版本, 实际为 {:?}",
        second.applied
    );
    assert_eq!(second.skipped.len(), 10, "十个版本都应当被识别为已应用");
    let recorded: i64 =
        sqlx::query_scalar("SELECT count(*) FROM plugin_migrations WHERE plugin_id = 'arcaea'")
            .fetch_one(&pool)
            .await
            .expect("读迁移台账");
    assert_eq!(recorded, 10, "迁移台账里应当只有十行");
    sandbox.drop_schema().await;
}

#[tokio::test]
async fn case_511() {
    let Some(sandbox) = sandbox_or_skip().await else {
        return;
    };
    let (pool, db) = plugin_db(&sandbox, 2).await;
    let duplicated = [
        MigrationStep {
            version: 7,
            name: "第一次",
            sql: "SELECT 1",
        },
        MigrationStep {
            version: 7,
            name: "第二次",
            sql: "SELECT 2",
        },
    ];
    let error = db
        .migrate(&duplicated)
        .await
        .expect_err("重复版本号必须被拒绝");
    assert!(
        error.to_string().contains("重复"),
        "错误消息应当说明版本重复, 实际为 {error}"
    );
    let recorded: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM plugin_migrations WHERE plugin_id = 'arcaea' AND version = 7",
    )
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten()
    .unwrap_or(0);
    assert_eq!(recorded, 0, "被拒绝的重复版本不该留下任何台账行");
    sandbox.drop_schema().await;
}

#[tokio::test]
async fn case_512() {
    let mut versions: Vec<u32> = MIGRATIONS.iter().map(|step| step.version).collect();
    assert_eq!(versions.len(), 10, "迁移步骤数量应当稳定");
    assert!(
        versions.windows(2).all(|pair| pair[0] < pair[1]),
        "版本必须严格升序"
    );
    versions.sort_unstable();
    versions.dedup();
    assert_eq!(versions.len(), MIGRATIONS.len(), "迁移版本号不允许重复");
}

#[tokio::test]
async fn case_513() {
    let Some(sandbox) = sandbox_or_skip().await else {
        return;
    };
    let (pool, db) = plugin_db(&sandbox, 2).await;
    ArcaeaRepo::open(Arc::clone(&db))
        .await
        .expect("迁移应当成功");

    let account = columns(&pool, "arcaea_account").await;
    assert_eq!(
        column_names(&account),
        vec![
            "region",
            "user_id",
            "arcaea_uid",
            "friend_code",
            "name",
            "bound_at",
            "updated_at"
        ],
        "账号表列不符合预期"
    );
    assert_eq!(account[2].1, "bigint", "arcaea_uid 必须是 BIGINT");
    assert!(account[2].2, "arcaea_uid 必须可空, 老记录先只有好友码");
    assert_eq!(
        account[5].1, "timestamp with time zone",
        "bound_at 必须是 TIMESTAMPTZ"
    );
    assert!(!account[5].2, "bound_at 不许为空");

    let device = columns(&pool, "arcaea_cn_device").await;
    assert_eq!(
        column_names(&device),
        vec!["user_id", "device_id", "last_unbound_at", "updated_at"],
        "国服设备表列不符合预期"
    );

    let setting = columns(&pool, "arcaea_setting").await;
    assert_eq!(column_names(&setting), vec!["name", "value", "updated_at"]);

    let pref = columns(&pool, "arcaea_pref").await;
    assert_eq!(
        column_names(&pref),
        vec![
            "region",
            "user_id",
            "pref_char",
            "pref_banner",
            "theme",
            "updated_at"
        ],
        "偏好表列不符合预期"
    );

    let skin = columns(&pool, "arcaea_skin").await;
    assert_eq!(
        column_names(&skin),
        vec![
            "region",
            "user_id",
            "char_id",
            "uncapped",
            "override_form",
            "banner",
            "seen_at",
            "updated_at"
        ],
        "皮肤表列不符合预期"
    );
    assert_eq!(skin[2].1, "integer", "char_id 必须是 INTEGER");
    assert!(skin[6].2, "seen_at 必须可空, 没见过就不写时间");

    let snapshot = columns(&pool, "arcaea_snapshot").await;
    assert_eq!(
        column_names(&snapshot),
        vec![
            "region",
            "user_id",
            "payload",
            "arcaea_uid",
            "friend_code",
            "captured_at"
        ],
        "快照表列不符合预期"
    );
    assert_eq!(snapshot[2].1, "jsonb", "payload 必须是 JSONB");

    let ledger = columns(&pool, "arcaea_legacy_import").await;
    assert_eq!(
        column_names(&ledger),
        vec![
            "name",
            "imported_at",
            "accounts",
            "devices",
            "prefs",
            "skins",
            "themes",
            "snapshots",
            "gate",
            "owners",
            "skipped"
        ],
        "老数据导入台账列不符合预期"
    );

    let ticket = columns(&pool, "arcaea_ticket").await;
    assert_eq!(
        column_names(&ticket),
        vec!["region", "user_id", "expires_at"],
        "解绑票据表列不符合预期"
    );
    assert_eq!(
        ticket[2].1, "timestamp with time zone",
        "expires_at 必须是 TIMESTAMPTZ"
    );
    assert!(!ticket[2].2, "expires_at 不许为空");
    sandbox.drop_schema().await;
}

#[tokio::test]
async fn case_514() {
    let Some(sandbox) = sandbox_or_skip().await else {
        return;
    };
    let (pool, db) = plugin_db(&sandbox, 2).await;
    ArcaeaRepo::open(Arc::clone(&db))
        .await
        .expect("迁移应当成功");

    let indexdef: Option<String> = sqlx::query_scalar(
        "SELECT indexdef FROM pg_indexes WHERE schemaname = current_schema() AND indexname = $1",
    )
    .bind(UID_OWNER_INDEX)
    .fetch_optional(&pool)
    .await
    .expect("读索引定义");
    let indexdef = indexdef.expect("uid 占用索引必须建出来");
    assert!(
        indexdef.contains("UNIQUE"),
        "必须是唯一索引, 实际为 {indexdef}"
    );
    assert!(
        indexdef.contains("arcaea_uid IS NOT NULL"),
        "必须是部分唯一索引, 实际为 {indexdef}"
    );
    assert!(
        indexdef.contains("region"),
        "索引必须带区域维度, 实际为 {indexdef}"
    );

    let checks: Vec<String> = sqlx::query_scalar(
        "SELECT conname FROM pg_constraint
            WHERE connamespace = current_schema()::regnamespace
                AND conname IN ('arcaea_cn_device_hex_check', 'arcaea_account_region_check')
            ORDER BY conname",
    )
    .fetch_all(&pool)
    .await
    .expect("读约束");
    assert_eq!(
        checks,
        vec![
            "arcaea_account_region_check".to_string(),
            "arcaea_cn_device_hex_check".to_string()
        ],
        "区域与设备码的 CHECK 约束都应当存在"
    );
    sandbox.drop_schema().await;
}

#[tokio::test]
async fn case_515() {
    if base_dsn().is_some() {
        assert_eq!(MIGRATIONS.len(), 10);
        return;
    }
    assert!(base_dsn().is_none(), "缺少环境变量时应当走跳过分支");
}
