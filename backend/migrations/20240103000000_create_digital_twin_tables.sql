-- Digital Twin Tables Migration
-- This migration creates tables for digital twin functionality including
-- simulations, predictions, optimizations, and state tracking

-- Digital Twins table
CREATE TABLE IF NOT EXISTS digital_twins (
    id UUID PRIMARY KEY,
    product_id VARCHAR(255) NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    twin_type TEXT NOT NULL CHECK (twin_type IN ('Product', 'Warehouse', 'Vehicle', 'Container', 'Facility')),
    name VARCHAR(500) NOT NULL,
    description TEXT,
    current_state JSONB NOT NULL DEFAULT '{}',
    metadata JSONB NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_sync_at TIMESTAMP WITH TIME ZONE
);

-- Indexes for digital_twins
CREATE INDEX idx_digital_twins_product ON digital_twins(product_id);
CREATE INDEX idx_digital_twins_type ON digital_twins(twin_type);
CREATE INDEX idx_digital_twins_active ON digital_twins(is_active) WHERE is_active = true;
CREATE INDEX idx_digital_twins_created ON digital_twins(created_at DESC);

-- Twin States table (historical state tracking)
CREATE TABLE IF NOT EXISTS twin_states (
    id UUID PRIMARY KEY,
    twin_id UUID NOT NULL REFERENCES digital_twins(id) ON DELETE CASCADE,
    state_data JSONB NOT NULL,
    metrics JSONB NOT NULL DEFAULT '{}',
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    source VARCHAR(255) NOT NULL
);

-- Indexes for twin_states
CREATE INDEX idx_twin_states_twin ON twin_states(twin_id);
CREATE INDEX idx_twin_states_timestamp ON twin_states(timestamp DESC);
CREATE INDEX idx_twin_states_twin_time ON twin_states(twin_id, timestamp DESC);

-- Simulations table
CREATE TABLE IF NOT EXISTS simulations (
    id UUID PRIMARY KEY,
    twin_id UUID NOT NULL REFERENCES digital_twins(id) ON DELETE CASCADE,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    simulation_type TEXT NOT NULL CHECK (simulation_type IN (
        'RouteOptimization',
        'DemandForecasting',
        'InventoryOptimization',
        'RiskAssessment',
        'CostAnalysis',
        'TimelineProjection'
    )),
    parameters JSONB NOT NULL DEFAULT '{}',
    status TEXT NOT NULL CHECK (status IN ('Pending', 'Running', 'Completed', 'Failed', 'Cancelled')),
    results JSONB,
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by VARCHAR(255) NOT NULL
);

-- Indexes for simulations
CREATE INDEX idx_simulations_twin ON simulations(twin_id);
CREATE INDEX idx_simulations_status ON simulations(status);
CREATE INDEX idx_simulations_type ON simulations(simulation_type);
CREATE INDEX idx_simulations_created ON simulations(created_at DESC);
CREATE INDEX idx_simulations_twin_created ON simulations(twin_id, created_at DESC);

-- Predictions table
CREATE TABLE IF NOT EXISTS predictions (
    id UUID PRIMARY KEY,
    twin_id UUID NOT NULL REFERENCES digital_twins(id) ON DELETE CASCADE,
    prediction_type TEXT NOT NULL CHECK (prediction_type IN (
        'ArrivalTime',
        'Delay',
        'QualityIssue',
        'DemandSpike',
        'SupplyDisruption',
        'CostOverrun'
    )),
    predicted_value JSONB NOT NULL,
    confidence_score DOUBLE PRECISION NOT NULL CHECK (confidence_score >= 0 AND confidence_score <= 1),
    prediction_horizon INTEGER NOT NULL, -- hours
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    valid_until TIMESTAMP WITH TIME ZONE NOT NULL,
    actual_value JSONB,
    accuracy_score DOUBLE PRECISION CHECK (accuracy_score >= 0 AND accuracy_score <= 1)
);

-- Indexes for predictions
CREATE INDEX idx_predictions_twin ON predictions(twin_id);
CREATE INDEX idx_predictions_type ON predictions(prediction_type);
CREATE INDEX idx_predictions_created ON predictions(created_at DESC);
CREATE INDEX idx_predictions_valid ON predictions(valid_until) WHERE actual_value IS NULL;
CREATE INDEX idx_predictions_twin_created ON predictions(twin_id, created_at DESC);

-- Optimizations table
CREATE TABLE IF NOT EXISTS optimizations (
    id UUID PRIMARY KEY,
    twin_id UUID NOT NULL REFERENCES digital_twins(id) ON DELETE CASCADE,
    optimization_type TEXT NOT NULL CHECK (optimization_type IN (
        'Route',
        'Inventory',
        'Cost',
        'Time',
        'Carbon',
        'Risk'
    )),
    current_metrics JSONB NOT NULL,
    optimized_metrics JSONB NOT NULL,
    recommendations TEXT[] NOT NULL DEFAULT '{}',
    estimated_savings DOUBLE PRECISION,
    implementation_complexity TEXT NOT NULL CHECK (implementation_complexity IN ('Low', 'Medium', 'High')),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    applied_at TIMESTAMP WITH TIME ZONE
);

-- Indexes for optimizations
CREATE INDEX idx_optimizations_twin ON optimizations(twin_id);
CREATE INDEX idx_optimizations_type ON optimizations(optimization_type);
CREATE INDEX idx_optimizations_created ON optimizations(created_at DESC);
CREATE INDEX idx_optimizations_applied ON optimizations(applied_at) WHERE applied_at IS NOT NULL;
CREATE INDEX idx_optimizations_pending ON optimizations(twin_id) WHERE applied_at IS NULL;

-- Comments for documentation
COMMENT ON TABLE digital_twins IS 'Virtual replicas of physical supply chain entities for simulation and optimization';
COMMENT ON TABLE twin_states IS 'Historical state snapshots of digital twins for time-series analysis';
COMMENT ON TABLE simulations IS 'What-if scenarios for supply chain optimization and planning';
COMMENT ON TABLE predictions IS 'AI/ML-based predictions for supply chain events and metrics';
COMMENT ON TABLE optimizations IS 'Optimization recommendations and their implementation status';

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_digital_twin_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to automatically update updated_at
CREATE TRIGGER trigger_update_digital_twin_updated_at
    BEFORE UPDATE ON digital_twins
    FOR EACH ROW
    EXECUTE FUNCTION update_digital_twin_updated_at();

-- Function to clean up old twin states (keep last 1000 per twin)
CREATE OR REPLACE FUNCTION cleanup_old_twin_states()
RETURNS void AS $$
BEGIN
    DELETE FROM twin_states
    WHERE id IN (
        SELECT id FROM (
            SELECT id, ROW_NUMBER() OVER (PARTITION BY twin_id ORDER BY timestamp DESC) as rn
            FROM twin_states
        ) t
        WHERE t.rn > 1000
    );
END;
$$ LANGUAGE plpgsql;

-- Grant permissions (adjust as needed for your setup)
-- GRANT SELECT, INSERT, UPDATE, DELETE ON digital_twins TO chainlog_app;
-- GRANT SELECT, INSERT, UPDATE, DELETE ON twin_states TO chainlog_app;
-- GRANT SELECT, INSERT, UPDATE, DELETE ON simulations TO chainlog_app;
-- GRANT SELECT, INSERT, UPDATE, DELETE ON predictions TO chainlog_app;
-- GRANT SELECT, INSERT, UPDATE, DELETE ON optimizations TO chainlog_app;
