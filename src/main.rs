use crate::message::Message;
use crate::state::AppState;
use env_logger::Env;
use iced::{Element, Size, Subscription, Task, Theme, keyboard, window};

mod files;
mod message;
mod state;
mod table_view;
mod update;
mod view;

pub fn main() -> iced::Result {
    // env_logger::init();
    env_logger::init_from_env(
        Env::new().default_filter_or(log::Level::Info.as_str()),
    );

    iced::application(Application::load, Application::update, Application::view)
        .title(Application::title)
        .theme(Application::theme)
        .subscription(Application::subscription)
        .window_size(Application::window_size())
        .exit_on_close_request(false)
        .run()
}

struct Application {
    state: AppState,
    loading: bool,
    dirty: bool,
    saving: bool,
}

impl Default for Application {
    fn default() -> Self {
        Self {
            state: AppState::default(),
            loading: true,
            dirty: false,
            saving: false,
        }
    }
}

impl Application {
    const STATE_PATH: &'static str = "state.json";

    fn load() -> (Self, Task<Message>) {
        let future = AppState::load(Self::STATE_PATH);

        (Self::default(), Task::perform(future, Message::AppLoaded))
    }

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
                window::Event::CloseRequested => Some(Message::AppExit),
                _ => None,
            });

        use keyboard::{Event, Key, key::Named, listen};

        let keyboard_sub = listen().filter_map(|event| {
            let Event::KeyPressed {
                modified_key,
                // neat way to skip repeated keys
                // repeat,
                ..
            } = event
            else {
                return None;
            };

            match modified_key.as_ref() {
                Key::Named(Named::Escape) => Some(Message::AppExit),
                Key::Character("f") => Some(Message::WindowToggleFullscreen),
                Key::Character("m") => Some(Message::WindowToggleMaximize),
                Key::Named(Named::Tab) => Some(Message::ToggleSideFocus),
                Key::Named(Named::ArrowDown) => Some(Message::KeyArrowDown),
                Key::Named(Named::ArrowUp) => Some(Message::KeyArrowUp),
                Key::Named(Named::Enter) => Some(Message::KeyEnter),
                Key::Named(Named::Backspace) => Some(Message::KeyBackspace),
                _ => None,
            }
        });

        // TODO: interval to save state?

        Subscription::batch([window_sub, keyboard_sub])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        update::update(self, message)
    }

    fn view(&self) -> Element<'_, Message> {
        view::view(self)
    }
}
