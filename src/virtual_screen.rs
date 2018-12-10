use crate::{
    terminal_context::{Color, TerminalContext},
    virtual_screen_buffer::VirtualScreenBuffer,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenCell {
    pub fg: Color,
    pub bg: Color,
    pub char: char,
}

impl Default for ScreenCell {
    fn default() -> ScreenCell {
        ScreenCell {
            fg: Color::Reset,
            bg: Color::Reset,
            char: ' ',
        }
    }
}

#[derive(Debug)]
struct ScreenDifference {
    x: usize,
    y: usize,
    text: String,
    fg: Color,
    bg: Color,
}

struct ScreenDifferenceIterator<'a> {
    current: &'a VirtualScreenBuffer,
    previous: &'a VirtualScreenBuffer,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    all_dirty: bool,
}

impl<'a> ScreenDifferenceIterator<'a> {
    pub fn new(
        current: &'a VirtualScreenBuffer,
        previous: &'a VirtualScreenBuffer,
        all_dirty: bool,
    ) -> ScreenDifferenceIterator<'a> {
        let (width, height) = current.get_size();

        ScreenDifferenceIterator {
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

impl<'a> Iterator for ScreenDifferenceIterator<'a> {
    type Item = ScreenDifference;

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
                return Some(ScreenDifference {
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

    fn get_changes(&self, whole_screen: bool) -> ScreenDifferenceIterator {
        ScreenDifferenceIterator::new(&self.current_buffer, &self.previous_buffer, whole_screen)
    }

    fn commit_changes(&mut self, context: &mut TerminalContext, whole_screen: bool) {
        context.hide_cursor();

        for change in self.get_changes(whole_screen) {
            context.move_cursor(change.x, change.y);
            context.paint_str(&change.text, change.fg, change.bg);
        }

        if let Some((cursor_x, cursor_y)) = self.current_buffer.cursor_position {
            context.move_cursor(cursor_x, cursor_y);
            context.show_cursor();
        }

        self.previous_buffer.cursor_position = self.current_buffer.cursor_position;
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

    pub fn set_cursor_position(&mut self, x: usize, y: usize) {
        self.current_buffer.cursor_position = Some((x, y));
    }

    pub fn commit(&mut self, context: &mut TerminalContext) {
        if self.should_redraw_everything {
            self.should_redraw_everything = false;
            context.clear_screen();
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