use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use redis::AsyncCommands;
use std::sync::Arc;
use chrono::{Utc, Timelike};

use crate::{AppState, error::AppError, models::ApiKeyTier};

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Get auth context from request extensions
    let auth_context = request
        .extensions()
        .get::<crate::middleware::auth::AuthContext>()
        .ok_or_else(|| AppError::Unauthorized)?;

    // Get API key details to check rate limit
    let api_key = state
        .api_key_service
        .get_api_key(auth_context.api_key_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized)?;

    // Check rate limit based on tier
    let tier_limit = match api_key.tier {
        ApiKeyTier::Basic => 60,
        ApiKeyTier::Standard => 300,
        ApiKeyTier::Premium => 1000,
        ApiKeyTier::Enterprise => 5000,
    };

    // Use the custom rate limit if it's set and lower than the tier limit
    let effective_limit = if api_key.rate_limit_per_minute > 0 && api_key.rate_limit_per_minute < tier_limit {
        api_key.rate_limit_per_minute
    } else {
        tier_limit
    };

    // Redis rate limiting logic (Fixed Window)
    let mut conn = state.redis_client.get_multiplexed_tokio_connection().await
        .map_err(|e| {
            tracing::error!("Redis connection error: {}", e);
            AppError::Internal(e.to_string())
        })?;

    let now = Utc::now();
    let window_key = format!("ratelimit:{}:{}", auth_context.api_key_id, now.format("%Y%m%d%H%M"));
    
    let count: u32 = conn.incr(&window_key, 1).await
        .map_err(|e| {
            tracing::error!("Redis INCR error: {}", e);
            AppError::Internal(e.to_string())
        })?;

    if count == 1 {
        let _: () = conn.expire(&window_key, 60).await
            .map_err(|e| {
                tracing::error!("Redis EXPIRE error: {}", e);
                AppError::Internal(e.to_string())
            })?;
    }

    if count > effective_limit {
        return Err(AppError::RateLimit);
    }

    // Add rate limit headers to response
    let response = next.run(request).await;
    
    let (mut parts, body) = response.into_parts();
    parts.headers.insert(
        "X-RateLimit-Limit",
        effective_limit.to_string().parse().unwrap(),
    );
    parts.headers.insert(
        "X-RateLimit-Remaining",
        effective_limit.saturating_sub(count).to_string().parse().unwrap(),
    );
    parts.headers.insert(
        "X-RateLimit-Reset",
        ((now + chrono::Duration::minutes(1)).with_second(0).unwrap().with_nanosecond(0).unwrap())
            .timestamp()
            .to_string()
            .parse()
            .unwrap(),
    );

    Ok(Response::from_parts(parts, body))
}
