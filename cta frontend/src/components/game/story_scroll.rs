use leptos::prelude::*;
use wasm_bindgen::prelude::*;

use crate::api::newgrounds::get_session_id;
use crate::domain::adventure::AdventureNode;
use crate::state::adventure::use_adventure_state;

use super::contribute_form::ContributeForm;
use super::ContributeMode;

#[wasm_bindgen(inline_js = "
export function toggle_fullscreen() {
    if (document.fullscreenElement) {
        document.exitFullscreen();
    } else {
        document.documentElement.requestFullscreen();
    }
}

export function is_fullscreen() {
    return !!document.fullscreenElement;
}

let _fs_callback = null;
export function on_fullscreen_change(cb) {
    if (_fs_callback) document.removeEventListener('fullscreenchange', _fs_callback);
    _fs_callback = () => cb(!!document.fullscreenElement);
    document.addEventListener('fullscreenchange', _fs_callback);
}
")]
extern "C" {
    fn toggle_fullscreen();
    fn is_fullscreen() -> bool;
    fn on_fullscreen_change(cb: &Closure<dyn Fn(bool)>);
}

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
    let graph = state.graph();
    let ng_username = use_context::<RwSignal<Option<String>>>()
        .expect("NG username signal must be provided by App");

    let at_root = Memo::new(move |_| path.get().is_empty());
    let (fullscreen, set_fullscreen) = signal(false);

    let fs_closure = Closure::new(move |is_fs: bool| {
        set_fullscreen.set(is_fs);
    });
    on_fullscreen_change(&fs_closure);
    fs_closure.forget();

    view! {
        <div class="story-scroll">
            <Show when=move || !at_root.get()>
                <button
                    class="back-to-root-btn"
                    on:click=move |_| state.reset_path()
                >
                    "Back to root"
                </button>
            </Show>

            <button
                class="fullscreen-btn"
                title=move || if fullscreen.get() { "Exit fullscreen" } else { "Enter fullscreen" }
                on:click=move |_| {
                    toggle_fullscreen();
                    set_fullscreen.set(is_fullscreen());
                }
            >
                {move || if fullscreen.get() { "\u{2715}" } else { "\u{26F6}" }}
            </button>

            <Show when=move || at_root.get()>
                <div class="intro-text">
                    <p>"Choose an opening below, or create a new one."</p>
                </div>
            </Show>

            <For
                each={move || segments.get()}
                key={|(i, unit)| (*i, unit.id.clone())}
                children={move |(i, unit): (usize, AdventureNode)| {
                    let is_last = move || i == path.get().len().saturating_sub(1);
                    let segment_id = format!("segment-{}", unit.id);

                    let unit_id = unit.id.clone();
                    let unit_created_by = unit.created_by.clone();

                    let can_delete = move || {
                        let current_user = ng_username.get();
                        let no_children = graph.get().children_ids(&unit_id).is_empty();

                        if current_user.as_deref() == Some("comicstosteal") {
                            return no_children;
                        }

                        if !is_last() {
                            return false;
                        }
                        match (&current_user, &unit_created_by) {
                            (Some(user), Some(creator)) if user == creator => {}
                            _ => return false,
                        }
                        no_children
                    };

                    let delete_node_id = unit.id.clone();

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
                                <Show when=can_delete>
                                    {
                                        let delete_id = delete_node_id.clone();
                                        view! {
                                            <button
                                                class="delete-btn-inline"
                                                title="Delete this node"
                                                on:click=move |_| {
                                                    state.delete_node(delete_id.clone(), get_session_id())
                                                }
                                            >
                                                "Delete"
                                            </button>
                                        }
                                    }
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
