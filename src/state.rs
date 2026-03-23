use std::{
    fmt::{self, Display},
    time::Instant,
};

use imgui::MouseCursor;

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
pub struct AppFiles {
    pub left_path: String,
    pub right_path: String,
    pub left_files: Vec<String>,
    pub right_files: Vec<String>,
}

// TODO: make constructor and private fields

#[derive(Debug)]
pub struct AppState {
    pub demo_open: bool,
    pub limit_fps: bool,
    pub last_frame: Instant,

    pub last_cursor: Option<MouseCursor>,

    pub last_frame_measure_time: Instant,
    pub last_measure_frame_count: i32,
    pub frame_rate: i32,
    pub frame_count: i32,

    focused_window_left: bool,

    // TODO: make selected idexes optional
    pub left_item_selected_idx: i32,
    pub right_item_selected_idx: i32,
    pub app_files: AppFiles,
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
            frame_count: 0,

            focused_window_left: true,

            left_item_selected_idx: 0,
            right_item_selected_idx: 0,

            app_files: AppFiles {
                left_path: String::from("/Users/piotrlosiniecki"),
                right_path: String::from("/Users/piotrlosiniecki/Projects"),
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

    pub fn get_path(&self, side: Side) -> &str {
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
}
