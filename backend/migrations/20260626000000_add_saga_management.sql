-- Add saga management tables for distributed transaction orchestration

-- Create enum for saga states
CREATE TYPE saga_state AS ENUM ('Pending', 'InProgress', 'Compensating', 'Completed', 'Failed', 'Aborted');

-- Saga instances table
CREATE TABLE saga_instances (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    saga_type TEXT NOT NULL,
    state saga_state NOT NULL DEFAULT 'Pending',
    current_step TEXT,
    completed_steps TEXT[] DEFAULT '{}',
    failed_step TEXT,
    context JSONB NOT NULL DEFAULT '{}',
    error_message TEXT,
    started_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    metadata JSONB DEFAULT '{}'
);

-- Create indexes for saga queries
CREATE INDEX idx_saga_instances_state ON saga_instances(state);
CREATE INDEX idx_saga_instances_type ON saga_instances(saga_type);
CREATE INDEX idx_saga_instances_started_at ON saga_instances(started_at);
CREATE INDEX idx_saga_instances_updated_at ON saga_instances(updated_at);

-- Create trigger for updated_at
CREATE OR REPLACE FUNCTION update_saga_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_saga_updated_at
    BEFORE UPDATE ON saga_instances
    FOR EACH ROW
    EXECUTE FUNCTION update_saga_updated_at();

-- Add comments
COMMENT ON TABLE saga_instances IS 'Stores saga instances for distributed transaction orchestration';
COMMENT ON COLUMN saga_instances.saga_type IS 'Type of saga (e.g., product_registration)';
COMMENT ON COLUMN saga_instances.state IS 'Current state of the saga';
COMMENT ON COLUMN saga_instances.current_step IS 'ID of the currently executing step';
COMMENT ON COLUMN saga_instances.completed_steps IS 'List of completed step IDs';
COMMENT ON COLUMN saga_instances.failed_step IS 'ID of the step that failed (if any)';
COMMENT ON COLUMN saga_instances.context IS 'Execution context and data';
COMMENT ON COLUMN saga_instances.error_message IS 'Error message if saga failed';
