use crate::{terminal_context::Color, virtual_screen::ScreenCell};

#[derive(Debug, Clone)]
pub struct VirtualScreenBuffer {
    width: usize,
    height: usize,
    data: Vec<ScreenCell>,
    pub cursor_position: Option<(usize, usize)>,
}

impl VirtualScreenBuffer {
    pub fn new(width: usize, height: usize) -> VirtualScreenBuffer {
        VirtualScreenBuffer {
            width,
            height,
            data: vec![ScreenCell::default(); width * height],
            cursor_position: None,
        }
    }

    pub fn get_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn clear(&mut self) {
        for i in 0..(self.width * self.height) {
            self.data[i] = ScreenCell::default();
        }
        self.cursor_position = None;
    }

    pub fn copy_from(&mut self, other: &VirtualScreenBuffer) {
        assert!(self.width == other.width);
        assert!(self.height == other.height);

        self.data.copy_from_slice(&other.data);
    }

    pub fn set_block(&mut self, x: usize, y: usize, block: ScreenCell) {
        if x >= self.width || y >= self.height {
            panic!(
                "Could not write ({}, {}) on screen size ({}, {})",
                x, y, self.width, self.height
            );
        }

        self.data[x + y * self.width] = block;
    }

    pub fn get_block(&self, x: usize, y: usize) -> ScreenCell {
        if x >= self.width || y >= self.height {
            panic!(
                "Could not read ({}, {}) on screen size ({}, {})",
                x, y, self.width, self.height
            );
        }

        self.data[x + y * self.width]
    }

    pub fn write_str(&mut self, x: usize, y: usize, value: &str) {
        self.write_str_color(x, y, value, Color::Reset, Color::Reset);
    }

    pub fn write_str_color(
        &mut self,
        start_x: usize,
        start_y: usize,
        value: &str,
        fg: Color,
        bg: Color,
    ) {
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
                self.set_block(x, y, ScreenCell { fg, bg, char });

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
