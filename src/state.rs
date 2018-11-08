use open;

use std::{
    cmp::Ordering,
    fs,
    path::PathBuf,
    thread,
};

use crate::{
    action::Action,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FileEntryKind {
    Parent,
    Directory,
    File,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FileEntry {
    pub kind: FileEntryKind,
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
        match (self.kind, other.kind) {
            (FileEntryKind::Parent, _) => Ordering::Less,
            (_, FileEntryKind::Parent) => Ordering::Greater,
            (FileEntryKind::Directory, FileEntryKind::File) => Ordering::Less,
            (FileEntryKind::File, FileEntryKind::Directory) => Ordering::Greater,
            _ => self.display.to_lowercase().cmp(&other.display.to_lowercase()),
        }
    }
}

#[derive(Debug)]
pub struct State {
    pub last_action: Option<Action>,
    pub working_directory: PathBuf,
    pub entries: Vec<FileEntry>,
    pub cursor: usize,
    pub entry_window_start: usize,
    pub find_target: Option<char>,
}

impl State {
    pub fn new(start_dir: PathBuf) -> State {
        let mut state = State {
            last_action: None,
            working_directory: PathBuf::new(),
            entries: Vec::new(),
            cursor: 0,
            entry_window_start: 0,
            find_target: None,
        };

        state.set_working_directory(start_dir);

        state
    }

    fn refresh_working_directory(&mut self) {
        self.entries.clear();

        if let Some(parent) = self.working_directory.parent() {
            self.entries.push(FileEntry {
                kind: FileEntryKind::Parent,
                display: "..".to_string(),
                path: parent.to_path_buf(),
            });
        }

        for entry in fs::read_dir(&self.working_directory).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let mut display = path.file_name().unwrap().to_string_lossy().to_string();
            let mut kind = FileEntryKind::File;

            if path.is_dir() {
                kind = FileEntryKind::Directory;
                display.push_str("/");
            }

            self.entries.push(FileEntry {
                kind,
                display,
                path,
            });
        }

        self.entries.sort();
        self.cursor = self.cursor.min(self.entries.len());
    }

    pub fn set_working_directory(&mut self, path: PathBuf) {
        self.cursor = 0;
        self.entry_window_start = 0;
        self.working_directory = path;

        self.refresh_working_directory();
    }

    pub fn open_file(&self, path: PathBuf) {
        // Open can sometimes take awhile, like when opening Visual Studio.
        // To mitigate that, call open on a throwaway new thread.
        thread::spawn(move || {
            open::that(path).expect("Could not open file");
        });
    }

    fn perform_find_previous(&mut self) {
        if let Some(first_char) = self.find_target {
            let mut found_index = None;
            for i in (0..self.cursor).rev() {
                if self.entries[i].display.starts_with(first_char) {
                    found_index = Some(i);
                    break;
                }
            }

            if let Some(index) = found_index {
                self.cursor = index;
            }
        }
    }

    fn perform_find_next(&mut self) {
        if let Some(first_char) = self.find_target {
            let mut found_index = None;
            for i in (self.cursor + 1)..self.entries.len() {
                if self.entries[i].display.starts_with(first_char) {
                    found_index = Some(i);
                    break;
                }
            }

            if let Some(index) = found_index {
                self.cursor = index;
            }
        }
    }

    pub fn process_action(&mut self, action: Action) {
        self.last_action = Some(action);

        match action {
            Action::Up(count) => {
                for _ in 0..count {
                    if self.cursor > 0 {
                        self.cursor -= 1;
                    }
                }
            },
            Action::Down(count) => {
                for _ in 0..count {
                    if self.cursor < self.entries.len() - 1 {
                        self.cursor += 1;
                    }
                }
            },
            Action::Top => {
                self.cursor = 0;
            },
            Action::Bottom => {
                self.cursor = self.entries.len() - 1;
            },
            Action::Activate => {
                let entry = &self.entries[self.cursor];

                match entry.kind {
                    FileEntryKind::Directory | FileEntryKind::Parent => {
                        self.set_working_directory(entry.path.clone());
                    },
                    FileEntryKind::File => {
                        self.open_file(entry.path.clone());
                    },
                }
            },
            Action::Delete => {
                let entry = &self.entries[self.cursor];

                match entry.kind {
                    FileEntryKind::Directory => {
                        fs::remove_dir_all(&entry.path)
                            .expect("Could not remove directory and its contents!");
                    },
                    FileEntryKind::File => {
                        fs::remove_file(&entry.path)
                            .expect("Could not remove file!");
                    },
                    FileEntryKind::Parent => {},
                }

                self.refresh_working_directory();
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