use open;

use std::{
    fs,
    path::{Path, PathBuf},
    cmp::Ordering,
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
    pub last_action_count: u64,
    pub working_directory: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_entry: usize,
    pub entry_window_start: usize,
    pub action_count_buffer: String,
    pub in_find_specify_mode: bool,
    pub find_target: Option<char>,
}

impl State {
    pub fn new(start_dir: PathBuf) -> State {
        let mut state = State {
            last_action: None,
            last_action_count: 1,
            working_directory: PathBuf::new(),
            entries: Vec::new(),
            selected_entry: 0,
            entry_window_start: 0,
            action_count_buffer: String::new(),
            in_find_specify_mode: false,
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

    fn consume_repeat(&mut self) -> u64 {
        let count = self.action_count_buffer.parse::<u64>().unwrap_or(1);
        self.action_count_buffer.clear();

        count
    }

    fn perform_single_action(&mut self, action: Action) {
        match action {
            Action::Cancel => {
                self.in_find_specify_mode = false;
            },
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
            Action::Top => {
                self.selected_entry = 0;
            },
            Action::Bottom => {
                self.selected_entry = self.entries.len() - 1;
            },
            Action::Select => {
                let entry = &self.entries[self.selected_entry];

                if entry.is_dir {
                    self.set_working_directory(entry.path.to_path_buf());
                } else {
                    self.open_file(&entry.path);
                }
            },
            Action::AddToRepeatBuffer(digit) => {
                self.action_count_buffer.push(digit);
            },
            Action::SetFindTarget(first_char) => {
                self.in_find_specify_mode = false;
                self.find_target = Some(first_char);
            },
            _ => {},
        }
    }

    pub fn process_action(&mut self, action: Action) {
        if action.show_in_status_bar() {
            self.last_action = Some(action);
        }

        let repeat_count = if action.should_consume_repeat() {
            self.consume_repeat()
        } else {
            1
        };

        if action.should_repeat() {
            for _ in 0..repeat_count {
                self.perform_single_action(action);
            }
            self.last_action_count = repeat_count;
        } else {
            self.perform_single_action(action);
            self.last_action_count = 1;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    Up,
    Down,
    Top,
    Bottom,
    Select,
    Cancel,
    AddToRepeatBuffer(char),
    EnterFindSpecifyMode,
    SetFindTarget(char),
    NextFind,
    PreviousFind,

    DebugDumpVisible,
    Unknown(char),
}

impl Action {
    fn should_repeat(&self) -> bool {
        match self {
            Action::Up | Action::Down => true,
            _ => false,
        }
    }

    fn should_consume_repeat(&self) -> bool {
        match self {
            Action::AddToRepeatBuffer(_) => false,
            _ => true,
        }
    }

    fn show_in_status_bar(&self) -> bool {
        match self {
            Action::AddToRepeatBuffer(_) => false,
            _ => true,
        }
    }
}