use all_term::Key;

use crate::{
    action::Action,
    terminal_context::TerminalContext,
};

#[derive(Debug)]
pub struct InputState {
    mode: InputMode,
    repeat_count_buffer: String,
    text_buffer: Vec<char>,
    text_cursor: usize,
}

/// Magic School Bus is loosely modal. InputMode is the value that determines
/// what keys map to what actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// The mode from which most commands are started.
    Normal,

    /// The user is entering a search string to find files.
    FindPrompt,

    /// The user is being prompted to delete one or more entries.
    DeletePrompt,

    /// The user is entering a name for a new file.
    NewFilePrompt,

    /// The user is entering a name for a new directory.
    NewDirectoryPrompt,

    /// The user is entering a command to run.
    CommandPrompt,
}

impl InputState {
    pub fn new() -> InputState {
        InputState {
            mode: InputMode::Normal,
            repeat_count_buffer: String::new(),
            text_buffer: Vec::new(),
            text_cursor: 0,
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
        self.text_cursor
    }

    pub fn get_text_buffer(&self) -> &[char] {
        &self.text_buffer
    }

    fn consume_repeat_count(&mut self) -> u64 {
        let count = self.repeat_count_buffer.parse::<u64>().unwrap_or(1);
        self.repeat_count_buffer.clear();

        count
    }

    fn handle_text_key(&mut self, key: Key) {
        match key {
            Key::Backspace => {
                if self.text_cursor > 0 {
                    self.text_buffer.remove(self.text_cursor - 1);
                    self.text_cursor -= 1;
                }
            },
            Key::Char(char) => {
                self.text_buffer.insert(self.text_cursor, char);
                self.text_cursor += 1;
            },
            Key::Left => {
                if self.text_cursor > 0 {
                    self.text_cursor -= 1;
                }
            },
            Key::Right => {
                if self.text_cursor < self.text_buffer.len() {
                    self.text_cursor += 1;
                }
            },
            _ => {},
        }
    }

    fn process_input_internal(&mut self, context: &mut TerminalContext) -> Option<Action> {
        let key = context.read_key();

        if key == Key::Escape {
            self.mode = InputMode::Normal;
            return Some(Action::Cancel);
        }

        match self.mode {
            InputMode::Normal => {
                match key {
                    Key::Char('q') => Some(Action::Quit),
                    Key::Char(char @ '0'...'9') => {
                        self.repeat_count_buffer.push(char);
                        None
                    },
                    Key::Char('f') => {
                        self.text_cursor = 0;
                        self.text_buffer.clear();
                        self.mode = InputMode::FindPrompt;

                        Some(Action::Find(String::new()))
                    },
                    Key::Char('n') => {
                        self.text_cursor = 0;
                        self.text_buffer.clear();
                        self.mode = InputMode::NewFilePrompt;
                        None
                    },
                    Key::Char('N') => {
                        self.text_cursor = 0;
                        self.text_buffer.clear();
                        self.mode = InputMode::NewDirectoryPrompt;
                        None
                    },
                    Key::Char(':') => {
                        self.text_cursor = 0;
                        self.text_buffer.clear();
                        self.mode = InputMode::CommandPrompt;
                        None
                    },
                    Key::Char('j') | Key::Down => Some(Action::Down(self.consume_repeat_count())),
                    Key::Char('k') | Key::Up => Some(Action::Up(self.consume_repeat_count())),
                    Key::Char('g') => Some(Action::Top),
                    Key::Char('G') => Some(Action::Bottom),
                    Key::Char('r') => Some(Action::Refresh),
                    Key::Char('x') => {
                        self.repeat_count_buffer.clear();
                        self.mode = InputMode::DeletePrompt;
                        None
                    },
                    Key::Char('\n') => Some(Action::Activate),

                    Key::Char('[') => Some(Action::DebugDumpVisible),
                    _ => Some(Action::Unknown(key)),
                }
            },
            InputMode::DeletePrompt => {
                match key {
                    Key::Char('y') => {
                        self.mode = InputMode::Normal;
                        Some(Action::Delete)
                    },
                    _ => None,
                }
            },
            InputMode::FindPrompt => {
                match key {
                    Key::Char('\n') => {
                        self.mode = InputMode::Normal;
                        None
                    },
                    Key::Char('\t') => {
                        Some(Action::FindNext)
                    },
                    _ => {
                        self.handle_text_key(key);
                        let text: String = self.text_buffer.iter().collect();

                        Some(Action::Find(text))
                    },
                }
            },
            InputMode::CommandPrompt => {
                match key {
                    Key::Char('\n') => {
                        let text: String = self.text_buffer.iter().collect();
                        self.mode = InputMode::Normal;

                        Some(Action::RunCommand(text))
                    },
                    _ => {
                        self.handle_text_key(key);
                        None
                    },
                }
            },
            InputMode::NewFilePrompt => {
                match key {
                    Key::Char('\n') => {
                        let text: String = self.text_buffer.iter().collect();
                        self.mode = InputMode::Normal;

                        Some(Action::CreateFile(text))
                    },
                    _ => {
                        self.handle_text_key(key);
                        None
                    },
                }
            },
            InputMode::NewDirectoryPrompt => {
                match key {
                    Key::Char('\n') => {
                        let text: String = self.text_buffer.iter().collect();
                        self.mode = InputMode::Normal;

                        Some(Action::CreateDirectory(text))
                    },
                    _ => {
                        self.handle_text_key(key);
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