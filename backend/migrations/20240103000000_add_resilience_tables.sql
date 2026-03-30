"""-- Create Disruption Predictions Table
CREATE TABLE disruption_predictions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id VARCHAR(255) NOT NULL,
    predicted_at TIMESTAMPTZ NOT NULL,
    probability FLOAT NOT NULL,
    impact_level VARCHAR(50) NOT NULL,
    details JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create Supplier Risks Table
CREATE TABLE supplier_risks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    supplier_name VARCHAR(255) NOT NULL,
    risk_score FLOAT NOT NULL,
    risk_factors JSONB NOT NULL,
    last_assessed_at TIMESTAMPTZ NOT NULL
);

-- Create Geographic Risks Table
CREATE TABLE geographic_risks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    location VARCHAR(255) NOT NULL,
    risk_score FLOAT NOT NULL,
    risk_factors JSONB NOT NULL,
    last_assessed_at TIMESTAMPTZ NOT NULL
);

-- Create Alternative Sources Table
CREATE TABLE alternative_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id VARCHAR(255) NOT NULL,
    alternative_supplier VARCHAR(255) NOT NULL,
    viability_score FLOAT NOT NULL,
    details JSONB NOT NULL
);

-- Create Inventory Recommendations Table
CREATE TABLE inventory_recommendations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id VARCHAR(255) NOT NULL,
    recommended_safety_stock INT NOT NULL,
    rationale TEXT NOT NULL,
    generated_at TIMESTAMPTZ NOT NULL
);
"""