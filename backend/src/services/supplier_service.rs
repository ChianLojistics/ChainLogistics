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
        sqlx::query_as::<Supplier, _>(
            r#"
            INSERT INTO suppliers (
                supplier_id, name, legal_name, tax_id, registration_number,
                business_type, tier, contact_email, contact_phone, address,
                city, country, postal_code, website, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING
                id, supplier_id, name, legal_name, tax_id, registration_number,
                business_type, tier, contact_email, contact_phone, address,
                city, country, postal_code, website, metadata, is_verified,
                verification_status, verification_date, verified_by, risk_level,
                created_at, updated_at
            "#,
        )
        .bind(supplier.supplier_id)
        .bind(supplier.name)
        .bind(supplier.legal_name)
        .bind(supplier.tax_id)
        .bind(supplier.registration_number)
        .bind(supplier.business_type)
        .bind(supplier.tier)
        .bind(supplier.contact_email)
        .bind(supplier.contact_phone)
        .bind(supplier.address)
        .bind(supplier.city)
        .bind(supplier.country)
        .bind(supplier.postal_code)
        .bind(supplier.website)
        .bind(supplier.metadata.unwrap_or(serde_json::json!({})))
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_supplier(&self, supplier_id: &str) -> Result<Option<Supplier>, sqlx::Error> {
        sqlx::query_as::<Supplier, _>(
            "SELECT id, supplier_id, name, legal_name, tax_id, registration_number, business_type, tier, contact_email, contact_phone, address, city, country, postal_code, website, metadata, is_verified, verification_status, verification_date, verified_by, risk_level, created_at, updated_at FROM suppliers WHERE supplier_id = $1",
        )
        .bind(supplier_id)
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
        let mut query = "SELECT id, supplier_id, name, legal_name, tax_id, registration_number, business_type, tier, contact_email, contact_phone, address, city, country, postal_code, website, metadata, is_verified, verification_status, verification_date, verified_by, risk_level, created_at, updated_at FROM suppliers WHERE 1=1".to_string();
        let mut bindings = Vec::new();
        let mut bind_index = 1;

        if let Some(bt) = business_type {
            query.push_str(&format!(" AND business_type = ${}", bind_index));
            bindings.push(bt);
            bind_index += 1;
        }
        if let Some(t) = tier {
            query.push_str(&format!(" AND tier = ${}", bind_index));
            bindings.push(t);
            bind_index += 1;
        }
        if let Some(vs) = verification_status {
            query.push_str(&format!(" AND verification_status = ${}", bind_index));
            bindings.push(vs);
            bind_index += 1;
        }
        if let Some(iv) = is_verified {
            query.push_str(&format!(" AND is_verified = ${}", bind_index));
            bindings.push(iv.to_string());
            bind_index += 1;
        }

        query.push_str(&format!(" ORDER BY created_at DESC LIMIT ${}", bind_index));
        bindings.push(limit.to_string());

        let mut q = sqlx::QueryBuilder::new(query);
        for binding in bindings {
            q = q.bind(binding);
        }

        q.build_query_as::<Supplier>()
            .fetch_all(&self.pool)
            .await
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

        sqlx::query_as::<Supplier, _>(
            r#"
            UPDATE suppliers SET
                verification_status = $2,
                is_verified = $3,
                verification_date = $4,
                verified_by = $5
            WHERE supplier_id = $1
            RETURNING
                id, supplier_id, name, legal_name, tax_id, registration_number,
                business_type, tier, contact_email, contact_phone, address,
                city, country, postal_code, website, metadata, is_verified,
                verification_status, verification_date, verified_by, risk_level,
                created_at, updated_at
            "#,
        )
        .bind(supplier_id)
        .bind(verification_status)
        .bind(is_verified)
        .bind(verification_date)
        .bind(verified_by)
        .fetch_one(&self.pool)
        .await
    }

    // Supplier Ratings
    pub async fn create_rating(&self, rating: NewSupplierRating) -> Result<SupplierRating, sqlx::Error> {
        sqlx::query_as::<SupplierRating, _>(
            r#"
            INSERT INTO supplier_ratings (
                supplier_id, rater_id, rating_type, score, comment,
                rating_period_start, rating_period_end
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, supplier_id, rater_id, rating_type, score, comment, rating_period_start, rating_period_end, created_at
            "#,
        )
        .bind(rating.supplier_id)
        .bind(rating.rater_id)
        .bind(rating.rating_type)
        .bind(rating.score)
        .bind(rating.comment)
        .bind(rating.rating_period_start)
        .bind(rating.rating_period_end)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_ratings(&self, supplier_id: &str, limit: i64) -> Result<Vec<SupplierRating>, sqlx::Error> {
        sqlx::query_as::<SupplierRating, _>(
            "SELECT id, supplier_id, rater_id, rating_type, score, comment, rating_period_start, rating_period_end, created_at FROM supplier_ratings WHERE supplier_id = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(supplier_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_average_rating(&self, supplier_id: &str, rating_type: Option<String>) -> Result<Option<Decimal>, sqlx::Error> {
        if let Some(rt) = rating_type {
            sqlx::query_scalar::<_, Decimal>(
                "SELECT AVG(score) FROM supplier_ratings WHERE supplier_id = $1 AND rating_type = $2",
            )
            .bind(supplier_id)
            .bind(rt)
            .fetch_one(&self.pool)
            .await
        } else {
            sqlx::query_scalar::<_, Decimal>(
                "SELECT AVG(score) FROM supplier_ratings WHERE supplier_id = $1",
            )
            .bind(supplier_id)
            .fetch_one(&self.pool)
            .await
        }
    }

    // Supplier Performance
    pub async fn create_performance(&self, perf: NewSupplierPerformance) -> Result<SupplierPerformance, sqlx::Error> {
        sqlx::query_as::<SupplierPerformance, _>(
            r#"
            INSERT INTO supplier_performance (
                supplier_id, metric_type, metric_value, unit,
                measurement_period_start, measurement_period_end,
                target_value, benchmark_value
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, supplier_id, metric_type, metric_value, unit, measurement_period_start, measurement_period_end, target_value, benchmark_value, created_at
            "#,
        )
        .bind(perf.supplier_id)
        .bind(perf.metric_type)
        .bind(perf.metric_value)
        .bind(perf.unit)
        .bind(perf.measurement_period_start)
        .bind(perf.measurement_period_end)
        .bind(perf.target_value)
        .bind(perf.benchmark_value)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_performance(&self, supplier_id: &str, limit: i64) -> Result<Vec<SupplierPerformance>, sqlx::Error> {
        sqlx::query_as::<SupplierPerformance, _>(
            "SELECT id, supplier_id, metric_type, metric_value, unit, measurement_period_start, measurement_period_end, target_value, benchmark_value, created_at FROM supplier_performance WHERE supplier_id = $1 ORDER BY measurement_period_start DESC LIMIT $2",
        )
        .bind(supplier_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    // Supplier Compliance
    pub async fn create_compliance(&self, compliance: NewSupplierCompliance) -> Result<SupplierCompliance, sqlx::Error> {
        sqlx::query_as::<SupplierCompliance, _>(
            r#"
            INSERT INTO supplier_compliance (
                supplier_id, compliance_type, certificate_number, issuing_authority,
                issue_date, expiry_date, document_url, verification_notes
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, supplier_id, compliance_type, certificate_number, issuing_authority, issue_date, expiry_date, status, document_url, verified_by, verified_at, verification_notes, created_at, updated_at
            "#,
        )
        .bind(compliance.supplier_id)
        .bind(compliance.compliance_type)
        .bind(compliance.certificate_number)
        .bind(compliance.issuing_authority)
        .bind(compliance.issue_date)
        .bind(compliance.expiry_date)
        .bind(compliance.document_url)
        .bind(compliance.verification_notes)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn verify_compliance(
        &self,
        compliance_id: Uuid,
        verified_by: String,
        status: String,
    ) -> Result<SupplierCompliance, sqlx::Error> {
        sqlx::query_as::<SupplierCompliance, _>(
            r#"
            UPDATE supplier_compliance SET
                status = $2,
                verified_by = $3,
                verified_at = NOW()
            WHERE id = $1
            RETURNING id, supplier_id, compliance_type, certificate_number, issuing_authority, issue_date, expiry_date, status, document_url, verified_by, verified_at, verification_notes, created_at, updated_at
            "#,
        )
        .bind(compliance_id)
        .bind(status)
        .bind(verified_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_compliance(&self, supplier_id: &str) -> Result<Vec<SupplierCompliance>, sqlx::Error> {
        sqlx::query_as::<SupplierCompliance, _>(
            "SELECT id, supplier_id, compliance_type, certificate_number, issuing_authority, issue_date, expiry_date, status, document_url, verified_by, verified_at, verification_notes, created_at, updated_at FROM supplier_compliance WHERE supplier_id = $1 ORDER BY created_at DESC",
        )
        .bind(supplier_id)
        .fetch_all(&self.pool)
        .await
    }

    // Supplier Products
    pub async fn add_supplier_product(&self, sp: NewSupplierProduct) -> Result<SupplierProduct, sqlx::Error> {
        sqlx::query_as::<SupplierProduct, _>(
            r#"
            INSERT INTO supplier_products (
                supplier_id, product_id, is_primary_supplier, supply_capacity,
                lead_time_days, unit_price, currency, min_order_quantity,
                contract_start_date, contract_end_date
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, supplier_id, product_id, is_primary_supplier, supply_capacity, lead_time_days, unit_price, currency, min_order_quantity, contract_start_date, contract_end_date, created_at, updated_at
            "#,
        )
        .bind(sp.supplier_id)
        .bind(sp.product_id)
        .bind(sp.is_primary_supplier)
        .bind(sp.supply_capacity)
        .bind(sp.lead_time_days)
        .bind(sp.unit_price)
        .bind(sp.currency)
        .bind(sp.min_order_quantity)
        .bind(sp.contract_start_date)
        .bind(sp.contract_end_date)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_supplier_products(&self, supplier_id: &str) -> Result<Vec<SupplierProduct>, sqlx::Error> {
        sqlx::query_as::<SupplierProduct, _>(
            "SELECT id, supplier_id, product_id, is_primary_supplier, supply_capacity, lead_time_days, unit_price, currency, min_order_quantity, contract_start_date, contract_end_date, created_at, updated_at FROM supplier_products WHERE supplier_id = $1",
        )
        .bind(supplier_id)
        .fetch_all(&self.pool)
        .await
    }

    // Supplier Summary
    pub async fn get_supplier_summary(&self, supplier_id: &str) -> Result<Option<SupplierSummary>, sqlx::Error> {
        let supplier = self.get_supplier(supplier_id).await?;
        
        if let Some(s) = supplier {
            let overall_rating = self.get_average_rating(supplier_id, None).await?;
            let total_ratings = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM supplier_ratings WHERE supplier_id = $1",
            )
            .bind(supplier_id)
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);
            
            let active_compliance_count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM supplier_compliance WHERE supplier_id = $1 AND status = 'active'",
            )
            .bind(supplier_id)
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);
            
            let total_products = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM supplier_products WHERE supplier_id = $1",
            )
            .bind(supplier_id)
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
        sqlx::query_as::<SupplierAuditTrail, _>(
            r#"
            INSERT INTO supplier_audit_trail (
                supplier_id, action_type, previous_value, new_value,
                performed_by, reason, ip_address
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, supplier_id, action_type, previous_value, new_value, performed_at, performed_by, reason, ip_address
            "#,
        )
        .bind(supplier_id)
        .bind(action_type)
        .bind(previous_value)
        .bind(new_value)
        .bind(performed_by)
        .bind(reason)
        .bind(ip_address)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_audit_trail(&self, supplier_id: &str, limit: i64) -> Result<Vec<SupplierAuditTrail>, sqlx::Error> {
        sqlx::query_as::<SupplierAuditTrail, _>(
            "SELECT id, supplier_id, action_type, previous_value, new_value, performed_at, performed_by, reason, ip_address FROM supplier_audit_trail WHERE supplier_id = $1 ORDER BY performed_at DESC LIMIT $2",
        )
        .bind(supplier_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }
}
