extern crate crossterm;

pub mod state;
pub mod terminal_context;
pub mod virtual_screen;

use std::{
    env,
    path::PathBuf,
};

use crossterm::{
    Crossterm,
    Screen,
};

use crate::{
    state::{State, Action},
    virtual_screen::{Color, VirtualScreen},
    terminal_context::TerminalContext,
};

fn paint(state: &State, screen: &mut VirtualScreen) {
    let (width, height) = screen.get_size();

    let item_count_clamped = state.entries.len().min(height as usize - 4);

    let full_width_line = "-".repeat(width as usize);
    let gutter_line = "|\n".repeat(item_count_clamped);

    screen.write_str(0, 0, &full_width_line);
    screen.write_str(0, 1, &gutter_line);

    for (index, entry) in state.entries.iter().enumerate() {
        if index >= item_count_clamped {
            break;
        }

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

fn main() {
    let screen = Screen::default();
    let alternate = screen.enable_alternate_modes(true).unwrap();
    let crossterm = Crossterm::new(&alternate.screen);

    {
        let cursor = crossterm.cursor();
        cursor.hide();
    }

    let context = TerminalContext {
        screen: &alternate.screen,
        crossterm: &crossterm,
    };

    let mut state = State {
        screen: &alternate.screen,
        crossterm: &crossterm,
        last_action: None,
        working_directory: PathBuf::new(),
        entries: Vec::new(),
        selected_entry: 0,
    };

    let (width, height) = state.get_terminal_size();
    let mut screen = VirtualScreen::new(width, height);

    state.set_working_directory(&env::current_dir().unwrap());

    screen.prepaint(&context);
    paint(&state, &mut screen);
    screen.commit(&context);

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

        screen.prepaint(&context);
        paint(&state, &mut screen);
        screen.commit(&context);
    }

    let working_directory = state.working_directory.clone();

    drop(alternate);

    eprintln!("{}", working_directory.display());
}