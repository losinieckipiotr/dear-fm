use crate::message::{Message, SaveError};
use crate::side_view::side_view;
use crate::state::{AppState, Side};
use crate::table_view::table_view;
use env_logger::Env;
use iced::widget::{row, rule};
use iced::{Element, Size, Subscription, Task, Theme, keyboard, window};
use std::path::PathBuf;

mod files;
mod message;
mod side_view;
mod state;
mod table_view;

pub fn main() -> iced::Result {
    // env_logger::init();
    env_logger::init_from_env(
        Env::new().default_filter_or(log::Level::Info.as_str()),
    );

    let res = iced::application(
        Application::default,
        Application::update,
        Application::view,
    )
    .title(Application::title)
    .theme(Application::theme)
    .subscription(Application::subscription)
    .window_size(Application::window_size())
    // TODO:
    // .exit_on_close_request(false)
    .run();

    log::info!("exit");

    res
}

struct Application {
    state: AppState,
    dirty: bool,
    saving: bool,
    // left_files: Vec<TableEntry>,
}

impl Default for Application {
    fn default() -> Self {
        Self {
            state: AppState::default(),
            dirty: false,
            saving: false,
        }
    }
}

impl Application {
    fn title(&self) -> String {
        let version = env!("CARGO_PKG_VERSION");

        format!("Dear File Manager {version}")
    }

    fn theme(&self) -> Theme {
        Theme::Dracula
    }

    fn window_size() -> Size {
        Size {
            width: 800.0,
            height: 600.0,
        }
    }

    // TODO:
    // async fn load() -> Result<SavedState, LoadError> {
    //     let contents = tokio::fs::read_to_string(Self::path())
    //         .await
    //         .map_err(|_| LoadError::File)?;

    //     serde_json::from_str(&contents).map_err(|_| LoadError::Format)
    // }

    async fn save(state: AppState) -> Result<(), SaveError> {
        use iced::time::milliseconds;

        let json = serde_json::to_string_pretty(&state)
            .map_err(|_| SaveError::Format)?;

        let mut path = std::env::current_dir().unwrap_or_default();
        path.push("state.json");

        log::info!("saving to path: {}", path.display());
        log::info!("{}", json);

        tokio::fs::write(path, json.as_bytes())
            .await
            .map_err(|_| SaveError::Write)?;

        // This is a simple way to save at most twice every second
        tokio::time::sleep(milliseconds(500)).await;

        Ok(())
    }

    fn subscription(&self) -> Subscription<Message> {
        let window_sub =
            window::events().filter_map(|(_id, event)| match event {
                window::Event::Opened {
                    position: _,
                    size: _,
                } => Some(Message::WindowOpened),
                // TODO:
                // window::Event::CloseRequested => Some(Message::WindowClosed),
                _ => None,
            });

        use keyboard::{Event, Key, key::Named, listen};

        let keyboard_sub = listen().filter_map(|event| {
            let Event::KeyPressed {
                modified_key,
                repeat: false,
                ..
            } = event
            else {
                return None;
            };

            match modified_key.as_ref() {
                Key::Named(Named::Escape) => Some(Message::Exit),
                Key::Character("f") => Some(Message::ToggleFullscreen),
                Key::Character("m") => Some(Message::ToggleMaximize),
                _ => None,
            }
        });

        Subscription::batch([window_sub, keyboard_sub])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        let mut saved = false;
        let mut task = Task::none();

        match message {
            Message::Saved(_result) => {
                self.saving = false;
                saved = true;
            }
            Message::WindowOpened => {
                log::info!("Window opened");

                self.state.go_to_directory(
                    Side::Left,
                    PathBuf::from("/Users/piotrlosiniecki"),
                );
                self.state.go_to_directory(
                    Side::Right,
                    PathBuf::from("/Users/piotrlosiniecki/Projects"),
                );
                self.state.focus_window(Side::Left);
            }
            // TODO:
            // Message::WindowClosed => {
            //     log::info!("Window closed")
            // }
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
                task =
                    window::latest().and_then(|id| window::toggle_maximize(id));
            }
            Message::Exit => {
                task = iced::exit();
            }
        }

        if !saved {
            self.dirty = true;
        }

        let save = if self.dirty && !self.saving {
            self.dirty = false;
            self.saving = true;

            Task::perform(Application::save(self.state.clone()), Message::Saved)
        } else {
            Task::none()
        };

        Task::batch([task, save])
    }

    fn view(&self) -> Element<'_, Message> {
        row![
            side_view(&self.state, Side::Left),
            rule::vertical(1),
            side_view(&self.state, Side::Right),
        ]
        .into()
    }
}
