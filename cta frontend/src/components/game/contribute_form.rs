use leptos::callback::Callable;
use leptos::ev;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

use crate::api::llm::call_llm_stream;
use crate::api::newgrounds::get_session_id;
use crate::domain::adventure::AdventureNode;
use crate::state::adventure::use_adventure_state;
use crate::state::llm::use_llm_state;

use super::ContributeMode;

#[wasm_bindgen(inline_js = "
export function copy_to_clipboard(text) {
    if (navigator.clipboard && navigator.clipboard.writeText) {
        return navigator.clipboard.writeText(text);
    }
    const textarea = document.createElement('textarea');
    textarea.value = text;
    textarea.style.position = 'fixed';
    textarea.style.opacity = '0';
    document.body.appendChild(textarea);
    textarea.select();
    document.execCommand('copy');
    document.body.removeChild(textarea);
    return Promise.resolve();
}

export function auto_resize_story_textarea() {
    requestAnimationFrame(() => {
        const el = document.querySelector('.contribute-form textarea');
        if (el) {
            el.style.height = 'auto';
            el.style.height = el.scrollHeight + 'px';
        }
    });
}
")]
extern "C" {
    fn copy_to_clipboard(text: &str) -> js_sys::Promise;
    fn auto_resize_story_textarea();
}

fn build_llm_prompt(
    template: &str,
    path_nodes: &[(usize, AdventureNode)],
    choice_text: &str,
    story_text: &str,
) -> String {
    let mut history = String::new();
    for (_i, node) in path_nodes {
        history.push_str(&format!("Choice: \"{}\"\n\n", node.choice_text));
        history.push_str(&node.story_text);
        history.push_str("\n\n---\n\n");
    }
    let history = history.trim_end().to_string();

    template
        .replace("{story path node history}", &history)
        .replace("{choice text}", choice_text)
        .replace("{story text}", story_text)
}

/// Form for continuing the story or adding a branch.
#[component]
pub fn ContributeForm(
    parent_id: String,
    mode: ContributeMode,
    #[prop(optional)] on_cancel: Option<Callback<ev::MouseEvent>>,
) -> impl IntoView {
    let state = use_adventure_state();
    let llm = use_llm_state();
    let (choice_text, set_choice_text) = signal(String::new());
    let (story_text, set_story_text) = signal(String::new());
    let (submitting, set_submitting) = signal(false);
    let (copied, set_copied) = signal(false);
    let (llm_loading, set_llm_loading) = signal(false);
    let (llm_error, set_llm_error) = signal::<Option<String>>(None);
    let (llm_gen, set_llm_gen) = signal(0u64);
    let show_cancel = on_cancel.is_some();
    let is_new_story = mode == ContributeMode::NewStory;

    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        set_submitting.set(true);

        let session_id = get_session_id();

        let node = AdventureNode {
            id: format!("user_{}", js_sys::Date::now() as u64),
            parent_id: if parent_id.is_empty() { None } else { Some(parent_id.clone()) },
            choice_text: choice_text.get(),
            story_text: story_text.get(),
            created_by: None,
        };

        state.add_node(node, session_id);
    };

    let on_copy_prompt = move |_: ev::MouseEvent| {
        let config = llm.config().get();
        let graph = state.graph().get();
        let path = state.path().get();
        let path_nodes: Vec<(usize, AdventureNode)> = path
            .iter()
            .enumerate()
            .filter_map(|(i, id)| graph.node(id).map(|n| (i, n.clone())))
            .collect();

        let template = if path_nodes.is_empty() {
            &config.prompt_new_story
        } else {
            &config.prompt_continuing
        };
        let prompt = build_llm_prompt(template, &path_nodes, &choice_text.get(), &story_text.get());

        let promise = copy_to_clipboard(&prompt);

        set_copied.set(true);
        leptos::task::spawn_local(async move {
            let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
            gloo_timers::callback::Timeout::new(2_000, move || {
                set_copied.set(false);
            })
            .forget();
        });
    };

    let on_call_llm = move |_: ev::MouseEvent| {
        let config = llm.config().get();
        let graph = state.graph().get();
        let path = state.path().get();
        let path_nodes: Vec<(usize, AdventureNode)> = path
            .iter()
            .enumerate()
            .filter_map(|(i, id)| graph.node(id).map(|n| (i, n.clone())))
            .collect();

        let template = if path_nodes.is_empty() {
            &config.prompt_new_story
        } else {
            &config.prompt_continuing
        };
        let prompt = build_llm_prompt(template, &path_nodes, &choice_text.get(), &story_text.get());

        let gen = llm_gen.get() + 1;
        set_llm_gen.set(gen);
        set_llm_loading.set(true);
        set_llm_error.set(None);

        leptos::task::spawn_local(async move {
            match call_llm_stream(
                &config.api_base_url,
                &config.api_key,
                &config.model,
                &prompt,
                move |chunk| {
                    if llm_gen.get() == gen {
                        set_story_text.update(|s| s.push_str(chunk));
                        auto_resize_story_textarea();
                    }
                },
            )
            .await
            {
                Ok(()) => {
                    set_llm_loading.set(false);
                }
                Err(e) => {
                    set_llm_error.set(Some(e));
                    set_llm_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="contribute-section">
            <div class="contribute-header">
                <div>
                    <h2>{mode.title()}</h2>
                    <p class="hint">{mode.hint()}</p>
                </div>
                <Show when=move || llm.config().get().llm_enabled>
                    <div class="llm-actions">
                        <button
                            type="button"
                            class="llm-call-btn"
                            on:click=on_call_llm
                            disabled=move || {
                                let c = llm.config().get();
                                llm_loading.get() || c.api_key.is_empty() || c.api_base_url.is_empty() || c.model.is_empty()
                            }
                            title=move || {
                                let c = llm.config().get();
                                if c.api_key.is_empty() || c.api_base_url.is_empty() || c.model.is_empty() {
                                    "Configure LLM settings in the sidebar first".to_string()
                                } else {
                                    "Generate story text using LLM".to_string()
                                }
                            }
                        >
                            {move || if llm_loading.get() { "Generating..." } else { "Call LLM" }}
                        </button>
                        <button
                            type="button"
                            class="llm-prompt-btn"
                            on:click=on_copy_prompt
                        >
                            {move || if copied.get() { "Copied!" } else { "Copy as LLM prompt" }}
                        </button>
                    </div>
                </Show>
            </div>

            <Show when=move || llm_error.get().is_some()>
                <div class="llm-error">
                    {move || llm_error.get().unwrap_or_default()}
                </div>
            </Show>

            <form class="contribute-form" on:submit=on_submit>
                <div class="form-group">
                    <label>{if is_new_story { "Story premise" } else { "Choice text" }}</label>
                    <input
                        type="text"
                        placeholder={if is_new_story {
                            "e.g., You wake up in an abandoned space station"
                        } else {
                            "e.g., Open the mysterious door"
                        }}
                        prop:value=move || choice_text.get()
                        on:input=move |ev| set_choice_text.set(event_target_value(&ev))
                        required
                    />
                </div>

                <div class="form-group">
                    <label>"Story text"</label>
                    <textarea
                        placeholder={if is_new_story {
                            "Write the opening of your adventure..."
                        } else {
                            "What happens when the player makes this choice..."
                        }}
                        prop:value=move || story_text.get()
                        on:input=move |ev| {
                            set_story_text.set(event_target_value(&ev));
                            auto_resize_story_textarea();
                        }
                        required
                    />
                </div>

                <div class="form-actions">
                    <button type="submit" class="submit-btn" disabled=move || submitting.get()>
                        {move || if submitting.get() { "Submitting..." } else { "Add to Story" }}
                    </button>
                    <Show when=move || show_cancel>
                        <button
                            type="button"
                            class="cancel-btn"
                            on:click=move |ev| {
                                if let Some(cb) = &on_cancel {
                                    cb.run(ev);
                                }
                            }
                        >
                            "Cancel"
                        </button>
                    </Show>
                </div>
            </form>
        </div>
    }
}
