# Data Pipeline

This pipeline fetches rental property data from RentCast for the DC and Fairfax areas.

## Setup

1. Get a RentCast API key from https://developers.rentcast.io/

2. Create a `.env` file in the `data_pipeline` directory with your API key:

   **Option A - Using the example file:**

   ```bash
   cd data_pipeline
   cp env.example .env
   # Then edit .env with your actual API key
   ```

   **Option B - Create manually:**
   Create a `.env` file in the `data_pipeline/` directory with this content:

   ```
   RENTCAST_API_KEY=your_actual_api_key_here
   ```

   **Note:** The `.env` file is already in `.gitignore` and won't be committed to version control.

3. (Optional) Add GeoJSON files to `data/` directory:
   - `data/dc_lots.geojson` - DC lot data
   - `data/fairfax_parcels.geojson` - Fairfax County parcel data

## Running

```bash
# From the dmv_price_tracker directory
cd dmv_price_tracker
cargo run --bin data_pipeline
```

Make sure your `.env` file is in the `data_pipeline/` directory before running.

## Output

The pipeline outputs `output/aggregated_data.json` which contains:

- Property data from RentCast (address, price, bedrooms, bathrooms, etc.)
- Geographic coordinates for each property
- Spatial indexing using R-Tree
- Aggregated data ready for visualization

## API Rate Limiting

The pipeline includes 500ms delays between requests to respect RentCast API rate limits.
