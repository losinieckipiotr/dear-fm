use std::{
    fmt::{self, Display},
    fs::canonicalize,
    path::{Path, PathBuf},
};

// TODO: file operations should be async
use crate::files::{
    FileColumn, FileRecord, SortBy, SortDirection, is_dir, open_file,
    read_directory, sort_records,
};

#[derive(Debug, Clone)]
pub enum LoadError {
    File,
    Format,
}

#[derive(Debug, Clone, Copy)]
pub enum SaveError {
    Write,
    Format,
}

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

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
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

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AppState {
    // pub last_frame: Instant,
    // pub last_frame_measure_time: Instant,
    // pub last_measure_frame_count: i32,
    // pub frame_rate: i32,
    // pub frame_count: i32,
    focused_window_left: bool,

    left: SideData,
    right: SideData,

    pub fullscreen: bool,
    pub maximized: bool,
    // TODO: save index position in given folder
    // and select this index if we go back to that folder again
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            focused_window_left: true,

            left: SideData::default(),
            right: SideData::default(),

            fullscreen: false,
            maximized: false,
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

        sort_records(files, sort_by, direction);
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

    pub fn get_selected_side(&self) -> Side {
        match self.focused_window_left {
            true => Side::Left,
            false => Side::Right,
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
        let mut files = read_directory(&canon_path);

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

        sort_records(&mut files, sort_options.sort_by, sort_options.direction);

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

        if is_dir(&path_to_open) {
            self.go_to_directory(side, path_to_open);
        } else {
            open_file(path_to_open);
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

    pub async fn load(path: &str) -> Result<AppState, LoadError> {
        let state_str = tokio::fs::read_to_string(path)
            .await
            .map_err(|_| LoadError::File)?;

        let mut state: AppState =
            serde_json::from_str(&state_str).map_err(|_| LoadError::Format)?;

        if state.focused_window_left {
            let side = Side::Left;
            let idx = state.get_selected_idx(side).unwrap_or(0);

            // TODO: refactor
            state.go_to_directory(
                Side::Left,
                state.get_path(Side::Left).to_path_buf(),
            );

            state.go_to_directory(
                Side::Right,
                state.get_path(Side::Right).to_path_buf(),
            );

            state.focus_window(side);
            state.set_selected_idx(side, idx);
        } else {
            let side = Side::Right;
            let idx = state.get_selected_idx(side).unwrap_or(0);

            // TODO: refactor
            state.go_to_directory(
                Side::Left,
                state.get_path(Side::Left).to_path_buf(),
            );

            state.go_to_directory(
                Side::Right,
                state.get_path(Side::Right).to_path_buf(),
            );

            state.focus_window(side);
            state.set_selected_idx(side, idx);
        }

        Ok(state)
    }

    pub async fn save(self, save_path: &str) -> Result<(), SaveError> {
        use iced::time::milliseconds;

        let json = serde_json::to_string_pretty(&self)
            .map_err(|_| SaveError::Format)?;

        let mut path = std::env::current_dir().unwrap_or_default();
        path.push(save_path);

        log::debug!("saving to path: {}", path.display());
        log::debug!("{}", json);

        tokio::fs::write(path, json.as_bytes())
            .await
            .map_err(|_| SaveError::Write)?;

        // This is a simple way to save at most twice every second
        tokio::time::sleep(milliseconds(500)).await;

        Ok(())
    }

    pub fn get_hover(&self, side: Side, idx: usize) -> bool {
        let files = match side {
            Side::Left => &self.left.files,
            Side::Right => &self.right.files,
        };
        let hover = files[idx].hover;

        hover.iter().any(|i| *i)
    }

    pub fn update_hover(
        &mut self,
        side: Side,
        idx: usize,
        file_col: FileColumn,
        hover: bool,
    ) {
        let files = match side {
            Side::Left => &mut self.left.files,
            Side::Right => &mut self.right.files,
        };

        let file = &mut files[idx];
        let col_idx: usize = file_col.into();
        file.hover[col_idx] = hover;
    }
}
