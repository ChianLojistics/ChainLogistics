"""import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { DisruptionPrediction } from "@/lib/resilience";

interface DisruptionPredictionsProps {
    predictions: DisruptionPrediction[];
}

export function DisruptionPredictions({ predictions }: DisruptionPredictionsProps) {
    return (
        <Card>
            <CardHeader>
                <CardTitle>Disruption Predictions</CardTitle>
            </CardHeader>
            <CardContent>
                {predictions.map((prediction) => (
                    <div key={prediction.id} className="mb-4">
                        <p><strong>Product:</strong> {prediction.product_id}</p>
                        <p><strong>Probability:</strong> {prediction.probability}</p>
                        <p><strong>Impact:</strong> {prediction.impact_level}</p>
                    </div>
                ))}
            </CardContent>
        </Card>
    );
}
"""