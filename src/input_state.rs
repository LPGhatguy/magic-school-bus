use crate::{
    action::Action,
    terminal_context::TerminalContext,
};

#[derive(Debug)]
pub struct InputState {
    pub mode: InputMode,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputMode {
    Normal,
    FindNextInput, // Invoked via 'f'
    FindPreviousInput, // Invoked via 'F'
}

impl InputState {
    pub fn new() -> InputState {
        InputState {
            mode: InputMode::Normal,
        }
    }

    pub fn process_input(&mut self, context: &TerminalContext) -> Option<Action> {
        let input = context.crossterm.input();
        let key = input.read_char().ok()?;

        if key == '\u{1b}' {
            self.mode = InputMode::Normal;

            // TODO: Eliminate cancel action
            return Some(Action::Cancel);
        }

        match self.mode {
            InputMode::Normal => {
                match key {
                    // TODO: Convert from action to mutation
                    '0'...'9' => Some(Action::AddToRepeatBuffer(key)),

                    'f' => {
                        self.mode = InputMode::FindNextInput;
                        None
                    },
                    'F' => {
                        self.mode = InputMode::FindPreviousInput;
                        None
                    },

                    'q' => Some(Action::Quit),
                    'j' => Some(Action::Down),
                    'k' => Some(Action::Up),
                    'g' => Some(Action::Top),
                    'G' => Some(Action::Bottom),
                    '\r' => Some(Action::Select),
                    ';' => Some(Action::FindNext),
                    ',' => Some(Action::FindPrevious),

                    '[' => Some(Action::DebugDumpVisible),
                    _ => Some(Action::Unknown(key)),
                }
            },
            InputMode::FindNextInput => {
                self.mode = InputMode::Normal;
                Some(Action::SetAndFindNext(key))
            },
            InputMode::FindPreviousInput => {
                self.mode = InputMode::Normal;
                Some(Action::SetAndFindPrevious(key))
            },
        }
    }
}