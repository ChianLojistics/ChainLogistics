'use client';

import React, { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { toast } from 'sonner';

interface ComplianceCheckResult {
  is_compliant: boolean;
  compliance_type: string;
  violations: string[];
  warnings: string[];
}

export function ComplianceChecker() {
  const [selectedType, setSelectedType] = useState<string>('gdpr');
  const [result, setResult] = useState<ComplianceCheckResult | null>(null);
  const [loading, setLoading] = useState(false);

  const complianceTypes = [
    { value: 'gdpr', label: 'GDPR' },
    { value: 'fda_21_cfr_11', label: 'FDA 21 CFR Part 11' },
    { value: 'fsma', label: 'FSMA' },
    { value: 'conflict_minerals', label: 'Conflict Minerals' },
    { value: 'organic_certification', label: 'Organic Certification' },
  ];

  const handleCheck = async () => {
    setLoading(true);
    try {
      const response = await fetch('/api/v1/compliance/check', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          compliance_type: selectedType,
          data: {}, // TODO: Get actual product data
        }),
      });

      if (!response.ok) throw new Error('Failed to check compliance');

      const data = await response.json();
      setResult(data);

      if (data.is_compliant) {
        toast.success('Compliance check passed');
      } else {
        toast.error(`${data.violations.length} compliance violations found`);
      }
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-4">
      <Card className="p-6">
        <h2 className="text-2xl font-bold mb-4">Compliance Checker</h2>
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-2">Compliance Type</label>
            <select
              value={selectedType}
              onChange={(e) => setSelectedType(e.target.value)}
              className="w-full border rounded px-3 py-2"
            >
              {complianceTypes.map((type) => (
                <option key={type.value} value={type.value}>
                  {type.label}
                </option>
              ))}
            </select>
          </div>
          <Button onClick={handleCheck} disabled={loading} className="w-full">
            {loading ? 'Checking...' : 'Check Compliance'}
          </Button>
        </div>
      </Card>

      {result && (
        <Card className="p-6">
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <h3 className="text-lg font-semibold">Results</h3>
              <Badge variant={result.is_compliant ? 'default' : 'destructive'}>
                {result.is_compliant ? 'Compliant' : 'Non-Compliant'}
              </Badge>
            </div>

            {result.violations.length > 0 && (
              <div>
                <h4 className="font-medium text-red-600 mb-2">Violations</h4>
                <ul className="space-y-1">
                  {result.violations.map((violation, idx) => (
                    <li key={idx} className="text-sm text-red-600">
                      • {violation}
                    </li>
                  ))}
                </ul>
              </div>
            )}

            {result.warnings.length > 0 && (
              <div>
                <h4 className="font-medium text-yellow-600 mb-2">Warnings</h4>
                <ul className="space-y-1">
                  {result.warnings.map((warning, idx) => (
                    <li key={idx} className="text-sm text-yellow-600">
                      • {warning}
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        </Card>
      )}
    </div>
  );
}
