use axum::{
    body::Body,
    extract::Request,
    http::{header::HeaderName, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;

use crate::middleware::audit::{correlation_id_from_headers, CORRELATION_ID_HEADER};
use crate::error::{ErrorCode, ErrorResponse};

/// Global error handler middleware
/// Catches any unhandled errors and panics, ensuring consistent error responses
pub async fn global_error_handler(
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let correlation_id = request
        .extensions()
        .get::<String>()
        .cloned()
        .or_else(|| correlation_id_from_headers(request.headers()))
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    
    // Add correlation ID to request extensions for tracking
    let mut request = request;
    request.extensions_mut().insert(correlation_id.clone());
    
    // Execute the request
    let response = next.run(request).await;
    
    // Check if the response is an error status
    let status = response.status();
    
    if status.is_client_error() || status.is_server_error() {
        // Log the error with correlation ID
        if status.is_server_error() {
            tracing::error!(
                correlation_id = %correlation_id,
                status = %status,
                "Server error occurred"
            );
        } else {
            tracing::debug!(
                correlation_id = %correlation_id,
                status = %status,
                "Client error occurred"
            );
        }
    }
    
    Ok(response)
}

/// Panic handler that converts panics to proper error responses
pub fn handle_panic(err: Box<dyn std::any::Any + Send + 'static>) -> Response {
    let correlation_id = Uuid::new_v4().to_string();
    
    let details = if let Some(s) = err.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = err.downcast_ref::<&str>() {
        s.to_string()
    } else {
        "Unknown panic".to_string()
    };

    tracing::error!(
        correlation_id = %correlation_id,
        panic_details = %details,
        "Panic occurred in request handler"
    );

    let mut response = ErrorResponse::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        ErrorCode::InternalServerError,
        "An unexpected error occurred. Please contact support if the problem persists.".to_string(),
    );
    response.correlation_id = correlation_id;

    (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
}

/// Request logging middleware with correlation ID
pub async fn request_logger(
    request: Request,
    next: Next,
) -> Response {
    let correlation_id = request
        .extensions()
        .get::<String>()
        .cloned()
        .or_else(|| correlation_id_from_headers(request.headers()))
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = std::time::Instant::now();
    let mut request = request;
    request.extensions_mut().insert(correlation_id.clone());
    
    tracing::info!(
        correlation_id = %correlation_id,
        method = %method,
        uri = %uri,
        "Request started"
    );
    
    let mut response = next.run(request).await;
    
    let duration = start.elapsed();
    let status = response.status();
    
    let log_level = if status.is_server_error() {
        tracing::Level::ERROR
    } else if status.is_client_error() {
        tracing::Level::WARN
    } else {
        tracing::Level::INFO
    };
    
    tracing::event!(
        log_level,
        correlation_id = %correlation_id,
        method = %method,
        uri = %uri,
        status = %status.as_u16(),
        duration_ms = %duration.as_millis(),
        "Request completed"
    );

    if let Ok(header_value) = HeaderValue::from_str(&correlation_id) {
        response.headers_mut().insert(
            HeaderName::from_static(CORRELATION_ID_HEADER),
            header_value,
        );
    }
    
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        response::IntoResponse,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn test_handler() -> impl IntoResponse {
        (StatusCode::OK, "OK")
    }

    async fn error_handler() -> Result<impl IntoResponse, crate::error::AppError> {
        Err(crate::error::AppError::Internal("Test error".to_string()))
    }

    #[tokio::test]
    async fn test_request_logger_success() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(request_logger));

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_request_logger_error() {
        let app = Router::new()
            .route("/error", get(error_handler))
            .layer(middleware::from_fn(request_logger));

        let response = app
            .oneshot(Request::builder().uri("/error").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert!(response.status().is_server_error());
    }
}
