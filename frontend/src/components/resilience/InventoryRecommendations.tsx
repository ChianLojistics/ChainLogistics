"""import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { InventoryRecommendation } from "@/lib/resilience";

interface InventoryRecommendationsProps {
    recommendations: InventoryRecommendation[];
}

export function InventoryRecommendations({ recommendations }: InventoryRecommendationsProps) {
    return (
        <Card>
            <CardHeader>
                <CardTitle>Inventory Recommendations</CardTitle>
            </CardHeader>
            <CardContent>
                {recommendations.map((rec) => (
                    <div key={rec.id} className="mb-4">
                        <p><strong>Product:</strong> {rec.product_id}</p>
                        <p><strong>Recommended Safety Stock:</strong> {rec.recommended_safety_stock}</p>
                        <p><strong>Rationale:</strong> {rec.rationale}</p>
                    </div>
                ))}
            </CardContent>
        </Card>
    );
}
"""