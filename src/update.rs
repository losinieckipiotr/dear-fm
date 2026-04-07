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
                            .chain(Task::done(Message::WindowMaximized(true)))
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
        Message::ToggleWindowFocus => {
            app.state.toggle_window_focus();

            (Task::none(), true)
        }
        Message::SelectIdx(side, idx) => {
            app.state.focus_window(side);
            app.state.set_selected_idx(side, idx);

            (Task::none(), true)
        }
        Message::ArrowDown => {
            let side = app.state.get_selected_side();
            app.state.select_next_idx(side);

            (Task::none(), true)
        }
        Message::ArrowUp => {
            let side = app.state.get_selected_side();
            app.state.select_prev_idx(side);

            (Task::none(), true)
        }
        Message::Enter => {
            let side = app.state.get_selected_side();
            let idx = app
                .state
                .get_selected_idx(side)
                .expect("selected side must have idx");
            let path_to_open = app.state.get_path_to_open_at(side, idx);
            app.state.go_to_or_open(side, path_to_open);

            (Task::none(), true)
        }
        Message::PathButtonClick(side, path_to_open) => {
            app.state.go_to_directory(side, path_to_open);

            (Task::none(), false)
        }
        Message::FileHover(side, idx, file_col, hover) => {
            app.state.update_hover(side, idx, file_col, hover);

            (Task::none(), false)
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
