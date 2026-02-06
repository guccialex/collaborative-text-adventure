use leptos::prelude::*;

#[component]
pub fn StoryHeader() -> impl IntoView {
    view! {
        <header class="story-header">
            <h1 class="story-heading">"Collaborative Text Adventure"</h1>
            <p class="story-lede">
                "Read existing stories, and branch into a new one at any point."
            </p>
        </header>
    }
}
