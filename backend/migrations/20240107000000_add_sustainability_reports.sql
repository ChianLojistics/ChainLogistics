-- Add sustainability reports table
CREATE TABLE IF NOT EXISTS sustainability_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    report_type VARCHAR(50) NOT NULL, -- 'EU_GREEN_DEAL', 'SEC_CLIMATE', 'ESG'
    content JSONB NOT NULL,
    generated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(id)
);

CREATE INDEX idx_sustainability_reports_product_id ON sustainability_reports(product_id);
