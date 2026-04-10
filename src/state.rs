use std::{
    fmt::{self, Display},
    fs::canonicalize,
    path::{Path, PathBuf},
};

use iced::time::milliseconds;

use crate::files::{
    FileColumn, FileRecord, FilesError, SortDirection, SortingOptions, is_dir,
    open_file, read_directory, sort_records,
};

#[derive(Debug, Clone)]
pub enum ReadDirectoryError {
    ReadDir(FilesError),
    Canonicalize,
}

impl Display for ReadDirectoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            ReadDirectoryError::ReadDir(err) => {
                format!("failed to read directory {:#?}", err).to_string()
            }
            ReadDirectoryError::Canonicalize => {
                "failed to canonicalize path".to_string()
            }
        };

        write!(f, "{msg}")
    }
}

#[derive(Debug, Clone)]
pub enum LoadError {
    File,

    Format,
    GoToDir(ReadDirectoryError),
}

impl Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            LoadError::File => "missing file?".to_string(),
            LoadError::Format => "invalid file format".to_string(),
            LoadError::GoToDir(err) => {
                format!("reading directory error {:#?}", err).to_string()
            }
        };

        write!(f, "{msg}")
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SaveError {
    Write,
    Format,
}

#[derive(Debug, Clone, Copy)]
pub enum OpenRecordError {
    Error,
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

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
struct SideData {
    // TODO: selected index should be one on top level
    selected_idx: Option<usize>,
    sorting_options: SortingOptions,
    path: PathBuf,

    #[serde(skip)]
    records: Vec<FileRecord>,
}

impl Default for SideData {
    fn default() -> Self {
        Self {
            selected_idx: None,
            sorting_options: SortingOptions {
                sort_by: FileColumn::Name,
                direction: SortDirection::Ascending,
            },
            path: PathBuf::new(),
            records: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReadDirData {
    pub path: PathBuf,
    pub records: Vec<FileRecord>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AppState {
    // TODO: save window size
    focused_side_left: bool,

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
            focused_side_left: true,

            left: SideData::default(),
            right: SideData::default(),

            fullscreen: false,
            maximized: false,
        }
    }
}

impl AppState {
    pub async fn load(path: &str) -> Result<AppState, LoadError> {
        let state_str = tokio::fs::read_to_string(path)
            .await
            .map_err(|_| LoadError::File)?;

        let mut state: AppState =
            serde_json::from_str(&state_str).map_err(|_| LoadError::Format)?;

        let side = state.get_selected_side();
        let other_side = match side {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        };

        let idx = state.get_selected_idx(side).unwrap_or(0);

        // TODO: refactor?
        {
            let path_to_open = state.get_path(side);
            let ReadDirData { path, records } =
                AppState::read_directory(path_to_open.to_path_buf())
                    .await
                    .map_err(|err| LoadError::GoToDir(err))?;

            state.save_read_directory_data(side, path, records);
        }
        {
            let side = other_side;
            let path_to_open = state.get_path(side);
            let ReadDirData { path, records } =
                AppState::read_directory(path_to_open.to_path_buf())
                    .await
                    .map_err(|err| LoadError::GoToDir(err))?;

            state.save_read_directory_data(side, path, records);
        }

        state.focus_side(side);
        state.set_selected_idx(side, idx);

        Ok(state)
    }

    pub async fn save(self, save_path: &str) -> Result<(), SaveError> {
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

    pub fn get_selected_side(&self) -> Side {
        match self.focused_side_left {
            true => Side::Left,
            false => Side::Right,
        }
    }

    pub fn get_path(&self, side: Side) -> &Path {
        match side {
            Side::Left => &self.left.path,
            Side::Right => &self.right.path,
        }
    }

    pub fn get_records(&self, side: Side) -> &Vec<FileRecord> {
        match side {
            Side::Left => &self.left.records,
            Side::Right => &self.right.records,
        }
    }

    pub fn get_sorting_options(&self, side: Side) -> &SortingOptions {
        match side {
            Side::Left => &self.left.sorting_options,
            Side::Right => &self.right.sorting_options,
        }
    }

    pub fn get_selected_idx(&self, side: Side) -> Option<usize> {
        match side {
            Side::Left => self.left.selected_idx,
            Side::Right => self.right.selected_idx,
        }
    }

    fn set_selected_idx(&mut self, side: Side, idx: usize) {
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

    fn focus_side(&mut self, side: Side) {
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

        self.focused_side_left = side.is_left();
    }

    pub fn toggle_side_focus(&mut self) {
        let new_side = match self.get_selected_side() {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        };

        self.focus_side(new_side);
    }

    pub fn select_record_at_idx(&mut self, side: Side, idx: usize) {
        self.focus_side(side);
        self.set_selected_idx(side, idx);
    }

    pub fn select_next_idx(&mut self, side: Side) {
        let records_len = self.get_records(side).len();
        let current_item = match self.get_selected_idx(side) {
            Some(idx) => idx,
            None => 0,
        };

        let next_item = current_item + 1;
        if next_item < records_len {
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

    pub fn get_hover_for_idx(&self, side: Side, idx: usize) -> bool {
        let records = match side {
            Side::Left => &self.left.records,
            Side::Right => &self.right.records,
        };
        let hover = records[idx].hover;

        hover.iter().any(|i| *i)
    }

    pub fn set_hover_for_idx_and_col(
        &mut self,
        side: Side,
        idx: usize,
        file_col: FileColumn,
        hover: bool,
    ) {
        let records = match side {
            Side::Left => &mut self.left.records,
            Side::Right => &mut self.right.records,
        };

        let record = &mut records[idx];
        let col_idx: usize = file_col.into();
        record.hover[col_idx] = hover;
    }

    pub fn sort_records(
        &mut self,
        side: Side,
        sorting_options: SortingOptions,
    ) {
        let records = match side {
            Side::Left => {
                self.left.sorting_options = sorting_options;
                &mut self.left.records
            }
            Side::Right => {
                self.right.sorting_options = sorting_options;
                &mut self.right.records
            }
        };

        sort_records(records, &sorting_options);
    }

    pub fn get_selected_file_name(&self, side: Side) -> &String {
        let idx = self
            .get_selected_idx(side)
            .expect("selected side must have idx");
        let records = self.get_records(side);

        &records[idx].file_name
    }

    pub async fn read_directory(
        path_to_open: PathBuf,
    ) -> Result<ReadDirData, ReadDirectoryError> {
        let canon_path = canonicalize(path_to_open)
            .map_err(|_| ReadDirectoryError::Canonicalize)?;

        let records = read_directory(&canon_path)
            .await
            .map_err(|e| ReadDirectoryError::ReadDir(e))?;

        Ok(ReadDirData {
            path: canon_path,
            records,
        })
    }

    pub fn save_read_directory_data(
        &mut self,
        side: Side,
        path: PathBuf,
        mut records: Vec<FileRecord>,
    ) {
        if path != PathBuf::from("/") {
            records.insert(0, FileRecord::new_go_back_record());
        }

        let side_data = match side {
            Side::Left => &mut self.left,
            Side::Right => &mut self.right,
        };

        sort_records(&mut records, &side_data.sorting_options);

        side_data.path = path;
        side_data.records = records;

        // TODO: we need cache to remeber previous select positon
        self.set_selected_idx(side, 0);
    }

    pub async fn read_dir_or_open_file(
        path: PathBuf,
        file_name: String,
    ) -> Result<Option<ReadDirData>, OpenRecordError> {
        let mut path_to_open = PathBuf::new();
        path_to_open.push(path);
        path_to_open.push(file_name);

        if is_dir(&path_to_open) {
            let data = AppState::read_directory(path_to_open)
                .await
                .map_err(|_| OpenRecordError::Error)?;

            Ok(Some(data))
        } else {
            open_file(path_to_open);
            Ok(None)
        }
    }
}
