-- Add structured, tamper-evident audit logging fields without dropping existing logs.

ALTER TABLE audit_logs
    ADD COLUMN IF NOT EXISTS correlation_id TEXT,
    ADD COLUMN IF NOT EXISTS event_category VARCHAR(32),
    ADD COLUMN IF NOT EXISTS event_type VARCHAR(64),
    ADD COLUMN IF NOT EXISTS severity VARCHAR(16),
    ADD COLUMN IF NOT EXISTS actor_api_key_id UUID REFERENCES api_keys(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS target_resource_id TEXT,
    ADD COLUMN IF NOT EXISTS http_method VARCHAR(8),
    ADD COLUMN IF NOT EXISTS http_path TEXT,
    ADD COLUMN IF NOT EXISTS http_status SMALLINT,
    ADD COLUMN IF NOT EXISTS success BOOLEAN,
    ADD COLUMN IF NOT EXISTS error_code VARCHAR(64),
    ADD COLUMN IF NOT EXISTS business_context TEXT,
    ADD COLUMN IF NOT EXISTS prev_hash BYTEA,
    ADD COLUMN IF NOT EXISTS row_hash BYTEA,
    ADD COLUMN IF NOT EXISTS retention_days INTEGER NOT NULL DEFAULT 365;

ALTER TABLE audit_logs
    ALTER COLUMN created_at TYPE TIMESTAMPTZ
    USING created_at AT TIME ZONE 'UTC';

UPDATE audit_logs
SET target_resource_id = COALESCE(target_resource_id, resource_id::TEXT)
WHERE target_resource_id IS NULL;

CREATE INDEX IF NOT EXISTS idx_audit_logs_correlation_id ON audit_logs(correlation_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_event_category ON audit_logs(event_category);
CREATE INDEX IF NOT EXISTS idx_audit_logs_event_type ON audit_logs(event_type);
CREATE INDEX IF NOT EXISTS idx_audit_logs_severity ON audit_logs(severity);
CREATE INDEX IF NOT EXISTS idx_audit_logs_actor_api_key_id ON audit_logs(actor_api_key_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_http_status ON audit_logs(http_status);
CREATE INDEX IF NOT EXISTS idx_audit_logs_success ON audit_logs(success);
CREATE INDEX IF NOT EXISTS idx_audit_logs_target_resource_id ON audit_logs(target_resource_id);
