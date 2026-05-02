-- Scalability improvements: composite indexes for high-frequency query patterns

-- Events ordered by product + time (most common access pattern for product history)
CREATE INDEX IF NOT EXISTS idx_tracking_events_product_timestamp
    ON tracking_events(product_id, timestamp DESC);

-- Events filtered by product + type (used by event_type filter queries)
CREATE INDEX IF NOT EXISTS idx_tracking_events_product_type_timestamp
    ON tracking_events(product_id, event_type, timestamp DESC);

-- Active products sorted by creation time (paginated list queries)
CREATE INDEX IF NOT EXISTS idx_products_active_created
    ON products(is_active, created_at DESC);

-- Active API keys only — the auth hot path only ever queries active keys
CREATE INDEX IF NOT EXISTS idx_api_keys_active_hash
    ON api_keys(key_hash)
    WHERE is_active = true;

-- Keys by user filtered to active — used by key-management list endpoint
CREATE INDEX IF NOT EXISTS idx_api_keys_user_active
    ON api_keys(user_id, is_active, created_at DESC);

-- Full-text search index on products for name/description/category queries
CREATE INDEX IF NOT EXISTS idx_products_fts
    ON products
    USING GIN(to_tsvector('english', name || ' ' || COALESCE(description, '') || ' ' || category));
