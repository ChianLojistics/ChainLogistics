-- Recall management and notification system

CREATE TABLE IF NOT EXISTS recalls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    batch_id TEXT,
    title TEXT NOT NULL,
    reason TEXT NOT NULL,
    severity VARCHAR(20) NOT NULL DEFAULT 'medium',
    status VARCHAR(30) NOT NULL DEFAULT 'open',
    trigger_type VARCHAR(30) NOT NULL DEFAULT 'manual',
    triggered_by TEXT,
    triggered_event_id BIGINT REFERENCES tracking_events(id) ON DELETE SET NULL,
    started_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    closed_at TIMESTAMP WITH TIME ZONE,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS recall_affected_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recall_id UUID NOT NULL REFERENCES recalls(id) ON DELETE CASCADE,
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    batch_id TEXT,
    stakeholder_role VARCHAR(30),
    stakeholder_address TEXT,
    detected_via VARCHAR(30) NOT NULL DEFAULT 'metadata',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS recall_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recall_id UUID NOT NULL REFERENCES recalls(id) ON DELETE CASCADE,
    recipient TEXT NOT NULL,
    channel VARCHAR(30) NOT NULL DEFAULT 'in_app',
    status VARCHAR(30) NOT NULL DEFAULT 'queued',
    sent_at TIMESTAMP WITH TIME ZONE,
    acknowledged_at TIMESTAMP WITH TIME ZONE,
    payload JSONB DEFAULT '{}',
    error TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS recall_effectiveness (
    recall_id UUID PRIMARY KEY REFERENCES recalls(id) ON DELETE CASCADE,
    affected_count INTEGER NOT NULL DEFAULT 0,
    notified_count INTEGER NOT NULL DEFAULT 0,
    acknowledged_count INTEGER NOT NULL DEFAULT 0,
    recovered_count INTEGER NOT NULL DEFAULT 0,
    disposed_count INTEGER NOT NULL DEFAULT 0,
    last_updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_recalls_product_id ON recalls(product_id);
CREATE INDEX IF NOT EXISTS idx_recalls_batch_id ON recalls(batch_id);
CREATE INDEX IF NOT EXISTS idx_recalls_status ON recalls(status);
CREATE INDEX IF NOT EXISTS idx_recall_affected_items_recall_id ON recall_affected_items(recall_id);
CREATE INDEX IF NOT EXISTS idx_recall_affected_items_product_batch ON recall_affected_items(product_id, batch_id);
CREATE INDEX IF NOT EXISTS idx_recall_notifications_recall_id ON recall_notifications(recall_id);
CREATE INDEX IF NOT EXISTS idx_recall_notifications_recipient ON recall_notifications(recipient);

-- Keep updated_at in sync
CREATE TRIGGER update_recalls_updated_at BEFORE UPDATE ON recalls
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
