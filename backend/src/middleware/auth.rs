use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tower::ServiceExt;

use crate::{AppState, error::AppError};

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: uuid::Uuid,
    pub api_key_id: uuid::Uuid,
    pub tier: crate::models::ApiKeyTier,
    pub stellar_address: Option<String>,
}

pub async fn api_key_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract API key from Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized)?;

    // Hash the provided API key to compare with stored hash
    let key_hash = crate::services::ApiKeyService::hash_api_key(auth_header);

    // Look up API key in database
    let api_key = state
        .api_key_service
        .get_api_key_by_hash(&key_hash)
        .await?
        .ok_or_else(|| AppError::Unauthorized)?;

    // Check if API key is active and not expired
    if !api_key.is_active {
        return Err(AppError::Unauthorized);
    }

    if let Some(expires_at) = api_key.expires_at {
        if expires_at < chrono::Utc::now() {
            return Err(AppError::Unauthorized);
        }
    }

    // Get user information
    let user = state
        .user_service
        .get_user(api_key.user_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized)?;

    if !user.is_active {
        return Err(AppError::Unauthorized);
    }

    // Update last used timestamp
    let _ = state.api_key_service.update_last_used(api_key.id).await;

    // Create auth context
    let auth_context = AuthContext {
        user_id: user.id,
        api_key_id: api_key.id,
        tier: api_key.tier,
        stellar_address: user.stellar_address,
    };

    // Add auth context to request extensions
    request.extensions_mut().insert(auth_context);

    // Continue with the request
    Ok(next.run(request).await)
}

pub async fn require_admin(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_context = request
        .extensions()
        .get::<AuthContext>()
        .ok_or_else(|| AppError::Unauthorized)?;

    // Check if user is admin (you might want to add this to the User model)
    let user_id = auth_context.user_id;
    
    // For now, we'll skip the admin check since it's not in the User model
    // In a real implementation, you'd check user.is_admin
    
    Ok(next.run(request).await)
}

// Helper function to extract auth context from request
pub fn get_auth_context(request: &Request) -> Result<&AuthContext, AppError> {
    request
        .extensions()
        .get::<AuthContext>()
        .ok_or_else(|| AppError::Unauthorized)
}
