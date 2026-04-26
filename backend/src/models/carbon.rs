use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ── DB row types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CarbonFootprint {
    pub id: Uuid,
    pub product_id: String,
    pub tracking_event_id: Option<i64>,
    pub calculation_method: String,
    pub transport_emissions: f64,
    pub manufacturing_emissions: f64,
    pub packaging_emissions: f64,
    pub storage_emissions: f64,
    pub total_emissions: f64,
    pub baseline_emissions: Option<f64>,
    pub emissions_reduction: Option<f64>,
    pub reduction_percentage: Option<f64>,
    pub distance_km: Option<f64>,
    pub transport_mode: Option<String>,
    pub energy_source: Option<String>,
    pub raw_data: serde_json::Value,
    pub calculated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CarbonCredit {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub product_id: Option<String>,
    pub serial_number: String,
    pub vintage_year: i32,
    pub credit_type: String,
    pub standard: String,
    pub quantity: f64,
    pub price_per_tonne: Option<f64>,
    pub status: String,
    pub registry_id: Option<String>,
    pub registry_url: Option<String>,
    pub verification_body: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
    pub retired_at: Option<DateTime<Utc>>,
    pub retirement_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CarbonTrade {
    pub id: Uuid,
    pub credit_id: Uuid,
    pub seller_id: Uuid,
    pub buyer_id: Option<Uuid>,
    pub quantity: f64,
    pub price_per_tonne: f64,
    pub total_amount: f64,
    pub currency: String,
    pub status: String,
    pub trade_type: String,
    pub settlement_date: Option<DateTime<Utc>>,
    pub blockchain_tx_hash: Option<String>,
    pub platform_fee: Option<f64>,
    pub notes: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CarbonVerification {
    pub id: Uuid,
    pub credit_id: Uuid,
    pub requested_by: Uuid,
    pub verifier_name: String,
    pub verifier_accreditation: Option<String>,
    pub status: String,
    pub methodology: Option<String>,
    pub scope: Option<String>,
    pub findings: serde_json::Value,
    pub certificate_url: Option<String>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CarbonReport {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub report_type: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_emissions: f64,
    pub total_reductions: f64,
    pub net_emissions: f64,
    pub credits_generated: f64,
    pub credits_retired: f64,
    pub credits_sold: f64,
    pub revenue_from_credits: f64,
    pub summary: serde_json::Value,
    pub generated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// ── Request / input types ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct CalculateFootprintRequest {
    pub product_id: String,
    pub tracking_event_id: Option<i64>,
    pub transport_mode: Option<String>,
    pub distance_km: Option<f64>,
    pub energy_source: Option<String>,
    pub weight_kg: Option<f64>,
    pub packaging_type: Option<String>,
    pub storage_hours: Option<f64>,
    pub baseline_emissions: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenerateCreditRequest {
    pub product_id: Option<String>,
    pub footprint_id: Uuid,
    pub vintage_year: i32,
    pub credit_type: Option<String>,
    pub standard: Option<String>,
    pub price_per_tonne: Option<f64>,
    pub registry_id: Option<String>,
    pub verification_body: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTradeRequest {
    pub credit_id: Uuid,
    pub quantity: f64,
    pub price_per_tonne: f64,
    pub trade_type: Option<String>,
    pub notes: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PurchaseCreditRequest {
    pub trade_id: Uuid,
    pub quantity: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RetireCreditRequest {
    pub credit_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RequestVerificationRequest {
    pub credit_id: Uuid,
    pub verifier_name: String,
    pub verifier_accreditation: Option<String>,
    pub methodology: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenerateReportRequest {
    pub report_type: Option<String>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListTradesQuery {
    pub status: Option<String>,
    pub trade_type: Option<String>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListCreditsQuery {
    pub status: Option<String>,
    pub vintage_year: Option<i32>,
    pub standard: Option<String>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

// ── Calculation result ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootprintBreakdown {
    pub transport_emissions: f64,
    pub manufacturing_emissions: f64,
    pub packaging_emissions: f64,
    pub storage_emissions: f64,
    pub total_emissions: f64,
    pub emissions_reduction: Option<f64>,
    pub reduction_percentage: Option<f64>,
    pub eligible_credits: f64, // tonnes CO2e eligible for credit generation
}

// ── Market summary ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSummary {
    pub total_credits_available: f64,
    pub total_credits_listed: f64,
    pub total_credits_sold: f64,
    pub total_credits_retired: f64,
    pub avg_price_per_tonne: f64,
    pub total_market_volume_usd: f64,
    pub open_trades: i64,
    pub recent_trades: Vec<CarbonTrade>,
}
