# System Requirements

This page outlines the minimum and recommended system requirements for deploying the Graph Learning System with OAuth authentication in production.

## Production Environment Requirements

### Minimum Requirements

**Hardware**:
- CPU: 2 cores (x86_64 or ARM64)
- RAM: 2GB available memory
- Storage: 10GB SSD (minimum), 50GB recommended
- Network: Stable internet connection with 10 Mbps minimum bandwidth

**Operating System**:
- Ubuntu 20.04 LTS or later
- CentOS 8+ / RHEL 8+
- Debian 11+
- Amazon Linux 2
- Or any Linux distribution with Docker support

**Software Dependencies**:
- Docker 20.10+ and Docker Compose 2.0+
- SSL certificate (Let's Encrypt recommended)
- Domain name with DNS management access

### Recommended Requirements

**Hardware**:
- CPU: 4+ cores (x86_64 or ARM64)
- RAM: 4GB+ available memory
- Storage: 100GB+ NVMe SSD with backup storage
- Network: High-speed internet with redundancy

**Infrastructure**:
- Load balancer (Nginx, HAProxy, or cloud-based)
- CDN for static assets (CloudFlare, AWS CloudFront)
- Monitoring solution (Prometheus, DataDog, etc.)
- Backup storage (cloud or local)

## OAuth Provider Requirements

### Apple Sign In Requirements

**Apple Developer Account**:
- Active Apple Developer Program membership ($99/year)
- Team ID and appropriate permissions
- iOS app with Bundle Identifier configured

**Technical Requirements**:
- Domain ownership verification with Apple
- HTTPS-enabled website for web authentication
- Valid SSL certificate with proper certificate chain
- Apple private key file (AuthKey_XXXXXXXXXX.p8)

**App Store Requirements** (for iOS app):
- iOS 13.0+ minimum deployment target
- AuthenticationServices framework integration
- App Store compliant button design and placement
- Privacy policy accessible from your app

### GitHub OAuth Requirements

**GitHub Organization/Account**:
- GitHub account with OAuth app creation permissions
- Verified email address
- Two-factor authentication enabled (recommended)

**OAuth Application**:
- Application name and description
- Homepage URL (your application's main website)
- Authorization callback URL (https://abcdeez.fg-goose.online/auth/github/callback)
- Application logo (optional but recommended)

## Security Requirements

### SSL/TLS Configuration

**Certificate Requirements**:
- Valid SSL certificate from a trusted CA
- Support for TLS 1.2 and TLS 1.3
- Proper certificate chain configuration
- Regular certificate renewal (automated recommended)

**Security Headers**:
```nginx
# Required security headers
add_header X-Frame-Options DENY;
add_header X-Content-Type-Options nosniff;
add_header X-XSS-Protection "1; mode=block";
add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
add_header Referrer-Policy "strict-origin-when-cross-origin";
```

### Firewall Configuration

**Open Ports**:
- Port 80 (HTTP) - For Let's Encrypt and HTTP-to-HTTPS redirect
- Port 443 (HTTPS) - For all application traffic
- Port 22 (SSH) - For server management (restrict to admin IPs)

**Blocked Ports**:
- Port 8080 - Backend application port (should only be accessible to reverse proxy)
- All other ports not explicitly required

## Database Requirements

### SQLite Configuration

**File System Requirements**:
- SSD storage for optimal performance
- Regular backup storage (local and remote)
- Proper file permissions (600 for database file)
- Write access for the application user

**Performance Considerations**:
- WAL mode enabled for better concurrent access
- Regular VACUUM operations for maintenance
- Index optimization for OAuth queries

**Backup Requirements**:
- Automated daily backups
- Point-in-time recovery capability
- Backup retention policy (minimum 30 days)
- Backup encryption for sensitive data

## Network Requirements

### Internet Connectivity

**Required External Connections**:
- `appleid.apple.com` (Apple's identity servers)
- `api.github.com` (GitHub API endpoints)
- Package registries for updates
- Certificate authority servers (for SSL validation)

**Bandwidth Requirements**:
- Minimum 10 Mbps for small deployments (< 100 concurrent users)
- 100 Mbps+ for larger deployments
- Low latency connection (< 100ms) to OAuth providers

### DNS Configuration

**Required DNS Records**:
```dns
# A record for your domain
abcdeez.fg-goose.online.    A    YOUR_SERVER_IP

# CNAME for www (optional)
www.abcdeez.fg-goose.online. CNAME abcdeez.fg-goose.online.

# Apple domain verification (if using web Apple Sign In)
apple-app-site-association.abcdeez.fg-goose.online. CNAME abcdeez.fg-goose.online.
```

## Monitoring Requirements

### Health Check Endpoints

**Application Health**:
- `/health` - Overall system health
- `/health/oauth` - OAuth provider connectivity
- `/health/db` - Database connectivity

**Expected Response Time**:
- Health checks: < 100ms
- OAuth authentication: < 2 seconds
- Database queries: < 50ms

### Log Storage

**Disk Space**:
- Minimum 5GB for log storage
- Log rotation configured
- Structured logging format (JSON recommended)

**Log Retention**:
- Application logs: 30 days minimum
- Security logs: 90 days minimum
- Audit logs: 1 year minimum

## Performance Benchmarks

### Expected Performance

**Authentication Performance**:
- OAuth sign-in: < 3 seconds end-to-end
- JWT verification: < 100ms
- Session validation: < 50ms
- Database queries: < 25ms

**Concurrent Users**:
- Minimum setup: 50 concurrent users
- Recommended setup: 500+ concurrent users
- Horizontal scaling: Unlimited with load balancer

### Resource Usage

**Memory Usage**:
- Base application: ~100MB
- Per concurrent user: ~2-5MB
- OAuth token cache: ~50MB
- Database connection pool: ~20MB

**CPU Usage**:
- Idle: < 5% CPU utilization
- Normal load: 20-40% CPU utilization
- Peak load: 60-80% CPU utilization

## Compliance Requirements

### Data Protection

**GDPR Compliance** (if serving EU users):
- User consent mechanisms
- Data processing lawful basis
- Right to erasure implementation
- Data portability features
- Privacy policy and terms of service

**CCPA Compliance** (if serving California users):
- Personal information disclosure
- Opt-out mechanisms
- Data deletion capabilities
- Privacy rights information

### Security Standards

**Authentication Security**:
- OAuth 2.0 and OpenID Connect compliance
- PKCE (Proof Key for Code Exchange) implementation
- Secure token storage and transmission
- Regular security audits

**Data Security**:
- Encryption at rest and in transit
- Secure backup procedures
- Access logging and monitoring
- Incident response procedures

## Development Environment

### Local Development

**Minimum Requirements**:
- Rust 1.75+ with Cargo
- Git version control
- Code editor with Rust support (VS Code recommended)
- 4GB RAM for development builds

**Recommended Tools**:
- Docker Desktop for container testing
- Postman or similar API testing tool
- Database administration tools
- SSL certificate for local HTTPS testing

### Testing Environment

**Staging Environment**:
- Mirror of production environment
- Separate OAuth applications for testing
- Test user accounts and data
- Automated testing pipeline

**Security Testing**:
- OAuth flow security validation
- SQL injection prevention testing
- Rate limiting verification
- SSL/TLS configuration testing

## Next Steps

Once you've verified these requirements:

1. **Set up your production environment** - [Installation Guide](./installation.md)
2. **Configure OAuth providers** - [OAuth Overview](../oauth/overview.md)
3. **Deploy the application** - [Production Environment](../deployment/production.md)

For specific deployment scenarios or questions about requirements, see the [Troubleshooting](../troubleshooting/common-issues.md) section.