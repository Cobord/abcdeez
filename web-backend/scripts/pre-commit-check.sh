#!/bin/bash

# Pre-commit hook to check for secrets and sensitive information
# Install this by running: cp scripts/pre-commit-check.sh .git/hooks/pre-commit

echo "Running pre-commit security checks..."

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

FOUND_ISSUES=0

# Patterns to check for potential secrets
SECRET_PATTERNS=(
    "password.*=.*['\"].*['\"]"
    "api[_-]?key.*=.*['\"].*['\"]"
    "secret.*=.*['\"].*['\"]"
    "token.*=.*['\"].*['\"]"
    "placeholder_.*client"
    "placeholder_.*secret"
    "test.*secret"
    "CHANGE_ME"
    "change_me"
    "BEGIN RSA PRIVATE KEY"
    "BEGIN PRIVATE KEY"
    "BEGIN CERTIFICATE"
    "aws_access_key_id"
    "aws_secret_access_key"
)

# Files to check (staged files only)
FILES=$(git diff --cached --name-only --diff-filter=ACM)

for file in $FILES; do
    # Skip binary files
    if file "$file" | grep -q "binary"; then
        continue
    fi
    
    # Skip .env.template and documentation
    if [[ "$file" == *.template ]] || [[ "$file" == *.md ]]; then
        continue
    fi
    
    # Check for .env files
    if [[ "$file" == *.env* ]] && [[ "$file" != *.template ]]; then
        echo -e "${RED}✗ Attempting to commit environment file: $file${NC}"
        echo "  Environment files should never be committed"
        FOUND_ISSUES=$((FOUND_ISSUES + 1))
    fi
    
    # Check for private keys
    if [[ "$file" == *.pem ]] || [[ "$file" == *.p8 ]] || [[ "$file" == *.key ]]; then
        echo -e "${RED}✗ Attempting to commit private key file: $file${NC}"
        FOUND_ISSUES=$((FOUND_ISSUES + 1))
    fi
    
    # Check for database files
    if [[ "$file" == *.db ]] || [[ "$file" == *.sqlite* ]]; then
        echo -e "${RED}✗ Attempting to commit database file: $file${NC}"
        FOUND_ISSUES=$((FOUND_ISSUES + 1))
    fi
    
    # Check file contents for secrets
    for pattern in "${SECRET_PATTERNS[@]}"; do
        if grep -iE "$pattern" "$file" > /dev/null 2>&1; then
            echo -e "${YELLOW}⚠ Potential secret found in $file${NC}"
            echo "  Pattern matched: $pattern"
            echo "  Please review and ensure no real secrets are being committed"
            grep -n -iE "$pattern" "$file" | head -3
            FOUND_ISSUES=$((FOUND_ISSUES + 1))
        fi
    done
done

# Check for large files (potential data dumps)
for file in $FILES; do
    if [ -f "$file" ]; then
        size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null)
        if [ "$size" -gt 1048576 ]; then  # 1MB
            echo -e "${YELLOW}⚠ Large file detected: $file ($(($size/1024))KB)${NC}"
            echo "  Large files might contain data dumps or logs with sensitive information"
        fi
    fi
done

# Summary
echo ""
if [ $FOUND_ISSUES -eq 0 ]; then
    echo -e "${GREEN}✓ Pre-commit security checks passed${NC}"
    exit 0
else
    echo -e "${RED}✗ Found $FOUND_ISSUES potential security issues${NC}"
    echo ""
    echo "If you're sure these are false positives, you can:"
    echo "  1. Review and fix the issues"
    echo "  2. Use 'git commit --no-verify' to skip this check (NOT RECOMMENDED)"
    echo ""
    echo "For actual secrets, use environment variables or a secrets management system."
    exit 1
fi