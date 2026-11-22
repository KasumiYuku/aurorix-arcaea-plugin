pub mod legacy;

use std::sync::Arc;

use aurorix_db::{DbError, MigrationReport, MigrationStep, PluginDb};
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;

use crate::model::{Account, Prefs, Region, Skin};

pub const ACCOUNT_TABLE: &str = "arcaea_account";

pub const DEVICE_TABLE: &str = "arcaea_cn_device";

pub const SETTING_TABLE: &str = "arcaea_setting";

pub const PREF_TABLE: &str = "arcaea_pref";

pub const SKIN_TABLE: &str = "arcaea_skin";

pub const SNAPSHOT_TABLE: &str = "arcaea_snapshot";

pub const LEGACY_LEDGER_TABLE: &str = "arcaea_legacy_import";
pub const TICKET_TABLE: &str = "arcaea_ticket";

pub const UID_OWNER_INDEX: &str = "arcaea_account_uid_owner";

pub const USER_INDEX: &str = "arcaea_account_user";

pub const BIND_GATE_KEY: &str = "cn_bind_gate";

pub const TABLES: [&str; 8] = [
    ACCOUNT_TABLE,
    DEVICE_TABLE,
    SETTING_TABLE,
    PREF_TABLE,
    SKIN_TABLE,
    SNAPSHOT_TABLE,
    LEGACY_LEDGER_TABLE,
    TICKET_TABLE,
];

const ACCOUNT_DDL: &str = "\
CREATE TABLE IF NOT EXISTS arcaea_account (
\tregion      TEXT NOT NULL,
\tuser_id     TEXT NOT NULL,
\tarcaea_uid  BIGINT,
\tfriend_code TEXT NOT NULL DEFAULT '',
\tname        TEXT NOT NULL DEFAULT '',
\tbound_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
\tupdated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
\tPRIMARY KEY (region, user_id),
\tCONSTRAINT arcaea_account_region_check CHECK (region IN ('intl', 'cn'))
)";

const UID_OWNER_DDL: &str = "\
CREATE UNIQUE INDEX IF NOT EXISTS arcaea_account_uid_owner
\tON arcaea_account (region, arcaea_uid)
\tWHERE arcaea_uid IS NOT NULL";

const USER_INDEX_DDL: &str = "\
CREATE INDEX IF NOT EXISTS arcaea_account_user ON arcaea_account (user_id)";

const DEVICE_DDL: &str = "\
CREATE TABLE IF NOT EXISTS arcaea_cn_device (
\tuser_id         TEXT PRIMARY KEY,
\tdevice_id       TEXT,
\tlast_unbound_at TIMESTAMPTZ,
\tupdated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
\tCONSTRAINT arcaea_cn_device_hex_check
\t\tCHECK (device_id IS NULL OR device_id ~ '^[0-9a-f]{16}$')
)";

const SETTING_DDL: &str = "\
CREATE TABLE IF NOT EXISTS arcaea_setting (
\tname       TEXT PRIMARY KEY,
\tvalue      BOOLEAN NOT NULL,
\tupdated_at TIMESTAMPTZ NOT NULL DEFAULT now()
)";

const PREF_DDL: &str = "\
CREATE TABLE IF NOT EXISTS arcaea_pref (
\tregion      TEXT NOT NULL,
\tuser_id     TEXT NOT NULL,
\tpref_char   TEXT NOT NULL DEFAULT '',
\tpref_banner TEXT NOT NULL DEFAULT '',
\ttheme       TEXT NOT NULL DEFAULT '',
\tupdated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
\tPRIMARY KEY (region, user_id),
\tCONSTRAINT arcaea_pref_region_check CHECK (region IN ('intl', 'cn'))
)";

const SKIN_DDL: &str = "\
CREATE TABLE IF NOT EXISTS arcaea_skin (
\tregion        TEXT NOT NULL,
\tuser_id       TEXT NOT NULL,
\tchar_id       INTEGER NOT NULL DEFAULT -1,
\tuncapped      BOOLEAN NOT NULL DEFAULT false,
\toverride_form BOOLEAN NOT NULL DEFAULT false,
\tbanner        TEXT NOT NULL DEFAULT '',
\tseen_at       TIMESTAMPTZ,
\tupdated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
\tPRIMARY KEY (region, user_id),
\tCONSTRAINT arcaea_skin_region_check CHECK (region IN ('intl', 'cn'))
)";

const SNAPSHOT_DDL: &str = "\
CREATE TABLE IF NOT EXISTS arcaea_snapshot (
\tregion      TEXT NOT NULL,
\tuser_id     TEXT NOT NULL,
\tpayload     JSONB NOT NULL,
\tarcaea_uid  BIGINT,
\tfriend_code TEXT NOT NULL DEFAULT '',
\tcaptured_at TIMESTAMPTZ NOT NULL DEFAULT now(),
\tPRIMARY KEY (region, user_id),
\tCONSTRAINT arcaea_snapshot_region_check CHECK (region IN ('intl', 'cn'))
)";

const LEGACY_LEDGER_DDL: &str = "\
CREATE TABLE IF NOT EXISTS arcaea_legacy_import (
\tname        TEXT PRIMARY KEY,
\timported_at TIMESTAMPTZ NOT NULL DEFAULT now(),
\taccounts    INTEGER NOT NULL DEFAULT 0,
\tdevices     INTEGER NOT NULL DEFAULT 0,
\tprefs       INTEGER NOT NULL DEFAULT 0,
\tskins       INTEGER NOT NULL DEFAULT 0,
\tthemes      INTEGER NOT NULL DEFAULT 0,
\tsnapshots   INTEGER NOT NULL DEFAULT 0,
\tgate        INTEGER NOT NULL DEFAULT 0,
\towners      INTEGER NOT NULL DEFAULT 0,
\tskipped     INTEGER NOT NULL DEFAULT 0
)";

const TICKET_DDL: &str = "\
CREATE TABLE IF NOT EXISTS arcaea_ticket (
\tregion     TEXT NOT NULL,
\tuser_id    TEXT NOT NULL,
\texpires_at TIMESTAMPTZ NOT NULL,
\tPRIMARY KEY (region, user_id),
\tCONSTRAINT arcaea_ticket_region_check CHECK (region IN ('intl', 'cn'))
)";

pub const MIGRATIONS: [MigrationStep; 10] = [
    MigrationStep {
        version: 1,
        name: "create_arcaea_account",
        sql: ACCOUNT_DDL,
    },
    MigrationStep {
        version: 2,
        name: "create_arcaea_account_uid_owner",
        sql: UID_OWNER_DDL,
    },
    MigrationStep {
        version: 3,
        name: "create_arcaea_account_user_index",
        sql: USER_INDEX_DDL,
    },
    MigrationStep {
        version: 4,
        name: "create_arcaea_cn_device",
        sql: DEVICE_DDL,
    },
    MigrationStep {
        version: 5,
        name: "create_arcaea_setting",
        sql: SETTING_DDL,
    },
    MigrationStep {
        version: 6,
        name: "create_arcaea_pref",
        sql: PREF_DDL,
    },
    MigrationStep {
        version: 7,
        name: "create_arcaea_skin",
        sql: SKIN_DDL,
    },
    MigrationStep {
        version: 8,
        name: "create_arcaea_snapshot",
        sql: SNAPSHOT_DDL,
    },
    MigrationStep {
        version: 9,
        name: "create_arcaea_legacy_import",
        sql: LEGACY_LEDGER_DDL,
    },
    MigrationStep {
        version: 10,
        name: "create_arcaea_ticket",
        sql: TICKET_DDL,
    },
];

pub const ACCOUNT_SELECT: &str = "\
SELECT region, user_id, arcaea_uid, friend_code, name, bound_at
\tFROM arcaea_account WHERE region = $1 AND user_id = $2";

pub const ACCOUNT_UPSERT: &str = "\
INSERT INTO arcaea_account (region, user_id, arcaea_uid, friend_code, name, bound_at, updated_at)
\tVALUES ($1, $2, $3, $4, $5, $6, now())
\tON CONFLICT (region, user_id) DO UPDATE SET
\t\tarcaea_uid = excluded.arcaea_uid,
\t\tfriend_code = excluded.friend_code,
\t\tname = excluded.name,
\t\tbound_at = excluded.bound_at,
\t\tupdated_at = now()";

pub const ACCOUNT_DELETE: &str = "DELETE FROM arcaea_account WHERE region = $1 AND user_id = $2";

pub const ACCOUNT_RESOLVE: &str = "\
UPDATE arcaea_account SET
\t\tarcaea_uid = $3,
\t\tname = CASE WHEN $4 = '' THEN name ELSE $4 END,
\t\tupdated_at = now()
\tWHERE region = $1 AND user_id = $2 AND arcaea_uid IS NULL
\tRETURNING region, user_id, arcaea_uid, friend_code, name, bound_at";

pub const DEVICE_TAKE: &str = "\
INSERT INTO arcaea_cn_device (user_id, device_id)
\tVALUES ($1, substring(md5(random()::text || clock_timestamp()::text || $1), 1, 16))
\tON CONFLICT (user_id) DO UPDATE SET
\t\tdevice_id = COALESCE(arcaea_cn_device.device_id, excluded.device_id)
\tRETURNING device_id";

pub const DEVICE_UNBOUND_MARK: &str = "\
INSERT INTO arcaea_cn_device (user_id, last_unbound_at) VALUES ($1, now())
\tON CONFLICT (user_id) DO UPDATE SET last_unbound_at = now(), updated_at = now()";

pub const DEVICE_UNBOUND_SELECT: &str =
    "SELECT last_unbound_at FROM arcaea_cn_device WHERE user_id = $1";

pub const GATE_SELECT: &str = "SELECT value FROM arcaea_setting WHERE name = $1";

pub const GATE_UPSERT: &str = "\
INSERT INTO arcaea_setting (name, value, updated_at) VALUES ($1, $2, now())
\tON CONFLICT (name) DO UPDATE SET value = excluded.value, updated_at = now()";

pub const PREF_SELECT: &str =
    "SELECT pref_char, pref_banner, theme FROM arcaea_pref WHERE region = $1 AND user_id = $2";

pub const PREF_UPSERT: &str = "\
INSERT INTO arcaea_pref (region, user_id, pref_char, pref_banner, theme, updated_at)
\tVALUES ($1, $2, $3, $4, $5, now())
\tON CONFLICT (region, user_id) DO UPDATE SET
\t\tpref_char = excluded.pref_char,
\t\tpref_banner = excluded.pref_banner,
\t\ttheme = excluded.theme,
\t\tupdated_at = now()";

pub const SKIN_SELECT: &str = "\
SELECT char_id, uncapped, override_form, banner, seen_at
\tFROM arcaea_skin WHERE region = $1 AND user_id = $2";

pub const SKIN_UPSERT: &str = "\
INSERT INTO arcaea_skin (region, user_id, char_id, uncapped, override_form, banner, seen_at, updated_at)
\tVALUES ($1, $2, $3, $4, $5, $6, $7, now())
\tON CONFLICT (region, user_id) DO UPDATE SET
\t\tchar_id = excluded.char_id,
\t\tuncapped = excluded.uncapped,
\t\toverride_form = excluded.override_form,
\t\tbanner = excluded.banner,
\t\tseen_at = excluded.seen_at,
\t\tupdated_at = now()";

pub const SNAPSHOT_UPSERT: &str = "\
INSERT INTO arcaea_snapshot (region, user_id, payload, arcaea_uid, friend_code, captured_at)
\tVALUES ($1, $2, $3, $4, $5, $6)
\tON CONFLICT (region, user_id) DO UPDATE SET
\t\tpayload = excluded.payload,
\t\tarcaea_uid = excluded.arcaea_uid,
\t\tfriend_code = excluded.friend_code,
\t\tcaptured_at = excluded.captured_at";

pub const SNAPSHOT_SELECT: &str = "\
SELECT region, user_id, payload, arcaea_uid, friend_code, captured_at
\tFROM arcaea_snapshot WHERE region = $1 AND user_id = $2";

pub const TICKET_PUT: &str = "\
INSERT INTO arcaea_ticket (region, user_id, expires_at) VALUES ($1, $2, $3)
\tON CONFLICT (region, user_id) DO UPDATE SET expires_at = excluded.expires_at";

pub const TICKET_TAKE: &str = "\
DELETE FROM arcaea_ticket WHERE region = $1 AND user_id = $2 RETURNING expires_at";

pub const TICKET_CLEAR: &str = "DELETE FROM arcaea_ticket WHERE region = $1 AND user_id = $2";

pub const ACCOUNT_LOCK: &str = "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))";

pub type AccountRow = (String, String, Option<i64>, String, String, DateTime<Utc>);

pub type SkinRow = (i32, bool, bool, String, Option<DateTime<Utc>>);

pub type SnapshotRow = (String, String, Value, Option<i64>, String, DateTime<Utc>);

pub type RepoResult<T> = Result<T, RepoError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoErrorKind {
    NotBound,
    UidTaken,
    PendingMissing,
    Import,
    Database,
    Invalid,
}

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("这个账号还没有绑定 Arcaea 账号")]
    NotBound,
    #[error("这个账号已经绑在别人名下了")]
    UidTaken,
    #[error("找不到待解析的绑定记录")]
    PendingMissing,
    #[error("导入老数据失败: {0}")]
    Import(String),
    #[error("数据库错误: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("数据库错误: {0}")]
    Db(#[from] DbError),
    #[error("参数不合法: {0}")]
    Invalid(String),
}

impl RepoError {
    pub fn kind(&self) -> RepoErrorKind {
        match self {
            RepoError::NotBound => RepoErrorKind::NotBound,
            RepoError::UidTaken => RepoErrorKind::UidTaken,
            RepoError::PendingMissing => RepoErrorKind::PendingMissing,
            RepoError::Import(_) => RepoErrorKind::Import,
            RepoError::Sqlx(_) | RepoError::Db(_) => RepoErrorKind::Database,
            RepoError::Invalid(_) => RepoErrorKind::Invalid,
        }
    }

    pub fn is(&self, kind: RepoErrorKind) -> bool {
        self.kind() == kind
    }

    pub fn is_not_bound(&self) -> bool {
        self.is(RepoErrorKind::NotBound)
    }

    pub fn is_uid_taken(&self) -> bool {
        self.is(RepoErrorKind::UidTaken)
    }
}

pub mod errors {
    use super::{RepoError, RepoErrorKind};

    pub const UID_TAKEN_MESSAGE: &str = "这个账号已经绑在别人名下了";

    pub const NOT_BOUND_MESSAGE: &str = "这个账号还没有绑定 Arcaea 账号";

    pub fn is(error: &RepoError, kind: RepoErrorKind) -> bool {
        error.kind() == kind
    }

    pub fn is_uid_taken(error: &RepoError) -> bool {
        is(error, RepoErrorKind::UidTaken)
    }

    pub fn is_not_bound(error: &RepoError) -> bool {
        is(error, RepoErrorKind::NotBound)
    }

    pub fn is_pending_missing(error: &RepoError) -> bool {
        is(error, RepoErrorKind::PendingMissing)
    }

    pub fn is_import(error: &RepoError) -> bool {
        is(error, RepoErrorKind::Import)
    }

    pub fn is_database(error: &RepoError) -> bool {
        is(error, RepoErrorKind::Database)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SnapshotRecord {
    pub region: Region,
    pub user_id: String,
    pub payload: Value,
    pub arcaea_uid: Option<i64>,
    pub friend_code: String,
    pub captured_at: DateTime<Utc>,
}

fn uid_violation(error: &sqlx::Error) -> bool {
    match error {
        sqlx::Error::Database(database) => {
            database.is_unique_violation() && database.constraint() == Some(UID_OWNER_INDEX)
        }
        _ => false,
    }
}

fn classify(error: sqlx::Error) -> RepoError {
    if uid_violation(&error) {
        RepoError::UidTaken
    } else {
        RepoError::Sqlx(error)
    }
}

fn ensure_user_id(user_id: &str) -> RepoResult<&str> {
    let trimmed = user_id.trim();
    if trimmed.is_empty() {
        return Err(RepoError::Invalid("用户 id 不能为空".to_string()));
    }
    Ok(trimmed)
}

fn account_of(row: (String, String, Option<i64>, String, String, DateTime<Utc>)) -> Account {
    let (region, user_id, arcaea_uid, friend_code, name, bound_at) = row;
    Account {
        region: Region::normalize(&region),
        user_id,
        arcaea_uid,
        friend_code,
        name,
        bound_at,
    }
}

pub struct ArcaeaRepo {
    db: Arc<PluginDb>,
}

impl ArcaeaRepo {
    pub fn from_db(db: Arc<PluginDb>) -> Self {
        Self { db }
    }

    pub async fn open(db: Arc<PluginDb>) -> RepoResult<MigrationReport> {
        let report = db.migrate(&MIGRATIONS).await?;
        Ok(report)
    }

    pub fn plugin_db(&self) -> &Arc<PluginDb> {
        &self.db
    }

    pub fn pool(&self) -> &PgPool {
        self.db.kv().pool()
    }

    pub async fn get_account(&self, region: Region, user_id: &str) -> RepoResult<Option<Account>> {
        let user_id = ensure_user_id(user_id)?;
        if let Some(account) = self.read_account(region, user_id).await? {
            return Ok(Some(account));
        }
        self.import_legacy_once().await?;
        self.read_account(region, user_id).await
    }

    pub async fn require_account(&self, region: Region, user_id: &str) -> RepoResult<Account> {
        self.get_account(region, user_id)
            .await?
            .ok_or(RepoError::NotBound)
    }

    pub async fn put_account(&self, account: &Account) -> RepoResult<()> {
        let user_id = ensure_user_id(&account.user_id)?;
        let _permit = self.db.acquire().await?;
        sqlx::query(ACCOUNT_UPSERT)
            .bind(account.region.as_str())
            .bind(user_id)
            .bind(account.uid())
            .bind(&account.friend_code)
            .bind(&account.name)
            .bind(account.bound_at)
            .execute(self.pool())
            .await
            .map_err(classify)?;
        Ok(())
    }

    pub async fn delete_account(&self, region: Region, user_id: &str) -> RepoResult<bool> {
        let user_id = ensure_user_id(user_id)?;
        let _permit = self.db.acquire().await?;
        let mut tx = self.pool().begin().await?;
        let lock = format!("arcaea:account:{}:{user_id}", region.as_str());
        sqlx::query(ACCOUNT_LOCK)
            .bind(lock)
            .execute(&mut *tx)
            .await?;
        let deleted = sqlx::query(ACCOUNT_DELETE)
            .bind(region.as_str())
            .bind(user_id)
            .execute(&mut *tx)
            .await?
            .rows_affected();
        tx.commit().await?;
        Ok(deleted > 0)
    }

    pub async fn resolve_pending(
        &self,
        region: Region,
        user_id: &str,
        uid: i64,
        name: &str,
    ) -> RepoResult<Account> {
        let user_id = ensure_user_id(user_id)?;
        if uid <= 0 {
            return Err(RepoError::Invalid("Arcaea 账号必须是正整数".to_string()));
        }
        let _permit = self.db.acquire().await?;
        let resolved: Option<AccountRow> = sqlx::query_as(ACCOUNT_RESOLVE)
            .bind(region.as_str())
            .bind(user_id)
            .bind(uid)
            .bind(name)
            .fetch_optional(self.pool())
            .await
            .map_err(classify)?;
        if let Some(row) = resolved {
            return Ok(account_of(row));
        }
        match self.read_account(region, user_id).await? {
            Some(account) if account.uid() == Some(uid) => Ok(account),
            _ => Err(RepoError::PendingMissing),
        }
    }

    pub async fn take_device_id(&self, user_id: &str) -> RepoResult<String> {
        let user_id = ensure_user_id(user_id)?;
        let _permit = self.db.acquire().await?;
        let device: Option<String> = sqlx::query_scalar(DEVICE_TAKE)
            .bind(user_id)
            .fetch_one(self.pool())
            .await?;
        device.ok_or_else(|| RepoError::Invalid("设备码生成失败".to_string()))
    }

    pub async fn mark_unbound(&self, user_id: &str) -> RepoResult<()> {
        let user_id = ensure_user_id(user_id)?;
        let _permit = self.db.acquire().await?;
        sqlx::query(DEVICE_UNBOUND_MARK)
            .bind(user_id)
            .execute(self.pool())
            .await?;
        Ok(())
    }

    pub async fn unbound_at(&self, user_id: &str) -> RepoResult<Option<DateTime<Utc>>> {
        let user_id = ensure_user_id(user_id)?;
        let _permit = self.db.acquire().await?;
        let found: Option<Option<DateTime<Utc>>> = sqlx::query_scalar(DEVICE_UNBOUND_SELECT)
            .bind(user_id)
            .fetch_optional(self.pool())
            .await?;
        Ok(found.flatten())
    }

    pub async fn bind_gate(&self) -> RepoResult<bool> {
        let _permit = self.db.acquire().await?;
        let value: Option<bool> = sqlx::query_scalar(GATE_SELECT)
            .bind(BIND_GATE_KEY)
            .fetch_optional(self.pool())
            .await?;
        Ok(value.unwrap_or(true))
    }

    pub async fn set_bind_gate(&self, open: bool) -> RepoResult<()> {
        let _permit = self.db.acquire().await?;
        sqlx::query(GATE_UPSERT)
            .bind(BIND_GATE_KEY)
            .bind(open)
            .execute(self.pool())
            .await?;
        Ok(())
    }

    pub async fn prefs(&self, region: Region, user_id: &str) -> RepoResult<Prefs> {
        let user_id = ensure_user_id(user_id)?;
        let _permit = self.db.acquire().await?;
        let row: Option<(String, String, String)> = sqlx::query_as(PREF_SELECT)
            .bind(region.as_str())
            .bind(user_id)
            .fetch_optional(self.pool())
            .await?;
        Ok(row
            .map(|(char, banner, theme)| Prefs {
                char,
                banner,
                theme,
            })
            .unwrap_or_default())
    }

    pub async fn set_prefs(&self, region: Region, user_id: &str, prefs: &Prefs) -> RepoResult<()> {
        let user_id = ensure_user_id(user_id)?;
        let _permit = self.db.acquire().await?;
        sqlx::query(PREF_UPSERT)
            .bind(region.as_str())
            .bind(user_id)
            .bind(&prefs.char)
            .bind(&prefs.banner)
            .bind(&prefs.theme)
            .execute(self.pool())
            .await?;
        Ok(())
    }

    pub async fn skin(&self, region: Region, user_id: &str) -> RepoResult<Skin> {
        let user_id = ensure_user_id(user_id)?;
        let _permit = self.db.acquire().await?;
        let row: Option<SkinRow> = sqlx::query_as(SKIN_SELECT)
            .bind(region.as_str())
            .bind(user_id)
            .fetch_optional(self.pool())
            .await?;
        Ok(row
            .map(|(char_id, uncapped, override_form, banner, seen_at)| Skin {
                char_id,
                uncapped,
                override_form,
                banner,
                seen_at,
            })
            .unwrap_or_else(Skin::unknown))
    }

    pub async fn remember_skin(
        &self,
        region: Region,
        user_id: &str,
        skin: &Skin,
    ) -> RepoResult<()> {
        let user_id = ensure_user_id(user_id)?;
        let _permit = self.db.acquire().await?;
        sqlx::query(SKIN_UPSERT)
            .bind(region.as_str())
            .bind(user_id)
            .bind(skin.char_id)
            .bind(skin.uncapped)
            .bind(skin.override_form)
            .bind(&skin.banner)
            .bind(skin.seen_at)
            .execute(self.pool())
            .await?;
        Ok(())
    }

    pub async fn save_snapshot(&self, snapshot: &SnapshotRecord) -> RepoResult<()> {
        let user_id = ensure_user_id(&snapshot.user_id)?;
        let _permit = self.db.acquire().await?;
        sqlx::query(SNAPSHOT_UPSERT)
            .bind(snapshot.region.as_str())
            .bind(user_id)
            .bind(&snapshot.payload)
            .bind(snapshot.arcaea_uid)
            .bind(&snapshot.friend_code)
            .bind(snapshot.captured_at)
            .execute(self.pool())
            .await?;
        Ok(())
    }

    pub async fn load_snapshot(
        &self,
        region: Region,
        user_id: &str,
    ) -> RepoResult<Option<SnapshotRecord>> {
        let user_id = ensure_user_id(user_id)?;
        let _permit = self.db.acquire().await?;
        let row: Option<SnapshotRow> = sqlx::query_as(SNAPSHOT_SELECT)
            .bind(region.as_str())
            .bind(user_id)
            .fetch_optional(self.pool())
            .await?;
        Ok(row.map(
            |(region, user_id, payload, arcaea_uid, friend_code, captured_at)| SnapshotRecord {
                region: Region::normalize(&region),
                user_id,
                payload,
                arcaea_uid,
                friend_code,
                captured_at,
            },
        ))
    }

    pub async fn put_ticket(
        &self,
        region: Region,
        user_id: &str,
        expires_at: DateTime<Utc>,
    ) -> RepoResult<()> {
        let user_id = ensure_user_id(user_id)?;
        let _permit = self.db.acquire().await?;
        sqlx::query(TICKET_PUT)
            .bind(region.as_str())
            .bind(user_id)
            .bind(expires_at)
            .execute(self.pool())
            .await?;
        Ok(())
    }

    pub async fn take_ticket(&self, region: Region, user_id: &str) -> RepoResult<bool> {
        let user_id = ensure_user_id(user_id)?;
        let _permit = self.db.acquire().await?;
        let expires: Option<DateTime<Utc>> = sqlx::query_scalar(TICKET_TAKE)
            .bind(region.as_str())
            .bind(user_id)
            .fetch_optional(self.pool())
            .await?;
        Ok(expires.is_some_and(|at| at > Utc::now()))
    }

    pub async fn clear_ticket(&self, region: Region, user_id: &str) -> RepoResult<bool> {
        let user_id = ensure_user_id(user_id)?;
        let _permit = self.db.acquire().await?;
        let cleared = sqlx::query(TICKET_CLEAR)
            .bind(region.as_str())
            .bind(user_id)
            .execute(self.pool())
            .await?
            .rows_affected();
        Ok(cleared > 0)
    }
    pub async fn import_legacy_once(&self) -> RepoResult<legacy::LegacyImportReport> {
        legacy::import_once(&self.db)
            .await
            .map_err(|err| RepoError::Import(err.to_string()))
    }

    async fn read_account(&self, region: Region, user_id: &str) -> RepoResult<Option<Account>> {
        let _permit = self.db.acquire().await?;
        let row: Option<AccountRow> = sqlx::query_as(ACCOUNT_SELECT)
            .bind(region.as_str())
            .bind(user_id)
            .fetch_optional(self.pool())
            .await?;
        Ok(row.map(account_of))
    }
}
