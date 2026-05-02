use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;

use crate::{
    AppState,
    error::AppError,
    models::{ApiKey, ApiKeyTier, NewApiKey},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub tier: Option<ApiKeyTier>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiKeyCreatedResponse {
    pub id: Uuid,
    pub name: String,
    pub tier: ApiKeyTier,
    pub key: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    pub tier: ApiKeyTier,
    pub is_active: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<ApiKey> for ApiKeyResponse {
    fn from(k: ApiKey) -> Self {
        Self {
            id: k.id,
            name: k.name,
            tier: k.tier,
            is_active: k.is_active,
            expires_at: k.expires_at,
            last_used_at: k.last_used_at,
            created_at: k.created_at,
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/keys",
    tag = "api_keys",
    request_body = CreateApiKeyRequest,
    responses(
        (status = 201, description = "API key created successfully", body = ApiKeyCreatedResponse),
        (status = 400, description = "Bad request - invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn create_key(
    State(state): State<AppState>,
    axum::Extension(auth): axum::Extension<crate::middleware::auth::AuthContext>,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<ApiKeyCreatedResponse>), AppError> {
    if req.name.trim().is_empty() {
        return Err(AppError::Validation("Key name must not be empty".into()));
    }

    let plaintext = crate::services::ApiKeyService::generate_api_key();
    let key_hash = crate::services::ApiKeyService::hash_api_key(&plaintext);

    let tier = req.tier.unwrap_or(ApiKeyTier::Basic);
    let rate_limit = match tier {
        ApiKeyTier::Basic => 60,
        ApiKeyTier::Standard => 300,
        ApiKeyTier::Premium => 1000,
        ApiKeyTier::Enterprise => 5000,
    };

    let new_key = NewApiKey {
        user_id: auth.user_id,
        key_hash,
        name: req.name,
        tier,
        rate_limit_per_minute: rate_limit,
        expires_at: req.expires_at,
    };

    let created = state.api_key_service.create_api_key(new_key).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiKeyCreatedResponse {
            id: created.id,
            name: created.name,
            tier: created.tier,
            key: plaintext,
            expires_at: created.expires_at,
            created_at: created.created_at,
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/keys",
    tag = "api_keys",
    responses(
        (status = 200, description = "API keys listed successfully", body = Vec<ApiKeyResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn list_keys(
    State(state): State<AppState>,
    axum::Extension(auth): axum::Extension<crate::middleware::auth::AuthContext>,
) -> Result<Json<Vec<ApiKeyResponse>>, AppError> {
    let keys = state
        .api_key_service
        .list_api_keys(auth.user_id)
        .await?
        .into_iter()
        .map(ApiKeyResponse::from)
        .collect();

    Ok(Json(keys))
}

#[utoipa::path(
    post,
    path = "/api/v1/keys/{id}/revoke",
    tag = "api_keys",
    params(
        ("id" = Uuid, Path, description = "API Key ID")
    ),
    responses(
        (status = 204, description = "API key revoked successfully"),
        (status = 404, description = "API key not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - cannot revoke another user's key"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn revoke_key(
    State(state): State<AppState>,
    axum::Extension(auth): axum::Extension<crate::middleware::auth::AuthContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let key = state
        .api_key_service
        .get_api_key(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("API key {} not found", id)))?;

    if key.user_id != auth.user_id {
        return Err(AppError::Forbidden("Cannot revoke another user's key".into()));
    }

    state.api_key_service.revoke_api_key(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/keys/{id}/rotate",
    tag = "api_keys",
    params(
        ("id" = Uuid, Path, description = "API Key ID")
    ),
    responses(
        (status = 201, description = "API key rotated successfully", body = ApiKeyCreatedResponse),
        (status = 404, description = "API key not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - cannot rotate another user's key"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn rotate_key(
    State(state): State<AppState>,
    axum::Extension(auth): axum::Extension<crate::middleware::auth::AuthContext>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiKeyCreatedResponse>), AppError> {
    let old_key = state
        .api_key_service
        .get_api_key(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("API key {} not found", id)))?;

    if old_key.user_id != auth.user_id {
        return Err(AppError::Forbidden("Cannot rotate another user's key".into()));
    }

    state.api_key_service.revoke_api_key(id).await?;

    let plaintext = crate::services::ApiKeyService::generate_api_key();
    let key_hash = crate::services::ApiKeyService::hash_api_key(&plaintext);

    let new_key = NewApiKey {
        user_id: auth.user_id,
        key_hash,
        name: format!("{} (rotated)", old_key.name),
        tier: old_key.tier,
        rate_limit_per_minute: old_key.rate_limit_per_minute,
        expires_at: old_key.expires_at,
    };

    let created = state.api_key_service.create_api_key(new_key).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiKeyCreatedResponse {
            id: created.id,
            name: created.name,
            tier: created.tier,
            key: plaintext,
            expires_at: created.expires_at,
            created_at: created.created_at,
        }),
    ))
}
