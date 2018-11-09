use crossterm;

use crate::{
    terminal_context::TerminalContext,
    virtual_screen_buffer::VirtualScreenBuffer,
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
pub struct ScreenCell {
    pub fg: Color,
    pub bg: Color,
    pub char: char,
}

impl Default for ScreenCell {
    fn default() -> ScreenCell {
        ScreenCell {
            fg: Color::White,
            bg: Color::Black,
            char: ' ',
        }
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

struct DifferenceIterator<'a> {
    current: &'a VirtualScreenBuffer,
    previous: &'a VirtualScreenBuffer,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    all_dirty: bool,
}

impl<'a> DifferenceIterator<'a> {
    pub fn new(
        current: &'a VirtualScreenBuffer,
        previous: &'a VirtualScreenBuffer,
        all_dirty: bool,
    ) -> DifferenceIterator<'a> {
        let (width, height) = current.get_size();

        DifferenceIterator {
            current,
            previous,
            x: 0,
            y: 0,
            width,
            height,
            all_dirty,
        }
    }

    fn step(&mut self) {
        self.x += 1;
        if self.x == self.width {
            self.x = 0;
            self.y += 1;
        }
    }
}

impl<'a> Iterator for DifferenceIterator<'a> {
    type Item = Difference;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.y >= self.height {
                return None;
            }

            let new_block = self.current.get_block(self.x, self.y);
            let old_block = self.previous.get_block(self.x, self.y);

            if new_block != old_block || self.all_dirty {
                let mut text = new_block.char.to_string();

                let change_x = self.x;
                let change_y = self.y;

                // Attempt to cluster contiguous text with the same colors in
                // order to reduce the number of changes to the actual screen.
                loop {
                    if self.x + 1 == self.width {
                        break;
                    }

                    let next_block = self.current.get_block(self.x + 1, self.y);

                    if next_block.fg != new_block.fg || next_block.bg != new_block.bg {
                        break;
                    }

                    text.push(next_block.char);
                    self.x += 1;
                }

                self.step();
                return Some(Difference {
                    x: change_x,
                    y: change_y,
                    text,
                    fg: new_block.fg,
                    bg: new_block.bg,
                });
            }

            self.step();
        }
    }
}

#[derive(Debug)]
pub struct VirtualScreen {
    previous_buffer: VirtualScreenBuffer,
    current_buffer: VirtualScreenBuffer,
    should_redraw_everything: bool,
}

impl VirtualScreen {
    pub fn new(width: usize, height: usize) -> VirtualScreen {
        VirtualScreen {
            previous_buffer: VirtualScreenBuffer::new(width, height),
            current_buffer: VirtualScreenBuffer::new(width, height),
            should_redraw_everything: false,
        }
    }

    pub fn show_current_buffer(&self) -> String {
        self.current_buffer.show()
    }

    pub fn write_str(&mut self, x: usize, y: usize, value: &str) {
        self.current_buffer.write_str(x, y, value);
    }

    pub fn write_str_color(&mut self, x: usize, y: usize, value: &str, fg: Color, bg: Color) {
        self.current_buffer.write_str_color(x, y, value, fg, bg);
    }

    pub fn get_size(&self) -> (usize, usize) {
        self.current_buffer.get_size()
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.previous_buffer = VirtualScreenBuffer::new(width, height);
        self.current_buffer = VirtualScreenBuffer::new(width, height);
        self.should_redraw_everything = true;
    }

    fn commit_changes(&mut self, context: &TerminalContext, whole_screen: bool) {
        let cursor = context.crossterm.cursor();
        cursor.hide();

        for change in DifferenceIterator::new(&self.current_buffer, &self.previous_buffer, whole_screen) {
            cursor.goto(change.x as u16, change.y as u16);
            crossterm::style(&change.text)
                .with(change.fg.into())
                .on(change.bg.into())
                .paint(context.get_screen());
        }

        self.previous_buffer.copy_from(&self.current_buffer);
    }

    pub fn render_prepare(&mut self, context: &TerminalContext) {
        let (term_width, term_height) = context.get_terminal_size();
        let (width, height) = self.get_size();

        if term_width != width || term_height != height {
            self.resize(term_width, term_height);
        }

        self.current_buffer.clear();
    }

    pub fn commit(&mut self, context: &TerminalContext) {
        let terminal = context.crossterm.terminal();

        if self.should_redraw_everything {
            self.should_redraw_everything = false;
            terminal.clear(crossterm::terminal::ClearType::All);
            self.commit_changes(context, true);
        } else {
            self.commit_changes(context, false);
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
        let default_block = ScreenCell::default();

        assert_eq!(screen.get_block(0, 0), default_block);
        assert_eq!(screen.get_block(width - 1, 0), default_block);
        assert_eq!(screen.get_block(0, height - 1), default_block);
        assert_eq!(screen.get_block(width - 1, height - 1), default_block);
    }
}