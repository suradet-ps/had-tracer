//! Local SQLite store for HAD Tracer.
//!
//! The store owns the HAD catalog, alert rules and events, tracer cases,
//! findings, CAPA, standards, KPIs, audit log, backup, export, and import.
//! It never touches HOSxP.

pub mod backup;
pub mod error;
pub mod repository;
pub mod schema;
pub mod seed;

use std::path::{Path, PathBuf};
use std::time::Duration;

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};

pub use backup::{
    BackupResult, ExportResult, ImportOptions, ImportResult, file_checksum, verify_sqlite_file,
};
pub use error::{Result, StoreError};
pub use repository::{AlertCounts, AuditEntry, DashboardMetrics, KpiDefinition, StandardRef};

/// A cloneable handle to the local SQLite store.
#[derive(Clone)]
pub struct Store {
    pub(crate) pool: SqlitePool,
    path: PathBuf,
}

impl Store {
    /// Open or create the SQLite database and run migrations and seed data.
    pub async fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let options = SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .foreign_keys(true)
            .busy_timeout(Duration::from_secs(5));
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;
        let store = Self { pool, path };
        store.migrate().await?;
        store.record_app_meta().await?;
        seed::seed_if_empty(&store).await?;
        Ok(store)
    }

    /// Open an in-memory store for tests.
    pub async fn open_in_memory() -> Result<Self> {
        let options = SqliteConnectOptions::new()
            .in_memory(true)
            .foreign_keys(true)
            .busy_timeout(Duration::from_secs(5));
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;
        let store = Self {
            pool,
            path: PathBuf::from(":memory:"),
        };
        store.migrate().await?;
        store.record_app_meta().await?;
        seed::seed_if_empty(&store).await?;
        Ok(store)
    }

    /// Path of the live database file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Flush the WAL before a file-level restore.
    pub async fn checkpoint(&self) -> Result<()> {
        sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Close the pool. Used before restoring a database file.
    pub async fn close(&self) {
        self.pool.close().await;
    }

    /// Direct pool access for integration tests and maintenance.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    async fn migrate(&self) -> Result<()> {
        let version: i64 = sqlx::query_scalar("PRAGMA user_version")
            .fetch_one(&self.pool)
            .await?;
        if version < schema::SCHEMA_VERSION {
            for statement in schema::SCHEMA_SQL.split(';') {
                let statement = statement.trim();
                if statement.is_empty() {
                    continue;
                }
                sqlx::query(sqlx::AssertSqlSafe(statement.to_string()))
                    .execute(&self.pool)
                    .await?;
            }
            let pragma = format!("PRAGMA user_version = {}", schema::SCHEMA_VERSION);
            sqlx::query(sqlx::AssertSqlSafe(pragma))
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    async fn record_app_meta(&self) -> Result<()> {
        for (key, value) in [
            ("schema_version", schema::SCHEMA_VERSION.to_string()),
            ("app_version", env!("CARGO_PKG_VERSION").to_string()),
            ("product", "HAD Tracer".to_string()),
        ] {
            sqlx::query("INSERT OR IGNORE INTO app_meta (key, value) VALUES (?, ?)")
                .bind(key)
                .bind(value)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn opens_in_memory_and_seeds_catalog() {
        let store = Store::open_in_memory().await.unwrap();
        let drugs = store.list_had_drugs().await.unwrap();
        assert!(drugs.len() >= 20);
        let rules = store.list_alert_rules().await.unwrap();
        assert!(!rules.is_empty());
        let metrics = store.dashboard_metrics().await.unwrap();
        assert!(metrics.had_total >= 20);
    }

    #[tokio::test]
    async fn creates_tracer_with_all_phases() {
        let store = Store::open_in_memory().await.unwrap();
        let case = had_tracer_core::TracerCase::new(
            had_tracer_core::CaseType::Individual,
            "Warfarin tracer".into(),
            "Review a high INR case".into(),
            "HN001".into(),
            Some("pharmacist".into()),
        );
        let workspace = store.create_tracer_case(&case).await.unwrap();
        assert_eq!(workspace.phases.len(), 7);
        assert!(!workspace.checks.is_empty());
    }

    #[tokio::test]
    async fn backup_export_and_import_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("live.db")).await.unwrap();
        let backup_path = dir.path().join("backup.db");
        let backup = store.backup_to(&backup_path).await.unwrap();
        assert_eq!(backup.integrity, "ok");
        assert!(std::path::Path::new(&backup.path).exists());
        assert!(std::path::Path::new(&backup.manifest_path).exists());

        let export_path = dir.path().join("export.json");
        let export = store.export_json(&export_path).await.unwrap();
        assert!(export.rows > 0);

        let target = Store::open(dir.path().join("target.db")).await.unwrap();
        let result = target
            .import_json(
                &export_path,
                &ImportOptions {
                    mode: "replace".to_string(),
                    dry_run: false,
                },
            )
            .await
            .unwrap();
        assert!(!result.dry_run);
        assert!(result.rows > 0);
        assert!(!target.list_had_drugs().await.unwrap().is_empty());
    }
}
