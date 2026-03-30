"""'use client';

import { useEffect, useState } from 'react';
import { ResilienceMetrics } from '@/lib/resilience';
import { DisruptionPredictions } from '@/components/resilience/DisruptionPredictions';
import { SupplierRisks } from '@/components/resilience/SupplierRisks';
import { GeographicRisks } from '@/components/resilience/GeographicRisks';
import { AlternativeSources } from '@/components/resilience/AlternativeSources';
import { InventoryRecommendations } from '@/components/resilience/InventoryRecommendations';

export default function ResiliencePage() {
    const [metrics, setMetrics] = useState<ResilienceMetrics | null>(null);
    const [productId, setProductId] = useState('some-product-id'); // Replace with actual product ID

    useEffect(() => {
        async function fetchMetrics() {
            const res = await fetch(`/api/resilience/${productId}`);
            const data = await res.json();
            setMetrics(data);
        }

        fetchMetrics();
    }, [productId]);

    if (!metrics) {
        return <div>Loading...</div>;
    }

    return (
        <div className="container mx-auto p-4">
            <h1 className="text-2xl font-bold mb-4">Supply Chain Resilience</h1>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                <DisruptionPredictions predictions={metrics.disruption_predictions} />
                <SupplierRisks risks={metrics.supplier_risks} />
                <GeographicRisks risks={metrics.geographic_risks} />
                <AlternativeSources sources={metrics.alternative_sources} />
                <InventoryRecommendations recommendations={metrics.inventory_recommendations} />
            </div>
        </div>
    );
}
"""