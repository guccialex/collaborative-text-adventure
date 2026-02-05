use leptos::ev;
use leptos::callback::Callable;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::web_sys;

use crate::api::{fetch_adventure, Adventure, AdventureChoiceUnit};

#[derive(Clone)]
enum LoadState {
    Loading,
    Ready,
    Error(String),
}

#[derive(Clone, Copy)]
enum ContributeMode {
    DeadEnd,
    Branch,
}

impl ContributeMode {
    fn title(self) -> &'static str {
        match self {
            Self::DeadEnd => "Continue the story",
            Self::Branch => "Add a new path",
        }
    }

    fn hint(self) -> &'static str {
        match self {
            Self::DeadEnd => "This path hasn't been written yet. Be the first to add to it.",
            Self::Branch => "Create a new option branching from this point.",
        }
    }
}

fn root_path(adventure: &Adventure) -> Vec<String> {
    adventure
        .root()
        .map(|root| vec![root.id.clone()])
        .unwrap_or_default()
}

fn build_user_unit(parent_id: &str, choice_text: String, story_text: String) -> AdventureChoiceUnit {
    AdventureChoiceUnit {
        id: format!("user_{}", js_sys::Date::now() as u64),
        parent_id: Some(parent_id.to_string()),
        choice_text,
        story_text,
    }
}

fn scroll_to_segment(id: &str) {
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        if let Some(el) = doc.get_element_by_id(&format!("segment-{}", id)) {
            let _ = el.scroll_into_view_with_bool(true);
        }
    }
}

#[component]
pub fn Game() -> impl IntoView {
    let (adventure, set_adventure) = signal(Adventure::default());
    let (path, set_path) = signal::<Vec<String>>(Vec::new());
    let (load_state, set_load_state) = signal(LoadState::Loading);
    let (show_contribute, set_show_contribute) = signal(false);
    let (reload_token, set_reload_token) = signal(0u64);

    // Load data on mount and when reloading
    Effect::new(move |_| {
        let _ = reload_token.get();
        set_load_state.set(LoadState::Loading);
        spawn_local(async move {
            match fetch_adventure().await {
                Ok(data) => {
                    set_path.set(root_path(&data));
                    set_adventure.set(data);
                    set_load_state.set(LoadState::Ready);
                }
                Err(e) => {
                    set_load_state.set(LoadState::Error(e));
                }
            }
        })
    });

    // All segments in path with their data
    let segments = create_memo(move |_| {
        let p = path.get();
        let adv = adventure.get();
        p.iter()
            .enumerate()
            .filter_map(|(i, id)| adv.get(id).map(|unit| (i, unit.clone())))
            .collect::<Vec<_>>()
    });

    // Options for the last segment
    let current_options = create_memo(move |_| -> Vec<AdventureChoiceUnit> {
        path.get()
            .last()
            .map(|id| adventure.get().children(id).into_iter().cloned().collect())
            .unwrap_or_default()
    });

    let current_parent_id = create_memo(move |_| path.get().last().cloned());

    let is_loading = create_memo(move |_| matches!(load_state.get(), LoadState::Loading));
    let error_message = create_memo(move |_| match load_state.get() {
        LoadState::Error(msg) => Some(msg),
        _ => None,
    });
    let is_ready = create_memo(move |_| matches!(load_state.get(), LoadState::Ready));

    let on_choose = move |unit: AdventureChoiceUnit| {
        set_path.update(|p| p.push(unit.id));
        set_show_contribute.set(false);
    };

    let revert_to = move |index: usize| {
        set_path.update(|p| p.truncate(index + 1));
        set_show_contribute.set(false);
    };

    let restart = move |_: ev::MouseEvent| {
        set_show_contribute.set(false);
        set_reload_token.update(|value| *value = value.wrapping_add(1));
    };

    view! {
        <div class="app-layout">
            // Left sidebar - path navigation
            <aside class="sidebar">
                <div class="sidebar-brand">
                    <div class="brand-mark">"∞"</div>
                    <div>
                        <p class="brand-title">"Endless Tale"</p>
                        <p class="brand-subtitle">"Collaborative Adventure"</p>
                    </div>
                </div>
                <h2 class="sidebar-title">"Path"</h2>
                <nav class="path-list">
                    <For
                        each={move || segments.get()}
                        key={|(i, unit)| (*i, unit.id.clone())}
                        children={move |(i, unit): (usize, AdventureChoiceUnit)| {
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
                                            title="Branch from here"
                                            on:click=move |_| revert_to(i)
                                        >
                                            "↩"
                                        </button>
                                    </Show>
                                </div>
                            }
                        }}
                    />
                </nav>
            </aside>

            // Main content area
            <main class="main-content">
                <header class="story-header">
                    <p class="story-eyebrow">"Live, community-written story"</p>
                    <h1 class="story-heading">"The Endless Tale"</h1>
                    <p class="story-lede">
                        "Read, choose, and continue the thread. Every branch is written by players."
                    </p>
                </header>

                <Show when=move || is_loading.get()>
                    <div class="loading"><span class="loading-dots">"..."</span></div>
                </Show>

                <Show when=move || error_message.get().is_some()>
                    <div class="error">
                        <p>{move || error_message.get().unwrap_or_else(|| "Unknown error".to_string())}</p>
                        <button class="restart-btn" on:click=restart>"Try Again"</button>
                    </div>
                </Show>

                <Show when=move || is_ready.get()>
                    <div class="story-scroll">
                        <For
                            each={move || segments.get()}
                            key={|(i, unit)| (*i, unit.id.clone())}
                            children={move |(i, unit): (usize, AdventureChoiceUnit)| {
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
                                                    on:click=move |_| revert_to(i)
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
                                                    <button class="option-btn" on:click=move |_| on_choose(o.clone())>
                                                        {opt.choice_text}
                                                    </button>
                                                }
                                            }).collect::<Vec<_>>()}
                                            <button
                                                class="option-btn add-option-btn"
                                                on:click=move |_| set_show_contribute.update(|v| *v = !*v)
                                            >
                                                {move || if show_contribute.get() { "Cancel" } else { "Add your own option" }}
                                            </button>
                                        </div>

                                        <Show when=move || show_contribute.get()>
                                            <ContributeForm
                                                parent_id=parent_id.clone()
                                                mode=ContributeMode::Branch
                                                on_cancel=Callback::new(move |_| set_show_contribute.set(false))
                                            />
                                        </Show>
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                </Show>
            </main>
        </div>
    }
}

/// Form for continuing the story or adding a branch.
#[component]
fn ContributeForm(
    parent_id: String,
    mode: ContributeMode,
    #[prop(optional)] on_cancel: Option<Callback<ev::MouseEvent>>,
) -> impl IntoView {
    let (choice_text, set_choice_text) = signal(String::new());
    let (story_text, set_story_text) = signal(String::new());
    let (submitting, set_submitting) = signal(false);
    let show_cancel = on_cancel.is_some();

    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        set_submitting.set(true);

        let new_unit = build_user_unit(
            &parent_id,
            choice_text.get(),
            story_text.get(),
        );

        log::info!("Would submit: {:?}", new_unit);
        set_submitting.set(false);
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
