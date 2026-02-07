use std::collections::HashMap;

use crate::domain::adventure::AdventureGraph;
use shared::{AdventureNode, ServerMessage};

pub async fn fetch_adventure() -> Result<AdventureGraph, String> {
    match super::api_fetch(ServerMessage::RequestAdventureNodes).await? {
        ServerMessage::ReturnAdventureNodes(nodes) => Ok(AdventureGraph::from_nodes(nodes)),
        ServerMessage::Error(e) => Err(e),
        other => Err(format!("Unexpected response: {:?}", other)),
    }
}

pub async fn fetch_descendant_counts() -> Result<HashMap<String, u64>, String> {
    match super::api_fetch(ServerMessage::RequestDescendantCounts).await? {
        ServerMessage::ReturnDescendantCounts(counts) => Ok(counts),
        ServerMessage::Error(e) => Err(e),
        other => Err(format!("Unexpected response: {:?}", other)),
    }
}

pub async fn submit_node(node: AdventureNode, session_id: Option<String>) -> Result<(), String> {
    match super::api_fetch(ServerMessage::SubmitAdventureNode { node, session_id }).await? {
        ServerMessage::Ok => Ok(()),
        ServerMessage::Error(e) => Err(e),
        other => Err(format!("Unexpected response: {:?}", other)),
    }
}
