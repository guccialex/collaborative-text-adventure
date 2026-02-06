use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::adventure::{fetch_adventure, submit_node};
use crate::domain::adventure::{AdventureGraph, AdventureNode};

#[derive(Clone)]
pub enum LoadState {
    Loading,
    Ready,
    Error(String),
}

#[derive(Clone, Copy)]
pub struct AdventureState {
    graph: RwSignal<AdventureGraph>,
    load_state: RwSignal<LoadState>,
    path: RwSignal<Vec<String>>,
    show_contribute: RwSignal<bool>,
}

impl AdventureState {
    pub fn new() -> Self {
        let state = Self {
            graph: RwSignal::new(AdventureGraph::default()),
            load_state: RwSignal::new(LoadState::Loading),
            path: RwSignal::new(Vec::new()),
            show_contribute: RwSignal::new(false),
        };
        state.reload();
        state
    }

    pub fn graph(&self) -> RwSignal<AdventureGraph> {
        self.graph
    }

    pub fn load_state(&self) -> RwSignal<LoadState> {
        self.load_state
    }

    pub fn path(&self) -> RwSignal<Vec<String>> {
        self.path
    }

    pub fn show_contribute(&self) -> RwSignal<bool> {
        self.show_contribute
    }

    pub fn reload(&self) {
        let graph = self.graph;
        let load_state = self.load_state;
        let path = self.path;
        let show_contribute = self.show_contribute;

        load_state.set(LoadState::Loading);
        spawn_local(async move {
            match fetch_adventure().await {
                Ok(data) => {
                    let root_path = data.root_path();
                    graph.set(data);
                    path.set(root_path);
                    show_contribute.set(false);
                    load_state.set(LoadState::Ready);
                }
                Err(error) => {
                    load_state.set(LoadState::Error(error));
                }
            }
        });
    }

    pub fn choose(&self, node: &AdventureNode) {
        self.path.update(|path| path.push(node.id.clone()));
        self.show_contribute.set(false);
    }

    pub fn revert_to(&self, index: usize) {
        self.path.update(|path| path.truncate(index + 1));
        self.show_contribute.set(false);
    }

    pub fn toggle_contribute(&self) {
        self.show_contribute.update(|value| *value = !*value);
    }

    pub fn close_contribute(&self) {
        self.show_contribute.set(false);
    }

    pub fn add_node(&self, node: AdventureNode) {
        let graph = self.graph;
        let load_state = self.load_state;
        let path = self.path;
        let show_contribute = self.show_contribute;
        let new_node_id = node.id.clone();

        spawn_local(async move {
            match submit_node(node).await {
                Ok(()) => {
                    match fetch_adventure().await {
                        Ok(data) => {
                            graph.set(data);
                            path.update(|p| p.push(new_node_id));
                            show_contribute.set(false);
                            load_state.set(LoadState::Ready);
                        }
                        Err(error) => {
                            load_state.set(LoadState::Error(error));
                        }
                    }
                }
                Err(error) => {
                    log::error!("Failed to submit node: {}", error);
                    load_state.set(LoadState::Error(error));
                }
            }
        });
    }

    pub fn _reset_path(&self) {
        let root_path = self.graph.get().root_path();
        self.path.set(root_path);
        self.show_contribute.set(false);
    }
}

pub fn provide_adventure_state() {
    let state = AdventureState::new();
    provide_context(state);
}

pub fn use_adventure_state() -> AdventureState {
    use_context::<AdventureState>().expect("AdventureState must be provided by an ancestor")
}
