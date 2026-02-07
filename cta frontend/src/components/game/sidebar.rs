use leptos::prelude::*;

use crate::domain::adventure::AdventureNode;
use crate::state::adventure::use_adventure_state;
use crate::state::llm::{use_llm_state, LlmProvider};

use super::helpers::scroll_to_segment;

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
                <button
                    class="llm-settings-toggle"
                    on:click=move |_| llm.toggle_settings()
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
                            <input
                                type="text"
                                placeholder="gpt-4o-mini"
                                prop:value=move || llm.config().get().model.clone()
                                on:input=move |ev| llm.update_model(event_target_value(&ev))
                            />
                        </div>
                    </div>
                </Show>
            </div>
        </aside>
    }
}
