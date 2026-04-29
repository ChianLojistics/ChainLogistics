-- Add batch and lot tracking functionality
-- This migration adds comprehensive batch management with genealogy, quality attributes, and recall capabilities

-- Batch status enum
CREATE TYPE batch_status AS ENUM ('pending', 'in_production', 'completed', 'quality_check', 'quarantined', 'shipped', 'recalled', 'disposed');

-- Batches table
CREATE TABLE batches (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    batch_number TEXT NOT NULL UNIQUE,
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    lot_number TEXT,
    production_date TIMESTAMP WITH TIME ZONE NOT NULL,
    expiry_date TIMESTAMP WITH TIME ZONE,
    quantity_produced INTEGER NOT NULL DEFAULT 0,
    quantity_available INTEGER NOT NULL DEFAULT 0,
    status batch_status NOT NULL DEFAULT 'pending',
    production_location TEXT NOT NULL,
    quality_grade TEXT,
    quality_score NUMERIC(5,2),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by TEXT NOT NULL,
    updated_by TEXT NOT NULL
);

-- Batch genealogy table for tracking parent-child relationships
CREATE TABLE batch_genealogy (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    parent_batch_id UUID NOT NULL REFERENCES batches(id) ON DELETE CASCADE,
    child_batch_id UUID NOT NULL REFERENCES batches(id) ON DELETE CASCADE,
    relationship_type TEXT NOT NULL, -- 'split', 'merge', 'transform', 'rework'
    quantity_transferred INTEGER NOT NULL DEFAULT 0,
    transfer_date TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    UNIQUE(parent_batch_id, child_batch_id)
);

-- Batch quality attributes table
CREATE TABLE batch_quality_attributes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    batch_id UUID NOT NULL REFERENCES batches(id) ON DELETE CASCADE,
    attribute_name TEXT NOT NULL,
    attribute_value TEXT NOT NULL,
    measurement_unit TEXT,
    tolerance_min NUMERIC(10,2),
    tolerance_max NUMERIC(10,2),
    measured_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    measured_by TEXT NOT NULL,
    is_within_tolerance BOOLEAN NOT NULL DEFAULT true,
    notes TEXT,
    UNIQUE(batch_id, attribute_name, measured_at)
);

-- Batch recall table
CREATE TABLE batch_recalls (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    batch_id UUID NOT NULL REFERENCES batches(id) ON DELETE CASCADE,
    recall_type TEXT NOT NULL, -- 'voluntary', 'mandatory', 'precautionary'
    recall_reason TEXT NOT NULL,
    recall_date TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    initiated_by TEXT NOT NULL,
    severity TEXT NOT NULL, -- 'low', 'medium', 'high', 'critical'
    status TEXT NOT NULL DEFAULT 'active', -- 'active', 'completed', 'cancelled'
    affected_quantity INTEGER NOT NULL DEFAULT 0,
    recovered_quantity INTEGER NOT NULL DEFAULT 0,
    notification_sent BOOLEAN NOT NULL DEFAULT false,
    public_announcement BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Batch inventory tracking
CREATE TABLE batch_inventory (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    batch_id UUID NOT NULL REFERENCES batches(id) ON DELETE CASCADE,
    location_id TEXT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 0,
    transaction_type TEXT NOT NULL, -- 'in', 'out', 'adjustment', 'transfer'
    reference_id TEXT,
    transaction_date TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    performed_by TEXT NOT NULL,
    notes TEXT
);

-- Create indexes for performance
CREATE INDEX idx_batches_product_id ON batches(product_id);
CREATE INDEX idx_batches_batch_number ON batches(batch_number);
CREATE INDEX idx_batches_lot_number ON batches(lot_number);
CREATE INDEX idx_batches_status ON batches(status);
CREATE INDEX idx_batches_production_date ON batches(production_date);
CREATE INDEX idx_batches_expiry_date ON batches(expiry_date);
CREATE INDEX idx_batches_quality_grade ON batches(quality_grade);
CREATE INDEX idx_batches_metadata ON batches USING GIN(metadata);

CREATE INDEX idx_batch_genealogy_parent ON batch_genealogy(parent_batch_id);
CREATE INDEX idx_batch_genealogy_child ON batch_genealogy(child_batch_id);
CREATE INDEX idx_batch_genealogy_type ON batch_genealogy(relationship_type);

CREATE INDEX idx_batch_quality_attributes_batch_id ON batch_quality_attributes(batch_id);
CREATE INDEX idx_batch_quality_attributes_name ON batch_quality_attributes(attribute_name);
CREATE INDEX idx_batch_quality_attributes_measured_at ON batch_quality_attributes(measured_at);

CREATE INDEX idx_batch_recalls_batch_id ON batch_recalls(batch_id);
CREATE INDEX idx_batch_recalls_status ON batch_recalls(status);
CREATE INDEX idx_batch_recalls_severity ON batch_recalls(severity);
CREATE INDEX idx_batch_recalls_date ON batch_recalls(recall_date);

CREATE INDEX idx_batch_inventory_batch_id ON batch_inventory(batch_id);
CREATE INDEX idx_batch_inventory_location ON batch_inventory(location_id);
CREATE INDEX idx_batch_inventory_date ON batch_inventory(transaction_date);

-- Create trigger to update updated_at timestamp for batches
CREATE TRIGGER update_batches_updated_at BEFORE UPDATE ON batches
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_batch_recalls_updated_at BEFORE UPDATE ON batch_recalls
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
