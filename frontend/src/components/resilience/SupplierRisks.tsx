"""import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { SupplierRisk } from "@/lib/resilience";

interface SupplierRisksProps {
    risks: SupplierRisk[];
}

export function SupplierRisks({ risks }: SupplierRisksProps) {
    return (
        <Card>
            <CardHeader>
                <CardTitle>Supplier Risks</CardTitle>
            </CardHeader>
            <CardContent>
                {risks.map((risk) => (
                    <div key={risk.id} className="mb-4">
                        <p><strong>Supplier:</strong> {risk.supplier_name}</p>
                        <p><strong>Risk Score:</strong> {risk.risk_score}</p>
                    </div>
                ))}
            </CardContent>
        </Card>
    );
}
"""