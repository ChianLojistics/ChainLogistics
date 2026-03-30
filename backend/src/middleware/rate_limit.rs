use axum::{
    extract::{Request, State},
    http::{StatusCode},
    middleware::Next,
    response::Response,
};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tower::Layer;

use crate::{AppState, error::AppError, models::ApiKeyTier};

#[derive(Debug, Clone)]
struct RateLimitEntry {
    count: u32,
    window_start: Instant,
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    // In-memory rate limiting storage
    // In production, you'd want to use Redis for this
    limits: Arc<RwLock<HashMap<uuid::Uuid, RateLimitEntry>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            limits: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn check_rate_limit(
        &self,
        api_key_id: uuid::Uuid,
        tier: ApiKeyTier,
        rate_limit_per_minute: u32,
    ) -> Result<(), AppError> {
        let mut limits = self.limits.write().await;
        let now = Instant::now();
        let window_duration = Duration::from_secs(60);

        // Clean up old entries
        limits.retain(|_, entry| now.duration_since(entry.window_start) < window_duration);

        let entry = limits.entry(api_key_id).or_insert(RateLimitEntry {
            count: 0,
            window_start: now,
        });

        // Reset window if it's expired
        if now.duration_since(entry.window_start) >= window_duration {
            entry.count = 0;
            entry.window_start = now;
        }

        // Check rate limit
        if entry.count >= rate_limit_per_minute {
            return Err(AppError::RateLimit);
        }

        entry.count += 1;
        Ok(())
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

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
    let rate_limit_per_minute = match api_key.tier {
        ApiKeyTier::Basic => 60,
        ApiKeyTier::Standard => 300,
        ApiKeyTier::Premium => 1000,
        ApiKeyTier::Enterprise => 5000,
    };

    // Use the custom rate limit if it's set and lower than the tier limit
    let effective_limit = if api_key.rate_limit_per_minute < rate_limit_per_minute {
        api_key.rate_limit_per_minute
    } else {
        rate_limit_per_minute
    };

    // Check rate limit using in-memory limiter
    // In production, you'd inject this as a dependency
    static RATE_LIMITER: std::sync::OnceLock<RateLimiter> = std::sync::OnceLock::new();
    let limiter = RATE_LIMITER.get_or_init(RateLimiter::new);

    limiter
        .check_rate_limit(auth_context.api_key_id, api_key.tier, effective_limit)
        .await?;

    // Add rate limit headers to response
    let response = next.run(request).await;
    
    let (mut parts, body) = response.into_parts();
    parts.headers.insert(
        "X-RateLimit-Limit",
        effective_limit.to_string().parse().unwrap(),
    );
    parts.headers.insert(
        "X-RateLimit-Remaining",
        (effective_limit.saturating_sub(1)).to_string().parse().unwrap(),
    );
    parts.headers.insert(
        "X-RateLimit-Reset",
        (chrono::Utc::now() + chrono::Duration::minutes(1))
            .timestamp()
            .to_string()
            .parse()
            .unwrap(),
    );

    Ok(Response::from_parts(parts, body))
}

// Rate limiting layer for Tower
#[derive(Debug, Clone)]
pub struct RateLimitLayer {
    limiter: Arc<RateLimiter>,
}

impl RateLimitLayer {
    pub fn new() -> Self {
        Self {
            limiter: Arc::new(RateLimiter::new()),
        }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            limiter: self.limiter.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitService<S> {
    inner: S,
    limiter: Arc<RateLimiter>,
}

impl<S> RateLimitService<S>
where
    S: Clone,
{
    pub fn new(inner: S, limiter: Arc<RateLimiter>) -> Self {
        Self { inner, limiter }
    }
}

// Note: Full Tower service implementation would require more complex trait bounds
// For now, the middleware function approach is sufficient
