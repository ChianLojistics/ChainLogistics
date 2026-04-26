'use client';

import React, { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Card } from '@/components/ui/card';
import { toast } from 'sonner';

export function InvoiceForm() {
  const [formData, setFormData] = useState({ amount: '', due_date: '' });
  const [errors, setErrors] = useState<{ amount?: string; due_date?: string }>({});
  const [loading, setLoading] = useState(false);

  const validate = (): boolean => {
    const newErrors: typeof errors = {};
    const amount = parseFloat(formData.amount);
    if (!formData.amount || isNaN(amount) || amount <= 0) {
      newErrors.amount = 'Amount must be a positive number';
    }
    if (!formData.due_date) {
      newErrors.due_date = 'Due date is required';
    } else if (new Date(formData.due_date) <= new Date()) {
      newErrors.due_date = 'Due date must be in the future';
    }
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    setFormData((prev) => ({ ...prev, [name]: value }));
    setErrors((prev) => ({ ...prev, [name]: undefined }));
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!validate()) return;
    setLoading(true);
    try {
      const response = await fetch('/api/v1/invoices', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(formData),
      });
      if (!response.ok) throw new Error('Failed to create invoice');
      toast.success('Invoice created successfully');
      setFormData({ amount: '', due_date: '' });
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  return (
    <Card className="p-6">
      <h2 className="text-2xl font-bold mb-4">Create Invoice</h2>
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label className="block text-sm font-medium mb-1">Amount</label>
          <Input
            type="number"
            name="amount"
            value={formData.amount}
            onChange={handleChange}
            placeholder="0.00"
            step="0.01"
            min="0.01"
            required
            aria-invalid={!!errors.amount}
            aria-describedby={errors.amount ? 'amount-error' : undefined}
          />
          {errors.amount && <p id="amount-error" role="alert" className="text-xs text-red-500 mt-1">{errors.amount}</p>}
        </div>
        <div>
          <label className="block text-sm font-medium mb-1">Due Date</label>
          <Input
            type="date"
            name="due_date"
            value={formData.due_date}
            onChange={handleChange}
            required
            aria-invalid={!!errors.due_date}
            aria-describedby={errors.due_date ? 'due-date-error' : undefined}
          />
          {errors.due_date && <p id="due-date-error" role="alert" className="text-xs text-red-500 mt-1">{errors.due_date}</p>}
        </div>
        <Button type="submit" disabled={loading} className="w-full">
          {loading ? 'Creating...' : 'Create Invoice'}
        </Button>
      </form>
    </Card>
  );
}
