-- Predictive Routing & Proactive Anomaly Detection
-- Implements graph-based anomaly scoring for 100k+ shipments

-- Route graph nodes: canonical locations/hubs
CREATE TABLE IF NOT EXISTS route_nodes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    node_key TEXT NOT NULL UNIQUE, -- normalised location string
    display_name TEXT NOT NULL,
    node_type TEXT NOT NULL DEFAULT 'waypoint', -- origin | waypoint | destination | hub
    latitude DECIMAL(10, 7),
    longitude DECIMAL(10, 7),
    -- Graph features updated incrementally
    avg_dwell_time_hours DECIMAL(10, 4) DEFAULT 0,
    historical_delay_rate DECIMAL(5, 4) DEFAULT 0, -- 0-1
    total_shipments_through INT NOT NULL DEFAULT 0,
    anomaly_signal DECIMAL(5, 4) NOT NULL DEFAULT 0, -- aggregated from neighbors (GNN pass)
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Route graph edges: transit legs between nodes
CREATE TABLE IF NOT EXISTS route_edges (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_node_id UUID NOT NULL REFERENCES route_nodes(id) ON DELETE CASCADE,
    dest_node_id UUID NOT NULL REFERENCES route_nodes(id) ON DELETE CASCADE,
    -- Historical performance stats (updated on each shipment completion)
    avg_transit_hours DECIMAL(10, 4) DEFAULT 0,
    stddev_transit_hours DECIMAL(10, 4) DEFAULT 0,
    historical_delay_rate DECIMAL(5, 4) DEFAULT 0, -- 0-1
    total_shipments INT NOT NULL DEFAULT 0,
    -- Current edge health
    is_disrupted BOOLEAN NOT NULL DEFAULT false,
    disruption_reason TEXT,
    disruption_started_at TIMESTAMP WITH TIME ZONE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (source_node_id, dest_node_id)
);

-- Active shipment routes: which path each shipment is taking
CREATE TABLE IF NOT EXISTS shipment_routes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    batch_id TEXT,
    -- Ordered list of node_ids (jsonb array of UUIDs)
    planned_path JSONB NOT NULL DEFAULT '[]',
    actual_path JSONB NOT NULL DEFAULT '[]',  -- nodes visited so far
    current_node_id UUID REFERENCES route_nodes(id),
    origin_node_id UUID REFERENCES route_nodes(id),
    destination_node_id UUID REFERENCES route_nodes(id),
    status TEXT NOT NULL DEFAULT 'in_transit', -- in_transit | completed | cancelled | delayed
    planned_arrival_at TIMESTAMP WITH TIME ZONE,
    actual_arrival_at TIMESTAMP WITH TIME ZONE,
    departed_at TIMESTAMP WITH TIME ZONE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Real-time anomaly scores per shipment (updated on each event ingestion)
CREATE TABLE IF NOT EXISTS anomaly_scores (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    shipment_route_id UUID NOT NULL REFERENCES shipment_routes(id) ON DELETE CASCADE,
    product_id TEXT NOT NULL,
    -- Composite score 0.0 (normal) – 1.0 (critical)
    composite_score DECIMAL(5, 4) NOT NULL DEFAULT 0,
    -- Sub-scores for interpretability
    temporal_score DECIMAL(5, 4) NOT NULL DEFAULT 0,   -- delay vs. expected
    path_score DECIMAL(5, 4) NOT NULL DEFAULT 0,       -- route deviation
    env_score DECIMAL(5, 4) NOT NULL DEFAULT 0,        -- IoT sensor anomaly
    graph_score DECIMAL(5, 4) NOT NULL DEFAULT 0,      -- GNN neighbor signal
    risk_level TEXT NOT NULL DEFAULT 'low',             -- low | medium | high | critical
    -- Feature vector used (for audit / federated-learning export)
    feature_vector JSONB NOT NULL DEFAULT '{}',
    -- Model version that produced this score
    model_version TEXT NOT NULL DEFAULT 'gnn-v1',
    computed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    valid_until TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW() + INTERVAL '1 hour',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Proactive routing alerts triggered when anomaly score exceeds threshold
CREATE TABLE IF NOT EXISTS routing_alerts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    shipment_route_id UUID NOT NULL REFERENCES shipment_routes(id) ON DELETE CASCADE,
    anomaly_score_id UUID REFERENCES anomaly_scores(id),
    product_id TEXT NOT NULL,
    alert_type TEXT NOT NULL, -- delay_predicted | route_disruption | env_anomaly | critical_risk
    severity TEXT NOT NULL DEFAULT 'medium', -- low | medium | high | critical
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    -- Recommended actions
    recommendations JSONB NOT NULL DEFAULT '[]',
    -- Alert lifecycle
    status TEXT NOT NULL DEFAULT 'open', -- open | acknowledged | resolved | suppressed
    triggered_score DECIMAL(5, 4) NOT NULL DEFAULT 0,
    acknowledged_by TEXT,
    acknowledged_at TIMESTAMP WITH TIME ZONE,
    resolved_at TIMESTAMP WITH TIME ZONE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Training data export log (for federated / offline ML training)
CREATE TABLE IF NOT EXISTS routing_export_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    export_format TEXT NOT NULL DEFAULT 'jsonl', -- jsonl | csv | parquet
    record_count INT NOT NULL DEFAULT 0,
    date_range_start TIMESTAMP WITH TIME ZONE,
    date_range_end TIMESTAMP WITH TIME ZONE,
    -- Federated-learning partition key (org/region) so raw records never leave the node
    fl_partition_key TEXT,
    status TEXT NOT NULL DEFAULT 'pending', -- pending | completed | failed
    export_path TEXT,
    checksum TEXT,
    exported_by TEXT,
    started_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    error TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'
);

-- ── Indexes ──────────────────────────────────────────────────────────────────

CREATE INDEX IF NOT EXISTS idx_route_nodes_key ON route_nodes (node_key);
CREATE INDEX IF NOT EXISTS idx_route_nodes_type ON route_nodes (node_type);

CREATE INDEX IF NOT EXISTS idx_route_edges_source ON route_edges (source_node_id);
CREATE INDEX IF NOT EXISTS idx_route_edges_dest ON route_edges (dest_node_id);

CREATE INDEX IF NOT EXISTS idx_shipment_routes_product ON shipment_routes (product_id);
CREATE INDEX IF NOT EXISTS idx_shipment_routes_status ON shipment_routes (status);
CREATE INDEX IF NOT EXISTS idx_shipment_routes_current_node ON shipment_routes (current_node_id);

CREATE INDEX IF NOT EXISTS idx_anomaly_scores_shipment ON anomaly_scores (shipment_route_id);
CREATE INDEX IF NOT EXISTS idx_anomaly_scores_product ON anomaly_scores (product_id);
CREATE INDEX IF NOT EXISTS idx_anomaly_scores_risk ON anomaly_scores (risk_level);
CREATE INDEX IF NOT EXISTS idx_anomaly_scores_computed ON anomaly_scores (computed_at DESC);

CREATE INDEX IF NOT EXISTS idx_routing_alerts_product ON routing_alerts (product_id);
CREATE INDEX IF NOT EXISTS idx_routing_alerts_status ON routing_alerts (status);
CREATE INDEX IF NOT EXISTS idx_routing_alerts_severity ON routing_alerts (severity);
CREATE INDEX IF NOT EXISTS idx_routing_alerts_created ON routing_alerts (created_at DESC);

-- ── Triggers ─────────────────────────────────────────────────────────────────

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_route_nodes_updated_at
    BEFORE UPDATE ON route_nodes
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_route_edges_updated_at
    BEFORE UPDATE ON route_edges
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_shipment_routes_updated_at
    BEFORE UPDATE ON shipment_routes
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_routing_alerts_updated_at
    BEFORE UPDATE ON routing_alerts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
