use aurorix_db::{DbResult, PluginDb};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;
use sqlx::PgConnection;

use crate::model::{Prefs, Region, Skin};

pub const LEGACY_SCOPE: &str = "plugin";

pub const LEGACY_NAMESPACE: &str = "arcaea";

pub const LEGACY_KV_TABLE: &str = "kv_data";

pub const LEGACY_OWNER_TABLE: &str = "arcaea_binding_owner";

pub const LEGACY_LEDGER_NAME: &str = "legacy_v1";

pub const LEGACY_LOCK_KEY: &str = "arcaea:legacy_import";

const LEDGER_LOCK: &str = "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))";

const LEDGER_PRESENT: &str = "SELECT EXISTS (SELECT 1 FROM arcaea_legacy_import WHERE name = $1)";

const LEDGER_INSERT: &str = "\
INSERT INTO arcaea_legacy_import
\t\t(name, accounts, devices, prefs, skins, themes, snapshots, gate, owners, skipped)
\tVALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
\tON CONFLICT (name) DO NOTHING";

const TABLE_PRESENT: &str = "SELECT to_regclass($1) IS NOT NULL";

const LEGACY_ROWS: &str = "\
SELECT key, value FROM kv_data WHERE scope = $1 AND namespace = $2 ORDER BY key";

const LEGACY_OWNERS: &str = "\
SELECT region, arcaea_uid, user_id, bound_at FROM arcaea_binding_owner ORDER BY region, arcaea_uid";

const ACCOUNT_INSERT: &str = "\
INSERT INTO arcaea_account (region, user_id, arcaea_uid, friend_code, name, bound_at, updated_at)
\tVALUES ($1, $2, $3, $4, $5, $6, now())
\tON CONFLICT DO NOTHING";

const OWNER_FILL: &str = "\
UPDATE arcaea_account SET arcaea_uid = $3, updated_at = now()
\tWHERE region = $1 AND user_id = $2 AND arcaea_uid IS NULL
\t\tAND NOT EXISTS (
\t\t\tSELECT 1 FROM arcaea_account occupied
\t\t\tWHERE occupied.region = $1 AND occupied.arcaea_uid = $3 AND occupied.user_id <> $2
\t\t)";

const DEVICE_INSERT: &str = "\
INSERT INTO arcaea_cn_device (user_id, device_id) VALUES ($1, $2)
\tON CONFLICT (user_id) DO UPDATE SET device_id = excluded.device_id, updated_at = now()
\tWHERE arcaea_cn_device.device_id IS NULL";

const UNBOUND_INSERT: &str = "\
INSERT INTO arcaea_cn_device (user_id, last_unbound_at) VALUES ($1, $2)
\tON CONFLICT (user_id) DO UPDATE SET last_unbound_at = excluded.last_unbound_at, updated_at = now()
\tWHERE arcaea_cn_device.last_unbound_at IS NULL";

const GATE_INSERT: &str = "\
INSERT INTO arcaea_setting (name, value, updated_at) VALUES ($1, $2, now())
\tON CONFLICT (name) DO NOTHING";

const SNAPSHOT_INSERT: &str = "\
INSERT INTO arcaea_snapshot (region, user_id, payload, arcaea_uid, friend_code, captured_at)
\tVALUES ($1, $2, $3, $4, $5, $6)
\tON CONFLICT (region, user_id) DO NOTHING";

const PREF_INSERT: &str = "\
INSERT INTO arcaea_pref (region, user_id, pref_char, pref_banner) VALUES ($1, $2, $3, $4)
\tON CONFLICT (region, user_id) DO NOTHING";

const THEME_FILL: &str = "\
INSERT INTO arcaea_pref (region, user_id, theme) VALUES ($1, $2, $3)
\tON CONFLICT (region, user_id) DO UPDATE SET theme = excluded.theme, updated_at = now()
\tWHERE arcaea_pref.theme = ''";

const SKIN_INSERT: &str = "\
INSERT INTO arcaea_skin (region, user_id, char_id, uncapped, override_form, banner, seen_at)
\tVALUES ($1, $2, $3, $4, $5, $6, $7)
\tON CONFLICT (region, user_id) DO NOTHING";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LegacyImportReport {
    pub accounts: usize,
    pub devices: usize,
    pub prefs: usize,
    pub skins: usize,
    pub themes: usize,
    pub snapshots: usize,
    pub gate: usize,
    pub owners: usize,
    pub skipped: usize,
    pub ignored: usize,
    pub already_imported: bool,
}

impl LegacyImportReport {
    pub fn total(&self) -> usize {
        self.accounts
            + self.devices
            + self.prefs
            + self.skins
            + self.themes
            + self.snapshots
            + self.gate
            + self.owners
    }

    pub fn is_empty(&self) -> bool {
        self.total() == 0 && self.skipped == 0 && self.ignored == 0
    }
}

#[derive(Debug, Clone, Copy)]
enum Bucket {
    Accounts,
    Devices,
    Prefs,
    Skins,
    Themes,
    Snapshots,
    Gate,
    Owners,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct GoBinding {
    #[serde(rename = "ArcaeaUID", default)]
    arcaea_uid: Option<i64>,
    #[serde(rename = "FriendCode", default)]
    friend_code: Option<Value>,
    #[serde(rename = "Name", default)]
    name: Option<Value>,
    #[serde(rename = "BoundAt", default)]
    bound_at: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct GoSnapshot {
    #[serde(rename = "At", default)]
    at: Option<Value>,
    #[serde(rename = "ArcaeaUID", default)]
    arcaea_uid: Option<i64>,
    #[serde(rename = "FriendCode", default)]
    friend_code: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct GoSkin {
    #[serde(rename = "ID", default)]
    id: Option<i32>,
    #[serde(rename = "Uncapped", default)]
    uncapped: Option<bool>,
    #[serde(rename = "Override", default)]
    override_form: Option<bool>,
    #[serde(rename = "Banner", default)]
    banner: Option<String>,
    #[serde(rename = "SeenAt", default)]
    seen_at: Option<Value>,
    #[serde(rename = "PrefChar", default)]
    pref_char: Option<String>,
    #[serde(rename = "PrefBanner", default)]
    pref_banner: Option<String>,
}

#[derive(Debug, Clone)]
enum LegacyKey<'a> {
    Account(Region, &'a str, bool),
    Device(&'a str),
    Unbound(&'a str),
    Gate,
    Snapshot(Region, &'a str),
    Skin(&'a str),
    Theme(&'a str),
    Unknown,
}

fn classify(key: &str) -> LegacyKey<'_> {
    if let Some(rest) = key.strip_prefix("acct:") {
        if let Some(user_id) = rest.strip_prefix("intl:") {
            return LegacyKey::Account(Region::Intl, user_id, true);
        }
        if let Some(user_id) = rest.strip_prefix("cn:") {
            return LegacyKey::Account(Region::Cn, user_id, true);
        }
        return LegacyKey::Account(Region::Intl, rest, false);
    }
    if let Some(user_id) = key.strip_prefix("cn:device:") {
        return LegacyKey::Device(user_id);
    }
    if let Some(user_id) = key.strip_prefix("cn:unbound:") {
        return LegacyKey::Unbound(user_id);
    }
    if key == "cn:bind_open" {
        return LegacyKey::Gate;
    }
    if let Some(rest) = key.strip_prefix("last:b50:") {
        if let Some(user_id) = rest.strip_prefix("intl:") {
            return LegacyKey::Snapshot(Region::Intl, user_id);
        }
        if let Some(user_id) = rest.strip_prefix("cn:") {
            return LegacyKey::Snapshot(Region::Cn, user_id);
        }
        return LegacyKey::Snapshot(Region::Intl, rest);
    }
    if let Some(user_id) = key.strip_prefix("skin:") {
        return LegacyKey::Skin(user_id);
    }
    if let Some(user_id) = key.strip_prefix("theme:") {
        return LegacyKey::Theme(user_id);
    }
    LegacyKey::Unknown
}

fn key_rank(key: &str) -> u8 {
    match classify(key) {
        LegacyKey::Account(_, _, qualified) => {
            if qualified {
                0
            } else {
                1
            }
        }
        LegacyKey::Unknown => 9,
        _ => 2,
    }
}

fn text_of(value: &Value) -> String {
    match value {
        Value::String(text) => text.trim().to_string(),
        Value::Number(number) => number.to_string(),
        Value::Bool(flag) => flag.to_string(),
        _ => String::new(),
    }
}

fn usable_time(value: &Value) -> Option<DateTime<Utc>> {
    let text = match value {
        Value::String(text) => text.trim(),
        _ => return None,
    };
    let parsed = DateTime::parse_from_rfc3339(text).ok()?;
    let utc = parsed.with_timezone(&Utc);
    if utc.timestamp_millis() <= 0 {
        return None;
    }
    Some(utc)
}

fn is_hex16(text: &str) -> bool {
    text.len() == 16
        && text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn tally(report: &mut LegacyImportReport, bucket: Bucket, affected: u64) {
    if affected == 0 {
        report.ignored += 1;
        return;
    }
    let amount = affected as usize;
    match bucket {
        Bucket::Accounts => report.accounts += amount,
        Bucket::Devices => report.devices += amount,
        Bucket::Prefs => report.prefs += amount,
        Bucket::Skins => report.skins += amount,
        Bucket::Themes => report.themes += amount,
        Bucket::Snapshots => report.snapshots += amount,
        Bucket::Gate => report.gate += amount,
        Bucket::Owners => report.owners += amount,
    }
}

async fn table_exists(connection: &mut PgConnection, table: &str) -> DbResult<bool> {
    let exists: bool = sqlx::query_scalar(TABLE_PRESENT)
        .bind(table)
        .fetch_one(connection)
        .await?;
    Ok(exists)
}

pub async fn import_once(db: &PluginDb) -> DbResult<LegacyImportReport> {
    let _permit = db.acquire().await?;
    let pool = db.kv().pool();
    let mut report = LegacyImportReport::default();
    let mut tx = pool.begin().await?;
    sqlx::query(LEDGER_LOCK)
        .bind(LEGACY_LOCK_KEY)
        .execute(&mut *tx)
        .await?;
    let imported: bool = sqlx::query_scalar(LEDGER_PRESENT)
        .bind(LEGACY_LEDGER_NAME)
        .fetch_one(&mut *tx)
        .await?;
    if imported {
        tx.rollback().await?;
        report.already_imported = true;
        return Ok(report);
    }
    if table_exists(&mut tx, LEGACY_KV_TABLE).await? {
        let rows: Vec<(String, Vec<u8>)> = sqlx::query_as(LEGACY_ROWS)
            .bind(LEGACY_SCOPE)
            .bind(LEGACY_NAMESPACE)
            .fetch_all(&mut *tx)
            .await?;
        apply_rows(&mut tx, rows, &mut report).await?;
    }
    if table_exists(&mut tx, LEGACY_OWNER_TABLE).await? {
        let owners: Vec<(String, i64, String, Option<DateTime<Utc>>)> =
            sqlx::query_as(LEGACY_OWNERS).fetch_all(&mut *tx).await?;
        apply_owners(&mut tx, owners, &mut report).await?;
    }
    sqlx::query(LEDGER_INSERT)
        .bind(LEGACY_LEDGER_NAME)
        .bind(report.accounts as i64)
        .bind(report.devices as i64)
        .bind(report.prefs as i64)
        .bind(report.skins as i64)
        .bind(report.themes as i64)
        .bind(report.snapshots as i64)
        .bind(report.gate as i64)
        .bind(report.owners as i64)
        .bind(report.skipped as i64)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(report)
}

async fn apply_rows(
    connection: &mut PgConnection,
    rows: Vec<(String, Vec<u8>)>,
    report: &mut LegacyImportReport,
) -> DbResult<()> {
    let mut accounts: Vec<(String, Vec<u8>)> = Vec::new();
    for (key, raw) in rows {
        match classify(&key) {
            LegacyKey::Account(_, user_id, _) => {
                if user_id.trim().is_empty() {
                    report.skipped += 1;
                    continue;
                }
                accounts.push((key, raw));
            }
            LegacyKey::Device(user_id) => {
                apply_device(connection, user_id, &raw, report).await?;
            }
            LegacyKey::Unbound(user_id) => {
                apply_unbound(connection, user_id, &raw, report).await?;
            }
            LegacyKey::Gate => {
                apply_gate(connection, &raw, report).await?;
            }
            LegacyKey::Snapshot(region, user_id) => {
                apply_snapshot(connection, region, user_id, &raw, report).await?;
            }
            LegacyKey::Skin(user_id) => {
                apply_skin(connection, user_id, &raw, report).await?;
            }
            LegacyKey::Theme(user_id) => {
                apply_theme(connection, user_id, &raw, report).await?;
            }
            LegacyKey::Unknown => {
                report.skipped += 1;
            }
        }
    }
    accounts.sort_by_key(|(key, _)| (key_rank(key), key.clone()));
    for (key, raw) in accounts {
        if let LegacyKey::Account(region, user_id, _) = classify(&key) {
            apply_account(connection, region, user_id, &raw, report).await?;
        }
    }
    Ok(())
}

async fn apply_account(
    connection: &mut PgConnection,
    region: Region,
    user_id: &str,
    raw: &[u8],
    report: &mut LegacyImportReport,
) -> DbResult<()> {
    let binding: GoBinding = match serde_json::from_slice(raw) {
        Ok(binding) => binding,
        Err(_) => {
            report.skipped += 1;
            return Ok(());
        }
    };
    let uid = binding.arcaea_uid.filter(|uid| *uid > 0);
    let friend_code = binding
        .friend_code
        .as_ref()
        .map(text_of)
        .unwrap_or_default();
    let name = binding.name.as_ref().map(text_of).unwrap_or_default();
    let bound_at = binding
        .bound_at
        .as_ref()
        .and_then(usable_time)
        .unwrap_or_else(Utc::now);
    let affected = sqlx::query(ACCOUNT_INSERT)
        .bind(region.as_str())
        .bind(user_id.trim())
        .bind(uid)
        .bind(friend_code)
        .bind(name)
        .bind(bound_at)
        .execute(&mut *connection)
        .await?
        .rows_affected();
    tally(report, Bucket::Accounts, affected);
    Ok(())
}

async fn apply_owners(
    connection: &mut PgConnection,
    owners: Vec<(String, i64, String, Option<DateTime<Utc>>)>,
    report: &mut LegacyImportReport,
) -> DbResult<()> {
    for (region, arcaea_uid, user_id, bound_at) in owners {
        let region_text = region.trim();
        let user_id = user_id.trim();
        if region_text.is_empty() || user_id.is_empty() || arcaea_uid <= 0 {
            report.skipped += 1;
            continue;
        }
        let region = Region::normalize(region_text);
        let bound_at = bound_at
            .filter(|moment| moment.timestamp_millis() > 0)
            .unwrap_or_else(Utc::now);
        let inserted = sqlx::query(ACCOUNT_INSERT)
            .bind(region.as_str())
            .bind(user_id)
            .bind(arcaea_uid)
            .bind("")
            .bind("")
            .bind(bound_at)
            .execute(&mut *connection)
            .await?
            .rows_affected();
        if inserted > 0 {
            tally(report, Bucket::Owners, inserted);
            continue;
        }
        let filled = sqlx::query(OWNER_FILL)
            .bind(region.as_str())
            .bind(user_id)
            .bind(arcaea_uid)
            .execute(&mut *connection)
            .await?
            .rows_affected();
        if filled > 0 {
            tally(report, Bucket::Owners, filled);
        } else {
            report.ignored += 1;
        }
    }
    Ok(())
}

async fn apply_device(
    connection: &mut PgConnection,
    user_id: &str,
    raw: &[u8],
    report: &mut LegacyImportReport,
) -> DbResult<()> {
    let user_id = user_id.trim();
    let value: Value = match serde_json::from_slice(raw) {
        Ok(value) => value,
        Err(_) => {
            report.skipped += 1;
            return Ok(());
        }
    };
    let device = text_of(&value);
    if user_id.is_empty() || !is_hex16(&device) {
        report.skipped += 1;
        return Ok(());
    }
    let affected = sqlx::query(DEVICE_INSERT)
        .bind(user_id)
        .bind(device)
        .execute(&mut *connection)
        .await?
        .rows_affected();
    tally(report, Bucket::Devices, affected);
    Ok(())
}

async fn apply_unbound(
    connection: &mut PgConnection,
    user_id: &str,
    raw: &[u8],
    report: &mut LegacyImportReport,
) -> DbResult<()> {
    let user_id = user_id.trim();
    let value: Value = match serde_json::from_slice(raw) {
        Ok(value) => value,
        Err(_) => {
            report.skipped += 1;
            return Ok(());
        }
    };
    let moment = match usable_time(&value) {
        Some(moment) if !user_id.is_empty() => moment,
        _ => {
            report.skipped += 1;
            return Ok(());
        }
    };
    let affected = sqlx::query(UNBOUND_INSERT)
        .bind(user_id)
        .bind(moment)
        .execute(&mut *connection)
        .await?
        .rows_affected();
    tally(report, Bucket::Devices, affected);
    Ok(())
}

async fn apply_gate(
    connection: &mut PgConnection,
    raw: &[u8],
    report: &mut LegacyImportReport,
) -> DbResult<()> {
    let value: Value = match serde_json::from_slice(raw) {
        Ok(value) => value,
        Err(_) => {
            report.skipped += 1;
            return Ok(());
        }
    };
    let open = match value {
        Value::Bool(flag) => flag,
        _ => {
            report.skipped += 1;
            return Ok(());
        }
    };
    let affected = sqlx::query(GATE_INSERT)
        .bind(super::BIND_GATE_KEY)
        .bind(open)
        .execute(&mut *connection)
        .await?
        .rows_affected();
    tally(report, Bucket::Gate, affected);
    Ok(())
}

async fn apply_snapshot(
    connection: &mut PgConnection,
    region: Region,
    user_id: &str,
    raw: &[u8],
    report: &mut LegacyImportReport,
) -> DbResult<()> {
    let user_id = user_id.trim();
    let payload: Value = match serde_json::from_slice(raw) {
        Ok(payload) => payload,
        Err(_) => {
            report.skipped += 1;
            return Ok(());
        }
    };
    if user_id.is_empty() || !payload.is_object() {
        report.skipped += 1;
        return Ok(());
    }
    let meta: GoSnapshot = serde_json::from_value(payload.clone()).unwrap_or_default();
    let uid = meta.arcaea_uid.filter(|uid| *uid > 0);
    let friend_code = meta.friend_code.as_ref().map(text_of).unwrap_or_default();
    let captured_at = meta
        .at
        .as_ref()
        .and_then(usable_time)
        .unwrap_or_else(Utc::now);
    let affected = sqlx::query(SNAPSHOT_INSERT)
        .bind(region.as_str())
        .bind(user_id)
        .bind(payload)
        .bind(uid)
        .bind(friend_code)
        .bind(captured_at)
        .execute(&mut *connection)
        .await?
        .rows_affected();
    tally(report, Bucket::Snapshots, affected);
    Ok(())
}

async fn apply_skin(
    connection: &mut PgConnection,
    user_id: &str,
    raw: &[u8],
    report: &mut LegacyImportReport,
) -> DbResult<()> {
    let user_id = user_id.trim();
    let skin: GoSkin = match serde_json::from_slice(raw) {
        Ok(skin) => skin,
        Err(_) => {
            report.skipped += 1;
            return Ok(());
        }
    };
    if user_id.is_empty() {
        report.skipped += 1;
        return Ok(());
    }
    let region = Region::Intl;
    let char_id = skin.id.unwrap_or(Skin::UNKNOWN_CHAR);
    let observed = if char_id < 0 {
        Skin::unknown()
    } else {
        Skin {
            char_id,
            uncapped: skin.uncapped.unwrap_or(false),
            override_form: skin.override_form.unwrap_or(false),
            banner: skin.banner.unwrap_or_default(),
            seen_at: skin.seen_at.as_ref().and_then(usable_time),
        }
    };
    let affected = sqlx::query(SKIN_INSERT)
        .bind(region.as_str())
        .bind(user_id)
        .bind(observed.char_id)
        .bind(observed.uncapped)
        .bind(observed.override_form)
        .bind(&observed.banner)
        .bind(observed.seen_at)
        .execute(&mut *connection)
        .await?
        .rows_affected();
    tally(report, Bucket::Skins, affected);
    let prefs = Prefs {
        char: skin.pref_char.unwrap_or_default(),
        banner: skin.pref_banner.unwrap_or_default(),
        theme: String::new(),
    };
    if prefs.is_empty() {
        return Ok(());
    }
    let affected = sqlx::query(PREF_INSERT)
        .bind(region.as_str())
        .bind(user_id)
        .bind(&prefs.char)
        .bind(&prefs.banner)
        .execute(&mut *connection)
        .await?
        .rows_affected();
    tally(report, Bucket::Prefs, affected);
    Ok(())
}

async fn apply_theme(
    connection: &mut PgConnection,
    user_id: &str,
    raw: &[u8],
    report: &mut LegacyImportReport,
) -> DbResult<()> {
    let user_id = user_id.trim();
    let value: Value = match serde_json::from_slice(raw) {
        Ok(value) => value,
        Err(_) => {
            report.skipped += 1;
            return Ok(());
        }
    };
    let theme = match &value {
        Value::String(text) => text.trim().to_string(),
        Value::Object(map) => map.get("name").map(text_of).unwrap_or_default(),
        _ => String::new(),
    };
    if user_id.is_empty() || theme.is_empty() {
        report.skipped += 1;
        return Ok(());
    }
    let affected = sqlx::query(THEME_FILL)
        .bind(Region::Intl.as_str())
        .bind(user_id)
        .bind(theme)
        .execute(&mut *connection)
        .await?
        .rows_affected();
    tally(report, Bucket::Themes, affected);
    Ok(())
}
