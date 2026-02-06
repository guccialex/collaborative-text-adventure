use crate::domain::adventure::AdventureGraph;
use shared::{AdventureNode, ServerMessage};

pub async fn fetch_adventure() -> Result<AdventureGraph, String> {
    match super::api_fetch(ServerMessage::RequestAdventureNodes).await? {
        ServerMessage::ReturnAdventureNodes(nodes) => Ok(AdventureGraph::from_nodes(nodes)),
        ServerMessage::Error(e) => Err(e),
        other => Err(format!("Unexpected response: {:?}", other)),
    }
}

pub async fn submit_node(node: AdventureNode) -> Result<(), String> {
    match super::api_fetch(ServerMessage::SubmitAdventureNode(node)).await? {
        ServerMessage::Ok => Ok(()),
        ServerMessage::Error(e) => Err(e),
        other => Err(format!("Unexpected response: {:?}", other)),
    }
}
