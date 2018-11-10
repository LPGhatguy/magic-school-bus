use crate::{
    action::Action,
    terminal_context::TerminalContext,
};

#[derive(Debug)]
pub struct InputState {
    mode: InputMode,
    repeat_count_buffer: String,
    command_buffer: Vec<char>,
    command_cursor: usize,
}

/// Magic School Bus is loosely modal. InputMode is the value that determines
/// what keys map to what actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// The mode from which most commands are started.
    Normal,

    /// Indicates that the next input should be processed as the
    /// `SetAndFindNext` action.
    FindNextInput,

    /// Indicates that the next input should be processed as the
    /// `SetAndFindPrevious` action.
    FindPreviousInput,

    /// Indicates that the user is being prompted to delete one or more entries.
    DeletePrompt,

    /// Command line mode, inputs should edit text.
    Command,
}

impl InputState {
    pub fn new() -> InputState {
        InputState {
            mode: InputMode::Normal,
            repeat_count_buffer: String::new(),
            command_buffer: Vec::new(),
            command_cursor: 0,
        }
    }

    pub fn get_mode(&self) -> InputMode {
        self.mode
    }

    pub fn get_count_progress(&self) -> Option<&str> {
        if self.repeat_count_buffer.is_empty() {
            None
        } else {
            Some(&self.repeat_count_buffer)
        }
    }

    pub fn get_cursor_position(&self) -> usize {
        self.command_cursor
    }

    pub fn get_command_buffer(&self) -> &[char] {
        &self.command_buffer
    }

    fn consume_repeat_count(&mut self) -> u64 {
        let count = self.repeat_count_buffer.parse::<u64>().unwrap_or(1);
        self.repeat_count_buffer.clear();

        count
    }

    fn process_input_internal(&mut self, context: &mut TerminalContext) -> Option<Action> {
        let key = context.read_char().ok()?;

        if key == '\u{1b}' {
            self.mode = InputMode::Normal;
            return Some(Action::Cancel);
        }

        match self.mode {
            InputMode::Normal => {
                match key {
                    'q' => Some(Action::Quit),
                    '0'...'9' => {
                        self.repeat_count_buffer.push(key);
                        None
                    },
                    'f' => {
                        self.mode = InputMode::FindNextInput;
                        None
                    },
                    'F' => {
                        self.mode = InputMode::FindPreviousInput;
                        None
                    },
                    ':' => {
                        self.command_cursor = 0;
                        self.command_buffer.clear();
                        self.mode = InputMode::Command;
                        None
                    },
                    'j' => Some(Action::Down(self.consume_repeat_count())),
                    'k' => Some(Action::Up(self.consume_repeat_count())),
                    'g' => Some(Action::Top),
                    'G' => Some(Action::Bottom),
                    'x' => {
                        self.repeat_count_buffer.clear();
                        self.mode = InputMode::DeletePrompt;
                        None
                    },
                    '\r' => Some(Action::Activate),
                    ';' => Some(Action::FindNext(self.consume_repeat_count())),
                    ',' => Some(Action::FindPrevious(self.consume_repeat_count())),

                    '[' => Some(Action::DebugDumpVisible),
                    _ => Some(Action::Unknown(key)),
                }
            },
            InputMode::FindNextInput => {
                self.mode = InputMode::Normal;
                Some(Action::SetAndFindNext(self.consume_repeat_count(), key))
            },
            InputMode::FindPreviousInput => {
                self.mode = InputMode::Normal;
                Some(Action::SetAndFindPrevious(self.consume_repeat_count(), key))
            },
            InputMode::DeletePrompt => {
                match key {
                    'y' => {
                        self.mode = InputMode::Normal;
                        Some(Action::Delete)
                    },
                    _ => None,
                }
            },
            InputMode::Command => {
                match key {
                    '\r' => {
                        let command: String = self.command_buffer.iter().collect();
                        self.mode = InputMode::Normal;
                        Some(Action::RunCommand(command))
                    },
                    '\u{8}' => {
                        if self.command_buffer.pop().is_some() {
                            self.command_cursor -= 1;
                        }
                        None
                    },
                    _ => {
                        self.command_buffer.push(key);
                        self.command_cursor = self.command_buffer.len();
                        None
                    },
                }
            },
        }
    }

    pub fn process_input(&mut self, context: &mut TerminalContext) -> Option<Action> {
        let action = self.process_input_internal(context);

        if action.is_some() {
            self.repeat_count_buffer.clear();
        }

        action
    }
}

impl Default for InputState {
    fn default() -> InputState {
        InputState::new()
    }
}