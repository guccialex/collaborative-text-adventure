use leptos::prelude::*;
use leptos::task::spawn_local;
use gloo_net::http::Request;
use serde::Deserialize;

const API_BASE: &str = "http://localhost:8080";

#[derive(Deserialize, Clone)]
struct CounterResponse {
    value: u64,
}

async fn fetch_counter() -> Result<u64, String> {
    let resp = Request::get(&format!("{}/api/counter", API_BASE))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data: CounterResponse = resp.json().await.map_err(|e| e.to_string())?;
    Ok(data.value)
}

async fn increment_counter() -> Result<u64, String> {
    let resp = Request::post(&format!("{}/api/counter/increment", API_BASE))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data: CounterResponse = resp.json().await.map_err(|e| e.to_string())?;
    Ok(data.value)
}

#[component]
pub fn ServerCounter() -> impl IntoView {
    let (count, set_count) = signal::<Option<u64>>(None);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<String>>(None);

    // Fetch counter on mount
    Effect::new(move |_| {
        spawn_local(async move {
            match fetch_counter().await {
                Ok(value) => {
                    set_count.set(Some(value));
                    set_loading.set(false);
                }
                Err(e) => {
                    log::error!("Failed to fetch counter: {}", e);
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    });

    let on_increment = move |_| {
        set_loading.set(true);
        spawn_local(async move {
            match increment_counter().await {
                Ok(value) => {
                    set_count.set(Some(value));
                    set_loading.set(false);
                }
                Err(e) => {
                    log::error!("Failed to increment counter: {}", e);
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="server-counter">
            <Show
                when=move || error.get().is_some()
                fallback=move || {
                    view! {
                        <span class="counter-value">
                            {move || {
                                if loading.get() {
                                    "...".to_string()
                                } else {
                                    count.get().map(|v| v.to_string()).unwrap_or("?".to_string())
                                }
                            }}
                        </span>
                        <button class="counter-btn" on:click=on_increment disabled=move || loading.get()>
                            "+"
                        </button>
                    }
                }
            >
                <span class="counter-error" title=move || error.get().unwrap_or_default()>
                    "Server offline"
                </span>
            </Show>
        </div>
    }
}
