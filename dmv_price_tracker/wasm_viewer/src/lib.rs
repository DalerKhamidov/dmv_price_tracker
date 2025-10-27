use plotlars::charts::{Chart, DensityMapbox};
use plotlars::specs::{DensityMapboxSpec, MapboxStyle};
use polars::prelude::*;
use serde_json::Value;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    Ok(())
}

#[wasm_bindgen]
pub async fn load_and_render_map() -> Result<String, JsValue> {
    // 1. Fetch processed data from the output directory
    let data_url = "output/aggregated_data.json";
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(data_url, &opts)?;
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    let text = JsFuture::from(resp.text()?).await?.as_string().unwrap();

    // 2. Load data into Polars DataFrame
    let file = std::io::Cursor::new(text);
    let df = JsonReader::new(file)
        .with_json_format(JsonFormat::JsonLines)
        .finish()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // 3. Create Density Mapbox spec using Plotlars
    let map_spec = DensityMapboxSpec {
        data: df,
        lat: "latitude".to_string(),
        lon: "longitude".to_string(),
        z: None, // Use count aggregation
        radius: Some(10),
        mapbox_style: MapboxStyle::Dark,
        center_lat: 38.9, // Center on DC/NoVA
        center_lon: -77.1,
        zoom: 9.0,
        title: "DC/NoVA Property Density".to_string(),
        ..Default::default()
    };

    // 4. Generate the Vega-Lite JSON specification
    let chart = DensityMapbox::new(map_spec);
    let vega_spec = chart
        .to_json()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(vega_spec)
}
