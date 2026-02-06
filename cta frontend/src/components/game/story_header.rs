use leptos::prelude::*;

#[component]
pub fn StoryHeader() -> impl IntoView {
    view! {
        <header class="story-header">
            <p class="story-eyebrow">"Live, community-written story"</p>
            <h1 class="story-heading">"Collaborative Text Adventure"</h1>
            <p class="story-lede">
                "Read, choose, and continue the thread. Every branch is written by players."
            </p>
        </header>
    }
}
