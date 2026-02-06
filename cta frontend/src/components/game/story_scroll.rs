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
    let counts = state.descendant_counts();

    let at_root = Memo::new(move |_| path.get().is_empty());

    view! {
        <div class="story-scroll">
            <Show when=move || at_root.get()>
                <div class="intro-text">
                    <p>"This is a branching text adventure. Choose an opening below to begin reading, or start your own story."</p>
                </div>
            </Show>

            <Show when=move || !at_root.get()>
                <button
                    class="back-to-root-btn"
                    on:click=move |_| state.reset_path()
                >
                    "Back to root"
                </button>
            </Show>

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
                                        title="Return here"
                                        on:click=move |_| state.revert_to(i)
                                    >
                                        "Return here"
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
                let is_root = at_root.get();
                let contribute_mode = if is_root {
                    ContributeMode::NewStory
                } else {
                    ContributeMode::DeadEnd
                };

                if opts.is_empty() {
                    // No options - show contribute form directly
                    view! {
                        <div id="story-end">
                            <ContributeForm parent_id=parent_id mode=contribute_mode />
                        </div>
                    }.into_any()
                } else {
                    let branch_mode = if is_root {
                        ContributeMode::NewStory
                    } else {
                        ContributeMode::Branch
                    };

                    // Has options - show them with "add your own" button
                    view! {
                        <div id="story-end">
                            <h3 class="options-label">
                                {if is_root { "Choose a story" } else { "What happens next?" }}
                            </h3>
                            <div class="options">
                                {opts.into_iter().map(|opt| {
                                    let o = opt.clone();
                                    let opt_id = opt.id.clone();
                                    let count_label = move || {
                                        let n = counts.get().get(&opt_id).copied().unwrap_or(0);
                                        if n == 0 {
                                            String::new()
                                        } else {
                                            format!("{}", n)
                                        }
                                    };
                                    view! {
                                        <button class="option-btn" on:click=move |_| state.choose(&o)>
                                            <span class="option-text">{opt.choice_text}</span>
                                            {move || {
                                                let label = count_label();
                                                if label.is_empty() {
                                                    None
                                                } else {
                                                    Some(view! { <span class="option-count">{label}</span> })
                                                }
                                            }}
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                                <button
                                    class="option-btn add-option-btn"
                                    on:click=move |_| state.toggle_contribute()
                                >
                                    {move || if show_contribute.get() { "Cancel" } else if is_root { "Start a new story" } else { "Add your own option" }}
                                </button>
                            </div>

                            <Show when=move || show_contribute.get()>
                                <div>
                                    <ContributeForm
                                        parent_id=parent_id.clone()
                                        mode=branch_mode
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
