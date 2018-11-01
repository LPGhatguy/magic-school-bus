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

#[derive(Debug)]
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
        assert!(x < self.width);
        assert!(y < self.height);

        self.data[x + y * self.height] = block;
    }

    pub fn get_block(&self, x: usize, y: usize) -> Block {
        assert!(x < self.width);
        assert!(y < self.height);

        self.data[x + y * self.height]
    }
}

#[derive(Debug)]
pub struct VirtualScreen {
    width: usize,
    height: usize,
    visible: VirtualScreenBuffer,
    in_progress: VirtualScreenBuffer,
}

impl VirtualScreen {
    pub fn new(width: usize, height: usize) -> VirtualScreen {
        VirtualScreen {
            width,
            height,
            visible: VirtualScreenBuffer::new(width, height),
            in_progress: VirtualScreenBuffer::new(width, height),
        }
    }

    pub fn commit(&mut self, state: &mut State) {
        // TODO: Check if screen size changed, repaint everything

        let mut writes = Vec::new();

        for y in 0..self.width {
            for x in 0..self.height {
                let new_value = self.in_progress.get_block(x, y);
                let old_value = self.visible.get_block(x, y);

                if new_value != old_value {
                    self.visible.set_block(x, y, new_value);
                    writes.push(((x, y), new_value));
                }
            }
        }

        let cursor = state.crossterm.cursor();
        let mut buffer = String::new();

        for ((x, y), block) in writes {
            buffer.clear();
            buffer.write_char(block.char).unwrap();
            cursor.goto(x as u16, y as u16);
            crossterm::style(&buffer).with(block.fg.into()).on(block.bg.into()).paint(state.screen);
        }
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