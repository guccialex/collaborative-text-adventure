use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub use shared::AdventureNode;

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
