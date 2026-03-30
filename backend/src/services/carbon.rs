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

        let record = sqlx::query_as!(
            CarbonFootprint,
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
            RETURNING *
            "#,
            req.product_id,
            req.tracking_event_id,
            breakdown.transport_emissions,
            breakdown.manufacturing_emissions,
            breakdown.packaging_emissions,
            breakdown.storage_emissions,
            breakdown.total_emissions,
            req.baseline_emissions,
            breakdown.emissions_reduction,
            breakdown.reduction_percentage,
            req.distance_km,
            req.transport_mode,
            req.energy_source,
            serde_json::to_value(req).unwrap_or_default(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    /// Get all footprint records for a product.
    pub async fn list_footprints(
        &self,
        product_id: &str,
    ) -> Result<Vec<CarbonFootprint>, AppError> {
        let records = sqlx::query_as!(
            CarbonFootprint,
            "SELECT * FROM carbon_footprints WHERE product_id = $1 ORDER BY calculated_at DESC",
            product_id
        )
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
        let footprint = sqlx::query_as!(
            CarbonFootprint,
            "SELECT * FROM carbon_footprints WHERE id = $1",
            req.footprint_id
        )
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

        let credit = sqlx::query_as!(
            CarbonCredit,
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
            RETURNING *
            "#,
            owner_id,
            req.product_id,
            serial,
            req.vintage_year,
            credit_type,
            standard,
            breakdown.eligible_credits,
            req.price_per_tonne,
            req.registry_id,
            req.verification_body,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(credit)
    }

    pub async fn get_credit(&self, id: Uuid) -> Result<CarbonCredit, AppError> {
        sqlx::query_as!(CarbonCredit, "SELECT * FROM carbon_credits WHERE id = $1", id)
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

        let records = sqlx::query_as!(
            CarbonCredit,
            r#"
            SELECT * FROM carbon_credits
            WHERE owner_id = $1
              AND ($2::TEXT IS NULL OR status = $2)
              AND ($3::INT IS NULL OR vintage_year = $3)
              AND ($4::TEXT IS NULL OR standard = $4)
            ORDER BY created_at DESC
            LIMIT $5 OFFSET $6
            "#,
            owner_id,
            query.status,
            query.vintage_year,
            query.standard,
            limit,
            offset,
        )
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

        let updated = sqlx::query_as!(
            CarbonCredit,
            r#"
            UPDATE carbon_credits
            SET status = 'retired', retired_at = NOW(), retirement_reason = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
            req.credit_id,
            req.reason,
        )
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

        let trade = sqlx::query_as!(
            CarbonTrade,
            r#"
            INSERT INTO carbon_trades (
                credit_id, seller_id, quantity, price_per_tonne,
                total_amount, trade_type, status, notes, expires_at
            ) VALUES (
                $1, $2, $3, $4,
                $5, $6, 'open', $7, $8
            )
            RETURNING *
            "#,
            req.credit_id,
            seller_id,
            req.quantity,
            req.price_per_tonne,
            total_amount,
            trade_type,
            req.notes,
            req.expires_at,
        )
        .fetch_one(&self.pool)
        .await?;

        // Mark credit as listed
        sqlx::query!(
            "UPDATE carbon_credits SET status = 'listed', updated_at = NOW() WHERE id = $1",
            req.credit_id
        )
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
        let trade = sqlx::query_as!(
            CarbonTrade,
            "SELECT * FROM carbon_trades WHERE id = $1",
            req.trade_id
        )
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

        let settled = sqlx::query_as!(
            CarbonTrade,
            r#"
            UPDATE carbon_trades
            SET buyer_id = $2, status = 'settled', quantity = $3,
                total_amount = $4, platform_fee = $5,
                settlement_date = NOW(), updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
            req.trade_id,
            buyer_id,
            req.quantity,
            total,
            platform_fee,
        )
        .fetch_one(&self.pool)
        .await?;

        // Transfer credit ownership
        sqlx::query!(
            "UPDATE carbon_credits SET owner_id = $1, status = 'sold', updated_at = NOW() WHERE id = $2",
            buyer_id,
            trade.credit_id,
        )
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

        let trades = sqlx::query_as!(
            CarbonTrade,
            r#"
            SELECT * FROM carbon_trades
            WHERE ($1::TEXT IS NULL OR status = $1)
              AND ($2::TEXT IS NULL OR trade_type = $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
            query.status,
            query.trade_type,
            limit,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(trades)
    }

    pub async fn get_market_summary(&self) -> Result<MarketSummary, AppError> {
        let row = sqlx::query!(
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

        let price_row = sqlx::query!(
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

        let recent_trades = sqlx::query_as!(
            CarbonTrade,
            "SELECT * FROM carbon_trades WHERE status = 'settled' ORDER BY updated_at DESC LIMIT 5"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(MarketSummary {
            total_credits_available: row.total_available.unwrap_or_default(),
            total_credits_listed: row.total_listed.unwrap_or_default(),
            total_credits_sold: row.total_sold.unwrap_or_default(),
            total_credits_retired: row.total_retired.unwrap_or_default(),
            avg_price_per_tonne: price_row.avg_price.unwrap_or_default(),
            total_market_volume_usd: price_row.total_volume.unwrap_or_default(),
            open_trades: price_row.open_trades.unwrap_or(0),
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

        let verification = sqlx::query_as!(
            CarbonVerification,
            r#"
            INSERT INTO carbon_verifications (
                credit_id, requested_by, verifier_name,
                verifier_accreditation, status, methodology, scope
            ) VALUES ($1, $2, $3, $4, 'requested', $5, $6)
            RETURNING *
            "#,
            req.credit_id,
            requester_id,
            req.verifier_name,
            req.verifier_accreditation,
            req.methodology,
            req.scope,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(verification)
    }

    pub async fn list_verifications(
        &self,
        credit_id: Uuid,
    ) -> Result<Vec<CarbonVerification>, AppError> {
        let records = sqlx::query_as!(
            CarbonVerification,
            "SELECT * FROM carbon_verifications WHERE credit_id = $1 ORDER BY created_at DESC",
            credit_id
        )
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
        let emissions_row = sqlx::query!(
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
            owner_id,
            req.period_start,
            req.period_end,
        )
        .fetch_one(&self.pool)
        .await?;

        // Aggregate credits for the period
        let credits_row = sqlx::query!(
            r#"
            SELECT
                COALESCE(SUM(quantity), 0)                                          AS generated,
                COALESCE(SUM(quantity) FILTER (WHERE status = 'retired'), 0)        AS retired,
                COALESCE(SUM(quantity) FILTER (WHERE status = 'sold'), 0)           AS sold
            FROM carbon_credits
            WHERE owner_id = $1
              AND created_at BETWEEN $2 AND $3
            "#,
            owner_id,
            req.period_start,
            req.period_end,
        )
        .fetch_one(&self.pool)
        .await?;

        let revenue_row = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(total_amount), 0) AS revenue
            FROM carbon_trades
            WHERE seller_id = $1
              AND status = 'settled'
              AND settlement_date BETWEEN $2 AND $3
            "#,
            owner_id,
            req.period_start,
            req.period_end,
        )
        .fetch_one(&self.pool)
        .await?;

        let total_emissions = emissions_row.total_emissions.unwrap_or_default();
        let total_reductions = emissions_row.total_reductions.unwrap_or_default();
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

        let report = sqlx::query_as!(
            CarbonReport,
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
            RETURNING *
            "#,
            owner_id,
            report_type,
            req.period_start,
            req.period_end,
            total_emissions,
            total_reductions,
            net_emissions,
            credits_row.generated.unwrap_or_default(),
            credits_row.retired.unwrap_or_default(),
            credits_row.sold.unwrap_or_default(),
            revenue_row.revenue.unwrap_or_default(),
            summary,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(report)
    }

    pub async fn list_reports(&self, owner_id: Uuid) -> Result<Vec<CarbonReport>, AppError> {
        let reports = sqlx::query_as!(
            CarbonReport,
            "SELECT * FROM carbon_reports WHERE owner_id = $1 ORDER BY generated_at DESC",
            owner_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(reports)
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
