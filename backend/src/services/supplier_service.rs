use sqlx::PgPool;
use uuid::Uuid;
use crate::models::supplier::*;
use rust_decimal::Decimal;

pub struct SupplierService {
    pool: PgPool,
}

impl SupplierService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Supplier Management
    pub async fn create_supplier(&self, supplier: NewSupplier) -> Result<Supplier, sqlx::Error> {
        sqlx::query_as!(
            Supplier,
            r#"
            INSERT INTO suppliers (
                supplier_id, name, legal_name, tax_id, registration_number,
                business_type, tier, contact_email, contact_phone, address,
                city, country, postal_code, website, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING *
            "#,
            supplier.supplier_id,
            supplier.name,
            supplier.legal_name,
            supplier.tax_id,
            supplier.registration_number,
            supplier.business_type,
            supplier.tier,
            supplier.contact_email,
            supplier.contact_phone,
            supplier.address,
            supplier.city,
            supplier.country,
            supplier.postal_code,
            supplier.website,
            supplier.metadata.unwrap_or(serde_json::json!({}))
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_supplier(&self, supplier_id: &str) -> Result<Option<Supplier>, sqlx::Error> {
        sqlx::query_as!(
            Supplier,
            "SELECT * FROM suppliers WHERE supplier_id = $1",
            supplier_id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_suppliers(
        &self,
        business_type: Option<String>,
        tier: Option<String>,
        verification_status: Option<String>,
        is_verified: Option<bool>,
        limit: i64,
    ) -> Result<Vec<Supplier>, sqlx::Error> {
        match (business_type, tier, verification_status, is_verified) {
            (Some(bt), Some(t), Some(vs), Some(iv)) => {
                sqlx::query_as!(
                    Supplier,
                    "SELECT * FROM suppliers WHERE business_type = $1 AND tier = $2 AND verification_status = $3 AND is_verified = $4 ORDER BY created_at DESC LIMIT $5",
                    bt, t, vs, iv, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(bt), Some(t), Some(vs), None) => {
                sqlx::query_as!(
                    Supplier,
                    "SELECT * FROM suppliers WHERE business_type = $1 AND tier = $2 AND verification_status = $3 ORDER BY created_at DESC LIMIT $4",
                    bt, t, vs, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(bt), Some(t), None, Some(iv)) => {
                sqlx::query_as!(
                    Supplier,
                    "SELECT * FROM suppliers WHERE business_type = $1 AND tier = $2 AND is_verified = $3 ORDER BY created_at DESC LIMIT $4",
                    bt, t, iv, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(bt), None, Some(vs), Some(iv)) => {
                sqlx::query_as!(
                    Supplier,
                    "SELECT * FROM suppliers WHERE business_type = $1 AND verification_status = $2 AND is_verified = $3 ORDER BY created_at DESC LIMIT $4",
                    bt, vs, iv, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(t), Some(vs), Some(iv)) => {
                sqlx::query_as!(
                    Supplier,
                    "SELECT * FROM suppliers WHERE tier = $1 AND verification_status = $2 AND is_verified = $3 ORDER BY created_at DESC LIMIT $4",
                    t, vs, iv, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(bt), Some(t), None, None) => {
                sqlx::query_as!(
                    Supplier,
                    "SELECT * FROM suppliers WHERE business_type = $1 AND tier = $2 ORDER BY created_at DESC LIMIT $3",
                    bt, t, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(bt), None, Some(vs), None) => {
                sqlx::query_as!(
                    Supplier,
                    "SELECT * FROM suppliers WHERE business_type = $1 AND verification_status = $2 ORDER BY created_at DESC LIMIT $3",
                    bt, vs, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(bt), None, None, Some(iv)) => {
                sqlx::query_as!(
                    Supplier,
                    "SELECT * FROM suppliers WHERE business_type = $1 AND is_verified = $2 ORDER BY created_at DESC LIMIT $3",
                    bt, iv, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(t), Some(vs), None) => {
                sqlx::query_as!(
                    Supplier,
                    "SELECT * FROM suppliers WHERE tier = $1 AND verification_status = $2 ORDER BY created_at DESC LIMIT $3",
                    t, vs, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(t), None, Some(iv)) => {
                sqlx::query_as!(
                    Supplier,
                    "SELECT * FROM suppliers WHERE tier = $1 AND is_verified = $2 ORDER BY created_at DESC LIMIT $3",
                    t, iv, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, None, Some(vs), Some(iv)) => {
                sqlx::query_as!(
                    Supplier,
                    "SELECT * FROM suppliers WHERE verification_status = $1 AND is_verified = $2 ORDER BY created_at DESC LIMIT $3",
                    vs, iv, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(bt), None, None, None) => {
                sqlx::query_as!(
                    Supplier,
                    "SELECT * FROM suppliers WHERE business_type = $1 ORDER BY created_at DESC LIMIT $2",
                    bt, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(t), None, None) => {
                sqlx::query_as!(
                    Supplier,
                    "SELECT * FROM suppliers WHERE tier = $1 ORDER BY created_at DESC LIMIT $2",
                    t, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, None, Some(vs), None) => {
                sqlx::query_as!(
                    Supplier,
                    "SELECT * FROM suppliers WHERE verification_status = $1 ORDER BY created_at DESC LIMIT $2",
                    vs, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, None, None, Some(iv)) => {
                sqlx::query_as!(
                    Supplier,
                    "SELECT * FROM suppliers WHERE is_verified = $1 ORDER BY created_at DESC LIMIT $2",
                    iv, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, None, None, None) => {
                sqlx::query_as!(
                    Supplier,
                    "SELECT * FROM suppliers ORDER BY created_at DESC LIMIT $1",
                    limit
                )
                .fetch_all(&self.pool)
                .await
            }
        }
    }

    pub async fn update_supplier_verification(
        &self,
        supplier_id: &str,
        verification_status: String,
        verified_by: String,
        notes: Option<String>,
    ) -> Result<Supplier, sqlx::Error> {
        let is_verified = verification_status == "verified";
        let verification_date = if is_verified {
            Some(chrono::Utc::now())
        } else {
            None
        };

        sqlx::query_as!(
            Supplier,
            r#"
            UPDATE suppliers SET
                verification_status = $2,
                is_verified = $3,
                verification_date = $4,
                verified_by = $5
            WHERE supplier_id = $1
            RETURNING *
            "#,
            supplier_id,
            verification_status,
            is_verified,
            verification_date,
            verified_by
        )
        .fetch_one(&self.pool)
        .await
    }

    // Supplier Ratings
    pub async fn create_rating(&self, rating: NewSupplierRating) -> Result<SupplierRating, sqlx::Error> {
        sqlx::query_as!(
            SupplierRating,
            r#"
            INSERT INTO supplier_ratings (
                supplier_id, rater_id, rating_type, score, comment,
                rating_period_start, rating_period_end
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
            rating.supplier_id,
            rating.rater_id,
            rating.rating_type,
            rating.score,
            rating.comment,
            rating.rating_period_start,
            rating.rating_period_end
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_ratings(&self, supplier_id: &str, limit: i64) -> Result<Vec<SupplierRating>, sqlx::Error> {
        sqlx::query_as!(
            SupplierRating,
            "SELECT * FROM supplier_ratings WHERE supplier_id = $1 ORDER BY created_at DESC LIMIT $2",
            supplier_id,
            limit
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_average_rating(&self, supplier_id: &str, rating_type: Option<String>) -> Result<Option<Decimal>, sqlx::Error> {
        let query = if let Some(rt) = rating_type {
            sqlx::query_scalar!(
                "SELECT AVG(score) FROM supplier_ratings WHERE supplier_id = $1 AND rating_type = $2",
                supplier_id,
                rt
            )
        } else {
            sqlx::query_scalar!(
                "SELECT AVG(score) FROM supplier_ratings WHERE supplier_id = $1",
                supplier_id
            )
        };

        query.fetch_one(&self.pool).await
    }

    // Supplier Performance
    pub async fn create_performance(&self, perf: NewSupplierPerformance) -> Result<SupplierPerformance, sqlx::Error> {
        sqlx::query_as!(
            SupplierPerformance,
            r#"
            INSERT INTO supplier_performance (
                supplier_id, metric_type, metric_value, unit,
                measurement_period_start, measurement_period_end,
                target_value, benchmark_value
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            perf.supplier_id,
            perf.metric_type,
            perf.metric_value,
            perf.unit,
            perf.measurement_period_start,
            perf.measurement_period_end,
            perf.target_value,
            perf.benchmark_value
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_performance(&self, supplier_id: &str, limit: i64) -> Result<Vec<SupplierPerformance>, sqlx::Error> {
        sqlx::query_as!(
            SupplierPerformance,
            "SELECT * FROM supplier_performance WHERE supplier_id = $1 ORDER BY measurement_period_start DESC LIMIT $2",
            supplier_id,
            limit
        )
        .fetch_all(&self.pool)
        .await
    }

    // Supplier Compliance
    pub async fn create_compliance(&self, compliance: NewSupplierCompliance) -> Result<SupplierCompliance, sqlx::Error> {
        sqlx::query_as!(
            SupplierCompliance,
            r#"
            INSERT INTO supplier_compliance (
                supplier_id, compliance_type, certificate_number, issuing_authority,
                issue_date, expiry_date, document_url, verification_notes
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            compliance.supplier_id,
            compliance.compliance_type,
            compliance.certificate_number,
            compliance.issuing_authority,
            compliance.issue_date,
            compliance.expiry_date,
            compliance.document_url,
            compliance.verification_notes
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn verify_compliance(
        &self,
        compliance_id: Uuid,
        verified_by: String,
        status: String,
    ) -> Result<SupplierCompliance, sqlx::Error> {
        sqlx::query_as!(
            SupplierCompliance,
            r#"
            UPDATE supplier_compliance SET
                status = $2,
                verified_by = $3,
                verified_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
            compliance_id,
            status,
            verified_by
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_compliance(&self, supplier_id: &str) -> Result<Vec<SupplierCompliance>, sqlx::Error> {
        sqlx::query_as!(
            SupplierCompliance,
            "SELECT * FROM supplier_compliance WHERE supplier_id = $1 ORDER BY created_at DESC",
            supplier_id
        )
        .fetch_all(&self.pool)
        .await
    }

    // Supplier Products
    pub async fn add_supplier_product(&self, sp: NewSupplierProduct) -> Result<SupplierProduct, sqlx::Error> {
        sqlx::query_as!(
            SupplierProduct,
            r#"
            INSERT INTO supplier_products (
                supplier_id, product_id, is_primary_supplier, supply_capacity,
                lead_time_days, unit_price, currency, min_order_quantity,
                contract_start_date, contract_end_date
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
            sp.supplier_id,
            sp.product_id,
            sp.is_primary_supplier,
            sp.supply_capacity,
            sp.lead_time_days,
            sp.unit_price,
            sp.currency,
            sp.min_order_quantity,
            sp.contract_start_date,
            sp.contract_end_date
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_supplier_products(&self, supplier_id: &str) -> Result<Vec<SupplierProduct>, sqlx::Error> {
        sqlx::query_as!(
            SupplierProduct,
            "SELECT * FROM supplier_products WHERE supplier_id = $1",
            supplier_id
        )
        .fetch_all(&self.pool)
        .await
    }

    // Supplier Summary
    pub async fn get_supplier_summary(&self, supplier_id: &str) -> Result<Option<SupplierSummary>, sqlx::Error> {
        let supplier = self.get_supplier(supplier_id).await?;
        
        if let Some(s) = supplier {
            let overall_rating = self.get_average_rating(supplier_id, None).await?;
            let total_ratings = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM supplier_ratings WHERE supplier_id = $1",
                supplier_id
            )
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);
            
            let active_compliance_count = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM supplier_compliance WHERE supplier_id = $1 AND status = 'active'",
                supplier_id
            )
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);
            
            let total_products = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM supplier_products WHERE supplier_id = $1",
                supplier_id
            )
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);

            Ok(Some(SupplierSummary {
                supplier_id: s.supplier_id,
                name: s.name,
                tier: s.tier,
                verification_status: s.verification_status,
                overall_rating,
                total_ratings,
                risk_level: s.risk_level,
                active_compliance_count,
                total_products,
            }))
        } else {
            Ok(None)
        }
    }

    // Audit Trail
    pub async fn create_audit_entry(
        &self,
        supplier_id: &str,
        action_type: String,
        previous_value: Option<serde_json::Value>,
        new_value: Option<serde_json::Value>,
        performed_by: String,
        reason: Option<String>,
        ip_address: Option<String>,
    ) -> Result<SupplierAuditTrail, sqlx::Error> {
        sqlx::query_as!(
            SupplierAuditTrail,
            r#"
            INSERT INTO supplier_audit_trail (
                supplier_id, action_type, previous_value, new_value,
                performed_by, reason, ip_address
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
            supplier_id,
            action_type,
            previous_value,
            new_value,
            performed_by,
            reason,
            ip_address
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_audit_trail(&self, supplier_id: &str, limit: i64) -> Result<Vec<SupplierAuditTrail>, sqlx::Error> {
        sqlx::query_as!(
            SupplierAuditTrail,
            "SELECT * FROM supplier_audit_trail WHERE supplier_id = $1 ORDER BY performed_at DESC LIMIT $2",
            supplier_id,
            limit
        )
        .fetch_all(&self.pool)
        .await
    }
}
