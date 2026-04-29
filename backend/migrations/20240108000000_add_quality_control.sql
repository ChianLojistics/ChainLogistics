-- Add quality control checkpoints and workflows tables

-- QC checkpoints table
CREATE TABLE qc_checkpoints (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    checkpoint_id TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT,
    checkpoint_type TEXT NOT NULL, -- inspection, testing, verification, validation
    category TEXT NOT NULL, -- incoming, in_process, final, packaging
    product_category TEXT, -- Optional: specific to product category
    required_fields JSONB DEFAULT '{}',
    acceptance_criteria JSONB DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- QC workflow definitions table
CREATE TABLE qc_workflows (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    workflow_id TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT,
    product_category TEXT,
    checkpoint_ids TEXT[] NOT NULL, -- Ordered list of checkpoint IDs
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- QC inspections table (individual checkpoint executions)
CREATE TABLE qc_inspections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    inspection_id TEXT NOT NULL UNIQUE,
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    checkpoint_id TEXT NOT NULL REFERENCES qc_checkpoints(checkpoint_id) ON DELETE RESTRICT,
    workflow_id TEXT REFERENCES qc_workflows(workflow_id) ON DELETE SET NULL,
    status TEXT NOT NULL DEFAULT 'pending', -- pending, in_progress, passed, failed, skipped
    inspector_id TEXT,
    inspection_date TIMESTAMP WITH TIME ZONE,
    location TEXT,
    results JSONB DEFAULT '{}',
    quality_metrics JSONB DEFAULT '{}',
    notes TEXT,
    evidence_documents TEXT[] DEFAULT '{}',
    is_passed BOOLEAN,
    failure_reason TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Non-conformance records table
CREATE TABLE non_conformances (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    nc_id TEXT NOT NULL UNIQUE,
    inspection_id UUID REFERENCES qc_inspections(id) ON DELETE SET NULL,
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    severity TEXT NOT NULL, -- minor, major, critical
    category TEXT NOT NULL, -- dimensional, visual, functional, documentation, safety
    description TEXT NOT NULL,
    root_cause TEXT,
    correction_action TEXT,
    correction_status TEXT NOT NULL DEFAULT 'open', -- open, in_progress, resolved, verified
    responsible_party TEXT,
    due_date TIMESTAMP WITH TIME ZONE,
    resolved_at TIMESTAMP WITH TIME ZONE,
    verified_by TEXT,
    verified_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Quality metrics tracking table
CREATE TABLE quality_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    metric_id TEXT NOT NULL UNIQUE,
    product_id TEXT REFERENCES products(id) ON DELETE CASCADE,
    metric_type TEXT NOT NULL, -- defect_rate, pass_rate, rework_rate, customer_returns
    metric_value DECIMAL(10, 2) NOT NULL,
    unit TEXT,
    measurement_period_start TIMESTAMP WITH TIME ZONE NOT NULL,
    measurement_period_end TIMESTAMP WITH TIME ZONE NOT NULL,
    target_value DECIMAL(10, 2),
    threshold_min DECIMAL(10, 2),
    threshold_max DECIMAL(10, 2),
    is_within_threshold BOOLEAN,
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_qc_checkpoints_checkpoint_id ON qc_checkpoints(checkpoint_id);
CREATE INDEX idx_qc_checkpoints_type ON qc_checkpoints(checkpoint_type);
CREATE INDEX idx_qc_checkpoints_category ON qc_checkpoints(category);
CREATE INDEX idx_qc_checkpoints_is_active ON qc_checkpoints(is_active);

CREATE INDEX idx_qc_workflows_workflow_id ON qc_workflows(workflow_id);
CREATE INDEX idx_qc_workflows_product_category ON qc_workflows(product_category);
CREATE INDEX idx_qc_workflows_is_active ON qc_workflows(is_active);

CREATE INDEX idx_qc_inspections_inspection_id ON qc_inspections(inspection_id);
CREATE INDEX idx_qc_inspections_product_id ON qc_inspections(product_id);
CREATE INDEX idx_qc_inspections_checkpoint_id ON qc_inspections(checkpoint_id);
CREATE INDEX idx_qc_inspections_workflow_id ON qc_inspections(workflow_id);
CREATE INDEX idx_qc_inspections_status ON qc_inspections(status);
CREATE INDEX idx_qc_inspections_inspection_date ON qc_inspections(inspection_date);

CREATE INDEX idx_non_conformances_nc_id ON non_conformances(nc_id);
CREATE INDEX idx_non_conformances_inspection_id ON non_conformances(inspection_id);
CREATE INDEX idx_non_conformances_product_id ON non_conformances(product_id);
CREATE INDEX idx_non_conformances_severity ON non_conformances(severity);
CREATE INDEX idx_non_conformances_status ON non_conformances(correction_status);

CREATE INDEX idx_quality_metrics_metric_id ON quality_metrics(metric_id);
CREATE INDEX idx_quality_metrics_product_id ON quality_metrics(product_id);
CREATE INDEX idx_quality_metrics_type ON quality_metrics(metric_type);
CREATE INDEX idx_quality_metrics_period ON quality_metrics(measurement_period_start, measurement_period_end);

-- Create trigger to update updated_at timestamp
CREATE TRIGGER update_qc_checkpoints_updated_at BEFORE UPDATE ON qc_checkpoints
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_qc_workflows_updated_at BEFORE UPDATE ON qc_workflows
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_qc_inspections_updated_at BEFORE UPDATE ON qc_inspections
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_non_conformances_updated_at BEFORE UPDATE ON non_conformances
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
