-- Saga state management tables
CREATE TABLE IF NOT EXISTS saga_states (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL,
    steps JSONB NOT NULL,
    current_step_index INTEGER NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    metadata JSONB DEFAULT '{}',
    
    INDEX idx_saga_status (status),
    INDEX idx_saga_updated (updated_at)
);

-- Rule engine tables
CREATE TABLE IF NOT EXISTS rule_sets (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    rules JSONB NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    INDEX idx_ruleset_name (name)
);

-- Event processing metrics
CREATE TABLE IF NOT EXISTS event_processing_metrics (
    id BIGSERIAL PRIMARY KEY,
    event_id VARCHAR(255) NOT NULL,
    processing_time_ms INTEGER NOT NULL,
    processed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    worker_id VARCHAR(255),
    success BOOLEAN NOT NULL,
    error_message TEXT,
    
    INDEX idx_metrics_event (event_id),
    INDEX idx_metrics_processed (processed_at),
    INDEX idx_metrics_worker (worker_id)
);
