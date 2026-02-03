use std::collections::HashMap;

/// The atomic building block of the adventure.
#[derive(Clone, Debug, PartialEq)]
pub struct AdventureChoiceUnit {
    pub id: String,
    pub parent_id: Option<String>,
    pub choice_text: String,
    pub story_text: String,
}

/// Adventure data store with helper methods
#[derive(Clone, Default)]
pub struct Adventure {
    pub units: HashMap<String, AdventureChoiceUnit>,
}

impl Adventure {
    pub fn get(&self, id: &str) -> Option<&AdventureChoiceUnit> {
        self.units.get(id)
    }

    pub fn root(&self) -> Option<&AdventureChoiceUnit> {
        self.units.values().find(|u| u.parent_id.is_none())
    }

    pub fn children(&self, parent_id: &str) -> Vec<&AdventureChoiceUnit> {
        self.units.values()
            .filter(|u| u.parent_id.as_deref() == Some(parent_id))
            .collect()
    }
}

/// Fetch adventure data (replace with real API call later)
pub async fn fetch_adventure() -> Result<Adventure, String> {
    Ok(Adventure {
        units: crate::seed_data::get_seed_data(),
    })
}
