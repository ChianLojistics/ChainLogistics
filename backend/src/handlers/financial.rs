use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::AppState;
use crate::validation::{validate_amount, validate_string, sanitize_input};
use crate::middleware::auth::AuthContext;
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateTransactionRequest {
    pub transaction_type: String,
    pub amount: String,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateInvoiceRequest {
    pub amount: String,
    pub due_date: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FinancingRequestBody {
    pub financing_type: String,
    pub amount: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/transactions",
    tag = "financial",
    request_body = CreateTransactionRequest,
    responses(
        (status = 201, description = "Transaction created successfully"),
        (status = 400, description = "Bad request - invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("jwt" = [])
    )
)]
/// Create a new financial transaction for the authenticated user.
pub async fn create_transaction(
    State(state): State<AppState>,
    axum::Extension(auth): axum::Extension<AuthContext>,
    Json(req): Json<CreateTransactionRequest>,
) -> Result<impl IntoResponse, AppError> {
    validate_string("transaction_type", &req.transaction_type, 64)?;
    validate_string("currency", &req.currency, 10)?;
    validate_amount(&req.amount)?;

    let user_id = auth.user_id.to_string();

    let tx = state.financial_service.create_transaction(
        &user_id,
        &sanitize_input(&req.transaction_type),
        &sanitize_input(&req.amount),
        &sanitize_input(&req.currency),
    ).await.map_err(AppError::DatabaseError)?;

    Ok((StatusCode::CREATED, Json(tx)))
}

#[utoipa::path(
    get,
    path = "/api/v1/transactions/{id}",
    tag = "financial",
    params(
        ("id" = String, Path, description = "Transaction ID")
    ),
    responses(
        (status = 200, description = "Transaction retrieved successfully"),
        (status = 404, description = "Transaction not found"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_transaction(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let tx = state.financial_service.get_transaction(&id).await
        .map_err(|_| AppError::NotFound("Transaction not found".to_string()))?;
    
    Ok((StatusCode::OK, Json(tx)))
}

#[utoipa::path(
    get,
    path = "/api/v1/transactions",
    tag = "financial",
    responses(
        (status = 200, description = "Transactions listed successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn list_transactions(
    State(state): State<AppState>,
    axum::Extension(auth): axum::Extension<AuthContext>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = auth.user_id.to_string();

    let txs = state.financial_service.list_user_transactions(&user_id).await
        .map_err(AppError::DatabaseError)?;
    
    Ok((StatusCode::OK, Json(txs)))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/invoices",
    tag = "financial",
    request_body = CreateInvoiceRequest,
    responses(
        (status = 201, description = "Invoice created successfully"),
        (status = 400, description = "Bad request - invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn create_invoice(
    State(state): State<AppState>,
    axum::Extension(auth): axum::Extension<AuthContext>,
    Json(req): Json<CreateInvoiceRequest>,
) -> Result<impl IntoResponse, AppError> {
    validate_amount(&req.amount)?;
    validate_string("due_date", &req.due_date, 32)?;

    let user_id = auth.user_id.to_string();

    let invoice = state.financial_service.create_invoice(
        &user_id, 
        &sanitize_input(&req.amount), 
        &sanitize_input(&req.due_date)
    ).await.map_err(AppError::DatabaseError)?;

    Ok((StatusCode::CREATED, Json(invoice)))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/financing/request",
    tag = "financial",
    request_body = FinancingRequestBody,
    responses(
        (status = 201, description = "Financing requested successfully"),
        (status = 400, description = "Bad request - invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn request_financing(
    State(state): State<AppState>,
    axum::Extension(auth): axum::Extension<AuthContext>,
    Json(req): Json<FinancingRequestBody>,
) -> Result<impl IntoResponse, AppError> {
    validate_string("financing_type", &req.financing_type, 64)?;
    validate_amount(&req.amount)?;

    let user_id = auth.user_id.to_string();

    let financing = state.financial_service.request_financing(
        &user_id, 
        &sanitize_input(&req.financing_type), 
        &sanitize_input(&req.amount)
    ).await.map_err(AppError::DatabaseError)?;

    Ok((StatusCode::CREATED, Json(financing)))
}
