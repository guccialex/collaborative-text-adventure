use std::collections::HashMap;

use js_sys;
use serde::{Deserialize, Serialize};

/// The atomic building block of the adventure.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdventureNode {
    pub id: String,
    pub parent_id: Option<String>,
    pub choice_text: String,
    pub story_text: String,
}

impl AdventureNode {
    pub fn user(parent_id: &str, choice_text: String, story_text: String) -> Self {
        Self {
            id: format!("user_{}", js_sys::Date::now() as u64),
            parent_id: Some(parent_id.to_string()),
            choice_text,
            story_text,
        }
    }
}

/// Indexed adventure graph with fast lookups.
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct AdventureGraph {
    nodes: HashMap<String, AdventureNode>,
    children_by_parent: HashMap<String, Vec<String>>,
    root_id: Option<String>,
}

impl AdventureGraph {
    pub fn from_nodes(nodes: impl IntoIterator<Item = AdventureNode>) -> Self {
        let mut graph = AdventureGraph::default();
        for node in nodes {
            let node_id = node.id.clone();
            if let Some(parent_id) = node.parent_id.clone() {
                graph
                    .children_by_parent
                    .entry(parent_id)
                    .or_default()
                    .push(node_id.clone());
            } else {
                graph.root_id = Some(node_id.clone());
            }
            graph.nodes.insert(node_id, node);
        }
        graph
    }

    pub fn node(&self, id: &str) -> Option<&AdventureNode> {
        self.nodes.get(id)
    }

    pub fn root_id(&self) -> Option<&str> {
        self.root_id.as_deref()
    }

    pub fn root_path(&self) -> Vec<String> {
        self.root_id()
            .map(|id| vec![id.to_string()])
            .unwrap_or_default()
    }

    pub fn children(&self, parent_id: &str) -> Vec<&AdventureNode> {
        self.children_ids(parent_id)
            .iter()
            .filter_map(|id| self.nodes.get(id))
            .collect()
    }

    pub fn children_ids(&self, parent_id: &str) -> &[String] {
        self.children_by_parent
            .get(parent_id)
            .map(|ids| ids.as_slice())
            .unwrap_or(&[])
    }
}
