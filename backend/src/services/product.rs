use async_trait::async_trait;
use sqlx::PgPool;
use redis::AsyncCommands;
use crate::database::{ProductRepository, ProductFilters};
use crate::models::{Product, NewProduct, AppError};

pub struct ProductService {
    pub(crate) pool: PgPool,
    pub(crate) redis_client: redis::Client,
}

impl ProductService {
    pub fn new(pool: PgPool, redis_client: redis::Client) -> Self {
        Self { pool, redis_client }
    }

    pub async fn invalidate_product_cache(&self, id: &str) -> Result<(), AppError> {
        if let Ok(mut conn) = self.redis_client.get_multiplexed_tokio_connection().await {
            let _: Result<(), _> = conn.del(format!("cache:product:{}", id)).await;
        }
        Ok(())
    }

    pub async fn invalidate_global_stats(&self) -> Result<(), AppError> {
        if let Ok(mut conn) = self.redis_client.get_multiplexed_tokio_connection().await {
            let _: Result<(), _> = conn.del("cache:global_stats").await;
        }
        Ok(())
    }
}

#[async_trait]
impl ProductRepository for ProductService {
    async fn create_product(&self, product: NewProduct) -> Result<Product, sqlx::Error> {
        let created = sqlx::query_as!(
            Product,
            r#"
            INSERT INTO products (
                id, name, description, origin_location, category, tags,
                certifications, media_hashes, custom_fields, owner_address,
                is_active, created_by, updated_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, true, $11, $11)
            RETURNING *
            "#,
            product.id,
            product.name,
            product.description,
            product.origin_location,
            product.category,
            &product.tags,
            &product.certifications,
            &product.media_hashes,
            product.custom_fields,
            product.owner_address,
            product.created_by
        )
        .fetch_one(&self.pool)
        .await?;

        let _ = self.invalidate_global_stats().await;

        Ok(created)
    }

    async fn get_product(&self, id: &str) -> Result<Option<Product>, sqlx::Error> {
        let cache_key = format!("cache:product:{}", id);

        if let Ok(mut conn) = self.redis_client.get_multiplexed_tokio_connection().await {
            if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
                if let Ok(product) = serde_json::from_str::<Product>(&cached) {
                    return Ok(Some(product));
                }
            }
        }

        let product = sqlx::query_as!(
            Product,
            "SELECT * FROM products WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref p) = product {
            if let Ok(mut conn) = self.redis_client.get_multiplexed_tokio_connection().await {
                if let Ok(serialized) = serde_json::to_string(p) {
                    let _: Result<(), _> = conn.set_ex(&cache_key, serialized, 3600).await;
                }
            }
        }

        Ok(product)
    }

    async fn update_product(&self, id: &str, product: Product) -> Result<Product, sqlx::Error> {
        let updated = sqlx::query_as!(
            Product,
            r#"
            UPDATE products SET
                name = $2,
                description = $3,
                origin_location = $4,
                category = $5,
                tags = $6,
                certifications = $7,
                media_hashes = $8,
                custom_fields = $9,
                owner_address = $10,
                is_active = $11,
                updated_by = $12
            WHERE id = $1
            RETURNING *
            "#,
            id,
            product.name,
            product.description,
            product.origin_location,
            product.category,
            &product.tags,
            &product.certifications,
            &product.media_hashes,
            product.custom_fields,
            product.owner_address,
            product.is_active,
            product.updated_by
        )
        .fetch_one(&self.pool)
        .await?;

        let _ = self.invalidate_product_cache(id).await;
        let _ = self.invalidate_global_stats().await;

        Ok(updated)
    }

    async fn delete_product(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM products WHERE id = $1", id)
            .execute(&self.pool)
            .await?;

        let _ = self.invalidate_product_cache(id).await;
        let _ = self.invalidate_global_stats().await;

        Ok(())
    }

    async fn list_products(
        &self,
        offset: i64,
        limit: i64,
        filters: Option<ProductFilters>,
    ) -> Result<Vec<Product>, sqlx::Error> {
        let mut query = "SELECT * FROM products WHERE 1=1".to_string();
        let mut bindings = Vec::new();
        let mut bind_index = 1;

        if let Some(f) = filters {
            if let Some(owner) = f.owner_address {
                query.push_str(&format!(" AND owner_address = ${}", bind_index));
                bindings.push(owner);
                bind_index += 1;
            }
            if let Some(category) = f.category {
                query.push_str(&format!(" AND category = ${}", bind_index));
                bindings.push(category);
                bind_index += 1;
            }
            if let Some(is_active) = f.is_active {
                query.push_str(&format!(" AND is_active = ${}", bind_index));
                bindings.push(is_active.to_string());
                bind_index += 1;
            }
            if let Some(after) = f.created_after {
                query.push_str(&format!(" AND created_at >= ${}", bind_index));
                bindings.push(after.to_rfc3339());
                bind_index += 1;
            }
            if let Some(before) = f.created_before {
                query.push_str(&format!(" AND created_at <= ${}", bind_index));
                bindings.push(before.to_rfc3339());
                bind_index += 1;
            }
        }

        query.push_str(&format!(
            " ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            bind_index,
            bind_index + 1
        ));
        bindings.push(limit.to_string());
        bindings.push(offset.to_string());

        let mut q = sqlx::QueryBuilder::new(query);
        for binding in bindings {
            q = q.bind(binding);
        }

        q.build_query_as::<Product>()
            .fetch_all(&self.pool)
            .await
    }

    async fn count_products(&self, filters: Option<ProductFilters>) -> Result<i64, sqlx::Error> {
        let mut query = "SELECT COUNT(*) FROM products WHERE 1=1".to_string();
        let mut bindings = Vec::new();
        let mut bind_index = 1;

        if let Some(f) = filters {
            if let Some(owner) = f.owner_address {
                query.push_str(&format!(" AND owner_address = ${}", bind_index));
                bindings.push(owner);
                bind_index += 1;
            }
            if let Some(category) = f.category {
                query.push_str(&format!(" AND category = ${}", bind_index));
                bindings.push(category);
                bind_index += 1;
            }
            if let Some(is_active) = f.is_active {
                query.push_str(&format!(" AND is_active = ${}", bind_index));
                bindings.push(is_active.to_string());
                bind_index += 1;
            }
            if let Some(after) = f.created_after {
                query.push_str(&format!(" AND created_at >= ${}", bind_index));
                bindings.push(after.to_rfc3339());
                bind_index += 1;
            }
            if let Some(before) = f.created_before {
                query.push_str(&format!(" AND created_at <= ${}", bind_index));
                bindings.push(before.to_rfc3339());
                bind_index += 1;
            }
        }

        let mut q = sqlx::QueryBuilder::new(query);
        for binding in bindings {
            q = q.bind(binding);
        }

        q.build_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
    }

    async fn search_products(&self, query: &str, limit: i64) -> Result<Vec<Product>, sqlx::Error> {
        sqlx::query_as!(
            Product,
            r#"
            SELECT * FROM products
            WHERE
                to_tsvector('english', name || ' ' || COALESCE(description, '') || ' ' || category)
                @@ plainto_tsquery('english', $1)
                OR name ILIKE $2
                OR id ILIKE $2
            ORDER BY ts_rank(to_tsvector('english', name || ' ' || COALESCE(description, '') || ' ' || category), plainto_tsquery('english', $1)) DESC
            LIMIT $3
            "#,
            query,
            format!("%{}%", query),
            limit
        )
        .fetch_all(&self.pool)
        .await
    }
}
