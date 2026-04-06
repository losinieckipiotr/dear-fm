use crate::{
    message::{Message, SaveError},
    state::AppState,
};
use iced::{Task, window};

use crate::Application;

pub fn update(app: &mut Application, message: Message) -> Task<Message> {
    let mut saved = false;
    let mut task = Task::none();

    log::info!("update() message: {:#?}", message);

    match message {
        Message::Loaded(result) => match result {
            Ok(state) => {
                app.loaded = true;
                app.state = state;
            }
            Err(_) => {
                log::error!("failed to load application state");
                task = iced::exit();
            }
        },
        Message::TestClick => {
            log::info!("test click")
        }
        Message::Saved(_result) => {
            app.saving = false;
            saved = true;
        }
        Message::ToggleFullscreen => {
            use window::{Mode, latest, mode, set_mode};

            task = latest().and_then(move |id| {
                mode(id).then(move |mode| match mode {
                    Mode::Fullscreen => set_mode(id, Mode::Windowed),
                    Mode::Windowed => set_mode(id, Mode::Fullscreen),
                    Mode::Hidden => Task::none(),
                })
            });
        }
        Message::ToggleMaximize => {
            task = window::latest().and_then(|id| window::toggle_maximize(id));
        }
        Message::Exit => {
            task = iced::exit();
        }
        Message::OpenFileOrDir(side, idx) => {
            let path_to_open = app.state.get_path_to_open_at(side, idx);
            app.state.go_to_or_open(side, path_to_open);
        }
    }

    // TODO: diff the state, and save only if changed?
    if !saved {
        app.dirty = true;
    }

    let save = if app.dirty && !app.saving {
        app.dirty = false;
        app.saving = true;

        Task::perform(save(app.state.clone()), Message::Saved)
    } else {
        Task::none()
    };

    Task::batch([task, save])
}

async fn save(state: AppState) -> Result<(), SaveError> {
    use iced::time::milliseconds;

    let json =
        serde_json::to_string_pretty(&state).map_err(|_| SaveError::Format)?;

    let mut path = std::env::current_dir().unwrap_or_default();
    path.push(Application::STATE_PATH);

    log::debug!("saving to path: {}", path.display());
    log::debug!("{}", json);

    tokio::fs::write(path, json.as_bytes())
        .await
        .map_err(|_| SaveError::Write)?;

    // This is a simple way to save at most twice every second
    tokio::time::sleep(milliseconds(500)).await;

    Ok(())
}
