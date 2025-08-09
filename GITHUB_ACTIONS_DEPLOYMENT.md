# GitHub Actions Automatic Deployment

This document outlines the complete GitHub Actions setup for automatic deployment to staging and production environments.

## 🚀 Deployment Strategy

### Automatic Deployments

```mermaid
gitGraph
    commit id: "Development"
    branch dev
    commit id: "Feature work"
    commit id: "OAuth implementation"
    commit id: "Push to dev" tag: "→ STAGING"
    checkout main
    merge dev
    commit id: "Merge to main" tag: "→ PRODUCTION"
```

**Trigger Summary:**
- **Staging**: Every push to `dev` branch → `staging.abcdeez.fg-goose.online`
- **Production**: Every push to `main` branch → `abcdeez.fg-goose.online`
- **Documentation**: Manual trigger or doc changes → Current environment

## 📋 Workflow Configurations

### 1. Staging Deployment (`.github/workflows/deploy-staging.yml`)

**Triggers:**
```yaml
on:
  push:
    branches: [ dev ]
  workflow_dispatch:
```

**Process:**
1. **Test Phase**: Rust formatting, clippy, tests with OAuth placeholders
2. **Build Phase**: Documentation + staging-specific landing page
3. **Deploy Phase**: Deploy to `staging.abcdeez.fg-goose.online`

**Key Features:**
- 🧪 Staging-specific branding and warnings
- 📊 Latest commit information displayed
- 🔄 Daily change frequency for search engines
- 🚧 Clear staging environment indicators
- ⚡ Fast deployment with cancel-in-progress

### 2. Production Deployment (`.github/workflows/deploy-production.yml`)

**Triggers:**
```yaml
on:
  push:
    branches: [ main ]
    tags: [ 'v*' ]
  workflow_dispatch:
```

**Process:**
1. **Test Phase**: Full test suite + security audit
2. **Build Phase**: Production-optimized documentation + PWA
3. **Deploy Phase**: Deploy to `abcdeez.fg-goose.online`

**Key Features:**
- 🔒 Security audit with `cargo audit`
- 🌟 Production-grade landing page design
- 📈 SEO optimization with meta tags and sitemap
- 🎯 Enterprise feature highlighting
- 🛡️ No cancel-in-progress for production safety

### 3. Documentation Only (`.github/workflows/docs.yml`)

**Triggers:**
```yaml
on:
  workflow_dispatch:
  push:
    paths:
      - 'user-manual/**'
      - 'admin-manual/**'
      - 'dev-manual/**'
```

**Use Case:** Manual documentation updates without full deployment

## 🌐 Deployment Environments

### Staging Environment

**URL**: `https://staging.abcdeez.fg-goose.online`

**Configuration:**
```yaml
environment:
  name: staging
  url: https://staging.abcdeez.fg-goose.online

concurrency:
  group: "staging-deployment"
  cancel-in-progress: true  # Allow cancelling for faster iteration
```

**Features:**
- 🚧 Clear staging environment branding
- ⚠️ Data reset warnings
- 🔗 Easy navigation to production
- 📊 Commit information display
- 🎯 Latest development features

**DNS Configuration:**
```dns
staging.abcdeez.fg-goose.online.  CNAME  your-org.github.io.
```

### Production Environment

**URL**: `https://abcdeez.fg-goose.online`

**Configuration:**
```yaml
environment:
  name: production
  url: https://abcdeez.fg-goose.online

concurrency:
  group: "production-deployment"
  cancel-in-progress: false  # Never cancel production deployments
```

**Features:**
- ✅ Production-ready branding
- 🎨 Enhanced visual design
- 📈 SEO optimization
- 🔍 Rich meta tags and Open Graph
- 🌟 Enterprise feature showcase

## 🔒 Automatic SSL/TLS with Let's Encrypt

The server now includes **automatic SSL certificate provisioning** using Let's Encrypt! 

### **Zero-Configuration HTTPS**
```env
# Production environment - automatic HTTPS
TLS_DOMAIN=abcdeez.fg-goose.online
TLS_USE_LETSENCRYPT=true
TLS_PORT=443
ADMIN_EMAIL=admin@yourdomain.com
```

### **How It Works**
1. **Certificate Provisioning**: Automatically requests SSL certificates from Let's Encrypt
2. **HTTP-01 Challenge**: Handles ACME challenges via `/.well-known/acme-challenge/` endpoint
3. **Auto-Renewal**: Checks daily and renews certificates before expiration
4. **HTTP Redirect**: Automatically redirects HTTP traffic to HTTPS
5. **Graceful Fallback**: Uses self-signed certificates if Let's Encrypt fails

### **DNS Configuration Required**
```dns
# Point your domain to the server
abcdeez.fg-goose.online.     A     YOUR_SERVER_IP
staging.abcdeez.fg-goose.online. A  YOUR_STAGING_IP

# Enable HTTPS (automatic redirect from HTTP)
Port 80  -> HTTP (redirects to HTTPS + ACME challenges)
Port 443 -> HTTPS (Let's Encrypt certificates)
```

### **Certificate Management**
- **Storage**: Certificates saved to `./certs/` directory
- **Renewal**: Automatic daily checks, 30-day renewal window
- **Monitoring**: Certificate status visible in admin panel
- **Backup**: Certificate files backed up with deployment

### **Development vs Production**
```bash
# Development (HTTP only)
TLS_DOMAIN=  # Empty = no TLS

# Production (Automatic HTTPS)
TLS_DOMAIN=abcdeez.fg-goose.online
TLS_USE_LETSENCRYPT=true
```

## 🔧 OAuth Environment Configuration

### GitHub Repository Secrets

**Required Secrets (for future backend deployment):**
```bash
# Staging Environment
STAGING_APPLE_CLIENT_ID=your.app.bundle.identifier.staging
STAGING_APPLE_TEAM_ID=YOUR_TEAM_ID
STAGING_APPLE_KEY_ID=YOUR_KEY_ID
STAGING_APPLE_PRIVATE_KEY=-----BEGIN PRIVATE KEY-----...
STAGING_GITHUB_CLIENT_ID=staging_github_client_id
STAGING_GITHUB_CLIENT_SECRET=staging_github_client_secret

# Production Environment  
PRODUCTION_APPLE_CLIENT_ID=your.app.bundle.identifier
PRODUCTION_APPLE_TEAM_ID=YOUR_TEAM_ID
PRODUCTION_APPLE_KEY_ID=YOUR_KEY_ID
PRODUCTION_APPLE_PRIVATE_KEY=-----BEGIN PRIVATE KEY-----...
PRODUCTION_GITHUB_CLIENT_ID=production_github_client_id
PRODUCTION_GITHUB_CLIENT_SECRET=production_github_client_secret

# Database & Security
STAGING_DATABASE_URL=postgresql://staging-user:pass@staging-db/graphlearning
PRODUCTION_DATABASE_URL=postgresql://prod-user:pass@prod-db/graphlearning
STAGING_JWT_SECRET=staging-jwt-secret-minimum-32-characters
PRODUCTION_JWT_SECRET=production-jwt-secret-minimum-32-characters

# TLS/SSL Configuration (Let's Encrypt)
STAGING_TLS_DOMAIN=staging.abcdeez.fg-goose.online
STAGING_TLS_USE_LETSENCRYPT=true
STAGING_TLS_PORT=443
STAGING_ADMIN_EMAIL=admin@yourdomain.com

PRODUCTION_TLS_DOMAIN=abcdeez.fg-goose.online
PRODUCTION_TLS_USE_LETSENCRYPT=true
PRODUCTION_TLS_PORT=443
PRODUCTION_ADMIN_EMAIL=admin@yourdomain.com
```

### OAuth Provider Setup

**Apple Developer Console:**
- **Development**: `com.yourcompany.app.dev`
- **Staging**: `com.yourcompany.app.staging` → `staging.abcdeez.fg-goose.online`
- **Production**: `com.yourcompany.app` → `abcdeez.fg-goose.online`

**GitHub OAuth Apps:**
- **Development**: `http://localhost:8080/auth/github/callback`
- **Staging**: `https://staging.abcdeez.fg-goose.online/auth/github/callback`
- **Production**: `https://abcdeez.fg-goose.online/auth/github/callback`

## 📊 Monitoring & Status

### Deployment Status Badges

Add to your README.md:
```markdown
[![Deploy Staging](https://github.com/your-org/graph-learning-system/workflows/Deploy%20to%20Staging/badge.svg)](https://github.com/your-org/graph-learning-system/actions/workflows/deploy-staging.yml)

[![Deploy Production](https://github.com/your-org/graph-learning-system/workflows/Deploy%20to%20Production/badge.svg)](https://github.com/your-org/graph-learning-system/actions/workflows/deploy-production.yml)
```

### GitHub Environments

**Configure in GitHub Repository Settings > Environments:**

**Staging Environment:**
- No protection rules (fast iteration)
- Optional reviewers for major changes
- Auto-deploy on every push to `dev`

**Production Environment:**
- Required reviewers (recommended)
- Branch protection for `main`
- Deployment protection rules
- Auto-deploy on push to `main` or tags

### Health Checks

**Staging:**
```bash
curl https://staging.abcdeez.fg-goose.online/health
```

**Production:**
```bash
curl https://abcdeez.fg-goose.online/health
```

## 🔄 Development Workflow

### Typical Development Flow

```bash
# 1. Work on feature branch
git checkout -b feature/new-oauth-feature

# 2. Develop and test locally
cargo test
cargo run

# 3. Push to dev for staging deployment
git checkout dev
git merge feature/new-oauth-feature
git push origin dev
# → Automatically deploys to staging.abcdeez.fg-goose.online

# 4. Test on staging environment
# Visit https://staging.abcdeez.fg-goose.online
# Test OAuth flows, documentation, etc.

# 5. Deploy to production
git checkout main
git merge dev
git push origin main
# → Automatically deploys to abcdeez.fg-goose.online
```

### Hotfix Workflow

```bash
# For urgent production fixes
git checkout -b hotfix/urgent-fix main
# Make minimal fix
git checkout main
git merge hotfix/urgent-fix
git push origin main
# → Immediately deploys to production

# Don't forget to merge back to dev
git checkout dev
git merge main
git push origin dev
```

## 🚨 Deployment Safety

### Pre-deployment Checks

**Both environments run:**
- ✅ Rust code formatting (`cargo fmt`)
- ✅ Linting with Clippy (`cargo clippy`)
- ✅ Full test suite (`cargo test`)
- ✅ OAuth placeholder validation

**Production additionally runs:**
- 🔐 Security audit (`cargo audit`)
- 🛡️ Advanced safety checks

### Rollback Strategy

**If deployment fails:**
1. Check GitHub Actions logs
2. Fix issues in new commit
3. Push fix to trigger re-deployment

**Emergency rollback:**
1. Revert problematic commit
2. Push revert to main/dev
3. Auto-deployment will rollback

## 🎯 Performance & SEO

### Staging Environment
- Fast iteration focus
- Debug information visible
- Daily sitemap updates
- Development-friendly features

### Production Environment
- SEO optimization
- Rich metadata and Open Graph tags
- Production sitemap
- Performance optimized
- Enterprise branding

## 🔗 Useful Links

**GitHub Actions:**
- [Staging Workflow](../../actions/workflows/deploy-staging.yml)
- [Production Workflow](../../actions/workflows/deploy-production.yml)
- [Documentation Workflow](../../actions/workflows/docs.yml)

**Deployed Sites:**
- [🚧 Staging Environment](https://staging.abcdeez.fg-goose.online)
- [🌟 Production Environment](https://abcdeez.fg-goose.online)

**Repository Settings:**
- [Environments](../../settings/environments)
- [Secrets](../../settings/secrets/actions)
- [Pages](../../settings/pages)

---

## ✅ Summary

**The deployment system is FULLY CONFIGURED and ready to use:**

🟢 **Automatic Staging**: Every push to `dev` → `staging.abcdeez.fg-goose.online`
🟢 **Automatic Production**: Every push to `main` → `abcdeez.fg-goose.online`
🟢 **OAuth Ready**: Environment-specific Apple/GitHub OAuth configurations
🟢 **Security Focused**: Testing, auditing, and validation before deployment
🟢 **SEO Optimized**: Production environment optimized for search engines
🟢 **Documentation Integrated**: Full manual deployment with each release

**Start using immediately:** Push to `dev` for staging, push to `main` for production!