# Backup Strategy for ChainLogistics

## Overview

This document outlines the automated backup strategy for the ChainLogistics platform, ensuring data safety, business continuity, and compliance with data protection standards.

## Backup Objectives

- **RPO (Recovery Point Objective)**: Maximum 1 hour of data loss
- **RTO (Recovery Time Objective)**: Maximum 4 hours to restore operations
- **Data Retention**: 30 days of daily backups, 12 months of weekly backups
- **Compliance**: GDPR, data protection regulations

## Backup Scope

### Critical Data

1. **Database**
   - Product information and metadata
   - Event tracking data
   - User authentication credentials
   - Blockchain transaction records
   - Analytics and reporting data

2. **Blockchain Data**
   - Smart contract state
   - Transaction logs
   - Event history

3. **Application Configuration**
   - Environment configurations
   - Deployment manifests
   - SSL certificates
   - API keys and secrets (encrypted)

4. **User Data**
   - User profiles and preferences
   - Wallet addresses and balances
   - Transaction history

### Non-Critical Data

- Cache data (can be rebuilt)
- Temporary files
- Build artifacts
- Log files (retained separately)

## Backup Architecture

### Multi-Layer Backup Strategy

```
┌─────────────────────────────────────────────────┐
│          Production Environment                  │
│  ┌──────────────────────────────────────────┐   │
│  │       Primary Database (PostgreSQL)       │   │
│  └──────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
           │                   │
           ↓                   ↓
    ┌─────────────┐    ┌──────────────────────┐
    │ Point-in-   │    │  Continuous          │
    │ Time        │    │  Replication (WAL)   │
    │ Snapshots   │    │  to Backup DB        │
    └─────────────┘    └──────────────────────┘
           │                   │
           ↓                   ↓
    ┌──────────────────────────────────────────┐
    │    AWS S3 / Cloud Storage (Encrypted)    │
    │  - Hourly incremental backups            │
    │  - Daily full backups                    │
    │  - Versioning enabled                    │
    │  - Cross-region replication              │
    └──────────────────────────────────────────┘
```

### Backup Types

#### 1. Full Backups
- **Frequency**: Daily at 2:00 AM UTC
- **Duration**: ~30-45 minutes
- **Retention**: 30 days
- **Compression**: gzip + encryption (AES-256)
- **Location**: Primary cloud region + secondary region (replicated)

#### 2. Incremental Backups
- **Frequency**: Every 4 hours (6 per day)
- **Duration**: ~5-10 minutes
- **Retention**: 30 days
- **Based on**: Transaction logs (WAL)
- **Size**: Typically 50-100 MB

#### 3. Continuous Replication
- **Type**: Streaming replication to standby database
- **Lag**: < 1 second
- **Location**: Same region, different availability zone
- **Purpose**: Immediate failover capability

#### 4. Archive Backups
- **Frequency**: Weekly snapshots
- **Retention**: 12 months
- **Location**: Separate cold storage (lower cost)
- **Purpose**: Long-term compliance and audit

## Backup Implementation

### Prerequisites

```bash
# Required tools
- aws-cli >= 2.0 (for S3 backups)
- pg_dump/pg_basebackup (PostgreSQL utilities)
- Docker (for containerized backup service)
- Kubernetes (for orchestration)
- GPG (for encryption)
```

### Automated Backup Service

#### Docker Image Setup

```dockerfile
FROM postgres:16-alpine

RUN apk add --no-cache \
    aws-cli \
    postgresql-client \
    bash \
    curl \
    gpg

COPY backup.sh /usr/local/bin/backup.sh
RUN chmod +x /usr/local/bin/backup.sh

ENTRYPOINT ["bash", "/usr/local/bin/backup.sh"]
```

#### Backup Script (`scripts/backup.sh`)

```bash
#!/bin/bash

set -e

# Configuration
BACKUP_DIR="/backups"
S3_BUCKET="${BACKUP_BUCKET:-chain-logistics-backups}"
DB_HOST="${DB_HOST:-postgres}"
DB_USER="${DB_USER:-postgres}"
DB_NAME="${DB_NAME:-chainlogistics}"
RETENTION_DAYS=30
ARCHIVE_RETENTION_DAYS=365

# Logging
LOG_FILE="/var/log/backup.log"
log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Generate backup filename with timestamp
TIMESTAMP=$(date +'%Y%m%d_%H%M%S')
BACKUP_FILE="$BACKUP_DIR/backup_${TIMESTAMP}.sql.gz"
MANIFEST_FILE="$BACKUP_DIR/backup_${TIMESTAMP}.manifest"

log "Starting backup: $BACKUP_FILE"

# Perform full database backup
pg_dump \
    -h "$DB_HOST" \
    -U "$DB_USER" \
    -d "$DB_NAME" \
    --format=custom \
    --verbose \
    --file="$BACKUP_DIR/backup_${TIMESTAMP}.dump" \
    2>&1 | tee -a "$LOG_FILE"

# Compress backup
log "Compressing backup..."
gzip -9 "$BACKUP_DIR/backup_${TIMESTAMP}.dump"

# Encrypt backup (optional, for sensitive environments)
if [ -n "$BACKUP_ENCRYPTION_KEY" ]; then
    log "Encrypting backup..."
    gpg --symmetric \
        --cipher-algo AES256 \
        --batch --yes \
        --passphrase "$BACKUP_ENCRYPTION_KEY" \
        "$BACKUP_FILE"
    rm "$BACKUP_FILE"
    BACKUP_FILE="${BACKUP_FILE}.gpg"
fi

# Calculate checksum
CHECKSUM=$(sha256sum "$BACKUP_FILE" | awk '{print $1}')
log "Backup checksum: $CHECKSUM"

# Create manifest
cat > "$MANIFEST_FILE" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "filename": "$(basename "$BACKUP_FILE")",
  "checksum_sha256": "$CHECKSUM",
  "size_bytes": $(stat -f%z "$BACKUP_FILE" 2>/dev/null || stat -c%s "$BACKUP_FILE"),
  "database": "$DB_NAME",
  "backup_type": "full",
  "retention_days": $RETENTION_DAYS,
  "status": "completed"
}
EOF

# Upload to S3 with encryption
log "Uploading to S3..."
aws s3 cp "$BACKUP_FILE" \
    "s3://${S3_BUCKET}/backups/$(date +%Y/%m)/$BACKUP_FILE" \
    --sse AES256 \
    --storage-class STANDARD_IA \
    2>&1 | tee -a "$LOG_FILE"

aws s3 cp "$MANIFEST_FILE" \
    "s3://${S3_BUCKET}/manifests/$(date +%Y/%m)/$MANIFEST_FILE" \
    --sse AES256 \
    2>&1 | tee -a "$LOG_FILE"

# Enable versioning on S3 bucket
aws s3api put-bucket-versioning \
    --bucket "$S3_BUCKET" \
    --versioning-configuration Status=Enabled

# Clean up local files
rm "$BACKUP_FILE" "$MANIFEST_FILE"
log "Backup completed successfully"
```

### Kubernetes CronJob

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: database-backup
  namespace: production
spec:
  # Daily at 2 AM UTC
  schedule: "0 2 * * *"
  jobTemplate:
    spec:
      backoffLimit: 3
      template:
        spec:
          serviceAccountName: backup-service
          containers:
          - name: backup
            image: chainlogistics/backup:latest
            imagePullPolicy: Always
            env:
            - name: DB_HOST
              value: postgres-service
            - name: DB_USER
              valueFrom:
                secretKeyRef:
                  name: postgres-credentials
                  key: username
            - name: DB_PASSWORD
              valueFrom:
                secretKeyRef:
                  name: postgres-credentials
                  key: password
            - name: DB_NAME
              value: chainlogistics
            - name: BACKUP_BUCKET
              value: chain-logistics-backups
            - name: AWS_REGION
              value: us-east-1
            - name: AWS_ACCESS_KEY_ID
              valueFrom:
                secretKeyRef:
                  name: aws-credentials
                  key: access-key-id
            - name: AWS_SECRET_ACCESS_KEY
              valueFrom:
                secretKeyRef:
                  name: aws-credentials
                  key: secret-access-key
            resources:
              requests:
                memory: "512Mi"
                cpu: "500m"
              limits:
                memory: "2Gi"
                cpu: "1000m"
          restartPolicy: OnFailure
```

### Incremental Backups (4-hourly)

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: database-backup-incremental
  namespace: production
spec:
  # Every 4 hours
  schedule: "0 */4 * * *"
  jobTemplate:
    spec:
      template:
        spec:
          serviceAccountName: backup-service
          containers:
          - name: backup
            image: chainlogistics/backup:latest
            imagePullPolicy: Always
            command: ["/usr/local/bin/backup-incremental.sh"]
            # ... env vars same as above
```

## Backup Verification

### Automated Verification

```bash
#!/bin/bash

# Verify backup integrity
verify_backup() {
    BACKUP_FILE=$1
    
    # Check file exists and has content
    if [ ! -f "$BACKUP_FILE" ] || [ ! -s "$BACKUP_FILE" ]; then
        return 1
    fi
    
    # Verify checksum
    if [ -f "${BACKUP_FILE}.sha256" ]; then
        sha256sum -c "${BACKUP_FILE}.sha256" || return 1
    fi
    
    # Test backup integrity (decompress without extraction)
    gzip -t "$BACKUP_FILE" || return 1
    
    return 0
}

# Weekly restoration test
test_restore() {
    BACKUP_FILE=$1
    TEST_DB="chainlogistics_test"
    
    # Create test database
    createdb -T template0 "$TEST_DB"
    
    # Restore from backup
    pg_restore --dbname="$TEST_DB" "$BACKUP_FILE" || {
        dropdb "$TEST_DB"
        return 1
    }
    
    # Verify database integrity
    psql -d "$TEST_DB" -c "SELECT COUNT(*) FROM products;" || {
        dropdb "$TEST_DB"
        return 1
    }
    
    # Cleanup
    dropdb "$TEST_DB"
    return 0
}
```

### Manual Verification Checklist

- ✅ Backup file exists and has appropriate size
- ✅ Checksum verification passes
- ✅ File can be decompressed without errors
- ✅ Test restore to separate database succeeds
- ✅ Restored data matches production snapshots
- ✅ S3 storage shows file with correct size
- ✅ CloudWatch logs show successful backup

## Disaster Recovery Testing

### Monthly DR Test

**Objective**: Verify backup restoration and RTO/RPO targets

```bash
#!/bin/bash

# Monthly disaster recovery test
# Schedule: First Sunday of each month at 3:00 AM

# 1. Create snapshot from latest backup
# 2. Provision temporary database from snapshot
# 3. Run data validation queries
# 4. Measure restoration time
# 5. Generate DR test report
# 6. Cleanup temporary resources

TEST_DATE=$(date +'%Y-%m-%d')
REPORT_FILE="dr-test-${TEST_DATE}.log"

log "Starting monthly DR test..."

# Get latest backup
LATEST_BACKUP=$(aws s3 ls s3://chain-logistics-backups/backups/ \
    --recursive --sort=modified | tail -1 | awk '{print $NF}')

# Measure restore time
START_TIME=$(date +%s)
pg_restore --dbname="chainlogistics_dr_test" "$LATEST_BACKUP"
END_TIME=$(date +%s)
RESTORE_TIME=$((END_TIME - START_TIME))

echo "Restoration Time: ${RESTORE_TIME} seconds" >> "$REPORT_FILE"
echo "RPO Target: 1 hour - $([ $RESTORE_TIME -lt 3600 ] && echo 'PASS' || echo 'FAIL')" >> "$REPORT_FILE"
echo "RTO Target: 4 hours - $([ $RESTORE_TIME -lt 14400 ] && echo 'PASS' || echo 'FAIL')" >> "$REPORT_FILE"

# Data validation
log "Running data validation..."
psql -d chainlogistics_dr_test << SQL >> "$REPORT_FILE"
SELECT COUNT(*) as product_count FROM products;
SELECT COUNT(*) as event_count FROM events;
SELECT MAX(created_at) as latest_event FROM events;
SQL

# Cleanup
dropdb chainlogistics_dr_test

log "DR test completed"
```

### Quarterly Full Disaster Recovery Exercise

- Complete outage simulation
- Full failover to backup systems
- Measure actual RTO/RPO
- Document lessons learned
- Update recovery procedures

## Backup Retention Policy

| Backup Type | Frequency | Retention | Location | Purpose |
|-------------|-----------|-----------|----------|---------|
| Full | Daily | 30 days | Primary + Secondary Region | Current data protection |
| Incremental | Every 4 hours | 30 days | Primary Region | Point-in-time recovery |
| Weekly Archive | Weekly | 12 months | Cold Storage | Compliance & audit |
| Point-in-Time | Continuous | 7 days | Streaming Replication | Immediate failover |

### Retention Cleanup

```sql
-- Remove backups older than 30 days
aws s3 ls s3://chain-logistics-backups/backups/ \
    --recursive \
    --query "Contents[?LastModified<='$(date -d '30 days ago' -Iseconds)']" \
    | xargs -I {} aws s3 rm "s3://chain-logistics-backups/{}"

-- Archive weekly backups to Glacier
aws s3api put-bucket-lifecycle-configuration \
    --bucket chain-logistics-backups \
    --lifecycle-configuration file://lifecycle-policy.json
```

## Backup Monitoring and Alerting

### CloudWatch Metrics

```python
import boto3

cloudwatch = boto3.client('cloudwatch')

def publish_backup_metrics(backup_size, restore_time, success):
    cloudwatch.put_metric_data(
        Namespace='ChainLogistics/Backups',
        MetricData=[
            {
                'MetricName': 'BackupSize',
                'Value': backup_size,
                'Unit': 'Bytes'
            },
            {
                'MetricName': 'RestoreTime',
                'Value': restore_time,
                'Unit': 'Seconds'
            },
            {
                'MetricName': 'BackupStatus',
                'Value': 1 if success else 0
            }
        ]
    )
```

### Alert Conditions

- ⚠️ Backup duration > 60 minutes
- ⚠️ Backup size increase > 50% from baseline
- ❌ Backup failed (retry and alert)
- ❌ S3 upload failed
- ❌ Verification test failed
- ❌ No backup in last 24 hours

## Security Considerations

### Encryption

- **In Transit**: HTTPS/TLS for all S3 transfers
- **At Rest**: AES-256 encryption for S3 objects
- **Database**: SSL connection to PostgreSQL

### Access Control

```yaml
# IAM Policy for Backup Service
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:PutObject",
        "s3:GetObject",
        "s3:ListBucket"
      ],
      "Resource": [
        "arn:aws:s3:::chain-logistics-backups/*",
        "arn:aws:s3:::chain-logistics-backups"
      ]
    }
  ]
}
```

### Credential Management

- Store credentials in Kubernetes Secrets
- Rotate credentials quarterly
- Use IAM roles when possible
- Never commit credentials to git

## Backup Costs

### Estimated Monthly Costs

| Component | Size/Count | Cost |
|-----------|-----------|------|
| S3 Storage (30 daily backups) | 300 GB | ~$7 |
| S3 Storage (12 monthly archives) | 1.2 TB | ~$20 |
| Data Transfer | 300 GB/month | ~$25 |
| Replication | 300 GB | ~$6 |
| **Total** | | ~$58 |

### Cost Optimization

- Use S3 Intelligent-Tiering
- Enable S3 lifecycle policies
- Delete old verification test restores
- Monitor backup growth trends

## References

- [AWS RDS Backups](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/USER_BackupRestore.html)
- [PostgreSQL Backup Best Practices](https://www.postgresql.org/docs/current/backup.html)
- [RPO and RTO Definitions](https://en.wikipedia.org/wiki/Disaster_recovery#RPO_and_RTO)

