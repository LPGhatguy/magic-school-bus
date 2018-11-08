#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    Up(u64),
    Down(u64),
    Top,
    Bottom,
    Select,
    Cancel,

    SetAndFindNext(u64, char),
    SetAndFindPrevious(u64, char),
    FindNext(u64),
    FindPrevious(u64),

    DebugDumpVisible,
    Unknown(char),
}