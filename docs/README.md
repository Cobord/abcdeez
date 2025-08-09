# Graph Learning System Documentation

This repository contains comprehensive documentation for the Graph Learning System, including OAuth authentication with Apple Sign In and GitHub OAuth.

## 📚 Available Manuals

### 👤 [User Manual](../user-manual/)
Complete guide for learners using the system:
- Account creation with Apple Sign In
- Training sessions and progress tracking  
- Dashboard features and analytics
- Mobile app usage (iOS/Android)
- Tips and troubleshooting

### ⚙️ [Administrator Manual](../admin-manual/)
Comprehensive guide for system administrators:
- Production deployment with OAuth
- Apple Sign In and GitHub OAuth setup
- Security configuration and monitoring
- User management and data privacy
- Maintenance and troubleshooting

### 💻 [Developer Manual](../dev-manual/)
Technical documentation for developers:
- Architecture and API reference
- OAuth implementation details
- Backend and frontend development
- Testing strategies and debugging
- Contributing guidelines

## 🚀 OAuth Authentication Features

The Graph Learning System now includes production-ready OAuth authentication:

✅ **Apple Sign In**
- Native iOS integration with AuthenticationServices
- JWT verification using Apple's JWKS
- App Store compliant button design
- Private email relay support
- Real user detection

✅ **GitHub OAuth**
- Authorization code flow with PKCE
- State parameter for CSRF protection
- Developer-friendly authentication
- Secure token management

✅ **Security & Privacy**
- HTTPS-only communication
- Encrypted token storage
- Input validation and rate limiting
- GDPR/CCPA compliance ready
- Comprehensive audit logging

## 🌐 Live Documentation

The documentation is automatically built and deployed to GitHub Pages:

- **Production**: [https://abcdeez.fg-goose.online](https://abcdeez.fg-goose.online) (Documentation + Landing Page)
- **PWA Application**: [https://abcdeez.fg-goose.online/app](https://abcdeez.fg-goose.online/app) (Web App)

## 🛠 Building Documentation Locally

### Prerequisites

Install mdBook:
```bash
cargo install mdbook
```

### Build All Manuals

```bash
# Build user manual
cd user-manual
mdbook build
mdbook serve # Optional: serve locally at http://localhost:3000

# Build admin manual
cd ../admin-manual
mdbook build
mdbook serve --port 3001

# Build developer manual  
cd ../dev-manual
mdbook build
mdbook serve --port 3002
```

### Automated Documentation Site

Build the complete documentation site:
```bash
./scripts/build-docs.sh
```

This creates a unified documentation site in `docs-site/` with:
- Landing page with navigation
- All three manuals integrated
- Search functionality
- Responsive design

## 📝 Contributing to Documentation

### File Structure

```
├── user-manual/          # User-facing documentation
│   ├── src/
│   └── book.toml
├── admin-manual/         # Administrator documentation  
│   ├── src/
│   │   ├── oauth/        # OAuth setup guides
│   │   ├── deployment/   # Production deployment
│   │   ├── security/     # Security configuration
│   │   └── ...
│   └── book.toml
├── dev-manual/          # Technical documentation
│   ├── src/
│   │   ├── oauth/       # OAuth implementation
│   │   ├── architecture/ # System architecture
│   │   ├── testing/     # Testing strategies
│   │   └── ...
│   └── book.toml
└── .github/workflows/   # CI/CD for documentation
    └── docs.yml
```

### Writing Guidelines

**Markdown Standards**:
- Use clear, descriptive headings
- Include code examples with syntax highlighting
- Add diagrams for complex concepts (ASCII art or Mermaid)
- Cross-reference between manuals when appropriate

**OAuth Documentation**:
- Include security warnings for sensitive operations
- Provide complete configuration examples
- Document troubleshooting steps for common issues
- Show both iOS and web integration examples

**Code Examples**:
```rust
// Always include context and error handling
pub async fn verify_apple_token(token: &str) -> Result<User, AuthError> {
    let service = AppleAuthService::new(config)?;
    let claims = service.verify_identity_token(token).await?;
    let user = create_or_update_user(&claims).await?;
    Ok(user)
}
```

### Review Process

1. Create feature branch: `docs/your-update-description`
2. Make changes to relevant manual(s)
3. Test locally with `mdbook serve`
4. Submit pull request with documentation changes
5. Automated CI builds and deploys to staging
6. Review and merge to deploy to production

## 🚦 CI/CD Pipeline

The documentation is automatically deployed using GitHub Actions:

**Triggers**:
- Push to `main` or `dev` branches
- Changes to any `*-manual/` directories
- Manual workflow dispatch

**Process**:
1. Install mdBook and dependencies
2. Build all three manuals
3. Create unified documentation site
4. Deploy to GitHub Pages
5. Update DNS (if configured)

**Deployment Status**: ![Docs Status](https://github.com/your-org/graph-learning-system/workflows/Deploy%20Documentation%20to%20GitHub%20Pages/badge.svg)

## 🔧 Configuration

### GitHub Pages Setup

1. Enable GitHub Pages in repository settings
2. Set source to "GitHub Actions"
3. Configure custom domain (optional):
   ```
   Settings > Pages > Custom domain: docs.graphlearning.io
   ```

### DNS Configuration (Optional)

If using a custom domain:
```dns
abcdeez.fg-goose.online.  CNAME  your-org.github.io.
```

### Environment Variables

For advanced configuration, set in GitHub repository secrets:
- `DOCS_DOMAIN`: Custom domain name
- `ANALYTICS_ID`: Google Analytics tracking ID (optional)

## 📊 Documentation Metrics

Track documentation usage with:
- GitHub Pages analytics
- Google Analytics (if configured)
- User feedback forms in manuals
- Issue tracking for documentation bugs

## 🆘 Getting Help

**Documentation Issues**:
- File issues in the main repository
- Tag with `documentation` label
- Include manual name and page reference

**OAuth Setup Help**:
- Check the [Admin Manual OAuth section](../admin-manual/src/oauth/overview.md)
- Review [troubleshooting guides](../admin-manual/src/troubleshooting/oauth-problems.md)
- Ask in project discussions

**Technical Implementation**:
- See [Developer Manual](../dev-manual/)
- Check existing code examples
- Review test implementations

---

## 📄 License

This documentation is provided under the same license as the Graph Learning System codebase.

**Built with**: mdBook, GitHub Actions, and ❤️

**OAuth Authentication**: Apple Sign In + GitHub OAuth ready for production deployment.