use std::path::PathBuf;

use crate::{
    state::State,
    virtual_screen::{Color, VirtualScreen},
    terminal_context::TerminalContext,
    action::Action,
    input_state::InputState,
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
fn nudge_state(state: &mut State, screen: &VirtualScreen) {
    let height = screen.get_size().1;

    let max_item_count = height - 4;

    let window_top = state.entry_window_start;
    let window_bottom = state.entry_window_start + max_item_count;

    if state.selected_entry <= window_top {
        state.entry_window_start = state.selected_entry;
    }

    if state.selected_entry >= window_bottom {
        state.entry_window_start = state.selected_entry - max_item_count + 1;
    }
}

fn render(state: &State, input_state: &InputState, screen: &mut VirtualScreen) {
    let (width, height) = screen.get_size();

    let max_item_count = height - 4;
    let window_start = state.entry_window_start;
    let window_size = max_item_count.min(state.entries.len() - window_start);

    let mut working_dir_text = format!("{}", state.working_directory.display());
    pad_right_with_spaces(&mut working_dir_text, width);
    screen.write_str_color(0, 0, &working_dir_text, Color::Black, Color::White);

    let mut widest_entry_width = 0;

    for (index, entry) in state.entries.iter().enumerate() {
        widest_entry_width = widest_entry_width.max(entry.display.chars().count());

        if index >= window_start && index < window_start + window_size {
            let y = 2 + index - window_start;

            if index == state.selected_entry {
                screen.write_str_color(2, y, &entry.display, Color::Black, Color::White);
            } else {
                screen.write_str(2, y, &entry.display);
            }
        }
    }

    // Draw a border around all the entries
    let end_of_list_line = "-".repeat(widest_entry_width + 4);
    let more_list_line = "~".repeat(widest_entry_width + 4);

    let top_line = if window_start > 0 {
        &more_list_line
    } else {
        &end_of_list_line
    };
    let bottom_line = if window_start + window_size < state.entries.len() {
        &more_list_line
    } else {
        &end_of_list_line
    };

    let entry_vertical_line = "|\n".repeat(window_size);
    screen.write_str(0, 2, &entry_vertical_line);
    screen.write_str(widest_entry_width + 3, 2, &entry_vertical_line);
    screen.write_str(0, 1, top_line);
    screen.write_str(0, 2 + window_size, bottom_line);

    let mut status_bar_text = "Last action: ".to_string();

    match state.last_action {
        Some(last_action) => {
            status_bar_text.push_str(&format!("{:?}", last_action));
        },
        None => status_bar_text.push_str("None"),
    };

    if let Some(count) = input_state.get_count_progress() {
        status_bar_text.push_str(" | ");
        status_bar_text.push_str(count);
    }

    pad_right_with_spaces(&mut status_bar_text, width);
    screen.write_str_color(0, height - 1, &status_bar_text, Color::Black, Color::White);
}

fn draw(state: &State, input_state: &InputState, context: &TerminalContext, screen: &mut VirtualScreen) {
    screen.prepaint(context);
    render(state, input_state, screen);
    screen.commit(context);
}

pub struct AppConfig {
    pub print_working_directory: bool,
    pub start_dir: PathBuf,
}

pub fn start(config: AppConfig) {
    let mut state = State::new(config.start_dir.clone());
    let mut input_state = InputState::new();

    let context = TerminalContext::init();
    let (width, height) = context.get_terminal_size();
    let mut screen = VirtualScreen::new(width, height);

    nudge_state(&mut state, &screen);
    draw(&state, &input_state, &context, &mut screen);

    loop {
        if let Some(action) = input_state.process_input(&context) {
            state.process_action(action);

            if action == Action::Quit {
                break;
            }

            if action == Action::DebugDumpVisible {
                eprintln!("{}", screen.show_visible_buffer());
            }
        }

        nudge_state(&mut state, &screen);
        draw(&state, &input_state, &context, &mut screen);
    }

    drop(context);

    if config.print_working_directory {
        eprintln!("{}", state.working_directory.display());
    }
}