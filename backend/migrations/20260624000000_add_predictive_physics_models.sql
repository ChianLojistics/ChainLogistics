-- Add Predictive Physics Models for Digital Twins
-- This migration adds support for biology/decay models, accuracy tracking, 
-- dynamic expiry updates, and Monte Carlo simulation results

-- Add decay model parameters to digital_twins
ALTER TABLE digital_twins ADD COLUMN IF NOT EXISTS decay_model_params JSONB DEFAULT '{}';
ALTER TABLE digital_twins ADD COLUMN IF NOT EXISTS predicted_expiry_date TIMESTAMP WITH TIME ZONE;
ALTER TABLE digital_twins ADD COLUMN IF NOT EXISTS current_health_score DOUBLE PRECISION DEFAULT 1.0;
ALTER TABLE digital_twins ADD COLUMN IF NOT EXISTS health_history JSONB DEFAULT '[]';

-- Add Monte Carlo simulation results to simulations
ALTER TABLE simulations ADD COLUMN IF NOT EXISTS monte_carlo_runs INTEGER DEFAULT 100;
ALTER TABLE simulations ADD COLUMN IF NOT EXISTS confidence_interval JSONB;
ALTER TABLE simulations ADD COLUMN IF NOT EXISTS confidence_level DOUBLE PRECISION DEFAULT 0.95;

-- Add accuracy tracking to predictions
ALTER TABLE predictions ADD COLUMN IF NOT EXISTS prediction_metadata JSONB DEFAULT '{}';
ALTER TABLE predictions ADD COLUMN IF NOT EXISTS calibration_data JSONB DEFAULT '{}';
ALTER TABLE predictions ADD COLUMN IF NOT EXISTS model_version VARCHAR(50);

-- Create prediction accuracy audit table
CREATE TABLE IF NOT EXISTS prediction_accuracy_audit (
    id UUID PRIMARY KEY,
    twin_id UUID NOT NULL REFERENCES digital_twins(id) ON DELETE CASCADE,
    prediction_id UUID REFERENCES predictions(id) ON DELETE SET NULL,
    prediction_type TEXT NOT NULL,
    predicted_value JSONB NOT NULL,
    actual_value JSONB NOT NULL,
    accuracy_score DOUBLE PRECISION NOT NULL CHECK (accuracy_score >= 0 AND accuracy_score <= 1),
    error_magnitude DOUBLE PRECISION,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

-- Indexes for prediction_accuracy_audit
CREATE INDEX IF NOT EXISTS idx_prediction_audit_twin ON prediction_accuracy_audit(twin_id);
CREATE INDEX IF NOT EXISTS idx_prediction_audit_type ON prediction_accuracy_audit(prediction_type);
CREATE INDEX IF NOT EXISTS idx_prediction_audit_timestamp ON prediction_accuracy_audit(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_prediction_audit_score ON prediction_accuracy_audit(accuracy_score);

-- Create IoT to Twin integration table
CREATE TABLE IF NOT EXISTS iot_twin_sync (
    id UUID PRIMARY KEY,
    device_id VARCHAR(255) NOT NULL,
    twin_id UUID NOT NULL REFERENCES digital_twins(id) ON DELETE CASCADE,
    sync_type TEXT NOT NULL CHECK (sync_type IN ('temperature', 'humidity', 'location', 'quality', 'decay')),
    last_sync_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    sync_frequency_seconds INTEGER DEFAULT 300,
    is_active BOOLEAN NOT NULL DEFAULT true,
    sync_parameters JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(device_id, twin_id, sync_type)
);

-- Indexes for iot_twin_sync
CREATE INDEX IF NOT EXISTS idx_iot_twin_sync_device ON iot_twin_sync(device_id);
CREATE INDEX IF NOT EXISTS idx_iot_twin_sync_twin ON iot_twin_sync(twin_id);
CREATE INDEX IF NOT EXISTS idx_iot_twin_sync_active ON iot_twin_sync(is_active) WHERE is_active = true;

-- Create health gauge data table
CREATE TABLE IF NOT EXISTS twin_health_metrics (
    id UUID PRIMARY KEY,
    twin_id UUID NOT NULL REFERENCES digital_twins(id) ON DELETE CASCADE,
    metric_type TEXT NOT NULL CHECK (metric_type IN ('decay_rate', 'quality', 'temperature_stress', 'humidity_stress', 'overall_health')),
    metric_value DOUBLE PRECISION NOT NULL,
    threshold_min DOUBLE PRECISION,
    threshold_max DOUBLE PRECISION,
    severity TEXT CHECK (severity IN ('normal', 'warning', 'critical')),
    calculated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

-- Indexes for twin_health_metrics
CREATE INDEX IF NOT EXISTS idx_twin_health_twin ON twin_health_metrics(twin_id);
CREATE INDEX IF NOT EXISTS idx_twin_health_type ON twin_health_metrics(metric_type);
CREATE INDEX IF NOT EXISTS idx_twin_health_calculated ON twin_health_metrics(calculated_at DESC);
CREATE INDEX IF NOT EXISTS idx_twin_health_severity ON twin_health_metrics(severity);

-- Add expiry tracking to products if not exists
ALTER TABLE products ADD COLUMN IF NOT EXISTS base_expiry_date TIMESTAMP WITH TIME ZONE;
ALTER TABLE products ADD COLUMN IF NOT EXISTS dynamic_expiry_date TIMESTAMP WITH TIME ZONE;
ALTER TABLE products ADD COLUMN IF NOT EXISTS expiry_adjustment_reason TEXT;

-- Trigger to update updated_at for iot_twin_sync
CREATE OR REPLACE FUNCTION update_iot_twin_sync_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER IF NOT EXISTS trigger_update_iot_twin_sync_updated_at
    BEFORE UPDATE ON iot_twin_sync
    FOR EACH ROW
    EXECUTE FUNCTION update_iot_twin_sync_updated_at();

-- Comments for documentation
COMMENT ON COLUMN digital_twins.decay_model_params IS 'Parameters for biology/decay physics models (e.g., decay rates, temperature coefficients)';
COMMENT ON COLUMN digital_twins.predicted_expiry_date IS 'Dynamically calculated expiry based on decay model and IoT readings';
COMMENT ON COLUMN digital_twins.current_health_score IS 'Current health score (0-1) based on decay model and sensor readings';
COMMENT ON COLUMN digital_twins.health_history IS 'Historical health scores for trend analysis';
COMMENT ON TABLE prediction_accuracy_audit IS 'Audit trail for prediction accuracy vs actual outcomes';
COMMENT ON TABLE iot_twin_sync IS 'Configuration for live IoT to digital twin synchronization';
COMMENT ON TABLE twin_health_metrics IS 'Detailed health metrics for the health gauge visualization';
