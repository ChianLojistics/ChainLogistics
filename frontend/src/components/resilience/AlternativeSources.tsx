"""import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { AlternativeSource } from "@/lib/resilience";

interface AlternativeSourcesProps {
    sources: AlternativeSource[];
}

export function AlternativeSources({ sources }: AlternativeSourcesProps) {
    return (
        <Card>
            <CardHeader>
                <CardTitle>Alternative Sources</CardTitle>
            </CardHeader>
            <CardContent>
                {sources.map((source) => (
                    <div key={source.id} className="mb-4">
                        <p><strong>Product:</strong> {source.product_id}</p>
                        <p><strong>Alternative Supplier:</strong> {source.alternative_supplier}</p>
                        <p><strong>Viability Score:</strong> {source.viability_score}</p>
                    </div>
                ))}
            </CardContent>
        </Card>
    );
}
"""