use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};

// Modules
pub mod api;
pub mod config;
mod domain;
mod state;
mod components;
mod pages;

// Top-Level pages
use crate::pages::home::Home;
use crate::components::newgrounds_user::NewgroundsUser;
use crate::components::server_counter::ServerCounter;

/// An app router which renders the homepage and handles 404's
#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Html attr:lang="en" attr:dir="ltr" attr:data-theme="dark" />

        // sets the document title
        <Title text="Collaborative Text Adventure" />

        // injects metadata in the <head> of the page
        <Meta charset="UTF-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />

        <header class="app-header">
            <NewgroundsUser />
            <ServerCounter />
        </header>

        <Router>
            <Routes fallback=|| view! { <Home /> }>
                <Route path=path!("/") view=Home />
            </Routes>
        </Router>
    }
}
