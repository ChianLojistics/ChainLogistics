use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::validation::{validate_amount, validate_string};
use crate::middleware::auth::AuthContext;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTransactionRequest {
    pub transaction_type: String,
    pub amount: String,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInvoiceRequest {
    pub amount: String,
    pub due_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinancingRequestBody {
    pub financing_type: String,
    pub amount: String,
}

/// Create a new financial transaction for the authenticated user.
///
/// This handler:
/// - Validates request fields (length/format)
/// - Extracts the authenticated user from request extensions (set by `api_key_auth` middleware)
/// - Delegates persistence/business rules to `FinancialService`
pub async fn create_transaction(
    State(state): State<AppState>,
    axum::Extension(auth): axum::Extension<AuthContext>,
    Json(req): Json<CreateTransactionRequest>,
) -> impl IntoResponse {
    if let Err(e) = validate_string("transaction_type", &req.transaction_type, 64)
        .and_then(|_| validate_string("currency", &req.currency, 10))
        .and_then(|_| validate_amount(&req.amount))
    {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response();
    }

    let user_id = auth.user_id.to_string();

    match state.financial_service.create_transaction(
        &user_id,
        &req.transaction_type,
        &req.amount,
        &req.currency,
    ).await {
        Ok(tx) => (StatusCode::CREATED, Json(tx)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e}))).into_response(),
    }
}

pub async fn get_transaction(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.financial_service.get_transaction(&id).await {
        Ok(tx) => (StatusCode::OK, Json(tx)).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Transaction not found"}))).into_response(),
    }
}

pub async fn list_transactions(
    State(state): State<AppState>,
    axum::Extension(auth): axum::Extension<AuthContext>,
) -> impl IntoResponse {
    let user_id = auth.user_id.to_string();

    match state.financial_service.list_user_transactions(&user_id).await {
        Ok(txs) => (StatusCode::OK, Json(txs)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e}))).into_response(),
    }
}

pub async fn create_invoice(
    State(state): State<AppState>,
    axum::Extension(auth): axum::Extension<AuthContext>,
    Json(req): Json<CreateInvoiceRequest>,
) -> impl IntoResponse {
    if let Err(e) = validate_amount(&req.amount)
        .and_then(|_| validate_string("due_date", &req.due_date, 32))
    {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response();
    }

    let user_id = auth.user_id.to_string();

    match state.financial_service.create_invoice(&user_id, &req.amount, &req.due_date).await {
        Ok(invoice) => (StatusCode::CREATED, Json(invoice)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e}))).into_response(),
    }
}

pub async fn request_financing(
    State(state): State<AppState>,
    axum::Extension(auth): axum::Extension<AuthContext>,
    Json(req): Json<FinancingRequestBody>,
) -> impl IntoResponse {
    if let Err(e) = validate_string("financing_type", &req.financing_type, 64)
        .and_then(|_| validate_amount(&req.amount))
    {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response();
    }

    let user_id = auth.user_id.to_string();

    match state.financial_service.request_financing(&user_id, &req.financing_type, &req.amount).await {
        Ok(financing) => (StatusCode::CREATED, Json(financing)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e}))).into_response(),
    }
}
