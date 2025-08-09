#!/bin/bash

# Security Remediation Script
# Helps fix common security issues in the codebase

set -e

echo "Security Remediation Helper"
echo "==========================="
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Function to generate secure random string
generate_secret() {
    local length=$1
    openssl rand -base64 $length | tr -d '\n'
}

# Function to prompt user
prompt_yes_no() {
    local prompt=$1
    local response
    read -p "$prompt (y/n): " response
    [[ "$response" == "y" || "$response" == "Y" ]]
}

echo -e "${BLUE}This script will help you remediate security issues found in the audit${NC}"
echo ""

# Check if .env exists
if [ ! -f .env ]; then
    echo -e "${YELLOW}Creating .env from template...${NC}"
    cp .env.template .env
    echo -e "${GREEN}✓ Created .env file${NC}"
fi

# Generate JWT secret if not set
source .env 2>/dev/null || true
if [ -z "$JWT_SECRET" ] || [[ "$JWT_SECRET" == *"CHANGE_ME"* ]]; then
    echo ""
    echo -e "${YELLOW}JWT_SECRET needs to be generated${NC}"
    if prompt_yes_no "Generate a secure JWT secret?"; then
        NEW_SECRET=$(generate_secret 64)
        if grep -q "^JWT_SECRET=" .env; then
            # Update existing
            sed -i.bak "s/^JWT_SECRET=.*/JWT_SECRET=$NEW_SECRET/" .env
        else
            # Add new
            echo "JWT_SECRET=$NEW_SECRET" >> .env
        fi
        echo -e "${GREEN}✓ Generated and saved secure JWT secret${NC}"
    fi
fi

# Check for test database files
echo ""
echo -e "${BLUE}Checking for test database files...${NC}"
TEST_DB_FILES=$(ls test.db* 2>/dev/null || true)
if [ ! -z "$TEST_DB_FILES" ]; then
    echo -e "${YELLOW}Found test database files: $TEST_DB_FILES${NC}"
    if prompt_yes_no "Remove test database files?"; then
        rm -f test.db*
        echo -e "${GREEN}✓ Removed test database files${NC}"
    fi
fi

# Install pre-commit hook
echo ""
echo -e "${BLUE}Setting up pre-commit hook...${NC}"
if [ ! -f .git/hooks/pre-commit ]; then
    if prompt_yes_no "Install pre-commit security hook?"; then
        cp scripts/pre-commit-check.sh .git/hooks/pre-commit
        chmod +x .git/hooks/pre-commit
        echo -e "${GREEN}✓ Installed pre-commit hook${NC}"
    fi
else
    echo -e "${GREEN}✓ Pre-commit hook already installed${NC}"
fi

# Check for hardcoded secrets in code
echo ""
echo -e "${BLUE}Scanning for potential hardcoded secrets...${NC}"
SUSPECT_FILES=$(grep -r "placeholder_" src/ 2>/dev/null | grep -v "Binary file" | cut -d: -f1 | sort -u || true)
if [ ! -z "$SUSPECT_FILES" ]; then
    echo -e "${YELLOW}Found files with potential placeholder secrets:${NC}"
    echo "$SUSPECT_FILES"
    echo ""
    echo "These have been addressed in the code changes, but verify they're updated."
fi

# Create secure directories
echo ""
echo -e "${BLUE}Setting up secure directories...${NC}"
if [ ! -d data ]; then
    mkdir -p data
    chmod 700 data
    echo -e "${GREEN}✓ Created secure data directory${NC}"
fi

if [ ! -d logs ]; then
    mkdir -p logs
    chmod 700 logs
    echo -e "${GREEN}✓ Created secure logs directory${NC}"
fi

# Validate Cargo dependencies for security
echo ""
echo -e "${BLUE}Checking for dependency vulnerabilities...${NC}"
if command -v cargo-audit &> /dev/null; then
    cargo audit || echo -e "${YELLOW}⚠ Some vulnerabilities found - review cargo audit output${NC}"
else
    echo -e "${YELLOW}Install cargo-audit for vulnerability scanning:${NC}"
    echo "  cargo install cargo-audit"
fi

# Generate development certificates for TLS testing
echo ""
if [ ! -f certs/localhost.crt ] && prompt_yes_no "Generate development TLS certificates?"; then
    mkdir -p certs
    openssl req -x509 -newkey rsa:4096 -nodes \
        -keyout certs/localhost.key \
        -out certs/localhost.crt \
        -days 365 \
        -subj "/CN=localhost" 2>/dev/null
    chmod 600 certs/*.key
    echo -e "${GREEN}✓ Generated development TLS certificates${NC}"
fi

# Summary of actions needed
echo ""
echo "==========================================="
echo -e "${BLUE}Manual Actions Required:${NC}"
echo ""
echo "1. Review and update OAuth credentials in .env:"
echo "   - Set APPLE_* variables if using Apple Sign In"
echo "   - Set GITHUB_* variables if using GitHub OAuth"
echo ""
echo "2. For production deployment:"
echo "   - Use a secrets management system (e.g., HashiCorp Vault)"
echo "   - Set up proper TLS certificates"
echo "   - Configure CORS_ORIGIN to your frontend URL"
echo "   - Use PostgreSQL instead of SQLite"
echo ""
echo "3. Security best practices:"
echo "   - Rotate secrets regularly"
echo "   - Enable audit logging"
echo "   - Set up monitoring and alerting"
echo "   - Implement rate limiting"
echo ""
echo -e "${GREEN}Run './scripts/validate_env.sh' to verify your configuration${NC}"