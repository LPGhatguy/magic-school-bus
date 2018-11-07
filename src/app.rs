use std::path::PathBuf;

use crate::{
    state::{State, Action},
    virtual_screen::{Color, VirtualScreen},
    terminal_context::TerminalContext,
};

/// A hack to adjust state to match the screen, used for windowing the list.
fn nudge_state(state: &mut State, screen: &VirtualScreen) {
    let height = screen.get_size().1;

    let max_item_count = height - 2;

    let window_top = state.entry_window_start;
    let window_bottom = state.entry_window_start + max_item_count;

    if state.selected_entry <= window_top {
        state.entry_window_start = state.selected_entry;
    }

    if state.selected_entry >= window_bottom {
        state.entry_window_start = state.selected_entry - max_item_count + 1;
    }
}

fn render(state: &State, screen: &mut VirtualScreen) {
    let (width, height) = screen.get_size();

    let max_item_count = height - 2;
    let window_start = state.entry_window_start;
    let window_size = max_item_count.min(state.entries.len() - window_start);

    let full_width_line = "-".repeat(width as usize);
    let gutter_line = "|\n".repeat(max_item_count);

    screen.write_str(0, 0, &full_width_line);
    screen.write_str(0, 1, &gutter_line);

    let entry_iter = state.entries.iter()
        .enumerate()
        .skip(window_start)
        .take(window_size);

    for (index, entry) in entry_iter {
        let y = 1 + index - window_start;

        if index == state.selected_entry {
            screen.write_str_color(2, y, &entry.display, Color::Black, Color::White);
        } else {
            screen.write_str(2, y, &entry.display);
        }
    }

    let last_action_name = match state.last_action {
        Some(last_action) => format!("{:?}", last_action),
        None => "None".to_string(),
    };

    let status_bar_text = &format!("Last action: {}", last_action_name);

    // TODO: More accurate string width calculation for multi-codepoint
    // characters. Unicode support in terminals is dicey.
    let status_bar_text_width = status_bar_text.chars().count();

    let padding_size = if status_bar_text_width < width {
        width - status_bar_text_width
    } else {
        0
    };

    let status_bar_contents = format!("{}{}", status_bar_text, " ".repeat(padding_size));

    screen.write_str_color(0, height - 1, &status_bar_contents, Color::Black, Color::White);
}

fn process_input(context: &TerminalContext) -> Option<Action> {
    let input = context.crossterm.input();

    if let Ok(key) = input.read_char() {
        match key {
            'q' => Some(Action::Quit),
            'j' => Some(Action::Down),
            'k' => Some(Action::Up),
            '\r' => Some(Action::Select),
            '[' => Some(Action::DebugDumpVisible),
            _ => None,
        }
    } else {
        None
    }
}

fn draw(state: &State, context: &TerminalContext, screen: &mut VirtualScreen) {
    screen.prepaint(context);
    render(state, screen);
    screen.commit(context);
}

pub struct AppConfig {
    pub print_working_directory: bool,
    pub start_dir: PathBuf,
}

pub fn start(config: AppConfig) {
    let mut state = State::new(config.start_dir.clone());

    let context = TerminalContext::init();
    let (width, height) = context.get_terminal_size();
    let mut screen = VirtualScreen::new(width, height);

    nudge_state(&mut state, &screen);
    draw(&state, &context, &mut screen);

    loop {
        if let Some(action) = process_input(&context) {
            state.process_action(action);

            if action == Action::Quit {
                break;
            }

            if action == Action::DebugDumpVisible {
                eprintln!("{}", screen.show_visible_buffer());
            }
        }

        nudge_state(&mut state, &screen);
        draw(&state, &context, &mut screen);
    }

    drop(context);

    if config.print_working_directory {
        eprintln!("{}", state.working_directory.display());
    }
}