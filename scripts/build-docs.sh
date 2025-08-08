#!/bin/bash
set -e

echo "Building documentation sites..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Build technical documentation
echo -e "${BLUE}Building technical documentation...${NC}"
if [ -d "alphabet-terminal-prototype/book" ]; then
    cd alphabet-terminal-prototype/book
    mdbook build
    cd ../..
    echo -e "${GREEN}✓ Technical documentation built${NC}"
else
    echo -e "${RED}✗ Technical documentation directory not found${NC}"
fi

# Build user manual
echo -e "${BLUE}Building user manual...${NC}"
if [ -d "xilem-cross-platform/user-manual" ]; then
    cd xilem-cross-platform/user-manual
    mdbook build
    cd ../..
    echo -e "${GREEN}✓ User manual built${NC}"
else
    echo -e "${RED}✗ User manual directory not found${NC}"
fi

# Create combined output with landing page
echo -e "${BLUE}Creating combined documentation site...${NC}"
rm -rf docs-output
mkdir -p docs-output

# Copy built books
if [ -d "alphabet-terminal-prototype/book/build" ]; then
    cp -r alphabet-terminal-prototype/book/build docs-output/technical
fi

if [ -d "xilem-cross-platform/user-manual/book" ]; then
    cp -r xilem-cross-platform/user-manual/book docs-output/user-manual
fi

# Create landing page
cat > docs-output/index.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Graph Learning Documentation</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 900px;
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
        h1 {
            color: #2d3748;
            border-bottom: 3px solid #667eea;
            padding-bottom: 1rem;
            margin-bottom: 2rem;
        }
        .books {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 2rem;
            margin-top: 2rem;
        }
        .book-card {
            border: 1px solid #e2e8f0;
            border-radius: 8px;
            padding: 1.5rem;
            transition: transform 0.2s, box-shadow 0.2s;
            background: #f7fafc;
        }
        .book-card:hover {
            transform: translateY(-4px);
            box-shadow: 0 10px 20px rgba(0,0,0,0.1);
            background: white;
        }
        .book-card h2 {
            color: #667eea;
            margin-top: 0;
        }
        .book-card a {
            display: inline-block;
            margin-top: 1rem;
            padding: 0.5rem 1rem;
            background: #667eea;
            color: white;
            text-decoration: none;
            border-radius: 4px;
            transition: background 0.2s;
        }
        .book-card a:hover {
            background: #5a67d8;
        }
        .description {
            color: #4a5568;
            margin: 1rem 0;
        }
        .overview {
            background: #edf2f7;
            padding: 1.5rem;
            border-radius: 8px;
            margin-bottom: 2rem;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>📚 Graph Learning System Documentation</h1>
        
        <div class="overview">
            <h2>About the Project</h2>
            <p>
                The Graph Learning System is an adaptive cognitive modeling platform that uses
                graph-coded representations for efficient learning. It combines cutting-edge
                research in cognitive science with practical applications in education and
                skill acquisition.
            </p>
            <p>
                This documentation hub provides comprehensive guides for both users and developers
                working with the system.
            </p>
        </div>

        <div class="books">
            <div class="book-card">
                <h2>📖 User Manual</h2>
                <p class="description">
                    Complete guide for using the Graph Learning application. Covers installation,
                    getting started, training sessions, and understanding your learning metrics.
                </p>
                <ul>
                    <li>Getting started guide</li>
                    <li>Training with different domains</li>
                    <li>Understanding metrics and progress</li>
                    <li>Tips for effective learning</li>
                </ul>
                <a href="user-manual/">Open User Manual →</a>
            </div>

            <div class="book-card">
                <h2>🔬 Technical Documentation</h2>
                <p class="description">
                    In-depth technical documentation for researchers and developers. Includes
                    mathematical foundations, API references, and implementation details.
                </p>
                <ul>
                    <li>Mathematical foundations</li>
                    <li>Algorithm descriptions</li>
                    <li>API reference</li>
                    <li>Statistical validation</li>
                </ul>
                <a href="technical/">Open Technical Docs →</a>
            </div>
        </div>

        <div style="margin-top: 3rem; padding-top: 2rem; border-top: 1px solid #e2e8f0; color: #718096; text-align: center;">
            <p>
                Built with mdBook | 
                <a href="https://github.com/yourusername/alphabet-terminal-prototype" style="color: #667eea;">View on GitHub</a>
            </p>
        </div>
    </div>
</body>
</html>
EOF

echo -e "${GREEN}✓ Documentation site created at docs-output/${NC}"
echo ""
echo "To view locally, run:"
echo "  cd docs-output && python3 -m http.server 8000"
echo "Then open http://localhost:8000 in your browser"