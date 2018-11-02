use crossterm::{
    Crossterm,
    Screen,
};

pub struct TerminalContext<'a> {
    pub crossterm: &'a Crossterm,
    pub screen: &'a Screen,
}

impl<'a> TerminalContext<'a> {
    pub fn get_terminal_size(&self) -> (usize, usize) {
        let terminal = self.crossterm.terminal();
        let (term_width, term_height) = {
            let size = terminal.terminal_size();
            (size.0 as usize, size.1 as usize)
        };

        (term_width + 1, term_height + 1)
    }
}