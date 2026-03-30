use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use crate::AppState;

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

pub async fn create_transaction(
    State(state): State<AppState>,
    Json(req): Json<CreateTransactionRequest>,
) -> impl IntoResponse {
    // TODO: Extract user_id from auth context
    let user_id = "user_id";

    match state.financial_service.create_transaction(
        user_id,
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
) -> impl IntoResponse {
    // TODO: Extract user_id from auth context
    let user_id = "user_id";

    match state.financial_service.list_user_transactions(user_id).await {
        Ok(txs) => (StatusCode::OK, Json(txs)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e}))).into_response(),
    }
}

pub async fn create_invoice(
    State(state): State<AppState>,
    Json(req): Json<CreateInvoiceRequest>,
) -> impl IntoResponse {
    // TODO: Extract user_id from auth context
    let user_id = "user_id";

    match state.financial_service.create_invoice(user_id, &req.amount, &req.due_date).await {
        Ok(invoice) => (StatusCode::CREATED, Json(invoice)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e}))).into_response(),
    }
}

pub async fn request_financing(
    State(state): State<AppState>,
    Json(req): Json<FinancingRequestBody>,
) -> impl IntoResponse {
    // TODO: Extract user_id from auth context
    let user_id = "user_id";

    match state.financial_service.request_financing(user_id, &req.financing_type, &req.amount).await {
        Ok(financing) => (StatusCode::CREATED, Json(financing)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e}))).into_response(),
    }
}
