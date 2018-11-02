extern crate crossterm;

pub mod virtual_screen;
pub mod state;

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
};

fn paint(state: &State, screen: &mut VirtualScreen) {
    let cursor = state.crossterm.cursor();

    let (width, height) = screen.get_size();

    cursor.hide();

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
    screen.write_str(0, height - 2, &full_width_line);
    screen.write_str(0, height - 1, &format!("Last action: {:?}", state.last_action));
    screen.write_str(0, height, &full_width_line);
}

fn process_input(state: &mut State) -> Option<Action> {
    if let Ok(key) = state.input.read_char() {
        match key {
            'q' => Some(Action::Quit),
            'j' => Some(Action::Down),
            'k' => Some(Action::Up),
            '\r' => Some(Action::Select),
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
    let input = crossterm.input();

    let (width, height) = {
        let size = crossterm.terminal().terminal_size();
        (size.0 as usize, size.1 as usize)
    };

    let mut screen = VirtualScreen::new(width, height);

    let mut state = State {
        input,
        screen: &alternate.screen,
        crossterm: &crossterm,
        last_action: None,
        working_directory: PathBuf::new(),
        last_screen_size: (width, height),
        entries: Vec::new(),
        selected_entry: 0,
    };

    state.set_working_directory(&env::current_dir().unwrap());

    screen.prepaint(&mut state);
    paint(&state, &mut screen);
    screen.commit(&mut state);

    loop {
        if let Some(action) = process_input(&mut state) {
            state.process_action(action);

            if action == Action::Quit {
                break;
            }
        }

        screen.prepaint(&mut state);
        paint(&state, &mut screen);
        screen.commit(&mut state);
    }

    let working_directory = state.working_directory.clone();

    drop(alternate);

    eprintln!("{}", working_directory.display());
}
