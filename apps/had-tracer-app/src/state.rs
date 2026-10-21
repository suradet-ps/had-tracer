use had_tracer_config::ConfigStore;
use had_tracer_hosxp::{HosxpClient, HosxpConfig};
use had_tracer_store::Store;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::commands::CommandError;

/// Live connection state shown in the top bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionHealth {
    #[default]
    Unconfigured,
    Connecting,
    Connected,
    Degraded,
    Disconnected,
}

/// Tauri-managed application state.
pub struct AppState {
    config: ConfigStore,
    store: RwLock<Option<Store>>,
    hosxp: RwLock<Option<HosxpClient>>,
    health: RwLock<ConnectionHealth>,
}

impl AppState {
    pub fn new() -> Self {
        let config = ConfigStore::open().unwrap_or_else(|error| {
            tracing::warn!("keychain unavailable, using ephemeral vault: {error}");
            ConfigStore::open_ephemeral()
        });
        Self {
            config,
            store: RwLock::new(None),
            hosxp: RwLock::new(None),
            health: RwLock::new(ConnectionHealth::Unconfigured),
        }
    }

    pub fn config(&self) -> &ConfigStore {
        &self.config
    }

    /// Open the local SQLite store lazily and cache the handle.
    pub async fn store(&self) -> Result<Store, CommandError> {
        if let Some(store) = self.store.read().await.as_ref() {
            return Ok(store.clone());
        }
        self.config.ensure_dirs()?;
        let store = Store::open(self.config.database_path()).await?;
        let mut guard = self.store.write().await;
        if let Some(existing) = guard.as_ref() {
            return Ok(existing.clone());
        }
        *guard = Some(store.clone());
        Ok(store)
    }

    /// Replace the cached store after a restore.
    pub async fn set_store(&self, store: Store) {
        *self.store.write().await = Some(store);
    }

    /// Connect to HOSxP lazily using the saved encrypted config.
    pub async fn hosxp(&self) -> Result<HosxpClient, CommandError> {
        if let Some(client) = self.hosxp.read().await.as_ref() {
            return Ok(client.clone());
        }
        let connection = self.config.load_connection().map_err(|error| {
            CommandError::new(
                "notConfigured",
                format!("ยังไม่ได้ตั้งค่าการเชื่อมต่อ HOSxP: {error}"),
            )
        })?;
        *self.health.write().await = ConnectionHealth::Connecting;
        let config = HosxpConfig {
            host: connection.host,
            port: connection.port,
            database: connection.database,
            user: connection.user,
            password: connection.password,
            tls_mode: connection.tls_mode,
            statement_timeout_ms: 15_000,
        };
        match HosxpClient::connect(&config).await {
            Ok(client) => {
                *self.hosxp.write().await = Some(client.clone());
                *self.health.write().await = ConnectionHealth::Connected;
                Ok(client)
            }
            Err(error) => {
                *self.health.write().await = ConnectionHealth::Disconnected;
                Err(error.into())
            }
        }
    }

    pub async fn reset_hosxp(&self) {
        *self.hosxp.write().await = None;
        *self.health.write().await = ConnectionHealth::Unconfigured;
    }

    pub async fn health(&self) -> ConnectionHealth {
        *self.health.read().await
    }

    pub async fn set_health(&self, health: ConnectionHealth) {
        *self.health.write().await = health;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
