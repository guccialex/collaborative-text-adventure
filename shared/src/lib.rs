use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdventureNode {
    pub id: String,
    pub parent_id: Option<String>,
    pub choice_text: String,
    pub story_text: String,
    #[serde(default)]
    pub created_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ServerMessage {
    RequestAdventureNodes,
    ReturnAdventureNodes(Vec<AdventureNode>),

    RequestDescendantCounts,
    ReturnDescendantCounts(HashMap<String, u64>),

    SubmitAdventureNode {
        node: AdventureNode,
        session_id: Option<String>,
    },

    Ok,
    Error(String),
}
