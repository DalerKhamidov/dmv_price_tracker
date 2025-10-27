use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    Ok(())
}

#[wasm_bindgen]
pub async fn load_and_render_map() -> Result<JsValue, JsValue> {
    // 1. Fetch processed data from the output directory
    let data_url = "./output/aggregated_data.json";
    let opts = {
        let mut o = RequestInit::new();
        o.set_method("GET");
        o.set_mode(RequestMode::Cors);
        o
    };

    let request = Request::new_with_str_and_init(data_url, &opts)?;
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    
    // Check if response is ok
    if !resp.ok() {
        return Err(JsValue::from_str(&format!(
            "Failed to fetch data: status {}",
            resp.status()
        )));
    }

    let text = JsFuture::from(resp.text()?).await?.as_string()
        .ok_or_else(|| JsValue::from_str("Failed to get response text"))?;

    // Return the JSON string directly to JavaScript
    Ok(JsValue::from_str(&text))
}
