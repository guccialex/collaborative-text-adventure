use leptos::callback::Callable;
use leptos::ev;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

use crate::domain::adventure::AdventureNode;
use crate::state::adventure::use_adventure_state;

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
")]
extern "C" {
    fn copy_to_clipboard(text: &str) -> js_sys::Promise;
}

fn build_llm_prompt(
    path_nodes: &[(usize, AdventureNode)],
    choice_text: &str,
    story_text: &str,
) -> String {
    let mut prompt = String::new();

    if path_nodes.is_empty() {
        // Starting a brand new story — no prior context
        prompt.push_str(
            "You are writing the opening segment of a collaborative text adventure story.\n\n",
        );

        prompt.push_str(&format!(
            "The premise is: \"{}\"\n\n",
            choice_text,
        ));

        if !story_text.is_empty() {
            prompt.push_str(&format!(
                "The author has described what they want to happen: \"{}\"\n\n",
                story_text,
            ));
        }

        prompt.push_str(
            "Write the opening segment (a few paragraphs). Set the scene, establish the \
             atmosphere, and draw the reader in. Write only the narrative text \
             — do not include choices or options at the end.",
        );
    } else {
        // Continuing an existing story
        prompt.push_str(
            "You are continuing a collaborative text adventure story. Below is the story so far, \
             presented as a series of segments. Each segment begins with the choice that led to it, \
             followed by the narrative.\n\n",
        );

        for (_i, node) in path_nodes {
            prompt.push_str(&format!("Choice: \"{}\"\n\n", node.choice_text));
            prompt.push_str(&node.story_text);
            prompt.push_str("\n\n---\n\n");
        }

        prompt.push_str(&format!(
            "The reader has chosen: \"{}\"\n\n",
            choice_text,
        ));

        if !story_text.is_empty() {
            prompt.push_str(&format!(
                "They have described what should happen: \"{}\"\n\n",
                story_text,
            ));
        }

        prompt.push_str(
            "Write the next segment of the story (a few paragraphs). Match the tone, style, \
             and atmosphere established so far. Make it vivid and engaging. Write only the \
             narrative text for this segment — do not include choices or options at the end.",
        );
    }

    prompt
}

/// Form for continuing the story or adding a branch.
#[component]
pub fn ContributeForm(
    parent_id: String,
    mode: ContributeMode,
    #[prop(optional)] on_cancel: Option<Callback<ev::MouseEvent>>,
) -> impl IntoView {
    let state = use_adventure_state();
    let (choice_text, set_choice_text) = signal(String::new());
    let (story_text, set_story_text) = signal(String::new());
    let (submitting, set_submitting) = signal(false);
    let (copied, set_copied) = signal(false);
    let show_cancel = on_cancel.is_some();
    let is_new_story = mode == ContributeMode::NewStory;

    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        set_submitting.set(true);

        let node = AdventureNode {
            id: format!("user_{}", js_sys::Date::now() as u64),
            parent_id: if parent_id.is_empty() { None } else { Some(parent_id.clone()) },
            choice_text: choice_text.get(),
            story_text: story_text.get(),
        };

        state.add_node(node);
    };

    let on_copy_prompt = move |_: ev::MouseEvent| {
        let graph = state.graph().get();
        let path = state.path().get();
        let path_nodes: Vec<(usize, AdventureNode)> = path
            .iter()
            .enumerate()
            .filter_map(|(i, id)| graph.node(id).map(|n| (i, n.clone())))
            .collect();

        let prompt = build_llm_prompt(&path_nodes, &choice_text.get(), &story_text.get());

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

    view! {
        <div class="contribute-section">
            <div class="contribute-header">
                <div>
                    <h2>{mode.title()}</h2>
                    <p class="hint">{mode.hint()}</p>
                </div>
                <button
                    type="button"
                    class="llm-prompt-btn"
                    on:click=on_copy_prompt
                >
                    {move || if copied.get() { "Copied!" } else { "Copy as LLM prompt" }}
                </button>
            </div>

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
                        on:input=move |ev| set_story_text.set(event_target_value(&ev))
                        rows="4"
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
