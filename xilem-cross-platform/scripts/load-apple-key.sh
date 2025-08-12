#!/bin/bash

# Script to load Apple Sign In private key for development
# Place your .p8 file in ~/dev/ directory

echo "Loading Apple Sign In private key..."

# Find .p8 files in ~/dev/
P8_FILES=(~/dev/*.p8)

if [ ! -f "${P8_FILES[0]}" ]; then
    echo "Error: No .p8 files found in ~/dev/"
    echo "Please download your Sign in with Apple key from Apple Developer Console"
    echo "and save it to ~/dev/ directory"
    exit 1
fi

# If multiple .p8 files, let user choose
if [ ${#P8_FILES[@]} -gt 1 ]; then
    echo "Multiple .p8 files found:"
    for i in "${!P8_FILES[@]}"; do
        echo "  $((i+1)). ${P8_FILES[$i]}"
    done
    read -p "Select file number: " FILE_NUM
    P8_FILE="${P8_FILES[$((FILE_NUM-1))]}"
else
    P8_FILE="${P8_FILES[0]}"
fi

echo "Using key file: $P8_FILE"

# Extract key ID from filename (usually AuthKey_XXXXXX.p8)
KEY_ID=$(basename "$P8_FILE" | sed -n 's/.*_\([A-Z0-9]*\)\.p8/\1/p')

if [ -z "$KEY_ID" ]; then
    read -p "Enter your Apple Key ID: " KEY_ID
fi

# Read the private key content
PRIVATE_KEY=$(cat "$P8_FILE")

# Create or update .env file
ENV_FILE="../web-backend/.env"

if [ -f "$ENV_FILE" ]; then
    echo "Updating existing .env file..."
    # Backup existing file
    cp "$ENV_FILE" "$ENV_FILE.bak"
else
    echo "Creating new .env file from template..."
    cp "../web-backend/.env.example" "$ENV_FILE"
fi

# Update the .env file with the key
# First, escape the private key for sed
ESCAPED_KEY=$(echo "$PRIVATE_KEY" | sed ':a;N;$!ba;s/\n/\\n/g' | sed 's/[[\.*^$()+?{|]/\\&/g')

# Update or add APPLE_KEY_ID
if grep -q "^APPLE_KEY_ID=" "$ENV_FILE"; then
    sed -i '' "s/^APPLE_KEY_ID=.*/APPLE_KEY_ID=$KEY_ID/" "$ENV_FILE"
else
    echo "APPLE_KEY_ID=$KEY_ID" >> "$ENV_FILE"
fi

# Update or add APPLE_PRIVATE_KEY
if grep -q "^APPLE_PRIVATE_KEY=" "$ENV_FILE"; then
    # Remove old APPLE_PRIVATE_KEY line(s)
    sed -i '' '/^APPLE_PRIVATE_KEY=/d' "$ENV_FILE"
fi

# Add the new private key
echo "APPLE_PRIVATE_KEY=\"$PRIVATE_KEY\"" >> "$ENV_FILE"

echo "✅ Apple Sign In key loaded successfully!"
echo "   Key ID: $KEY_ID"
echo "   Environment file: $ENV_FILE"
echo ""
echo "Next steps:"
echo "1. Verify other environment variables in $ENV_FILE"
echo "2. Start your backend server"
echo "3. The key will be used for generating client secrets"