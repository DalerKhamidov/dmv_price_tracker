# WASM Viewer

Interactive map visualization for DC/Fairfax property data using WebAssembly and Mapbox.

## Setup

### 1. Get a Mapbox Access Token

1. Go to https://www.mapbox.com/
2. Sign up for a free account
3. Create an access token
4. Copy your token

### 2. Setup Your Token

**Option A - Using .env file (Recommended):**

```bash
# Copy the example .env file
cp .env.example .env

# Edit .env and add your Mapbox token:
# MAPBOX_TOKEN=pk.eyJ1IjoieW91cm5hbWUiLCJhIjoiY2...

# Generate config.js from .env
./setup-env.sh    # Linux/Mac
# or
setup-env.bat     # Windows
```

**Option B - Manual setup:**

Edit `www/config.js` directly and replace `YOUR_MAPBOX_ACCESS_TOKEN_HERE` with your actual token:

```javascript
const MAPBOX_ACCESS_TOKEN = "pk.eyJ1IjoieW91cm5hbWUiLCJhIjoiY2...";
```

### 3. Build WASM Package

```bash
cd wasm_viewer
wasm-pack build --target web
```

This creates the `pkg/` directory with compiled WASM files.

### 4. Run Data Pipeline First

Make sure the data pipeline has generated the aggregated data:

```bash
cd ../data_pipeline
# Set RENTCAST_API_KEY in .env file
cargo run --bin data_pipeline
```

This creates `output/aggregated_data.json` which the viewer needs.

### 5. Serve the Application

**Quick Start (Recommended):**

The serve script automatically:

- Generates `config.js` from `.env` if needed
- Copies WASM package to `www/`
- Copies data output to `www/`

```bash
# From wasm_viewer directory
./serve.sh    # Linux/Mac
# or
serve.bat     # Windows
```

Then open http://localhost:8000 in your browser.

**Manual Start:**

```bash
# Generate config from .env
./setup-env.sh

# Copy pkg to www (if not already done)
cp -r pkg www/

# Copy output data
cp -r ../data_pipeline/output www/

# Serve from www directory
cd www
python -m http.server 8000
```

## Features

- ✅ Interactive Mapbox GL map centered on DC
- ✅ Property markers showing rental properties
- ✅ Click markers to see property details (price, bedrooms, bathrooms)
- ✅ Dark theme for better visualization
- ✅ WebAssembly for efficient data loading

## Architecture

```
www/
├── index.html    # Main HTML page
├── main.js       # JavaScript that:
│                  - Loads WASM module
│                  - Fetches data via WASM
│                  - Creates Mapbox map
│                  - Renders property markers
└── style.css     # Styling

pkg/
└── wasm_viewer.js  # Compiled WASM module

src/
└── lib.rs          # Rust/WASM code
```

## Troubleshooting

**"Could not fetch data"**

- Make sure `data_pipeline` has been run and created `output/aggregated_data.json`
- The file should be at: `../data_pipeline/output/aggregated_data.json` (from www directory)
- If serving from `www/`, ensure the `output/` directory is copied there or accessible

**WASM file not found (404 on wasm_viewer.js)**

- Run `cp -r pkg www/` to copy the WASM package to www
- Or use the `serve.sh` or `serve.bat` script which does this automatically

**Map not rendering**

- Check your Mapbox token is set correctly
- Open browser console to see error messages
- Ensure Mapbox GL JS CDN loaded successfully

**WASM errors**

- Rebuild: `wasm-pack build --target web`
- Clear browser cache
- Check browser console for specific errors

## File Structure

The WASM module (`lib.rs`) provides:

- `load_and_render_map()` - Fetches and returns JSON data
- Called by JavaScript to get property data
- Simple JSON passthrough (JavaScript handles parsing)

JavaScript (`main.js`) handles:

- Initializing WASM module
- Fetching data from WASM
- Parsing JSON
- Creating Mapbox map
- Adding interactive markers
- Displaying popups
