use std::io;

use crossterm::{
    Crossterm,
    Screen,
    AlternateScreen,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    White,
    Red,
}

impl Into<crossterm::Color> for Color {
    fn into(self) -> crossterm::Color {
        match self {
            Color::Black => crossterm::Color::Black,
            Color::White => crossterm::Color::White,
            Color::Red => crossterm::Color::Red,
        }
    }
}

pub struct TerminalContext {
    crossterm: Crossterm,
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

    pub fn get_terminal_size(&self) -> (usize, usize) {
        let terminal = self.crossterm.terminal();
        let (term_width, term_height) = {
            let size = terminal.terminal_size();
            (size.0 as usize, size.1 as usize)
        };

        (term_width + 1, term_height + 1)
    }

    pub fn read_char(&mut self) -> io::Result<char> {
        let input = self.crossterm.input();
        input.read_char()
    }

    pub fn paint_str(&mut self, text: &str, fg: Color, bg: Color) {
        crossterm::style(text)
            .with(fg.into())
            .on(bg.into())
            .paint(&self.alternate_screen.screen);
    }

    pub fn clear_screen(&mut self) {
        let terminal = self.crossterm.terminal();
        terminal.clear(crossterm::terminal::ClearType::All);
    }

    pub fn show_cursor(&mut self) {
        let cursor = self.crossterm.cursor();
        cursor.show();
    }

    pub fn hide_cursor(&mut self) {
        let cursor = self.crossterm.cursor();
        cursor.hide();
    }

    pub fn move_cursor(&mut self, x: usize, y: usize) {
        let cursor = self.crossterm.cursor();
        cursor.goto(x as u16, y as u16);
    }
}