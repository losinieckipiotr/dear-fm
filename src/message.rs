use crate::state::{AppState, Side};

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

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Result<AppState, LoadError>),
    Saved(Result<(), SaveError>),
    OpenFileOrDir(Side, usize),
    // WindowOpened,
    ToggleFullscreen,
    ToggleMaximize,
    Exit,
    TestClick,
}
