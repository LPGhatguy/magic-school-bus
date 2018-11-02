use std::fmt::Write;

use crossterm;

use crate::{
    terminal_context::TerminalContext,
};

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

    pub fn clear(&mut self) {
        for i in 0..(self.width * self.height) {
            self.data[i] = Block::default();
        }
    }

    pub fn copy_from(&mut self, other: &VirtualScreenBuffer) {
        assert!(self.width == other.width);
        assert!(self.height == other.height);

        self.data.copy_from_slice(&other.data);
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

    pub fn show(&self) -> String {
        let mut output = String::new();

        for y in 0..self.height {
            for x in 0..self.width {
                output.push(self.get_block(x, y).char);
            }
            output.push('\n');
        }

        output
    }
}

#[derive(Debug)]
struct Difference {
    x: usize,
    y: usize,
    text: String,
    fg: Color,
    bg: Color,
}

#[derive(Debug)]
pub struct VirtualScreen {
    visible_buffer: VirtualScreenBuffer,
    active_buffer: VirtualScreenBuffer,
    should_redraw_everything: bool,
}

impl VirtualScreen {
    pub fn new(width: usize, height: usize) -> VirtualScreen {
        VirtualScreen {
            visible_buffer: VirtualScreenBuffer::new(width, height),
            active_buffer: VirtualScreenBuffer::new(width, height),
            should_redraw_everything: false,
        }
    }

    pub fn show_visible_buffer(&self) -> String {
        self.visible_buffer.show()
    }

    pub fn write_str(&mut self, x: usize, y: usize, value: &str) {
        self.active_buffer.write_str(x, y, value);
    }

    pub fn write_str_color(&mut self, x: usize, y: usize, value: &str, fg: Color, bg: Color) {
        self.active_buffer.write_str_color(x, y, value, fg, bg);
    }

    pub fn get_size(&self) -> (usize, usize) {
        (self.active_buffer.width, self.active_buffer.height)
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.visible_buffer = VirtualScreenBuffer::new(width, height);
        self.active_buffer = VirtualScreenBuffer::new(width, height);
        self.should_redraw_everything = true;
    }

    fn commit_whole_screen(&mut self, context: &TerminalContext) {
        let cursor = context.crossterm.cursor();
        let mut buffer = String::new();
        let (width, height) = self.get_size();

        for y in 0..height {
            // TODO: Use same clustering algorithm to draw more quickly
            for x in 0..width {
                let block = self.active_buffer.get_block(x, y);

                buffer.clear();
                buffer.write_char(block.char).unwrap();
                cursor.goto(x as u16, y as u16);
                crossterm::style(&buffer).with(block.fg.into()).on(block.bg.into()).paint(context.get_screen());
            }
        }

        self.visible_buffer.copy_from(&self.active_buffer);
    }

    fn commit_changes(&mut self, context: &TerminalContext, changes: &[Difference]) {
        let cursor = context.crossterm.cursor();

        for change in changes {
            cursor.goto(change.x as u16, change.y as u16);
            crossterm::style(&change.text)
                .with(change.fg.into())
                .on(change.bg.into())
                .paint(context.get_screen());
        }
    }

    pub fn prepaint(&mut self, context: &TerminalContext) {
        let (term_width, term_height) = context.get_terminal_size();
        let (width, height) = self.get_size();

        if term_width != width || term_height != height {
            self.resize(term_width, term_height);
        }

        self.active_buffer.clear();
    }

    pub fn commit(&mut self, context: &TerminalContext) {
        let terminal = context.crossterm.terminal();

        if self.should_redraw_everything {
            self.should_redraw_everything = false;
            terminal.clear(crossterm::terminal::ClearType::All);
            self.commit_whole_screen(context);
            return;
        }

        let (width, height) = self.get_size();
        let mut changes = Vec::new();

        let mut x = 0;
        let mut y = 0;
        loop {
            let new_block = self.active_buffer.get_block(x, y);
            let old_block = self.visible_buffer.get_block(x, y);

            if new_block != old_block {
                self.visible_buffer.set_block(x, y, new_block);

                let mut text = new_block.char.to_string();

                let change_x = x;
                let change_y = y;

                // Attempt to cluster contiguous text with the same colors in
                // order to reduce the number of changes to the actual screen.
                loop {
                    if x + 1 == width {
                        break;
                    }

                    let next_block = self.active_buffer.get_block(x + 1, y);

                    if next_block.fg != new_block.fg || next_block.bg != new_block.bg {
                        break;
                    }

                    self.visible_buffer.set_block(x + 1, y, next_block);
                    text.push(next_block.char);
                    x += 1;
                }

                changes.push(Difference {
                    x: change_x,
                    y: change_y,
                    text,
                    fg: new_block.fg,
                    bg: new_block.bg,
                });
            }

            x += 1;
            if x == width {
                x = 0;
                y += 1;
            }

            if y == height {
                break;
            }
        }

        self.commit_changes(context, &changes);
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