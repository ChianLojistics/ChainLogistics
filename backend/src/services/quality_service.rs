use sqlx::PgPool;
use uuid::Uuid;
use crate::models::quality::*;
use rust_decimal::Decimal;

pub struct QualityService {
    pool: PgPool,
}

impl QualityService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // QC Checkpoints
    pub async fn create_checkpoint(&self, checkpoint: NewQCCheckpoint) -> Result<QCCheckpoint, sqlx::Error> {
        sqlx::query_as!(
            QCCheckpoint,
            r#"
            INSERT INTO qc_checkpoints (
                checkpoint_id, name, description, checkpoint_type, category,
                product_category, required_fields, acceptance_criteria
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            checkpoint.checkpoint_id,
            checkpoint.name,
            checkpoint.description,
            checkpoint.checkpoint_type,
            checkpoint.category,
            checkpoint.product_category,
            checkpoint.required_fields,
            checkpoint.acceptance_criteria
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_checkpoint(&self, checkpoint_id: &str) -> Result<Option<QCCheckpoint>, sqlx::Error> {
        sqlx::query_as!(
            QCCheckpoint,
            "SELECT * FROM qc_checkpoints WHERE checkpoint_id = $1",
            checkpoint_id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_checkpoints(
        &self,
        checkpoint_type: Option<String>,
        category: Option<String>,
        is_active: Option<bool>,
    ) -> Result<Vec<QCCheckpoint>, sqlx::Error> {
        match (checkpoint_type, category, is_active) {
            (Some(ct), Some(cat), Some(active)) => {
                sqlx::query_as!(
                    QCCheckpoint,
                    "SELECT * FROM qc_checkpoints WHERE checkpoint_type = $1 AND category = $2 AND is_active = $3 ORDER BY created_at DESC",
                    ct, cat, active
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(ct), Some(cat), None) => {
                sqlx::query_as!(
                    QCCheckpoint,
                    "SELECT * FROM qc_checkpoints WHERE checkpoint_type = $1 AND category = $2 ORDER BY created_at DESC",
                    ct, cat
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(ct), None, Some(active)) => {
                sqlx::query_as!(
                    QCCheckpoint,
                    "SELECT * FROM qc_checkpoints WHERE checkpoint_type = $1 AND is_active = $2 ORDER BY created_at DESC",
                    ct, active
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(cat), Some(active)) => {
                sqlx::query_as!(
                    QCCheckpoint,
                    "SELECT * FROM qc_checkpoints WHERE category = $1 AND is_active = $2 ORDER BY created_at DESC",
                    cat, active
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(ct), None, None) => {
                sqlx::query_as!(
                    QCCheckpoint,
                    "SELECT * FROM qc_checkpoints WHERE checkpoint_type = $1 ORDER BY created_at DESC",
                    ct
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(cat), None) => {
                sqlx::query_as!(
                    QCCheckpoint,
                    "SELECT * FROM qc_checkpoints WHERE category = $1 ORDER BY created_at DESC",
                    cat
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, None, Some(active)) => {
                sqlx::query_as!(
                    QCCheckpoint,
                    "SELECT * FROM qc_checkpoints WHERE is_active = $1 ORDER BY created_at DESC",
                    active
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, None, None) => {
                sqlx::query_as!(
                    QCCheckpoint,
                    "SELECT * FROM qc_checkpoints ORDER BY created_at DESC"
                )
                .fetch_all(&self.pool)
                .await
            }
        }
    }

    // QC Workflows
    pub async fn create_workflow(&self, workflow: NewQCWorkflow) -> Result<QCWorkflow, sqlx::Error> {
        sqlx::query_as!(
            QCWorkflow,
            r#"
            INSERT INTO qc_workflows (
                workflow_id, name, description, product_category, checkpoint_ids
            ) VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            workflow.workflow_id,
            workflow.name,
            workflow.description,
            workflow.product_category,
            &workflow.checkpoint_ids
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_workflow(&self, workflow_id: &str) -> Result<Option<QCWorkflow>, sqlx::Error> {
        sqlx::query_as!(
            QCWorkflow,
            "SELECT * FROM qc_workflows WHERE workflow_id = $1",
            workflow_id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_workflows(&self, is_active: Option<bool>) -> Result<Vec<QCWorkflow>, sqlx::Error> {
        if let Some(active) = is_active {
            sqlx::query_as!(
                QCWorkflow,
                "SELECT * FROM qc_workflows WHERE is_active = $1 ORDER BY created_at DESC",
                active
            )
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as!(
                QCWorkflow,
                "SELECT * FROM qc_workflows ORDER BY created_at DESC"
            )
            .fetch_all(&self.pool)
            .await
        }
    }

    // QC Inspections
    pub async fn create_inspection(&self, inspection: NewQCInspection) -> Result<QCInspection, sqlx::Error> {
        sqlx::query_as!(
            QCInspection,
            r#"
            INSERT INTO qc_inspections (
                inspection_id, product_id, checkpoint_id, workflow_id,
                inspector_id, location, results, quality_metrics, notes, evidence_documents
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
            inspection.inspection_id,
            inspection.product_id,
            inspection.checkpoint_id,
            inspection.workflow_id,
            inspection.inspector_id,
            inspection.location,
            inspection.results,
            inspection.quality_metrics,
            inspection.notes,
            &inspection.evidence_documents
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_inspection_status(
        &self,
        inspection_id: &str,
        status: String,
        is_passed: Option<bool>,
        failure_reason: Option<String>,
    ) -> Result<QCInspection, sqlx::Error> {
        sqlx::query_as!(
            QCInspection,
            r#"
            UPDATE qc_inspections SET
                status = $2,
                inspection_date = NOW(),
                is_passed = $3,
                failure_reason = $4
            WHERE inspection_id = $1
            RETURNING *
            "#,
            inspection_id,
            status,
            is_passed,
            failure_reason
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_inspection(&self, inspection_id: &str) -> Result<Option<QCInspection>, sqlx::Error> {
        sqlx::query_as!(
            QCInspection,
            "SELECT * FROM qc_inspections WHERE inspection_id = $1",
            inspection_id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_inspections(
        &self,
        product_id: Option<String>,
        checkpoint_id: Option<String>,
        status: Option<String>,
        limit: i64,
    ) -> Result<Vec<QCInspection>, sqlx::Error> {
        match (product_id, checkpoint_id, status) {
            (Some(pid), Some(cid), Some(s)) => {
                sqlx::query_as!(
                    QCInspection,
                    "SELECT * FROM qc_inspections WHERE product_id = $1 AND checkpoint_id = $2 AND status = $3 ORDER BY inspection_date DESC LIMIT $4",
                    pid, cid, s, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(pid), Some(cid), None) => {
                sqlx::query_as!(
                    QCInspection,
                    "SELECT * FROM qc_inspections WHERE product_id = $1 AND checkpoint_id = $2 ORDER BY inspection_date DESC LIMIT $3",
                    pid, cid, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(pid), None, Some(s)) => {
                sqlx::query_as!(
                    QCInspection,
                    "SELECT * FROM qc_inspections WHERE product_id = $1 AND status = $2 ORDER BY inspection_date DESC LIMIT $3",
                    pid, s, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(cid), Some(s)) => {
                sqlx::query_as!(
                    QCInspection,
                    "SELECT * FROM qc_inspections WHERE checkpoint_id = $1 AND status = $2 ORDER BY inspection_date DESC LIMIT $3",
                    cid, s, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(pid), None, None) => {
                sqlx::query_as!(
                    QCInspection,
                    "SELECT * FROM qc_inspections WHERE product_id = $1 ORDER BY inspection_date DESC LIMIT $2",
                    pid, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(cid), None) => {
                sqlx::query_as!(
                    QCInspection,
                    "SELECT * FROM qc_inspections WHERE checkpoint_id = $1 ORDER BY inspection_date DESC LIMIT $2",
                    cid, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, None, Some(s)) => {
                sqlx::query_as!(
                    QCInspection,
                    "SELECT * FROM qc_inspections WHERE status = $1 ORDER BY inspection_date DESC LIMIT $2",
                    s, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, None, None) => {
                sqlx::query_as!(
                    QCInspection,
                    "SELECT * FROM qc_inspections ORDER BY inspection_date DESC LIMIT $1",
                    limit
                )
                .fetch_all(&self.pool)
                .await
            }
        }
    }

    // Execute Workflow
    pub async fn execute_workflow(&self, request: WorkflowExecutionRequest) -> Result<WorkflowExecutionResult, Box<dyn std::error::Error>> {
        let workflow = self.get_workflow(&request.workflow_id).await?
            .ok_or_else(|| "Workflow not found")?;

        let mut inspections = Vec::new();
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        for checkpoint_id in &workflow.checkpoint_ids {
            let inspection_id = format!("INS-{}", Uuid::new_v4());
            
            let inspection = self.create_inspection(NewQCInspection {
                inspection_id: inspection_id.clone(),
                product_id: request.product_id.clone(),
                checkpoint_id: checkpoint_id.clone(),
                workflow_id: Some(request.workflow_id.clone()),
                inspector_id: Some(request.inspector_id.clone()),
                location: None,
                results: serde_json::json!({}),
                quality_metrics: serde_json::json!({}),
                notes: None,
                evidence_documents: vec![],
            }).await?;

            inspections.push(inspection);
        }

        let total = workflow.checkpoint_ids.len() as i32;
        let completed = inspections.len() as i32;
        let overall_status = if failed == 0 { "passed" } else { "failed" };

        Ok(WorkflowExecutionResult {
            workflow_id: request.workflow_id,
            product_id: request.product_id,
            total_checkpoints: total,
            completed,
            passed,
            failed,
            skipped,
            overall_status: overall_status.to_string(),
            inspections,
        })
    }

    // Non-Conformances
    pub async fn create_non_conformance(&self, nc: NewNonConformance) -> Result<NonConformance, sqlx::Error> {
        sqlx::query_as!(
            NonConformance,
            r#"
            INSERT INTO non_conformances (
                nc_id, inspection_id, product_id, severity, category,
                description, root_cause, correction_action, responsible_party, due_date
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
            nc.nc_id,
            nc.inspection_id,
            nc.product_id,
            nc.severity,
            nc.category,
            nc.description,
            nc.root_cause,
            nc.correction_action,
            nc.responsible_party,
            nc.due_date
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_non_conformance(
        &self,
        nc_id: &str,
        correction_action: Option<String>,
        correction_status: String,
        responsible_party: Option<String>,
    ) -> Result<NonConformance, sqlx::Error> {
        let resolved_at = if correction_status == "resolved" {
            Some(chrono::Utc::now())
        } else {
            None
        };

        sqlx::query_as!(
            NonConformance,
            r#"
            UPDATE non_conformances SET
                correction_action = $2,
                correction_status = $3,
                responsible_party = $4,
                resolved_at = $5
            WHERE nc_id = $1
            RETURNING *
            "#,
            nc_id,
            correction_action,
            correction_status,
            responsible_party,
            resolved_at
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn verify_non_conformance(
        &self,
        nc_id: &str,
        verified_by: String,
    ) -> Result<NonConformance, sqlx::Error> {
        sqlx::query_as!(
            NonConformance,
            r#"
            UPDATE non_conformances SET
                correction_status = 'verified',
                verified_by = $2,
                verified_at = NOW()
            WHERE nc_id = $1
            RETURNING *
            "#,
            nc_id,
            verified_by
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_non_conformances(
        &self,
        product_id: Option<String>,
        severity: Option<String>,
        status: Option<String>,
        limit: i64,
    ) -> Result<Vec<NonConformance>, sqlx::Error> {
        match (product_id, severity, status) {
            (Some(pid), Some(s), Some(st)) => {
                sqlx::query_as!(
                    NonConformance,
                    "SELECT * FROM non_conformances WHERE product_id = $1 AND severity = $2 AND correction_status = $3 ORDER BY created_at DESC LIMIT $4",
                    pid, s, st, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(pid), Some(s), None) => {
                sqlx::query_as!(
                    NonConformance,
                    "SELECT * FROM non_conformances WHERE product_id = $1 AND severity = $2 ORDER BY created_at DESC LIMIT $3",
                    pid, s, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(pid), None, Some(st)) => {
                sqlx::query_as!(
                    NonConformance,
                    "SELECT * FROM non_conformances WHERE product_id = $1 AND correction_status = $2 ORDER BY created_at DESC LIMIT $3",
                    pid, st, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(s), Some(st)) => {
                sqlx::query_as!(
                    NonConformance,
                    "SELECT * FROM non_conformances WHERE severity = $1 AND correction_status = $2 ORDER BY created_at DESC LIMIT $3",
                    s, st, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(pid), None, None) => {
                sqlx::query_as!(
                    NonConformance,
                    "SELECT * FROM non_conformances WHERE product_id = $1 ORDER BY created_at DESC LIMIT $2",
                    pid, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(s), None) => {
                sqlx::query_as!(
                    NonConformance,
                    "SELECT * FROM non_conformances WHERE severity = $1 ORDER BY created_at DESC LIMIT $2",
                    s, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, None, Some(st)) => {
                sqlx::query_as!(
                    NonConformance,
                    "SELECT * FROM non_conformances WHERE correction_status = $1 ORDER BY created_at DESC LIMIT $2",
                    st, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, None, None) => {
                sqlx::query_as!(
                    NonConformance,
                    "SELECT * FROM non_conformances ORDER BY created_at DESC LIMIT $1",
                    limit
                )
                .fetch_all(&self.pool)
                .await
            }
        }
    }

    // Quality Metrics
    pub async fn create_metric(&self, metric: NewQualityMetric) -> Result<QualityMetric, sqlx::Error> {
        // Check if within threshold
        let is_within_threshold = if let (Some(min), Some(max)) = (metric.threshold_min, metric.threshold_max) {
            Some(metric.metric_value >= min && metric.metric_value <= max)
        } else if let Some(min) = metric.threshold_min {
            Some(metric.metric_value >= min)
        } else if let Some(max) = metric.threshold_max {
            Some(metric.metric_value <= max)
        } else {
            None
        };

        sqlx::query_as!(
            QualityMetric,
            r#"
            INSERT INTO quality_metrics (
                metric_id, product_id, metric_type, metric_value, unit,
                measurement_period_start, measurement_period_end,
                target_value, threshold_min, threshold_max, is_within_threshold, notes
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
            metric.metric_id,
            metric.product_id,
            metric.metric_type,
            metric.metric_value,
            metric.unit,
            metric.measurement_period_start,
            metric.measurement_period_end,
            metric.target_value,
            metric.threshold_min,
            metric.threshold_max,
            is_within_threshold,
            metric.notes
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_metrics(
        &self,
        product_id: Option<String>,
        metric_type: Option<String>,
        limit: i64,
    ) -> Result<Vec<QualityMetric>, sqlx::Error> {
        match (product_id, metric_type) {
            (Some(pid), Some(mt)) => {
                sqlx::query_as!(
                    QualityMetric,
                    "SELECT * FROM quality_metrics WHERE product_id = $1 AND metric_type = $2 ORDER BY measurement_period_start DESC LIMIT $3",
                    pid, mt, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (Some(pid), None) => {
                sqlx::query_as!(
                    QualityMetric,
                    "SELECT * FROM quality_metrics WHERE product_id = $1 ORDER BY measurement_period_start DESC LIMIT $2",
                    pid, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(mt)) => {
                sqlx::query_as!(
                    QualityMetric,
                    "SELECT * FROM quality_metrics WHERE metric_type = $1 ORDER BY measurement_period_start DESC LIMIT $2",
                    mt, limit
                )
                .fetch_all(&self.pool)
                .await
            }
            (None, None) => {
                sqlx::query_as!(
                    QualityMetric,
                    "SELECT * FROM quality_metrics ORDER BY measurement_period_start DESC LIMIT $1",
                    limit
                )
                .fetch_all(&self.pool)
                .await
            }
        }
    }
}
