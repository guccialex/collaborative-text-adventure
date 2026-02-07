use gloo_net::http::Request;
use serde::Deserialize;

const APP_ID: &str = "61527:TKbPOk1F";
const GATEWAY_URL: &str = "https://newgrounds.io/gateway_v3.php";

#[derive(Deserialize)]
struct GatewayResponse {
    result: GatewayResult,
}

#[derive(Deserialize)]
struct GatewayResult {
    data: GatewayData,
}

#[derive(Deserialize)]
struct GatewayData {
    session: Option<SessionData>,
}

#[derive(Deserialize)]
struct SessionData {
    user: Option<NgUser>,
}

#[derive(Deserialize, Clone)]
pub struct NgUser {
    pub name: String,
}

pub fn get_session_id() -> Option<String> {
    let window = web_sys::window()?;
    let search = window.location().search().ok()?;
    search
        .trim_start_matches('?')
        .split('&')
        .find_map(|pair| {
            let (key, value) = pair.split_once('=')?;
            if key == "ngio_session_id" {
                Some(value.to_string())
            } else {
                None
            }
        })
}

pub async fn check_session(session_id: &str) -> Result<Option<NgUser>, String> {
    let payload = serde_json::json!({
        "app_id": APP_ID,
        "session_id": session_id,
        "call": {
            "component": "App.checkSession",
            "parameters": {}
        }
    });

    let resp = Request::post(GATEWAY_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(format!("input={}", payload.to_string()))
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data: GatewayResponse = resp.json().await.map_err(|e| e.to_string())?;
    Ok(data.result.data.session.and_then(|s| s.user))
}
