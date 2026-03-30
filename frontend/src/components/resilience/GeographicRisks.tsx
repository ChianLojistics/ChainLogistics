"""import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { GeographicRisk } from "@/lib/resilience";

interface GeographicRisksProps {
    risks: GeographicRisk[];
}

export function GeographicRisks({ risks }: GeographicRisksProps) {
    return (
        <Card>
            <CardHeader>
                <CardTitle>Geographic Risks</CardTitle>
            </CardHeader>
            <CardContent>
                {risks.map((risk) => (
                    <div key={risk.id} className="mb-4">
                        <p><strong>Location:</strong> {risk.location}</p>
                        <p><strong>Risk Score:</strong> {risk.risk_score}</p>
                    </div>
                ))}
            </CardContent>
        </Card>
    );
}
"""