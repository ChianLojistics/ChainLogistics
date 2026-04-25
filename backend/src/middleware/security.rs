use axum::{
    extract::{Request, State},
    http::{header, HeaderValue, StatusCode, Uri},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{AppState, error::AppError};

/// Middleware to enforce HTTPS connections
/// Redirects HTTP requests to HTTPS in production
pub async fn enforce_https(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Skip enforcement if disabled (e.g., local development)
    if !state.config.security.enforce_https {
        return Ok(next.run(request).await);
    }

    // Check if request is already HTTPS
    let is_https = request
        .headers()
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "https")
        .unwrap_or_else(|| {
            // Check if TLS is enabled on the server
            state.config.server.tls_enabled
        });

    if !is_https {
        // Redirect to HTTPS
        let uri = request.uri();
        let host = request
            .headers()
            .get(header::HOST)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("localhost");

        let https_url = format!("https://{}{}", host, uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/"));

        return Ok((
            StatusCode::MOVED_PERMANENTLY,
            [(header::LOCATION, https_url)],
        )
            .into_response());
    }

    Ok(next.run(request).await)
}

/// Middleware to add security headers
/// Implements HSTS, CSP, and other security best practices
pub async fn security_headers(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // HTTP Strict Transport Security (HSTS)
    // Tells browsers to only use HTTPS for future requests
    if state.config.security.enforce_https {
        let hsts_value = format!(
            "max-age={}; includeSubDomains; preload",
            state.config.security.hsts_max_age
        );
        headers.insert(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_str(&hsts_value).unwrap(),
        );
    }

    // Content Security Policy (CSP)
    // Prevents XSS and other injection attacks
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline' 'unsafe-eval'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data: https:; \
             font-src 'self' data:; \
             connect-src 'self' https:; \
             frame-ancestors 'none';"
        ),
    );

    // X-Frame-Options
    // Prevents clickjacking attacks
    headers.insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    );

    // X-Content-Type-Options
    // Prevents MIME type sniffing
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );

    // X-XSS-Protection
    // Enables browser XSS protection (legacy browsers)
    headers.insert(
        HeaderValue::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block"),
    );

    // Referrer-Policy
    // Controls how much referrer information is sent
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Permissions-Policy (formerly Feature-Policy)
    // Controls which browser features can be used
    headers.insert(
        HeaderValue::from_static("permissions-policy"),
        HeaderValue::from_static(
            "geolocation=(), microphone=(), camera=(), payment=()"
        ),
    );

    Ok(response)
}

/// Middleware to validate and enforce CORS policies
pub async fn cors_policy(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let origin = request
        .headers()
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok());

    // Check if origin is allowed
    if let Some(origin_value) = origin {
        let is_allowed = state.config.security.allowed_origins.iter().any(|allowed| {
            // Support wildcard subdomains
            if allowed.starts_with("*.") {
                let domain = &allowed[2..];
                origin_value.ends_with(domain)
            } else {
                origin_value == allowed
            }
        });

        if !is_allowed {
            tracing::warn!("Blocked request from unauthorized origin: {}", origin_value);
            return Err(AppError::Forbidden);
        }
    }

    let mut response = next.run(request).await;

    // Add CORS headers for allowed origins
    if let Some(origin_value) = origin {
        let headers = response.headers_mut();
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            HeaderValue::from_str(origin_value).unwrap(),
        );
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
            HeaderValue::from_static("true"),
        );
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"),
        );
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            HeaderValue::from_static("Content-Type, Authorization, X-Requested-With"),
        );
        headers.insert(
            header::ACCESS_CONTROL_MAX_AGE,
            HeaderValue::from_static("86400"), // 24 hours
        );
    }

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;

    #[tokio::test]
    async fn test_security_headers_added() {
        // Test that security headers are properly added
        // This is a placeholder - full implementation would require test setup
    }

    #[tokio::test]
    async fn test_https_enforcement() {
        // Test that HTTP requests are redirected to HTTPS
        // This is a placeholder - full implementation would require test setup
    }

    #[tokio::test]
    async fn test_cors_validation() {
        // Test that CORS policies are enforced
        // This is a placeholder - full implementation would require test setup
    }
}
