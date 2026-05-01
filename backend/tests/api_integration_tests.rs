use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use chainlojistic_backend::{AppState, routes};

// Test helper to create a test app
async fn create_test_app() -> axum::Router {
    // Note: In real tests, you'd use a test database
    // This is a simplified version for demonstration
    let app_state = AppState::new().await.expect("Failed to create app state");
    
    axum::Router::new()
        .merge(routes::health_routes())
        .merge(routes::api_routes())
        .with_state(app_state)
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_health_db_endpoint() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .uri("/health/db")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_products_unauthorized() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .uri("/api/v1/products")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    // Should require API key authentication
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_product_unauthorized() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .uri("/api/v1/products/test-product-id")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    // Should require API key authentication
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_list_events_unauthorized() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .uri("/api/v1/events")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    // Should require API key authentication
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_product_unauthorized() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/admin/products")
            .header("content-type", "application/json")
            .body(Body::from(r#"{
                "id": "test-product",
                "name": "Test Product",
                "description": "A test product",
                "origin_location": "Test Location",
                "category": "Test Category",
                "tags": [],
                "certifications": [],
                "media_hashes": [],
                "custom_fields": {}
            }"#))
            .unwrap())
        .await
        .unwrap();
    
    // Should require admin authentication
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_update_product_unauthorized() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .method("PUT")
            .uri("/api/v1/admin/products/test-product")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"name": "Updated Name"}"#))
            .unwrap())
        .await
        .unwrap();
    
    // Should require admin authentication
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_delete_product_unauthorized() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .method("DELETE")
            .uri("/api/v1/admin/products/test-product")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    // Should require admin authentication
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_event_unauthorized() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/admin/events")
            .header("content-type", "application/json")
            .body(Body::from(r#"{
                "product_id": "test-product",
                "actor_address": "GCFXHS5DRCQZ4QZ7Z4X7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7",
                "timestamp": "2024-01-01T00:00:00Z",
                "event_type": "TEST",
                "location": "Test Location",
                "data_hash": "test-hash",
                "note": "Test note",
                "metadata": {}
            }"#))
            .unwrap())
        .await
        .unwrap();
    
    // Should require admin authentication
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_stats_unauthorized() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .uri("/api/v1/stats")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    // Should require API key authentication
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_compliance_check_unauthorized() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/compliance/check")
            .header("content-type", "application/json")
            .body(Body::from(r#"{
                "product_id": "test-product",
                "check_type": "full"
            }"#))
            .unwrap())
        .await
        .unwrap();
    
    // Should require API key authentication
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_compliance_report_unauthorized() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .uri("/api/v1/compliance/report/test-product")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    // Should require API key authentication
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_generate_audit_report_unauthorized() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .uri("/api/v1/audit/report")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    // Should require API key authentication
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_list_transactions_unauthorized() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .uri("/api/v1/transactions")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    // Should require API key authentication
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_transaction_unauthorized() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .uri("/api/v1/transactions/test-transaction-id")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    // Should require API key authentication
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_transaction_unauthorized() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/admin/transactions")
            .header("content-type", "application/json")
            .body(Body::from(r#"{
                "product_id": "test-product",
                "amount": 100.00,
                "currency": "USD",
                "transaction_type": "payment"
            }"#))
            .unwrap())
        .await
        .unwrap();
    
    // Should require admin authentication
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_invoice_unauthorized() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/admin/invoices")
            .header("content-type", "application/json")
            .body(Body::from(r#"{
                "product_id": "test-product",
                "amount": 100.00,
                "currency": "USD"
            }"#))
            .unwrap())
        .await
        .unwrap();
    
    // Should require admin authentication
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_request_financing_unauthorized() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/admin/financing/request")
            .header("content-type", "application/json")
            .body(Body::from(r#"{
                "product_id": "test-product",
                "amount": 1000.00,
                "reason": "expansion"
            }"#))
            .unwrap())
        .await
        .unwrap();
    
    // Should require admin authentication
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_login() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/admin/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(r#"{
                "email": "test@example.com",
                "password": "password123"
            }"#))
            .unwrap())
        .await
        .unwrap();
    
    // Will return error since user doesn't exist, but endpoint should be accessible
    assert!(response.status() == StatusCode::UNAUTHORIZED || response.status() == StatusCode::OK);
}

#[tokio::test]
async fn test_auth_register() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/admin/auth/register")
            .header("content-type", "application/json")
            .body(Body::from(r#"{
                "email": "newuser@example.com",
                "password": "password123",
                "stellar_address": "GCFXHS5DRCQZ4QZ7Z4X7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7"
            }"#))
            .unwrap())
        .await
        .unwrap();
    
    // Endpoint should be accessible
    assert!(response.status() == StatusCode::CREATED || response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_user_unauthorized() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/admin/users")
            .header("content-type", "application/json")
            .body(Body::from(r#"{
                "email": "user@example.com",
                "password_hash": "hashedpassword",
                "stellar_address": "GCFXHS5DRCQZ4QZ7Z4X7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7"
            }"#))
            .unwrap())
        .await
        .unwrap();
    
    // Should require admin authentication
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_current_user_unauthorized() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .uri("/api/v1/admin/users/me")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    // Should require admin authentication
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// Edge case tests
#[tokio::test]
async fn test_invalid_json_body() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/admin/products")
            .header("content-type", "application/json")
            .body(Body::from(r#"invalid json"#))
            .unwrap())
        .await
        .unwrap();
    
    // Should return bad request for invalid JSON
    assert!(response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_missing_content_type() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/admin/products")
            .body(Body::from(r#"{}"#))
            .unwrap())
        .await
        .unwrap();
    
    // Should handle missing content type
    assert!(response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::UNAUTHORIZED || response.status() == StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn test_not_found_endpoint() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .uri("/api/v1/nonexistent-endpoint")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    // Should return not found
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_method_not_allowed() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .method("DELETE")
            .uri("/health")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    // Should return method not allowed
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}
