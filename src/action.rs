/// Describes a complete operation that the user can perform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Exits Magic School Bus.
    Quit,

    /// Cancels the current operation and returns the user to normal mode.
    Cancel,

    /// Moves the cursor up `count` times.
    Up(u64),

    /// Moves the cursor down `count` times.
    Down(u64),

    /// Moves the cursor to the top of the list.
    Top,

    /// Moves the cursor to the bottom of the list.
    Bottom,

    /// Activates the selected entry, opening it according to operating system
    /// preferences.
    Activate,

    /// Deletes the selected entries.
    Delete,

    /// Creates a file here.
    CreateFile(String),

    /// Creates a directory here.
    CreateDirectory(String),

    /// Refreshes the directory view.
    Refresh,

    /// Finds the next entry after the cursor that starts with `char`. Repeats
    /// `count` times.
    SetAndFindNext(u64, char),

    /// Finds the previous entry before the cursor that starts with `char`.
    /// Repeats `count` times.
    SetAndFindPrevious(u64, char),

    /// Finds the next entry after the cursor using the search from the previous
    /// find command, `SetAndFindNext` or `SetAndFindPrevious`. Repeats `count`
    /// times.
    FindNext(u64),

    /// Finds the previous entry before the cursor using the search from the
    /// previous find command, `SetAndFindNext` or `SetAndFindPrevious`. Repeats
    /// `count` times.
    FindPrevious(u64),

    /// Run a command issued by the command bar.
    RunCommand(String),

    /// Dumps the current buffer to stderr to be inspected when the program
    /// closes. Usually used to debug the virtual screen implementation.
    DebugDumpVisible,

    /// A fallback action to indicate to the user why a key doesn't do anything.
    Unknown(char),
}