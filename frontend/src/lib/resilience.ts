"""export interface DisruptionPrediction {
    id: string;
    product_id: string;
    predicted_at: string;
    probability: number;
    impact_level: string;
    details: any;
    created_at: string;
}

export interface SupplierRisk {
    id: string;
    supplier_name: string;
    risk_score: number;
    risk_factors: any;
    last_assessed_at: string;
}

export interface GeographicRisk {
    id: string;
    location: string;
    risk_score: number;
    risk_factors: any;
    last_assessed_at: string;
}

export interface AlternativeSource {
    id: string;
    product_id: string;
    alternative_supplier: string;
    viability_score: number;
    details: any;
}

export interface InventoryRecommendation {
    id: string;
    product_id: string;
    recommended_safety_stock: number;
    rationale: string;
    generated_at: string;
}

export interface ResilienceMetrics {
    disruption_predictions: DisruptionPrediction[];
    supplier_risks: SupplierRisk[];
    geographic_risks: GeographicRisk[];
    alternative_sources: AlternativeSource[];
    inventory_recommendations: InventoryRecommendation[];
}
"""