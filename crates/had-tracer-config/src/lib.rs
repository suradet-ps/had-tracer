//! Encrypted HOSxP connection settings and local application settings.
//!
//! Two files live under the platform config directory:
//!
//! * `connection.json` - encrypted connection settings.
//! * `settings.json` - plain, non-secret site settings.
//!
//! The SQLite app store lives under the platform data directory so that a
//! configuration reset never deletes tracer evidence by accident.

use std::fs;
use std::path::{Path, PathBuf};

use encryptman_keyring::Vault;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const KEYRING_SERVICE: &str = "HADTracer";
pub const CONNECTION_FILE: &str = "connection.json";
pub const SETTINGS_FILE: &str = "settings.json";
pub const DATABASE_FILE: &str = "had-tracer.db";
pub const BACKUP_DIR: &str = "backups";

/// HOSxP connection settings. Passwords never touch disk in plaintext.
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub password: SecretString,
    pub tls_mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConnectionConfigRaw {
    host: String,
    port: u16,
    database: String,
    user: String,
    password: String,
    #[serde(default = "default_tls_mode")]
    tls_mode: String,
}

fn default_tls_mode() -> String {
    "preferred".to_string()
}

/// Plain application settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SiteSettings {
    pub site_name: String,
    pub history_days: u32,
    pub alert_lookback_hours: i64,
    pub auto_backup: bool,
    pub backup_retention: u32,
}

impl Default for SiteSettings {
    fn default() -> Self {
        Self {
            site_name: "โรงพยาบาลสระโบสถ์".to_string(),
            history_days: 30,
            alert_lookback_hours: 72,
            auto_backup: false,
            backup_retention: 14,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct EncryptedFile {
    version: u32,
    ciphertext: String,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("keychain error: {0}")]
    Keychain(#[from] encryptman_keyring::Error),
    #[error("crypto error: {0}")]
    Crypto(#[from] encryptman::CryptoError),
    #[error("config file error at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("config file is not valid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("vault error: {0}")]
    Vault(String),
    #[error("unsupported config version: {0}")]
    UnsupportedVersion(u32),
    #[error("no connection configuration saved")]
    NoConfig,
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait SecretVault: Send + Sync {
    fn encrypt(
        &self,
        plaintext: &str,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>>;
    fn decrypt(
        &self,
        ciphertext: &str,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct KeyringVault {
    vault: Vault,
}

impl KeyringVault {
    pub fn new() -> Result<Self> {
        Ok(Self {
            vault: Vault::new(KEYRING_SERVICE)?,
        })
    }
}

impl SecretVault for KeyringVault {
    fn encrypt(
        &self,
        plaintext: &str,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.vault.encrypt(plaintext).map_err(Box::from)
    }

    fn decrypt(
        &self,
        ciphertext: &str,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.vault.decrypt(ciphertext).map_err(Box::from)
    }
}

/// In-memory vault used when the OS keychain is unavailable. The key lives
/// only for the process lifetime, so saved credentials cannot be decrypted
/// after a restart. This keeps the app usable in headless test environments.
pub struct EphemeralVault {
    key: encryptman::MasterKey,
}

impl EphemeralVault {
    pub fn new() -> Self {
        Self {
            key: encryptman::MasterKey::generate().expect("invariant: OS randomness"),
        }
    }
}

impl Default for EphemeralVault {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretVault for EphemeralVault {
    fn encrypt(
        &self,
        plaintext: &str,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok(encryptman::encrypt(&self.key, plaintext)?)
    }

    fn decrypt(
        &self,
        ciphertext: &str,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok(encryptman::decrypt(&self.key, ciphertext)?)
    }
}

pub struct ConfigStore {
    vault: Box<dyn SecretVault>,
    config_dir: PathBuf,
    data_dir: PathBuf,
}

impl ConfigStore {
    pub fn open() -> Result<Self> {
        let vault = KeyringVault::new()?;
        Ok(Self::with_vault(
            Box::new(vault),
            default_config_dir(),
            default_data_dir(),
        ))
    }

    pub fn with_vault(vault: Box<dyn SecretVault>, config_dir: PathBuf, data_dir: PathBuf) -> Self {
        Self {
            vault,
            config_dir,
            data_dir,
        }
    }

    /// Open with an in-memory key when the OS keychain is unavailable.
    /// Credentials saved this way do not survive a restart.
    pub fn open_ephemeral() -> Self {
        Self::with_vault(
            Box::new(EphemeralVault::new()),
            default_config_dir(),
            default_data_dir(),
        )
    }

    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn connection_path(&self) -> PathBuf {
        self.config_dir.join(CONNECTION_FILE)
    }

    pub fn settings_path(&self) -> PathBuf {
        self.config_dir.join(SETTINGS_FILE)
    }

    pub fn database_path(&self) -> PathBuf {
        self.data_dir.join(DATABASE_FILE)
    }

    pub fn backup_dir(&self) -> PathBuf {
        self.data_dir.join(BACKUP_DIR)
    }

    pub fn connection_exists(&self) -> bool {
        self.connection_path().exists()
    }

    pub fn load_connection(&self) -> Result<ConnectionConfig> {
        let raw = read_optional(&self.connection_path())?.ok_or(Error::NoConfig)?;
        let file: EncryptedFile = serde_json::from_str(&raw)?;
        if file.version != 1 {
            return Err(Error::UnsupportedVersion(file.version));
        }
        let plaintext = self
            .vault
            .decrypt(&file.ciphertext)
            .map_err(|e| Error::Vault(e.to_string()))?;
        let raw_config: ConnectionConfigRaw = serde_json::from_str(&plaintext)?;
        Ok(raw_config.into())
    }

    pub fn save_connection(&self, config: &ConnectionConfig) -> Result<()> {
        let raw_config: ConnectionConfigRaw = config.clone().into();
        let plaintext = serde_json::to_string(&raw_config)?;
        let ciphertext = self
            .vault
            .encrypt(&plaintext)
            .map_err(|e| Error::Vault(e.to_string()))?;
        write_json(
            &self.connection_path(),
            &EncryptedFile {
                version: 1,
                ciphertext,
            },
        )
    }

    pub fn load_settings(&self) -> Result<SiteSettings> {
        let path = self.settings_path();
        let raw = match read_optional(&path)? {
            Some(raw) if !raw.trim().is_empty() => raw,
            _ => return Ok(SiteSettings::default()),
        };
        Ok(serde_json::from_str(&raw)?)
    }

    pub fn save_settings(&self, settings: &SiteSettings) -> Result<()> {
        write_json(&self.settings_path(), settings)
    }

    pub fn clear_connection(&self) -> Result<()> {
        remove_optional(&self.connection_path())
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        fs::create_dir_all(&self.config_dir).map_err(|e| io(e, &self.config_dir))?;
        fs::create_dir_all(&self.data_dir).map_err(|e| io(e, &self.data_dir))?;
        fs::create_dir_all(self.backup_dir()).map_err(|e| io(e, &self.backup_dir()))?;
        Ok(())
    }
}

impl From<ConnectionConfig> for ConnectionConfigRaw {
    fn from(c: ConnectionConfig) -> Self {
        Self {
            host: c.host,
            port: c.port,
            database: c.database,
            user: c.user,
            password: c.password.expose_secret().to_string(),
            tls_mode: c.tls_mode,
        }
    }
}

impl From<ConnectionConfigRaw> for ConnectionConfig {
    fn from(r: ConnectionConfigRaw) -> Self {
        Self {
            host: r.host,
            port: r.port,
            database: r.database,
            user: r.user,
            password: SecretString::from(r.password),
            tls_mode: r.tls_mode,
        }
    }
}

fn default_config_dir() -> PathBuf {
    directories::ProjectDirs::from("", "", "HADTracer")
        .map(|d| d.config_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn default_data_dir() -> PathBuf {
    directories::ProjectDirs::from("", "", "HADTracer")
        .map(|d| d.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn io(source: std::io::Error, path: &Path) -> Error {
    Error::Io {
        path: path.to_path_buf(),
        source,
    }
}

fn read_optional(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(raw) => Ok(Some(raw)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(io(e, path)),
    }
}

fn remove_optional(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(io(e, path)),
    }
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| io(e, path))?;
    }
    fs::write(path, json).map_err(|e| io(e, path))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MemoryVault {
        key: encryptman::MasterKey,
    }

    impl MemoryVault {
        fn new() -> Self {
            Self {
                key: encryptman::MasterKey::generate().expect("test key"),
            }
        }
    }

    impl SecretVault for MemoryVault {
        fn encrypt(
            &self,
            plaintext: &str,
        ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>> {
            Ok(encryptman::encrypt(&self.key, plaintext)?)
        }
        fn decrypt(
            &self,
            ciphertext: &str,
        ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>> {
            Ok(encryptman::decrypt(&self.key, ciphertext)?)
        }
    }

    fn store(dir: &tempfile::TempDir) -> ConfigStore {
        ConfigStore::with_vault(
            Box::new(MemoryVault::new()),
            dir.path().join("config"),
            dir.path().join("data"),
        )
    }

    fn connection() -> ConnectionConfig {
        ConnectionConfig {
            host: "10.0.0.5".into(),
            port: 3306,
            database: "hos".into(),
            user: "had_ro".into(),
            password: SecretString::from("s3cret"),
            tls_mode: "preferred".into(),
        }
    }

    #[test]
    fn connection_roundtrip_is_encrypted_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(&dir);
        store.ensure_dirs().unwrap();
        store.save_connection(&connection()).unwrap();
        let loaded = store.load_connection().unwrap();
        assert_eq!(loaded.host, "10.0.0.5");
        assert_eq!(loaded.password.expose_secret(), "s3cret");
        let disk = fs::read_to_string(store.connection_path()).unwrap();
        assert!(!disk.contains("s3cret"));
    }

    #[test]
    fn settings_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(&dir);
        let settings = SiteSettings {
            site_name: "รพ.ทดสอบ".into(),
            history_days: 60,
            ..SiteSettings::default()
        };
        store.save_settings(&settings).unwrap();
        assert_eq!(store.load_settings().unwrap(), settings);
    }

    #[test]
    fn missing_settings_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(&dir);
        assert_eq!(store.load_settings().unwrap(), SiteSettings::default());
    }
}
