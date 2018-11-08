use crate::{
    action::Action,
    terminal_context::TerminalContext,
};

#[derive(Debug)]
pub struct InputState {
    pub mode: InputMode,
    repeat_count_buffer: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    FindNextInput, // Invoked via 'f'
    FindPreviousInput, // Invoked via 'F'
}

impl InputState {
    pub fn new() -> InputState {
        InputState {
            mode: InputMode::Normal,
            repeat_count_buffer: String::new(),
        }
    }

    pub fn get_count_progress(&self) -> Option<&str> {
        if self.repeat_count_buffer.is_empty() {
            None
        } else {
            Some(&self.repeat_count_buffer)
        }
    }

    fn consume_repeat_count(&mut self) -> u64 {
        let count = self.repeat_count_buffer.parse::<u64>().unwrap_or(1);
        self.repeat_count_buffer.clear();

        count
    }

    fn process_input_internal(&mut self, context: &TerminalContext) -> Option<Action> {
        let input = context.crossterm.input();
        let key = input.read_char().ok()?;

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
                    'j' => Some(Action::Down(self.consume_repeat_count())),
                    'k' => Some(Action::Up(self.consume_repeat_count())),
                    'g' => Some(Action::Top),
                    'G' => Some(Action::Bottom),
                    '\r' => Some(Action::Select),
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
        }
    }

    pub fn process_input(&mut self, context: &TerminalContext) -> Option<Action> {
        let action = self.process_input_internal(context);

        if action.is_some() {
            self.repeat_count_buffer.clear();
        }

        action
    }
}