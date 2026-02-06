/// Set at build time via `API_BASE=https://1.2.3.4.nip.io trunk build --release`
/// Falls back to localhost for local development.
pub const API_BASE: &str = match option_env!("API_BASE") {
    Some(url) => url,
    None => "http://localhost:8080",
};
