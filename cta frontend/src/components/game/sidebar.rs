use leptos::prelude::*;

use crate::domain::adventure::AdventureNode;
use crate::state::adventure::use_adventure_state;

use super::helpers::scroll_to_segment;

#[component]
pub fn Sidebar(
    segments: Memo<Vec<(usize, AdventureNode)>>,
) -> impl IntoView {
    let state = use_adventure_state();
    let path = state.path();

    view! {
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
                                        title="Branch from here"
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
        </aside>
    }
}
