#!/bin/bash

# Build script for ABCDeez Core mdBook documentation

set -e

echo "Building ABCDeez Core documentation..."

# Check if mdbook is installed
if ! command -v mdbook &> /dev/null; then
    echo "mdBook not found. Installing..."
    cargo install mdbook
fi

# Check for mdbook-mermaid if using mermaid diagrams
if ! command -v mdbook-mermaid &> /dev/null; then
    echo "mdbook-mermaid not found. Installing for diagram support..."
    cargo install mdbook-mermaid
fi

# Clean previous build
rm -rf book/

# Build the book
mdbook build

echo "Documentation built successfully!"
echo "Open book/index.html to view the documentation"

# Optional: serve the book locally
read -p "Would you like to serve the book locally? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Serving at http://localhost:3000"
    mdbook serve --open
fi