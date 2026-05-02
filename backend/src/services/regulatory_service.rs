use sqlx::PgPool;
use uuid::Uuid;
use crate::models::regulatory::*;
use crate::compliance::{ComplianceRule, ComplianceType, ComplianceValidator};
use serde_json::json;

pub struct RegulatoryService {
    pool: PgPool,
}

impl RegulatoryService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Regulatory Requirements Management
    pub async fn create_requirement(&self, req: NewRegulatoryRequirement) -> Result<RegulatoryRequirement, sqlx::Error> {
        sqlx::query_as!(
            RegulatoryRequirement,
            r#"
            INSERT INTO regulatory_requirements (
                requirement_id, name, description, regulation_type, category,
                severity, required_fields, validation_logic, is_active,
                effective_date, expiry_date
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, $9, $10)
            RETURNING *
            "#,
            req.requirement_id,
            req.name,
            req.description,
            req.regulation_type,
            req.category,
            req.severity,
            req.required_fields,
            req.validation_logic,
            req.effective_date.unwrap_or(chrono::Utc::now()),
            req.expiry_date
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_requirement(&self, requirement_id: &str) -> Result<Option<RegulatoryRequirement>, sqlx::Error> {
        sqlx::query_as!(
            RegulatoryRequirement,
            "SELECT * FROM regulatory_requirements WHERE requirement_id = $1",
            requirement_id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_requirements(
        &self,
        regulation_type: Option<String>,
        category: Option<String>,
        is_active: Option<bool>,
    ) -> Result<Vec<RegulatoryRequirement>, sqlx::Error> {
        let mut query = "SELECT * FROM regulatory_requirements WHERE 1=1".to_string();
        let mut bindings = Vec::new();
        let mut bind_index = 1;

        if let Some(reg_type) = regulation_type {
            query.push_str(&format!(" AND regulation_type = ${}", bind_index));
            bindings.push(reg_type);
            bind_index += 1;
        }
        if let Some(cat) = category {
            query.push_str(&format!(" AND category = ${}", bind_index));
            bindings.push(cat);
            bind_index += 1;
        }
        if let Some(active) = is_active {
            query.push_str(&format!(" AND is_active = ${}", bind_index));
            bindings.push(active.to_string());
            bind_index += 1;
        }

        query.push_str(" ORDER BY created_at DESC");

        let mut q = sqlx::QueryBuilder::new(query);
        for binding in bindings {
            q = q.bind(binding);
        }

        q.build_query_as::<RegulatoryRequirement>()
            .fetch_all(&self.pool)
            .await
    }

    pub async fn update_requirement(&self, requirement_id: &str, req: NewRegulatoryRequirement) -> Result<RegulatoryRequirement, sqlx::Error> {
        sqlx::query_as!(
            RegulatoryRequirement,
            r#"
            UPDATE regulatory_requirements SET
                name = $2,
                description = $3,
                regulation_type = $4,
                category = $5,
                severity = $6,
                required_fields = $7,
                validation_logic = $8,
                expiry_date = $9
            WHERE requirement_id = $1
            RETURNING *
            "#,
            requirement_id,
            req.name,
            req.description,
            req.regulation_type,
            req.category,
            req.severity,
            req.required_fields,
            req.validation_logic,
            req.expiry_date
        )
        .fetch_one(&self.pool)
        .await
    }

    // Product Compliance Management
    pub async fn create_product_compliance(&self, compliance: NewProductCompliance) -> Result<ProductCompliance, sqlx::Error> {
        sqlx::query_as!(
            ProductCompliance,
            r#"
            INSERT INTO product_compliance (
                product_id, requirement_id, status, notes, checked_by
            ) VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            compliance.product_id,
            compliance.requirement_id,
            compliance.status,
            compliance.notes,
            compliance.checked_by
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_product_compliance(&self, product_id: &str, requirement_id: &str) -> Result<Option<ProductCompliance>, sqlx::Error> {
        sqlx::query_as!(
            ProductCompliance,
            "SELECT * FROM product_compliance WHERE product_id = $1 AND requirement_id = $2",
            product_id,
            requirement_id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_product_compliance(&self, product_id: &str) -> Result<Vec<ProductCompliance>, sqlx::Error> {
        sqlx::query_as!(
            ProductCompliance,
            "SELECT * FROM product_compliance WHERE product_id = $1 ORDER BY created_at DESC",
            product_id
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update_product_compliance(
        &self,
        product_id: &str,
        requirement_id: &str,
        status: String,
        violations: Vec<String>,
        warnings: Vec<String>,
        notes: Option<String>,
        checked_by: String,
    ) -> Result<ProductCompliance, sqlx::Error> {
        sqlx::query_as!(
            ProductCompliance,
            r#"
            UPDATE product_compliance SET
                status = $3,
                last_checked_at = NOW(),
                last_check_result = $4,
                violations = $5,
                warnings = $6,
                notes = $7,
                checked_by = $8
            WHERE product_id = $1 AND requirement_id = $2
            RETURNING *
            "#,
            product_id,
            requirement_id,
            status,
            json!({"checked_at": chrono::Utc::now().to_rfc3339()}),
            json!(violations),
            json!(warnings),
            notes,
            checked_by
        )
        .fetch_one(&self.pool)
        .await
    }

    // Automated Compliance Check
    pub async fn run_compliance_check(&self, request: ComplianceCheckRequest) -> Result<ComplianceCheckResult, Box<dyn std::error::Error>> {
        let requirement_ids = if let Some(ids) = request.requirement_ids {
            ids
        } else {
            // Get all active requirements
            let reqs = self.list_requirements(None, None, Some(true)).await?;
            reqs.iter().map(|r| r.requirement_id.clone()).collect()
        };

        let mut details = Vec::new();
        let mut compliant = 0;
        let mut non_compliant = 0;
        let mut pending = 0;

        for req_id in &requirement_ids {
            if let Some(requirement) = self.get_requirement(req_id).await? {
                // Get product data for validation
                let product_data = self.get_product_data_for_validation(&request.product_id).await?;
                
                // Create compliance rule from requirement
                let rule = self.requirement_to_rule(&requirement);
                
                // Validate
                let validation_result = ComplianceValidator::validate(&rule, &product_data);
                
                let status = if validation_result.is_compliant {
                    "compliant".to_string()
                } else {
                    "non_compliant".to_string()
                };

                // Update product compliance
                let _ = self.update_product_compliance(
                    &request.product_id,
                    req_id,
                    status.clone(),
                    validation_result.violations.clone(),
                    validation_result.warnings.clone(),
                    None,
                    &request.performed_by,
                ).await;

                // Log audit trail
                let _ = self.create_audit_trail(NewComplianceAuditTrail {
                    product_id: request.product_id.clone(),
                    requirement_id: Some(req_id.clone()),
                    action_type: "check".to_string(),
                    previous_status: None,
                    new_status: Some(status.clone()),
                    action_details: json!({
                        "violations": validation_result.violations,
                        "warnings": validation_result.warnings
                    }),
                    performed_by: request.performed_by.clone(),
                    ip_address: None,
                    user_agent: None,
                }).await;

                details.push(ComplianceDetail {
                    requirement_id: req_id.clone(),
                    requirement_name: requirement.name,
                    regulation_type: requirement.regulation_type,
                    status,
                    violations: validation_result.violations,
                    warnings: validation_result.warnings,
                });

                if validation_result.is_compliant {
                    compliant += 1;
                } else {
                    non_compliant += 1;
                }
            } else {
                pending += 1;
            }
        }

        Ok(ComplianceCheckResult {
            product_id: request.product_id,
            total_requirements: requirement_ids.len() as i32,
            compliant,
            non_compliant,
            pending,
            details,
        })
    }

    async fn get_product_data_for_validation(&self, product_id: &str) -> Result<serde_json::Value, sqlx::Error> {
        // Fetch product data from database
        let product = sqlx::query!(
            "SELECT * FROM products WHERE id = $1",
            product_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(p) = product {
            Ok(json!({
                "id": p.id,
                "name": p.name,
                "category": p.category,
                "origin_location": p.origin_location,
                "certifications": p.certifications,
                "custom_fields": p.custom_fields,
                "tags": p.tags
            }))
        } else {
            Ok(json!({}))
        }
    }

    fn requirement_to_rule(&self, requirement: &RegulatoryRequirement) -> ComplianceRule {
        ComplianceRule {
            rule_id: requirement.requirement_id.clone(),
            compliance_type: self.map_regulation_type(&requirement.regulation_type),
            description: requirement.description.clone().unwrap_or_default(),
            validation_logic: requirement.validation_logic.clone().unwrap_or_default(),
            required_fields: requirement.required_fields
                .as_array()
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect())
                .unwrap_or_default(),
            is_active: requirement.is_active,
        }
    }

    fn map_regulation_type(&self, reg_type: &str) -> ComplianceType {
        match reg_type.to_lowercase().as_str() {
            "gdpr" => ComplianceType::GDPR,
            "fda" | "fda_21_cfr_11" => ComplianceType::FDA21CFR11,
            "fsma" => ComplianceType::FSMA,
            "conflict_minerals" => ComplianceType::ConflictMinerals,
            "organic" | "organic_certification" => ComplianceType::OrganicCertification,
            "soc2" => ComplianceType::SOC2,
            "iso" | "iso27001" => ComplianceType::ISO27001,
            _ => ComplianceType::GDPR, // Default
        }
    }

    // Audit Trail
    pub async fn create_audit_trail(&self, audit: NewComplianceAuditTrail) -> Result<ComplianceAuditTrail, sqlx::Error> {
        sqlx::query_as!(
            ComplianceAuditTrail,
            r#"
            INSERT INTO compliance_audit_trail (
                product_id, requirement_id, action_type, previous_status,
                new_status, action_details, performed_by, ip_address, user_agent
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
            audit.product_id,
            audit.requirement_id,
            audit.action_type,
            audit.previous_status,
            audit.new_status,
            audit.action_details,
            audit.performed_by,
            audit.ip_address,
            audit.user_agent
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_audit_trail(&self, product_id: &str, limit: i64) -> Result<Vec<ComplianceAuditTrail>, sqlx::Error> {
        sqlx::query_as!(
            ComplianceAuditTrail,
            "SELECT * FROM compliance_audit_trail WHERE product_id = $1 ORDER BY performed_at DESC LIMIT $2",
            product_id,
            limit
        )
        .fetch_all(&self.pool)
        .await
    }

    // Compliance Reports
    pub async fn generate_report(&self, report: NewComplianceReport) -> Result<ComplianceReport, sqlx::Error> {
        let report_id = format!("RPT-{}", Uuid::new_v4());
        
        // Calculate report data based on scope
        let scope = &report.scope;
        let report_data = self.calculate_report_data(scope).await?;
        
        let total = report_data["total_products"].as_i64().unwrap_or(0) as i32;
        let compliant = report_data["compliant"].as_i64().unwrap_or(0) as i32;
        let non_compliant = report_data["non_compliant"].as_i64().unwrap_or(0) as i32;
        let pending = report_data["pending"].as_i64().unwrap_or(0) as i32;
        
        let compliance_rate = if total > 0 {
            Some((compliant as f64 / total as f64 * 100.0).into())
        } else {
            None
        };

        sqlx::query_as!(
            ComplianceReport,
            r#"
            INSERT INTO compliance_reports (
                report_id, report_type, scope, generated_by, period_start,
                period_end, total_products_checked, compliant_count,
                non_compliant_count, pending_count, compliance_rate, report_data, status
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 'completed')
            RETURNING *
            "#,
            report_id,
            report.report_type,
            report.scope,
            report.generated_by,
            report.period_start,
            report.period_end,
            total,
            compliant,
            non_compliant,
            pending,
            compliance_rate,
            report_data
        )
        .fetch_one(&self.pool)
        .await
    }

    async fn calculate_report_data(&self, scope: &serde_json::Value) -> Result<serde_json::Value, sqlx::Error> {
        // Simple aggregation based on scope
        let result = sqlx::query!(
            r#"
            SELECT 
                COUNT(DISTINCT pc.product_id) as total_products,
                SUM(CASE WHEN pc.status = 'compliant' THEN 1 ELSE 0 END) as compliant,
                SUM(CASE WHEN pc.status = 'non_compliant' THEN 1 ELSE 0 END) as non_compliant,
                SUM(CASE WHEN pc.status = 'pending' THEN 1 ELSE 0 END) as pending
            FROM product_compliance pc
            WHERE pc.last_checked_at IS NOT NULL
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(json!({
            "total_products": result.total_products.unwrap_or(0),
            "compliant": result.compliant.unwrap_or(0),
            "non_compliant": result.non_compliant.unwrap_or(0),
            "pending": result.pending.unwrap_or(0)
        }))
    }

    pub async fn get_report(&self, report_id: &str) -> Result<Option<ComplianceReport>, sqlx::Error> {
        sqlx::query_as!(
            ComplianceReport,
            "SELECT * FROM compliance_reports WHERE report_id = $1",
            report_id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_reports(&self, limit: i64) -> Result<Vec<ComplianceReport>, sqlx::Error> {
        sqlx::query_as!(
            ComplianceReport,
            "SELECT * FROM compliance_reports ORDER BY generated_at DESC LIMIT $1",
            limit
        )
        .fetch_all(&self.pool)
        .await
    }
}
