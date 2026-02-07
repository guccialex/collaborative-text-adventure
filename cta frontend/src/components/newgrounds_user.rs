use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::newgrounds::{check_session, get_session_id};

#[component]
pub fn NewgroundsUser() -> impl IntoView {
    let (username, set_username) = signal::<Option<String>>(None);

    Effect::new(move |_| {
        if let Some(session_id) = get_session_id() {
            spawn_local(async move {
                match check_session(&session_id).await {
                    Ok(Some(user)) => set_username.set(Some(user.name)),
                    Ok(None) => {}
                    Err(e) => log::error!("Newgrounds session check failed: {}", e),
                }
            });
        }
    });

    view! {
        <Show when=move || username.get().is_some()>
            <span class="ng-user">{move || username.get().unwrap_or_default()}</span>
        </Show>
    }
}
