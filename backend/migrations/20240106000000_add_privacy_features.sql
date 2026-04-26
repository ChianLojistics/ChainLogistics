-- Add privacy features to products table
ALTER TABLE products 
ADD COLUMN zk_proof TEXT,
ADD COLUMN encrypted_data TEXT,
ADD COLUMN selective_disclosure JSONB DEFAULT '{}';

-- Add privacy features to tracking_events table
ALTER TABLE tracking_events 
ADD COLUMN zk_proof TEXT,
ADD COLUMN encrypted_data TEXT,
ADD COLUMN selective_disclosure JSONB DEFAULT '{}';

-- Create indexes for searching proofs
CREATE INDEX idx_products_zk_proof ON products(zk_proof) WHERE zk_proof IS NOT NULL;
CREATE INDEX idx_tracking_events_zk_proof ON tracking_events(zk_proof) WHERE zk_proof IS NOT NULL;
