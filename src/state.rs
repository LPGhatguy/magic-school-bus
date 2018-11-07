use open;

use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct FileEntry {
    pub is_dir: bool,
    pub display: String,
    pub path: PathBuf,
}

#[derive(Debug)]
pub struct State {
    pub last_action: Option<Action>,
    pub working_directory: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_entry: usize,
    pub entry_window_start: usize,
}

impl State {
    pub fn new(start_dir: PathBuf) -> State {
        let mut state = State {
            last_action: None,
            working_directory: PathBuf::new(),
            entries: Vec::new(),
            selected_entry: 0,
            entry_window_start: 0,
        };

        state.set_working_directory(start_dir);

        state
    }

    pub fn set_working_directory(&mut self, path: PathBuf) {
        self.entries.clear();
        self.selected_entry = 0;
        self.entry_window_start = 0;

        if let Some(parent) = path.parent() {
            self.entries.push(FileEntry {
                is_dir: true,
                display: "..".to_string(),
                path: parent.to_path_buf(),
            });
        }

        for entry in fs::read_dir(&path).unwrap() {
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

        self.working_directory = path;
    }

    pub fn open_file(&self, path: &Path) {
        open::that(path).expect("Could not open file");
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
                    self.set_working_directory(entry.path.to_path_buf());
                } else {
                    self.open_file(&entry.path);
                }
            },
            _ => {},
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    Up,
    Down,
    Select,

    DebugDumpVisible,
}
