use crate::message::Message;
use iced::{Task, window};

use crate::Application;

pub fn update(app: &mut Application, message: Message) -> Task<Message> {
    log::info!("update() message: {:#?}", message);

    let (task, save_state): (Task<Message>, bool) = match message {
        Message::Loaded(result) => match result {
            Ok(state) => {
                app.loaded = true;
                app.state = state;

                let task = if app.state.fullscreen {
                    app.state.maximized = false;

                    window::latest().and_then(|id| {
                        window::set_mode(id, window::Mode::Fullscreen)
                    })
                } else {
                    if app.state.maximized {
                        window::latest()
                            .and_then(|id| window::maximize(id, true))
                    } else {
                        Task::none()
                    }
                };

                (task, false)
            }
            Err(_) => {
                log::error!("failed to load application state");

                (iced::exit(), false)
            }
        },
        Message::HeaderHover(idx, hover) => {
            app.state.header_hover[idx] = hover;

            (Task::none(), false)
        }
        Message::TestClick => {
            log::info!("test click");

            (Task::none(), false)
        }
        Message::Saved(_result) => {
            app.saving = false;

            (Task::none(), false)
        }
        Message::WindowMode(mode) => {
            let fullscreen = match mode {
                window::Mode::Fullscreen => true,
                _ => false,
            };
            app.state.fullscreen = fullscreen;

            (Task::none(), true)
        }
        Message::ToggleFullscreen => {
            use window::{Mode, latest, mode, set_mode};

            let task: Task<Message> = latest().and_then(move |id| {
                mode(id).then(move |mode| match mode {
                    Mode::Fullscreen => set_mode(id, Mode::Windowed)
                        .chain(Task::done(Message::WindowMode(Mode::Windowed))),
                    Mode::Windowed => set_mode(id, Mode::Fullscreen).chain(
                        Task::done(Message::WindowMode(Mode::Fullscreen)),
                    ),
                    window::Mode::Hidden => Task::none(),
                })
            });

            (task, false)
        }
        Message::WindowMaximized(maximized) => {
            app.state.maximized = maximized;

            (Task::none(), true)
        }
        Message::ToggleMaximize => {
            let is_maximized_t = window::latest().and_then(|id| {
                window::is_maximized(id)
                    .map(|maximized| Message::WindowMaximized(maximized))
            });

            let toggle_t = window::latest()
                .and_then(|id| window::toggle_maximize(id))
                .chain(is_maximized_t);

            (toggle_t, false)
        }
        Message::Exit => (iced::exit(), true),
        Message::OpenFileOrDir(side, idx) => {
            let path_to_open = app.state.get_path_to_open_at(side, idx);
            app.state.go_to_or_open(side, path_to_open);

            (Task::none(), true)
        }
    };

    if save_state {
        app.dirty = true;
    }

    let save = if app.dirty && !app.saving {
        app.dirty = false;
        app.saving = true;

        Task::perform(
            app.state.clone().save(Application::STATE_PATH),
            Message::Saved,
        )
    } else {
        Task::none()
    };

    Task::batch([task, save])
}
