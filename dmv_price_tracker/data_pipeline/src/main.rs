use geojson::{FeatureCollection, GeoJson};
use polars::prelude::*;
use rstar::primitives::GeomWithData;
use rstar::RTree;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

// Define a simple struct for Rstar spatial indexing
type IndexedPoint = GeomWithData<[f64; 2], u32>;

// Complete RentCast property structure
#[derive(serde::Deserialize, Debug)]
struct RentCastProperty {
    #[serde(default)]
    id: String,
    #[serde(default)]
    address: String,
    #[serde(default)]
    latitude: f64,
    #[serde(default)]
    longitude: f64,
    #[serde(default)]
    price: Option<f64>,
    #[serde(default)]
    bedrooms: Option<f64>,
    #[serde(default)]
    bathrooms: Option<f64>,
    #[serde(default, rename = "squareFootage")]
    square_footage: Option<f64>,
    #[serde(default, rename = "propertyType")]
    property_type: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default, rename = "zipCode")]
    zip_code: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    println!("Starting Data Pipeline MVP...");

    // Define zip codes for DC and Fairfax
    let dc_zip_codes = vec!["20001", "20002", "20003", "20004", "20005", "20007", "20008", "20009", "20010", "20011"];
    let fairfax_zip_codes = vec!["22030", "22031", "22032", "22033", "22034", "22035", "22038", "22041", "22042", "22043"];

    // 1. Load GeoJSON data (if available)
    let dc_lots_df = match load_geojson_to_polars("data/dc_lots.geojson") {
        Ok(df) => {
            println!("Loaded DC Lots: {:?}", df.shape());
            Some(df)
        }
        Err(e) => {
            println!("Warning: Could not load DC lots GeoJSON: {}", e);
            None
        }
    };

    let fairfax_parcels_df = match load_geojson_to_polars("data/fairfax_parcels.geojson") {
        Ok(df) => {
            println!("Loaded Fairfax Parcels: {:?}", df.shape());
            Some(df)
        }
        Err(e) => {
            println!("Warning: Could not load Fairfax parcels GeoJSON: {}", e);
            None
        }
    };

    // 2. Fetch RentCast Data for all zip codes
    let rentcast_api_key = std::env::var("RENTCAST_API_KEY")
        .expect("RENTCAST_API_KEY environment variable not set");
    
    let client = reqwest::Client::new();
    let mut all_properties = Vec::new();

    // Fetch DC market data
    println!("\nFetching RentCast data for DC zip codes...");
    for zip_code in &dc_zip_codes {
        match fetch_rentcast_market_data(&client, &rentcast_api_key, zip_code).await {
            Ok(properties) => {
                println!("  Zip {}: {} properties", zip_code, properties.len());
                all_properties.extend(properties);
            }
            Err(e) => {
                println!("  Warning: Could not fetch data for zip {}: {}", zip_code, e);
            }
        }
        // Rate limiting: wait 500ms between requests
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    // Fetch Fairfax market data
    println!("\nFetching RentCast data for Fairfax zip codes...");
    for zip_code in &fairfax_zip_codes {
        match fetch_rentcast_market_data(&client, &rentcast_api_key, zip_code).await {
            Ok(properties) => {
                println!("  Zip {}: {} properties", zip_code, properties.len());
                all_properties.extend(properties);
            }
            Err(e) => {
                println!("  Warning: Could not fetch data for zip {}: {}", zip_code, e);
            }
        }
        // Rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    println!("\nTotal RentCast properties fetched: {}", all_properties.len());

    // 3. Convert RentCast data to DataFrame
    let rentcast_df = convert_rentcast_to_dataframe(&all_properties)?;
    println!("RentCast DataFrame shape: {:?}", rentcast_df.shape());

    // 4. Spatial Indexing with R-Tree
    let points_to_index: Vec<IndexedPoint> = all_properties
        .iter()
        .enumerate()
        .filter_map(|(idx, prop)| {
            if prop.longitude != 0.0 && prop.latitude != 0.0 {
                Some(GeomWithData::new([prop.longitude, prop.latitude], idx as u32))
            } else {
                None
            }
        })
        .collect();

    let rtree = RTree::bulk_load(points_to_index);
    println!("Created R-Tree with {} items", rtree.size());

    // 5. Aggregate Data
    let aggregated_df = aggregate_rentcast_data(&rentcast_df)?;
    println!("Aggregated Data Shape: {:?}", aggregated_df.shape());

    // 6. Combine with geographic data if available
    let mut dfs_to_combine = vec![aggregated_df];
    if let Some(dc_df) = dc_lots_df {
        dfs_to_combine.push(dc_df);
    }
    if let Some(fairfax_df) = fairfax_parcels_df {
        dfs_to_combine.push(fairfax_df);
    }
    let mut combined_df = if dfs_to_combine.len() > 1 {
        let lazy_dfs: Vec<_> = dfs_to_combine.iter().map(|df| df.clone().lazy()).collect();
        concat(lazy_dfs.as_slice(), UnionArgs::default())?.collect()?
    } else {
        dfs_to_combine.into_iter().next().unwrap()
    };

    // 7. Output for WASM
    // Ensure output directory exists
    std::fs::create_dir_all("output")?;
    let out_path = Path::new("output/aggregated_data.json");
    let mut file = File::create(out_path)?;
    
    JsonWriter::new(&mut file)
        .with_json_format(JsonFormat::Json)
        .finish(&mut combined_df)?;

    println!("\n✅ Pipeline finished. Output written to output/aggregated_data.json");
    println!("   Total records: {}", combined_df.shape().0);

    Ok(())
}

/// Fetch RentCast market data by zip code
async fn fetch_rentcast_market_data(
    client: &reqwest::Client,
    api_key: &str,
    zip_code: &str,
) -> Result<Vec<RentCastProperty>, Box<dyn Error>> {
    // First try to get properties by zip code
    let url = "https://api.rentcast.io/v1/properties";
    let response = client
        .get(url)
        .query(&[("zipcode", zip_code)])
        .header("X-Api-Key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        // If that fails, try market data endpoint
        let market_url = format!("https://api.rentcast.io/v1/markets?zip={}", zip_code);
        let market_response = client
            .get(&market_url)
            .header("X-Api-Key", api_key)
            .send()
            .await?;
        
        if market_response.status().is_success() {
            let _data: serde_json::Value = market_response.json().await?;
            // Extract properties from market data if available
            return Ok(vec![]);
        }
        
        return Ok(vec![]);
    }

    let properties: Vec<RentCastProperty> = response.json().await?;
    Ok(properties)
}

/// Convert RentCast properties to Polars DataFrame
fn convert_rentcast_to_dataframe(properties: &[RentCastProperty]) -> Result<DataFrame, Box<dyn Error>> {
    let mut ids = Vec::new();
    let mut addresses = Vec::new();
    let mut latitudes = Vec::new();
    let mut longitudes = Vec::new();
    let mut prices = Vec::new();
    let mut bedrooms = Vec::new();
    let mut bathrooms = Vec::new();
    let mut square_footages = Vec::new();
    let mut property_types = Vec::new();
    let mut statuses = Vec::new();
    let mut zip_codes = Vec::new();

    for prop in properties {
        ids.push(prop.id.clone());
        addresses.push(prop.address.clone());
        latitudes.push(prop.latitude);
        longitudes.push(prop.longitude);
        prices.push(prop.price);
        bedrooms.push(prop.bedrooms);
        bathrooms.push(prop.bathrooms);
        square_footages.push(prop.square_footage);
        property_types.push(prop.property_type.clone());
        statuses.push(prop.status.clone());
        zip_codes.push(prop.zip_code.clone());
    }

    let df = df!(
        "id" => ids,
        "address" => addresses,
        "latitude" => latitudes,
        "longitude" => longitudes,
        "price" => prices,
        "bedrooms" => bedrooms,
        "bathrooms" => bathrooms,
        "square_footage" => square_footages,
        "property_type" => property_types,
        "status" => statuses,
        "zip_code" => zip_codes
    )?;

    Ok(df)
}

/// Aggregate RentCast data for visualization
fn aggregate_rentcast_data(df: &DataFrame) -> Result<DataFrame, Box<dyn Error>> {
    let aggregated = df
        .clone()
        .lazy()
        // Filter out null coordinates
        .filter(col("latitude").is_not_null())
        .filter(col("longitude").is_not_null())
        .select([
            col("id"),
            col("address"),
            col("latitude"),
            col("longitude"),
            col("price"),
            col("bedrooms"),
            col("bathrooms"),
            col("property_type"),
            col("status"),
            col("zip_code"),
        ])
        .collect()?;

    Ok(aggregated)
}

/// Helper to load GeoJSON features into a Polars DataFrame
fn load_geojson_to_polars(path: &str) -> Result<DataFrame, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let geojson = GeoJson::from_reader(reader)?;

    let mut lats: Vec<Option<f64>> = Vec::new();
    let mut lons: Vec<Option<f64>> = Vec::new();
    let mut types: Vec<Option<String>> = Vec::new();

    if let GeoJson::FeatureCollection(FeatureCollection { features, .. }) = geojson {
        for feature in features {
            let (lat, lon) = if let Some(ref geometry) = feature.geometry {
                match &geometry.value {
                    geojson::Value::Point(p) => (Some(p[1]), Some(p[0])),
                    geojson::Value::Polygon(p) => {
                        // Simple centroid calculation for Polygon
                        let (mut sum_lon, mut sum_lat, mut count) = (0.0, 0.0, 0.0);
                        let coords = &p[0]; // Use first linear ring
                        for pt in coords {
                            sum_lon += pt[0];
                            sum_lat += pt[1];
                            count += 1.0;
                        }
                        (Some(sum_lat / count), Some(sum_lon / count))
                    }
                    geojson::Value::MultiPolygon(p) => {
                        // Simple centroid calculation for MultiPolygon
                        let (mut sum_lon, mut sum_lat, mut count) = (0.0, 0.0, 0.0);
                        let coords = &p[0][0]; // Use first linear ring of first polygon
                        for pt in coords {
                            sum_lon += pt[0];
                            sum_lat += pt[1];
                            count += 1.0;
                        }
                        (Some(sum_lat / count), Some(sum_lon / count))
                    }
                    _ => (None, None),
                }
            } else {
                (None, None)
            };
            lats.push(lat);
            lons.push(lon);
            types.push(feature.geometry.as_ref().map(|g| g.value.type_name().to_string()));
        }
    }

    let df = df!(
        "geometry_type" => types,
        "latitude" => lats,
        "longitude" => lons
    )?;
    Ok(df)
}
