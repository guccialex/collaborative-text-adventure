pub mod adventure;
pub mod counter;

use js_sys::Uint8Array;
use shared::ServerMessage;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use crate::config::API_BASE;

pub async fn api_fetch(msg: ServerMessage) -> Result<ServerMessage, String> {
    let bytes = bincode::serialize(&msg)
        .map_err(|e| format!("Serialize error: {}", e))?;

    let body = Uint8Array::from(bytes.as_slice());

    let headers = web_sys::Headers::new()
        .map_err(|e| format!("Headers error: {:?}", e))?;
    headers
        .set("Content-Type", "application/octet-stream")
        .map_err(|e| format!("Header set error: {:?}", e))?;

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&body.into());
    opts.set_headers(&headers.into());

    let url = format!("{}/api", API_BASE);
    let request = web_sys::Request::new_with_str_and_init(&url, &opts)
        .map_err(|e| format!("Request error: {:?}", e))?;

    let window = web_sys::window().ok_or("No window object")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;

    let resp: web_sys::Response = resp_value
        .dyn_into()
        .map_err(|_| "Response cast failed".to_string())?;

    if !resp.ok() {
        return Err(format!("API error: status {}", resp.status()));
    }

    let buffer = JsFuture::from(
        resp.array_buffer()
            .map_err(|e| format!("ArrayBuffer error: {:?}", e))?,
    )
    .await
    .map_err(|e| format!("Buffer error: {:?}", e))?;

    let uint8 = Uint8Array::new(&buffer);
    let response_bytes = uint8.to_vec();

    bincode::deserialize(&response_bytes)
        .map_err(|e| format!("Deserialize error: {}", e))
}
