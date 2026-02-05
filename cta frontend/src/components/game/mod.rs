mod contribute_form;
mod game_layout;
mod helpers;
mod sidebar;
mod story_header;
mod story_scroll;

#[derive(Clone, Copy)]
pub(crate) enum ContributeMode {
    DeadEnd,
    Branch,
}

impl ContributeMode {
    pub fn title(self) -> &'static str {
        match self {
            Self::DeadEnd => "Continue the story",
            Self::Branch => "Add a new path",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            Self::DeadEnd => "This path hasn't been written yet. Be the first to add to it.",
            Self::Branch => "Create a new option branching from this point.",
        }
    }
}

pub use game_layout::Game;
