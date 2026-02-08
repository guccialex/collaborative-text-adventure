use leptos::prelude::*;

#[component]
pub fn NewgroundsUser() -> impl IntoView {
    let username = use_context::<RwSignal<Option<String>>>()
        .expect("NG username signal must be provided by App");

    view! {
        <Show when=move || username.get().is_some()>
            <span class="ng-user">{move || username.get().unwrap_or_default()}</span>
        </Show>
    }
}
