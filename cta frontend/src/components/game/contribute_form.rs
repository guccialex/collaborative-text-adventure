use leptos::callback::Callable;
use leptos::ev;
use leptos::prelude::*;

use crate::domain::adventure::AdventureNode;
use crate::state::adventure::use_adventure_state;

use super::ContributeMode;

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
    let show_cancel = on_cancel.is_some();

    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        set_submitting.set(true);

        let node = AdventureNode {
            id: format!("user_{}", js_sys::Date::now() as u64),
            parent_id: Some(parent_id.clone()),
            choice_text: choice_text.get(),
            story_text: story_text.get(),
        };

        state.add_node(node);
    };

    view! {
        <div class="contribute-section">
            <h2>{mode.title()}</h2>
            <p class="hint">{mode.hint()}</p>

            <form class="contribute-form" on:submit=on_submit>
                <div class="form-group">
                    <label>"Choice text"</label>
                    <input
                        type="text"
                        placeholder="e.g., Open the mysterious door"
                        prop:value=move || choice_text.get()
                        on:input=move |ev| set_choice_text.set(event_target_value(&ev))
                        required
                    />
                </div>

                <div class="form-group">
                    <label>"Story text"</label>
                    <textarea
                        placeholder="What happens when the player makes this choice..."
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
