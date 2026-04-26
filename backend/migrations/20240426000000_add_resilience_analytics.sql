-- Migration to add supply chain resilience analytics tables

-- Risk Assessments table
CREATE TABLE IF NOT EXISTS risk_assessments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type TEXT NOT NULL, -- 'supplier', 'route', 'product', 'location'
    entity_id TEXT NOT NULL,
    risk_score FLOAT NOT NULL, -- 0.0 to 1.0 (1.0 is highest risk)
    risk_level TEXT NOT NULL, -- 'low', 'medium', 'high', 'critical'
    factors JSONB NOT NULL, -- detailed breakdown of risk factors (e.g., weather: 0.8, geopolitical: 0.2)
    last_assessment_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    next_assessment_due TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Disruption Alerts table
CREATE TABLE IF NOT EXISTS disruption_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    alert_type TEXT NOT NULL, -- 'predicted', 'active', 'resolved'
    severity TEXT NOT NULL, -- 'minor', 'major', 'catastrophic'
    description TEXT NOT NULL,
    probability FLOAT NOT NULL, -- 0.0 to 1.0
    estimated_impact_usd FLOAT,
    start_time TIMESTAMPTZ,
    end_time TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Resilience Plans table
CREATE TABLE IF NOT EXISTS resilience_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_id UUID REFERENCES disruption_alerts(id),
    product_id TEXT NOT NULL,
    mitigation_strategies TEXT[] NOT NULL,
    backup_suppliers JSONB NOT NULL, -- list of backup suppliers with scores
    alternative_routes JSONB NOT NULL, -- list of alternative routes
    safety_stock_recommendation FLOAT,
    status TEXT NOT NULL, -- 'draft', 'active', 'completed'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_risk_assessments_entity ON risk_assessments(entity_type, entity_id);
CREATE INDEX idx_disruption_alerts_entity ON disruption_alerts(entity_type, entity_id);
CREATE INDEX idx_disruption_alerts_status ON disruption_alerts(alert_type);
CREATE INDEX idx_resilience_plans_product ON resilience_plans(product_id);
