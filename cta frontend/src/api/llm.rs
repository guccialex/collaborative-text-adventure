use serde::Serialize;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;

use crate::config::API_BASE;

#[derive(Serialize)]
struct LlmRequest {
    api_base_url: String,
    api_key: String,
    model: String,
    prompt: String,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

#[wasm_bindgen(inline_js = "
export function stream_llm_fetch(url, body_json, on_chunk) {
    return new Promise(async (resolve, reject) => {
        try {
            const resp = await fetch(url, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: body_json,
            });
            if (!resp.ok) {
                let msg = `HTTP ${resp.status}`;
                try {
                    const body = await resp.json();
                    if (body.error) msg = body.error;
                } catch {}
                reject(msg);
                return;
            }
            const reader = resp.body.getReader();
            const decoder = new TextDecoder();
            let buffer = '';
            while (true) {
                const { done, value } = await reader.read();
                if (done) break;
                buffer += decoder.decode(value, { stream: true });
                const lines = buffer.split('\\n');
                buffer = lines.pop();
                for (const line of lines) {
                    if (line.startsWith('data: ')) {
                        const data = line.slice(6).trim();
                        if (data === '[DONE]') continue;
                        try {
                            const parsed = JSON.parse(data);
                            const content = parsed.choices?.[0]?.delta?.content;
                            if (content) on_chunk(content);
                        } catch {}
                    }
                }
            }
            resolve();
        } catch (e) {
            reject(e.toString());
        }
    });
}
")]
extern "C" {
    fn stream_llm_fetch(
        url: &str,
        body_json: &str,
        on_chunk: &Closure<dyn FnMut(String)>,
    ) -> js_sys::Promise;
}

pub async fn call_llm_stream(
    api_base_url: &str,
    api_key: &str,
    model: &str,
    prompt: &str,
    on_chunk: impl Fn(&str) + 'static,
) -> Result<(), String> {
    let body = LlmRequest {
        api_base_url: api_base_url.to_string(),
        api_key: api_key.to_string(),
        model: model.to_string(),
        prompt: prompt.to_string(),
        max_tokens: Some(1024),
        temperature: Some(0.8),
    };

    let body_json = serde_json::to_string(&body)
        .map_err(|e| format!("Serialize error: {}", e))?;

    let url = format!("{}/api/llm", API_BASE);

    let closure = Closure::wrap(Box::new(move |chunk: String| {
        on_chunk(&chunk);
    }) as Box<dyn FnMut(String)>);

    let promise = stream_llm_fetch(&url, &body_json, &closure);
    closure.forget();

    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map(|_| ())
        .map_err(|e| e.as_string().unwrap_or_else(|| "Stream failed".to_string()))
}
