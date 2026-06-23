-- Decentralized content anchors (manuals/PDFs) with CAS deduplication
-- Supports up to 50 MB per file (52,428,800 bytes)

CREATE TABLE IF NOT EXISTS content_anchors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    on_chain_anchor_id BIGINT,
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    content_hash TEXT NOT NULL,
    cid TEXT NOT NULL,
    storage_scheme TEXT NOT NULL CHECK (storage_scheme IN ('ipfs', 'arweave')),
    byte_size BIGINT NOT NULL CHECK (byte_size > 0 AND byte_size <= 52428800),
    storage_uri TEXT NOT NULL,
    verification_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (verification_status IN ('pending', 'verified', 'tampered', 'unavailable')),
    last_verified_at TIMESTAMPTZ,
    failure_reason TEXT,
    failure_count INT NOT NULL DEFAULT 0,
    deduplicated BOOLEAN NOT NULL DEFAULT FALSE,
    anchored_by TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (content_hash)
);

CREATE INDEX IF NOT EXISTS idx_content_anchors_product_id ON content_anchors(product_id);
CREATE INDEX IF NOT EXISTS idx_content_anchors_verification_status ON content_anchors(verification_status);
CREATE INDEX IF NOT EXISTS idx_content_anchors_last_verified ON content_anchors(last_verified_at);

CREATE TABLE IF NOT EXISTS content_tamper_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    anchor_id UUID NOT NULL REFERENCES content_anchors(id) ON DELETE CASCADE,
    product_id TEXT NOT NULL,
    expected_hash TEXT NOT NULL,
    actual_hash TEXT,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved BOOLEAN NOT NULL DEFAULT FALSE,
    alert_payload JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_content_tamper_alerts_unresolved
    ON content_tamper_alerts(resolved, detected_at DESC);
