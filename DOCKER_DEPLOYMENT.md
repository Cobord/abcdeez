# Docker Deployment Guide

This guide explains how to deploy the Research Platform using Docker containers for consistent deployment across environments.

## Architecture Overview

The platform consists of the following containerized services:

- **Web Backend**: Rust-based API server with SQLite/PostgreSQL support
- **Xilem App**: Cross-platform research application with GUI (headless + VNC)
- **Python Client**: Jupyter Lab environment for data analysis
- **PostgreSQL**: Optional database (alternative to SQLite)
- **Redis**: Optional caching layer
- **Nginx**: Reverse proxy for production deployments
- **Prometheus/Grafana**: Optional monitoring stack

## Quick Start

### Prerequisites

- Docker 20.10+
- Docker Compose 2.0+
- Make (optional, for convenience commands)

### Basic Deployment

```bash
# Clone the repository
git clone <repository-url>
cd research-platform

# Build and start all services
make build
make up

# Or using docker-compose directly
docker-compose up -d --build
```

### Development Mode

For development with live code reloading:

```bash
# Start development environment
make dev

# Or manually
docker-compose -f docker-compose.yml -f docker-compose.override.yml --profile dev up -d
```

## Service Access

Once deployed, access services at:

- **Web API**: http://localhost:8080
- **Jupyter Lab**: http://localhost:8888  
- **VNC (Xilem GUI)**: vnc://localhost:5900
- **Prometheus**: http://localhost:9090 (monitoring profile)
- **Grafana**: http://localhost:3000 (monitoring profile)

## Deployment Profiles

Use profiles to enable different service combinations:

### Production Deployment
```bash
make prod
# Includes: nginx reverse proxy, optimized settings
```

### With PostgreSQL Database
```bash
make postgres
# Uses PostgreSQL instead of SQLite
```

### With Redis Caching
```bash
make cache
# Enables Redis for session storage and caching
```

### Full Monitoring Stack
```bash
make monitoring
# Adds Prometheus and Grafana
```

### Complete Production Setup
```bash
make full
# All profiles: postgres + cache + monitoring + nginx
```

## Configuration

### Environment Variables

Key environment variables for production:

```bash
# Web Backend
RUST_LOG=info
DATABASE_URL=sqlite:///app/data/research.db
JWT_SECRET=your-super-secret-jwt-key-change-in-production
BIND_ADDRESS=0.0.0.0:8080

# PostgreSQL (if using postgres profile)
POSTGRES_DB=research
POSTGRES_USER=research_user
POSTGRES_PASSWORD=research_password_change_me

# Xilem App
API_BASE_URL=http://web-backend:8080
RUST_LOG=info

# Python Client  
API_BASE_URL=http://web-backend:8080
JUPYTER_ENABLE_LAB=yes
```

### Volume Mounts

Important data volumes:

- `backend_data`: Backend database and files
- `audio_recordings`: Research audio recordings
- `irb_documents`: Generated IRB compliance documents
- `notebooks`: Jupyter notebooks and analysis
- `postgres_data`: PostgreSQL data (if using postgres)

## Security Considerations

### Production Security

1. **Change Default Passwords**:
   ```bash
   # Update in docker-compose.yml
   - POSTGRES_PASSWORD=secure_password_here
   - GF_SECURITY_ADMIN_PASSWORD=grafana_password_here
   - JWT_SECRET=long_random_jwt_secret_here
   ```

2. **Use HTTPS**: Configure SSL certificates in nginx
3. **Network Security**: Use Docker networks and firewall rules
4. **Regular Updates**: Keep base images updated

### SSL/TLS Setup

For production HTTPS:

1. Obtain SSL certificates
2. Place certificates in `ssl/` directory
3. Uncomment HTTPS server block in `nginx.conf`
4. Update docker-compose to mount certificates

## Monitoring and Logging

### Health Checks

All services include health checks. Monitor with:

```bash
make health
make status
```

### Log Management

View logs:

```bash
# All services
make logs

# Specific services
make logs-backend
make logs-app
make logs-jupyter
```

### Prometheus Metrics

Enable monitoring profile for metrics collection:

- Application metrics: `/api/metrics`
- System metrics: Node exporter (if added)
- Database metrics: Postgres exporter (if using postgres)

## Troubleshooting

### Common Issues

1. **Port Conflicts**: Ensure ports 8080, 8888, 5900 are available
2. **Permission Issues**: Check file ownership in mounted volumes
3. **Memory Issues**: Xilem app may need increased memory limits
4. **VNC Access**: Use VNC client to connect to localhost:5900

### Debug Commands

```bash
# Check service status
docker-compose ps

# View container logs
docker-compose logs -f [service-name]

# Access container shell
docker-compose exec [service-name] /bin/bash

# Check resource usage
docker stats
```

### Recovery Procedures

```bash
# Restart specific service
docker-compose restart [service-name]

# Rebuild and restart
docker-compose up -d --build [service-name]

# Clean restart (loses data)
make clean
make up
```

## Backup and Recovery

### Data Backup

Important data to backup:

```bash
# Create backup directory
mkdir -p backups/$(date +%Y%m%d)

# Backup volumes
docker run --rm -v research-platform_backend_data:/data -v $(pwd)/backups/$(date +%Y%m%d):/backup alpine tar czf /backup/backend_data.tar.gz -C /data .

# Backup PostgreSQL (if using)
docker-compose exec postgres pg_dump -U research_user research > backups/$(date +%Y%m%d)/research_db.sql
```

### Restore Procedures

```bash
# Restore volume data
docker run --rm -v research-platform_backend_data:/data -v $(pwd)/backups/20231201:/backup alpine tar xzf /backup/backend_data.tar.gz -C /data

# Restore PostgreSQL
docker-compose exec -T postgres psql -U research_user research < backups/20231201/research_db.sql
```

## Performance Tuning

### Resource Limits

Add resource limits in docker-compose.yml:

```yaml
services:
  web-backend:
    deploy:
      resources:
        limits:
          memory: 512M
          cpus: '0.5'
        reservations:
          memory: 256M
          cpus: '0.25'
```

### Optimization Tips

1. **Use multi-stage builds** for smaller images
2. **Enable caching** with Redis profile
3. **Use PostgreSQL** for better performance at scale
4. **Configure log rotation** to prevent disk space issues
5. **Monitor resource usage** with monitoring profile

## Scaling

### Horizontal Scaling

Scale services with:

```bash
# Scale backend replicas
docker-compose up -d --scale web-backend=3

# Use load balancer
# Update nginx.conf upstream configuration
```

### Database Scaling

For high-traffic deployments:

1. Use PostgreSQL with connection pooling
2. Consider read replicas
3. Implement caching with Redis
4. Monitor database performance

## Development Workflow

### Local Development

```bash
# Start development environment
make dev

# The development override provides:
# - Live code reloading
# - Debug logging
# - Exposed database ports
# - Volume mounts for source code
```

### Testing Changes

```bash
# Test deployment
make test

# Build specific service
docker-compose build [service-name]

# View changes
docker-compose logs -f [service-name]
```

This Docker deployment provides a robust, scalable foundation for the Research Platform with consistent deployment across development, staging, and production environments.