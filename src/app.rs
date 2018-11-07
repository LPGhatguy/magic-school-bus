use std::path::PathBuf;

use crate::{
    state::{State, Action},
    virtual_screen::{Color, VirtualScreen},
    terminal_context::TerminalContext,
};

fn render(state: &State, screen: &mut VirtualScreen) {
    let (width, height) = screen.get_size();

    let item_count_clamped = state.entries.len().min(height as usize - 4);

    let full_width_line = "-".repeat(width as usize);
    let gutter_line = "|\n".repeat(item_count_clamped);

    screen.write_str(0, 0, &full_width_line);
    screen.write_str(0, 1, &gutter_line);

    for (index, entry) in state.entries.iter().take(item_count_clamped).enumerate() {
        if index == state.selected_entry {
            screen.write_str_color(2, 1 + index, &entry.display, Color::Black, Color::White);
        } else {
            screen.write_str(2, 1 + index, &entry.display);
        }
    }

    screen.write_str(0, 1 + item_count_clamped, &full_width_line);
    screen.write_str(0, height - 3, &full_width_line);
    screen.write_str(0, height - 2, &format!("Last action: {:?}", state.last_action));
    screen.write_str(0, height - 1, &full_width_line);
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

        draw(&state, &context, &mut screen);
    }

    drop(context);

    if config.print_working_directory {
        eprintln!("{}", state.working_directory.display());
    }
}