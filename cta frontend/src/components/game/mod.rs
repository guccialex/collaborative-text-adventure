mod contribute_form;
mod game_layout;
mod helpers;
mod sidebar;
mod story_header;
mod story_scroll;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ContributeMode {
    DeadEnd,
    Branch,
    NewStory,
}

impl ContributeMode {
    pub fn title(self) -> &'static str {
        match self {
            Self::DeadEnd => "Continue the story",
            Self::Branch => "Add a new path",
            Self::NewStory => "Start a new story",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            Self::DeadEnd => "A next path hasn't been written yet. Add one.",
            Self::Branch => "Create a new option branching from this point.",
            Self::NewStory => "Write the opening of a brand new adventure.",
        }
    }
}

pub use game_layout::Game;
