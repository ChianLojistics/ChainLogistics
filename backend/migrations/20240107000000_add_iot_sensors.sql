-- Add IoT temperature sensor tables

-- IoT devices table
CREATE TABLE iot_devices (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    device_id TEXT NOT NULL UNIQUE,
    device_type TEXT NOT NULL, -- temperature, humidity, gps, etc.
    product_id TEXT REFERENCES products(id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    description TEXT,
    manufacturer TEXT,
    model TEXT,
    serial_number TEXT,
    firmware_version TEXT,
    location TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_seen_at TIMESTAMP WITH TIME ZONE,
    calibration_date TIMESTAMP WITH TIME ZONE,
    next_calibration_date TIMESTAMP WITH TIME ZONE,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Temperature readings table
CREATE TABLE temperature_readings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    device_id TEXT NOT NULL REFERENCES iot_devices(device_id) ON DELETE CASCADE,
    product_id TEXT REFERENCES products(id) ON DELETE CASCADE,
    temperature_celsius DECIMAL(10, 2) NOT NULL,
    humidity_percent DECIMAL(5, 2),
    unit TEXT NOT NULL DEFAULT 'celsius',
    reading_timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    received_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    location TEXT,
    quality_score DECIMAL(3, 2), -- 0-1 score indicating data quality
    is_anomaly BOOLEAN NOT NULL DEFAULT false,
    anomaly_reason TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Temperature thresholds table
CREATE TABLE temperature_thresholds (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    device_id TEXT REFERENCES iot_devices(device_id) ON DELETE CASCADE,
    threshold_type TEXT NOT NULL, -- min, max, critical_min, critical_max
    min_temperature_celsius DECIMAL(10, 2),
    max_temperature_celsius DECIMAL(10, 2),
    duration_minutes INTEGER, -- How long threshold must be breached to trigger alert
    alert_level TEXT NOT NULL DEFAULT 'warning', -- info, warning, critical
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(product_id, device_id, threshold_type)
);

-- Temperature alerts table
CREATE TABLE temperature_alerts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    device_id TEXT NOT NULL REFERENCES iot_devices(device_id) ON DELETE CASCADE,
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    threshold_id UUID REFERENCES temperature_thresholds(id) ON DELETE SET NULL,
    alert_type TEXT NOT NULL, -- threshold_breach, anomaly, device_offline, calibration_due
    alert_level TEXT NOT NULL, -- info, warning, critical
    temperature_celsius DECIMAL(10, 2),
    threshold_value DECIMAL(10, 2),
    message TEXT NOT NULL,
    is_resolved BOOLEAN NOT NULL DEFAULT false,
    resolved_at TIMESTAMP WITH TIME ZONE,
    resolved_by TEXT,
    acknowledged_by TEXT,
    acknowledged_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Historical temperature summary table (for reporting)
CREATE TABLE temperature_summaries (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    device_id TEXT NOT NULL REFERENCES iot_devices(device_id) ON DELETE CASCADE,
    product_id TEXT REFERENCES products(id) ON DELETE CASCADE,
    summary_period TEXT NOT NULL, -- hourly, daily, weekly, monthly
    period_start TIMESTAMP WITH TIME ZONE NOT NULL,
    period_end TIMESTAMP WITH TIME ZONE NOT NULL,
    avg_temperature_celsius DECIMAL(10, 2),
    min_temperature_celsius DECIMAL(10, 2),
    max_temperature_celsius DECIMAL(10, 2),
    total_readings INTEGER,
    anomaly_count INTEGER,
    alert_count INTEGER,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(device_id, summary_period, period_start, period_end)
);

-- Create indexes
CREATE INDEX idx_iot_devices_device_id ON iot_devices(device_id);
CREATE INDEX idx_iot_devices_product_id ON iot_devices(product_id);
CREATE INDEX idx_iot_devices_device_type ON iot_devices(device_type);
CREATE INDEX idx_iot_devices_is_active ON iot_devices(is_active);

CREATE INDEX idx_temperature_readings_device_id ON temperature_readings(device_id);
CREATE INDEX idx_temperature_readings_product_id ON temperature_readings(product_id);
CREATE INDEX idx_temperature_readings_reading_timestamp ON temperature_readings(reading_timestamp);
CREATE INDEX idx_temperature_readings_is_anomaly ON temperature_readings(is_anomaly);

CREATE INDEX idx_temperature_thresholds_product_id ON temperature_thresholds(product_id);
CREATE INDEX idx_temperature_thresholds_device_id ON temperature_thresholds(device_id);
CREATE INDEX idx_temperature_thresholds_is_active ON temperature_thresholds(is_active);

CREATE INDEX idx_temperature_alerts_device_id ON temperature_alerts(device_id);
CREATE INDEX idx_temperature_alerts_product_id ON temperature_alerts(product_id);
CREATE INDEX idx_temperature_alerts_is_resolved ON temperature_alerts(is_resolved);
CREATE INDEX idx_temperature_alerts_created_at ON temperature_alerts(created_at);
CREATE INDEX idx_temperature_alerts_alert_level ON temperature_alerts(alert_level);

CREATE INDEX idx_temperature_summaries_device_id ON temperature_summaries(device_id);
CREATE INDEX idx_temperature_summaries_product_id ON temperature_summaries(product_id);
CREATE INDEX idx_temperature_summaries_period ON temperature_summaries(summary_period, period_start, period_end);

-- Create trigger to update updated_at timestamp
CREATE TRIGGER update_iot_devices_updated_at BEFORE UPDATE ON iot_devices
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_temperature_thresholds_updated_at BEFORE UPDATE ON temperature_thresholds
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
