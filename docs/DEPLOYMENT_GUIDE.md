# ChainLogistics Deployment Guide

## Overview

This comprehensive guide covers deploying ChainLogistics to various environments including development, staging, and production. The platform consists of multiple components that need to be deployed and configured correctly.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Architecture Overview](#architecture-overview)
3. [Environment Setup](#environment-setup)
4. [Database Deployment](#database-deployment)
5. [Backend Deployment](#backend-deployment)
6. [Frontend Deployment](#frontend-deployment)
7. [Smart Contract Deployment](#smart-contract-deployment)
8. [Infrastructure Setup](#infrastructure-setup)
9. [Monitoring & Logging](#monitoring--logging)
10. [Security Configuration](#security-configuration)
11. [Scaling & Performance](#scaling--performance)
12. [Troubleshooting](#troubleshooting)

## Prerequisites

### Required Software

- Docker 24.0+ and Docker Compose 2.0+
- Kubernetes 1.28+ (for production)
- PostgreSQL 14+
- Redis 6+
- Node.js 18+ and npm/yarn
- Rust 1.70+ and Cargo
- Soroban CLI
- Git

### Required Accounts

- Stellar account with testnet/mainnet XLM
- Cloud provider account (AWS/GCP/Azure)
- Domain name and SSL certificates
- Container registry access (Docker Hub/ECR/GCR)

### System Requirements

**Development Environment:**

- CPU: 4 cores
- RAM: 8GB
- Storage: 20GB

**Production Environment:**

- CPU: 8+ cores
- RAM: 16GB+
- Storage: 100GB+ SSD
- Network: 1Gbps+

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Load Balancer / CDN                     │
│                    (Nginx / CloudFlare)                     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      Frontend Cluster                       │
│              (Next.js on Vercel / Docker)                   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      Backend Cluster                        │
│                  (Rust/Axum on Kubernetes)                  │
└─────────────────────────────────────────────────────────────┘
                              │
                ┌─────────────┴─────────────┐
                ▼                           ▼
┌───────────────────────────┐   ┌───────────────────────────┐
│   PostgreSQL Cluster      │   │    Redis Cluster          │
│   (Primary + Replicas)    │   │    (Cache Layer)          │
└───────────────────────────┘   └───────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Stellar Network                          │
│                  (Soroban Smart Contracts)                  │
└─────────────────────────────────────────────────────────────┘
```

## Environment Setup

### 1. Clone Repository

```bash
git clone https://github.com/ChainLojistics/ChainLogistics.git
cd ChainLogistics
```

### 2. Environment Variables

Create environment files for each component:

**Backend (.env)**

```bash
# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=3001
RUST_LOG=info

# Database
DATABASE_URL=postgresql://chainlog:password@localhost:5432/chainlogistics
DATABASE_MAX_CONNECTIONS=20
DATABASE_MIN_CONNECTIONS=5

# Redis
REDIS_URL=redis://localhost:6379
REDIS_POOL_SIZE=10

# Stellar/Soroban
STELLAR_NETWORK=testnet
STELLAR_RPC_URL=https://soroban-testnet.stellar.org
CONTRACT_ID=YOUR_CONTRACT_ID
STELLAR_SECRET_KEY=YOUR_SECRET_KEY

# JWT
JWT_SECRET=your-super-secret-jwt-key-change-this
JWT_EXPIRATION=86400

# API Rate Limiting
RATE_LIMIT_PER_MINUTE=60
RATE_LIMIT_BURST=10

# Monitoring
PROMETHEUS_PORT=9090
GRAFANA_PORT=3000

# External Services
IPFS_GATEWAY=https://ipfs.io/ipfs/
WEBHOOK_TIMEOUT_SECONDS=30
```

**Frontend (.env.local)**

```bash
# API Configuration
NEXT_PUBLIC_API_URL=http://localhost:3001
NEXT_PUBLIC_WS_URL=ws://localhost:3001/ws

# Stellar Configuration
NEXT_PUBLIC_STELLAR_NETWORK=testnet
NEXT_PUBLIC_CONTRACT_ID=YOUR_CONTRACT_ID
NEXT_PUBLIC_STELLAR_RPC_URL=https://soroban-testnet.stellar.org

# Feature Flags
NEXT_PUBLIC_ENABLE_ANALYTICS=true
NEXT_PUBLIC_ENABLE_WEBSOCKETS=true
NEXT_PUBLIC_ENABLE_DIGITAL_TWIN=true

# Analytics
NEXT_PUBLIC_GA_ID=G-XXXXXXXXXX
```

## Database Deployment

### Local Development

```bash
# Start PostgreSQL with Docker
docker run -d \
  --name chainlog-postgres \
  -e POSTGRES_USER=chainlog \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=chainlogistics \
  -p 5432:5432 \
  -v pgdata:/var/lib/postgresql/data \
  postgres:14-alpine

# Run migrations
cd backend
sqlx migrate run
```

### Production (Managed Service)

**AWS RDS:**

```bash
# Create RDS instance
aws rds create-db-instance \
  --db-instance-identifier chainlog-prod \
  --db-instance-class db.t3.medium \
  --engine postgres \
  --engine-version 14.9 \
  --master-username chainlog \
  --master-user-password YOUR_PASSWORD \
  --allocated-storage 100 \
  --storage-type gp3 \
  --backup-retention-period 7 \
  --multi-az \
  --publicly-accessible false
```

**Google Cloud SQL:**

```bash
gcloud sql instances create chainlog-prod \
  --database-version=POSTGRES_14 \
  --tier=db-custom-4-16384 \
  --region=us-central1 \
  --backup \
  --availability-type=REGIONAL
```

### Database Configuration

**Optimize PostgreSQL:**

```sql
-- postgresql.conf optimizations
shared_buffers = 4GB
effective_cache_size = 12GB
maintenance_work_mem = 1GB
checkpoint_completion_target = 0.9
wal_buffers = 16MB
default_statistics_target = 100
random_page_cost = 1.1
effective_io_concurrency = 200
work_mem = 10MB
min_wal_size = 1GB
max_wal_size = 4GB
max_worker_processes = 8
max_parallel_workers_per_gather = 4
max_parallel_workers = 8
```

**Create Indexes:**

```sql
-- Performance indexes
CREATE INDEX CONCURRENTLY idx_products_owner ON products(owner_address);
CREATE INDEX CONCURRENTLY idx_products_category ON products(category);
CREATE INDEX CONCURRENTLY idx_products_active ON products(is_active) WHERE is_active = true;
CREATE INDEX CONCURRENTLY idx_events_product_time ON tracking_events(product_id, timestamp DESC);
CREATE INDEX CONCURRENTLY idx_events_type ON tracking_events(event_type);
CREATE INDEX CONCURRENTLY idx_users_email ON users(email);
CREATE INDEX CONCURRENTLY idx_api_keys_user ON api_keys(user_id) WHERE is_active = true;
```

## Backend Deployment

### Docker Build

```bash
cd backend

# Build optimized Docker image
docker build -t chainlogistics/backend:latest -f Dockerfile .

# Multi-stage build for smaller image
docker build \
  --target production \
  --build-arg RUST_VERSION=1.70 \
  -t chainlogistics/backend:v1.0.0 .
```

**Optimized Dockerfile:**

```dockerfile
# Build stage
FROM rust:1.70-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Build dependencies (cached layer)
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 chainlog

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/chainlojistic-backend /app/backend
COPY --from=builder /app/migrations /app/migrations

# Set ownership
RUN chown -R chainlog:chainlog /app

USER chainlog

EXPOSE 3001

CMD ["./backend"]
```

### Kubernetes Deployment

**Deployment manifest (k8s/backend-deployment.yaml):**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: chainlog-backend
  namespace: chainlogistics
spec:
  replicas: 3
  selector:
    matchLabels:
      app: chainlog-backend
  template:
    metadata:
      labels:
        app: chainlog-backend
    spec:
      containers:
        - name: backend
          image: chainlogistics/backend:v1.0.0
          ports:
            - containerPort: 3001
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: chainlog-secrets
                  key: database-url
            - name: REDIS_URL
              valueFrom:
                secretKeyRef:
                  name: chainlog-secrets
                  key: redis-url
            - name: JWT_SECRET
              valueFrom:
                secretKeyRef:
                  name: chainlog-secrets
                  key: jwt-secret
          resources:
            requests:
              memory: "512Mi"
              cpu: "500m"
            limits:
              memory: "2Gi"
              cpu: "2000m"
          livenessProbe:
            httpGet:
              path: /health
              port: 3001
            initialDelaySeconds: 30
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /health/ready
              port: 3001
            initialDelaySeconds: 10
            periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: chainlog-backend-service
  namespace: chainlogistics
spec:
  selector:
    app: chainlog-backend
  ports:
    - protocol: TCP
      port: 80
      targetPort: 3001
  type: LoadBalancer
```

**Deploy to Kubernetes:**

```bash
# Create namespace
kubectl create namespace chainlogistics

# Create secrets
kubectl create secret generic chainlog-secrets \
  --from-literal=database-url="postgresql://..." \
  --from-literal=redis-url="redis://..." \
  --from-literal=jwt-secret="..." \
  -n chainlogistics

# Apply deployments
kubectl apply -f k8s/backend-deployment.yaml
kubectl apply -f k8s/backend-service.yaml
kubectl apply -f k8s/backend-hpa.yaml

# Verify deployment
kubectl get pods -n chainlogistics
kubectl logs -f deployment/chainlog-backend -n chainlogistics
```

### Horizontal Pod Autoscaler

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: chainlog-backend-hpa
  namespace: chainlogistics
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: chainlog-backend
  minReplicas: 3
  maxReplicas: 10
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 80
```

## Frontend Deployment

### Vercel Deployment (Recommended)

```bash
# Install Vercel CLI
npm install -g vercel

# Deploy to Vercel
cd frontend
vercel --prod

# Configure environment variables in Vercel dashboard
# Settings > Environment Variables
```

### Docker Deployment

```bash
cd frontend

# Build Docker image
docker build -t chainlogistics/frontend:latest .

# Run container
docker run -d \
  -p 3000:3000 \
  --name chainlog-frontend \
  -e NEXT_PUBLIC_API_URL=https://api.chainlogistics.com \
  chainlogistics/frontend:latest
```

**Frontend Dockerfile:**

```dockerfile
FROM node:18-alpine AS builder

WORKDIR /app

# Copy package files
COPY package*.json ./
RUN npm ci

# Copy source
COPY . .

# Build application
RUN npm run build

# Production stage
FROM node:18-alpine

WORKDIR /app

# Copy built assets
COPY --from=builder /app/.next ./.next
COPY --from=builder /app/public ./public
COPY --from=builder /app/package*.json ./
COPY --from=builder /app/node_modules ./node_modules

EXPOSE 3000

CMD ["npm", "start"]
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: chainlog-frontend
  namespace: chainlogistics
spec:
  replicas: 2
  selector:
    matchLabels:
      app: chainlog-frontend
  template:
    metadata:
      labels:
        app: chainlog-frontend
    spec:
      containers:
        - name: frontend
          image: chainlogistics/frontend:v1.0.0
          ports:
            - containerPort: 3000
          env:
            - name: NEXT_PUBLIC_API_URL
              value: "https://api.chainlogistics.com"
          resources:
            requests:
              memory: "256Mi"
              cpu: "250m"
            limits:
              memory: "1Gi"
              cpu: "1000m"
```

## Smart Contract Deployment

### Testnet Deployment

```bash
cd smart-contract

# Build contract
cargo build --target wasm32-unknown-unknown --release

# Deploy to testnet
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/chainlogistics.wasm \
  --source-account YOUR_SECRET_KEY \
  --network testnet

# Save contract ID
echo "CONTRACT_ID=<output-contract-id>" >> ../.env
```

### Mainnet Deployment

```bash
# Build optimized contract
cargo build --target wasm32-unknown-unknown --release --features mainnet

# Optimize WASM
soroban contract optimize \
  --wasm target/wasm32-unknown-unknown/release/chainlogistics.wasm

# Deploy to mainnet
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/chainlogistics_optimized.wasm \
  --source-account YOUR_MAINNET_SECRET_KEY \
  --network mainnet

# Verify deployment
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source-account YOUR_SECRET_KEY \
  --network mainnet \
  -- ping
```

## Infrastructure Setup

### Docker Compose (Development)

```yaml
version: "3.8"

services:
  postgres:
    image: postgres:14-alpine
    environment:
      POSTGRES_USER: chainlog
      POSTGRES_PASSWORD: password
      POSTGRES_DB: chainlogistics
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U chainlog"]
      interval: 10s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redisdata:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 3s
      retries: 5

  backend:
    build:
      context: ./backend
      dockerfile: Dockerfile
    ports:
      - "3001:3001"
    environment:
      DATABASE_URL: postgresql://chainlog:password@postgres:5432/chainlogistics
      REDIS_URL: redis://redis:6379
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy

  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile
    ports:
      - "3000:3000"
    environment:
      NEXT_PUBLIC_API_URL: http://localhost:3001
    depends_on:
      - backend

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./docker/prometheus/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3002:3000"
    environment:
      GF_SECURITY_ADMIN_PASSWORD: admin
    volumes:
      - ./docker/grafana/datasources:/etc/grafana/provisioning/datasources
      - grafana-data:/var/lib/grafana

volumes:
  pgdata:
  redisdata:
  prometheus-data:
  grafana-data:
```

### Nginx Configuration

```nginx
# /etc/nginx/sites-available/chainlogistics

upstream backend {
    least_conn;
    server backend-1:3001 max_fails=3 fail_timeout=30s;
    server backend-2:3001 max_fails=3 fail_timeout=30s;
    server backend-3:3001 max_fails=3 fail_timeout=30s;
}

upstream frontend {
    server frontend-1:3000;
    server frontend-2:3000;
}

# Redirect HTTP to HTTPS
server {
    listen 80;
    server_name chainlogistics.com www.chainlogistics.com;
    return 301 https://$server_name$request_uri;
}

# HTTPS configuration
server {
    listen 443 ssl http2;
    server_name chainlogistics.com www.chainlogistics.com;

    # SSL certificates
    ssl_certificate /etc/letsencrypt/live/chainlogistics.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/chainlogistics.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # API routes
    location /api/ {
        proxy_pass http://backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;

        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # WebSocket routes
    location /ws {
        proxy_pass http://backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # Frontend routes
    location / {
        proxy_pass http://frontend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }

    # Static assets caching
    location ~* \.(jpg|jpeg|png|gif|ico|css|js|svg|woff|woff2|ttf|eot)$ {
        proxy_pass http://frontend;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

## Monitoring & Logging

### Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: "chainlog-backend"
    static_configs:
      - targets: ["backend:3001"]
    metrics_path: "/metrics"

  - job_name: "postgres"
    static_configs:
      - targets: ["postgres-exporter:9187"]

  - job_name: "redis"
    static_configs:
      - targets: ["redis-exporter:9121"]

  - job_name: "node"
    static_configs:
      - targets: ["node-exporter:9100"]
```

### Grafana Dashboards

Import pre-built dashboards:

- Node Exporter Full (ID: 1860)
- PostgreSQL Database (ID: 9628)
- Redis Dashboard (ID: 11835)
- Kubernetes Cluster Monitoring (ID: 7249)

### Logging Stack

```yaml
# ELK Stack deployment
version: "3.8"

services:
  elasticsearch:
    image: docker.elastic.co/elasticsearch/elasticsearch:8.10.0
    environment:
      - discovery.type=single-node
      - "ES_JAVA_OPTS=-Xms512m -Xmx512m"
    ports:
      - "9200:9200"
    volumes:
      - esdata:/usr/share/elasticsearch/data

  logstash:
    image: docker.elastic.co/logstash/logstash:8.10.0
    volumes:
      - ./logstash/pipeline:/usr/share/logstash/pipeline
    ports:
      - "5000:5000"
    depends_on:
      - elasticsearch

  kibana:
    image: docker.elastic.co/kibana/kibana:8.10.0
    ports:
      - "5601:5601"
    environment:
      ELASTICSEARCH_URL: http://elasticsearch:9200
    depends_on:
      - elasticsearch

volumes:
  esdata:
```

## Security Configuration

### SSL/TLS Setup

```bash
# Install Certbot
sudo apt-get install certbot python3-certbot-nginx

# Obtain SSL certificate
sudo certbot --nginx -d chainlogistics.com -d www.chainlogistics.com

# Auto-renewal
sudo certbot renew --dry-run
```

### Firewall Configuration

```bash
# UFW (Ubuntu)
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 80/tcp    # HTTP
sudo ufw allow 443/tcp   # HTTPS
sudo ufw enable

# AWS Security Group
aws ec2 authorize-security-group-ingress \
  --group-id sg-xxxxx \
  --protocol tcp \
  --port 443 \
  --cidr 0.0.0.0/0
```

### Secrets Management

**Using Kubernetes Secrets:**

```bash
# Create secret from file
kubectl create secret generic chainlog-secrets \
  --from-file=database-url=./secrets/db-url.txt \
  --from-file=jwt-secret=./secrets/jwt.txt \
  -n chainlogistics

# Create secret from literal
kubectl create secret generic api-keys \
  --from-literal=stellar-key='SXXXXX...' \
  -n chainlogistics
```

**Using HashiCorp Vault:**

```bash
# Store secrets
vault kv put secret/chainlog/prod \
  database_url="postgresql://..." \
  jwt_secret="..." \
  stellar_key="..."

# Retrieve secrets
vault kv get secret/chainlog/prod
```

## Scaling & Performance

### Database Scaling

**Read Replicas:**

```sql
-- Configure streaming replication
-- On primary server
ALTER SYSTEM SET wal_level = replica;
ALTER SYSTEM SET max_wal_senders = 10;
ALTER SYSTEM SET wal_keep_size = '1GB';

-- Create replication user
CREATE USER replicator WITH REPLICATION ENCRYPTED PASSWORD 'password';
```

**Connection Pooling:**

```rust
// PgBouncer configuration
[databases]
chainlogistics = host=localhost port=5432 dbname=chainlogistics

[pgbouncer]
pool_mode = transaction
max_client_conn = 1000
default_pool_size = 25
```

### Redis Clustering

```bash
# Create Redis cluster
redis-cli --cluster create \
  127.0.0.1:7000 127.0.0.1:7001 127.0.0.1:7002 \
  127.0.0.1:7003 127.0.0.1:7004 127.0.0.1:7005 \
  --cluster-replicas 1
```

### CDN Configuration

**CloudFlare Setup:**

```bash
# Configure caching rules
- Cache Level: Standard
- Browser Cache TTL: 4 hours
- Always Online: On
- Auto Minify: JS, CSS, HTML
```

## Troubleshooting

### Common Issues

**Database Connection Errors:**

```bash
# Check PostgreSQL status
sudo systemctl status postgresql

# Check connections
psql -U chainlog -d chainlogistics -c "SELECT count(*) FROM pg_stat_activity;"

# Reset connections
SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = 'chainlogistics';
```

**High Memory Usage:**

```bash
# Check memory usage
free -h
docker stats

# Restart services
kubectl rollout restart deployment/chainlog-backend -n chainlogistics
```

**Slow API Responses:**

```bash
# Check slow queries
SELECT query, mean_exec_time, calls
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;

# Analyze query plans
EXPLAIN ANALYZE SELECT * FROM products WHERE category = 'coffee';
```

### Health Checks

```bash
# Backend health
curl http://localhost:3001/health

# Database health
pg_isready -h localhost -p 5432

# Redis health
redis-cli ping

# Kubernetes health
kubectl get pods -n chainlogistics
kubectl describe pod <pod-name> -n chainlogistics
```

## Backup & Recovery

### Database Backups

```bash
# Automated backup script
#!/bin/bash
BACKUP_DIR="/backups/postgres"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/chainlog_$TIMESTAMP.sql.gz"

pg_dump -U chainlog chainlogistics | gzip > $BACKUP_FILE

# Keep only last 7 days
find $BACKUP_DIR -name "*.sql.gz" -mtime +7 -delete
```

### Disaster Recovery

```bash
# Restore from backup
gunzip < chainlog_20240315_120000.sql.gz | psql -U chainlog chainlogistics

# Point-in-time recovery
pg_basebackup -h primary -D /var/lib/postgresql/14/main -U replicator -P -v
```

## Maintenance

### Rolling Updates

```bash
# Update backend
kubectl set image deployment/chainlog-backend \
  backend=chainlogistics/backend:v1.1.0 \
  -n chainlogistics

# Monitor rollout
kubectl rollout status deployment/chainlog-backend -n chainlogistics

# Rollback if needed
kubectl rollout undo deployment/chainlog-backend -n chainlogistics
```

### Database Migrations

```bash
# Run migrations
cd backend
sqlx migrate run

# Rollback migration
sqlx migrate revert
```

---

## Support & Resources

- Documentation: https://docs.chainlogistics.com
- GitHub Issues: https://github.com/ChainLojistics/ChainLogistics/issues
- Discord: https://discord.gg/chainlogistics
- Email: devops@chainlogistics.com

## License

MIT License - See LICENSE file for details
