use crate::error::{Error, Result};

/// Validate that a statement is read-only.
///
/// This is defense in depth on top of a read-only database user and a
/// read-only session. It rejects multi-statement input and only allows
/// SELECT, SHOW, DESCRIBE, EXPLAIN, and WITH.
pub fn assert_read_only(sql: &str) -> Result<()> {
    let trimmed = sql.trim();
    if trimmed.is_empty() {
        return Err(Error::Guard("empty statement".to_string()));
    }
    if trimmed.contains(';') {
        return Err(Error::Guard(
            "multi-statement input is not allowed".to_string(),
        ));
    }
    let first = trimmed
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    match first.as_str() {
        "SELECT" | "SHOW" | "DESCRIBE" | "EXPLAIN" | "WITH" => Ok(()),
        other => Err(Error::Guard(format!("statement starts with {other}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_select_and_cte() {
        assert!(assert_read_only("SELECT 1").is_ok());
        assert!(assert_read_only("  WITH x AS (SELECT 1) SELECT * FROM x").is_ok());
    }

    #[test]
    fn rejects_writes_and_multi_statement() {
        assert!(assert_read_only("UPDATE patient SET hn = 'x'").is_err());
        assert!(assert_read_only("SELECT 1; DROP TABLE patient").is_err());
        assert!(assert_read_only("DELETE FROM patient").is_err());
    }
}
