# ADR-003: PostgreSQL as Primary Database

## Status
Accepted

## Context

ChainLogistics requires a robust database solution to store supply chain data, user information, and transaction records. We needed to evaluate database options based on our specific requirements:

### Requirements
- ACID compliance for data integrity
- Strong consistency for supply chain data
- Complex query capabilities for analytics
- JSON support for flexible metadata
- Full-text search capabilities
- Good performance for read-heavy workloads
- Strong ecosystem and tooling
- Reliable backup and recovery options

### Evaluated Options
1. **MongoDB**: Flexible schema but weaker consistency guarantees
2. **MySQL**: Good performance but limited JSON capabilities
3. **PostgreSQL**: ACID compliant, excellent JSON support, mature ecosystem
4. **Cassandra**: High scalability but complex for our use case
5. **SQLite**: Simple but not suitable for production scale

## Decision

We selected **PostgreSQL** as our primary database for the following reasons:

### Key Advantages
1. **ACID Compliance**: Ensures data integrity for critical supply chain transactions
2. **JSON Support**: Native JSONB type for flexible metadata storage
3. **Full-Text Search**: Built-in search capabilities with GIN indexes
4. **Mature Ecosystem**: Extensive tooling, monitoring, and community support
5. **Strong Consistency**: Essential for supply chain data accuracy
6. **Performance**: Excellent performance for complex queries
7. **Extensibility**: Support for custom functions and extensions

### Specific Fit for Supply Chain
- **Data Integrity**: ACID properties prevent data corruption
- **Complex Queries**: Advanced analytics and reporting capabilities
- **Flexibility**: JSONB for custom fields and metadata
- **Search**: Full-text search for product discovery
- **Reliability**: Proven track record in production environments

## Consequences

### Positive Consequences
- **Data Integrity**: Strong consistency guarantees prevent data corruption
- **Query Power**: Complex queries for analytics and reporting
- **Flexibility**: JSONB allows schema evolution without migrations
- **Performance**: Excellent performance for both OLTP and OLAP workloads
- **Ecosystem**: Rich tooling for monitoring, backup, and development
- **Reliability**: Proven reliability in production environments

### Negative Consequences
- **Vertical Scaling**: Requires vertical scaling for write-heavy workloads
- **Complexity**: More complex than NoSQL alternatives
- **Schema Management**: Requires careful migration planning
- **Resource Usage**: Higher memory usage compared to some alternatives

### Mitigation Strategies
- **Connection Pooling**: Efficient connection management
- **Read Replicas**: Offload read queries to replicas
- **Caching**: Redis caching for frequently accessed data
- **Monitoring**: Comprehensive monitoring for performance optimization
- **Migration Tools**: Automated migration tools for schema changes

## Implementation

### Database Schema Design
```sql
-- Core product table with JSONB for flexibility
CREATE TABLE products (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    origin_location VARCHAR(500),
    category VARCHAR(100),
    tags TEXT[],
    certifications TEXT[],
    media_hashes TEXT[],
    custom_fields JSONB,
    owner_address VARCHAR(255) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_by VARCHAR(255),
    updated_by VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_products_owner ON products(owner_address);
CREATE INDEX idx_products_category ON products(category);
CREATE INDEX idx_products_active ON products(is_active);
CREATE INDEX idx_products_created ON products(created_at);
CREATE INDEX idx_products_custom_fields ON products USING GIN(custom_fields);

-- Full-text search index
CREATE INDEX idx_products_search ON products USING GIN(
    to_tsvector('english', name || ' ' || COALESCE(description, '') || ' ' || category)
);

-- Tracking events table
CREATE TABLE tracking_events (
    id BIGSERIAL PRIMARY KEY,
    product_id VARCHAR(255) REFERENCES products(id) ON DELETE CASCADE,
    actor_address VARCHAR(255) NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    location VARCHAR(500),
    data_hash VARCHAR(255),
    note TEXT,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Event indexes
CREATE INDEX idx_events_product ON tracking_events(product_id);
CREATE INDEX idx_events_timestamp ON tracking_events(timestamp);
CREATE INDEX idx_events_type ON tracking_events(event_type);
CREATE INDEX idx_events_actor ON tracking_events(actor_address);
```

### Advanced Query Examples
```sql
-- Complex product search with full-text search
SELECT 
    p.*,
    ts_rank(
        to_tsvector('english', p.name || ' ' || COALESCE(p.description, '') || ' ' || p.category),
        plainto_tsquery('english', 'ethiopian coffee')
    ) as relevance_score
FROM products p
WHERE 
    to_tsvector('english', p.name || ' ' || COALESCE(p.description, '') || ' ' || p.category)
    @@ plainto_tsquery('english', 'ethiopian coffee')
    OR p.name ILIKE '%ethiopian coffee%'
ORDER BY relevance_score DESC, p.created_at DESC
LIMIT 20;

-- JSONB queries for custom fields
SELECT 
    id,
    name,
    custom_fields->>'altitude' as altitude,
    custom_fields->>'processing_method' as processing_method
FROM products
WHERE 
    custom_fields->>'altitude' IS NOT NULL
    AND (custom_fields->>'altitude')::numeric > 1500;

-- Analytics query with window functions
SELECT 
    DATE_TRUNC('month', created_at) as month,
    COUNT(*) as products_created,
    COUNT(*) FILTER (WHERE category = 'coffee') as coffee_products,
    COUNT(*) FILTER (WHERE category = 'tea') as tea_products
FROM products
WHERE created_at >= NOW() - INTERVAL '12 months'
GROUP BY DATE_TRUNC('month', created_at)
ORDER BY month DESC;
```

### Connection and Pool Configuration
```rust
// Database connection setup with SQLx
use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn create_database_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await
}
```

### Migration Strategy
```sql
-- Example migration file
-- Migration: 001_initial_schema.sql
BEGIN;

-- Create products table
CREATE TABLE products (
    -- Schema as defined above
);

-- Create tracking events table
CREATE TABLE tracking_events (
    -- Schema as defined above
);

-- Create indexes
CREATE INDEX CONCURRENTLY idx_products_owner ON products(owner_address);
-- ... other indexes

COMMIT;

-- Migration: 002_add_product_search.sql
BEGIN;

-- Add full-text search capabilities
ALTER TABLE products ADD COLUMN search_vector tsvector;
UPDATE products SET search_vector = to_tsvector('english', name || ' ' || COALESCE(description, '') || ' ' || category);
CREATE INDEX CONCURRENTLY idx_products_search_vector ON products USING GIN(search_vector);

-- Create trigger for automatic search vector updates
CREATE OR REPLACE FUNCTION update_product_search_vector()
RETURNS TRIGGER AS $$
BEGIN
    NEW.search_vector = to_tsvector('english', NEW.name || ' ' || COALESCE(NEW.description, '') || ' ' || NEW.category);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER product_search_vector_update
    BEFORE INSERT OR UPDATE ON products
    FOR EACH ROW EXECUTE FUNCTION update_product_search_vector();

COMMIT;
```

### Performance Optimization
```sql
-- Partitioning for large event tables
CREATE TABLE tracking_events_y2024m01 PARTITION OF tracking_events
FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');

CREATE TABLE tracking_events_y2024m02 PARTITION OF tracking_events
FOR VALUES FROM ('2024-02-01') TO ('2024-03-01');

-- Materialized views for analytics
CREATE MATERIALIZED VIEW product_stats AS
SELECT 
    p.id,
    p.name,
    p.category,
    COUNT(e.id) as event_count,
    MAX(e.timestamp) as last_event_at,
    MIN(e.timestamp) as first_event_at
FROM products p
LEFT JOIN tracking_events e ON p.id = e.product_id
GROUP BY p.id, p.name, p.category;

-- Refresh materialized view periodically
CREATE OR REPLACE FUNCTION refresh_product_stats()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY product_stats;
END;
$$ LANGUAGE plpgsql;

-- Schedule refresh with pg_cron
SELECT cron.schedule('refresh-product-stats', '0 */6 * * *', 'SELECT refresh_product_stats();');
```

## Monitoring and Maintenance

### Performance Monitoring
```sql
-- Query performance monitoring
SELECT 
    query,
    calls,
    total_time,
    mean_time,
    rows
FROM pg_stat_statements
ORDER BY total_time DESC
LIMIT 10;

-- Index usage monitoring
SELECT 
    schemaname,
    tablename,
    indexname,
    idx_scan,
    idx_tup_read,
    idx_tup_fetch
FROM pg_stat_user_indexes
ORDER BY idx_scan DESC;
```

### Backup Strategy
```bash
# Automated backup script
#!/bin/bash
BACKUP_DIR="/backups/postgresql"
DATE=$(date +%Y%m%d_%H%M%S)
DB_NAME="chainlogistics"

# Create backup
pg_dump -h localhost -U postgres -d $DB_NAME -f "$BACKUP_DIR/backup_$DATE.sql"

# Compress backup
gzip "$BACKUP_DIR/backup_$DATE.sql"

# Remove old backups (keep last 7 days)
find $BACKUP_DIR -name "backup_*.sql.gz" -mtime +7 -delete

# Verify backup
pg_restore --list "$BACKUP_DIR/backup_$DATE.sql.gz" > /dev/null
if [ $? -eq 0 ]; then
    echo "Backup successful: backup_$DATE.sql.gz"
else
    echo "Backup failed: backup_$DATE.sql.gz"
    exit 1
fi
```

## Scaling Strategy

### Read Replicas
```rust
// Read replica configuration
use sqlx::{PgPool, postgres::PgPoolOptions};

pub struct DatabaseConfig {
    pub primary: PgPool,
    pub replicas: Vec<PgPool>,
}

impl DatabaseConfig {
    pub async fn new(primary_url: &str, replica_urls: &[String]) -> Result<Self, sqlx::Error> {
        let primary = PgPoolOptions::new()
            .max_connections(10)
            .connect(primary_url)
            .await?;
            
        let mut replicas = Vec::new();
        for url in replica_urls {
            let replica = PgPoolOptions::new()
                .max_connections(5)
                .connect(url)
                .await?;
            replicas.push(replica);
        }
        
        Ok(Self { primary, replicas })
    }
    
    pub fn get_read_pool(&self) -> &PgPool {
        // Simple round-robin for read replicas
        let index = rand::thread_rng().gen_range(0..self.replicas.len());
        &self.replicas[index]
    }
}
```

## References

- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [PostgreSQL Performance Tuning](https://wiki.postgresql.org/wiki/Tuning_Your_PostgreSQL_Server)
- [JSONB Performance](https://www.postgresql.org/docs/current/datatype-json.html)

## Related ADRs

- [ADR-002: Rust Backend Selection](./ADR-002-rust-backend-selection.md)
- [ADR-008: Caching Strategy with Redis](./ADR-008-caching-strategy.md)
