-- Add supplier verification and rating system tables

-- Suppliers table
CREATE TABLE suppliers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    supplier_id TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    legal_name TEXT,
    tax_id TEXT,
    registration_number TEXT,
    business_type TEXT, -- manufacturer, distributor, logistics, service_provider
    tier TEXT NOT NULL DEFAULT 'standard', -- basic, standard, premium, strategic
    contact_email TEXT NOT NULL,
    contact_phone TEXT,
    address TEXT,
    city TEXT,
    country TEXT NOT NULL,
    postal_code TEXT,
    website TEXT,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    verification_status TEXT NOT NULL DEFAULT 'pending', -- pending, under_review, verified, rejected, suspended
    verification_date TIMESTAMP WITH TIME ZONE,
    verified_by TEXT,
    certification_expiry TIMESTAMP WITH TIME ZONE,
    risk_level TEXT NOT NULL DEFAULT 'medium', -- low, medium, high, critical
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Supplier ratings table
CREATE TABLE supplier_ratings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    supplier_id TEXT NOT NULL REFERENCES suppliers(supplier_id) ON DELETE CASCADE,
    rater_id TEXT NOT NULL, -- User or system that provided the rating
    rating_type TEXT NOT NULL, -- quality, delivery, communication, compliance, overall
    score DECIMAL(3, 2) NOT NULL CHECK (score >= 0 AND score <= 5),
    comment TEXT,
    rating_period_start TIMESTAMP WITH TIME ZONE,
    rating_period_end TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(supplier_id, rater_id, rating_type, rating_period_start)
);

-- Supplier performance metrics table
CREATE TABLE supplier_performance (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    supplier_id TEXT NOT NULL REFERENCES suppliers(supplier_id) ON DELETE CASCADE,
    metric_type TEXT NOT NULL, -- on_time_delivery, quality_score, response_time, complaint_rate
    metric_value DECIMAL(10, 2) NOT NULL,
    unit TEXT,
    measurement_period_start TIMESTAMP WITH TIME ZONE NOT NULL,
    measurement_period_end TIMESTAMP WITH TIME ZONE NOT NULL,
    target_value DECIMAL(10, 2),
    benchmark_value DECIMAL(10, 2),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(supplier_id, metric_type, measurement_period_start, measurement_period_end)
);

-- Supplier compliance records table
CREATE TABLE supplier_compliance (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    supplier_id TEXT NOT NULL REFERENCES suppliers(supplier_id) ON DELETE CASCADE,
    compliance_type TEXT NOT NULL, -- iso9001, iso14001, fda, gmp, haccp, custom
    certificate_number TEXT,
    issuing_authority TEXT,
    issue_date TIMESTAMP WITH TIME ZONE,
    expiry_date TIMESTAMP WITH TIME ZONE,
    status TEXT NOT NULL DEFAULT 'active', -- active, expired, suspended, revoked
    document_url TEXT,
    verification_notes TEXT,
    verified_by TEXT,
    verified_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Supplier products table (products supplied by each supplier)
CREATE TABLE supplier_products (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    supplier_id TEXT NOT NULL REFERENCES suppliers(supplier_id) ON DELETE CASCADE,
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    is_primary_supplier BOOLEAN NOT NULL DEFAULT false,
    supply_capacity INTEGER,
    lead_time_days INTEGER,
    unit_price DECIMAL(10, 2),
    currency TEXT DEFAULT 'USD',
    min_order_quantity INTEGER,
    contract_start_date TIMESTAMP WITH TIME ZONE,
    contract_end_date TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(supplier_id, product_id)
);

-- Supplier audit trail table
CREATE TABLE supplier_audit_trail (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    supplier_id TEXT NOT NULL REFERENCES suppliers(supplier_id) ON DELETE CASCADE,
    action_type TEXT NOT NULL, -- created, updated, verified, suspended, tier_changed, rating_updated
    previous_value JSONB,
    new_value JSONB,
    performed_by TEXT NOT NULL,
    performed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    reason TEXT,
    ip_address TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_suppliers_supplier_id ON suppliers(supplier_id);
CREATE INDEX idx_suppliers_business_type ON suppliers(business_type);
CREATE INDEX idx_suppliers_tier ON suppliers(tier);
CREATE INDEX idx_suppliers_verification_status ON suppliers(verification_status);
CREATE INDEX idx_suppliers_is_verified ON suppliers(is_verified);
CREATE INDEX idx_suppliers_country ON suppliers(country);
CREATE INDEX idx_suppliers_risk_level ON suppliers(risk_level);

CREATE INDEX idx_supplier_ratings_supplier_id ON supplier_ratings(supplier_id);
CREATE INDEX idx_supplier_ratings_rating_type ON supplier_ratings(rating_type);
CREATE INDEX idx_supplier_ratings_created_at ON supplier_ratings(created_at);

CREATE INDEX idx_supplier_performance_supplier_id ON supplier_performance(supplier_id);
CREATE INDEX idx_supplier_performance_metric_type ON supplier_performance(metric_type);
CREATE INDEX idx_supplier_performance_period ON supplier_performance(measurement_period_start, measurement_period_end);

CREATE INDEX idx_supplier_compliance_supplier_id ON supplier_compliance(supplier_id);
CREATE INDEX idx_supplier_compliance_compliance_type ON supplier_compliance(compliance_type);
CREATE INDEX idx_supplier_compliance_status ON supplier_compliance(status);
CREATE INDEX idx_supplier_compliance_expiry_date ON supplier_compliance(expiry_date);

CREATE INDEX idx_supplier_products_supplier_id ON supplier_products(supplier_id);
CREATE INDEX idx_supplier_products_product_id ON supplier_products(product_id);
CREATE INDEX idx_supplier_products_is_primary ON supplier_products(is_primary_supplier);

CREATE INDEX idx_supplier_audit_trail_supplier_id ON supplier_audit_trail(supplier_id);
CREATE INDEX idx_supplier_audit_trail_action_type ON supplier_audit_trail(action_type);
CREATE INDEX idx_supplier_audit_trail_performed_at ON supplier_audit_trail(performed_at);

-- Create trigger to update updated_at timestamp
CREATE TRIGGER update_suppliers_updated_at BEFORE UPDATE ON suppliers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_supplier_compliance_updated_at BEFORE UPDATE ON supplier_compliance
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_supplier_products_updated_at BEFORE UPDATE ON supplier_products
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
