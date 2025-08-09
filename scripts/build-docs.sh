#!/bin/bash

# Build all documentation manuals and create unified site
set -e

echo "🏗️  Building Graph Learning System Documentation"
echo "=================================================="

# Check if mdbook is installed
if ! command -v mdbook &> /dev/null; then
    echo "❌ mdbook is not installed. Install it with:"
    echo "   cargo install mdbook"
    exit 1
fi

# Create output directory
OUTPUT_DIR="docs-site"
rm -rf $OUTPUT_DIR
mkdir -p $OUTPUT_DIR

echo "📖 Building User Manual..."
cd user-manual && mdbook build && mv book ../$OUTPUT_DIR/user-manual && cd ..

echo "⚙️  Building Admin Manual..."
cd admin-manual && mdbook build && mv book ../$OUTPUT_DIR/admin-manual && cd ..

echo "💻 Building Developer Manual..."
cd dev-manual && mdbook build && mv book ../$OUTPUT_DIR/dev-manual && cd ..

echo "🌐 Creating unified documentation site..."

# Create main index.html
cat > $OUTPUT_DIR/index.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Graph Learning System Documentation</title>
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
        h1 {
            color: #2c3e50;
            text-align: center;
            margin-bottom: 3rem;
            font-size: 2.5rem;
        }
        .docs-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 2rem;
            margin: 3rem 0;
        }
        .doc-card {
            background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
            border-radius: 8px;
            padding: 2rem;
            text-decoration: none;
            color: #2c3e50;
            transition: transform 0.3s ease;
        }
        .doc-card:hover {
            transform: translateY(-5px);
            text-decoration: none;
            color: #2c3e50;
        }
        .oauth-badge {
            background: linear-gradient(45deg, #4CAF50, #45a049);
            color: white;
            padding: 0.5rem 1rem;
            border-radius: 20px;
            margin: 1rem 0;
            display: inline-block;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>📚 Graph Learning System Documentation</h1>
        
        <div class="oauth-badge">✨ Now with Apple Sign In & GitHub OAuth!</div>
        
        <div style="text-align: center; margin: 2rem 0;">
            <a href="./app/" style="background: linear-gradient(45deg, #667eea, #764ba2); color: white; padding: 1rem 2rem; border-radius: 50px; text-decoration: none; font-weight: bold; display: inline-block; font-size: 1.1rem; box-shadow: 0 10px 25px rgba(102, 126, 234, 0.3);">🚀 Launch App</a>
        </div>
        
        <div class="docs-grid">
            <a href="./user-manual/" class="doc-card">
                <h2>👤 User Manual</h2>
                <p>Learn how to use the system, create accounts with Apple Sign In, and track your learning progress.</p>
            </a>
            
            <a href="./admin-manual/" class="doc-card">
                <h2>⚙️ Administrator Manual</h2>
                <p>Deploy and maintain the system with OAuth authentication in production environments.</p>
            </a>
            
            <a href="./dev-manual/" class="doc-card">
                <h2>💻 Developer Manual</h2>
                <p>Technical documentation for OAuth implementation, system architecture, and development.</p>
            </a>
        </div>
    </div>
</body>
</html>
EOF

echo "✅ Documentation build complete!"
echo "📁 Output: $OUTPUT_DIR"
echo "🚀 Ready for deployment!"