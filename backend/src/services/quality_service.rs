use crate::models::quality::*;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

pub struct QualityService {
    pool: PgPool,
}

impl QualityService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // QC Checkpoints
    pub async fn create_checkpoint(
        &self,
        checkpoint: NewQCCheckpoint,
    ) -> Result<QCCheckpoint, sqlx::Error> {
        sqlx::query_as::<QCCheckpoint, _>(
            r#"
            INSERT INTO qc_checkpoints (
                checkpoint_id, name, description, checkpoint_type, category,
                product_category, required_fields, acceptance_criteria
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
                id, checkpoint_id, name, description, checkpoint_type, category,
                product_category, required_fields, acceptance_criteria, is_active, created_at, updated_at
            "#,
        )
        .bind(checkpoint.checkpoint_id)
        .bind(checkpoint.name)
        .bind(checkpoint.description)
        .bind(checkpoint.checkpoint_type)
        .bind(checkpoint.category)
        .bind(checkpoint.product_category)
        .bind(checkpoint.required_fields)
        .bind(checkpoint.acceptance_criteria)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_checkpoint(
        &self,
        checkpoint_id: &str,
    ) -> Result<Option<QCCheckpoint>, sqlx::Error> {
        sqlx::query_as::<QCCheckpoint, _>(
            "SELECT id, checkpoint_id, name, description, checkpoint_type, category, product_category, required_fields, acceptance_criteria, is_active, created_at, updated_at FROM qc_checkpoints WHERE checkpoint_id = $1",
        )
        .bind(checkpoint_id)
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
                sqlx::query_as::<QCCheckpoint, _>(
                    "SELECT id, checkpoint_id, name, description, checkpoint_type, category, product_category, required_fields, acceptance_criteria, is_active, created_at, updated_at FROM qc_checkpoints WHERE checkpoint_type = $1 AND category = $2 AND is_active = $3 ORDER BY created_at DESC",
                )
                .bind(ct)
                .bind(cat)
                .bind(active)
                .fetch_all(&self.pool)
                .await
            }
            (Some(ct), Some(cat), None) => {
                sqlx::query_as::<QCCheckpoint, _>(
                    "SELECT id, checkpoint_id, name, description, checkpoint_type, category, product_category, required_fields, acceptance_criteria, is_active, created_at, updated_at FROM qc_checkpoints WHERE checkpoint_type = $1 AND category = $2 ORDER BY created_at DESC",
                )
                .bind(ct)
                .bind(cat)
                .fetch_all(&self.pool)
                .await
            }
            (Some(ct), None, Some(active)) => {
                sqlx::query_as::<QCCheckpoint, _>(
                    "SELECT id, checkpoint_id, name, description, checkpoint_type, category, product_category, required_fields, acceptance_criteria, is_active, created_at, updated_at FROM qc_checkpoints WHERE checkpoint_type = $1 AND is_active = $2 ORDER BY created_at DESC",
                )
                .bind(ct)
                .bind(active)
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(cat), Some(active)) => {
                sqlx::query_as::<QCCheckpoint, _>(
                    "SELECT id, checkpoint_id, name, description, checkpoint_type, category, product_category, required_fields, acceptance_criteria, is_active, created_at, updated_at FROM qc_checkpoints WHERE category = $1 AND is_active = $2 ORDER BY created_at DESC",
                )
                .bind(cat)
                .bind(active)
                .fetch_all(&self.pool)
                .await
            }
            (Some(ct), None, None) => {
                sqlx::query_as::<QCCheckpoint, _>(
                    "SELECT id, checkpoint_id, name, description, checkpoint_type, category, product_category, required_fields, acceptance_criteria, is_active, created_at, updated_at FROM qc_checkpoints WHERE checkpoint_type = $1 ORDER BY created_at DESC",
                )
                .bind(ct)
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(cat), None) => {
                sqlx::query_as::<QCCheckpoint, _>(
                    "SELECT id, checkpoint_id, name, description, checkpoint_type, category, product_category, required_fields, acceptance_criteria, is_active, created_at, updated_at FROM qc_checkpoints WHERE category = $1 ORDER BY created_at DESC",
                )
                .bind(cat)
                .fetch_all(&self.pool)
                .await
            }
            (None, None, Some(active)) => {
                sqlx::query_as::<QCCheckpoint, _>(
                    "SELECT id, checkpoint_id, name, description, checkpoint_type, category, product_category, required_fields, acceptance_criteria, is_active, created_at, updated_at FROM qc_checkpoints WHERE is_active = $1 ORDER BY created_at DESC",
                )
                .bind(active)
                .fetch_all(&self.pool)
                .await
            }
            (None, None, None) => {
                sqlx::query_as::<QCCheckpoint, _>(
                    "SELECT id, checkpoint_id, name, description, checkpoint_type, category, product_category, required_fields, acceptance_criteria, is_active, created_at, updated_at FROM qc_checkpoints ORDER BY created_at DESC"
                )
                .fetch_all(&self.pool)
                .await
            }
        }
    }

    // QC Workflows
    pub async fn create_workflow(
        &self,
        workflow: NewQCWorkflow,
    ) -> Result<QCWorkflow, sqlx::Error> {
        sqlx::query_as::<QCWorkflow, _>(
            r#"
            INSERT INTO qc_workflows (
                workflow_id, name, description, product_category, checkpoint_ids
            ) VALUES ($1, $2, $3, $4, $5)
            RETURNING
                id, workflow_id, name, description, product_category,
                checkpoint_ids, is_active, created_at, updated_at
            "#,
        )
        .bind(workflow.workflow_id)
        .bind(workflow.name)
        .bind(workflow.description)
        .bind(workflow.product_category)
        .bind(&workflow.checkpoint_ids)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_workflow(&self, workflow_id: &str) -> Result<Option<QCWorkflow>, sqlx::Error> {
        sqlx::query_as::<QCWorkflow, _>(
            "SELECT id, workflow_id, name, description, product_category, checkpoint_ids, is_active, created_at, updated_at FROM qc_workflows WHERE workflow_id = $1",
        )
        .bind(workflow_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_workflows(
        &self,
        is_active: Option<bool>,
    ) -> Result<Vec<QCWorkflow>, sqlx::Error> {
        if let Some(active) = is_active {
            sqlx::query_as::<QCWorkflow, _>(
                "SELECT id, workflow_id, name, description, product_category, checkpoint_ids, is_active, created_at, updated_at FROM qc_workflows WHERE is_active = $1 ORDER BY created_at DESC",
            )
            .bind(active)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<QCWorkflow, _>(
                "SELECT id, workflow_id, name, description, product_category, checkpoint_ids, is_active, created_at, updated_at FROM qc_workflows ORDER BY created_at DESC"
            )
            .fetch_all(&self.pool)
            .await
        }
    }

    // QC Inspections
    pub async fn create_inspection(
        &self,
        inspection: NewQCInspection,
    ) -> Result<QCInspection, sqlx::Error> {
        sqlx::query_as::<QCInspection, _>(
            r#"
            INSERT INTO qc_inspections (
                inspection_id, product_id, checkpoint_id, workflow_id,
                inspector_id, location, results, quality_metrics, notes, evidence_documents
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING
                id, inspection_id, product_id, checkpoint_id, workflow_id,
                inspector_id, inspection_date, status, is_passed, failure_reason,
                location, results, quality_metrics, notes, evidence_documents, created_at
            "#,
        )
        .bind(inspection.inspection_id)
        .bind(inspection.product_id)
        .bind(inspection.checkpoint_id)
        .bind(inspection.workflow_id)
        .bind(inspection.inspector_id)
        .bind(inspection.location)
        .bind(inspection.results)
        .bind(inspection.quality_metrics)
        .bind(inspection.notes)
        .bind(&inspection.evidence_documents)
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
        sqlx::query_as::<QCInspection, _>(
            r#"
            UPDATE qc_inspections SET
                status = $2,
                inspection_date = NOW(),
                is_passed = $3,
                failure_reason = $4
            WHERE inspection_id = $1
            RETURNING
                id, inspection_id, product_id, checkpoint_id, workflow_id,
                inspector_id, inspection_date, status, is_passed, failure_reason,
                location, results, quality_metrics, notes, evidence_documents, created_at
            "#,
        )
        .bind(inspection_id)
        .bind(status)
        .bind(is_passed)
        .bind(failure_reason)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_inspection(
        &self,
        inspection_id: &str,
    ) -> Result<Option<QCInspection>, sqlx::Error> {
        sqlx::query_as::<QCInspection, _>(
            "SELECT id, inspection_id, product_id, checkpoint_id, workflow_id, inspector_id, inspection_date, status, is_passed, failure_reason, location, results, quality_metrics, notes, evidence_documents, created_at FROM qc_inspections WHERE inspection_id = $1",
        )
        .bind(inspection_id)
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
                sqlx::query_as::<QCInspection, _>(
                    "SELECT id, inspection_id, product_id, checkpoint_id, workflow_id, inspector_id, inspection_date, status, is_passed, failure_reason, location, results, quality_metrics, notes, evidence_documents, created_at FROM qc_inspections WHERE product_id = $1 AND checkpoint_id = $2 AND status = $3 ORDER BY inspection_date DESC LIMIT $4",
                )
                .bind(pid)
                .bind(cid)
                .bind(s)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            (Some(pid), Some(cid), None) => {
                sqlx::query_as::<QCInspection, _>(
                    "SELECT id, inspection_id, product_id, checkpoint_id, workflow_id, inspector_id, inspection_date, status, is_passed, failure_reason, location, results, quality_metrics, notes, evidence_documents, created_at FROM qc_inspections WHERE product_id = $1 AND checkpoint_id = $2 ORDER BY inspection_date DESC LIMIT $3",
                )
                .bind(pid)
                .bind(cid)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            (Some(pid), None, Some(s)) => {
                sqlx::query_as::<QCInspection, _>(
                    "SELECT id, inspection_id, product_id, checkpoint_id, workflow_id, inspector_id, inspection_date, status, is_passed, failure_reason, location, results, quality_metrics, notes, evidence_documents, created_at FROM qc_inspections WHERE product_id = $1 AND status = $2 ORDER BY inspection_date DESC LIMIT $3",
                )
                .bind(pid)
                .bind(s)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(cid), Some(s)) => {
                sqlx::query_as::<QCInspection, _>(
                    "SELECT id, inspection_id, product_id, checkpoint_id, workflow_id, inspector_id, inspection_date, status, is_passed, failure_reason, location, results, quality_metrics, notes, evidence_documents, created_at FROM qc_inspections WHERE checkpoint_id = $1 AND status = $2 ORDER BY inspection_date DESC LIMIT $3",
                )
                .bind(cid)
                .bind(s)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            (Some(pid), None, None) => {
                sqlx::query_as::<QCInspection, _>(
                    "SELECT id, inspection_id, product_id, checkpoint_id, workflow_id, inspector_id, inspection_date, status, is_passed, failure_reason, location, results, quality_metrics, notes, evidence_documents, created_at FROM qc_inspections WHERE product_id = $1 ORDER BY inspection_date DESC LIMIT $2",
                )
                .bind(pid)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(cid), None) => {
                sqlx::query_as::<QCInspection, _>(
                    "SELECT id, inspection_id, product_id, checkpoint_id, workflow_id, inspector_id, inspection_date, status, is_passed, failure_reason, location, results, quality_metrics, notes, evidence_documents, created_at FROM qc_inspections WHERE checkpoint_id = $1 ORDER BY inspection_date DESC LIMIT $2",
                )
                .bind(cid)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            (None, None, Some(s)) => {
                sqlx::query_as::<QCInspection, _>(
                    "SELECT id, inspection_id, product_id, checkpoint_id, workflow_id, inspector_id, inspection_date, status, is_passed, failure_reason, location, results, quality_metrics, notes, evidence_documents, created_at FROM qc_inspections WHERE status = $1 ORDER BY inspection_date DESC LIMIT $2",
                )
                .bind(s)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            (None, None, None) => {
                sqlx::query_as::<QCInspection, _>(
                    "SELECT id, inspection_id, product_id, checkpoint_id, workflow_id, inspector_id, inspection_date, status, is_passed, failure_reason, location, results, quality_metrics, notes, evidence_documents, created_at FROM qc_inspections ORDER BY inspection_date DESC LIMIT $1",
                )
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
        }
    }

    // Execute Workflow
    pub async fn execute_workflow(
        &self,
        request: WorkflowExecutionRequest,
    ) -> Result<WorkflowExecutionResult, Box<dyn std::error::Error>> {
        let workflow = self
            .get_workflow(&request.workflow_id)
            .await?
            .ok_or_else(|| "Workflow not found")?;

        let mut inspections = Vec::new();
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        for checkpoint_id in &workflow.checkpoint_ids {
            let inspection_id = format!("INS-{}", Uuid::new_v4());

            let inspection = self
                .create_inspection(NewQCInspection {
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
                })
                .await?;

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
    pub async fn create_non_conformance(
        &self,
        nc: NewNonConformance,
    ) -> Result<NonConformance, sqlx::Error> {
        sqlx::query_as::<NonConformance, _>(
            r#"
            INSERT INTO non_conformances (
                nc_id, inspection_id, product_id, severity, category,
                description, root_cause, correction_action, responsible_party, due_date
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING
                id, nc_id, inspection_id, product_id, severity, category,
                description, root_cause, correction_action, correction_status,
                responsible_party, due_date, resolved_at, verified_by, verified_at, created_at
            "#,
        )
        .bind(nc.nc_id)
        .bind(nc.inspection_id)
        .bind(nc.product_id)
        .bind(nc.severity)
        .bind(nc.category)
        .bind(nc.description)
        .bind(nc.root_cause)
        .bind(nc.correction_action)
        .bind(nc.responsible_party)
        .bind(nc.due_date)
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

        sqlx::query_as::<NonConformance, _>(
            r#"
            UPDATE non_conformances SET
                correction_action = $2,
                correction_status = $3,
                responsible_party = $4,
                resolved_at = $5
            WHERE nc_id = $1
            RETURNING
                id, nc_id, inspection_id, product_id, severity, category,
                description, root_cause, correction_action, correction_status,
                responsible_party, due_date, resolved_at, verified_by, verified_at, created_at
            "#,
        )
        .bind(nc_id)
        .bind(correction_action)
        .bind(correction_status)
        .bind(responsible_party)
        .bind(resolved_at)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn verify_non_conformance(
        &self,
        nc_id: &str,
        verified_by: String,
    ) -> Result<NonConformance, sqlx::Error> {
        sqlx::query_as::<NonConformance, _>(
            r#"
            UPDATE non_conformances SET
                correction_status = 'verified',
                verified_by = $2,
                verified_at = NOW()
            WHERE nc_id = $1
            RETURNING
                id, nc_id, inspection_id, product_id, severity, category,
                description, root_cause, correction_action, correction_status,
                responsible_party, due_date, resolved_at, verified_by, verified_at, created_at
            "#,
        )
        .bind(nc_id)
        .bind(verified_by)
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
                sqlx::query_as::<NonConformance, _>(
                    "SELECT id, nc_id, inspection_id, product_id, severity, category, description, root_cause, correction_action, correction_status, responsible_party, due_date, resolved_at, verified_by, verified_at, created_at FROM non_conformances WHERE product_id = $1 AND severity = $2 AND correction_status = $3 ORDER BY created_at DESC LIMIT $4",
                )
                .bind(pid)
                .bind(s)
                .bind(st)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            (Some(pid), Some(s), None) => {
                sqlx::query_as::<NonConformance, _>(
                    "SELECT id, nc_id, inspection_id, product_id, severity, category, description, root_cause, correction_action, correction_status, responsible_party, due_date, resolved_at, verified_by, verified_at, created_at FROM non_conformances WHERE product_id = $1 AND severity = $2 ORDER BY created_at DESC LIMIT $3",
                )
                .bind(pid)
                .bind(s)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            (Some(pid), None, Some(st)) => {
                sqlx::query_as::<NonConformance, _>(
                    "SELECT id, nc_id, inspection_id, product_id, severity, category, description, root_cause, correction_action, correction_status, responsible_party, due_date, resolved_at, verified_by, verified_at, created_at FROM non_conformances WHERE product_id = $1 AND correction_status = $2 ORDER BY created_at DESC LIMIT $3",
                )
                .bind(pid)
                .bind(st)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(s), Some(st)) => {
                sqlx::query_as::<NonConformance, _>(
                    "SELECT id, nc_id, inspection_id, product_id, severity, category, description, root_cause, correction_action, correction_status, responsible_party, due_date, resolved_at, verified_by, verified_at, created_at FROM non_conformances WHERE severity = $1 AND correction_status = $2 ORDER BY created_at DESC LIMIT $3",
                )
                .bind(s)
                .bind(st)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            (Some(pid), None, None) => {
                sqlx::query_as::<NonConformance, _>(
                    "SELECT id, nc_id, inspection_id, product_id, severity, category, description, root_cause, correction_action, correction_status, responsible_party, due_date, resolved_at, verified_by, verified_at, created_at FROM non_conformances WHERE product_id = $1 ORDER BY created_at DESC LIMIT $2",
                )
                .bind(pid)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(s), None) => {
                sqlx::query_as::<NonConformance, _>(
                    "SELECT id, nc_id, inspection_id, product_id, severity, category, description, root_cause, correction_action, correction_status, responsible_party, due_date, resolved_at, verified_by, verified_at, created_at FROM non_conformances WHERE severity = $1 ORDER BY created_at DESC LIMIT $2",
                )
                .bind(s)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            (None, None, Some(st)) => {
                sqlx::query_as::<NonConformance, _>(
                    "SELECT id, nc_id, inspection_id, product_id, severity, category, description, root_cause, correction_action, correction_status, responsible_party, due_date, resolved_at, verified_by, verified_at, created_at FROM non_conformances WHERE correction_status = $1 ORDER BY created_at DESC LIMIT $2",
                )
                .bind(st)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            (None, None, None) => {
                sqlx::query_as::<NonConformance, _>(
                    "SELECT id, nc_id, inspection_id, product_id, severity, category, description, root_cause, correction_action, correction_status, responsible_party, due_date, resolved_at, verified_by, verified_at, created_at FROM non_conformances ORDER BY created_at DESC LIMIT $1",
                )
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
        }
    }

    // Quality Metrics
    pub async fn create_metric(
        &self,
        metric: NewQualityMetric,
    ) -> Result<QualityMetric, sqlx::Error> {
        // Check if within threshold
        let is_within_threshold =
            if let (Some(min), Some(max)) = (metric.threshold_min, metric.threshold_max) {
                Some(metric.metric_value >= min && metric.metric_value <= max)
            } else if let Some(min) = metric.threshold_min {
                Some(metric.metric_value >= min)
            } else if let Some(max) = metric.threshold_max {
                Some(metric.metric_value <= max)
            } else {
                None
            };

        sqlx::query_as::<QualityMetric, _>(
            r#"
            INSERT INTO quality_metrics (
                metric_id, product_id, metric_type, metric_value, unit,
                measurement_period_start, measurement_period_end,
                target_value, threshold_min, threshold_max, is_within_threshold, notes
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING
                id, metric_id, product_id, metric_type, metric_value, unit,
                measurement_period_start, measurement_period_end,
                target_value, threshold_min, threshold_max, is_within_threshold, notes, created_at
            "#,
        )
        .bind(metric.metric_id)
        .bind(metric.product_id)
        .bind(metric.metric_type)
        .bind(metric.metric_value)
        .bind(metric.unit)
        .bind(metric.measurement_period_start)
        .bind(metric.measurement_period_end)
        .bind(metric.target_value)
        .bind(metric.threshold_min)
        .bind(metric.threshold_max)
        .bind(is_within_threshold)
        .bind(metric.notes)
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
                sqlx::query_as::<QualityMetric, _>(
                    "SELECT id, metric_id, product_id, metric_type, metric_value, unit, measurement_period_start, measurement_period_end, target_value, threshold_min, threshold_max, is_within_threshold, notes, created_at FROM quality_metrics WHERE product_id = $1 AND metric_type = $2 ORDER BY measurement_period_start DESC LIMIT $3",
                )
                .bind(pid)
                .bind(mt)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            (Some(pid), None) => {
                sqlx::query_as::<QualityMetric, _>(
                    "SELECT id, metric_id, product_id, metric_type, metric_value, unit, measurement_period_start, measurement_period_end, target_value, threshold_min, threshold_max, is_within_threshold, notes, created_at FROM quality_metrics WHERE product_id = $1 ORDER BY measurement_period_start DESC LIMIT $2",
                )
                .bind(pid)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(mt)) => {
                sqlx::query_as::<QualityMetric, _>(
                    "SELECT id, metric_id, product_id, metric_type, metric_value, unit, measurement_period_start, measurement_period_end, target_value, threshold_min, threshold_max, is_within_threshold, notes, created_at FROM quality_metrics WHERE metric_type = $1 ORDER BY measurement_period_start DESC LIMIT $2",
                )
                .bind(mt)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            (None, None) => {
                sqlx::query_as::<QualityMetric, _>(
                    "SELECT id, metric_id, product_id, metric_type, metric_value, unit, measurement_period_start, measurement_period_end, target_value, threshold_min, threshold_max, is_within_threshold, notes, created_at FROM quality_metrics ORDER BY measurement_period_start DESC LIMIT $1",
                )
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
        }
    }
}
