-- Carbon credit and footprint tracking tables

-- Carbon footprint records per product/event
CREATE TABLE IF NOT EXISTS carbon_footprints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    tracking_event_id BIGINT REFERENCES tracking_events(id) ON DELETE SET NULL,
    calculation_method VARCHAR(100) NOT NULL DEFAULT 'ghg_protocol',
    -- Emissions in kg CO2 equivalent
    transport_emissions DECIMAL(12, 4) NOT NULL DEFAULT 0,
    manufacturing_emissions DECIMAL(12, 4) NOT NULL DEFAULT 0,
    packaging_emissions DECIMAL(12, 4) NOT NULL DEFAULT 0,
    storage_emissions DECIMAL(12, 4) NOT NULL DEFAULT 0,
    total_emissions DECIMAL(12, 4) NOT NULL DEFAULT 0,
    -- Reduction data
    baseline_emissions DECIMAL(12, 4),
    emissions_reduction DECIMAL(12, 4),
    reduction_percentage DECIMAL(5, 2),
    -- Metadata
    distance_km DECIMAL(10, 2),
    transport_mode VARCHAR(50),  -- 'road', 'rail', 'sea', 'air'
    energy_source VARCHAR(50),   -- 'renewable', 'grid', 'diesel', 'natural_gas'
    raw_data JSONB DEFAULT '{}',
    calculated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Carbon credits (1 credit = 1 tonne CO2 equivalent reduced)
CREATE TABLE IF NOT EXISTS carbon_credits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    product_id TEXT REFERENCES products(id) ON DELETE SET NULL,
    serial_number VARCHAR(100) NOT NULL UNIQUE,
    vintage_year INTEGER NOT NULL,
    credit_type VARCHAR(50) NOT NULL DEFAULT 'verified_reduction',
    -- 'verified_reduction', 'removal', 'avoidance'
    standard VARCHAR(100) NOT NULL DEFAULT 'GHG_PROTOCOL',
    -- 'GHG_PROTOCOL', 'VERRA_VCS', 'GOLD_STANDARD', 'CDM'
    quantity DECIMAL(12, 4) NOT NULL,  -- tonnes CO2e
    price_per_tonne DECIMAL(12, 2),    -- USD
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    -- 'pending', 'verified', 'listed', 'sold', 'retired', 'cancelled'
    registry_id VARCHAR(255),          -- external registry reference
    registry_url TEXT,
    verification_body VARCHAR(255),
    verified_at TIMESTAMP WITH TIME ZONE,
    retired_at TIMESTAMP WITH TIME ZONE,
    retirement_reason TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Carbon credit trades / marketplace orders
CREATE TABLE IF NOT EXISTS carbon_trades (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    credit_id UUID NOT NULL REFERENCES carbon_credits(id) ON DELETE CASCADE,
    seller_id UUID NOT NULL REFERENCES users(id),
    buyer_id UUID REFERENCES users(id),
    quantity DECIMAL(12, 4) NOT NULL,
    price_per_tonne DECIMAL(12, 2) NOT NULL,
    total_amount DECIMAL(14, 2) NOT NULL,
    currency VARCHAR(10) NOT NULL DEFAULT 'USD',
    status VARCHAR(50) NOT NULL DEFAULT 'open',
    -- 'open', 'matched', 'settled', 'cancelled', 'expired'
    trade_type VARCHAR(50) NOT NULL DEFAULT 'spot',
    -- 'spot', 'forward', 'offset_purchase'
    settlement_date TIMESTAMP WITH TIME ZONE,
    blockchain_tx_hash VARCHAR(255),
    platform_fee DECIMAL(10, 2) DEFAULT 0,
    notes TEXT,
    expires_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Third-party verification requests
CREATE TABLE IF NOT EXISTS carbon_verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    credit_id UUID NOT NULL REFERENCES carbon_credits(id) ON DELETE CASCADE,
    requested_by UUID NOT NULL REFERENCES users(id),
    verifier_name VARCHAR(255) NOT NULL,
    verifier_accreditation VARCHAR(100),
    status VARCHAR(50) NOT NULL DEFAULT 'requested',
    -- 'requested', 'in_review', 'approved', 'rejected', 'expired'
    methodology VARCHAR(255),
    scope TEXT,
    findings JSONB DEFAULT '{}',
    certificate_url TEXT,
    submitted_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Carbon reporting snapshots
CREATE TABLE IF NOT EXISTS carbon_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    report_type VARCHAR(50) NOT NULL DEFAULT 'annual',
    -- 'annual', 'quarterly', 'monthly', 'custom'
    period_start TIMESTAMP WITH TIME ZONE NOT NULL,
    period_end TIMESTAMP WITH TIME ZONE NOT NULL,
    total_emissions DECIMAL(14, 4) NOT NULL DEFAULT 0,
    total_reductions DECIMAL(14, 4) NOT NULL DEFAULT 0,
    net_emissions DECIMAL(14, 4) NOT NULL DEFAULT 0,
    credits_generated DECIMAL(12, 4) NOT NULL DEFAULT 0,
    credits_retired DECIMAL(12, 4) NOT NULL DEFAULT 0,
    credits_sold DECIMAL(12, 4) NOT NULL DEFAULT 0,
    revenue_from_credits DECIMAL(14, 2) NOT NULL DEFAULT 0,
    summary JSONB DEFAULT '{}',
    generated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_carbon_footprints_product_id ON carbon_footprints(product_id);
CREATE INDEX idx_carbon_footprints_calculated_at ON carbon_footprints(calculated_at);
CREATE INDEX idx_carbon_credits_owner_id ON carbon_credits(owner_id);
CREATE INDEX idx_carbon_credits_status ON carbon_credits(status);
CREATE INDEX idx_carbon_credits_vintage_year ON carbon_credits(vintage_year);
CREATE INDEX idx_carbon_credits_serial ON carbon_credits(serial_number);
CREATE INDEX idx_carbon_trades_credit_id ON carbon_trades(credit_id);
CREATE INDEX idx_carbon_trades_seller_id ON carbon_trades(seller_id);
CREATE INDEX idx_carbon_trades_buyer_id ON carbon_trades(buyer_id);
CREATE INDEX idx_carbon_trades_status ON carbon_trades(status);
CREATE INDEX idx_carbon_verifications_credit_id ON carbon_verifications(credit_id);
CREATE INDEX idx_carbon_reports_owner_id ON carbon_reports(owner_id);
CREATE INDEX idx_carbon_reports_period ON carbon_reports(period_start, period_end);
