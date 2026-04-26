-- Sustainability metrics and verification tables

-- IoT sensor readings for real-time monitoring
CREATE TABLE IF NOT EXISTS iot_readings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    sensor_id VARCHAR(100) NOT NULL,
    metric_type VARCHAR(50) NOT NULL, -- 'energy', 'water', 'carbon', 'waste'
    value DECIMAL(16, 4) NOT NULL,
    unit VARCHAR(20) NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

-- Sustainability metrics tracking
CREATE TABLE IF NOT EXISTS sustainability_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    metric_type VARCHAR(50) NOT NULL, -- 'carbon_footprint', 'water_usage', 'labor_compliance', 'waste_management', 'renewable_energy'
    value DECIMAL(16, 4) NOT NULL,
    unit VARCHAR(20) NOT NULL,
    verified BOOLEAN DEFAULT FALSE,
    last_updated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

-- Sustainability verifications and certifications
CREATE TABLE IF NOT EXISTS sustainability_verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    metric_type VARCHAR(50) NOT NULL,
    verifier_name VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- 'pending', 'verified', 'rejected'
    certificate_hash VARCHAR(64),
    certificate_url TEXT,
    blockchain_tx_hash VARCHAR(66),
    verified_at TIMESTAMP WITH TIME ZONE,
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_iot_readings_product_id ON iot_readings(product_id);
CREATE INDEX idx_iot_readings_timestamp ON iot_readings(timestamp);
CREATE INDEX idx_sustainability_metrics_product_id ON sustainability_metrics(product_id);
CREATE INDEX idx_sustainability_verifications_product_id ON sustainability_verifications(product_id);
