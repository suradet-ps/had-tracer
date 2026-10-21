use thiserror::Error;

/// Errors raised by the read-only HOSxP connector.
#[derive(Debug, Error)]
pub enum Error {
    #[error("HOSxP connection failed: {0}")]
    Connect(String),
    #[error("HOSxP query failed: {0}")]
    Database(#[from] sqlx::Error),
    #[error("read-only guard rejected statement: {0}")]
    Guard(String),
    #[error("invalid HOSxP configuration: {0}")]
    Config(String),
    #[error("HOSxP schema is missing a required object: {0}")]
    Schema(String),
}

pub type Result<T> = std::result::Result<T, Error>;

/// MySQL error numbers that mean the object is absent at this site.
pub fn is_schema_variation(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .and_then(|db_err| db_err.try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>())
        .is_some_and(|mysql_err| matches!(mysql_err.number(), 1146 | 1054))
}
