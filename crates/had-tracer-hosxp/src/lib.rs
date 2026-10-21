//! Read-only HOSxP (MySQL/MariaDB) connector for HAD Tracer.
//!
//! The connector follows the verified Sa Bophospital schema contract in
//! AGENTS.md. Every statement is SELECT-only, the session is opened read-only,
//! and a statement timeout is applied per connection.

pub mod client;
pub mod config;
pub mod error;
pub mod queries;
pub mod readonly;

pub use client::{DrugItemRow, HosxpClient, LabItemRow, normalize_date_text};
pub use config::HosxpConfig;
pub use error::{Error, Result};
