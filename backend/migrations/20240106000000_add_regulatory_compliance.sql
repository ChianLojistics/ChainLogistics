-- Add regulatory compliance tracking tables

-- Regulatory requirements table
CREATE TABLE regulatory_requirements (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    requirement_id TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT,
    regulation_type TEXT NOT NULL, -- FDA, ISO, GDPR, etc.
    category TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'medium', -- low, medium, high, critical
    required_fields JSONB DEFAULT '{}',
    validation_logic TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    effective_date TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expiry_date TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Product compliance tracking table
CREATE TABLE product_compliance (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    requirement_id TEXT NOT NULL REFERENCES regulatory_requirements(requirement_id),
    status TEXT NOT NULL DEFAULT 'pending', -- pending, compliant, non_compliant, exempt, under_review
    last_checked_at TIMESTAMP WITH TIME ZONE,
    last_check_result JSONB,
    violations JSONB DEFAULT '[]',
    warnings JSONB DEFAULT '[]',
    evidence_documents TEXT[] DEFAULT '{}',
    notes TEXT,
    checked_by TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(product_id, requirement_id)
);

-- Compliance audit trail table
CREATE TABLE compliance_audit_trail (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    requirement_id TEXT REFERENCES regulatory_requirements(requirement_id),
    action_type TEXT NOT NULL, -- check, update, exemption, manual_override
    previous_status TEXT,
    new_status TEXT,
    action_details JSONB,
    performed_by TEXT NOT NULL,
    performed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    ip_address TEXT,
    user_agent TEXT
);

-- Compliance reports table
CREATE TABLE compliance_reports (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    report_id TEXT NOT NULL UNIQUE,
    report_type TEXT NOT NULL, -- product_summary, regulatory_summary, non_compliance, custom
    scope JSONB NOT NULL, -- filters for what's included in report
    generated_by TEXT NOT NULL,
    generated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    period_start TIMESTAMP WITH TIME ZONE,
    period_end TIMESTAMP WITH TIME ZONE,
    total_products_checked INTEGER DEFAULT 0,
    compliant_count INTEGER DEFAULT 0,
    non_compliant_count INTEGER DEFAULT 0,
    pending_count INTEGER DEFAULT 0,
    compliance_rate DECIMAL(5,2),
    report_data JSONB NOT NULL,
    file_path TEXT,
    status TEXT NOT NULL DEFAULT 'completed', -- pending, completed, failed
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_regulatory_requirements_type ON regulatory_requirements(regulation_type);
CREATE INDEX idx_regulatory_requirements_category ON regulatory_requirements(category);
CREATE INDEX idx_regulatory_requirements_is_active ON regulatory_requirements(is_active);

CREATE INDEX idx_product_compliance_product_id ON product_compliance(product_id);
CREATE INDEX idx_product_compliance_requirement_id ON product_compliance(requirement_id);
CREATE INDEX idx_product_compliance_status ON product_compliance(status);
CREATE INDEX idx_product_compliance_last_checked ON product_compliance(last_checked_at);

CREATE INDEX idx_compliance_audit_trail_product_id ON compliance_audit_trail(product_id);
CREATE INDEX idx_compliance_audit_trail_requirement_id ON compliance_audit_trail(requirement_id);
CREATE INDEX idx_compliance_audit_trail_action_type ON compliance_audit_trail(action_type);
CREATE INDEX idx_compliance_audit_trail_performed_at ON compliance_audit_trail(performed_at);

CREATE INDEX idx_compliance_reports_report_type ON compliance_reports(report_type);
CREATE INDEX idx_compliance_reports_generated_at ON compliance_reports(generated_at);
CREATE INDEX idx_compliance_reports_status ON compliance_reports(status);

-- Create trigger to update updated_at timestamp
CREATE TRIGGER update_regulatory_requirements_updated_at BEFORE UPDATE ON regulatory_requirements
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_product_compliance_updated_at BEFORE UPDATE ON product_compliance
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
