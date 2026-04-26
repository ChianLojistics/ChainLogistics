use axum::{
    extract::{Request, State, Extension},
    http::{header, StatusCode},
    middleware::Next,
    response::{Response, IntoResponse},
};
use std::sync::Arc;

use crate::{AppState, error::AppError, models::UserRole};
use crate::database::{UserRepository, ApiKeyRepository};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: uuid::Uuid,
    pub api_key_id: Option<uuid::Uuid>,
    pub tier: Option<crate::models::ApiKeyTier>,
    pub stellar_address: Option<String>,
    pub role: UserRole,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID
    pub exp: usize,
    pub role: UserRole,
}

pub async fn jwt_auth(
    Extension(state): Extension<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let auth_result = async {
        let auth_header = request
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| AppError::Unauthorized)?;

        let token_data = jsonwebtoken::decode::<Claims>(
            auth_header,
            &jsonwebtoken::DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
            &jsonwebtoken::Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized)?;

        let user_id = uuid::Uuid::parse_str(&token_data.claims.sub)
            .map_err(|_| AppError::Unauthorized)?;

        let user = state.user_service.get_user(user_id).await?
            .ok_or_else(|| AppError::Unauthorized)?;

        if !user.is_active {
            return Err(AppError::Unauthorized);
        }

        let auth_context = AuthContext {
            user_id: user.id,
            api_key_id: None,
            tier: None,
            stellar_address: user.stellar_address,
            role: user.role,
        };

        request.extensions_mut().insert(auth_context);
        Ok(())
    }.await;

    match auth_result {
        Ok(_) => next.run(request).await,
        Err(e) => e.into_response(),
    }
}

pub async fn api_key_auth(
    Extension(state): Extension<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let auth_result = async {
        let auth_header = request
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| AppError::Unauthorized)?;

        let key_hash = crate::services::ApiKeyService::hash_api_key(auth_header);

        let api_key = state
            .api_key_service
            .get_api_key_by_hash(&key_hash)
            .await?
            .ok_or_else(|| AppError::Unauthorized)?;

        if !api_key.is_active {
            return Err(AppError::Unauthorized);
        }

        if let Some(expires_at) = api_key.expires_at {
            if expires_at < chrono::Utc::now() {
                return Err(AppError::Unauthorized);
            }
        }

        let user = state
            .user_service
            .get_user(api_key.user_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized)?;

        if !user.is_active {
            return Err(AppError::Unauthorized);
        }

        let _ = state.api_key_service.update_last_used(api_key.id).await;

        let auth_context = AuthContext {
            user_id: user.id,
            api_key_id: Some(api_key.id),
            tier: Some(api_key.tier),
            stellar_address: user.stellar_address,
            role: user.role,
        };

        request.extensions_mut().insert(auth_context);
        Ok(())
    }.await;

    match auth_result {
        Ok(_) => next.run(request).await,
        Err(e) => e.into_response(),
    }
}

pub fn require_role(roles: Vec<UserRole>) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Clone {
    move |request: Request, next: Next| {
        let roles = roles.clone();
        Box::pin(async move {
            let res = match async {
                let auth_context = request
                    .extensions()
                    .get::<AuthContext>()
                    .ok_or_else(|| AppError::Unauthorized)?;

                if !roles.iter().any(|r| std::mem::discriminant(r) == std::mem::discriminant(&auth_context.role)) {
                    return Err(AppError::Forbidden("Insufficient permissions".to_string()));
                }

                Ok(next.run(request).await)
            }.await {
                Ok(r) => r,
                Err(e) => e.into_response(),
            };
            res
        })
    }
}

pub async fn require_admin(
    request: Request,
    next: Next,
) -> Response {
    let auth_context = request.extensions().get::<AuthContext>();
    
    match auth_context {
        Some(ctx) if ctx.role == UserRole::Administrator => next.run(request).await,
        Some(_) => AppError::Forbidden("Admin access required".to_string()).into_response(),
        None => AppError::Unauthorized.into_response(),
    }
}

pub fn get_auth_context(request: &Request) -> Result<&AuthContext, AppError> {
    request
        .extensions()
        .get::<AuthContext>()
        .ok_or_else(|| AppError::Unauthorized)
}
