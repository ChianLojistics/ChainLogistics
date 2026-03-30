# Supply Chain Financial Integration Guide

## Overview

ChainLogistics now includes comprehensive financial integration capabilities, enabling supply chain financing, invoice management, and multi-currency payment processing. This transforms the platform from a tracking-only solution to a complete supply chain finance platform.

## Features

### Payment Methods

Support for multiple payment methods:
- Bank transfers
- Credit/debit cards
- Cryptocurrency payments
- Wire transfers

### Transaction Management

- Real-time transaction tracking
- Multi-currency support
- Blockchain transaction linking
- Transaction history and reconciliation

### Invoice Management

- Invoice creation and tracking
- Automated invoice numbering
- Due date management
- Payment status tracking
- Invoice line items

### Supply Chain Financing

- Invoice factoring
- Supply chain financing requests
- Working capital financing
- Financing approval workflow
- Interest rate management

### Wallet Integration

- Multi-chain wallet balance tracking
- Real-time balance updates
- Token symbol support
- Wallet address management

## API Endpoints

### Transactions

#### Create Transaction

```bash
POST /api/v1/admin/transactions
Content-Type: application/json

{
  "transaction_type": "payment",
  "amount": "1000.00",
  "currency": "USD"
}
```

Response:
```json
{
  "id": "uuid",
  "user_id": "uuid",
  "transaction_type": "payment",
  "amount": "1000.00",
  "currency": "USD",
  "status": "pending",
  "blockchain_network": null,
  "blockchain_tx_hash": null
}
```

#### List Transactions

```bash
GET /api/v1/transactions
```

#### Get Transaction

```bash
GET /api/v1/transactions/:id
```

### Invoices

#### Create Invoice

```bash
POST /api/v1/admin/invoices
Content-Type: application/json

{
  "amount": "5000.00",
  "due_date": "2024-12-31"
}
```

Response:
```json
{
  "id": "uuid",
  "user_id": "uuid",
  "invoice_number": "INV-1704067200",
  "amount": "5000.00",
  "currency": "USD",
  "status": "draft",
  "due_date": "2024-12-31"
}
```

### Financing

#### Request Financing

```bash
POST /api/v1/admin/financing/request
Content-Type: application/json

{
  "financing_type": "invoice_factoring",
  "amount": "5000.00"
}
```

Response:
```json
{
  "id": "uuid",
  "user_id": "uuid",
  "financing_type": "invoice_factoring",
  "amount_requested": "5000.00",
  "amount_approved": null,
  "status": "pending",
  "interest_rate": null
}
```

## Database Schema

### payment_methods

```sql
CREATE TABLE payment_methods (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  payment_type VARCHAR(50), -- 'bank_transfer', 'card', 'crypto', 'wire'
  provider VARCHAR(100),
  provider_id VARCHAR(255),
  is_default BOOLEAN,
  is_active BOOLEAN,
  created_at TIMESTAMP,
  updated_at TIMESTAMP
);
```

### transactions

```sql
CREATE TABLE transactions (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  transaction_type VARCHAR(50), -- 'payment', 'refund', 'fee', 'financing'
  amount DECIMAL(20, 8),
  currency VARCHAR(10),
  status VARCHAR(50), -- 'pending', 'completed', 'failed', 'cancelled'
  payment_method_id UUID,
  blockchain_network VARCHAR(50),
  blockchain_tx_hash VARCHAR(255),
  description TEXT,
  metadata JSONB,
  created_at TIMESTAMP,
  updated_at TIMESTAMP,
  completed_at TIMESTAMP
);
```

### invoices

```sql
CREATE TABLE invoices (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  invoice_number VARCHAR(100) UNIQUE,
  amount DECIMAL(20, 8),
  currency VARCHAR(10),
  status VARCHAR(50), -- 'draft', 'sent', 'paid', 'overdue', 'cancelled'
  due_date DATE,
  description TEXT,
  line_items JSONB,
  created_at TIMESTAMP,
  updated_at TIMESTAMP,
  paid_at TIMESTAMP
);
```

### financing_requests

```sql
CREATE TABLE financing_requests (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  invoice_id UUID,
  financing_type VARCHAR(50), -- 'invoice_factoring', 'supply_chain_financing', 'working_capital'
  amount_requested DECIMAL(20, 8),
  amount_approved DECIMAL(20, 8),
  currency VARCHAR(10),
  status VARCHAR(50), -- 'pending', 'approved', 'rejected', 'funded'
  interest_rate DECIMAL(5, 2),
  term_days INTEGER,
  created_at TIMESTAMP,
  updated_at TIMESTAMP,
  approved_at TIMESTAMP,
  funded_at TIMESTAMP
);
```

### wallet_balances

```sql
CREATE TABLE wallet_balances (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  blockchain_network VARCHAR(50),
  wallet_address VARCHAR(255),
  balance DECIMAL(20, 8),
  token_symbol VARCHAR(20),
  last_synced TIMESTAMP
);
```

## Frontend Components

### TransactionHistory

Display user's transaction history:

```tsx
import { TransactionHistory } from '@/components/financial/TransactionHistory';

<TransactionHistory />
```

### InvoiceForm

Create new invoices:

```tsx
import { InvoiceForm } from '@/components/financial/InvoiceForm';

<InvoiceForm />
```

## Integration with Blockchain

### Linking Blockchain Transactions

When a transaction is processed on-chain:

```typescript
// Update transaction with blockchain details
await updateTransaction(transactionId, {
  status: 'confirmed',
  blockchain_network: 'ethereum',
  blockchain_tx_hash: '0x...',
});
```

### Multi-Currency Support

Transactions support any currency:

```typescript
const transaction = await createTransaction({
  amount: '1000',
  currency: 'EUR', // or USD, GBP, JPY, etc.
  transaction_type: 'payment',
});
```

## Revenue Model

### Transaction Fees

- 2% fee on all transactions
- Reduced to 1% for premium tier users
- 0.5% for enterprise customers

### Financing Margins

- 3-5% interest on invoice factoring
- 2-4% interest on supply chain financing
- 1-3% interest on working capital

## Compliance

All financial transactions are:
- Logged in audit trail
- Subject to compliance checks
- Encrypted in transit and at rest
- Compliant with PCI DSS standards

## Troubleshooting

### Transaction Failures

1. Check payment method is active
2. Verify sufficient balance
3. Check transaction amount limits
4. Review error logs

### Invoice Issues

1. Verify due date is in future
2. Check invoice number uniqueness
3. Ensure amount is positive
4. Verify user permissions

### Financing Requests

1. Check financing type is valid
2. Verify amount is reasonable
3. Review user credit history
4. Check for existing pending requests
