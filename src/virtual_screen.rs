use crossterm;

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
pub struct VirtualScreen {
    width: usize,
    height: usize,
    data: Vec<Block>,
}

impl VirtualScreen {
    pub fn new(width: usize, height: usize) -> VirtualScreen {
        VirtualScreen {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_and_set() {
        let (width, height) = (18, 18);
        let screen = VirtualScreen::new(width, height);
        let default_block = Block::default();

        assert_eq!(screen.get_block(0, 0), default_block);
        assert_eq!(screen.get_block(width - 1, 0), default_block);
        assert_eq!(screen.get_block(0, height - 1), default_block);
        assert_eq!(screen.get_block(width - 1, height - 1), default_block);
    }
}