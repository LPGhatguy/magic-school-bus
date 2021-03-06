use std::{
    io,
    sync::{Arc, Mutex},
};

use all_term::{terminal, Style, Terminal};

pub use all_term::{Color, Key};

pub struct TerminalContext {
    terminal: Arc<Mutex<Terminal>>,
}

impl TerminalContext {
    pub fn init() -> TerminalContext {
        let terminal = terminal();

        {
            let mut handle = terminal.lock().unwrap();
            handle.enable_raw_mode();
            handle.enable_alternate_screen();
        }

        TerminalContext { terminal }
    }

    pub fn get_terminal_size(&self) -> (usize, usize) {
        let handle = self.terminal.lock().unwrap();
        handle.get_size()
    }

    pub fn read_key(&mut self) -> Key {
        let mut handle = self.terminal.lock().unwrap();
        handle.read_key()
    }

    pub fn read_char(&mut self) -> io::Result<char> {
        // TODO: Port everything from read_char to read_key
        match self.read_key() {
            Key::Char(char) => Ok(char),
            _ => Ok(' '),
        }
    }

    pub fn paint_str(&mut self, text: &str, fg: Color, bg: Color) {
        let mut handle = self.terminal.lock().unwrap();
        handle.print(text, Style::new().fg(fg).bg(bg));
    }

    pub fn clear_screen(&mut self) {
        let mut handle = self.terminal.lock().unwrap();
        handle.clear_screen();
    }

    pub fn show_cursor(&mut self) {
        let mut handle = self.terminal.lock().unwrap();
        handle.show_cursor();
    }

    pub fn hide_cursor(&mut self) {
        let mut handle = self.terminal.lock().unwrap();
        handle.hide_cursor();
    }

    pub fn move_cursor(&mut self, x: usize, y: usize) {
        let mut handle = self.terminal.lock().unwrap();
        handle.move_cursor(x, y);
    }
}
