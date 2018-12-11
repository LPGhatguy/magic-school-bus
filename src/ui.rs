use crate::{
    input_state::{InputState, InputMode},
    app_state::{DirectoryListState, AppState},
    virtual_screen::VirtualScreen,
    terminal_context::{Color},
};

fn pad_right_with_spaces(text: &mut String, width: usize) {
    let text_width = text.chars().count();

    if text_width < width {
        for _ in 0..(width - text_width) {
            text.push(' ');
        }
    }
}

/// A hack to adjust state to match the screen, used for windowing the list.
pub fn nudge_state(state: &mut AppState, screen: &VirtualScreen) {
    let height = screen.get_size().1;

    let max_item_count = height - 4;

    let window_top = state.entry_list.window_start;
    let window_bottom = state.entry_list.window_start + max_item_count;

    if state.entry_list.cursor <= window_top {
        state.entry_list.window_start = state.entry_list.cursor;
    }

    if state.entry_list.cursor >= window_bottom {
        state.entry_list.window_start = state.entry_list.cursor - max_item_count + 1;
    }
}

fn render_entry_list(entry_list: &DirectoryListState, position: (usize, usize), max_size: (usize, usize), screen: &mut VirtualScreen) {
    let (list_x, list_y) = position;

    let max_item_count = max_size.1 - 2;
    let window_start = entry_list.window_start;
    let window_size = max_item_count
        .min(entry_list.entries.len() - window_start)
        .min(max_size.1);

    let mut list_width = 0;

    for (index, entry) in entry_list.entries.iter().enumerate() {
        list_width = list_width
            .max(entry.display.chars().count())
            .min(max_size.0);

        if index >= window_start && index < window_start + window_size {
            let x = list_x + 2;
            let y = list_y + 1 + index - window_start;

            let (fg, bg) = if index == entry_list.cursor {
                (Color::Black, Color::White)
            } else {
                (Color::Reset, Color::Reset)
            };

            screen.write_str_color(x, y, &entry.display, fg, bg);
        }
    }

    // Draw a border around all the entries
    let entry_vertical_line = "|\n".repeat(window_size);
    let end_of_list_line = "-".repeat(list_width + 4);
    let more_list_line = "~".repeat(list_width + 4);

    let top_line = if window_start > 0 {
        &more_list_line
    } else {
        &end_of_list_line
    };

    let bottom_line = if window_start + window_size < entry_list.entries.len() {
        &more_list_line
    } else {
        &end_of_list_line
    };

    screen.write_str(list_x, list_y + 1, &entry_vertical_line);
    screen.write_str(list_x + list_width + 3, list_y + 1, &entry_vertical_line);
    screen.write_str(list_x, list_y, top_line);
    screen.write_str(list_x, list_y + window_size + 1, bottom_line);
}

pub fn render(state: &AppState, input_state: &InputState, screen: &mut VirtualScreen) {
    let (width, height) = screen.get_size();

    let mut working_dir_text = format!("{}", state.entry_list.directory.display());
    pad_right_with_spaces(&mut working_dir_text, width);
    screen.write_str_color(0, 0, &working_dir_text, Color::Black, Color::White);

    render_entry_list(&state.entry_list, (0, 1), (width, height - 2), screen);

    let mut prompt_foreground = Color::Black;
    let mut prompt_background = Color::White;
    let mut status_bar_text = String::new();

    match input_state.get_mode() {
        InputMode::Normal => {
            status_bar_text.push_str("Last action: ");

            match &state.last_action {
                Some(last_action) => {
                    status_bar_text.push_str(&format!("{:?}", last_action));
                },
                None => status_bar_text.push_str("None"),
            };

            if let Some(count) = input_state.get_count_progress() {
                status_bar_text.push_str(" | ");
                status_bar_text.push_str(count);
            }
        },
        InputMode::DeletePrompt => {
            status_bar_text.push_str("Are you sure you want to delete selected? (y or escape)")
        },
        InputMode::FindPrompt => {
            let prompt_string = "Find: ";
            status_bar_text.push_str(prompt_string);

            for &char in input_state.get_text_buffer() {
                status_bar_text.push(char);
            }

            if state.no_find_match {
                prompt_foreground = Color::White;
                prompt_background = Color::Red;
            }

            screen.set_cursor_position(prompt_string.len() + input_state.get_cursor_position(), height - 1);
        },
        InputMode::CommandPrompt => {
            status_bar_text.push(':');

            for &char in input_state.get_text_buffer() {
                status_bar_text.push(char);
            }

            screen.set_cursor_position(1 + input_state.get_cursor_position(), height - 1);
        },
        InputMode::NewFilePrompt => {
            let prompt_string = "New file: ";
            status_bar_text.push_str(prompt_string);

            for &char in input_state.get_text_buffer() {
                status_bar_text.push(char);
            }

            screen.set_cursor_position(prompt_string.len() + input_state.get_cursor_position(), height - 1);
        },
        InputMode::NewDirectoryPrompt => {
            let prompt_string = "New dir: ";
            status_bar_text.push_str(prompt_string);

            for &char in input_state.get_text_buffer() {
                status_bar_text.push(char);
            }

            screen.set_cursor_position(prompt_string.len() + input_state.get_cursor_position(), height - 1);
        },
    }

    pad_right_with_spaces(&mut status_bar_text, width);
    screen.write_str_color(0, height - 1, &status_bar_text, prompt_foreground, prompt_background);
}