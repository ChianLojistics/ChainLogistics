use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use utoipa::ToSchema;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Supplier {
    pub id: Uuid,
    pub supplier_id: String,
    pub name: String,
    pub legal_name: Option<String>,
    pub tax_id: Option<String>,
    pub registration_number: Option<String>,
    pub business_type: String,
    pub tier: String,
    pub contact_email: String,
    pub contact_phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub country: String,
    pub postal_code: Option<String>,
    pub website: Option<String>,
    pub is_verified: bool,
    pub verification_status: String,
    pub verification_date: Option<DateTime<Utc>>,
    pub verified_by: Option<String>,
    pub certification_expiry: Option<DateTime<Utc>>,
    pub risk_level: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewSupplier {
    pub supplier_id: String,
    pub name: String,
    pub legal_name: Option<String>,
    pub tax_id: Option<String>,
    pub registration_number: Option<String>,
    pub business_type: String,
    pub tier: String,
    pub contact_email: String,
    pub contact_phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub country: String,
    pub postal_code: Option<String>,
    pub website: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct SupplierRating {
    pub id: Uuid,
    pub supplier_id: String,
    pub rater_id: String,
    pub rating_type: String,
    pub score: Decimal,
    pub comment: Option<String>,
    pub rating_period_start: Option<DateTime<Utc>>,
    pub rating_period_end: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewSupplierRating {
    pub supplier_id: String,
    pub rater_id: String,
    pub rating_type: String,
    pub score: Decimal,
    pub comment: Option<String>,
    pub rating_period_start: Option<DateTime<Utc>>,
    pub rating_period_end: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct SupplierPerformance {
    pub id: Uuid,
    pub supplier_id: String,
    pub metric_type: String,
    pub metric_value: Decimal,
    pub unit: Option<String>,
    pub measurement_period_start: DateTime<Utc>,
    pub measurement_period_end: DateTime<Utc>,
    pub target_value: Option<Decimal>,
    pub benchmark_value: Option<Decimal>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewSupplierPerformance {
    pub supplier_id: String,
    pub metric_type: String,
    pub metric_value: Decimal,
    pub unit: Option<String>,
    pub measurement_period_start: DateTime<Utc>,
    pub measurement_period_end: DateTime<Utc>,
    pub target_value: Option<Decimal>,
    pub benchmark_value: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct SupplierCompliance {
    pub id: Uuid,
    pub supplier_id: String,
    pub compliance_type: String,
    pub certificate_number: Option<String>,
    pub issuing_authority: Option<String>,
    pub issue_date: Option<DateTime<Utc>>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub status: String,
    pub document_url: Option<String>,
    pub verification_notes: Option<String>,
    pub verified_by: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewSupplierCompliance {
    pub supplier_id: String,
    pub compliance_type: String,
    pub certificate_number: Option<String>,
    pub issuing_authority: Option<String>,
    pub issue_date: Option<DateTime<Utc>>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub document_url: Option<String>,
    pub verification_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct SupplierProduct {
    pub id: Uuid,
    pub supplier_id: String,
    pub product_id: String,
    pub is_primary_supplier: bool,
    pub supply_capacity: Option<i32>,
    pub lead_time_days: Option<i32>,
    pub unit_price: Option<Decimal>,
    pub currency: String,
    pub min_order_quantity: Option<i32>,
    pub contract_start_date: Option<DateTime<Utc>>,
    pub contract_end_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewSupplierProduct {
    pub supplier_id: String,
    pub product_id: String,
    pub is_primary_supplier: bool,
    pub supply_capacity: Option<i32>,
    pub lead_time_days: Option<i32>,
    pub unit_price: Option<Decimal>,
    pub currency: String,
    pub min_order_quantity: Option<i32>,
    pub contract_start_date: Option<DateTime<Utc>>,
    pub contract_end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct SupplierAuditTrail {
    pub id: Uuid,
    pub supplier_id: String,
    pub action_type: String,
    pub previous_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub performed_by: String,
    pub performed_at: DateTime<Utc>,
    pub reason: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SupplierSummary {
    pub supplier_id: String,
    pub name: String,
    pub tier: String,
    pub verification_status: String,
    pub overall_rating: Option<Decimal>,
    pub total_ratings: i64,
    pub risk_level: String,
    pub active_compliance_count: i64,
    pub total_products: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VerificationRequest {
    pub supplier_id: String,
    pub verified_by: String,
    pub verification_status: String,
    pub notes: Option<String>,
}
