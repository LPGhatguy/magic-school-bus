#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    Up,
    Down,
    Top,
    Bottom,
    Select,
    Cancel,
    AddToRepeatBuffer(char),
    EnterFindSpecifyMode,

    SetAndFindNext(char),
    SetAndFindPrevious(char),
    FindNext,
    FindPrevious,

    DebugDumpVisible,
    Unknown(char),
}

impl Action {
    pub fn should_repeat(&self) -> bool {
        match self {
            Action::Up | Action::Down => true,
            _ => false,
        }
    }

    pub fn should_consume_repeat(&self) -> bool {
        match self {
            Action::AddToRepeatBuffer(_) => false,
            _ => true,
        }
    }

    pub fn show_in_status_bar(&self) -> bool {
        match self {
            Action::AddToRepeatBuffer(_) => false,
            _ => true,
        }
    }
}