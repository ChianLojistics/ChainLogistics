'use client';

import React, { useEffect, useState } from 'react';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { LoadingSpinner } from '@/components/ui/loading-spinner';

interface Transaction {
  id: string;
  transaction_type: string;
  amount: string;
  currency: string;
  status: string;
  blockchain_network?: string;
  blockchain_tx_hash?: string;
  created_at?: string;
}

export function TransactionHistory() {
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchTransactions = async () => {
      try {
        const response = await fetch('/api/v1/transactions');
        if (!response.ok) throw new Error('Failed to fetch transactions');
        const data = await response.json();
        setTransactions(data);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Unknown error');
      } finally {
        setLoading(false);
      }
    };

    fetchTransactions();
  }, []);

  if (loading) return <LoadingSpinner />;
  if (error) return <div className="text-red-500">Error: {error}</div>;

  return (
    <div className="space-y-4">
      <h2 className="text-2xl font-bold">Transaction History</h2>
      {transactions.length === 0 ? (
        <p className="text-gray-500">No transactions yet</p>
      ) : (
        <div className="space-y-2">
          {transactions.map((tx) => (
            <Card key={tx.id} className="p-4">
              <div className="flex justify-between items-start">
                <div>
                  <p className="font-semibold">{tx.transaction_type}</p>
                  <p className="text-sm text-gray-600">
                    {tx.amount} {tx.currency}
                  </p>
                  {tx.blockchain_network && (
                    <p className="text-xs text-gray-500 mt-1">
                      Network: {tx.blockchain_network}
                    </p>
                  )}
                </div>
                <Badge
                  variant={
                    tx.status === 'completed'
                      ? 'default'
                      : tx.status === 'pending'
                        ? 'secondary'
                        : 'destructive'
                  }
                >
                  {tx.status}
                </Badge>
              </div>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
