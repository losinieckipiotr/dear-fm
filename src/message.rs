#[derive(Debug, Clone, Copy)]
pub enum Message {
    WindowOpened,
    ToggleFullscreen,
    ToggleMaximize,
    Exit,
}
