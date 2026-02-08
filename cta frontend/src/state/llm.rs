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
            Self::OpenRouter => "moonshotai/kimi-k2.5",
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

pub const DEFAULT_PROMPT_NEW_STORY: &str = "\
You are writing the opening segment of a text adventure story.

The premise is: \"{choice text}\"

{story text}


Style: Write in a natural, lean, grounded voice. Avoid ornamental prose and dramatic flair. Write concrete sentences. \
Don't include irrelevant, unimportant details, actions or observations. \
Advance and progress the story.
Write the opening segment. Set the scene and establish the atmosphere. \
This is the start of an endless story. Write only the narrative text — no choices or options at the end.";

pub const DEFAULT_PROMPT_CONTINUING: &str = "\
You are continuing a text adventure story presented as a series of segments. Each segment begins with the choice that led to it.

Return a short response. 2-4 paragraphs (about 50-150 words) unless specified to be longer. \

Style: Write in a natural, lean, grounded voice. Avoid ornamental prose and dramatic flair. Write concrete sentences. \
Don't include irrelevant, unimportant details, actions or observations. \
Advance and progress the story.


the story so far:
{story path node history}

Choice selected: \"{choice text}\"

Details about what should happen: \"{story text}\"

Write only the narrative text for this segment — no choices or options at the end.
";

fn default_false() -> bool {
    false
}
fn default_prompt_new_story() -> String {
    DEFAULT_PROMPT_NEW_STORY.to_string()
}
fn default_prompt_continuing() -> String {
    DEFAULT_PROMPT_CONTINUING.to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub api_base_url: String,
    pub api_key: String,
    pub model: String,
    #[serde(default = "default_false")]
    pub llm_enabled: bool,
    #[serde(default = "default_prompt_new_story")]
    pub prompt_new_story: String,
    #[serde(default = "default_prompt_continuing")]
    pub prompt_continuing: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::OpenAI,
            api_base_url: LlmProvider::OpenAI.base_url().to_string(),
            api_key: String::new(),
            model: LlmProvider::OpenAI.default_model().to_string(),
            llm_enabled: false,
            prompt_new_story: DEFAULT_PROMPT_NEW_STORY.to_string(),
            prompt_continuing: DEFAULT_PROMPT_CONTINUING.to_string(),
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

    pub fn toggle_llm_enabled(&self) {
        self.config.update(|c| c.llm_enabled = !c.llm_enabled);
        self.persist();
    }

    pub fn update_prompt_new_story(&self, prompt: String) {
        self.config.update(|c| c.prompt_new_story = prompt);
        self.persist();
    }

    pub fn update_prompt_continuing(&self, prompt: String) {
        self.config.update(|c| c.prompt_continuing = prompt);
        self.persist();
    }

    pub fn reset_prompt_new_story(&self) {
        self.config.update(|c| c.prompt_new_story = DEFAULT_PROMPT_NEW_STORY.to_string());
        self.persist();
    }

    pub fn reset_prompt_continuing(&self) {
        self.config.update(|c| c.prompt_continuing = DEFAULT_PROMPT_CONTINUING.to_string());
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
