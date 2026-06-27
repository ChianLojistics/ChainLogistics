-- Decentralized content integrity anchors (IPFS / Arweave CAS registry)
-- Files are stored directly on decentralized networks; this table tracks
-- anchors for periodic tamper verification (no central file silo).

CREATE TABLE IF NOT EXISTS content_anchors (
    content_hash TEXT PRIMARY KEY,
    cid TEXT NOT NULL,
    storage_backend TEXT NOT NULL CHECK (storage_backend IN ('ipfs', 'arweave')),
    product_id TEXT,
    byte_size BIGINT NOT NULL CHECK (byte_size > 0 AND byte_size <= 52428800),
    mime_type TEXT,
    anchored_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    anchored_by TEXT,
    last_verified_at TIMESTAMPTZ,
    verification_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (verification_status IN ('pending', 'verified', 'tampered', 'unavailable')),
    tamper_alert_sent BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_content_anchors_product_id ON content_anchors(product_id);
CREATE INDEX IF NOT EXISTS idx_content_anchors_verification
    ON content_anchors(verification_status, last_verified_at NULLS FIRST);

CREATE TABLE IF NOT EXISTS content_tamper_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content_hash TEXT NOT NULL REFERENCES content_anchors(content_hash) ON DELETE CASCADE,
    expected_hash TEXT NOT NULL,
    actual_hash TEXT,
    cid TEXT NOT NULL,
    storage_backend TEXT NOT NULL,
    product_id TEXT,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    alert_sent BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX IF NOT EXISTS idx_content_tamper_alerts_detected_at
    ON content_tamper_alerts(detected_at DESC);
