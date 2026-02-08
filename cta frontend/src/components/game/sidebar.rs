use leptos::prelude::*;
use wasm_bindgen::prelude::*;

use crate::domain::adventure::AdventureNode;
use crate::state::adventure::use_adventure_state;
use crate::state::llm::{use_llm_state, LlmProvider};

use super::helpers::scroll_to_segment;

#[wasm_bindgen(inline_js = "
export function auto_resize_prompt_textareas() {
    requestAnimationFrame(() => {
        const scrollY = window.scrollY;
        const sidebar = document.querySelector('.sidebar');
        const sidebarScroll = sidebar ? sidebar.scrollTop : 0;

        document.querySelectorAll('.llm-prompt-textarea').forEach(el => {
            el.style.height = 'auto';
            el.style.height = (el.scrollHeight + 32) + 'px';
        });

        if (sidebar) sidebar.scrollTop = sidebarScroll;
        window.scrollTo(0, scrollY);
    });
}
")]
extern "C" {
    fn auto_resize_prompt_textareas();
}

#[component]
pub fn Sidebar(
    segments: Memo<Vec<(usize, AdventureNode)>>,
) -> impl IntoView {
    let state = use_adventure_state();
    let path = state.path();
    let llm = use_llm_state();
    let settings_open = llm.settings_open();

    view! {
        <aside class="sidebar">
            <div class="sidebar-brand">
                <div class="brand-mark">"∞"</div>
                <div>
                    <p class="brand-subtitle">"Collaborative Adventure"</p>
                </div>
            </div>
            <h2 class="sidebar-title">"Path"</h2>
            <nav class="path-list">
                <For
                    each={move || segments.get()}
                    key={|(i, unit)| (*i, unit.id.clone())}
                    children={move |(i, unit): (usize, AdventureNode)| {
                        let id_for_scroll = unit.id.clone();
                        let is_current = move || i == path.get().len().saturating_sub(1);
                        view! {
                            <div class="path-item-row">
                                <button
                                    class="path-item"
                                    class:active={is_current}
                                    on:click={
                                        let id = id_for_scroll.clone();
                                        move |_| scroll_to_segment(&id)
                                    }
                                >
                                    {unit.choice_text.clone()}
                                </button>
                                <Show when={move || !is_current()}>
                                    <button
                                        class="revert-btn"
                                        title="Return here"
                                        on:click=move |_| state.revert_to(i)
                                    >
                                        "↩"
                                    </button>
                                </Show>
                            </div>
                        }
                    }}
                />
            </nav>

            <div class="llm-settings-section">
                <label class="llm-enable-checkbox">
                    <input
                        type="checkbox"
                        prop:checked=move || llm.config().get().llm_enabled
                        on:change=move |_| llm.toggle_llm_enabled()
                    />
                    <span>"LLM features"</span>
                </label>

                <Show when=move || llm.config().get().llm_enabled>
                    <button
                        class="llm-settings-toggle"
                        on:click=move |_| {
                            llm.toggle_settings();
                            auto_resize_prompt_textareas();
                        }
                    >
                        <span class="sidebar-title" style="margin-bottom: 0">"LLM Settings"</span>
                        <span class="toggle-arrow">
                            {move || if settings_open.get() { "\u{25BE}" } else { "\u{25B8}" }}
                        </span>
                    </button>

                    <Show when=move || settings_open.get()>
                        <div class="llm-settings-body">
                            <div class="llm-providers">
                                {LlmProvider::all().iter().map(|provider| {
                                    let p = provider.clone();
                                    let p2 = provider.clone();
                                    let name = provider.display_name();
                                    view! {
                                        <button
                                            type="button"
                                            class="llm-provider-btn"
                                            class:active=move || llm.config().get().provider == p
                                            on:click=move |_| llm.update_provider(p2.clone())
                                        >
                                            {name}
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>

                            <div class="llm-field">
                                <label>"API Base URL"</label>
                                <input
                                    type="text"
                                    placeholder="https://api.openai.com/v1"
                                    prop:value=move || llm.config().get().api_base_url.clone()
                                    on:input=move |ev| llm.update_api_base_url(event_target_value(&ev))
                                    disabled=move || llm.config().get().provider != LlmProvider::Custom
                                />
                            </div>

                            <div class="llm-field">
                                <label>"API Key"</label>
                                <input
                                    type="password"
                                    placeholder="sk-..."
                                    prop:value=move || llm.config().get().api_key.clone()
                                    on:input=move |ev| llm.update_api_key(event_target_value(&ev))
                                />
                            </div>

                            <div class="llm-field">
                                <label>"Model"</label>
                                {move || {
                                    let config = llm.config().get();
                                    if config.provider == LlmProvider::OpenRouter {
                                        let presets = [
                                            "moonshotai/kimi-k2.5",
                                            "anthropic/claude-sonnet-4.5",
                                            "arcee-ai/trinity-large-preview:free",
                                            "stepfun/step-3.5-flash:free",
                                        ];
                                        view! {
                                            <div class="llm-model-presets">
                                                {presets.iter().map(|&m| {
                                                    let model = m.to_string();
                                                    let model2 = m.to_string();
                                                    view! {
                                                        <button
                                                            type="button"
                                                            class="llm-provider-btn"
                                                            class:active=move || llm.config().get().model == model
                                                            on:click=move |_| llm.update_model(model2.clone())
                                                        >
                                                            {m.split('/').last().unwrap_or(m)}
                                                        </button>
                                                    }
                                                }).collect::<Vec<_>>()}
                                                <button
                                                    type="button"
                                                    class="llm-provider-btn"
                                                    class:active=move || !presets.contains(&llm.config().get().model.as_str())
                                                    on:click=move |_| llm.update_model(String::new())
                                                >
                                                    "Other"
                                                </button>
                                            </div>
                                            <Show when=move || !presets.contains(&llm.config().get().model.as_str())>
                                                <input
                                                    type="text"
                                                    placeholder="e.g. openai/gpt-4o-mini"
                                                    prop:value=move || llm.config().get().model.clone()
                                                    on:input=move |ev| llm.update_model(event_target_value(&ev))
                                                />
                                            </Show>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <input
                                                type="text"
                                                placeholder="gpt-4o-mini"
                                                prop:value=move || llm.config().get().model.clone()
                                                on:input=move |ev| llm.update_model(event_target_value(&ev))
                                            />
                                        }.into_any()
                                    }
                                }}
                            </div>

                            <div class="llm-prompt-section">
                                <p class="llm-prompt-hint">
                                    "Use these variables in your prompts and they will be replaced when sent:"
                                </p>
                                <ul class="llm-prompt-vars">
                                    <li><code>"{story path node history}"</code>" — the full story so far (each choice + narrative)"</li>
                                    <li><code>"{choice text}"</code>" — the choice or premise the user entered"</li>
                                    <li><code>"{story text}"</code>" — the story text the user has written (can be empty)"</li>
                                </ul>

                                <div class="llm-field">
                                    <div class="llm-prompt-header">
                                        <label>"Prompt (New Story)"</label>
                                        <button
                                            type="button"
                                            class="llm-reset-btn"
                                            on:click=move |_| {
                                                llm.reset_prompt_new_story();
                                                auto_resize_prompt_textareas();
                                            }
                                        >
                                            "Reset Prompt"
                                        </button>
                                    </div>
                                    <textarea
                                        class="llm-prompt-textarea"
                                        prop:value=move || llm.config().get().prompt_new_story.clone()
                                        on:input=move |ev| {
                                            llm.update_prompt_new_story(event_target_value(&ev));
                                            auto_resize_prompt_textareas();
                                        }
                                    />
                                </div>

                                <div class="llm-field">
                                    <div class="llm-prompt-header">
                                        <label>"Prompt (Continuing Story)"</label>
                                        <button
                                            type="button"
                                            class="llm-reset-btn"
                                            on:click=move |_| {
                                                llm.reset_prompt_continuing();
                                                auto_resize_prompt_textareas();
                                            }
                                        >
                                            "Reset Prompt"
                                        </button>
                                    </div>
                                    <textarea
                                        class="llm-prompt-textarea"
                                        prop:value=move || llm.config().get().prompt_continuing.clone()
                                        on:input=move |ev| {
                                            llm.update_prompt_continuing(event_target_value(&ev));
                                            auto_resize_prompt_textareas();
                                        }
                                    />
                                </div>
                            </div>
                        </div>
                    </Show>
                </Show>
            </div>

            <div class="sidebar-disclaimer">
                <p class="disclaimer-heading">"THIS GAME INTERACTS WITH A THIRD PARTY SERVER"</p>
                <p class="disclaimer-body">
                    "owned by me, hosted on Google Cloud. "
                    "All submitted text is saved there, mapped to the newgrounds id of the user who submitted it. "
                    "No data will be sent to the third party server if you don't submit anything "
                    "(or press the button in the top right). "
                    "Data is fetched from the third party server when getting the stories."
                </p>
            </div>
        </aside>
    }
}
