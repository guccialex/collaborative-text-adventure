use gloo_net::http::Request;
use serde::Deserialize;

use crate::config::API_BASE;

#[derive(Deserialize, Clone)]
struct CounterResponse {
    value: u64,
}

pub async fn fetch_counter() -> Result<u64, String> {
    let resp = Request::get(&format!("{}/api/counter", API_BASE))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data: CounterResponse = resp.json().await.map_err(|e| e.to_string())?;
    Ok(data.value)
}

pub async fn increment_counter() -> Result<u64, String> {
    let resp = Request::post(&format!("{}/api/counter/increment", API_BASE))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data: CounterResponse = resp.json().await.map_err(|e| e.to_string())?;
    Ok(data.value)
}
