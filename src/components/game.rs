use leptos::prelude::*;
use leptos::ev;
use leptos::task::spawn_local;
use crate::api::{fetch_adventure, Adventure, AdventureChoiceUnit};

#[component]
pub fn Game() -> impl IntoView {
    let (adventure, set_adventure) = signal(Adventure::default());
    let (path, set_path) = signal::<Vec<String>>(vec![]);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<String>>(None);

    // Load data on mount
    Effect::new(move |_| {
        spawn_local(async move {
            match fetch_adventure().await {
                Ok(data) => {
                    if let Some(root) = data.root() {
                        set_path.set(vec![root.id.clone()]);
                    }
                    set_adventure.set(data);
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    });

    // Current unit is last in path
    let current = move || {
        path.get().last().and_then(|id| adventure.get().get(id).cloned())
    };

    // Options are children of current
    let options = move || {
        path.get().last()
            .map(|id| adventure.get().children(id).into_iter().cloned().collect())
            .unwrap_or_default()
    };

    // Path items for sidebar
    let path_items = move || {
        let p = path.get();
        let adv = adventure.get();
        p.iter().map(|id| adv.get(id).cloned()).collect::<Vec<_>>()
    };

    let on_choose = move |unit: AdventureChoiceUnit| {
        set_path.update(|p| p.push(unit.id));
    };

    let revert_to = move |index: usize| {
        set_path.update(|p| p.truncate(index + 1));
    };

    let restart = move |_: ev::MouseEvent| {
        if let Some(root) = adventure.get().root() {
            set_path.set(vec![root.id.clone()]);
        }
    };

    view! {
        <div class="app-layout">
            // Left sidebar - path history
            <aside class="sidebar">
                <h2 class="sidebar-title">"Path"</h2>
                <nav class="path-list">
                    <For
                        each={move || path_items().into_iter().enumerate().collect::<Vec<_>>()}
                        key={|(i, unit)| (*i, unit.as_ref().map(|u| u.id.clone()))}
                        children={move |(i, unit): (usize, Option<AdventureChoiceUnit>)| {
                            let is_current = move || i == path.get().len().saturating_sub(1);
                            unit.map(|u| {
                                let title = u.choice_text.clone();
                                view! {
                                    <button
                                        class="path-item"
                                        class:active={is_current}
                                        on:click=move |_| revert_to(i)
                                    >
                                        {title}
                                    </button>
                                }
                            })
                        }}
                    />
                </nav>
            </aside>

            // Main content area
            <main class="main-content">
                <Show when=move || loading.get()>
                    <div class="loading"><span class="loading-dots">"..."</span></div>
                </Show>

                <Show when=move || error.get().is_some()>
                    <div class="error">
                        <p>{move || error.get()}</p>
                        <button on:click=restart>"Try Again"</button>
                    </div>
                </Show>

                // Current segment with options
                <Show when=move || current().is_some() && !loading.get()>
                    {move || current().map(|unit| {
                        let opts: Vec<AdventureChoiceUnit> = options();

                        view! {
                            <article class="story-panel">
                                <h1 class="story-title">{unit.choice_text.clone()}</h1>
                                <p class="story-text">{unit.story_text.clone()}</p>

                                {if opts.is_empty() {
                                    view! {
                                        <ContributeForm parent_id=unit.id.clone() on_restart=restart />
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="options">
                                            {opts.into_iter().map(|opt| {
                                                let o = opt.clone();
                                                view! {
                                                    <button class="option-btn" on:click=move |_| on_choose(o.clone())>
                                                        {opt.choice_text}
                                                    </button>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                }}
                            </article>
                        }
                    })}
                </Show>
            </main>
        </div>
    }
}

#[component]
fn ContributeForm(
    parent_id: String,
    on_restart: impl Fn(ev::MouseEvent) + 'static + Clone + Send + Sync,
) -> impl IntoView {
    let (choice_text, set_choice_text) = signal(String::new());
    let (story_text, set_story_text) = signal(String::new());
    let (submitting, set_submitting) = signal(false);

    let parent = parent_id.clone();
    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        set_submitting.set(true);

        let new_unit = AdventureChoiceUnit {
            id: format!("user_{}", js_sys::Date::now() as u64),
            parent_id: Some(parent.clone()),
            choice_text: choice_text.get(),
            story_text: story_text.get(),
        };

        log::info!("Would submit: {:?}", new_unit);
        set_submitting.set(false);
    };

    view! {
        <div class="contribute-section">
            <h2>"Continue the story"</h2>
            <p class="hint">"This path hasn't been written yet. Be the first to add to it."</p>

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
                    <button type="button" class="restart-btn" on:click=on_restart>
                        "Start Over"
                    </button>
                </div>
            </form>
        </div>
    }
}
