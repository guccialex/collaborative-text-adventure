use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum LlmProvider {
    OpenAI,
    OpenRouter,
    DeepSeek,
    Groq,
    Custom,
}

impl LlmProvider {
    pub fn base_url(&self) -> &'static str {
        match self {
            Self::OpenAI => "https://api.openai.com/v1",
            Self::OpenRouter => "https://openrouter.ai/api/v1",
            Self::DeepSeek => "https://api.deepseek.com/v1",
            Self::Groq => "https://api.groq.com/openai/v1",
            Self::Custom => "",
        }
    }

    pub fn default_model(&self) -> &'static str {
        match self {
            Self::OpenAI => "gpt-4o-mini",
            Self::OpenRouter => "openai/gpt-4o-mini",
            Self::DeepSeek => "deepseek-chat",
            Self::Groq => "llama-3.1-8b-instant",
            Self::Custom => "",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::OpenAI => "OpenAI",
            Self::OpenRouter => "OpenRouter",
            Self::DeepSeek => "DeepSeek",
            Self::Groq => "Groq",
            Self::Custom => "Custom",
        }
    }

    pub fn all() -> &'static [LlmProvider] {
        &[
            Self::OpenAI,
            Self::OpenRouter,
            Self::DeepSeek,
            Self::Groq,
            Self::Custom,
        ]
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub api_base_url: String,
    pub api_key: String,
    pub model: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::OpenAI,
            api_base_url: LlmProvider::OpenAI.base_url().to_string(),
            api_key: String::new(),
            model: LlmProvider::OpenAI.default_model().to_string(),
        }
    }
}

const STORAGE_KEY: &str = "cta_llm_config";

fn load_config() -> LlmConfig {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item(STORAGE_KEY).ok().flatten())
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

fn save_config(config: &LlmConfig) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        if let Ok(json) = serde_json::to_string(config) {
            let _ = storage.set_item(STORAGE_KEY, &json);
        }
    }
}

#[derive(Clone, Copy)]
pub struct LlmState {
    config: RwSignal<LlmConfig>,
    settings_open: RwSignal<bool>,
}

impl LlmState {
    pub fn new() -> Self {
        Self {
            config: RwSignal::new(load_config()),
            settings_open: RwSignal::new(false),
        }
    }

    pub fn config(&self) -> RwSignal<LlmConfig> {
        self.config
    }

    pub fn settings_open(&self) -> RwSignal<bool> {
        self.settings_open
    }

    pub fn toggle_settings(&self) {
        self.settings_open.update(|v| *v = !*v);
    }

    pub fn update_provider(&self, provider: LlmProvider) {
        self.config.update(|c| {
            c.api_base_url = provider.base_url().to_string();
            c.model = provider.default_model().to_string();
            c.provider = provider;
        });
        self.persist();
    }

    pub fn update_api_base_url(&self, url: String) {
        self.config.update(|c| c.api_base_url = url);
        self.persist();
    }

    pub fn update_api_key(&self, key: String) {
        self.config.update(|c| c.api_key = key);
        self.persist();
    }

    pub fn update_model(&self, model: String) {
        self.config.update(|c| c.model = model);
        self.persist();
    }

    fn persist(&self) {
        let c = self.config.get_untracked();
        save_config(&c);
    }
}

pub fn provide_llm_state() {
    let state = LlmState::new();
    provide_context(state);
}

pub fn use_llm_state() -> LlmState {
    use_context::<LlmState>().expect("LlmState must be provided by an ancestor")
}
