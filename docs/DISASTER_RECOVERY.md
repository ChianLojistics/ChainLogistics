# Disaster Recovery Plan for ChainLogistics

## Executive Summary

This document defines the Disaster Recovery (DR) strategy for ChainLogistics to ensure business continuity and rapid recovery from infrastructure failures, data loss, or other catastrophic events.

## Recovery Objectives

| Metric | Target | Rationale |
|--------|--------|-----------|
| **RPO** (Recovery Point Objective) | 1 hour | Acceptable data loss = 1 hour of transactions |
| **RTO** (Recovery Time Objective) | 4 hours | Maximum acceptable downtime for operations |
| **MTTR** (Mean Time to Repair) | 30 minutes | Target average recovery time |
| **Availability Target** | 99.5% | 3.7 hours downtime per month acceptable |

## Disaster Classification

### Tier 1: Severe (Full Infrastructure Loss)
- Complete data center failure
- Multiple region outage
- Ransomware attack
- **Response Time**: Activate immediate failover

### Tier 2: Major (Partial Service Loss)
- Database corruption
- Primary database node failure
- Smart contract compromise
- **Response Time**: Failover within 30 minutes

### Tier 3: Minor (Degraded Performance)
- Single server failure
- Performance degradation
- API rate limiting issues
- **Response Time**: Manual recovery within 4 hours

## Disaster Recovery Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Production Environment                    │
│  Primary Region (us-east-1)                                  │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Database Cluster                                      │   │
│  │ ├─ Primary (Writes)                                  │   │
│  │ ├─ Standby 1 (Sync Replication)                     │   │
│  │ └─ Standby 2 (Async Replication)                    │   │
│  └──────────────────────────────────────────────────────┘   │
│              ↓ Streaming Replication ↓                       │
└─────────────────────────────────────────────────────────────┘
                        │
                        │ Daily Backup
                        ↓
┌─────────────────────────────────────────────────────────────┐
│              Backup Region (us-west-2)                       │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Warm Standby Database                                │   │
│  │ ├─ Read-only replica                                │   │
│  │ ├─ 30-minute lag from primary                       │   │
│  │ └─ Can be promoted to primary in seconds            │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                        │
                        │ S3 Backups
                        ↓
┌─────────────────────────────────────────────────────────────┐
│          Cold Storage (Glacier Archive)                      │
│  - Monthly snapshots                                         │
│  - 1-year retention                                          │
│  - 24-hour retrieval time                                    │
└─────────────────────────────────────────────────────────────┘
```

## Recovery Scenarios

### Scenario 1: Primary Database Failure

**Symptoms**:
- Database connection timeouts
- Read/write errors
- High error rate in logs

**Recovery Steps**:

1. **Detection** (Automatic - 2 min):
   - Health checks detect primary down
   - CloudWatch alarm triggered
   - Slack notification sent

2. **Diagnosis** (Manual - 5 min):
   ```bash
   # Check standby status
   psql -h standby-1 -c "SELECT status FROM pg_stat_replication;"
   
   # Check replication lag
   psql -h standby-1 -c "SELECT slot_name, restart_lsn FROM pg_replication_slots;"
   ```

3. **Failover** (Automatic - 30 sec):
   ```bash
   # Promote standby to primary
   pg_ctl promote -D /var/lib/postgresql/data
   
   # Update DNS to point to new primary
   aws route53 change-resource-record-sets \
     --hosted-zone-id Z123ABC \
     --change-batch '{
       "Changes": [{
         "Action": "UPSERT",
         "ResourceRecordSet": {
           "Name": "db.chainlogistics.internal",
           "Type": "CNAME",
           "TTL": 60,
           "ResourceRecords": [{"Value": "standby-1.internal"}]
         }
       }]
     }'
   ```

4. **Application Recovery** (Automatic - 2 min):
   - Applications detect new primary via DNS
   - Connection pool refreshes
   - Queries resume

5. **Verification** (Manual - 5 min):
   ```bash
   # Verify application connectivity
   curl -s http://api.chainlogistics.com/health | jq .
   
   # Check error rates
   aws cloudwatch get-metric-statistics \
     --namespace ChainLogistics \
     --metric-name ErrorRate \
     --start-time $(date -u -d '10 min ago' +%Y-%m-%dT%H:%M:%S) \
     --end-time $(date -u +%Y-%m-%dT%H:%M:%S) \
     --period 300 \
     --statistics Average
   ```

**Total Recovery Time**: ~10 minutes
**Data Loss**: 0 (synchronous replication)

### Scenario 2: Data Corruption

**Symptoms**:
- Application crashes on specific queries
- Integrity constraint violations
- Unexplained data inconsistencies

**Recovery Steps**:

1. **Isolation** (Immediate):
   ```bash
   # Stop applications from writing
   kubectl set env deployment/api DB_READ_ONLY=true
   
   # Create point-in-time snapshot
   aws rds create-db-snapshot \
     --db-instance-identifier chainlogistics-prod \
     --db-snapshot-identifier chainlogistics-corruption-$(date +%s)
   ```

2. **Analysis** (5-10 min):
   ```sql
   -- Check for integrity violations
   SELECT table_name FROM information_schema.tables
   WHERE table_schema = 'public';
   
   -- Validate specific tables
   REINDEX TABLE products;
   REINDEX TABLE events;
   ```

3. **Restore from Backup** (15-30 min):
   ```bash
   # Restore to last known good backup (1 hour old)
   RESTORE_POINT=$(date -d '1 hour ago' +%Y-%m-%dT%H:%M:%S)
   
   pg_restore --dbname=chainlogistics_restore \
     backup_2024_01_15_01_00.sql.gz
   
   # Verify restored data
   psql -d chainlogistics_restore -c "SELECT COUNT(*) FROM products;"
   ```

4. **Application Recovery**:
   ```bash
   # Point to restored database
   kubectl set env deployment/api DB_HOST=restored-db-instance
   
   # Re-enable writes
   kubectl set env deployment/api DB_READ_ONLY=false
   ```

5. **Investigation**:
   - Analyze what caused corruption
   - Update validation rules
   - Deploy fixes

**Total Recovery Time**: 30-60 minutes
**Data Loss**: Up to 1 hour

### Scenario 3: Smart Contract Compromise

**Symptoms**:
- Unauthorized transactions
- Token balance discrepancies
- Suspicious contract calls

**Recovery Steps**:

1. **Emergency Action** (Immediate):
   ```bash
   # Pause contract operations
   soroban contract invoke \
     --id CONTRACTID \
     --fn pause
   
   # Notify users
   kubectl create configmap incident-notice \
     --from-literal=status="Contract paused for security investigation"
   ```

2. **Forensics** (30-60 min):
   ```bash
   # Export transaction history
   soroban contract invoke \
     --id CONTRACTID \
     --fn get_events \
     --arg-from-xdr '...' > /tmp/events.xdr
   
   # Analyze blockchain logs
   stellar_lfb="https://horizon-testnet.stellar.org"
   curl "$stellar_lfb/accounts/CONTRACTID"
   ```

3. **Remediation**:
   - Deploy patched contract
   - Update contract code
   - Verify fix on testnet first

4. **Deployment**:
   ```bash
   # Deploy to testnet first
   soroban contract deploy --network testnet \
     --path target/wasm32-unknown-unknown/release/contract.wasm
   
   # Verify on testnet (24-48 hours)
   # Then deploy to mainnet
   soroban contract deploy --network public \
     --path target/wasm32-unknown-unknown/release/contract.wasm
   ```

**Total Recovery Time**: 4-24 hours
**Data Loss**: Potentially reversible via blockchain

### Scenario 4: Complete Region Failure

**Symptoms**:
- All services in region unavailable
- Network timeouts
- All health checks failing

**Recovery Steps**:

1. **Activate Failover** (Immediate):
   ```bash
   # Promote warm standby in backup region
   kubectl config use-context chainlogistics-backup-region
   
   # Promote database
   aws rds promote-read-replica \
     --db-instance-identifier chainlogistics-backup-replica
   ```

2. **Traffic Reroute** (1-2 min):
   ```bash
   # Update global load balancer
   aws globalaccelerator update-accelerator \
     --accelerator-arn arn:aws:globalaccelerator::ACCOUNT:accelerator/chainlogistics \
     --traffic-dial-percentage '{
       "primary-region": 0,
       "backup-region": 100
     }'
   ```

3. **Verify Backup Systems**:
   ```bash
   # Health checks in backup region
   for i in {1..10}; do
     curl -s https://backup.chainlogistics.com/health
     sleep 5
   done
   ```

4. **Communicate Status**:
   ```bash
   # Update status page
   aws sns publish \
     --topic-arn arn:aws:sns:us-east-1:ACCOUNT:disasters \
     --subject "ChainLogistics Failover Activated" \
     --message "Traffic rerouted to backup region. ETA full recovery: 30 min"
   ```

**Total Recovery Time**: 5-15 minutes
**Data Loss**: 0-30 minutes (depending on replication lag)

## Pre-Disaster Preparation

### Pre-Disaster Checklist

**Monthly Review** (1st of each month):
- [ ] Review and update this DR plan
- [ ] Update contact information
- [ ] Verify backup integrity
- [ ] Test restore procedures

**Quarterly Validation** (Every 3 months):
- [ ] Conduct partial failover test
- [ ] Verify RTO/RPO targets
- [ ] Test backup restoration
- [ ] Document findings

**Annual Full DR Exercise**:
- [ ] Complete infrastructure failover
- [ ] Measure actual RTO/RPO
- [ ] Test team communication
- [ ] Update playbooks

### Infrastructure Requirements

**Primary Region (us-east-1)**:
- 3-node database cluster
- Load balancer
- 2 AZs minimum

**Backup Region (us-west-2)**:
- Warm standby database
- Load balancer
- Cold storage for archives

**Redundancy**:
- Multiple subnets across AZs
- Multiple DNS providers
- Separate security groups per component

### Automation Setup

```yaml
# Helm Chart for DR-Aware Deployment
apiVersion: helm.sh/v1
kind: Chart
metadata:
  name: chainlogistics-ha
spec:
  values:
    replication:
      enabled: true
      regions:
        - name: primary
          endpoint: db-primary.us-east-1.rds.amazonaws.com
          priority: 1
        - name: backup
          endpoint: db-backup.us-west-2.rds.amazonaws.com
          priority: 2
    healthchecks:
      interval: 30s
      timeout: 5s
      failureThreshold: 3
```

## DR Team Responsibilities

### Incident Commander
- Declares disaster level (Tier 1/2/3)
- Activates DR procedures
- Communicates with stakeholders
- Approves failover decisions

### Database Administrator
- Monitors replication status
- Executes failover procedures
- Validates data integrity
- Manages backup operations

### Infrastructure Engineer
- Updates DNS records
- Configures load balancers
- Monitors resource utilization
- Verifies connectivity

### Application Developer
- Validates application connectivity
- Checks business logic
- Monitors error rates
- Tests critical workflows

### Communications Lead
- Updates status page
- Notifies users
- Coordinates with teams
- Documents timeline

## Communication Plan

### Escalation Chain

```
Event Detected (Automated Alert)
    ↓
On-Call DBA (Page via PagerDuty)
    ↓
Incident Commander (If RTO > 15 min)
    ↓
Team Leads (If RTO > 30 min)
    ↓
Executive Stakeholders (If RTO > 1 hour or data loss expected)
```

### Notification Templates

**Status Page Update**:
```
⚠️ SERVICE DISRUPTION
The ChainLogistics API is experiencing intermittent connectivity issues.
Our team is investigating and will provide updates every 15 minutes.
```

**Email Notification**:
```
Subject: ChainLogistics Incident Alert - Database Failover Initiated

Time: 2024-01-15 14:32:00 UTC
Severity: Major
Impact: Read-only mode active, writes temporarily suspended

Actions Taken:
- Database failover initiated
- Promoting standby replica to primary
- ETA for full recovery: 15 minutes

We will provide updates every 5 minutes.
```

## Monitoring and Alerting

### Critical Metrics

```yaml
Alerts:
  - ReplicationLagHigh:
      threshold: 30s
      action: page_on_call
  
  - PrimaryDatabaseDown:
      threshold: immediate
      action: page_on_call + activate_failover
  
  - BackupVerificationFailed:
      threshold: 24h
      action: create_ticket + page_dba
  
  - DRTestFailed:
      threshold: monthly_test_end
      action: create_ticket + notify_lead
```

### CloudWatch Dashboard

```json
{
  "widgets": [
    {
      "type": "metric",
      "properties": {
        "metrics": [
          ["AWS/RDS", "DatabaseConnections"],
          ["AWS/RDS", "ReplicaLag"],
          ["ChainLogistics", "BackupSize"],
          ["ChainLogistics", "RestoreTime"]
        ],
        "period": 300,
        "stat": "Average",
        "region": "us-east-1",
        "yAxis": {
          "left": {
            "min": 0
          }
        }
      }
    }
  ]
}
```

## Testing and Maintenance

### Monthly DR Drill

**Objective**: Verify failover procedures

```bash
#!/bin/bash

echo "=== Monthly DR Drill - $(date) ==="

# 1. Create backup from latest snapshot
echo "Creating test database from backup..."
aws rds restore-db-instance-from-db-snapshot \
  --db-instance-identifier chainlogistics-dr-test-$(date +%s) \
  --db-snapshot-identifier chainlogistics-latest

# 2. Run validation queries
echo "Validating data integrity..."
QUERIES=(
  "SELECT COUNT(*) FROM products;"
  "SELECT COUNT(*) FROM events;"
  "SELECT MAX(created_at) FROM events;"
)

for query in "${QUERIES[@]}"; do
  psql -h chainlogistics-dr-test-db -d chainlogistics -c "$query"
done

# 3. Measure restoration time
echo "Measuring restoration time..."
START=$(date +%s)
# Perform restore operations
END=$(date +%s)
echo "Restoration completed in $((END - START)) seconds"

# 4. Cleanup
echo "Cleaning up test database..."
aws rds delete-db-instance \
  --db-instance-identifier chainlogistics-dr-test-db \
  --skip-final-snapshot

echo "=== DR Drill Complete ==="
```

### Quarterly Full-Scale Exercise

- Simulate complete primary region failure
- Activate backup region (controlled test)
- Measure actual RTO/RPO
- Document all issues and resolutions
- Update playbooks

### Annual Executive Briefing

- Present DR capabilities to leadership
- Review past incidents and resolutions
- Discuss investment in DR infrastructure
- Plan for improvements

## Post-Disaster Actions

### Immediate After-Action (First Hour)
1. ✅ Verify all systems operational
2. ✅ Confirm data integrity
3. ✅ Monitor error rates
4. ✅ Notify users of resolution

### Investigation (First Day)
1. Collect logs and metrics
2. Determine root cause
3. Identify contributing factors
4. Document timeline

### Remediation (First Week)
1. Implement permanent fixes
2. Enhance monitoring/alerting
3. Update procedures
4. Conduct team training

### Communication (Ongoing)
1. Post-mortem report
2. Customer notification
3. Transparency blog post
4. Lessons learned documentation

## Compliance and Auditing

### Compliance Requirements
- ✅ SOC 2 Type II (annual audit)
- ✅ GDPR (data protection)
- ✅ Industry standards (per region)

### Audit Trail
```sql
-- Track all DR activities
CREATE TABLE dr_audit_log (
  id UUID PRIMARY KEY,
  event_type VARCHAR(50),
  severity VARCHAR(20),
  action_taken TEXT,
  initiated_by VARCHAR(100),
  timestamp TIMESTAMPTZ DEFAULT NOW(),
  status VARCHAR(20),
  duration_seconds INT,
  data_loss_bytes BIGINT
);
```

## Appendix

### A. Contact Information
- On-Call DBA: [PagerDuty]
- Incident Commander: [Slack: #incidents]
- Infrastructure Team: [Slack: #infrastructure]

### B. Useful Commands
```bash
# Check replication status
psql -h primary-db -c "SELECT * FROM pg_stat_replication;"

# Promote standby
pg_ctl promote -D /var/lib/postgresql/data

# Check S3 backups
aws s3 ls s3://chain-logistics-backups/ --recursive

# Monitor RTO
time pg_restore --dbname=test backup.dump
```

### C. References
- AWS RDS Failover: https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/Concepts.MultiAZ.html
- PostgreSQL Replication: https://www.postgresql.org/docs/current/warm-standby.html
- DR Best Practices: https://aws.amazon.com/blogs/publicsector/establishing-a-disaster-recovery-and-business-continuity-plan/

