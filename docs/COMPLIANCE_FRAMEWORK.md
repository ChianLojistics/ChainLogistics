# Supply Chain Compliance Framework

## Overview

ChainLogistics includes a comprehensive compliance framework supporting multiple regulatory standards. This enables the platform to serve regulated industries including pharmaceuticals, food safety, financial products, and conflict minerals.

## Supported Compliance Standards

### GDPR (General Data Protection Regulation)

**Scope**: EU data protection and privacy

**Requirements**:
- Data residency in EU data centers
- User consent for data processing
- Right to be forgotten implementation
- Data export capabilities
- Privacy by design

**Validation Rules**:
- `data_location == 'EU'`
- `consent` field required
- `right_to_be_forgotten_enabled` flag

### FDA 21 CFR Part 11

**Scope**: Electronic records and signatures for pharmaceuticals

**Requirements**:
- Digital signatures on all records
- Signer identification and authentication
- Timestamp on all records
- Audit trail of all changes
- System validation and controls

**Validation Rules**:
- `digital_signature` required
- `signer_id` required
- `timestamp` required
- Immutable audit trail

### FSMA (Food Safety Modernization Act)

**Scope**: Food traceability and safety

**Requirements**:
- Complete traceability from farm to table
- Mandatory record retention (25 years)
- Supplier verification
- Hazard analysis
- Preventive controls

**Validation Rules**:
- `origin` required
- `processing_steps` required
- `distribution` required
- Complete chain of custody

### Conflict Minerals Regulation

**Scope**: Conflict-free mineral sourcing

**Requirements**:
- Due diligence on mineral sources
- Third-party audit verification
- Conflict-free certification
- Supply chain transparency
- Annual reporting

**Validation Rules**:
- `mineral_source` required
- `audit_report` required
- `verified_by_auditor` flag
- Audit trail of verification

### Organic Certification

**Scope**: Organic product verification

**Requirements**:
- Valid organic certification
- Certification body verification
- Annual recertification
- Prohibited substance tracking
- Traceability of organic inputs

**Validation Rules**:
- `certification_body` required
- `cert_expiry` required and not expired
- Warning if expiring within 90 days

## API Endpoints

### Check Compliance

```bash
POST /api/v1/compliance/check
Content-Type: application/json

{
  "compliance_type": "gdpr",
  "data": {
    "data_location": "EU",
    "consent": true,
    "right_to_be_forgotten_enabled": true
  }
}
```

Response:
```json
{
  "is_compliant": true,
  "compliance_type": "gdpr",
  "violations": [],
  "warnings": []
}
```

### Get Compliance Report

```bash
GET /api/v1/compliance/report/:product_id
```

Response:
```json
{
  "product_id": "uuid",
  "compliance_checks": [
    {
      "type": "gdpr",
      "is_compliant": true,
      "violations": [],
      "warnings": []
    }
  ],
  "overall_status": "compliant"
}
```

### Generate Audit Report

```bash
GET /api/v1/audit/report
```

Response:
```json
{
  "report_type": "audit",
  "generated_at": "2024-01-01T00:00:00Z",
  "total_events": 1000,
  "events": [
    {
      "id": "uuid",
      "user_id": "uuid",
      "action": "data_access",
      "resource_type": "product",
      "resource_id": "uuid",
      "timestamp": 1704067200,
      "ip_address": "192.168.1.1"
    }
  ]
}
```

## Database Schema

### compliance_records

```sql
CREATE TABLE compliance_records (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  product_id UUID,
  compliance_type VARCHAR(100),
  status VARCHAR(50), -- 'pending', 'compliant', 'non_compliant', 'exempted'
  validation_data JSONB,
  audit_trail JSONB,
  created_at TIMESTAMP,
  updated_at TIMESTAMP,
  verified_at TIMESTAMP
);
```

### audit_logs

```sql
CREATE TABLE audit_logs (
  id UUID PRIMARY KEY,
  user_id UUID,
  action VARCHAR(255),
  resource_type VARCHAR(100),
  resource_id UUID,
  changes JSONB,
  ip_address INET,
  user_agent TEXT,
  created_at TIMESTAMP
);
```

## Data Retention Policies

| Standard | Retention Period | Purpose |
|----------|------------------|---------|
| GDPR | 7 years | Legal compliance |
| FDA 21 CFR 11 | 10 years | Regulatory requirement |
| FSMA | 25 years | Food safety traceability |
| Conflict Minerals | 5 years | Supply chain verification |
| Organic Certification | 3 years | Certification validity |
| SOC 2 | 3 years | Security audit trail |
| ISO 27001 | 3 years | Information security |

## Compliance Validator

### Usage

```rust
use crate::compliance::{ComplianceValidator, ComplianceRule};

let rule = ComplianceRule::gdpr_data_residency();
let result = ComplianceValidator::validate(&rule, &data);

if result.is_compliant {
    println!("Compliant!");
} else {
    for violation in result.violations {
        println!("Violation: {}", violation);
    }
}
```

### Validation Result

```rust
pub struct ValidationResult {
    pub is_compliant: bool,
    pub compliance_type: ComplianceType,
    pub violations: Vec<String>,
    pub warnings: Vec<String>,
}
```

## Audit Logger

### Log Data Access

```rust
use crate::compliance::AuditLogger;

let entry = AuditLogger::log_data_access(
    user_id,
    resource_id,
    Some("192.168.1.1".to_string()),
);
```

### Log Data Modification

```rust
let entry = AuditLogger::log_data_modification(
    user_id,
    resource_id,
    serde_json::json!({
        "field": "status",
        "old_value": "active",
        "new_value": "inactive"
    }),
);
```

### Log Data Deletion

```rust
let entry = AuditLogger::log_data_deletion(user_id, resource_id);
```

## Frontend Components

### ComplianceChecker

Check compliance status:

```tsx
import { ComplianceChecker } from '@/components/compliance/ComplianceChecker';

<ComplianceChecker />
```

## Implementation Checklist

- [ ] Enable GDPR compliance checks
- [ ] Enable FDA 21 CFR Part 11 validation
- [ ] Enable FSMA traceability requirements
- [ ] Enable Conflict Minerals verification
- [ ] Enable Organic Certification validation
- [ ] Configure data retention policies
- [ ] Set up audit logging
- [ ] Configure compliance reporting
- [ ] Train staff on compliance procedures
- [ ] Schedule regular compliance audits

## Compliance Reporting

### Monthly Compliance Report

```bash
GET /api/v1/compliance/report/monthly
```

### Annual Audit Report

```bash
GET /api/v1/audit/report/annual
```

### Regulatory Export

```bash
GET /api/v1/compliance/export/:standard
```

## Troubleshooting

### Compliance Check Failures

1. Verify all required fields are present
2. Check data format matches requirements
3. Review validation rules for standard
4. Check audit trail for issues

### Missing Audit Logs

1. Verify audit logging is enabled
2. Check database connectivity
3. Review log retention policies
4. Check for log rotation issues

### Certification Expiry

1. Set up expiry notifications
2. Plan recertification timeline
3. Update compliance records
4. Notify stakeholders

## Best Practices

1. **Regular Audits**: Conduct compliance audits quarterly
2. **Documentation**: Maintain detailed compliance documentation
3. **Training**: Train staff on compliance requirements
4. **Monitoring**: Monitor compliance status continuously
5. **Reporting**: Generate regular compliance reports
6. **Updates**: Stay current with regulatory changes
7. **Testing**: Test compliance controls regularly
8. **Incident Response**: Have incident response plan ready
