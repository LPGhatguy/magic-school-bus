extern crate crossterm;

use std::{
    env,
    fs,
    thread,
    path::{Path, PathBuf},
    io::{Read, Bytes},
    time::{Instant, Duration},
};

use crossterm::{
    style::{Color, style},
    terminal::ClearType,
    Crossterm,
    AsyncReader,
    Screen,
};

struct FileEntry {
    is_dir: bool,
    display: String,
    path: PathBuf,
}

struct State<'a> {
    input: Bytes<AsyncReader>,
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

    cursor.goto(0, 0);
    style("--------------------").with(Color::White).on(Color::Black).paint(state.screen);

    let item_count_clamped = state.entries.len().min(height as usize - 4);

    cursor.goto(0, 1);
    let gutter = "|\n".repeat(item_count_clamped);
    style(gutter).with(Color::White).on(Color::Black).paint(state.screen);

    for (index, entry) in state.entries.iter().enumerate() {
        if index >= item_count_clamped {
            break;
        }

        cursor.goto(2, 1 + index as u16);

        let mut styled = style(&entry.display);

        if index == state.selected_entry {
            styled = styled.with(Color::Black).on(Color::White);
        } else {
            styled = styled.with(Color::White).on(Color::Black);
        }

        styled.paint(state.screen);
    }

    cursor.goto(0, 1 + state.entries.len() as u16);
    style("--------------------").with(Color::White).on(Color::Black).paint(state.screen);

    cursor.goto(0, height - 2);
    style("-".repeat(width as usize)).with(Color::White).on(Color::Black).paint(state.screen);
    cursor.goto(0, height - 1);
    terminal.clear(ClearType::CurrentLine);
    style(format!("Last action: {:?}", state.last_action))
        .with(Color::White).on(Color::Black).paint(state.screen);
    cursor.goto(0, height);
    style("-".repeat(width as usize)).with(Color::White).on(Color::Black).paint(state.screen);
}

fn process_input(state: &mut State) -> Option<Action> {
    let pressed = state.input.next();

    if let Some(Ok(key)) = pressed {
        match key {
            b'q' => Some(Action::Quit),
            b'j' => Some(Action::Down),
            b'k' => Some(Action::Up),
            b'\r' => Some(Action::Select),
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
    let stdin = input.read_async().bytes();

    let mut state = State {
        input: stdin,
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

    loop {
        let start_of_frame = Instant::now();

        if let Some(action) = process_input(&mut state) {
            state.process_action(action);

            if action == Action::Quit {
                break;
            }
        }

        prepaint(&mut state);
        paint(&state);

        let processing_time = start_of_frame.elapsed();
        match Duration::from_millis(33).checked_sub(processing_time) {
            Some(difference) => thread::sleep(difference),
            None => {},
        }
    }
}
