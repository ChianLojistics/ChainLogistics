use chrono::{DateTime, Duration, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha256;
use sqlx::{FromRow, PgPool, QueryBuilder};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventCategory {
    UserAction,
    AuthEvent,
    SecurityEvent,
    DataAccess,
    BusinessEvent,
    SystemEvent,
}

impl AuditEventCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UserAction => "user_action",
            Self::AuthEvent => "auth_event",
            Self::SecurityEvent => "security_event",
            Self::DataAccess => "data_access",
            Self::BusinessEvent => "business_event",
            Self::SystemEvent => "system_event",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AuditSeverity {
    Info,
    Warn,
    Error,
}

impl AuditSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAuditEvent {
    pub correlation_id: Option<String>,
    pub user_id: Option<Uuid>,
    pub actor_api_key_id: Option<Uuid>,
    pub event_category: AuditEventCategory,
    pub event_type: String,
    pub severity: AuditSeverity,
    pub action: String,
    pub resource_type: Option<String>,
    pub target_resource_id: Option<String>,
    pub http_method: Option<String>,
    pub http_path: Option<String>,
    pub http_status: Option<u16>,
    pub success: bool,
    pub error_code: Option<String>,
    pub business_context: Option<String>,
    pub changes: Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLogRecord {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub actor_api_key_id: Option<Uuid>,
    pub correlation_id: Option<String>,
    pub event_category: Option<String>,
    pub event_type: Option<String>,
    pub severity: Option<String>,
    pub action: String,
    pub resource_type: Option<String>,
    pub target_resource_id: Option<String>,
    pub http_method: Option<String>,
    pub http_path: Option<String>,
    pub http_status: Option<i16>,
    pub success: Option<bool>,
    pub error_code: Option<String>,
    pub business_context: Option<String>,
    pub changes: Option<Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub prev_hash: Option<Vec<u8>>,
    pub row_hash: Option<Vec<u8>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuditLogQuery {
    pub user_id: Option<Uuid>,
    pub actor_api_key_id: Option<Uuid>,
    pub event_category: Option<String>,
    pub event_type: Option<String>,
    pub resource_type: Option<String>,
    pub target_resource_id: Option<String>,
    pub success: Option<bool>,
    pub correlation_id: Option<String>,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditChainVerification {
    pub checked: usize,
    pub valid: bool,
    pub broken_at: Option<Uuid>,
}

#[derive(Clone)]
pub struct AuditService {
    pool: PgPool,
    enabled: bool,
    hmac_key: Vec<u8>,
    retention_days: i64,
}

impl AuditService {
    pub fn new(pool: PgPool, enabled: bool, hmac_key: String, retention_days: i64) -> Self {
        Self {
            pool,
            enabled,
            hmac_key: hmac_key.into_bytes(),
            retention_days,
        }
    }

    pub async fn log(
        &self,
        mut event: NewAuditEvent,
    ) -> Result<Option<AuditLogRecord>, sqlx::Error> {
        if !self.enabled {
            return Ok(None);
        }

        sanitize_event(&mut event);
        let prev_hash = self.latest_hash().await?;
        let row_hash = self.compute_hash(&event, prev_hash.as_deref());

        let record = sqlx::query_as::<_, AuditLogRecord>(
            r#"
            INSERT INTO audit_logs (
                user_id, actor_api_key_id, correlation_id, event_category, event_type,
                severity, action, resource_type, target_resource_id, http_method,
                http_path, http_status, success, error_code, business_context,
                changes, ip_address, user_agent, prev_hash, row_hash, retention_days
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15,
                $16, $17::inet, $18, $19, $20, $21
            )
            RETURNING
                id, user_id, actor_api_key_id, correlation_id, event_category, event_type,
                severity, action, resource_type, target_resource_id, http_method,
                http_path, http_status, success, error_code, business_context,
                changes, ip_address::text, user_agent, prev_hash, row_hash, created_at
            "#,
        )
        .bind(event.user_id)
        .bind(event.actor_api_key_id)
        .bind(event.correlation_id)
        .bind(event.event_category.as_str())
        .bind(event.event_type)
        .bind(event.severity.as_str())
        .bind(event.action)
        .bind(event.resource_type)
        .bind(event.target_resource_id)
        .bind(event.http_method)
        .bind(event.http_path)
        .bind(event.http_status.map(|status| status as i16))
        .bind(event.success)
        .bind(event.error_code)
        .bind(event.business_context)
        .bind(event.changes)
        .bind(event.ip_address)
        .bind(event.user_agent)
        .bind(prev_hash)
        .bind(row_hash)
        .bind(self.retention_days as i32)
        .fetch_one(&self.pool)
        .await?;

        Ok(Some(record))
    }

    pub async fn query(&self, filters: AuditLogQuery) -> Result<Vec<AuditLogRecord>, sqlx::Error> {
        let mut query = QueryBuilder::new(
            r#"
            SELECT
                id, user_id, actor_api_key_id, correlation_id, event_category, event_type,
                severity, action, resource_type, target_resource_id, http_method,
                http_path, http_status, success, error_code, business_context,
                changes, ip_address::text, user_agent, prev_hash, row_hash, created_at
            FROM audit_logs
            WHERE 1 = 1
            "#,
        );

        if let Some(user_id) = filters.user_id {
            query.push(" AND user_id = ");
            query.push_bind(user_id);
        }
        if let Some(actor_api_key_id) = filters.actor_api_key_id {
            query.push(" AND actor_api_key_id = ");
            query.push_bind(actor_api_key_id);
        }
        if let Some(event_category) = filters.event_category {
            query.push(" AND event_category = ");
            query.push_bind(sanitize_log_value(&event_category, 32));
        }
        if let Some(event_type) = filters.event_type {
            query.push(" AND event_type = ");
            query.push_bind(sanitize_log_value(&event_type, 64));
        }
        if let Some(resource_type) = filters.resource_type {
            query.push(" AND resource_type = ");
            query.push_bind(sanitize_log_value(&resource_type, 100));
        }
        if let Some(target_resource_id) = filters.target_resource_id {
            query.push(" AND target_resource_id = ");
            query.push_bind(sanitize_log_value(&target_resource_id, 255));
        }
        if let Some(success) = filters.success {
            query.push(" AND success = ");
            query.push_bind(success);
        }
        if let Some(correlation_id) = filters.correlation_id {
            query.push(" AND correlation_id = ");
            query.push_bind(sanitize_log_value(&correlation_id, 128));
        }
        if let Some(start) = filters.start {
            query.push(" AND created_at >= ");
            query.push_bind(start);
        }
        if let Some(end) = filters.end {
            query.push(" AND created_at <= ");
            query.push_bind(end);
        }

        query.push(" ORDER BY created_at DESC LIMIT ");
        query.push_bind(filters.limit.unwrap_or(100).clamp(1, 500));
        query.push(" OFFSET ");
        query.push_bind(filters.offset.unwrap_or(0).max(0));

        query
            .build_query_as::<AuditLogRecord>()
            .fetch_all(&self.pool)
            .await
    }

    pub async fn purge_expired(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM audit_logs WHERE created_at < $1")
            .bind(Utc::now() - Duration::days(self.retention_days))
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    pub async fn verify_chain(&self) -> Result<AuditChainVerification, sqlx::Error> {
        let records = sqlx::query_as::<_, AuditLogRecord>(
            r#"
            SELECT
                id, user_id, actor_api_key_id, correlation_id, event_category, event_type,
                severity, action, resource_type, target_resource_id, http_method,
                http_path, http_status, success, error_code, business_context,
                changes, ip_address::text, user_agent, prev_hash, row_hash, created_at
            FROM audit_logs
            WHERE row_hash IS NOT NULL
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut prev_hash: Option<Vec<u8>> = None;
        for (index, record) in records.iter().enumerate() {
            if record.prev_hash != prev_hash {
                return Ok(AuditChainVerification {
                    checked: index,
                    valid: false,
                    broken_at: Some(record.id),
                });
            }

            let event = NewAuditEvent::from(record.clone());
            let expected = self.compute_hash(&event, record.prev_hash.as_deref());
            if record.row_hash.as_deref() != Some(expected.as_slice()) {
                return Ok(AuditChainVerification {
                    checked: index,
                    valid: false,
                    broken_at: Some(record.id),
                });
            }

            prev_hash = record.row_hash.clone();
        }

        Ok(AuditChainVerification {
            checked: records.len(),
            valid: true,
            broken_at: None,
        })
    }

    async fn latest_hash(&self) -> Result<Option<Vec<u8>>, sqlx::Error> {
        sqlx::query_scalar::<_, Vec<u8>>(
            "SELECT row_hash FROM audit_logs WHERE row_hash IS NOT NULL ORDER BY created_at DESC, id DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await
    }

    fn compute_hash(&self, event: &NewAuditEvent, prev_hash: Option<&[u8]>) -> Vec<u8> {
        compute_event_hmac(&self.hmac_key, event, prev_hash)
    }
}

impl From<AuditLogRecord> for NewAuditEvent {
    fn from(record: AuditLogRecord) -> Self {
        Self {
            correlation_id: record.correlation_id,
            user_id: record.user_id,
            actor_api_key_id: record.actor_api_key_id,
            event_category: parse_category(record.event_category.as_deref()),
            event_type: record.event_type.unwrap_or_else(|| "unknown".to_string()),
            severity: parse_severity(record.severity.as_deref()),
            action: record.action,
            resource_type: record.resource_type,
            target_resource_id: record.target_resource_id,
            http_method: record.http_method,
            http_path: record.http_path,
            http_status: record.http_status.map(|status| status as u16),
            success: record.success.unwrap_or(false),
            error_code: record.error_code,
            business_context: record.business_context,
            changes: record.changes.unwrap_or_else(|| serde_json::json!({})),
            ip_address: record.ip_address,
            user_agent: record.user_agent,
        }
    }
}

pub fn sanitize_log_value(value: &str, max_len: usize) -> String {
    let redacted = redact_sensitive_data(value);

    redacted
        .chars()
        .filter(|ch| !ch.is_control() || *ch == '\t')
        .take(max_len)
        .collect::<String>()
        .trim()
        .to_string()
}

pub fn redact_sensitive_data(value: &str) -> String {
    let pattern = regex::Regex::new(
        r"(?i)(bearer\s+|api[_-]?key[=:]\s*|token[=:]\s*|password[=:]\s*)[a-z0-9._~+/=-]+",
    )
    .expect("valid audit redaction regex");

    pattern.replace_all(value, "$1[redacted]").to_string()
}

fn sanitize_event(event: &mut NewAuditEvent) {
    event.correlation_id = event
        .correlation_id
        .take()
        .map(|value| sanitize_log_value(&value, 128));
    event.event_type = sanitize_log_value(&event.event_type, 64);
    event.action = sanitize_log_value(&event.action, 255);
    event.resource_type = event
        .resource_type
        .take()
        .map(|value| sanitize_log_value(&value, 100));
    event.target_resource_id = event
        .target_resource_id
        .take()
        .map(|value| sanitize_log_value(&value, 255));
    event.http_method = event
        .http_method
        .take()
        .map(|value| sanitize_log_value(&value, 8));
    event.http_path = event
        .http_path
        .take()
        .map(|value| sanitize_log_value(&value, 1024));
    event.error_code = event
        .error_code
        .take()
        .map(|value| sanitize_log_value(&value, 64));
    event.business_context = event
        .business_context
        .take()
        .map(|value| sanitize_log_value(&value, 512));
    event.ip_address = event
        .ip_address
        .take()
        .and_then(|value| sanitize_ip_address(&value));
    event.user_agent = event
        .user_agent
        .take()
        .map(|value| sanitize_log_value(&value, 512));
    event.changes = sanitize_json(event.changes.clone());
}

fn sanitize_ip_address(value: &str) -> Option<String> {
    let candidate = value.split(',').next().unwrap_or(value).trim();
    if candidate.parse::<std::net::IpAddr>().is_ok() {
        Some(candidate.to_string())
    } else {
        None
    }
}

fn sanitize_json(value: Value) -> Value {
    match value {
        Value::String(text) => Value::String(sanitize_log_value(&text, 1024)),
        Value::Array(values) => Value::Array(values.into_iter().map(sanitize_json).collect()),
        Value::Object(values) => Value::Object(
            values
                .into_iter()
                .map(|(key, value)| {
                    let safe_value = if is_sensitive_key(&key) {
                        Value::String("[redacted]".to_string())
                    } else {
                        sanitize_json(value)
                    };
                    (sanitize_log_value(&key, 128), safe_value)
                })
                .collect(),
        ),
        other => other,
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key.contains("password")
        || key.contains("token")
        || key.contains("secret")
        || key.contains("api_key")
        || key.contains("authorization")
}

fn compute_event_hmac(key: &[u8], event: &NewAuditEvent, prev_hash: Option<&[u8]>) -> Vec<u8> {
    let payload = serde_json::json!({
        "prev_hash": prev_hash.map(hex::encode),
        "correlation_id": event.correlation_id,
        "user_id": event.user_id,
        "actor_api_key_id": event.actor_api_key_id,
        "event_category": event.event_category.as_str(),
        "event_type": event.event_type,
        "severity": event.severity.as_str(),
        "action": event.action,
        "resource_type": event.resource_type,
        "target_resource_id": event.target_resource_id,
        "http_method": event.http_method,
        "http_path": event.http_path,
        "http_status": event.http_status,
        "success": event.success,
        "error_code": event.error_code,
        "business_context": event.business_context,
        "changes": event.changes,
    });

    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts keys of any size");
    mac.update(payload.to_string().as_bytes());
    mac.finalize().into_bytes().to_vec()
}

fn parse_category(value: Option<&str>) -> AuditEventCategory {
    match value {
        Some("auth_event") => AuditEventCategory::AuthEvent,
        Some("security_event") => AuditEventCategory::SecurityEvent,
        Some("data_access") => AuditEventCategory::DataAccess,
        Some("business_event") => AuditEventCategory::BusinessEvent,
        Some("system_event") => AuditEventCategory::SystemEvent,
        _ => AuditEventCategory::UserAction,
    }
}

fn parse_severity(value: Option<&str>) -> AuditSeverity {
    match value {
        Some("warn") => AuditSeverity::Warn,
        Some("error") => AuditSeverity::Error,
        _ => AuditSeverity::Info,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_event() -> NewAuditEvent {
        NewAuditEvent {
            correlation_id: Some("req-123".to_string()),
            user_id: Some(Uuid::nil()),
            actor_api_key_id: None,
            event_category: AuditEventCategory::UserAction,
            event_type: "http_request".to_string(),
            severity: AuditSeverity::Info,
            action: "GET /api/v1/products".to_string(),
            resource_type: Some("products".to_string()),
            target_resource_id: None,
            http_method: Some("GET".to_string()),
            http_path: Some("/api/v1/products".to_string()),
            http_status: Some(200),
            success: true,
            error_code: None,
            business_context: None,
            changes: serde_json::json!({"query": "all"}),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("test".to_string()),
        }
    }

    #[test]
    fn sanitizes_control_characters_and_redacts_tokens() {
        let value = sanitize_log_value("Bearer abc123\r\nnext token=secret", 128);

        assert!(!value.contains('\r'));
        assert!(!value.contains('\n'));
        assert!(value.contains("Bearer [redacted]"));
        assert!(value.contains("token=[redacted]"));
    }

    #[test]
    fn redacts_sensitive_json_keys() {
        let value = sanitize_json(serde_json::json!({
            "password": "secret",
            "nested": {"api_key": "abc123"},
            "safe": "visible"
        }));

        assert_eq!(value["password"], "[redacted]");
        assert_eq!(value["nested"]["api_key"], "[redacted]");
        assert_eq!(value["safe"], "visible");
    }

    #[test]
    fn hmac_is_deterministic_for_same_event() {
        let event = sample_event();
        let first = compute_event_hmac(b"secret", &event, None);
        let second = compute_event_hmac(b"secret", &event, None);

        assert_eq!(first, second);
    }

    #[test]
    fn hmac_changes_when_event_is_tampered() {
        let event = sample_event();
        let original = compute_event_hmac(b"secret", &event, None);

        let mut tampered = event;
        tampered.action = "DELETE /api/v1/products/123".to_string();
        let changed = compute_event_hmac(b"secret", &tampered, None);

        assert_ne!(original, changed);
    }

    #[test]
    fn hmac_chain_depends_on_previous_hash() {
        let event = sample_event();
        let first = compute_event_hmac(b"secret", &event, None);
        let chained = compute_event_hmac(b"secret", &event, Some(&first));

        assert_ne!(first, chained);
    }
}
