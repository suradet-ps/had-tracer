use secrecy::SecretString;

/// Connection settings for a read-only HOSxP session.
#[derive(Debug, Clone)]
pub struct HosxpConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub password: SecretString,
    pub tls_mode: String,
    pub statement_timeout_ms: u64,
}

impl HosxpConfig {
    pub fn new(
        host: impl Into<String>,
        port: u16,
        database: impl Into<String>,
        user: impl Into<String>,
        password: SecretString,
    ) -> Self {
        Self {
            host: host.into(),
            port,
            database: database.into(),
            user: user.into(),
            password,
            tls_mode: "preferred".to_string(),
            statement_timeout_ms: 15_000,
        }
    }
}
