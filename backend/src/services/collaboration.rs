use sqlx::PgPool;
use uuid::Uuid;
use crate::error::AppError;
use crate::models::collaboration::*;

pub struct CollaborationService {
    pool: PgPool,
}

impl CollaborationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn share_product(
        &self,
        actor_id: Uuid,
        req: ShareProductRequest,
    ) -> Result<ProductShare, AppError> {
        let share = sqlx::query_as::<ProductShare, _>(
            r#"
            INSERT INTO product_shares (product_id, shared_with_user_id, permission_level)
            VALUES ($1, $2, $3)
            RETURNING id, product_id, shared_with_user_id, permission_level, created_at, updated_at
            "#,
        )
        .bind(req.product_id)
        .bind(req.shared_with_user_id)
        .bind(req.permission_level)
        .fetch_one(&self.pool)
        .await?;

        self.log_audit(
            Some(actor_id),
            "share_product",
            "product",
            &req.product_id,
            serde_json::json!({ "shared_with": req.shared_with_user_id, "permission": req.permission_level })
        ).await?;

        Ok(share)
    }

    pub async fn list_shares(&self, product_id: &str) -> Result<Vec<ProductShare>, AppError> {
        let shares = sqlx::query_as::<ProductShare, _>(
            "SELECT id, product_id, shared_with_user_id, permission_level, created_at, updated_at FROM product_shares WHERE product_id = $1",
        )
        .bind(product_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(shares)
    }

    pub async fn create_collaboration_request(
        &self,
        requester_id: Uuid,
        req: CreateCollaborationRequest,
    ) -> Result<CollaborationRequest, AppError> {
        let request = sqlx::query_as::<CollaborationRequest, _>(
            r#"
            INSERT INTO collaboration_requests (product_id, requester_id, status)
            VALUES ($1, $2, 'pending')
            RETURNING id, product_id, requester_id, status, created_at, updated_at
            "#,
        )
        .bind(req.product_id)
        .bind(requester_id)
        .fetch_one(&self.pool)
        .await?;

        self.log_audit(
            Some(requester_id),
            "create_collaboration_request",
            "product",
            &req.product_id,
            serde_json::json!({})
        ).await?;

        Ok(request)
    }

    pub async fn update_collaboration_request(
        &self,
        actor_id: Uuid,
        request_id: Uuid,
        status: &str,
    ) -> Result<CollaborationRequest, AppError> {
        let updated = sqlx::query_as::<CollaborationRequest, _>(
            r#"
            UPDATE collaboration_requests
            SET status = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING id, product_id, requester_id, status, created_at, updated_at
            "#,
        )
        .bind(request_id)
        .bind(status)
        .fetch_one(&self.pool)
        .await?;

        self.log_audit(
            Some(actor_id),
            "update_collaboration_request",
            "collaboration_request",
            &request_id.to_string(),
            serde_json::json!({ "new_status": status })
        ).await?;

        Ok(updated)
    }

    pub async fn list_audit_trail(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<CollaborationAuditTrail>, AppError> {
        let trails = sqlx::query_as::<CollaborationAuditTrail, _>(
            "SELECT id, actor_id, action, entity_type, entity_id, details, created_at FROM collaboration_audit_trails WHERE entity_type = $1 AND entity_id = $2 ORDER BY created_at DESC",
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(trails)
    }

    async fn log_audit(
        &self,
        actor_id: Option<Uuid>,
        action: &str,
        entity_type: &str,
        entity_id: &str,
        details: serde_json::Value,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO collaboration_audit_trails (actor_id, action, entity_type, entity_id, details)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(actor_id)
        .bind(action)
        .bind(entity_type)
        .bind(entity_id)
        .bind(details)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
