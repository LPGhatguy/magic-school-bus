use std::fmt::Write;

use crossterm;

use crate::state::State;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    White,
}

impl Into<crossterm::Color> for Color {
    fn into(self) -> crossterm::Color {
        match self {
            Color::Black => crossterm::Color::Black,
            Color::White => crossterm::Color::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Block {
    fg: Color,
    bg: Color,
    char: char,
}

impl Default for Block {
    fn default() -> Block {
        Block {
            fg: Color::White,
            bg: Color::Black,
            char: ' ',
        }
    }
}

#[derive(Debug, Clone)]
pub struct VirtualScreenBuffer {
    width: usize,
    height: usize,
    data: Vec<Block>,
}

impl VirtualScreenBuffer {
    pub fn new(width: usize, height: usize) -> VirtualScreenBuffer {
        VirtualScreenBuffer {
            width,
            height,
            data: vec![Block::default(); width * height],
        }
    }

    pub fn set_block(&mut self, x: usize, y: usize, block: Block) {
        if x >= self.width || y >= self.height {
            panic!("Could not write ({}, {}) on screen size ({}, {})", x, y, self.width, self.height);
        }

        self.data[x + y * self.width] = block;
    }

    pub fn get_block(&self, x: usize, y: usize) -> Block {
        if x >= self.width || y >= self.height {
            panic!("Could not read ({}, {}) on screen size ({}, {})", x, y, self.width, self.height);
        }

        self.data[x + y * self.width]
    }

    pub fn write_str(&mut self, x: usize, y: usize, value: &str) {
        self.write_str_color(x, y, value, Color::White, Color::Black);
    }

    pub fn write_str_color(&mut self, start_x: usize, start_y: usize, value: &str, fg: Color, bg: Color) {
        let mut x = start_x;
        let mut y = start_y;

        for char in value.chars() {
            if x >= self.width || y >= self.height {
                break;
            }

            if char == '\n' {
                y += 1;
                x = start_x;
            } else {
                self.set_block(x, y, Block {
                    fg,
                    bg,
                    char,
                });

                x += 1;
            }
        }
    }
}

#[derive(Debug)]
pub struct VirtualScreen {
    visible: VirtualScreenBuffer,
    in_progress: VirtualScreenBuffer,
    should_clear: bool,
}

impl VirtualScreen {
    pub fn new(width: usize, height: usize) -> VirtualScreen {
        VirtualScreen {
            visible: VirtualScreenBuffer::new(width, height),
            in_progress: VirtualScreenBuffer::new(width, height),
            should_clear: false,
        }
    }

    pub fn write_str(&mut self, x: usize, y: usize, value: &str) {
        self.in_progress.write_str(x, y, value);
    }

    pub fn write_str_color(&mut self, x: usize, y: usize, value: &str, fg: Color, bg: Color) {
        self.in_progress.write_str_color(x, y, value, fg, bg);
    }

    pub fn get_size(&self) -> (usize, usize) {
        (self.in_progress.width, self.in_progress.height)
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.visible = VirtualScreenBuffer::new(width, height);
        self.in_progress = VirtualScreenBuffer::new(width, height);
        self.should_clear = true;
    }

    fn commit_all(&mut self, state: &mut State) {
        let cursor = state.crossterm.cursor();
        let mut buffer = String::new();
        let (width, height) = self.get_size();

        for y in 0..height {
            for x in 0..width {
                let block = self.in_progress.get_block(x, y);

                buffer.clear();
                buffer.write_char(block.char).unwrap();
                cursor.goto(x as u16, y as u16);
                crossterm::style(&buffer).with(block.fg.into()).on(block.bg.into()).paint(state.screen);
            }
        }

        self.visible = self.in_progress.clone();
    }

    fn commit_some(&mut self, state: &mut State, changes: &[((usize, usize), Block)]) {
        let cursor = state.crossterm.cursor();
        let mut buffer = String::new();

        for ((x, y), block) in changes {
            buffer.clear();
            buffer.write_char(block.char).unwrap();
            cursor.goto(*x as u16, *y as u16);
            crossterm::style(&buffer).with(block.fg.into()).on(block.bg.into()).paint(state.screen);
        }
    }

    pub fn prepaint(&mut self, state: &mut State) {
        let terminal = state.crossterm.terminal();
        let (term_width, term_height) = {
            let size = terminal.terminal_size();
            (size.0 as usize, size.1 as usize)
        };
        let (width, height) = self.get_size();

        if term_width != width || term_height != height {
            self.resize(term_width, term_height);
        }
    }

    pub fn commit(&mut self, state: &mut State) {
        let terminal = state.crossterm.terminal();

        if self.should_clear {
            self.should_clear = false;
            terminal.clear(crossterm::terminal::ClearType::All);
            self.commit_all(state);
            return;
        }

        let (width, height) = self.get_size();
        let mut changes = Vec::new();

        let mut x = 0;
        let mut y = 0;
        loop {
            let new_value = self.in_progress.get_block(x, y);
            let old_value = self.visible.get_block(x, y);

            if new_value != old_value {
                self.visible.set_block(x, y, new_value);
                changes.push(((x, y), new_value));
            }

            self.in_progress.set_block(x, y, Block::default());

            x += 1;
            if x == width {
                x = 0;
                y += 1;
            }

            if y == height {
                break;
            }
        }

        self.commit_some(state, &changes);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_and_set() {
        let (width, height) = (18, 18);
        let screen = VirtualScreenBuffer::new(width, height);
        let default_block = Block::default();

        assert_eq!(screen.get_block(0, 0), default_block);
        assert_eq!(screen.get_block(width - 1, 0), default_block);
        assert_eq!(screen.get_block(0, height - 1), default_block);
        assert_eq!(screen.get_block(width - 1, height - 1), default_block);
    }
}