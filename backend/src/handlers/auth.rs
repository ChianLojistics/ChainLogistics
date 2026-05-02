use axum::{extract::State, http::HeaderMap, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::{AppState, database::UserRepository, error::AppError, models::{UserRole, NewUser, User}, middleware::{audit::{client_ip_from_headers, correlation_id_from_headers, user_agent_from_headers}, auth::Claims}, services::audit_service::{AuditEventCategory, AuditSeverity, NewAuditEvent}, validation::{validate_email, validate_string}};
use bcrypt::verify;
use jsonwebtoken::{encode, Header, EncodingKey};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub role: UserRole,
    pub stellar_address: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/auth/login",
    tag = "authentication",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 429, description = "Rate limit exceeded")
    )
)]
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    validate_email(&req.email)?;

    let user = match state.user_service.get_user_by_email(&req.email).await? {
        Some(user) => user,
        None => {
            spawn_login_audit(state, headers, None, req.email, false);
            return Err(AppError::Unauthorized);
        }
    };

    if !user.is_active {
        spawn_login_audit(state, headers, Some(user.id), req.email, false);
        return Err(AppError::Unauthorized);
    }

    let is_valid = verify(&req.password, &user.password_hash)
        .map_err(|_| AppError::Internal("Password verification failed".to_string()))?;

    if !is_valid {
        spawn_login_audit(state, headers, Some(user.id), req.email, false);
        return Err(AppError::Unauthorized);
    }

    // Update last login
    let _ = state.user_service.update_last_login(user.id).await;

    // Issue JWT
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user.id.to_string(),
        exp: expiration,
        role: user.role.clone(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|_| AppError::Internal("Token issuance failed".to_string()))?;

    spawn_login_audit(state, headers, Some(user.id), req.email, true);

    Ok(Json(AuthResponse { token, user }))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/auth/register",
    tag = "authentication",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = User),
        (status = 400, description = "Bad request - invalid input"),
        (status = 409, description = "Email already registered"),
        (status = 429, description = "Rate limit exceeded")
    )
)]
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<User>, AppError> {
    validate_email(&req.email)?;
    validate_string("password", &req.password, 128)?;

    let password_hash = crate::services::UserService::hash_password(&req.password).await?;

    let new_user = NewUser {
        email: req.email,
        password_hash,
        stellar_address: req.stellar_address,
        role: req.role,
    };

    let user = state.user_service.create_user(new_user).await?;
    Ok(Json(user))
}

fn spawn_login_audit(
    state: AppState,
    headers: HeaderMap,
    user_id: Option<Uuid>,
    email: String,
    success: bool,
) {
    let event = NewAuditEvent {
        correlation_id: correlation_id_from_headers(&headers),
        user_id,
        actor_api_key_id: None,
        event_category: if success { AuditEventCategory::AuthEvent } else { AuditEventCategory::SecurityEvent },
        event_type: "login".to_string(),
        severity: if success { AuditSeverity::Info } else { AuditSeverity::Warn },
        action: "admin_login".to_string(),
        resource_type: Some("auth".to_string()),
        target_resource_id: user_id.map(|id| id.to_string()),
        http_method: Some("POST".to_string()),
        http_path: Some("/api/v1/admin/auth/login".to_string()),
        http_status: Some(if success { 200 } else { 401 }),
        success,
        error_code: if success { None } else { Some("UNAUTHORIZED".to_string()) },
        business_context: None,
        changes: serde_json::json!({ "email_hash": hash_audit_email(&email) }),
        ip_address: client_ip_from_headers(&headers),
        user_agent: user_agent_from_headers(&headers),
    };

    tokio::spawn(async move {
        let correlation_id = event.correlation_id.clone();
        if let Err(err) = state.audit_service.log(event).await {
            tracing::warn!(
                correlation_id = correlation_id.as_deref().unwrap_or("unknown"),
                error = ?err,
                "Failed to persist login audit event"
            );
        }
    });
}

fn hash_audit_email(email: &str) -> String {
    let normalized = email.trim().to_ascii_lowercase();
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    hex::encode(hasher.finalize())
}
