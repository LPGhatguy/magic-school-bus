use crossterm::{
    Crossterm,
    Screen,
    AlternateScreen,
};

pub struct TerminalContext {
    pub crossterm: Crossterm,

    alternate_screen: AlternateScreen,
}

impl TerminalContext {
    pub fn init() -> TerminalContext {
        let screen = Screen::default();
        let alternate = screen.enable_alternate_modes(true).unwrap();
        let crossterm = Crossterm::new(&alternate.screen);

        TerminalContext {
            crossterm,
            alternate_screen: alternate,
        }
    }

    pub fn get_screen(&self) -> &Screen {
        &self.alternate_screen.screen
    }

    pub fn get_terminal_size(&self) -> (usize, usize) {
        let terminal = self.crossterm.terminal();
        let (term_width, term_height) = {
            let size = terminal.terminal_size();
            (size.0 as usize, size.1 as usize)
        };

        (term_width + 1, term_height + 1)
    }
}