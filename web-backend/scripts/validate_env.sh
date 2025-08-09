#!/bin/bash

# Environment Configuration Validator
# This script validates that your environment configuration is secure

set -e

echo "Environment Configuration Security Validator"
echo "==========================================="
echo ""

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track if any issues are found
ISSUES_FOUND=0

# Check if .env file exists
if [ ! -f .env ]; then
    echo -e "${RED}✗ .env file not found${NC}"
    echo "  Create one from .env.template"
    exit 1
fi

# Source the environment file
set -a
source .env
set +a

echo "Checking environment: $ENVIRONMENT"
echo ""

# Function to check a variable
check_var() {
    local var_name=$1
    local var_value=${!var_name}
    local min_length=$2
    local check_type=$3
    
    if [ -z "$var_value" ]; then
        echo -e "${RED}✗ $var_name is not set${NC}"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
        return 1
    fi
    
    if [ ! -z "$min_length" ] && [ ${#var_value} -lt $min_length ]; then
        echo -e "${RED}✗ $var_name is too short (minimum $min_length characters)${NC}"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
        return 1
    fi
    
    # Check for weak patterns
    if [ "$check_type" = "secret" ]; then
        if [[ "$var_value" == *"placeholder"* ]] || \
           [[ "$var_value" == *"test"* ]] || \
           [[ "$var_value" == *"demo"* ]] || \
           [[ "$var_value" == *"default"* ]] || \
           [[ "$var_value" == *"change_me"* ]] || \
           [[ "$var_value" == *"CHANGE_ME"* ]]; then
            echo -e "${RED}✗ $var_name contains weak/placeholder values${NC}"
            ISSUES_FOUND=$((ISSUES_FOUND + 1))
            return 1
        fi
    fi
    
    echo -e "${GREEN}✓ $var_name is configured${NC}"
    return 0
}

# Check critical security variables
echo "Security Configuration:"
echo "----------------------"
check_var "JWT_SECRET" 64 "secret"

if [ "$ENVIRONMENT" = "production" ] || [ "$ENVIRONMENT" = "staging" ]; then
    echo ""
    echo "Production/Staging Checks:"
    echo "--------------------------"
    
    # Check CORS is not wildcard
    if [ "$CORS_ORIGIN" = "*" ]; then
        echo -e "${RED}✗ CORS_ORIGIN cannot be wildcard in production${NC}"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    else
        echo -e "${GREEN}✓ CORS_ORIGIN is restricted${NC}"
    fi
    
    # Check strong passwords are required
    if [ "$REQUIRE_STRONG_PASSWORDS" != "true" ]; then
        echo -e "${YELLOW}⚠ Strong passwords should be required in production${NC}"
    else
        echo -e "${GREEN}✓ Strong passwords are required${NC}"
    fi
    
    # Check database URL
    if [[ "$DATABASE_URL" == *"sqlite"* ]]; then
        echo -e "${YELLOW}⚠ SQLite is not recommended for production${NC}"
    else
        echo -e "${GREEN}✓ Production database configured${NC}"
    fi
    
    # Check OAuth configuration if variables are present
    if [ ! -z "$APPLE_CLIENT_ID" ]; then
        echo ""
        echo "Apple OAuth Configuration:"
        echo "-------------------------"
        check_var "APPLE_CLIENT_ID" 10
        check_var "APPLE_TEAM_ID" 10
        check_var "APPLE_KEY_ID" 10
        check_var "APPLE_PRIVATE_KEY_PATH" 1
        
        if [ ! -f "$APPLE_PRIVATE_KEY_PATH" ]; then
            echo -e "${RED}✗ Apple private key file not found: $APPLE_PRIVATE_KEY_PATH${NC}"
            ISSUES_FOUND=$((ISSUES_FOUND + 1))
        fi
    fi
    
    if [ ! -z "$GITHUB_CLIENT_ID" ]; then
        echo ""
        echo "GitHub OAuth Configuration:"
        echo "--------------------------"
        check_var "GITHUB_CLIENT_ID" 10
        check_var "GITHUB_CLIENT_SECRET" 40 "secret"
    fi
fi

echo ""
echo "General Configuration:"
echo "---------------------"

# Check logging level
if [ "$ENVIRONMENT" = "production" ] && [ "$LOG_LEVEL" = "debug" ]; then
    echo -e "${YELLOW}⚠ Debug logging in production may expose sensitive data${NC}"
else
    echo -e "${GREEN}✓ Log level appropriate for environment${NC}"
fi

# Check rate limiting
if [ "$RATE_LIMIT_REQUESTS" -gt 1000 ]; then
    echo -e "${YELLOW}⚠ Rate limit may be too high${NC}"
else
    echo -e "${GREEN}✓ Rate limiting configured${NC}"
fi

# Summary
echo ""
echo "==========================================="
if [ $ISSUES_FOUND -eq 0 ]; then
    echo -e "${GREEN}✓ All checks passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Found $ISSUES_FOUND configuration issues${NC}"
    echo "Please fix these issues before deploying to production"
    exit 1
fi