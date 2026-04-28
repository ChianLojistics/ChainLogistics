use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::carbon::{
        CalculateFootprintRequest, CreateTradeRequest, GenerateCreditRequest,
        GenerateReportRequest, ListCreditsQuery, ListTradesQuery, PurchaseCreditRequest,
        RequestVerificationRequest, RetireCreditRequest,
    },
    AppState,
};

// ── Footprint ─────────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/v1/carbon/footprint/calculate",
    tag = "carbon",
    request_body = CalculateFootprintRequest,
    responses(
        (status = 201, description = "Carbon footprint calculated successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("jwt" = [])
    )
)]
/// POST /api/v1/carbon/footprint/calculate
pub async fn calculate_footprint(
    State(state): State<AppState>,
    Json(req): Json<CalculateFootprintRequest>,
) -> Result<impl IntoResponse, AppError> {
    let record = state.carbon_service.calculate_footprint(&req).await?;
    Ok((StatusCode::CREATED, Json(record)))
}

#[utoipa::path(
    post,
    path = "/api/v1/carbon/footprint/preview",
    tag = "carbon",
    request_body = CalculateFootprintRequest,
    responses(
        (status = 200, description = "Footprint preview generated successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("jwt" = [])
    )
)]
/// POST /api/v1/carbon/footprint/preview
pub async fn preview_footprint(
    State(state): State<AppState>,
    Json(req): Json<CalculateFootprintRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let breakdown = state.carbon_service.preview_footprint(&req);
    Ok(Json(serde_json::json!(breakdown)))
}

#[utoipa::path(
    get,
    path = "/api/v1/carbon/footprint/{product_id}",
    tag = "carbon",
    params(
        ("product_id" = String, Path, description = "Product ID")
    ),
    responses(
        (status = 200, description = "Footprints listed successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("jwt" = [])
    )
)]
/// GET /api/v1/carbon/footprint/:product_id
pub async fn list_footprints(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let records = state.carbon_service.list_footprints(&product_id).await?;
    Ok(Json(serde_json::json!({ "footprints": records, "total": records.len() })))
}

// ── Credits ───────────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/v1/carbon/credits/generate",
    tag = "carbon",
    request_body = GenerateCreditRequest,
    responses(
        (status = 201, description = "Carbon credit generated successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("jwt" = [])
    )
)]
/// POST /api/v1/carbon/credits/generate
pub async fn generate_credit(
    State(state): State<AppState>,
    Json(req): Json<GenerateCreditRequest>,
) -> Result<impl IntoResponse, AppError> {
    // TODO: extract real owner_id from auth context
    let owner_id = Uuid::nil();
    let credit = state.carbon_service.generate_credit(owner_id, &req).await?;
    Ok((StatusCode::CREATED, Json(credit)))
}

#[utoipa::path(
    get,
    path = "/api/v1/carbon/credits",
    tag = "carbon",
    params(ListCreditsQuery),
    responses(
        (status = 200, description = "Credits listed successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("jwt" = [])
    )
)]
/// GET /api/v1/carbon/credits
pub async fn list_credits(
    State(state): State<AppState>,
    Query(query): Query<ListCreditsQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    // TODO: extract real owner_id from auth context
    let owner_id = Uuid::nil();
    let credits = state.carbon_service.list_credits(owner_id, &query).await?;
    Ok(Json(serde_json::json!({ "credits": credits, "total": credits.len() })))
}

#[utoipa::path(
    get,
    path = "/api/v1/carbon/credits/{id}",
    tag = "carbon",
    params(
        ("id" = Uuid, Path, description = "Credit ID")
    ),
    responses(
        (status = 200, description = "Credit retrieved successfully"),
        (status = 404, description = "Credit not found"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("jwt" = [])
    )
)]
/// GET /api/v1/carbon/credits/:id
pub async fn get_credit(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let credit = state.carbon_service.get_credit(id).await?;
    Ok(Json(serde_json::json!(credit)))
}

#[utoipa::path(
    post,
    path = "/api/v1/carbon/credits/retire",
    tag = "carbon",
    request_body = RetireCreditRequest,
    responses(
        (status = 200, description = "Credit retired successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("jwt" = [])
    )
)]
/// POST /api/v1/carbon/credits/retire
pub async fn retire_credit(
    State(state): State<AppState>,
    Json(req): Json<RetireCreditRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // TODO: extract real owner_id from auth context
    let owner_id = Uuid::nil();
    let credit = state.carbon_service.retire_credit(owner_id, &req).await?;
    Ok(Json(serde_json::json!(credit)))
}

// ── Marketplace ───────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/carbon/market",
    tag = "carbon",
    responses(
        (status = 200, description = "Market summary retrieved successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("jwt" = [])
    )
)]
/// GET /api/v1/carbon/market
pub async fn market_summary(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let summary = state.carbon_service.get_market_summary().await?;
    Ok(Json(serde_json::json!(summary)))
}

#[utoipa::path(
    get,
    path = "/api/v1/carbon/market/trades",
    tag = "carbon",
    params(ListTradesQuery),
    responses(
        (status = 200, description = "Trades listed successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("jwt" = [])
    )
)]
/// GET /api/v1/carbon/market/trades
pub async fn list_trades(
    State(state): State<AppState>,
    Query(query): Query<ListTradesQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let trades = state.carbon_service.list_marketplace(&query).await?;
    Ok(Json(serde_json::json!({ "trades": trades, "total": trades.len() })))
}

#[utoipa::path(
    post,
    path = "/api/v1/carbon/market/list",
    tag = "carbon",
    request_body = CreateTradeRequest,
    responses(
        (status = 201, description = "Credit listed for sale successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("jwt" = [])
    )
)]
/// POST /api/v1/carbon/market/list
pub async fn list_credit_for_sale(
    State(state): State<AppState>,
    Json(req): Json<CreateTradeRequest>,
) -> Result<impl IntoResponse, AppError> {
    // TODO: extract real seller_id from auth context
    let seller_id = Uuid::nil();
    let trade = state.carbon_service.create_trade(seller_id, &req).await?;
    Ok((StatusCode::CREATED, Json(trade)))
}

#[utoipa::path(
    post,
    path = "/api/v1/carbon/market/purchase",
    tag = "carbon",
    request_body = PurchaseCreditRequest,
    responses(
        (status = 200, description = "Credit purchased successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("jwt" = [])
    )
)]
/// POST /api/v1/carbon/market/purchase
pub async fn purchase_credit(
    State(state): State<AppState>,
    Json(req): Json<PurchaseCreditRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // TODO: extract real buyer_id from auth context
    let buyer_id = Uuid::nil();
    let trade = state.carbon_service.purchase_credit(buyer_id, &req).await?;
    Ok(Json(serde_json::json!(trade)))
}

// ── Verification ──────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/v1/carbon/verify",
    tag = "carbon",
    request_body = RequestVerificationRequest,
    responses(
        (status = 201, description = "Verification requested successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("jwt" = [])
    )
)]
/// POST /api/v1/carbon/verify
pub async fn request_verification(
    State(state): State<AppState>,
    Json(req): Json<RequestVerificationRequest>,
) -> Result<impl IntoResponse, AppError> {
    // TODO: extract real requester_id from auth context
    let requester_id = Uuid::nil();
    let verification = state
        .carbon_service
        .request_verification(requester_id, &req)
        .await?;
    Ok((StatusCode::CREATED, Json(verification)))
}

#[utoipa::path(
    get,
    path = "/api/v1/carbon/verify/{credit_id}",
    tag = "carbon",
    params(
        ("credit_id" = Uuid, Path, description = "Credit ID")
    ),
    responses(
        (status = 200, description = "Verifications listed successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("jwt" = [])
    )
)]
/// GET /api/v1/carbon/verify/:credit_id
pub async fn list_verifications(
    State(state): State<AppState>,
    Path(credit_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let verifications = state.carbon_service.list_verifications(credit_id).await?;
    Ok(Json(
        serde_json::json!({ "verifications": verifications, "total": verifications.len() }),
    ))
}

// ── Reporting ─────────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/v1/carbon/reports",
    tag = "carbon",
    request_body = GenerateReportRequest,
    responses(
        (status = 201, description = "Report generated successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("jwt" = [])
    )
)]
/// POST /api/v1/carbon/reports
pub async fn generate_report(
    State(state): State<AppState>,
    Json(req): Json<GenerateReportRequest>,
) -> Result<impl IntoResponse, AppError> {
    // TODO: extract real owner_id from auth context
    let owner_id = Uuid::nil();
    let report = state.carbon_service.generate_report(owner_id, &req).await?;
    Ok((StatusCode::CREATED, Json(report)))
}

#[utoipa::path(
    get,
    path = "/api/v1/carbon/reports",
    tag = "carbon",
    responses(
        (status = 200, description = "Reports listed successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limit exceeded")
    ),
    security(
        ("jwt" = [])
    )
)]
/// GET /api/v1/carbon/reports
pub async fn list_reports(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    // TODO: extract real owner_id from auth context
    let owner_id = Uuid::nil();
    let reports = state.carbon_service.list_reports(owner_id).await?;
    Ok(Json(serde_json::json!({ "reports": reports, "total": reports.len() })))
}
