#!/bin/bash
set -e

echo "📁 Creating docs directory structure..."
mkdir -p docs/assets/

echo "🔨 Generating OpenAPI specification..."
cargo run --bin generate_openapi

echo "📦 Downloading Swagger UI..."
# Download latest Swagger UI
SWAGGER_VERSION="5.11.0"
curl -L "https://github.com/swagger-api/swagger-ui/archive/v${SWAGGER_VERSION}.tar.gz" | tar -xz

# Move Swagger UI assets to assets subdirectory
cp -r "swagger-ui-${SWAGGER_VERSION}/dist/"* docs/assets/

# Clean up
rm -rf "swagger-ui-${SWAGGER_VERSION}"

echo "🔧 Configuring Swagger UI..."
# Update the HTML to point to our OpenAPI spec
sed -i.bak 's|https://petstore.swagger.io/v2/swagger.json|./openapi.json|g' docs/assets/swagger-initializer.js
rm docs/assets/swagger-initializer.js.bak

echo "📝 Creating index.html..."
cat > docs/index.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>LEADR API Documentation</title>
    <link rel="stylesheet" type="text/css" href="./assets/swagger-ui.css" />
    <link rel="stylesheet" type="text/css" href="./assets/index.css" />
    <link rel="icon" type="image/png" href="./assets/favicon-32x32.png" sizes="32x32" />
    <link rel="icon" type="image/png" href="./assets/favicon-16x16.png" sizes="16x16" />
    <style>
        html {
            box-sizing: border-box;
            overflow: -moz-scrollbars-vertical;
            overflow-y: scroll;
        }
        *, *:before, *:after {
            box-sizing: inherit;
        }
        body {
            margin:0;
            background: #fafafa;
        }
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="./assets/swagger-ui-bundle.js" charset="UTF-8"></script>
    <script src="./assets/swagger-ui-standalone-preset.js" charset="UTF-8"></script>
    <script>
        window.onload = function() {
            const ui = SwaggerUIBundle({
                url: './openapi.json',
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIStandalonePreset
                ],
                plugins: [
                    SwaggerUIBundle.plugins.DownloadUrl
                ],
                layout: "StandaloneLayout"
            });
        };
    </script>
</body>
</html>
EOF

echo "✅ Documentation generated successfully!"
echo ""
echo "📖 To view locally (due to CORS restrictions, you need a local server):"
echo "   Option 1: python3 -m http.server 8000 --directory docs"
echo "   Option 2: npx serve docs"
echo "   Then open: http://localhost:8000"
echo ""
echo "🌐 Will be available on GitHub Pages at: https://barneyjackson.github.io/leadr/"

# Offer to start a local server
read -p "🚀 Start local server now? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Starting server at http://localhost:8000"
    echo "Press Ctrl+C to stop"
    cd docs && python3 -m http.server 8000
fi
