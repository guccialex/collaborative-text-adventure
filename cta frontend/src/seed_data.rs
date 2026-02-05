//! Seed data for the adventure. Replace with API fetch in production.

use std::collections::HashMap;
use crate::api::AdventureChoiceUnit;

pub fn get_seed_data() -> HashMap<String, AdventureChoiceUnit> {
    [
        ("root", None, "Root", "This is a branching text adventure. Click on one of the options to read it and see its branching paths, or contribute one of your own at any point."),
        ("torch_passage", Some("root"), "Take the torch and explore the dark passage ahead", "You grab the torch from its sconce. The warmth is comforting against the chill. The passage ahead slopes downward, and you can hear the faint sound of dripping water echoing from somewhere deeper within."),
        ("search_chamber", Some("root"), "Search the chamber for clues about your identity", "You run your hands along the rough stone walls, searching for anything that might explain your situation. In a corner, your fingers brush against something metallic - a small iron key, covered in cobwebs. Near it lies a torn piece of parchment with faded writing."),
        ("call_out", Some("root"), "Call out into the darkness", "\"Hello?\" your voice echoes through the chamber, bouncing off unseen walls in the darkness. For a moment, silence. Then... footsteps. Slow, deliberate footsteps approaching from the passage ahead. A raspy voice calls back: \"Another one awakens...\""),
        ("deep_passage", Some("torch_passage"), "Continue deeper into the passage", "The passage opens into a vast underground cavern. Your torchlight barely reaches the ceiling high above. In the center, an ancient stone altar stands, covered in strange symbols that seem to glow faintly. Three corridors branch off in different directions."),
    ]
    .into_iter()
    .map(|(id, parent, choice, story)| {
        (id.into(), AdventureChoiceUnit {
            id: id.into(),
            parent_id: parent.map(Into::into),
            choice_text: choice.into(),
            story_text: story.into(),
        })
    })
    .collect()
}
