use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::carbon::{
    CalculateFootprintRequest, CarbonCredit, CarbonFootprint, CarbonReport, CarbonTrade,
    CarbonVerification, CreateTradeRequest, FootprintBreakdown, GenerateCreditRequest,
    GenerateReportRequest, ListCreditsQuery, ListTradesQuery, MarketSummary,
    PurchaseCreditRequest, RequestVerificationRequest, RetireCreditRequest,
};
use crate::services::carbon_calculator;

pub struct CarbonService {
    pool: PgPool,
}

impl CarbonService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ── Footprint ─────────────────────────────────────────────────────────────

    /// Calculate and persist a carbon footprint record for a product/event.
    pub async fn calculate_footprint(
        &self,
        req: &CalculateFootprintRequest,
    ) -> Result<CarbonFootprint, AppError> {
        let breakdown = carbon_calculator::calculate(req);

        let record = sqlx::query_as::<CarbonFootprint, _>(
            r#"
            INSERT INTO carbon_footprints (
                product_id, tracking_event_id, calculation_method,
                transport_emissions, manufacturing_emissions, packaging_emissions,
                storage_emissions, total_emissions,
                baseline_emissions, emissions_reduction, reduction_percentage,
                distance_km, transport_mode, energy_source, raw_data
            ) VALUES (
                $1, $2, 'ghg_protocol',
                $3, $4, $5, $6, $7,
                $8, $9, $10,
                $11, $12, $13, $14
            )
            RETURNING
                id, product_id, tracking_event_id, calculation_method,
                transport_emissions, manufacturing_emissions, packaging_emissions,
                storage_emissions, total_emissions,
                baseline_emissions, emissions_reduction, reduction_percentage,
                distance_km, transport_mode, energy_source, raw_data, calculated_at
            "#,
        )
        .bind(req.product_id.clone())
        .bind(req.tracking_event_id)
        .bind(breakdown.transport_emissions)
        .bind(breakdown.manufacturing_emissions)
        .bind(breakdown.packaging_emissions)
        .bind(breakdown.storage_emissions)
        .bind(breakdown.total_emissions)
        .bind(req.baseline_emissions)
        .bind(breakdown.emissions_reduction)
        .bind(breakdown.reduction_percentage)
        .bind(req.distance_km)
        .bind(req.transport_mode.clone())
        .bind(req.energy_source.clone())
        .bind(serde_json::to_value(req).unwrap_or_default())
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    /// Get all footprint records for a product.
    pub async fn list_footprints(
        &self,
        product_id: &str,
    ) -> Result<Vec<CarbonFootprint>, AppError> {
        let records = sqlx::query_as::<CarbonFootprint, _>(
            "SELECT id, product_id, tracking_event_id, calculation_method, transport_emissions, manufacturing_emissions, packaging_emissions, storage_emissions, total_emissions, baseline_emissions, emissions_reduction, reduction_percentage, distance_km, transport_mode, energy_source, raw_data, calculated_at FROM carbon_footprints WHERE product_id = $1 ORDER BY calculated_at DESC",
        )
        .bind(product_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(records)
    }

    /// Preview calculation without persisting.
    pub fn preview_footprint(&self, req: &CalculateFootprintRequest) -> FootprintBreakdown {
        carbon_calculator::calculate(req)
    }

    // ── Credits ───────────────────────────────────────────────────────────────

    /// Generate a carbon credit from a verified footprint reduction.
    pub async fn generate_credit(
        &self,
        owner_id: Uuid,
        req: &GenerateCreditRequest,
    ) -> Result<CarbonCredit, AppError> {
        // Fetch the footprint to validate eligible credits
        let footprint = sqlx::query_as::<CarbonFootprint, _>(
            "SELECT id, product_id, tracking_event_id, calculation_method, transport_emissions, manufacturing_emissions, packaging_emissions, storage_emissions, total_emissions, baseline_emissions, emissions_reduction, reduction_percentage, distance_km, transport_mode, energy_source, raw_data, calculated_at FROM carbon_footprints WHERE id = $1",
        )
        .bind(req.footprint_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Footprint record not found".into()))?;

        let breakdown = carbon_calculator::calculate(&CalculateFootprintRequest {
            product_id: footprint.product_id.clone(),
            tracking_event_id: footprint.tracking_event_id,
            transport_mode: footprint.transport_mode.clone(),
            distance_km: footprint.distance_km,
            energy_source: footprint.energy_source.clone(),
            weight_kg: None,
            packaging_type: None,
            storage_hours: None,
            baseline_emissions: footprint.baseline_emissions,
        });

        if breakdown.eligible_credits <= 0.0 {
            return Err(AppError::Validation(
                "Footprint does not meet minimum reduction threshold for credit generation".into(),
            ));
        }

        let serial = generate_serial_number(req.vintage_year);
        let credit_type = req.credit_type.as_deref().unwrap_or("verified_reduction");
        let standard = req.standard.as_deref().unwrap_or("GHG_PROTOCOL");

        let credit = sqlx::query_as::<CarbonCredit, _>(
            r#"
            INSERT INTO carbon_credits (
                owner_id, product_id, serial_number, vintage_year,
                credit_type, standard, quantity, price_per_tonne,
                status, registry_id, verification_body
            ) VALUES (
                $1, $2, $3, $4,
                $5, $6, $7, $8,
                'pending', $9, $10
            )
            RETURNING
                id, owner_id, product_id, serial_number, vintage_year,
                credit_type, standard, quantity, price_per_tonne,
                status, registry_id, verification_body, retired_at, retirement_reason, created_at, updated_at
            "#,
        )
        .bind(owner_id)
        .bind(req.product_id.clone())
        .bind(serial)
        .bind(req.vintage_year)
        .bind(credit_type)
        .bind(standard)
        .bind(breakdown.eligible_credits)
        .bind(req.price_per_tonne)
        .bind(req.registry_id.clone())
        .bind(req.verification_body.clone())
        .fetch_one(&self.pool)
        .await?;

        Ok(credit)
    }

    pub async fn get_credit(&self, id: Uuid) -> Result<CarbonCredit, AppError> {
        sqlx::query_as::<CarbonCredit, _>("SELECT id, owner_id, product_id, serial_number, vintage_year, credit_type, standard, quantity, price_per_tonne, status, registry_id, verification_body, retired_at, retirement_reason, created_at, updated_at FROM carbon_credits WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Credit {} not found", id)))
    }

    pub async fn list_credits(
        &self,
        owner_id: Uuid,
        query: &ListCreditsQuery,
    ) -> Result<Vec<CarbonCredit>, AppError> {
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(50).min(200);

        let records = sqlx::query_as::<CarbonCredit, _>(
            r#"
            SELECT id, owner_id, product_id, serial_number, vintage_year, credit_type, standard, quantity, price_per_tonne, status, registry_id, verification_body, retired_at, retirement_reason, created_at, updated_at FROM carbon_credits
            WHERE owner_id = $1
              AND ($2::TEXT IS NULL OR status = $2)
              AND ($3::INT IS NULL OR vintage_year = $3)
              AND ($4::TEXT IS NULL OR standard = $4)
            ORDER BY created_at DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(owner_id)
        .bind(query.status.clone())
        .bind(query.vintage_year)
        .bind(query.standard.clone())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// Retire a credit (permanently remove from circulation).
    pub async fn retire_credit(
        &self,
        owner_id: Uuid,
        req: &RetireCreditRequest,
    ) -> Result<CarbonCredit, AppError> {
        let credit = self.get_credit(req.credit_id).await?;

        if credit.owner_id != owner_id {
            return Err(AppError::Forbidden("You do not own this credit".into()));
        }
        if credit.status == "retired" {
            return Err(AppError::Validation("Credit is already retired".into()));
        }

        let updated = sqlx::query_as::<CarbonCredit, _>(
            r#"
            UPDATE carbon_credits
            SET status = 'retired', retired_at = NOW(), retirement_reason = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING
                id, owner_id, product_id, serial_number, vintage_year,
                credit_type, standard, quantity, price_per_tonne,
                status, registry_id, verification_body, retired_at, retirement_reason, created_at, updated_at
            "#,
        )
        .bind(req.credit_id)
        .bind(req.reason.clone())
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    // ── Marketplace ───────────────────────────────────────────────────────────

    /// List a credit for sale on the marketplace.
    pub async fn create_trade(
        &self,
        seller_id: Uuid,
        req: &CreateTradeRequest,
    ) -> Result<CarbonTrade, AppError> {
        let credit = self.get_credit(req.credit_id).await?;

        if credit.owner_id != seller_id {
            return Err(AppError::Forbidden("You do not own this credit".into()));
        }
        if !["verified", "pending"].contains(&credit.status.as_str()) {
            return Err(AppError::Validation(
                "Only verified or pending credits can be listed for trade".into(),
            ));
        }
        if req.quantity <= 0.0 || req.quantity > credit.quantity {
            return Err(AppError::Validation(
                "Trade quantity must be positive and not exceed credit quantity".into(),
            ));
        }

        let total_amount = req.quantity * req.price_per_tonne;
        let trade_type = req.trade_type.as_deref().unwrap_or("spot");

        let trade = sqlx::query_as::<CarbonTrade, _>(
            r#"
            INSERT INTO carbon_trades (
                credit_id, seller_id, quantity, price_per_tonne,
                total_amount, trade_type, status, notes, expires_at
            ) VALUES (
                $1, $2, $3, $4,
                $5, $6, 'open', $7, $8
            )
            RETURNING
                id, credit_id, seller_id, buyer_id, quantity, price_per_tonne,
                total_amount, platform_fee, status, trade_type,
                settlement_date, notes, expires_at, created_at, updated_at
            "#,
        )
        .bind(req.credit_id)
        .bind(seller_id)
        .bind(req.quantity)
        .bind(req.price_per_tonne)
        .bind(total_amount)
        .bind(trade_type)
        .bind(req.notes.clone())
        .bind(req.expires_at)
        .fetch_one(&self.pool)
        .await?;

        // Mark credit as listed
        sqlx::query("UPDATE carbon_credits SET status = 'listed', updated_at = NOW() WHERE id = $1")
            .bind(req.credit_id)
            .execute(&self.pool)
            .await?;

        Ok(trade)
    }

    /// Purchase credits from an open trade listing.
    pub async fn purchase_credit(
        &self,
        buyer_id: Uuid,
        req: &PurchaseCreditRequest,
    ) -> Result<CarbonTrade, AppError> {
        let trade = sqlx::query_as::<CarbonTrade, _>(
            "SELECT id, credit_id, seller_id, buyer_id, quantity, price_per_tonne, total_amount, platform_fee, status, trade_type, settlement_date, notes, expires_at, created_at, updated_at FROM carbon_trades WHERE id = $1",
        )
        .bind(req.trade_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Trade not found".into()))?;

        if trade.status != "open" {
            return Err(AppError::Validation("Trade is not open for purchase".into()));
        }
        if trade.seller_id == buyer_id {
            return Err(AppError::Validation("Cannot purchase your own listing".into()));
        }
        if req.quantity <= 0.0 || req.quantity > trade.quantity {
            return Err(AppError::Validation(
                "Purchase quantity must be positive and not exceed listed quantity".into(),
            ));
        }

        let total = req.quantity * trade.price_per_tonne;
        let platform_fee = total * 0.025; // 2.5% platform fee

        let settled = sqlx::query_as::<CarbonTrade, _>(
            r#"
            UPDATE carbon_trades
            SET buyer_id = $2, status = 'settled', quantity = $3,
                total_amount = $4, platform_fee = $5,
                settlement_date = NOW(), updated_at = NOW()
            WHERE id = $1
            RETURNING
                id, credit_id, seller_id, buyer_id, quantity, price_per_tonne,
                total_amount, platform_fee, status, trade_type,
                settlement_date, notes, expires_at, created_at, updated_at
            "#,
        )
        .bind(req.trade_id)
        .bind(buyer_id)
        .bind(req.quantity)
        .bind(total)
        .bind(platform_fee)
        .fetch_one(&self.pool)
        .await?;

        // Transfer credit ownership
        sqlx::query("UPDATE carbon_credits SET owner_id = $1, status = 'sold', updated_at = NOW() WHERE id = $2")
            .bind(buyer_id)
            .bind(trade.credit_id)
            .execute(&self.pool)
            .await?;

        Ok(settled)
    }

    pub async fn list_marketplace(
        &self,
        query: &ListTradesQuery,
    ) -> Result<Vec<CarbonTrade>, AppError> {
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(50).min(200);

        let trades = sqlx::query_as::<CarbonTrade, _>(
            r#"
            SELECT id, credit_id, seller_id, buyer_id, quantity, price_per_tonne, total_amount, platform_fee, status, trade_type, settlement_date, notes, expires_at, created_at, updated_at FROM carbon_trades
            WHERE ($1::TEXT IS NULL OR status = $1)
              AND ($2::TEXT IS NULL OR trade_type = $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(query.status.clone())
        .bind(query.trade_type.clone())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(trades)
    }

    pub async fn get_market_summary(&self) -> Result<MarketSummary, AppError> {
        let row = sqlx::query(
            r#"
            SELECT
                COALESCE(SUM(quantity) FILTER (WHERE status NOT IN ('retired','cancelled')), 0) AS total_available,
                COALESCE(SUM(quantity) FILTER (WHERE status = 'listed'), 0)                    AS total_listed,
                COALESCE(SUM(quantity) FILTER (WHERE status = 'sold'), 0)                      AS total_sold,
                COALESCE(SUM(quantity) FILTER (WHERE status = 'retired'), 0)                   AS total_retired
            FROM carbon_credits
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        let price_row = sqlx::query(
            r#"
            SELECT
                COALESCE(AVG(price_per_tonne), 0)   AS avg_price,
                COALESCE(SUM(total_amount), 0)       AS total_volume,
                COUNT(*) FILTER (WHERE status = 'open') AS open_trades
            FROM carbon_trades
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        let recent_trades = sqlx::query_as::<CarbonTrade, _>(
            "SELECT id, credit_id, seller_id, buyer_id, quantity, price_per_tonne, total_amount, platform_fee, status, trade_type, settlement_date, notes, expires_at, created_at, updated_at FROM carbon_trades WHERE status = 'settled' ORDER BY updated_at DESC LIMIT 5"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(MarketSummary {
            total_credits_available: row.get::<f64, _>("total_available"),
            total_credits_listed: row.get::<f64, _>("total_listed"),
            total_credits_sold: row.get::<f64, _>("total_sold"),
            total_credits_retired: row.get::<f64, _>("total_retired"),
            avg_price_per_tonne: price_row.get::<f64, _>("avg_price"),
            total_market_volume_usd: price_row.get::<f64, _>("total_volume"),
            open_trades: price_row.get::<i64, _>("open_trades"),
            recent_trades,
        })
    }

    // ── Verification ──────────────────────────────────────────────────────────

    pub async fn request_verification(
        &self,
        requester_id: Uuid,
        req: &RequestVerificationRequest,
    ) -> Result<CarbonVerification, AppError> {
        let credit = self.get_credit(req.credit_id).await?;
        if credit.owner_id != requester_id {
            return Err(AppError::Forbidden("You do not own this credit".into()));
        }

        let verification = sqlx::query_as::<CarbonVerification, _>(
            r#"
            INSERT INTO carbon_verifications (
                credit_id, requested_by, verifier_name,
                verifier_accreditation, status, methodology, scope
            ) VALUES ($1, $2, $3, $4, 'requested', $5, $6)
            RETURNING
                id, credit_id, requested_by, verifier_name,
                verifier_accreditation, status, verification_date,
                report_hash, report_url, methodology, scope, created_at, updated_at
            "#,
        )
        .bind(req.credit_id)
        .bind(requester_id)
        .bind(req.verifier_name.clone())
        .bind(req.verifier_accreditation.clone())
        .bind(req.methodology.clone())
        .bind(req.scope.clone())
        .fetch_one(&self.pool)
        .await?;

        Ok(verification)
    }

    pub async fn list_verifications(
        &self,
        credit_id: Uuid,
    ) -> Result<Vec<CarbonVerification>, AppError> {
        let records = sqlx::query_as::<CarbonVerification, _>(
            "SELECT id, credit_id, requested_by, verifier_name, verifier_accreditation, status, verification_date, report_hash, report_url, methodology, scope, created_at, updated_at FROM carbon_verifications WHERE credit_id = $1 ORDER BY created_at DESC",
        )
        .bind(credit_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(records)
    }

    // ── Reporting ─────────────────────────────────────────────────────────────

    pub async fn generate_report(
        &self,
        owner_id: Uuid,
        req: &GenerateReportRequest,
    ) -> Result<CarbonReport, AppError> {
        // Aggregate emissions for the period
        let emissions_row = sqlx::query(
            r#"
            SELECT
                COALESCE(SUM(cf.total_emissions), 0)    AS total_emissions,
                COALESCE(SUM(cf.emissions_reduction), 0) AS total_reductions
            FROM carbon_footprints cf
            JOIN products p ON p.id = cf.product_id
            WHERE p.owner_address IN (
                SELECT stellar_address FROM users WHERE id = $1 AND stellar_address IS NOT NULL
            )
            AND cf.calculated_at BETWEEN $2 AND $3
            "#,
        )
        .bind(owner_id)
        .bind(req.period_start)
        .bind(req.period_end)
        .fetch_one(&self.pool)
        .await?;

        // Aggregate credits for the period
        let credits_row = sqlx::query(
            r#"
            SELECT
                COALESCE(SUM(quantity), 0)                                          AS generated,
                COALESCE(SUM(quantity) FILTER (WHERE status = 'retired'), 0)        AS retired,
                COALESCE(SUM(quantity) FILTER (WHERE status = 'sold'), 0)           AS sold
            FROM carbon_credits
            WHERE owner_id = $1
              AND created_at BETWEEN $2 AND $3
            "#,
        )
        .bind(owner_id)
        .bind(req.period_start)
        .bind(req.period_end)
        .fetch_one(&self.pool)
        .await?;

        let revenue_row = sqlx::query(
            r#"
            SELECT COALESCE(SUM(total_amount), 0) AS revenue
            FROM carbon_trades
            WHERE seller_id = $1
              AND status = 'settled'
              AND settlement_date BETWEEN $2 AND $3
            "#,
        )
        .bind(owner_id)
        .bind(req.period_start)
        .bind(req.period_end)
        .fetch_one(&self.pool)
        .await?;

        let total_emissions: f64 = emissions_row.get::<f64, _>("total_emissions");
        let total_reductions: f64 = emissions_row.get::<f64, _>("total_reductions");
        let net_emissions = total_emissions - total_reductions;

        let report_type = req.report_type.as_deref().unwrap_or("custom");

        let summary = serde_json::json!({
            "methodology": "GHG Protocol",
            "scope": "Scope 1, 2, 3 supply chain",
            "period": {
                "start": req.period_start,
                "end": req.period_end,
            },
            "carbon_intensity": if total_emissions > 0.0 {
                total_reductions / total_emissions * 100.0
            } else { 0.0 },
        });

        let report = sqlx::query_as::<CarbonReport, _>(
            r#"
            INSERT INTO carbon_reports (
                owner_id, report_type, period_start, period_end,
                total_emissions, total_reductions, net_emissions,
                credits_generated, credits_retired, credits_sold,
                revenue_from_credits, summary
            ) VALUES (
                $1, $2, $3, $4,
                $5, $6, $7,
                $8, $9, $10,
                $11, $12
            )
            RETURNING
                id, owner_id, report_type, period_start, period_end,
                total_emissions, total_reductions, net_emissions,
                credits_generated, credits_retired, credits_sold,
                revenue_from_credits, summary, generated_at
            "#,
        )
        .bind(owner_id)
        .bind(report_type)
        .bind(req.period_start)
        .bind(req.period_end)
        .bind(total_emissions)
        .bind(total_reductions)
        .bind(net_emissions)
        .bind(credits_row.get::<f64, _>("generated"))
        .bind(credits_row.get::<f64, _>("retired"))
        .bind(credits_row.get::<f64, _>("sold"))
        .bind(revenue_row.get::<f64, _>("revenue"))
        .bind(summary)
        .fetch_one(&self.pool)
        .await?;

        Ok(report)
    }

    pub async fn list_reports(&self, owner_id: Uuid) -> Result<Vec<CarbonReport>, AppError> {
        let reports = sqlx::query_as::<CarbonReport, _>(
            "SELECT id, owner_id, report_type, period_start, period_end, total_emissions, total_reductions, net_emissions, credits_generated, credits_retired, credits_sold, revenue_from_credits, summary, generated_at FROM carbon_reports WHERE owner_id = $1 ORDER BY generated_at DESC",
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(reports)
    }

    /// Calculate the sustainability score for a supplier based on their products' emissions reductions.
    pub async fn get_supplier_score(&self, supplier_address: &str) -> Result<f64, AppError> {
        let row = sqlx::query(
            r#"
            SELECT AVG(cf.reduction_percentage) as avg_reduction
            FROM carbon_footprints cf
            JOIN products p ON p.id = cf.product_id
            WHERE p.owner_address = $1
            "#,
        )
        .bind(supplier_address)
        .fetch_one(&self.pool)
        .await?;

        // Score is base 50 + average reduction percentage, capped at 100
        let score = match row.get::<Option<sqlx::types::BigDecimal>, _>("avg_reduction") {
            Some(reduction) => {
                let r: f64 = reduction.to_string().parse().unwrap_or(0.0);
                (50.0 + r).min(100.0)
            }
            None => 50.0,
        };
        Ok((score * 10.0).round() / 10.0) // Round to 1 decimal place
    }

    /// Generates an overall sustainability metrics dashboard overview.
    pub async fn get_sustainability_dashboard(&self) -> Result<serde_json::Value, AppError> {
        let emissions = sqlx::query(
            r#"
            SELECT 
                COALESCE(SUM(total_emissions), 0) as total_emissions,
                COALESCE(SUM(emissions_reduction), 0) as total_reductions,
                AVG(reduction_percentage) as avg_reduction
            FROM carbon_footprints
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        let credits = sqlx::query(
            r#"
            SELECT
                COALESCE(SUM(quantity) FILTER (WHERE status = 'retired'), 0) AS retired_credits,
                COALESCE(SUM(quantity) FILTER (WHERE status = 'listed'), 0) AS listed_credits
            FROM carbon_credits
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        let top_suppliers = sqlx::query(
            r#"
            SELECT p.owner_address, AVG(cf.reduction_percentage) as avg_reduction
            FROM carbon_footprints cf
            JOIN products p ON p.id = cf.product_id
            GROUP BY p.owner_address
            ORDER BY avg_reduction DESC NULLS LAST
            LIMIT 5
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut suppliers_ranking = Vec::new();
        for s in top_suppliers {
            let avg: f64 = match s.get::<Option<sqlx::types::BigDecimal>, _>("avg_reduction") {
                Some(v) => v.to_string().parse().unwrap_or(0.0),
                None => 0.0,
            };
            suppliers_ranking.push(serde_json::json!({
                "supplier": s.get::<String, _>("owner_address"),
                "sustainability_score": (50.0 + avg).min(100.0)
            }));
        }

        let total_emissions: f64 = emissions.get::<sqlx::types::BigDecimal, _>("total_emissions").to_string().parse().unwrap_or(0.0);
        let total_reductions: f64 = emissions.get::<sqlx::types::BigDecimal, _>("total_reductions").to_string().parse().unwrap_or(0.0);
        let avg_reduction: f64 = match emissions.get::<Option<sqlx::types::BigDecimal>, _>("avg_reduction") {
            Some(v) => v.to_string().parse().unwrap_or(0.0),
            None => 0.0,
        };

        Ok(serde_json::json!({
            "metrics": {
                "total_emissions_kg": total_emissions,
                "total_reductions_kg": total_reductions,
                "average_reduction_percentage": avg_reduction,
                "credits_retired_tonnes": credits.get::<sqlx::types::BigDecimal, _>("retired_credits").to_string().parse::<f64>().unwrap_or(0.0),
                "credits_listed_tonnes": credits.get::<sqlx::types::BigDecimal, _>("listed_credits").to_string().parse::<f64>().unwrap_or(0.0),
            },
            "top_sustainable_suppliers": suppliers_ranking
        }))
    }
}


// ── Helpers ───────────────────────────────────────────────────────────────────

fn generate_serial_number(vintage_year: i32) -> String {
    format!(
        "CL-{}-{}-{}",
        vintage_year,
        Utc::now().format("%Y%m%d"),
        &Uuid::new_v4().to_string()[..8].to_uppercase()
    )
}
