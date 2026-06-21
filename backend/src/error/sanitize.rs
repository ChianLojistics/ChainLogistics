use regex::Regex;

/// Sanitize database errors to prevent information disclosure
pub fn sanitize_database_error(err: &sqlx::Error) -> String {
    match err {
        sqlx::Error::RowNotFound => "Record not found".to_string(),
        sqlx::Error::ColumnNotFound(_) => "Database schema error".to_string(),
        sqlx::Error::Database(_) => "Database constraint violation".to_string(),
        sqlx::Error::PoolTimedOut => "Database connection timeout".to_string(),
        sqlx::Error::PoolClosed => "Database connection closed".to_string(),
        _ => "Database operation failed".to_string(),
    }
}

/// Sanitize user messages to prevent information disclosure
pub fn sanitize_message(msg: &str) -> String {
    let msg = Regex::new(r"(/[a-zA-Z0-9_\-./]+)")
        .unwrap()
        .replace_all(msg, "[path]");

    let msg = Regex::new(r"(?i)(SELECT|INSERT|UPDATE|DELETE|FROM|WHERE|JOIN)")
        .unwrap()
        .replace_all(&msg, "[sql]");

    let msg = Regex::new(r"(postgres|mysql|mongodb)://[^\s]+")
        .unwrap()
        .replace_all(&msg, "[connection]");

    let msg = Regex::new(r"([a-zA-Z0-9_-]{32,})")
        .unwrap()
        .replace_all(&msg, "[token]");

    msg.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_message_removes_paths() {
        let msg = "Error in /usr/local/app/src/main.rs";
        let sanitized = sanitize_message(msg);
        assert!(sanitized.contains("[path]"));
        assert!(!sanitized.contains("/usr/local"));
    }

    #[test]
    fn test_sanitize_message_removes_sql() {
        let msg = "Error in SELECT * FROM users WHERE id = 1";
        let sanitized = sanitize_message(msg);
        assert!(sanitized.contains("[sql]"));
        assert!(!sanitized.contains("SELECT"));
    }

    #[test]
    fn test_sanitize_message_removes_connection_strings() {
        let msg = "Failed to connect to postgres://user:pass@localhost:5432/db";
        let sanitized = sanitize_message(msg);
        assert!(sanitized.contains("[connection]"));
        assert!(!sanitized.contains("postgres://"));
    }

    #[test]
    fn test_sanitize_message_removes_tokens() {
        let msg = "Invalid token: abc123def456ghi789jkl012mno345pqr678";
        let sanitized = sanitize_message(msg);
        assert!(sanitized.contains("[token]"));
    }
}
