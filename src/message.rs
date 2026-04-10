use crate::{
    files::{FileColumn, SortingOptions},
    state::{
        AppState, LoadError, ReadDirData, ReadDirectoryError, SaveError, Side,
    },
};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    AppExit,
    AppLoaded(Result<AppState, LoadError>),
    AppSaved(Result<(), SaveError>),

    WindowFullscreen(bool),
    WindowToggleFullscreen,

    WindowMaximized(bool),
    WindowToggleMaximize,

    ToggleSideFocus,
    SelectRecord(Side, usize),

    RecordHover(Side, usize, FileColumn, bool),
    SortRecords(Side, SortingOptions),

    KeyArrowDown,
    KeyArrowUp,
    KeyEnter,

    PathButtonClick(Side, PathBuf),
    RecordDoubleClick,
    DirectoryOpened(Side, Result<ReadDirData, ReadDirectoryError>),
    FileOpened,
}
