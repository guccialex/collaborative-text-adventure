use leptos::prelude::*;

use crate::domain::adventure::AdventureNode;
use crate::state::adventure::use_adventure_state;

use super::contribute_form::ContributeForm;
use super::ContributeMode;

#[component]
pub fn StoryScroll(
    segments: Memo<Vec<(usize, AdventureNode)>>,
    current_options: Memo<Vec<AdventureNode>>,
    current_parent_id: Memo<Option<String>>,
) -> impl IntoView {
    let state = use_adventure_state();
    let path = state.path();
    let show_contribute = state.show_contribute();

    view! {
        <div class="story-scroll">
            <For
                each={move || segments.get()}
                key={|(i, unit)| (*i, unit.id.clone())}
                children={move |(i, unit): (usize, AdventureNode)| {
                    let is_last = move || i == path.get().len().saturating_sub(1);
                    let segment_id = format!("segment-{}", unit.id);

                    view! {
                        <article
                            class="story-segment"
                            class:current={is_last}
                            id={segment_id}
                        >
                            <div class="segment-header">
                                <h2 class="story-title">{unit.choice_text.clone()}</h2>
                                <Show when={move || !is_last()}>
                                    <button
                                        class="revert-btn-inline"
                                        title="Branch from here"
                                        on:click=move |_| state.revert_to(i)
                                    >
                                        "Branch from here"
                                    </button>
                                </Show>
                            </div>
                            <p class="story-text">{unit.story_text.clone()}</p>
                        </article>
                    }
                }}
            />

            // Options for current segment
            {move || {
                let opts = current_options.get();
                let parent_id = current_parent_id.get().unwrap_or_default();

                if opts.is_empty() {
                    // No options - show contribute form directly
                    view! {
                        <div id="story-end">
                            <ContributeForm parent_id=parent_id mode=ContributeMode::DeadEnd />
                        </div>
                    }.into_any()
                } else {
                    // Has options - show them with "add your own" button
                    view! {
                        <div id="story-end">
                            <div class="options">
                                {opts.into_iter().map(|opt| {
                                    let o = opt.clone();
                                    view! {
                                        <button class="option-btn" on:click=move |_| state.choose(&o)>
                                            {opt.choice_text}
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                                <button
                                    class="option-btn add-option-btn"
                                    on:click=move |_| state.toggle_contribute()
                                >
                                    {move || if show_contribute.get() { "Cancel" } else { "Add your own option" }}
                                </button>
                            </div>

                            <Show when=move || show_contribute.get()>
                                <div>
                                    <ContributeForm
                                        parent_id=parent_id.clone()
                                        mode=ContributeMode::Branch
                                        on_cancel=Callback::new(move |_| state.close_contribute())
                                    />
                                </div>
                            </Show>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
