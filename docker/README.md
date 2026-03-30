# ChainLogistics Docker Sandbox

This directory contains the Docker Compose setup for running ChainLogistics in a sandbox environment with all dependencies.

## Services

- **PostgreSQL**: Primary database (port 5432)
- **Redis**: Caching and rate limiting (port 6379)
- **Backend**: Rust API server (port 3001)
- **Nginx**: Reverse proxy and load balancer (port 80)
- **Prometheus**: Metrics collection (port 9090)
- **Grafana**: Monitoring dashboards (port 3000)

## Quick Start

1. **Prerequisites**:
   ```bash
   # Install Docker and Docker Compose
   # On macOS:
   brew install docker docker-compose
   
   # On Ubuntu:
   sudo apt update
   sudo apt install docker.io docker-compose
   ```

2. **Start the stack**:
   ```bash
   docker-compose up -d
   ```

3. **Check service health**:
   ```bash
   # Check all services
   docker-compose ps
   
   # Check logs
   docker-compose logs -f backend
   
   # Health check
   curl http://localhost/health
   ```

4. **Access services**:
   - API: http://localhost/api/v1/
   - Swagger UI: http://localhost/swagger-ui/
   - Grafana: http://localhost:3000 (admin/admin)
   - Prometheus: http://localhost:9090

## Environment Variables

Create a `.env` file in the project root:

```bash
# Database
DATABASE_URL=postgres://chainlogistics:password@localhost:5432/chainlogistics

# Server
HOST=0.0.0.0
PORT=3001

# Redis
REDIS_URL=redis://localhost:6379

# Logging
RUST_LOG=info
```

## Development Workflow

### Local Development

1. **Backend only**:
   ```bash
   cd backend
   cargo run
   ```

2. **With database**:
   ```bash
   # Start only dependencies
   docker-compose up -d postgres redis
   
   # Run backend locally
   cd backend
   cargo run
   ```

### Testing

1. **Run tests**:
   ```bash
   cd backend
   cargo test
   ```

2. **Integration tests**:
   ```bash
   # Start test database
   docker-compose up -d postgres
   
   # Run integration tests
   cd backend
   cargo test --test integration_tests
   ```

### Building

1. **Build Docker image**:
   ```bash
   docker-compose build backend
   ```

2. **Production build**:
   ```bash
   cd backend
   cargo build --release
   ```

## API Usage

### Authentication

All API requests (except health checks) require an API key:

```bash
# Get API key (create user first)
curl -X POST http://localhost/api/v1/admin/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com", "password": "password123"}'

# Use API key
curl -X GET http://localhost/api/v1/products \
  -H "Authorization: Bearer YOUR_API_KEY"
```

### Example Requests

```bash
# List products
curl -X GET "http://localhost/api/v1/products?limit=10" \
  -H "Authorization: Bearer YOUR_API_KEY"

# Create product
curl -X POST http://localhost/api/v1/admin/products \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "id": "PROD-001",
    "name": "Test Product",
    "description": "A test product",
    "origin_location": "Factory A",
    "category": "Electronics",
    "tags": ["test", "electronics"],
    "certifications": [],
    "media_hashes": [],
    "custom_fields": {}
  }'

# Get product
curl -X GET http://localhost/api/v1/products/PROD-001 \
  -H "Authorization: Bearer YOUR_API_KEY"
```

## Monitoring

### Grafana Dashboards

Access Grafana at http://localhost:3000:
- Username: admin
- Password: admin

Available dashboards:
- API Metrics
- Database Performance
- System Overview

### Prometheus Metrics

Access Prometheus at http://localhost:9090:
- Built-in metrics: http://localhost:9090/metrics
- Target status: http://localhost:9090/targets

## Troubleshooting

### Common Issues

1. **Database connection failed**:
   ```bash
   # Check PostgreSQL status
   docker-compose logs postgres
   
   # Restart database
   docker-compose restart postgres
   ```

2. **Backend not starting**:
   ```bash
   # Check backend logs
   docker-compose logs backend
   
   # Check environment variables
   docker-compose exec backend env
   ```

3. **Rate limiting errors**:
   - Check API key tier and limits
   - Verify Redis is running
   - Check rate limit headers in response

### Logs

```bash
# View all logs
docker-compose logs -f

# Specific service
docker-compose logs -f backend

# Last 100 lines
docker-compose logs --tail=100 backend
```

### Reset Environment

```bash
# Stop and remove all containers
docker-compose down -v

# Remove all images
docker system prune -a

# Restart fresh
docker-compose up -d
```

## Production Considerations

This sandbox setup is for development only. For production:

1. **Security**:
   - Use strong passwords
   - Enable HTTPS
   - Configure proper firewalls
   - Use secrets management

2. **Performance**:
   - Optimize database configuration
   - Use connection pooling
   - Enable caching
   - Configure proper resource limits

3. **Monitoring**:
   - Set up alerting
   - Configure log aggregation
   - Use proper metrics retention
   - Set up backup strategies

4. **Scaling**:
   - Use container orchestration (Kubernetes)
   - Implement load balancing
   - Configure auto-scaling
   - Use managed databases

## Contributing

When adding new services:

1. Update `docker-compose.yml`
2. Add health checks
3. Update documentation
4. Add monitoring metrics
5. Test with `docker-compose config`
