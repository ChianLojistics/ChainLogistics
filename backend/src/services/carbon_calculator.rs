/// Carbon footprint calculation engine based on GHG Protocol methodology.
///
/// Emission factors are in kg CO2e per unit:
///   - Transport: kg CO2e per tonne-km
///   - Energy:    kg CO2e per kWh
///   - Packaging: kg CO2e per kg of material
///   - Storage:   kg CO2e per hour (per tonne stored)
use crate::models::carbon::{CalculateFootprintRequest, FootprintBreakdown};

// ── Emission factors (kg CO2e) ────────────────────────────────────────────────

/// Transport emission factors: kg CO2e per tonne-km
mod transport_factors {
    pub const ROAD_DIESEL: f64 = 0.096;
    pub const ROAD_ELECTRIC: f64 = 0.025;
    pub const RAIL: f64 = 0.028;
    pub const SEA: f64 = 0.016;
    pub const AIR: f64 = 0.602;
    pub const DEFAULT: f64 = 0.096; // assume road diesel
}

/// Energy emission factors: kg CO2e per kWh
mod energy_factors {
    pub const GRID_AVERAGE: f64 = 0.233;
    pub const RENEWABLE: f64 = 0.010;
    pub const DIESEL_GENERATOR: f64 = 0.708;
    pub const NATURAL_GAS: f64 = 0.202;
}

/// Packaging emission factors: kg CO2e per kg of material
mod packaging_factors {
    pub const CARDBOARD: f64 = 0.94;
    pub const PLASTIC: f64 = 2.53;
    pub const GLASS: f64 = 0.87;
    pub const METAL: f64 = 1.45;
    pub const MINIMAL: f64 = 0.20;
    pub const DEFAULT: f64 = 0.94;
}

/// Storage emission factor: kg CO2e per tonne per hour (refrigerated warehouse)
const STORAGE_FACTOR_PER_TONNE_HOUR: f64 = 0.0012;

/// Minimum reduction threshold to qualify for credit generation (5%)
const MIN_REDUCTION_PCT_FOR_CREDITS: f64 = 5.0;

// ── Public API ────────────────────────────────────────────────────────────────

pub fn calculate(req: &CalculateFootprintRequest) -> FootprintBreakdown {
    let weight_kg = req.weight_kg.unwrap_or(1000.0); // default 1 tonne
    let weight_tonnes = weight_kg / 1000.0;

    let transport = calculate_transport(
        req.transport_mode.as_deref(),
        req.distance_km.unwrap_or(0.0),
        weight_tonnes,
    );

    // Manufacturing: estimated from weight using a generic factor (0.5 kg CO2e/kg)
    let manufacturing = weight_kg * 0.5;

    let packaging = calculate_packaging(
        req.packaging_type.as_deref(),
        weight_kg,
    );

    let storage = calculate_storage(
        req.storage_hours.unwrap_or(0.0),
        weight_tonnes,
        req.energy_source.as_deref(),
    );

    let total = transport + manufacturing + packaging + storage;

    let (emissions_reduction, reduction_percentage) = match req.baseline_emissions {
        Some(baseline) if baseline > 0.0 => {
            let reduction = (baseline - total).max(0.0);
            let pct = (reduction / baseline) * 100.0;
            (Some(reduction), Some(pct))
        }
        _ => (None, None),
    };

    // Credits are generated only when reduction exceeds the minimum threshold
    let eligible_credits = match (emissions_reduction, reduction_percentage) {
        (Some(reduction), Some(pct)) if pct >= MIN_REDUCTION_PCT_FOR_CREDITS => {
            // Convert kg CO2e → tonnes CO2e
            reduction / 1000.0
        }
        _ => 0.0,
    };

    FootprintBreakdown {
        transport_emissions: round2(transport),
        manufacturing_emissions: round2(manufacturing),
        packaging_emissions: round2(packaging),
        storage_emissions: round2(storage),
        total_emissions: round2(total),
        emissions_reduction: emissions_reduction.map(round2),
        reduction_percentage: reduction_percentage.map(|p| round2(p)),
        eligible_credits: round4(eligible_credits),
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn calculate_transport(mode: Option<&str>, distance_km: f64, weight_tonnes: f64) -> f64 {
    let factor = match mode {
        Some("road") | Some("truck") => transport_factors::ROAD_DIESEL,
        Some("road_electric") => transport_factors::ROAD_ELECTRIC,
        Some("rail") | Some("train") => transport_factors::RAIL,
        Some("sea") | Some("ship") | Some("ocean") => transport_factors::SEA,
        Some("air") | Some("plane") => transport_factors::AIR,
        _ => transport_factors::DEFAULT,
    };
    // kg CO2e = factor (kg/tonne-km) × distance_km × weight_tonnes
    factor * distance_km * weight_tonnes
}

fn calculate_packaging(packaging_type: Option<&str>, weight_kg: f64) -> f64 {
    let factor = match packaging_type {
        Some("cardboard") | Some("paper") => packaging_factors::CARDBOARD,
        Some("plastic") => packaging_factors::PLASTIC,
        Some("glass") => packaging_factors::GLASS,
        Some("metal") | Some("aluminium") | Some("steel") => packaging_factors::METAL,
        Some("minimal") | Some("none") => packaging_factors::MINIMAL,
        _ => packaging_factors::DEFAULT,
    };
    // Packaging weight assumed to be ~5% of product weight
    let packaging_weight_kg = weight_kg * 0.05;
    factor * packaging_weight_kg
}

fn calculate_storage(hours: f64, weight_tonnes: f64, energy_source: Option<&str>) -> f64 {
    let energy_multiplier = match energy_source {
        Some("renewable") | Some("solar") | Some("wind") => 0.1,
        Some("grid") | Some("grid_average") => 1.0,
        Some("diesel") => 3.0,
        Some("natural_gas") => 0.87,
        _ => 1.0,
    };
    STORAGE_FACTOR_PER_TONNE_HOUR * hours * weight_tonnes * energy_multiplier
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

fn round4(v: f64) -> f64 {
    (v * 10_000.0).round() / 10_000.0
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn base_req() -> CalculateFootprintRequest {
        CalculateFootprintRequest {
            product_id: "prod-1".into(),
            tracking_event_id: None,
            transport_mode: Some("road".into()),
            distance_km: Some(500.0),
            energy_source: Some("grid".into()),
            weight_kg: Some(1000.0),
            packaging_type: Some("cardboard".into()),
            storage_hours: Some(24.0),
            baseline_emissions: None,
        }
    }

    #[test]
    fn test_basic_calculation() {
        let result = calculate(&base_req());
        assert!(result.total_emissions > 0.0);
        assert_eq!(
            result.total_emissions,
            round2(
                result.transport_emissions
                    + result.manufacturing_emissions
                    + result.packaging_emissions
                    + result.storage_emissions
            )
        );
    }

    #[test]
    fn test_air_transport_higher_than_sea() {
        let mut req = base_req();
        req.transport_mode = Some("air".into());
        let air = calculate(&req);

        req.transport_mode = Some("sea".into());
        let sea = calculate(&req);

        assert!(air.transport_emissions > sea.transport_emissions);
    }

    #[test]
    fn test_credit_generation_with_sufficient_reduction() {
        let mut req = base_req();
        req.baseline_emissions = Some(10_000.0); // high baseline
        let result = calculate(&req);
        assert!(result.eligible_credits > 0.0);
        assert!(result.reduction_percentage.unwrap() >= 5.0);
    }

    #[test]
    fn test_no_credits_without_baseline() {
        let result = calculate(&base_req());
        assert_eq!(result.eligible_credits, 0.0);
        assert!(result.emissions_reduction.is_none());
    }

    #[test]
    fn test_renewable_storage_lower_emissions() {
        let mut req = base_req();
        req.energy_source = Some("renewable".into());
        let renewable = calculate(&req);

        req.energy_source = Some("diesel".into());
        let diesel = calculate(&req);

        assert!(renewable.storage_emissions < diesel.storage_emissions);
    }
}
