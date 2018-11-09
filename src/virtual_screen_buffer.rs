use crate::{
    virtual_screen::{Block, Color},
};

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

    pub fn get_size(&self) -> (usize, usize) {
        (self.width, self.height)
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