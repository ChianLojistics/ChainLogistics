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
        let share = sqlx::query_as!(
            ProductShare,
            r#"
            INSERT INTO product_shares (product_id, shared_with_user_id, permission_level)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            req.product_id,
            req.shared_with_user_id,
            req.permission_level
        )
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
        let shares = sqlx::query_as!(
            ProductShare,
            "SELECT * FROM product_shares WHERE product_id = $1",
            product_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(shares)
    }

    pub async fn create_collaboration_request(
        &self,
        requester_id: Uuid,
        req: CreateCollaborationRequest,
    ) -> Result<CollaborationRequest, AppError> {
        let request = sqlx::query_as!(
            CollaborationRequest,
            r#"
            INSERT INTO collaboration_requests (product_id, requester_id, status)
            VALUES ($1, $2, 'pending')
            RETURNING *
            "#,
            req.product_id,
            requester_id
        )
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
        let updated = sqlx::query_as!(
            CollaborationRequest,
            r#"
            UPDATE collaboration_requests
            SET status = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
            request_id,
            status
        )
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
        let trails = sqlx::query_as!(
            CollaborationAuditTrail,
            "SELECT * FROM collaboration_audit_trails WHERE entity_type = $1 AND entity_id = $2 ORDER BY created_at DESC",
            entity_type,
            entity_id
        )
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
        sqlx::query!(
            r#"
            INSERT INTO collaboration_audit_trails (actor_id, action, entity_type, entity_id, details)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            actor_id,
            action,
            entity_type,
            entity_id,
            details
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
