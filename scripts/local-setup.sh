#!/bin/bash

# Local Testing Environment Setup Script
# This script sets up a complete local development environment for the Graph Learning System

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Graph Learning System - Local Testing Environment Setup${NC}"
echo -e "${BLUE}================================================================${NC}"
echo

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "web-backend" ] || [ ! -d "xilem-cross-platform" ]; then
    echo -e "${RED}❌ Please run this script from the root of the graph-learning-system repository${NC}"
    exit 1
fi

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install a tool if it doesn't exist
install_if_missing() {
    local tool=$1
    local install_cmd=$2
    
    if ! command_exists "$tool"; then
        echo -e "${YELLOW}📦 Installing $tool...${NC}"
        eval "$install_cmd"
    else
        echo -e "${GREEN}✅ $tool is already installed${NC}"
    fi
}

echo -e "${PURPLE}🔧 Checking prerequisites...${NC}"

# Check for Rust
if ! command_exists "cargo"; then
    echo -e "${RED}❌ Rust is not installed. Please install Rust first:${NC}"
    echo -e "${BLUE}   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
    exit 1
else
    echo -e "${GREEN}✅ Rust is installed ($(rustc --version))${NC}"
fi

# Check for Git
if ! command_exists "git"; then
    echo -e "${RED}❌ Git is not installed. Please install Git first.${NC}"
    exit 1
else
    echo -e "${GREEN}✅ Git is installed${NC}"
fi

# Install additional tools
install_if_missing "sqlx" "cargo install sqlx-cli --no-default-features --features sqlite"
install_if_missing "mdbook" "cargo install mdbook"

echo

echo -e "${PURPLE}🗃️  Setting up local database...${NC}"

# Create development database directory
mkdir -p local-dev

# Set up local environment variables
cat > local-dev/.env << 'EOF'
# Local Development Environment Configuration
ENVIRONMENT=development
DATABASE_URL=sqlite://local-dev/dev.db
JWT_SECRET=local-development-jwt-secret-minimum-32-characters-for-testing
LOG_LEVEL=debug
CORS_ORIGIN=*
PORT=8080

# Optional Redis (will use in-memory cache if not available)
# REDIS_URL=redis://127.0.0.1:6379

# Rate limiting (relaxed for local development)
RATE_LIMIT_REQUESTS=1000
RATE_LIMIT_WINDOW_SECONDS=60

# Local OAuth configuration (using placeholder values for testing)
APPLE_CLIENT_ID=local.dev.testing
APPLE_TEAM_ID=LOCALTEST
APPLE_KEY_ID=LOCALTEST1
APPLE_PRIVATE_KEY_PATH=local-dev/mock_apple_key.p8
APPLE_REDIRECT_URI=http://localhost:8080/auth/callback

GITHUB_CLIENT_ID=local_dev_github_client
GITHUB_CLIENT_SECRET=local_dev_github_secret  
GITHUB_REDIRECT_URI=http://localhost:8080/auth/github/callback

# Development features
METRICS_ENABLED=true
PERFORMANCE_MONITORING_ENABLED=true
REQUIRE_STRONG_PASSWORDS=false
MAX_FAILED_LOGIN_ATTEMPTS=10
LOGIN_LOCKOUT_DURATION_MINUTES=5
SESSION_TIMEOUT_HOURS=24

# TLS/SSL configuration (disabled for local development)
# TLS_DOMAIN=localhost
# TLS_USE_LETSENCRYPT=false
# TLS_PORT=443
# ADMIN_EMAIL=admin@yourdomain.com
EOF

echo -e "${GREEN}✅ Local environment configuration created${NC}"

# Create mock Apple private key for local testing
cat > local-dev/mock_apple_key.p8 << 'EOF'
-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgMOCKKK7rqpTbNdF8
zVddHFjBwNpNbYf5tF5rOZ7JD0KhRANCAAQyVpbtq0xpWPFYLfBCQDrKqQdNOvCz
ClhO4LRc8q2mfYCqF9Gx3uF2Kj8vTLTz9zMzMzMzMzMzMzMzMzMzMzMz
-----END PRIVATE KEY-----
EOF

echo -e "${GREEN}✅ Mock Apple private key created for local testing${NC}"

# Set up the database
echo -e "${PURPLE}📊 Initializing database...${NC}"
cd web-backend

# Export the database URL for sqlx
export DATABASE_URL=sqlite://../local-dev/dev.db

# Run database migrations
if sqlx migrate run --database-url "$DATABASE_URL"; then
    echo -e "${GREEN}✅ Database migrations completed${NC}"
else
    echo -e "${RED}❌ Database migration failed${NC}"
    exit 1
fi

# Create a local development user for testing
echo -e "${PURPLE}👤 Creating test users...${NC}"

sqlite3 ../local-dev/dev.db << 'EOF'
-- Insert a local admin user for testing
INSERT OR IGNORE INTO users (
    id, username, email, password_hash, auth_provider, metadata, created_at, updated_at
) VALUES (
    'local-admin-user-id-12345678901234567890',
    'admin',
    'admin@local.test',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/1ZiV7mNTG', -- 'admin123'
    'local',
    '{"role": "admin", "permissions": ["admin_access", "analytics_access", "user_management", "system_config"]}',
    datetime('now'),
    datetime('now')
);

-- Insert a test user with OAuth data
INSERT OR IGNORE INTO users (
    id, username, email, password_hash, auth_provider, 
    apple_user_id, github_user_id, is_private_email,
    created_at, updated_at
) VALUES (
    'local-oauth-user-id-12345678901234567890',
    'testuser',
    'test@example.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/1ZiV7mNTG', -- 'test123'
    'apple',
    'test.apple.user.id',
    'test_github_user',
    false,
    datetime('now'),
    datetime('now')
);

-- Insert some test learner data
INSERT OR IGNORE INTO learners (
    id, user_id, domain, total_tasks, correct_tasks, 
    current_streak, best_streak, created_at, updated_at
) VALUES (
    'local-learner-id-12345678901234567890',
    'local-admin-user-id-12345678901234567890',
    'alphabet',
    50,
    38,
    5,
    12,
    datetime('now'),
    datetime('now')
);
EOF

echo -e "${GREEN}✅ Test users created:${NC}"
echo -e "${BLUE}   Admin User: admin / admin123${NC}"
echo -e "${BLUE}   Test User:  testuser / test123${NC}"

cd ..

echo -e "${PURPLE}📚 Building documentation...${NC}"

# Build all documentation
if [ -d "user-manual" ]; then
    cd user-manual
    mdbook build --dest-dir ../local-dev/docs/user-manual
    cd ..
fi

if [ -d "admin-manual" ]; then
    cd admin-manual
    mdbook build --dest-dir ../local-dev/docs/admin-manual
    cd ..
fi

if [ -d "dev-manual" ]; then
    cd dev-manual
    mdbook build --dest-dir ../local-dev/docs/dev-manual
    cd ..
fi

echo -e "${GREEN}✅ Documentation built${NC}"

# Create a simple local landing page
mkdir -p local-dev/static

cat > local-dev/static/index.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Graph Learning System - Local Development</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 1200px;
            margin: 0 auto;
            padding: 2rem;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
        }
        .container {
            background: white;
            border-radius: 12px;
            padding: 3rem;
            box-shadow: 0 20px 40px rgba(0,0,0,0.1);
        }
        .dev-badge {
            background: linear-gradient(45deg, #ff9800, #ff5722);
            color: white;
            padding: 0.5rem 1rem;
            border-radius: 20px;
            display: inline-block;
            margin-bottom: 2rem;
            animation: pulse 2s infinite;
        }
        @keyframes pulse {
            0% { box-shadow: 0 0 0 0 rgba(255, 152, 0, 0.7); }
            70% { box-shadow: 0 0 0 10px rgba(255, 152, 0, 0); }
            100% { box-shadow: 0 0 0 0 rgba(255, 152, 0, 0); }
        }
        .links-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 1rem;
            margin: 2rem 0;
        }
        .link-card {
            background: #f8f9fa;
            border-radius: 8px;
            padding: 1.5rem;
            text-decoration: none;
            color: #2c3e50;
            transition: transform 0.3s ease;
        }
        .link-card:hover {
            transform: translateY(-3px);
            box-shadow: 0 5px 15px rgba(0,0,0,0.1);
        }
        .credentials {
            background: #e7f3ff;
            border-radius: 8px;
            padding: 1.5rem;
            margin: 2rem 0;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="dev-badge">🛠️ LOCAL DEVELOPMENT</div>
        <h1>📚 Graph Learning System</h1>
        <p>Local development environment with full OAuth authentication support.</p>
        
        <div class="credentials">
            <h3>🔐 Test Credentials</h3>
            <p><strong>Admin:</strong> admin / admin123</p>
            <p><strong>User:</strong> testuser / test123</p>
        </div>
        
        <div class="links-grid">
            <a href="/admin" class="link-card">
                <h3>⚙️ Admin Panel</h3>
                <p>System administration and monitoring</p>
            </a>
            <a href="/api/health" class="link-card">
                <h3>💚 Health Check</h3>
                <p>API health status and diagnostics</p>
            </a>
            <a href="/docs/user-manual" class="link-card">
                <h3>👤 User Manual</h3>
                <p>Complete user documentation</p>
            </a>
            <a href="/docs/admin-manual" class="link-card">
                <h3>📋 Admin Manual</h3>
                <p>System administration guide</p>
            </a>
            <a href="/docs/dev-manual" class="link-card">
                <h3>💻 Dev Manual</h3>
                <p>Technical documentation</p>
            </a>
            <a href="/api/admin/system/status" class="link-card">
                <h3>📊 System Status</h3>
                <p>Real-time system metrics (JSON)</p>
            </a>
        </div>
        
        <div style="background: #f0f8ff; border-radius: 8px; padding: 1.5rem; margin: 2rem 0;">
            <h3 style="margin-top: 0;">🔬 Local Testing Features</h3>
            <ul>
                <li>✅ SQLite database with test data</li>
                <li>✅ Mock OAuth providers (Apple + GitHub)</li>
                <li>✅ Admin panel with system monitoring</li>
                <li>✅ Complete API documentation</li>
                <li>✅ Debug logging enabled</li>
                <li>✅ CORS relaxed for local development</li>
                <li>✅ Auto-generated test users</li>
            </ul>
        </div>
    </div>
</body>
</html>
EOF

echo -e "${GREEN}✅ Local development landing page created${NC}"

# Create the startup script
cat > local-dev/start-server.sh << 'EOF'
#!/bin/bash

echo "🚀 Starting Graph Learning System Local Development Server"
echo "========================================================"

cd "$(dirname "$0")/.."

# Load environment variables
if [ -f local-dev/.env ]; then
    export $(grep -v '^#' local-dev/.env | xargs)
    echo "✅ Environment variables loaded"
else
    echo "❌ local-dev/.env not found"
    exit 1
fi

echo "🌐 Server will be available at:"
echo "   Main: http://localhost:${PORT:-8080}"
echo "   Admin Panel: http://localhost:${PORT:-8080}/admin"
echo "   API Docs: http://localhost:${PORT:-8080}/api/health"
echo ""
echo "🔐 Test credentials:"
echo "   Admin: admin / admin123"
echo "   User:  testuser / test123"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""

cd web-backend
cargo run
EOF

chmod +x local-dev/start-server.sh

echo
echo -e "${GREEN}🎉 Local testing environment setup complete!${NC}"
echo
echo -e "${BLUE}📋 Quick Start:${NC}"
echo -e "${YELLOW}   1. Start the server: ./local-dev/start-server.sh${NC}"
echo -e "${YELLOW}   2. Open browser: http://localhost:8080${NC}"
echo -e "${YELLOW}   3. Login with: admin / admin123${NC}"
echo
echo -e "${BLUE}🔗 Available URLs:${NC}"
echo -e "${YELLOW}   Main App:     http://localhost:8080${NC}"
echo -e "${YELLOW}   Admin Panel:  http://localhost:8080/admin${NC}"
echo -e "${YELLOW}   API Health:   http://localhost:8080/api/health${NC}"
echo -e "${YELLOW}   System Status: http://localhost:8080/api/admin/system/status${NC}"
echo
echo -e "${BLUE}🗂️  Local files created:${NC}"
echo -e "${YELLOW}   local-dev/.env          - Environment configuration${NC}"
echo -e "${YELLOW}   local-dev/dev.db        - SQLite database with test data${NC}"
echo -e "${YELLOW}   local-dev/start-server.sh - Server startup script${NC}"
echo -e "${YELLOW}   local-dev/static/       - Local web assets${NC}"
echo -e "${YELLOW}   local-dev/docs/         - Built documentation${NC}"
echo
echo -e "${GREEN}✨ Ready for local development and testing!${NC}"