use open;

use std::{
    fs,
    path::{Path, PathBuf},
    cmp::Ordering,
};

use crate::{
    action::Action,
};

#[derive(Debug, PartialEq, Eq)]
pub struct FileEntry {
    pub is_dir: bool,
    pub display: String,
    pub path: PathBuf,
}

impl PartialOrd for FileEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FileEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.display == ".." {
            Ordering::Less
        } else if other.display == ".." {
            Ordering::Greater
        } else if self.is_dir && !other.is_dir {
            Ordering::Less
        } else if other.is_dir && !self.is_dir {
            Ordering::Greater
        } else {
            self.display.to_lowercase().cmp(&other.display.to_lowercase())
        }
    }
}

#[derive(Debug)]
pub struct State {
    pub last_action: Option<Action>,
    pub working_directory: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_entry: usize,
    pub entry_window_start: usize,
    pub find_target: Option<char>,
}

impl State {
    pub fn new(start_dir: PathBuf) -> State {
        let mut state = State {
            last_action: None,
            working_directory: PathBuf::new(),
            entries: Vec::new(),
            selected_entry: 0,
            entry_window_start: 0,
            find_target: None,
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

        self.entries.sort();
        self.working_directory = path;
    }

    pub fn open_file(&self, path: &Path) {
        open::that(path).expect("Could not open file");
    }

    fn perform_find_previous(&mut self) {
        if let Some(first_char) = self.find_target {
            let mut found_index = None;
            for i in (0..self.selected_entry).rev() {
                if self.entries[i].display.starts_with(first_char) {
                    found_index = Some(i);
                    break;
                }
            }

            if let Some(index) = found_index {
                self.selected_entry = index;
            }
        }
    }

    fn perform_find_next(&mut self) {
        if let Some(first_char) = self.find_target {
            let mut found_index = None;
            for i in (self.selected_entry + 1)..self.entries.len() {
                if self.entries[i].display.starts_with(first_char) {
                    found_index = Some(i);
                    break;
                }
            }

            if let Some(index) = found_index {
                self.selected_entry = index;
            }
        }
    }

    pub fn process_action(&mut self, action: Action) {
        self.last_action = Some(action);

        match action {
            Action::Up(count) => {
                for _ in 0..count {
                    if self.selected_entry > 0 {
                        self.selected_entry -= 1;
                    }
                }
            },
            Action::Down(count) => {
                for _ in 0..count {
                    if self.selected_entry < self.entries.len() - 1 {
                        self.selected_entry += 1;
                    }
                }
            },
            Action::Top => {
                self.selected_entry = 0;
            },
            Action::Bottom => {
                self.selected_entry = self.entries.len() - 1;
            },
            Action::Activate => {
                let entry = &self.entries[self.selected_entry];

                if entry.is_dir {
                    self.set_working_directory(entry.path.to_path_buf());
                } else {
                    self.open_file(&entry.path);
                }
            },
            Action::SetAndFindNext(count, first_char) => {
                self.find_target = Some(first_char);

                for _ in 0..count {
                    self.perform_find_next();
                }
            },
            Action::SetAndFindPrevious(count, first_char) => {
                self.find_target = Some(first_char);

                for _ in 0..count {
                    self.perform_find_previous();
                }
            },
            Action::FindNext(count) => {
                for _ in 0..count {
                    self.perform_find_next();
                }
            },
            Action::FindPrevious(count) => {
                for _ in 0..count {
                    self.perform_find_previous();
                }
            },
            _ => {},
        }
    }
}