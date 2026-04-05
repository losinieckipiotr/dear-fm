#[derive(Debug, Clone, Copy)]
pub enum SaveError {
    Write,
    Format,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Saved(Result<(), SaveError>),
    WindowOpened,
    ToggleFullscreen,
    ToggleMaximize,
    Exit,
}
