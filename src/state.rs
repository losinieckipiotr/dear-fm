use std::{
    fmt::{self, Display},
    path::{Path, PathBuf},
    time::Instant,
};

use imgui::MouseCursor;

use crate::files;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    fn is_left(&self) -> bool {
        match self {
            Self::Left => true,
            Self::Right => false,
        }
    }
}

impl Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug)]
struct AppFiles {
    left_path: PathBuf,
    right_path: PathBuf,
    left_files: Vec<String>,
    right_files: Vec<String>,
}

// TODO: make constructor and private fields

#[derive(Debug)]
pub struct AppState {
    pub demo_open: bool,
    pub limit_fps: bool,
    pub last_frame: Instant,

    pub last_cursor: Option<MouseCursor>,

    // pub frame_count: i32,
    pub frame_rate: i32,
    pub last_frame_measure_time: Instant,
    pub last_measure_frame_count: i32,

    focused_window_left: bool,

    // TODO: make selected idexes optional
    pub left_item_selected_idx: i32,
    pub right_item_selected_idx: i32,

    app_files: AppFiles,
}

impl AppState {
    pub fn new() -> Self {
        let now = Instant::now();

        AppState {
            demo_open: false,
            limit_fps: true,
            last_frame: now,

            last_cursor: None,

            last_frame_measure_time: now,
            last_measure_frame_count: 0,
            frame_rate: 0,

            focused_window_left: true,

            left_item_selected_idx: 0,
            right_item_selected_idx: 0,

            app_files: AppFiles {
                left_path: PathBuf::new(),
                right_path: PathBuf::new(),
                left_files: Vec::new(),
                right_files: Vec::new(),
            },
        }
    }

    pub fn is_window_focused(&self, side: Side) -> bool {
        match side {
            Side::Left => self.focused_window_left,
            Side::Right => !self.focused_window_left,
        }
    }

    pub fn get_path(&self, side: Side) -> &Path {
        match side {
            Side::Left => &self.app_files.left_path,
            Side::Right => &self.app_files.right_path,
        }
    }

    pub fn get_window_files(&self, side: Side) -> &Vec<String> {
        match side {
            Side::Left => &self.app_files.left_files,
            Side::Right => &self.app_files.right_files,
        }
    }

    pub fn get_selected_idx(&self, side: Side) -> i32 {
        match side {
            Side::Left => self.left_item_selected_idx,
            Side::Right => self.right_item_selected_idx,
        }
    }

    pub fn set_selected_idx(&mut self, side: Side, idx: i32) {
        match side {
            Side::Left => {
                self.left_item_selected_idx = idx;
            }
            Side::Right => {
                self.right_item_selected_idx = idx;
            }
        }
    }

    pub fn focus_window(&mut self, side: Side) {
        self.focused_window_left = side.is_left();
    }

    /// TODO maybe not needed
    pub fn _has_window_focus(&mut self, side: Side) -> bool {
        match side {
            Side::Left => self.focused_window_left,
            Side::Right => !self.focused_window_left,
        }
    }

    pub fn toggle_window_focus(&mut self) {
        self.focused_window_left = !self.focused_window_left;
    }

    pub fn go_to_directory(&mut self, side: Side, path_to_open: PathBuf) {
        let files = files::read_directory(&path_to_open);

        match side {
            Side::Left => {
                self.app_files.left_path = path_to_open;
                self.app_files.left_files = files;
                // TODO: handle case if directory is empty?
                self.set_selected_idx(side, 0);
            }
            Side::Right => {
                self.app_files.right_path = path_to_open;
                self.app_files.right_files = files;
                // TODO: handle case if directory is empty?
                self.set_selected_idx(side, 0);
            }
        }
    }

    pub fn select_next_idx(&mut self, side: Side) {
        let files_len = self.get_window_files(side).len();
        let current_item = self.get_selected_idx(side);

        let next_item = current_item + 1;
        if next_item < files_len as i32 {
            self.set_selected_idx(side, next_item);
        }
    }

    pub fn select_prev_idx(&mut self, side: Side) {
        let current_item = self.get_selected_idx(side);

        let prev_item = current_item - 1;
        if prev_item >= 0 {
            self.set_selected_idx(side, prev_item);
        }
    }

    pub fn get_path_to_open_at(&self, side: Side, idx: i32) -> PathBuf {
        let files = self.get_window_files(side);
        let path = self.get_path(side);
        let element_to_open = &files[idx as usize];

        let mut path_to_open = PathBuf::new();
        path_to_open.push(path);
        path_to_open.push(element_to_open);

        path_to_open
    }
}
