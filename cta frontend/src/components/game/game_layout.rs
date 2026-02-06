use leptos::prelude::*;

use crate::domain::adventure::AdventureNode;
use crate::state::adventure::{use_adventure_state, LoadState};

use super::sidebar::Sidebar;
use super::story_header::StoryHeader;
use super::story_scroll::StoryScroll;

#[component]
pub fn Game() -> impl IntoView {
    let state = use_adventure_state();
    let graph = state.graph();
    let path = state.path();
    let load_state = state.load_state();

    // All segments in path with their data
    let segments = Memo::new(move |_| {
        let p = path.get();
        let adv = graph.get();
        p.iter()
            .enumerate()
            .filter_map(|(i, id)| adv.node(id).map(|unit| (i, unit.clone())))
            .collect::<Vec<_>>()
    });

    let counts = state.descendant_counts();

    // Options for the last segment (or roots if path is empty), sorted by descendant count
    let current_options = Memo::new(move |_| -> Vec<AdventureNode> {
        let mut opts: Vec<AdventureNode> = match path.get().last() {
            Some(id) => graph.get().children(id).into_iter().cloned().collect(),
            None => graph.get().roots().into_iter().cloned().collect(),
        };
        let c = counts.get();
        opts.sort_by(|a, b| {
            let ca = c.get(&a.id).copied().unwrap_or(0);
            let cb = c.get(&b.id).copied().unwrap_or(0);
            cb.cmp(&ca)
        });
        opts
    });

    let current_parent_id = Memo::new(move |_| path.get().last().cloned());

    let is_loading = Memo::new(move |_| matches!(load_state.get(), LoadState::Loading));
    let error_message = Memo::new(move |_| match load_state.get() {
        LoadState::Error(msg) => Some(msg),
        _ => None,
    });
    let is_ready = Memo::new(move |_| matches!(load_state.get(), LoadState::Ready));

    view! {
        <div class="app-layout">
            <Sidebar segments=segments />

            // Main content area
            <main class="main-content">
                <StoryHeader />

                <Show when=move || is_loading.get()>
                    <div class="loading"><span class="loading-dots">"..."</span></div>
                </Show>

                <Show when=move || error_message.get().is_some()>
                    <div class="error">
                        <p>{move || error_message.get().unwrap_or_else(|| "Unknown error".to_string())}</p>
                        <button class="restart-btn" on:click=move |_| state.reload()>"Try Again"</button>
                    </div>
                </Show>

                <Show when=move || is_ready.get()>
                    <StoryScroll
                        segments=segments
                        current_options=current_options
                        current_parent_id=current_parent_id
                    />
                </Show>
            </main>
        </div>
    }
}
