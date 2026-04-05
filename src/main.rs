use std::path::PathBuf;

use env_logger::Env;
use iced::widget::{row, rule};
use iced::{Element, Size, Subscription, Task, Theme, keyboard, window};

use crate::message::Message;
use crate::side_view::side_view;
use crate::state::{AppState, Side};
use crate::table_view::table_view;

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

    iced::application(
        Application::default,
        Application::update,
        Application::view,
    )
    .title(Application::title)
    .theme(Application::theme)
    .subscription(Application::subscription)
    .window_size(Application::window_size())
    .run()
}

struct Application {
    state: AppState,
    // left_files: Vec<TableEntry>,
}

impl Default for Application {
    fn default() -> Self {
        Self {
            state: AppState::new(),
            // left_files: Vec::new(),
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

    fn subscription(&self) -> Subscription<Message> {
        let window_sub =
            window::events().filter_map(|(_id, event)| match event {
                window::Event::Opened {
                    position: _,
                    size: _,
                } => Some(Message::WindowOpened),
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
        match message {
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
            Message::ToggleFullscreen => {
                use window::{Mode, latest, mode, set_mode};

                return latest().and_then(move |id| {
                    mode(id).then(move |mode| match mode {
                        Mode::Fullscreen => set_mode(id, Mode::Windowed),
                        Mode::Windowed => set_mode(id, Mode::Fullscreen),
                        Mode::Hidden => Task::none(),
                    })
                });
            }
            Message::ToggleMaximize => {
                return window::latest()
                    .and_then(|id| window::toggle_maximize(id));
            }
            Message::Exit => {
                return iced::exit();
            }
        }

        Task::none()
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
