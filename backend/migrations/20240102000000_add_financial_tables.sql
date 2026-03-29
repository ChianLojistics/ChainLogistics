-- Payment and financial transaction tables

CREATE TABLE IF NOT EXISTS payment_methods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    payment_type VARCHAR(50) NOT NULL, -- 'bank_transfer', 'card', 'crypto', 'wire'
    provider VARCHAR(100) NOT NULL, -- 'stripe', 'wise', 'circle', etc.
    provider_id VARCHAR(255) NOT NULL,
    is_default BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, provider, provider_id)
);

CREATE TABLE IF NOT EXISTS transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    transaction_type VARCHAR(50) NOT NULL, -- 'payment', 'refund', 'fee', 'financing'
    amount DECIMAL(20, 8) NOT NULL,
    currency VARCHAR(10) NOT NULL DEFAULT 'USD',
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- 'pending', 'completed', 'failed', 'cancelled'
    payment_method_id UUID REFERENCES payment_methods(id),
    blockchain_network VARCHAR(50),
    blockchain_tx_hash VARCHAR(255),
    description TEXT,
    metadata JSONB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP
);

CREATE TABLE IF NOT EXISTS invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    invoice_number VARCHAR(100) NOT NULL UNIQUE,
    amount DECIMAL(20, 8) NOT NULL,
    currency VARCHAR(10) NOT NULL DEFAULT 'USD',
    status VARCHAR(50) NOT NULL DEFAULT 'draft', -- 'draft', 'sent', 'paid', 'overdue', 'cancelled'
    due_date DATE NOT NULL,
    description TEXT,
    line_items JSONB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    paid_at TIMESTAMP
);

CREATE TABLE IF NOT EXISTS financing_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    invoice_id UUID REFERENCES invoices(id),
    financing_type VARCHAR(50) NOT NULL, -- 'invoice_factoring', 'supply_chain_financing', 'working_capital'
    amount_requested DECIMAL(20, 8) NOT NULL,
    amount_approved DECIMAL(20, 8),
    currency VARCHAR(10) NOT NULL DEFAULT 'USD',
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- 'pending', 'approved', 'rejected', 'funded'
    interest_rate DECIMAL(5, 2),
    term_days INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    approved_at TIMESTAMP,
    funded_at TIMESTAMP
);

CREATE TABLE IF NOT EXISTS wallet_balances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    blockchain_network VARCHAR(50) NOT NULL,
    wallet_address VARCHAR(255) NOT NULL,
    balance DECIMAL(20, 8) NOT NULL DEFAULT 0,
    token_symbol VARCHAR(20) NOT NULL,
    last_synced TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, blockchain_network, wallet_address)
);

CREATE TABLE IF NOT EXISTS compliance_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    product_id UUID REFERENCES products(id) ON DELETE CASCADE,
    compliance_type VARCHAR(100) NOT NULL, -- 'gdpr', 'fda_21_cfr_11', 'fsma', etc.
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- 'pending', 'compliant', 'non_compliant', 'exempted'
    validation_data JSONB,
    audit_trail JSONB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    verified_at TIMESTAMP
);

CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(255) NOT NULL,
    resource_type VARCHAR(100),
    resource_id UUID,
    changes JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_transactions_user_id ON transactions(user_id);
CREATE INDEX idx_transactions_status ON transactions(status);
CREATE INDEX idx_transactions_blockchain_tx_hash ON transactions(blockchain_tx_hash);
CREATE INDEX idx_invoices_user_id ON invoices(user_id);
CREATE INDEX idx_invoices_status ON invoices(status);
CREATE INDEX idx_financing_requests_user_id ON financing_requests(user_id);
CREATE INDEX idx_financing_requests_status ON financing_requests(status);
CREATE INDEX idx_wallet_balances_user_id ON wallet_balances(user_id);
CREATE INDEX idx_compliance_records_user_id ON compliance_records(user_id);
CREATE INDEX idx_compliance_records_type ON compliance_records(compliance_type);
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);
