use open;

use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct FileEntry {
    pub is_dir: bool,
    pub display: String,
    pub path: PathBuf,
}

pub struct State {
    pub last_action: Option<Action>,
    pub working_directory: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_entry: usize,
}

impl State {
    pub fn new() -> State {
        State {
            last_action: None,
            working_directory: PathBuf::new(),
            entries: Vec::new(),
            selected_entry: 0,
        }
    }

    pub fn set_working_directory(&mut self, path: &Path) {
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
                    self.set_working_directory(&entry.path.clone());
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
