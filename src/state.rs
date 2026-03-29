use std::{
    fmt::{self, Display},
    fs::canonicalize,
    path::{Path, PathBuf},
};

use crate::files::{self, FileRecord, SortBy, SortDirection};

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

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
#[serde(default)]
struct SortingOptions {
    sort_by: SortBy,
    direction: SortDirection,
}

impl Default for SortingOptions {
    fn default() -> Self {
        Self {
            sort_by: SortBy::Name,
            direction: SortDirection::Ascending,
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(default)]
struct SideData {
    selected_idx: Option<usize>,
    sorting_options: SortingOptions,
    path: PathBuf,

    #[serde(skip)]
    files: Vec<FileRecord>,
}

impl Default for SideData {
    fn default() -> Self {
        Self {
            selected_idx: None,
            sorting_options: SortingOptions {
                sort_by: SortBy::Name,
                direction: SortDirection::Ascending,
            },
            path: PathBuf::new(),
            files: Vec::new(),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AppState {
    // pub demo_open: bool,
    // pub limit_fps: bool,
    // pub last_frame: Instant,

    // pub last_frame_measure_time: Instant,
    // pub last_measure_frame_count: i32,
    // pub frame_rate: i32,
    // pub frame_count: i32,
    focused_window_left: bool,

    left: SideData,
    right: SideData,
    // TODO: save index position in given folder
    // and select this index if we go back to that folder again
}

impl Default for AppState {
    fn default() -> Self {
        // let now = Instant::now();

        Self {
            // demo_open: false,
            // limit_fps: true,
            // last_frame: now,

            // last_frame_measure_time: now,
            // last_measure_frame_count: 0,
            // frame_rate: 0,
            // frame_count: 0,
            focused_window_left: true,

            left: SideData::default(),
            right: SideData::default(),
        }
    }
}

impl AppState {
    pub fn is_window_focused(&self, side: Side) -> bool {
        match side {
            Side::Left => self.focused_window_left,
            Side::Right => !self.focused_window_left,
        }
    }

    pub fn get_path(&self, side: Side) -> &Path {
        match side {
            Side::Left => &self.left.path,
            Side::Right => &self.right.path,
        }
    }

    pub fn get_window_files(&self, side: Side) -> &Vec<FileRecord> {
        match side {
            Side::Left => &self.left.files,
            Side::Right => &self.right.files,
        }
    }

    pub fn sort_window_files(
        &mut self,
        side: Side,
        sort_by: SortBy,
        direction: SortDirection,
    ) {
        let sorting_options = SortingOptions { sort_by, direction };

        let files = match side {
            Side::Left => {
                self.left.sorting_options = sorting_options;
                &mut self.left.files
            }
            Side::Right => {
                self.right.sorting_options = sorting_options;
                &mut self.right.files
            }
        };

        files::sort_records(files, sort_by, direction);
    }

    pub fn get_selected_idx(&self, side: Side) -> Option<usize> {
        match side {
            Side::Left => self.left.selected_idx,
            Side::Right => self.right.selected_idx,
        }
    }

    pub fn set_selected_idx(&mut self, side: Side, idx: usize) {
        let some_idx = Some(idx);

        match side {
            Side::Left => {
                self.left.selected_idx = some_idx;
            }
            Side::Right => {
                self.right.selected_idx = some_idx;
            }
        }
    }

    pub fn focus_window(&mut self, side: Side) {
        match side {
            Side::Left => {
                self.left.selected_idx = Some(0);
                self.right.selected_idx = None;
            }
            Side::Right => {
                self.left.selected_idx = None;
                self.right.selected_idx = Some(0);
            }
        }

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
        if self.focused_window_left {
            self.focus_window(Side::Right);
        } else {
            self.focus_window(Side::Left);
        }
    }

    pub fn go_to_directory(&mut self, side: Side, path_to_open: PathBuf) {
        log::debug!("go_to_directory path_to_open: {}", path_to_open.display());

        let canon_path = canonicalize(path_to_open).unwrap();
        let mut files = files::read_directory(&canon_path);

        log::debug!("go_to_directory canon_path: {}", canon_path.display());
        if canon_path != PathBuf::from("/") {
            files.insert(0, FileRecord::new_go_back_record());
        }

        let data = match side {
            Side::Left => {
                log::debug!("new_path: {}", canon_path.display());

                &mut self.left
            }
            Side::Right => {
                log::debug!("new_path: {}", canon_path.display());

                &mut self.right
            }
        };

        let sort_options = data.sorting_options;

        files::sort_records(
            &mut files,
            sort_options.sort_by,
            sort_options.direction,
        );

        data.path = canon_path;
        data.files = files;

        // TODO: we need cache to remeber previous select positon
        self.set_selected_idx(side, 0);
    }

    pub fn go_to_or_open(&mut self, side: Side, path_to_open: PathBuf) {
        log::debug!(
            "go_to_if_directory side: {}, path_to_open: {}, ",
            side,
            path_to_open.display(),
        );

        if files::is_dir(&path_to_open) {
            self.go_to_directory(side, path_to_open);
        } else {
            files::open_file(path_to_open);
        }
    }

    pub fn select_next_idx(&mut self, side: Side) {
        let files_len = self.get_window_files(side).len();
        let current_item = match self.get_selected_idx(side) {
            Some(idx) => idx,
            None => 0,
        };

        let next_item = current_item + 1;
        if next_item < files_len {
            self.set_selected_idx(side, next_item);
        }
    }

    pub fn select_prev_idx(&mut self, side: Side) {
        let current_item = match self.get_selected_idx(side) {
            Some(idx) => idx,
            None => 0,
        };

        if current_item > 0 {
            self.set_selected_idx(side, current_item - 1);
        }
    }

    pub fn get_path_to_open_at(&self, side: Side, idx: usize) -> PathBuf {
        let files = self.get_window_files(side);
        let path = self.get_path(side);
        let element_to_open = &files[idx].file_name;

        let mut path_to_open = PathBuf::new();
        path_to_open.push(path);
        path_to_open.push(element_to_open);

        path_to_open
    }
}
