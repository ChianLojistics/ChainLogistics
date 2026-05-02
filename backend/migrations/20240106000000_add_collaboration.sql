-- Multi-party collaboration feature tables

-- Product shared visibility controls
CREATE TABLE IF NOT EXISTS product_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    shared_with_user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    permission_level VARCHAR(50) NOT NULL DEFAULT 'view', -- 'view', 'edit', 'manage'
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Collaboration workflows (e.g. approval requests)
CREATE TABLE IF NOT EXISTS collaboration_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    requester_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- 'pending', 'approved', 'rejected'
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Audit trails for collaboration actions
CREATE TABLE IF NOT EXISTS collaboration_audit_trails (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(100) NOT NULL,
    entity_type VARCHAR(50) NOT NULL,
    entity_id VARCHAR(255) NOT NULL,
    details JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_product_shares_product ON product_shares(product_id);
CREATE INDEX idx_product_shares_user ON product_shares(shared_with_user_id);
CREATE INDEX idx_collaboration_requests_product ON collaboration_requests(product_id);
CREATE INDEX idx_collaboration_requests_requester ON collaboration_requests(requester_id);
CREATE INDEX idx_collaboration_audit_actor ON collaboration_audit_trails(actor_id);
