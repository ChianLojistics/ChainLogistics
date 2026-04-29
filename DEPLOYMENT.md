# ChainLogistics Deployment Guide

This comprehensive guide explains how to deploy the ChainLogistics stack including smart contracts, backend API, and frontend application to production environments.

## Table of Contents

- [Deployment Overview](#deployment-overview)
- [Prerequisites](#prerequisites)
- [Environment Configuration](#environment-configuration)
- [Docker Deployment](#docker-deployment)
- [Manual Deployment](#manual-deployment)
- [Smart Contract Deployment](#smart-contract-deployment)
- [Backend Deployment](#backend-deployment)
- [Frontend Deployment](#frontend-deployment)
- [Production Checklist](#production-checklist)
- [Troubleshooting Guide](#troubleshooting-guide)
- [Monitoring and Maintenance](#monitoring-and-maintenance)

---

## Deployment Overview

ChainLogistics consists of three main components:

1. **Smart Contracts** (Rust/Soroban) - Deployed to Stellar blockchain
2. **Backend API** (Rust/Axum) - REST API server with PostgreSQL and Redis
3. **Frontend** (Next.js/TypeScript) - Web application

**Deployment Options:**
- **Docker Compose** - Recommended for development and small production deployments
- **Manual Deployment** - For custom infrastructure and scaling requirements
- **Cloud Services** - AWS, GCP, Azure deployment guides

---

## Prerequisites

### System Requirements

- **Operating System**: Linux (Ubuntu 20.04+ recommended) or macOS
- **RAM**: Minimum 4GB (8GB recommended)
- **Disk Space**: 20GB free space
- **CPU**: 2+ cores

### Software Requirements

- **Docker**: 20.10+ and Docker Compose 2.0+
- **Rust**: 1.70+ (for manual builds)
- **Node.js**: 18+ (for manual frontend builds)
- **PostgreSQL Client**: 14+ (for manual database setup)
- **Soroban CLI**: Latest version (for smart contract deployment)

### Required Accounts

- **Stellar Account**: With testnet XLM for contract deployment
- **Domain Name**: For production deployment (optional but recommended)
- **SSL Certificate**: For HTTPS (Let's Encrypt recommended)

### Install Docker and Docker Compose

```bash
# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Add your user to docker group
sudo usermod -aG docker $USER

# Install Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/download/v2.20.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# Verify installation
docker --version
docker-compose --version
```

---

## Environment Configuration

### Environment Variables

Create environment files for each component:

#### Backend (`.env`)

```bash
# Database Configuration
DATABASE_URL=postgres://chainlogistics:your_secure_password@localhost:5432/chainlogistics
DATABASE_POOL_SIZE=10

# Redis Configuration
REDIS_URL=redis://localhost:6379

# Server Configuration
HOST=0.0.0.0
PORT=3001
RUST_LOG=info

# JWT Configuration
JWT_SECRET=your_jwt_secret_key_min_32_chars
JWT_EXPIRATION=24h

# Stellar Configuration
STELLAR_NETWORK=testnet
STELLAR_RPC_URL=https://soroban-testnet.stellar.org
CONTRACT_ID=your_deployed_contract_id

# API Rate Limiting
RATE_LIMIT_ENABLED=true
RATE_LIMIT_PER_MINUTE=60

# CORS Configuration
CORS_ALLOWED_ORIGINS=https://yourdomain.com,https://www.yourdomain.com

# SSL/TLS (Production)
SSL_CERT_PATH=/etc/nginx/ssl/cert.pem
SSL_KEY_PATH=/etc/nginx/ssl/key.pem
```

#### Frontend (`.env.local`)

```bash
# API Configuration
NEXT_PUBLIC_API_URL=https://api.yourdomain.com
NEXT_PUBLIC_WS_URL=wss://api.yourdomain.com

# Stellar Configuration
NEXT_PUBLIC_STELLAR_NETWORK=testnet
NEXT_PUBLIC_CONTRACT_ID=your_deployed_contract_id
NEXT_PUBLIC_STELLAR_RPC_URL=https://soroban-testnet.stellar.org

# Feature Flags
NEXT_PUBLIC_ENABLE_ANALYTICS=true
NEXT_PUBLIC_ENABLE_QR_SCANNING=true
```

#### Docker Compose (`.env`)

```bash
# Database
POSTGRES_DB=chainlogistics
POSTGRES_USER=chainlogistics
POSTGRES_PASSWORD=your_secure_password

# Redis
REDIS_PASSWORD=your_redis_password

# Backend
BACKEND_PORT=3001

# Monitoring
GRAFANA_ADMIN_PASSWORD=your_grafana_password
```

### Security Configuration

#### Generate Secure Passwords

```bash
# Generate random passwords
openssl rand -base64 32

# Generate JWT secret
openssl rand -base64 48
```

#### SSL Certificate Setup (Production)

```bash
# Install Certbot
sudo apt-get install certbot python3-certbot-nginx

# Generate certificate
sudo certbot --nginx -d api.yourdomain.com -d yourdomain.com

# Auto-renewal is configured automatically
sudo certbot renew --dry-run
```

---

## Docker Deployment

### Quick Start

```bash
# Clone repository
git clone https://github.com/ChainLojistics/ChainLogistics.git
cd ChainLogistics

# Create environment files
cp backend/.env.example backend/.env
cp frontend/.env.example frontend/.env.local
cp .env.example .env

# Edit environment files with your configuration
nano backend/.env
nano frontend/.env.local
nano .env

# Start all services
docker-compose up -d

# Check service status
docker-compose ps
docker-compose logs -f
```

### Service Management

```bash
# Start services
docker-compose up -d

# Stop services
docker-compose down

# Stop and remove volumes
docker-compose down -v

# Restart specific service
docker-compose restart backend

# View logs
docker-compose logs backend
docker-compose logs -f  # Follow logs

# Scale services
docker-compose up -d --scale backend=3
```

### Database Initialization

```bash
# Run migrations
docker-compose exec backend cargo run --bin migrate

# Create admin user
docker-compose exec backend cargo run --bin create-admin

# Seed initial data (optional)
docker-compose exec backend cargo run --bin seed
```

### Health Checks

```bash
# Check backend health
curl http://localhost:3001/health

# Check database health
curl http://localhost:3001/health/db

# Check all services
docker-compose ps
```

---

## Manual Deployment

### Backend Deployment

#### 1. Install Dependencies

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install PostgreSQL
sudo apt-get install postgresql postgresql-contrib

# Install Redis
sudo apt-get install redis-server

# Install system libraries
sudo apt-get install libssl-dev pkg-config libudev-dev
```

#### 2. Setup Database

```bash
# Create database
sudo -u postgres psql
CREATE DATABASE chainlogistics;
CREATE USER chainlogistics WITH PASSWORD 'your_password';
GRANT ALL PRIVILEGES ON DATABASE chainlogistics TO chainlogistics;
\q

# Run migrations
cd backend
cargo install sqlx-cli
sqlx migrate run --database-url postgres://chainlogistics:your_password@localhost/chainlogistics
```

#### 3. Configure Backend

```bash
cd backend
cp .env.example .env
nano .env
# Edit with your configuration
```

#### 4. Build and Run

```bash
# Build release
cargo build --release

# Run server
./target/release/chainlogistics-backend

# Or run with systemd (recommended for production)
sudo nano /etc/systemd/system/chainlogistics-backend.service
```

#### 5. Systemd Service

```ini
[Unit]
Description=ChainLogistics Backend API
After=network.target postgresql.service redis.service

[Service]
Type=simple
User=chainlogistics
WorkingDirectory=/home/chainlogistics/backend
ExecStart=/home/chainlogistics/backend/target/release/chainlogistics-backend
Environment="DATABASE_URL=postgres://chainlogistics:password@localhost/chainlogistics"
Environment="REDIS_URL=redis://localhost:6379"
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

```bash
# Enable and start service
sudo systemctl enable chainlogistics-backend
sudo systemctl start chainlogistics-backend
sudo systemctl status chainlogistics-backend
```

### Frontend Deployment

#### 1. Install Dependencies

```bash
# Install Node.js
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# Install PM2 (process manager)
sudo npm install -g pm2
```

#### 2. Build Frontend

```bash
cd frontend
npm install
cp .env.example .env.local
nano .env.local
# Edit with your configuration

# Build production bundle
npm run build
```

#### 3. Serve with PM2

```bash
# Install next server globally
npm install -g next

# Start with PM2
pm2 start npm --name "chainlogistics-frontend" -- start

# Save PM2 configuration
pm2 save
pm2 startup
```

#### 4. Nginx Configuration

```nginx
server {
    listen 80;
    server_name yourdomain.com www.yourdomain.com;

    # Redirect to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name yourdomain.com www.yourdomain.com;

    ssl_certificate /etc/letsencrypt/live/yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/yourdomain.com/privkey.pem;

    # Frontend
    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }

    # Backend API
    location /api {
        proxy_pass http://localhost:3001;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # WebSocket support
    location /ws {
        proxy_pass http://localhost:3001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "Upgrade";
        proxy_set_header Host $host;
    }
}
```

```bash
# Enable Nginx configuration
sudo ln -s /etc/nginx/sites-available/chainlogistics /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

---

## Smart Contract Deployment

### Prerequisites

```bash
# Install Soroban CLI
cargo install --locked soroban-cli --features opt

# Add WASM target
rustup target add wasm32-unknown-unknown
```

### Get Testnet Account

```bash
# Generate keypair
soroban keys generate --global testnet-key

# Fund account with friendbot
soroban keys fund testnet-key --network testnet

# Verify balance
soroban keys address testnet-key
```

### Deploy Contract

```bash
cd smart-contract/contracts

# Build WASM
cargo build --target wasm32-unknown-unknown --release

# Deploy to testnet
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/chainlogistics.wasm \
  --source testnet-key \
  --network testnet

# Save the contract ID output
```

### Verify Deployment

```bash
# Test contract invocation
soroban contract invoke \
  --id YOUR_CONTRACT_ID \
  --source testnet-key \
  --network testnet \
  -- ping
```

### Mainnet Deployment

**WARNING:** Mainnet deployment involves real XLM costs. Test thoroughly on testnet first.

```bash
# Use mainnet network
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/chainlogistics.wasm \
  --source mainnet-key \
  --network pubnet

# Update environment variables
echo "CONTRACT_ID=YOUR_MAINNET_CONTRACT_ID" >> backend/.env
echo "NEXT_PUBLIC_CONTRACT_ID=YOUR_MAINNET_CONTRACT_ID" >> frontend/.env.local
echo "STELLAR_NETWORK=mainnet" >> backend/.env
echo "NEXT_PUBLIC_STELLAR_NETWORK=mainnet" >> frontend/.env.local
```

---

## Backend Deployment

### Using Docker

```bash
# Build backend image
docker build -t chainlogistics-backend:latest ./backend

# Run container
docker run -d \
  --name chainlogistics-backend \
  -p 3001:3001 \
  --env-file backend/.env \
  --network chainlogistics-network \
  chainlogistics-backend:latest
```

### Using Systemd

See the [Manual Deployment](#manual-deployment) section above for systemd service configuration.

### Database Backup

```bash
# Manual backup
docker-compose exec postgres pg_dump -U chainlogistics chainlogistics > backup.sql

# Restore backup
docker-compose exec -T postgres psql -U chainlogistics chainlogistics < backup.sql

# Automated backup (cron)
0 2 * * * docker-compose exec postgres pg_dump -U chainlogistics chainlogistics > /backups/chainlogistics_$(date +\%Y\%m\%d).sql
```

---

## Frontend Deployment

### Using Docker

```bash
# Build frontend image
docker build -t chainlogistics-frontend:latest ./frontend

# Run container
docker run -d \
  --name chainlogistics-frontend \
  -p 3000:3000 \
  --env-file frontend/.env.local \
  chainlogistics-frontend:latest
```

### Using Vercel (Recommended)

```bash
# Install Vercel CLI
npm install -g vercel

# Deploy
cd frontend
vercel

# Configure environment variables in Vercel dashboard
# - NEXT_PUBLIC_API_URL
# - NEXT_PUBLIC_CONTRACT_ID
# - NEXT_PUBLIC_STELLAR_NETWORK
```

### Using Netlify

```bash
# Install Netlify CLI
npm install -g netlify-cli

# Build and deploy
cd frontend
netlify deploy --prod
```

---

## Production Checklist

### Pre-Deployment

- [ ] All tests passing (unit, integration, E2E)
- [ ] Code reviewed and approved
- [ ] Security audit completed
- [ ] Environment variables configured
- [ ] SSL certificates obtained
- [ ] Domain DNS configured
- [ ] Database backups enabled
- [ ] Monitoring configured
- [ ] Logging configured
- [ ] Rate limiting enabled
- [ ] CORS configured correctly
- [ ] Smart contract audited (if mainnet)

### Post-Deployment

- [ ] Health checks passing
- [ ] Database migrations successful
- [ ] API endpoints responding
- [ ] Frontend loading correctly
- [ ] Smart contract accessible
- [ ] Wallet connection working
- [ ] Event tracking functional
- [ ] QR generation working
- [ ] Timeline displaying correctly
- [ ] Analytics collecting data
- [ ] Error monitoring active
- [ ] Backup system verified
- [ ] Rollback plan documented

### Security Checklist

- [ ] No secrets in code
- [ ] Environment variables secured
- [ ] Database credentials strong
- [ ] API keys rotated
- [ ] HTTPS enforced
- [ ] Security headers configured
- [ ] Rate limiting active
- [ ] Input validation enabled
- [ ] SQL injection prevention
- [ ] XSS prevention
- [ ] CSRF protection
- [ ] Authentication working
- [ ] Authorization working

---

## Troubleshooting Guide

### Database Issues

#### Connection Refused

```bash
# Check PostgreSQL status
sudo systemctl status postgresql

# Check if running
sudo systemctl start postgresql

# Check connection
psql -U chainlogistics -d chainlogistics -h localhost
```

#### Migration Failures

```bash
# Check migration status
cd backend
sqlx migrate info --database-url postgres://chainlogistics:password@localhost/chainlogistics

# Revert last migration
sqlx migrate revert --database-url postgres://chainlogistics:password@localhost/chainlogistics

# Force re-run migrations
sqlx migrate run --force --database-url postgres://chainlogistics:password@localhost/chainlogistics
```

#### Slow Queries

```bash
# Enable slow query log
sudo nano /etc/postgresql/14/main/postgresql.conf
# Add: log_min_duration_statement = 1000

# Restart PostgreSQL
sudo systemctl restart postgresql

# Analyze slow queries
cd backend
cargo install pgbench
pgbench -h localhost -U chainlogistics -d chainlogistics
```

### Backend Issues

#### Server Won't Start

```bash
# Check logs
journalctl -u chainlogistics-backend -f

# Check port availability
netstat -tulpn | grep 3001

# Kill process using port
sudo fuser -k 3001/tcp

# Check environment variables
cat backend/.env
```

#### Out of Memory

```bash
# Check memory usage
free -h

# Check process memory
ps aux | grep chainlogistics

# Increase swap
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

#### Database Connection Pool Exhausted

```bash
# Check active connections
docker-compose exec postgres psql -U chainlogistics -d chainlogistics -c "SELECT count(*) FROM pg_stat_activity;"

# Kill idle connections
docker-compose exec postgres psql -U chainlogistics -d chainlogistics -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE state = 'idle';"

# Increase pool size in .env
DATABASE_POOL_SIZE=20
```

### Frontend Issues

#### Build Failures

```bash
# Clear cache
cd frontend
rm -rf .next node_modules
npm install

# Check Node.js version
node --version  # Should be 18+

# Update dependencies
npm update
```

#### Environment Variables Not Loading

```bash
# Verify .env.local exists
ls -la frontend/.env.local

# Check file permissions
chmod 644 frontend/.env.local

# Restart dev server
npm run dev
```

#### API Connection Errors

```bash
# Check backend is running
curl http://localhost:3001/health

# Check CORS configuration
# Ensure NEXT_PUBLIC_API_URL matches backend URL

# Check network
curl -v https://api.yourdomain.com/health
```

### Docker Issues

#### Container Won't Start

```bash
# Check container logs
docker-compose logs backend

# Check container status
docker-compose ps

# Rebuild container
docker-compose up -d --build backend

# Remove and recreate
docker-compose down -v
docker-compose up -d
```

#### Volume Mount Issues

```bash
# Check volume permissions
docker-compose exec backend ls -la /backups

# Fix permissions
sudo chown -R $USER:$USER ./backups

# Check disk space
df -h
```

#### Network Issues

```bash
# Check network
docker network ls
docker network inspect chainlogistics-network

# Recreate network
docker network rm chainlogistics-network
docker-compose up -d
```

### Smart Contract Issues

#### Deployment Failed

```bash
# Check account balance
soroban keys address testnet-key
soroban keys fund testnet-key --network testnet

# Check network connectivity
curl https://soroban-testnet.stellar.org

# Verify WASM file
ls -lh target/wasm32-unknown-unknown/release/chainlogistics.wasm
```

#### Transaction Failed

```bash
# Check transaction status
soroban contract tx-status YOUR_TRANSACTION_ID --network testnet

# Check gas fees
soroban contract fee \
  --wasm target/wasm32-unknown-unknown/release/chainlogistics.wasm \
  --source testnet-key \
  --network testnet

# Retry with higher fee
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/chainlogistics.wasm \
  --source testnet-key \
  --network testnet \
  --fee 100000
```

### Performance Issues

#### Slow API Response

```bash
# Enable profiling
RUST_LOG=debug cargo run

# Check database query performance
docker-compose exec postgres psql -U chainlogistics -d chainlogistics -c "EXPLAIN ANALYZE SELECT * FROM products;"

# Check Redis cache hit rate
docker-compose exec redis redis-cli INFO stats
```

#### High Memory Usage

```bash
# Check Rust memory usage
cargo install valgrind
valgrind --leak-check=full ./target/release/chainlogistics-backend

# Enable memory profiling
export RUST_LOG=info,chainlogistics=debug
```

### SSL/TLS Issues

#### Certificate Errors

```bash
# Check certificate expiration
sudo certbot certificates

# Renew certificate
sudo certbot renew

# Force renewal
sudo certbot renew --force-renewal

# Check Nginx configuration
sudo nginx -t
```

#### Mixed Content Errors

```bash
# Ensure all resources use HTTPS
# Check browser console for mixed content warnings

# Update Nginx configuration
# Add HSTS header
add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
```

---

## Monitoring and Maintenance

### Health Checks

```bash
# Backend health
curl http://localhost:3001/health

# Database health
curl http://localhost:3001/health/db

# Overall system status
docker-compose ps
```

### Log Management

```bash
# View backend logs
docker-compose logs -f backend

# View all logs
docker-compose logs -f

# Rotate logs (logrotate)
sudo nano /etc/logrotate.d/chainlogistics
```

### Database Maintenance

```bash
# Weekly vacuum
docker-compose exec postgres psql -U chainlogistics -d chainlogistics -c "VACUUM ANALYZE;"

# Reindex
docker-compose exec postgres psql -U chainlogistics -d chainlogistics -c "REINDEX DATABASE chainlogistics;"

# Backup
docker-compose exec postgres pg_dump -U chainlogistics chainlogistics > backup.sql
```

### Updates and Upgrades

```bash
# Pull latest changes
git pull origin main

# Rebuild and restart
docker-compose down
docker-compose pull
docker-compose up -d --build

# Run migrations
docker-compose exec backend cargo run --bin migrate
```

### Backup Strategy

```bash
# Automated backup script
#!/bin/bash
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backups"
mkdir -p $BACKUP_DIR

# Database backup
docker-compose exec postgres pg_dump -U chainlogistics chainlogistics > $BACKUP_DIR/db_$DATE.sql

# Config backup
tar -czf $BACKUP_DIR/config_$DATE.tar.gz backend/.env frontend/.env.local

# Keep last 7 days
find $BACKUP_DIR -name "*.sql" -mtime +7 -delete
find $BACKUP_DIR -name "*.tar.gz" -mtime +7 -delete
```

### Security Updates

```bash
# Update system packages
sudo apt-get update && sudo apt-get upgrade -y

# Update Docker images
docker-compose pull

# Scan for vulnerabilities
docker scan chainlogistics-backend:latest
```

---

## Support and Resources

- **Documentation**: https://docs.chainlogistics.io
- **GitHub Issues**: https://github.com/ChainLojistics/ChainLogistics/issues
- **Discord Community**: https://discord.gg/chainlogistics
- **Email Support**: support@chainlogistics.io

---

## License

This deployment guide is part of the ChainLogistics project. See LICENSE for details.
