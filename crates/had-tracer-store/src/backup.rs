use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteRow};
use sqlx::{Column, QueryBuilder, Row, Sqlite};

use crate::Store;
use crate::error::{Result, StoreError};
use crate::schema::{EXPORT_TABLES, SCHEMA_VERSION};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupResult {
    pub path: String,
    pub manifest_path: String,
    pub checksum: String,
    pub bytes: u64,
    pub created_at: String,
    pub integrity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub path: String,
    pub checksum: String,
    pub bytes: u64,
    pub tables: usize,
    pub rows: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportOptions {
    pub mode: String,
    pub dry_run: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            mode: "merge".to_string(),
            dry_run: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub tables: usize,
    pub rows: usize,
    pub mode: String,
    pub dry_run: bool,
    pub checksum: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupManifest {
    format: String,
    version: u32,
    schema_version: i64,
    app_version: String,
    created_at: String,
    checksum: String,
    bytes: u64,
    counts: BTreeMap<String, i64>,
}

impl Store {
    /// Run SQLite integrity check.
    pub async fn integrity_check(&self) -> Result<String> {
        let value: String = sqlx::query_scalar("PRAGMA integrity_check")
            .fetch_one(&self.pool)
            .await?;
        Ok(value)
    }

    /// Create a consistent SQLite backup and a checksum manifest.
    pub async fn backup_to(&self, path: &Path) -> Result<BackupResult> {
        let integrity = self.integrity_check().await?;
        if integrity != "ok" {
            return Err(StoreError::Invalid(format!(
                "integrity check failed before backup: {integrity}"
            )));
        }
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        if tokio::fs::try_exists(path).await? {
            tokio::fs::remove_file(path).await?;
        }
        let escaped = path.to_string_lossy().replace('\'', "''");
        let sql = format!("VACUUM INTO '{escaped}'");
        sqlx::query(sqlx::AssertSqlSafe(sql))
            .execute(&self.pool)
            .await?;

        let integrity = verify_sqlite_file(path).await?;
        let bytes = tokio::fs::metadata(path).await?.len();
        let checksum = file_checksum(path).await?;
        let counts = self
            .table_counts()
            .await?
            .into_iter()
            .collect::<BTreeMap<_, _>>();
        let created_at = had_tracer_core::now_rfc3339();
        let manifest = BackupManifest {
            format: "had-tracer-backup".to_string(),
            version: 1,
            schema_version: SCHEMA_VERSION,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: created_at.clone(),
            checksum: checksum.clone(),
            bytes,
            counts,
        };
        let manifest_path = PathBuf::from(format!("{}.manifest.json", path.display()));
        tokio::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?).await?;

        self.audit(
            "system",
            "backup",
            "database",
            None,
            serde_json::json!({
                "path": path.to_string_lossy(),
                "checksum": checksum,
                "bytes": bytes,
            }),
        )
        .await?;

        Ok(BackupResult {
            path: path.to_string_lossy().to_string(),
            manifest_path: manifest_path.to_string_lossy().to_string(),
            checksum,
            bytes,
            created_at,
            integrity,
        })
    }

    /// Export all app-owned tables to a versioned JSON document.
    pub async fn export_json(&self, path: &Path) -> Result<ExportResult> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let mut tables = Map::new();
        let mut counts = Map::new();
        let mut total_rows = 0usize;
        for table in EXPORT_TABLES {
            let sql = format!("SELECT * FROM {table}");
            let rows = sqlx::query(sqlx::AssertSqlSafe(sql))
                .fetch_all(&self.pool)
                .await?;
            let count = rows.len();
            total_rows += count;
            counts.insert((*table).to_string(), Value::from(count as u64));
            let values = rows.iter().map(row_to_json).collect::<Vec<_>>();
            tables.insert((*table).to_string(), Value::Array(values));
        }
        let payload = serde_json::json!({
            "format": "had-tracer-export",
            "version": 1,
            "schemaVersion": SCHEMA_VERSION,
            "appVersion": env!("CARGO_PKG_VERSION"),
            "exportedAt": had_tracer_core::now_rfc3339(),
            "counts": counts,
            "tables": tables,
        });
        let text = serde_json::to_string_pretty(&payload)?;
        tokio::fs::write(path, text.as_bytes()).await?;
        let bytes = text.len() as u64;
        let checksum = sha256_hex(text.as_bytes());
        self.audit(
            "system",
            "export",
            "database",
            None,
            serde_json::json!({
                "path": path.to_string_lossy(),
                "checksum": checksum,
                "rows": total_rows,
            }),
        )
        .await?;
        Ok(ExportResult {
            path: path.to_string_lossy().to_string(),
            checksum,
            bytes,
            tables: EXPORT_TABLES.len(),
            rows: total_rows,
        })
    }

    /// Import a JSON export in merge or replace mode.
    pub async fn import_json(&self, path: &Path, options: &ImportOptions) -> Result<ImportResult> {
        let text = tokio::fs::read_to_string(path).await?;
        let payload: Value = serde_json::from_str(&text)?;
        validate_export(&payload)?;
        let checksum = sha256_hex(text.as_bytes());
        let tables = payload
            .get("tables")
            .and_then(Value::as_object)
            .ok_or_else(|| StoreError::Invalid("export has no tables object".to_string()))?;

        let mut total_rows = 0usize;
        let mut table_count = 0usize;
        for table in EXPORT_TABLES {
            if let Some(rows) = tables.get(*table).and_then(Value::as_array) {
                table_count += 1;
                total_rows += rows.len();
            }
        }

        if options.dry_run {
            return Ok(ImportResult {
                tables: table_count,
                rows: total_rows,
                mode: options.mode.clone(),
                dry_run: true,
                checksum,
            });
        }

        let replace = options.mode == "replace";
        if options.mode != "merge" && !replace {
            return Err(StoreError::Invalid(format!(
                "unknown import mode: {}",
                options.mode
            )));
        }

        let mut tx = self.pool.begin().await?;
        if replace {
            for table in EXPORT_TABLES.iter().rev() {
                let sql = format!("DELETE FROM {table}");
                sqlx::query(sqlx::AssertSqlSafe(sql))
                    .execute(&mut *tx)
                    .await?;
            }
        }
        for table in EXPORT_TABLES {
            let Some(rows) = tables.get(*table).and_then(Value::as_array) else {
                continue;
            };
            for row in rows {
                let Some(object) = row.as_object() else {
                    continue;
                };
                insert_json_row(&mut tx, table, object).await?;
            }
        }
        tx.commit().await?;

        self.audit(
            "system",
            "import",
            "database",
            None,
            serde_json::json!({
                "path": path.to_string_lossy(),
                "mode": options.mode,
                "rows": total_rows,
                "checksum": checksum,
            }),
        )
        .await?;

        Ok(ImportResult {
            tables: table_count,
            rows: total_rows,
            mode: options.mode.clone(),
            dry_run: false,
            checksum,
        })
    }
}

/// Verify a SQLite file without modifying it.
pub async fn verify_sqlite_file(path: &Path) -> Result<String> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .read_only(true)
        .busy_timeout(std::time::Duration::from_secs(5));
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;
    let value: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(&pool)
        .await?;
    pool.close().await;
    if value == "ok" {
        Ok(value)
    } else {
        Err(StoreError::Invalid(format!(
            "backup integrity check failed: {value}"
        )))
    }
}

/// SHA-256 of a file.
pub async fn file_checksum(path: &Path) -> Result<String> {
    let bytes = tokio::fs::read(path).await?;
    Ok(sha256_hex(&bytes))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    hex::encode(digest)
}

fn row_to_json(row: &SqliteRow) -> Value {
    let mut object = Map::new();
    for (index, column) in row.columns().iter().enumerate() {
        let name = column.name().to_string();
        let value = if let Ok(value) = row.try_get::<Option<i64>, _>(index) {
            value.map(Value::from).unwrap_or(Value::Null)
        } else if let Ok(value) = row.try_get::<Option<f64>, _>(index) {
            value.map(Value::from).unwrap_or(Value::Null)
        } else if let Ok(value) = row.try_get::<Option<String>, _>(index) {
            value.map(Value::from).unwrap_or(Value::Null)
        } else if let Ok(value) = row.try_get::<Option<Vec<u8>>, _>(index) {
            value
                .map(|bytes| Value::String(format!("0x{}", hex::encode(bytes))))
                .unwrap_or(Value::Null)
        } else {
            Value::Null
        };
        object.insert(name, value);
    }
    Value::Object(object)
}

fn validate_export(payload: &Value) -> Result<()> {
    let format = payload
        .get("format")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if format != "had-tracer-export" {
        return Err(StoreError::Invalid(
            "not a HAD Tracer export document".to_string(),
        ));
    }
    let version = payload
        .get("version")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    if version != 1 {
        return Err(StoreError::Invalid(format!(
            "unsupported export version: {version}"
        )));
    }
    let schema_version = payload
        .get("schemaVersion")
        .and_then(Value::as_i64)
        .unwrap_or_default();
    if schema_version > SCHEMA_VERSION {
        return Err(StoreError::Invalid(format!(
            "export schema {schema_version} is newer than app schema {SCHEMA_VERSION}"
        )));
    }
    Ok(())
}

async fn insert_json_row(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    table: &str,
    object: &Map<String, Value>,
) -> Result<()> {
    let columns = object
        .keys()
        .filter(|column| is_safe_identifier(column))
        .cloned()
        .collect::<Vec<_>>();
    if columns.is_empty() {
        return Ok(());
    }
    let mut builder = QueryBuilder::<Sqlite>::new("INSERT OR REPLACE INTO ");
    builder.push(table);
    builder.push(" (");
    for (index, column) in columns.iter().enumerate() {
        if index > 0 {
            builder.push(", ");
        }
        builder.push(column.as_str());
    }
    builder.push(") VALUES (");
    for (index, column) in columns.iter().enumerate() {
        if index > 0 {
            builder.push(", ");
        }
        bind_json(&mut builder, object.get(column).unwrap_or(&Value::Null));
    }
    builder.push(")");
    builder.build().execute(&mut **tx).await?;
    Ok(())
}

fn bind_json(builder: &mut QueryBuilder<Sqlite>, value: &Value) {
    match value {
        Value::Null => {
            builder.push_bind(Option::<String>::None);
        }
        Value::Bool(value) => {
            builder.push_bind(Some(if *value { 1_i64 } else { 0_i64 }));
        }
        Value::Number(number) => {
            if let Some(value) = number.as_i64() {
                builder.push_bind(Some(value));
            } else if let Some(value) = number.as_f64() {
                builder.push_bind(Some(value));
            } else {
                builder.push_bind(Some(number.to_string()));
            }
        }
        Value::String(value) => {
            builder.push_bind(Some(value.clone()));
        }
        other => {
            builder.push_bind(Some(other.to_string()));
        }
    }
}

fn is_safe_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}
