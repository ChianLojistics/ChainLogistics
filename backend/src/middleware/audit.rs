use axum::http::{HeaderMap, Method, StatusCode, Uri};
use serde_json::json;

use crate::{
    middleware::auth::AuthContext,
    services::audit_service::{AuditEventCategory, AuditSeverity, NewAuditEvent},
    AppState,
};

pub const CORRELATION_ID_HEADER: &str = "x-correlation-id";

pub fn should_skip_audit(path: &str) -> bool {
    path == "/health"
        || path == "/health/db"
        || path.starts_with("/docs")
        || path.starts_with("/swagger")
        || path.starts_with("/api-docs")
}

pub fn correlation_id_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get(CORRELATION_ID_HEADER)
        .or_else(|| headers.get("x-request-id"))
        .and_then(|value| value.to_str().ok())
        .map(|value| sanitize_correlation_id(value))
        .filter(|value| !value.is_empty())
}

pub fn client_ip_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
}

pub fn user_agent_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
}

pub fn spawn_http_audit(
    state: AppState,
    auth_context: Option<AuthContext>,
    method: Method,
    uri: Uri,
    status: StatusCode,
    headers: HeaderMap,
    correlation_id: Option<String>,
) {
    let path = uri.path().to_string();
    if should_skip_audit(&path) {
        return;
    }

    let event = build_http_audit_event(auth_context, method, uri, status, headers, correlation_id);

    tokio::spawn(async move {
        let correlation_id = event.correlation_id.clone();
        if let Err(err) = state.audit_service.log(event).await {
            tracing::warn!(
                correlation_id = correlation_id.as_deref().unwrap_or("unknown"),
                error = ?err,
                "Failed to persist audit event"
            );
        }
    });
}

pub fn build_http_audit_event(
    auth_context: Option<AuthContext>,
    method: Method,
    uri: Uri,
    status: StatusCode,
    headers: HeaderMap,
    correlation_id: Option<String>,
) -> NewAuditEvent {
    let path = uri.path().to_string();
    let event_category = classify_event(&method, &path, status);
    let severity = classify_severity(status);
    let success = status.is_success();
    let resource_type = resource_type_from_path(&path);
    let target_resource_id = target_resource_id_from_path(&path);
    let action = format!("{} {}", method.as_str(), path);

    NewAuditEvent {
        correlation_id,
        user_id: auth_context.as_ref().map(|context| context.user_id),
        actor_api_key_id: auth_context.as_ref().and_then(|context| context.api_key_id),
        event_category,
        event_type: "http_request".to_string(),
        severity,
        action,
        resource_type,
        target_resource_id,
        http_method: Some(method.as_str().to_string()),
        http_path: Some(path),
        http_status: Some(status.as_u16()),
        success,
        error_code: if success {
            None
        } else {
            Some(status.as_u16().to_string())
        },
        business_context: business_context_from_path(uri.path()),
        changes: json!({
            "query": uri.query(),
        }),
        ip_address: client_ip_from_headers(&headers),
        user_agent: user_agent_from_headers(&headers),
    }
}

fn classify_event(method: &Method, path: &str, status: StatusCode) -> AuditEventCategory {
    if status == StatusCode::UNAUTHORIZED
        || status == StatusCode::FORBIDDEN
        || status == StatusCode::TOO_MANY_REQUESTS
        || status.is_server_error()
    {
        return AuditEventCategory::SecurityEvent;
    }

    if is_business_path(path) && method != Method::GET {
        return AuditEventCategory::BusinessEvent;
    }

    if method == Method::GET {
        AuditEventCategory::DataAccess
    } else {
        AuditEventCategory::UserAction
    }
}

fn classify_severity(status: StatusCode) -> AuditSeverity {
    if status.is_server_error() {
        AuditSeverity::Error
    } else if status.is_client_error() {
        AuditSeverity::Warn
    } else {
        AuditSeverity::Info
    }
}

fn is_business_path(path: &str) -> bool {
    path.contains("/products")
        || path.contains("/events")
        || path.contains("/transactions")
        || path.contains("/invoices")
        || path.contains("/financing")
        || path.contains("/carbon")
        || path.contains("/compliance")
}

fn business_context_from_path(path: &str) -> Option<String> {
    if is_business_path(path) {
        Some("supply_chain_operation".to_string())
    } else {
        None
    }
}

fn resource_type_from_path(path: &str) -> Option<String> {
    let mut segments = path.split('/').filter(|segment| !segment.is_empty());
    while let Some(segment) = segments.next() {
        if segment == "v1" || segment == "admin" {
            continue;
        }
        return Some(segment.to_string());
    }
    None
}

fn target_resource_id_from_path(path: &str) -> Option<String> {
    let segments: Vec<&str> = path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();
    segments.windows(2).find_map(|pair| {
        let candidate = pair[1];
        if candidate.starts_with(':') || candidate == "me" {
            None
        } else if pair[0] != "v1" && pair[0] != "admin" {
            Some(candidate.to_string())
        } else {
            None
        }
    })
}

fn sanitize_correlation_id(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':'))
        .take(128)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_get_as_data_access() {
        assert_eq!(
            classify_event(&Method::GET, "/api/v1/products", StatusCode::OK),
            AuditEventCategory::DataAccess
        );
    }

    #[test]
    fn classifies_business_writes() {
        assert_eq!(
            classify_event(&Method::POST, "/api/v1/admin/products", StatusCode::CREATED),
            AuditEventCategory::BusinessEvent
        );
    }

    #[test]
    fn classifies_auth_failures_as_security_events() {
        assert_eq!(
            classify_event(&Method::GET, "/api/v1/products", StatusCode::FORBIDDEN),
            AuditEventCategory::SecurityEvent
        );
    }

    #[test]
    fn sanitizes_correlation_ids() {
        assert_eq!(sanitize_correlation_id("abc\r\n123"), "abc123");
    }
}
