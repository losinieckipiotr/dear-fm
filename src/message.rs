use std::path::PathBuf;

use iced::window;

use crate::state::{AppState, LoadError, SaveError, Side};

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Result<AppState, LoadError>),
    Saved(Result<(), SaveError>),
    OpenFileOrDir(Side, usize),
    ToggleFullscreen,
    ToggleMaximize,
    Exit,
    HeaderHover(usize, bool),
    WindowMode(window::Mode),
    WindowMaximized(bool),
    ToggleWindowFocus,
    SelectIdx(Side, usize),
    ArrowDown,
    ArrowUp,
    Enter,
    PathButtonClick(Side, PathBuf),
}
