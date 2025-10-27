#!/bin/bash
# Simple serve script for WASM viewer

echo "Setting up WASM viewer..."
echo "========================"
echo ""

# Check if config.js exists and setup env if needed
if [ ! -f "www/config.js" ]; then
    echo "config.js not found, running setup..."
    ./setup-env.sh
fi

# Check if pkg exists in www
if [ ! -d "www/pkg" ]; then
    echo "Copying pkg/ to www/..."
    cp -r pkg www/
    echo "✓ Package copied"
fi

# Check if output directory exists in www
if [ ! -d "www/output" ]; then
    echo "Copying output/ to www/..."
    cp -r ../data_pipeline/output www/
    echo "✓ Output data copied"
fi

# Check if output exists in data_pipeline
if [ ! -f "../data_pipeline/output/aggregated_data.json" ]; then
    echo ""
    echo "⚠️  WARNING: aggregated_data.json not found!"
    echo "   Please run the data pipeline first:"
    echo "   cd ../data_pipeline && cargo run --bin data_pipeline"
    echo ""
fi

echo ""
echo "Starting web server..."
echo "Open http://localhost:8000 in your browser"
echo ""

cd www
python -m http.server 8000

