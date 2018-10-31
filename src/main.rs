extern crate crossterm;

pub mod virtual_screen;

use std::{
    env,
    fs,
    path::{Path, PathBuf},
};

use crossterm::{
    style::{Color, style},
    terminal::ClearType,
    TerminalInput,
    Crossterm,
    Screen,
};

struct FileEntry {
    is_dir: bool,
    display: String,
    path: PathBuf,
}

struct State<'a> {
    input: TerminalInput<'a>,
    crossterm: &'a Crossterm,
    screen: &'a Screen,

    super_dirty: bool,
    last_action: Option<Action>,

    last_screen_size: (u16, u16),
    working_directory: PathBuf,
    entries: Vec<FileEntry>,
    selected_entry: usize,
}

impl<'a> State<'a> {
    pub fn set_working_directory(&mut self, path: &Path) {
        self.super_dirty = true;

        self.selected_entry = 0;
        self.working_directory = path.to_path_buf();
        self.entries.clear();

        if let Some(parent) = path.parent() {
            self.entries.push(FileEntry {
                is_dir: true,
                display: "..".to_string(),
                path: parent.to_path_buf(),
            });
        }

        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let mut display = path.file_name().unwrap().to_string_lossy().to_string();
            let mut is_dir = false;

            if path.is_dir() {
                is_dir = true;
                display.push_str("/");
            }

            self.entries.push(FileEntry {
                is_dir,
                display,
                path,
            });
        }
    }

    pub fn process_action(&mut self, action: Action) {
        self.last_action = Some(action);

        match action {
            Action::Up => {
                if self.selected_entry > 0 {
                    self.selected_entry -= 1;
                }
            },
            Action::Down => {
                if self.selected_entry < self.entries.len() - 1 {
                    self.selected_entry += 1;
                }
            },
            Action::Select => {
                let entry = &self.entries[self.selected_entry];

                if entry.is_dir {
                    self.set_working_directory(&entry.path.clone());
                }
            },
            _ => {},
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Quit,
    Up,
    Down,
    Select,
}

fn prepaint(state: &mut State) {
    let terminal = state.crossterm.terminal();
    let (width, height) = terminal.terminal_size();

    if state.last_screen_size != (width, height) || state.super_dirty {
        terminal.clear(ClearType::All);
        state.last_screen_size = (width, height);
        state.super_dirty = false;
    }
}

fn paint(state: &State) {
    let terminal = state.crossterm.terminal();
    let cursor = state.crossterm.cursor();

    let (width, height) = terminal.terminal_size();

    cursor.hide();

    let item_count_clamped = state.entries.len().min(height as usize - 4);

    cursor.goto(0, 0);
    style("-".repeat(width as usize)).paint(state.screen);

    cursor.goto(0, 1);
    let gutter = "|\n".repeat(item_count_clamped);
    style(gutter).paint(state.screen);

    for (index, entry) in state.entries.iter().enumerate() {
        if index >= item_count_clamped {
            break;
        }

        cursor.goto(2, 1 + index as u16);

        let mut styled = style(&entry.display);

        if index == state.selected_entry {
            styled = styled.with(Color::Black).on(Color::White);
        }

        styled.paint(state.screen);
    }

    cursor.goto(0, 1 + state.entries.len() as u16);
    style("-".repeat(width as usize)).paint(state.screen);

    cursor.goto(0, height - 2);
    style("-".repeat(width as usize)).with(Color::White).on(Color::Black).paint(state.screen);
    cursor.goto(0, height - 1);
    terminal.clear(ClearType::CurrentLine);
    style(format!("Last action: {:?}", state.last_action)).paint(state.screen);
    cursor.goto(0, height);
    style("-".repeat(width as usize)).paint(state.screen);
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

    let mut state = State {
        input,
        screen: &alternate.screen,
        crossterm: &crossterm,
        super_dirty: false,
        last_action: None,
        working_directory: PathBuf::new(),
        last_screen_size: (0, 0),
        entries: Vec::new(),
        selected_entry: 0,
    };

    state.set_working_directory(&env::current_dir().unwrap());
    prepaint(&mut state);
    paint(&state);

    loop {
        if let Some(action) = process_input(&mut state) {
            state.process_action(action);

            if action == Action::Quit {
                break;
            }
        }

        prepaint(&mut state);
        paint(&state);
    }
}
