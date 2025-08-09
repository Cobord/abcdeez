# OAuth Authentication Deployment Guide

This guide provides everything needed to deploy the Xilem learning app backend with Apple Sign In and GitHub OAuth authentication.

## 🚀 Quick Deploy Checklist

✅ **Ready for Production**
- [x] Apple Sign In JWT verification implemented
- [x] GitHub OAuth2 flow implemented  
- [x] Database migrations ready
- [x] Security measures implemented
- [x] Comprehensive test suite created
- [x] App Store compliance implemented
- [x] Configuration management ready

## 🔧 Environment Setup

### Required Environment Variables

```bash
# Database Configuration
DATABASE_URL=sqlite://./production.db

# JWT Security
JWT_SECRET=your-super-secure-jwt-secret-minimum-32-characters

# Server Configuration
PORT=8080
RUST_LOG=info

# Apple Sign In Configuration
APPLE_CLIENT_ID=your.app.bundle.identifier
APPLE_TEAM_ID=YOUR_TEAM_ID
APPLE_KEY_ID=YOUR_KEY_ID
APPLE_PRIVATE_KEY_PATH=/path/to/AuthKey_YOUR_KEY_ID.p8
APPLE_REDIRECT_URI=https://abcdeez.fg-goose.online/auth/callback

# GitHub OAuth Configuration
GITHUB_CLIENT_ID=your_github_client_id
GITHUB_CLIENT_SECRET=your_github_client_secret
GITHUB_REDIRECT_URI=https://abcdeez.fg-goose.online/auth/github/callback
```

### Production Environment Requirements

```bash
# Minimum system requirements
- CPU: 2 cores
- RAM: 2GB
- Storage: 10GB SSD
- OS: Linux (Ubuntu 20.04+ recommended)

# Network requirements
- HTTPS certificate (Let's Encrypt recommended)
- Domain name for OAuth redirects
- Open ports: 80, 443
```

## 🏗️ Infrastructure Setup

### Docker Deployment (Recommended)

```dockerfile
# Dockerfile for production deployment
FROM rust:1.75-slim as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/web-backend /usr/local/bin/app
COPY migrations ./migrations

EXPOSE 8080

CMD ["app"]
```

```yaml
# docker-compose.yml for production
version: '3.8'
services:
  web-backend:
    build: .
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=sqlite:///app/data/production.db
      - JWT_SECRET=${JWT_SECRET}
      - APPLE_CLIENT_ID=${APPLE_CLIENT_ID}
      - APPLE_TEAM_ID=${APPLE_TEAM_ID}
      - APPLE_KEY_ID=${APPLE_KEY_ID}
      - APPLE_PRIVATE_KEY_PATH=/app/keys/AuthKey.p8
      - APPLE_REDIRECT_URI=${APPLE_REDIRECT_URI}
      - GITHUB_CLIENT_ID=${GITHUB_CLIENT_ID}
      - GITHUB_CLIENT_SECRET=${GITHUB_CLIENT_SECRET}
      - GITHUB_REDIRECT_URI=${GITHUB_REDIRECT_URI}
      - RUST_LOG=info
    volumes:
      - ./data:/app/data
      - ./keys:/app/keys:ro
    restart: unless-stopped

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/nginx/ssl
    depends_on:
      - web-backend
    restart: unless-stopped
```

### Nginx Configuration

```nginx
# nginx.conf for reverse proxy and SSL termination
events {
    worker_connections 1024;
}

http {
    upstream backend {
        server web-backend:8080;
    }

    # HTTP redirect to HTTPS
    server {
        listen 80;
        server_name abcdeez.fg-goose.online;
        return 301 https://$server_name$request_uri;
    }

    # HTTPS server
    server {
        listen 443 ssl http2;
        server_name abcdeez.fg-goose.online;

        ssl_certificate /etc/nginx/ssl/cert.pem;
        ssl_certificate_key /etc/nginx/ssl/key.pem;
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers HIGH:!aNULL:!MD5;

        # Security headers
        add_header X-Frame-Options DENY;
        add_header X-Content-Type-Options nosniff;
        add_header X-XSS-Protection "1; mode=block";
        add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;

        location / {
            proxy_pass http://backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            
            # OAuth redirect support
            proxy_redirect off;
        }

        # Health check endpoint
        location /health {
            proxy_pass http://backend/health;
            access_log off;
        }
    }
}
```

## 🔐 Security Configuration

### Apple Developer Account Setup

1. **Create App ID**
   ```
   - Go to Apple Developer Console
   - Create new App ID with Sign In with Apple capability
   - Note the Bundle Identifier (use as APPLE_CLIENT_ID)
   ```

2. **Generate Sign In with Apple Key**
   ```
   - Go to Keys section in Apple Developer Console
   - Create new key with Sign In with Apple capability
   - Download AuthKey_XXXXXXXXXX.p8 file
   - Note the Key ID (use as APPLE_KEY_ID)
   ```

3. **Configure Service ID** (for web authentication)
   ```
   - Create Services ID in Apple Developer Console
   - Configure Return URLs with your domain
   - Verify domain ownership
   ```

### GitHub OAuth Application Setup

1. **Create OAuth App**
   ```
   - Go to GitHub Settings > Developer settings > OAuth Apps
   - Create new OAuth App
   - Set Authorization callback URL: https://abcdeez.fg-goose.online/auth/github/callback
   - Note Client ID and Client Secret
   ```

### SSL Certificate Setup

```bash
# Using Let's Encrypt (recommended)
sudo apt update
sudo apt install certbot python3-certbot-nginx

# Generate certificate
sudo certbot --nginx -d abcdeez.fg-goose.online

# Auto-renewal (add to crontab)
0 12 * * * /usr/bin/certbot renew --quiet
```

## 📊 Database Setup

### Production Database Migration

```bash
# Run migrations on deployment
cd /app && sqlx migrate run --database-url $DATABASE_URL

# Verify migrations
sqlx migrate info --database-url $DATABASE_URL
```

### Database Backup Strategy

```bash
# Create backup script: /usr/local/bin/backup-db.sh
#!/bin/bash
BACKUP_DIR="/app/backups"
DATE=$(date +%Y%m%d_%H%M%S)
DB_PATH="/app/data/production.db"

mkdir -p $BACKUP_DIR
sqlite3 $DB_PATH ".backup $BACKUP_DIR/backup_$DATE.db"
find $BACKUP_DIR -name "backup_*.db" -mtime +7 -delete

# Add to crontab: backup every 6 hours
0 */6 * * * /usr/local/bin/backup-db.sh
```

## 🔍 Monitoring and Logging

### Health Check Endpoint

The backend includes `/health` endpoint for monitoring:

```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T12:00:00Z",
  "version": "1.0.0",
  "database": "connected",
  "oauth_providers": {
    "apple": "configured",
    "github": "configured"
  }
}
```

### Log Configuration

```bash
# Production logging
export RUST_LOG=info,web_backend=debug

# Log rotation with logrotate
cat > /etc/logrotate.d/xilem-backend << EOF
/app/logs/*.log {
    daily
    rotate 30
    compress
    delaycompress
    copytruncate
    notifempty
    create 644 app app
}
EOF
```

### Monitoring Queries

```sql
-- Monitor OAuth usage
SELECT 
    auth_provider,
    COUNT(*) as user_count,
    COUNT(CASE WHEN created_at > datetime('now', '-24 hours') THEN 1 END) as daily_signups
FROM users 
WHERE auth_provider IN ('apple', 'github')
GROUP BY auth_provider;

-- Check OAuth credential validation status
SELECT 
    provider,
    credential_state,
    COUNT(*) as count,
    AVG(julianday('now') - julianday(last_check_time)) * 24 as avg_hours_since_check
FROM oauth_credential_checks 
GROUP BY provider, credential_state;
```

## 🚀 Deployment Process

### Step 1: Pre-deployment Checklist

```bash
# 1. Verify all environment variables
./scripts/verify-config.sh

# 2. Run security checks
cargo audit

# 3. Run test suite
cargo test --release

# 4. Build production binary
cargo build --release
```

### Step 2: Deploy to Production

```bash
# Using Docker Compose
docker-compose down
docker-compose build
docker-compose up -d

# Verify deployment
curl https://abcdeez.fg-goose.online/health
```

### Step 3: Post-deployment Verification

```bash
# Test OAuth endpoints
curl -X GET https://abcdeez.fg-goose.online/api/auth/oauth/apple/authorize
curl -X GET https://abcdeez.fg-goose.online/api/auth/oauth/github/authorize

# Check database migrations
sqlite3 /app/data/production.db ".tables"

# Verify SSL configuration
openssl s_client -connect abcdeez.fg-goose.online:443 -servername abcdeez.fg-goose.online
```

## 📱 Mobile App Configuration

### iOS App Configuration

```xml
<!-- Info.plist updates -->
<dict>
    <key>APIBaseURL</key>
    <string>https://abcdeez.fg-goose.online</string>
    
    <key>CFBundleURLTypes</key>
    <array>
        <dict>
            <key>CFBundleURLName</key>
            <string>Apple Sign In</string>
            <key>CFBundleURLSchemes</key>
            <array>
                <string>your-app-scheme</string>
            </array>
        </dict>
    </array>
</dict>
```

### Android App Configuration

```xml
<!-- strings.xml -->
<resources>
    <string name="api_base_url">https://abcdeez.fg-goose.online</string>
    <string name="github_client_id">your_github_client_id</string>
</resources>
```

## 🎯 Performance Optimization

### Database Optimization

```sql
-- Create indexes for OAuth queries
CREATE INDEX idx_users_auth_provider ON users(auth_provider);
CREATE INDEX idx_users_apple_user_id ON users(apple_user_id) WHERE apple_user_id IS NOT NULL;
CREATE INDEX idx_users_github_user_id ON users(github_user_id) WHERE github_user_id IS NOT NULL;
CREATE INDEX idx_oauth_checks_provider_user ON oauth_credential_checks(provider, user_id);
```

### Caching Configuration

```rust
// Redis cache (optional enhancement)
// Add to Cargo.toml: redis = "0.23"
// Configure connection pooling and JWT caching
```

## 🚨 Troubleshooting Guide

### Common Issues

**Apple Sign In JWT Verification Failed**
```bash
# Check Apple JWKS endpoint connectivity
curl https://appleid.apple.com/auth/keys

# Verify system time synchronization
ntpdate -s time.nist.gov

# Check Apple private key file permissions
ls -la /app/keys/AuthKey_*.p8
```

**GitHub OAuth Flow Issues**
```bash
# Verify OAuth app settings in GitHub
# Check redirect URI matches exactly
# Ensure client secret is correctly set

# Test GitHub API connectivity
curl -H "Authorization: token YOUR_TOKEN" https://api.github.com/user
```

**Database Connection Issues**
```bash
# Check SQLite database permissions
ls -la /app/data/production.db

# Test database connectivity
sqlite3 /app/data/production.db ".tables"

# Check available disk space
df -h /app/data
```

### Log Analysis

```bash
# OAuth-specific logs
grep "oauth" /app/logs/app.log

# Apple Sign In logs
grep -i "apple" /app/logs/app.log

# GitHub OAuth logs
grep -i "github" /app/logs/app.log

# Error patterns
grep -E "(ERROR|WARN)" /app/logs/app.log | tail -100
```

## 📋 Maintenance Tasks

### Daily Tasks
- [x] Monitor health check endpoint
- [x] Check error logs for OAuth failures
- [x] Verify backup completion
- [x] Monitor disk usage

### Weekly Tasks
- [x] Review OAuth usage metrics
- [x] Update dependencies (security patches)
- [x] Analyze performance metrics
- [x] Test disaster recovery procedures

### Monthly Tasks
- [x] Rotate secrets (JWT secret, OAuth secrets)
- [x] Review and update OAuth app configurations
- [x] Performance optimization review
- [x] Security audit

## 🔗 Useful Commands

```bash
# View active sessions
sqlite3 /app/data/production.db "SELECT COUNT(*) FROM users WHERE auth_provider != 'local';"

# OAuth provider statistics
sqlite3 /app/data/production.db "SELECT auth_provider, COUNT(*) FROM users GROUP BY auth_provider;"

# Check recent OAuth credential validations
sqlite3 /app/data/production.db "SELECT * FROM oauth_credential_checks ORDER BY last_check_time DESC LIMIT 10;"

# Monitor real-time logs
tail -f /app/logs/app.log | grep -E "(oauth|apple|github)"
```

---

## ✅ Deployment Readiness Summary

**Backend Implementation**: ✅ Complete
- Apple Sign In JWT verification with JWKS
- GitHub OAuth2 authorization code flow
- Database schema with OAuth provider support
- Secure credential storage and validation
- Background jobs for credential validation

**Security Measures**: ✅ Complete
- HTTPS-only configuration
- JWT token security with proper algorithms
- OAuth state parameter for CSRF protection
- Input validation and sanitization
- Rate limiting and security headers

**Testing**: ✅ Complete
- Comprehensive OAuth test suite
- Security validation tests
- Integration test structure
- Error handling tests

**Documentation**: ✅ Complete
- Deployment configuration guide
- Security setup instructions
- Monitoring and troubleshooting guide
- App Store compliance checklist

**Ready for SRE Team Deployment**: ✅ YES

The OAuth authentication system is production-ready with comprehensive security measures, testing, and deployment documentation. The SRE team has everything needed for a successful deployment.