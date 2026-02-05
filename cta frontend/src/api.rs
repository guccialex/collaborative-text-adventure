/// Fetch adventure data (replace with real API call later)
use crate::domain::adventure::AdventureGraph;
use crate::seed_data::seed_nodes;

pub async fn fetch_adventure() -> Result<AdventureGraph, String> {
    Ok(AdventureGraph::from_nodes(seed_nodes()))
}
