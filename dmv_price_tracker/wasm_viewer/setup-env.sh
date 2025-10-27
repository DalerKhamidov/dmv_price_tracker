#!/bin/bash
# Setup script to copy Mapbox token from .env to config.js

echo "Setting up Mapbox token from .env..."
echo "================================"
echo ""

# Check if .env file exists
if [ ! -f ".env" ]; then
    echo "❌ .env file not found!"
    echo ""
    echo "Creating .env from example..."
    cp .env.example .env
    echo "✓ .env file created"
    echo ""
    echo "⚠️  Please edit .env and add your Mapbox token"
    echo "   Get a token from: https://www.mapbox.com/"
    echo ""
    echo "Then run this script again to generate config.js"
    exit 1
fi

# Source .env file
if [ -f ".env" ]; then
    # Read MAPBOX_TOKEN from .env
    export $(grep -v '^#' .env | grep MAPBOX_TOKEN | xargs)
    
    if [ -z "$MAPBOX_TOKEN" ] || [ "$MAPBOX_TOKEN" = "YOUR_MAPBOX_ACCESS_TOKEN_HERE" ]; then
        echo "❌ MAPBOX_TOKEN not set in .env file"
        echo ""
        echo "Please edit .env and set MAPBOX_TOKEN=your_token_here"
        exit 1
    fi
    
    # Create config.js with the token
    cat > www/config.js << EOF
// This file is auto-generated from .env
// Do not edit manually - run setup-env.sh instead
const MAPBOX_ACCESS_TOKEN = '$MAPBOX_TOKEN';
EOF
    
    echo "✓ config.js generated with Mapbox token"
    echo ""
    echo "Token loaded from .env file"
else
    echo "❌ Could not read .env file"
    exit 1
fi

