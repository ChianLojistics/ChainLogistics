use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use crate::database::{ProductRepository, EventRepository, UserRepository, ApiKeyRepository, ProductFilters, GlobalStats};
use crate::models::*;
use bcrypt::{hash, DEFAULT_COST};

pub mod financial;
pub use financial::FinancialService;

pub mod analytics_service;
pub use analytics_service::AnalyticsService;

pub struct ProductService {
    pool: PgPool,
}

impl ProductService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductRepository for ProductService {
    async fn create_product(&self, product: NewProduct) -> Result<Product, sqlx::Error> {
        sqlx::query_as!(
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
        .await
    }

    async fn get_product(&self, id: &str) -> Result<Option<Product>, sqlx::Error> {
        sqlx::query_as!(
            Product,
            "SELECT * FROM products WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
    }

    async fn update_product(&self, id: &str, product: Product) -> Result<Product, sqlx::Error> {
        sqlx::query_as!(
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
        .await
    }

    async fn delete_product(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM products WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
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

        query.push_str(&format!(" ORDER BY created_at DESC LIMIT ${} OFFSET ${}", bind_index, bind_index + 1));
        bindings.push(limit.to_string());
        bindings.push(offset.to_string());

        // Build dynamic query
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

pub struct EventService {
    pool: PgPool,
}

impl EventService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventRepository for EventService {
    async fn create_event(&self, event: NewTrackingEvent) -> Result<TrackingEvent, sqlx::Error> {
        sqlx::query_as!(
            TrackingEvent,
            r#"
            INSERT INTO tracking_events (
                product_id, actor_address, timestamp, event_type,
                location, data_hash, note, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            event.product_id,
            event.actor_address,
            event.timestamp,
            event.event_type,
            event.location,
            event.data_hash,
            event.note,
            event.metadata
        )
        .fetch_one(&self.pool)
        .await
    }

    async fn get_event(&self, id: i64) -> Result<Option<TrackingEvent>, sqlx::Error> {
        sqlx::query_as!(
            TrackingEvent,
            "SELECT * FROM tracking_events WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
    }

    async fn list_events_by_product(
        &self,
        product_id: &str,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<TrackingEvent>, sqlx::Error> {
        sqlx::query_as!(
            TrackingEvent,
            "SELECT * FROM tracking_events WHERE product_id = $1 ORDER BY timestamp DESC LIMIT $2 OFFSET $3",
            product_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn count_events_by_product(&self, product_id: &str) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM tracking_events WHERE product_id = $1",
            product_id
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0)
    }

    async fn list_events_by_type(
        &self,
        product_id: &str,
        event_type: &str,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<TrackingEvent>, sqlx::Error> {
        sqlx::query_as!(
            TrackingEvent,
            "SELECT * FROM tracking_events WHERE product_id = $1 AND event_type = $2 ORDER BY timestamp DESC LIMIT $3 OFFSET $4",
            product_id,
            event_type,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn get_product_stats(&self, product_id: &str) -> Result<Option<ProductStats>, sqlx::Error> {
        sqlx::query_as!(
            ProductStats,
            r#"
            SELECT 
                p.id as product_id,
                COALESCE(e.event_count, 0) as event_count,
                p.is_active,
                e.last_event_at,
                e.last_event_type
            FROM products p
            LEFT JOIN (
                SELECT 
                    product_id,
                    COUNT(*) as event_count,
                    MAX(timestamp) as last_event_at,
                    (event_type) as last_event_type
                FROM tracking_events 
                WHERE product_id = $1
                GROUP BY product_id
            ) e ON p.id = e.product_id
            WHERE p.id = $1
            "#,
            product_id
        )
        .fetch_optional(&self.pool)
        .await
    }

    async fn get_global_stats(&self) -> Result<GlobalStats, sqlx::Error> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                (SELECT COUNT(*) FROM products) as total_products,
                (SELECT COUNT(*) FROM products WHERE is_active = true) as active_products,
                (SELECT COUNT(*) FROM tracking_events) as total_events,
                (SELECT COUNT(*) FROM users) as total_users,
                (SELECT COUNT(*) FROM api_keys WHERE is_active = true) as active_api_keys
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(GlobalStats {
            total_products: stats.total_products.unwrap_or(0),
            active_products: stats.active_products.unwrap_or(0),
            total_events: stats.total_events.unwrap_or(0),
            total_users: stats.total_users.unwrap_or(0),
            active_api_keys: stats.active_api_keys.unwrap_or(0),
        })
    }
}

pub struct UserService {
    pool: PgPool,
}

impl UserService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
        hash(password, DEFAULT_COST)
    }

    pub async fn generate_api_key() -> String {
        format!("cl_{}", uuid::Uuid::new_v4().to_string().replace("-", ""))
    }
}

#[async_trait]
impl UserRepository for UserService {
    async fn create_user(&self, user: NewUser) -> Result<User, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (email, password_hash, stellar_address)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            user.email,
            user.password_hash,
            user.stellar_address
        )
        .fetch_one(&self.pool)
        .await
    }

    async fn get_user(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email = $1",
            email
        )
        .fetch_optional(&self.pool)
        .await
    }

    async fn get_user_by_stellar_address(&self, address: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE stellar_address = $1",
            address
        )
        .fetch_optional(&self.pool)
        .await
    }

    async fn update_user(&self, id: Uuid, user: User) -> Result<User, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"
            UPDATE users SET
                email = $2,
                password_hash = $3,
                stellar_address = $4,
                api_key = $5,
                api_key_hash = $6,
                is_active = $7,
                is_admin = $8
            WHERE id = $1
            RETURNING *
            "#,
            id,
            user.email,
            user.password_hash,
            user.stellar_address,
            user.api_key,
            user.api_key_hash,
            user.is_active,
            user.is_admin
        )
        .fetch_one(&self.pool)
        .await
    }

    async fn update_last_login(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE users SET last_login_at = NOW() WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

pub struct ApiKeyService {
    pool: PgPool,
}

impl ApiKeyService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn hash_api_key(api_key: &str) -> Result<String, bcrypt::BcryptError> {
        hash(api_key, DEFAULT_COST)
    }
}

#[async_trait]
impl ApiKeyRepository for ApiKeyService {
    async fn create_api_key(&self, api_key: NewApiKey) -> Result<ApiKey, sqlx::Error> {
        sqlx::query_as!(
            ApiKey,
            r#"
            INSERT INTO api_keys (user_id, key_hash, name, tier, rate_limit_per_minute, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
            api_key.user_id,
            api_key.key_hash,
            api_key.name,
            api_key.tier as ApiKeyTier,
            api_key.rate_limit_per_minute,
            api_key.expires_at
        )
        .fetch_one(&self.pool)
        .await
    }

    async fn get_api_key(&self, id: Uuid) -> Result<Option<ApiKey>, sqlx::Error> {
        sqlx::query_as!(
            ApiKey,
            "SELECT * FROM api_keys WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
    }

    async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>, sqlx::Error> {
        sqlx::query_as!(
            ApiKey,
            "SELECT * FROM api_keys WHERE key_hash = $1 AND is_active = true",
            key_hash
        )
        .fetch_optional(&self.pool)
        .await
    }

    async fn list_api_keys(&self, user_id: Uuid) -> Result<Vec<ApiKey>, sqlx::Error> {
        sqlx::query_as!(
            ApiKey,
            "SELECT * FROM api_keys WHERE user_id = $1 ORDER BY created_at DESC",
            user_id
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn update_api_key(&self, id: Uuid, api_key: ApiKey) -> Result<ApiKey, sqlx::Error> {
        sqlx::query_as!(
            ApiKey,
            r#"
            UPDATE api_keys SET
                name = $2,
                tier = $3,
                rate_limit_per_minute = $4,
                is_active = $5,
                expires_at = $6
            WHERE id = $1
            RETURNING *
            "#,
            id,
            api_key.name,
            api_key.tier as ApiKeyTier,
            api_key.rate_limit_per_minute,
            api_key.is_active,
            api_key.expires_at
        )
        .fetch_one(&self.pool)
        .await
    }

    async fn update_last_used(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE api_keys SET last_used_at = NOW() WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn revoke_api_key(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE api_keys SET is_active = false WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

// Sync service for mirroring smart contract data
pub struct SyncService {
    pool: PgPool,
    product_service: ProductService,
    event_service: EventService,
}

impl SyncService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: pool.clone(),
            product_service: ProductService::new(pool.clone()),
            event_service: EventService::new(pool),
        }
    }

    pub async fn sync_product_from_contract(&self, product: NewProduct) -> Result<Product, sqlx::Error> {
        // Upsert product
        let existing = self.product_service.get_product(&product.id).await?;
        
        if let Some(mut existing_product) = existing {
            // Update existing product
            existing_product.name = product.name.clone();
            existing_product.description = product.description.clone();
            existing_product.origin_location = product.origin_location.clone();
            existing_product.category = product.category.clone();
            existing_product.tags = product.tags.clone();
            existing_product.certifications = product.certifications.clone();
            existing_product.media_hashes = product.media_hashes.clone();
            existing_product.custom_fields = product.custom_fields.clone();
            existing_product.owner_address = product.owner_address.clone();
            existing_product.updated_by = product.created_by.clone();
            
            self.product_service.update_product(&product.id, existing_product).await
        } else {
            // Create new product
            self.product_service.create_product(product).await
        }
    }

    pub async fn sync_event_from_contract(&self, event: NewTrackingEvent) -> Result<TrackingEvent, sqlx::Error> {
        self.event_service.create_event(event).await
    }

    pub async fn sync_batch_products(&self, products: Vec<NewProduct>) -> Result<Vec<Product>, sqlx::Error> {
        let mut results = Vec::new();
        for product in products {
            results.push(self.sync_product_from_contract(product).await?);
        }
        Ok(results)
    }

    pub async fn sync_batch_events(&self, events: Vec<NewTrackingEvent>) -> Result<Vec<TrackingEvent>, sqlx::Error> {
        let mut results = Vec::new();
        for event in events {
            results.push(self.sync_event_from_contract(event).await?);
        }
        Ok(results)
    }
}