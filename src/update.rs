use crate::{
    message::Message,
    state::{AppState, ReadDirData},
};
use iced::{Task, window};

use crate::Application;

pub fn update(app: &mut Application, message: Message) -> Task<Message> {
    log::debug!("update() message: {:#?}", message);

    let (task, save_state): (Task<Message>, bool) = match message {
        Message::AppExit => (iced::exit(), false),
        Message::AppLoaded(result) => match result {
            Ok(state) => {
                app.loading = false;
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

                (task, true)
            }
            Err(load_error) => {
                log::error!("failed to load application state: {load_error}");

                (iced::exit(), false)
            }
        },
        Message::AppSaved(_result) => {
            app.saving = false;

            (Task::none(), false)
        }
        Message::WindowFullscreen(fullscreen) => {
            app.state.fullscreen = fullscreen;

            (Task::none(), true)
        }
        Message::WindowToggleFullscreen => {
            use window::{Mode, latest, mode, set_mode};

            let task: Task<Message> = latest().and_then(move |id| {
                mode(id).then(move |mode| match mode {
                    Mode::Fullscreen => set_mode(id, Mode::Windowed)
                        .chain(Task::done(Message::WindowFullscreen(false))),
                    Mode::Windowed => set_mode(id, Mode::Fullscreen)
                        .chain(Task::done(Message::WindowFullscreen(true))),
                    window::Mode::Hidden => Task::none(),
                })
            });

            (task, false)
        }
        Message::WindowMaximized(maximized) => {
            app.state.maximized = maximized;

            (Task::none(), true)
        }
        Message::WindowToggleMaximize => {
            let is_maximized_t = window::latest().and_then(|id| {
                window::is_maximized(id)
                    .map(|maximized| Message::WindowMaximized(maximized))
            });

            let toggle_t = window::latest()
                .and_then(|id| window::toggle_maximize(id))
                .chain(is_maximized_t);

            (toggle_t, false)
        }
        Message::ToggleSideFocus => {
            app.state.toggle_side_focus();

            (Task::none(), true)
        }
        Message::SelectRecord(side, idx) => {
            app.state.select_record_at_idx(side, idx);

            (Task::none(), true)
        }
        Message::RecordHover(side, idx, file_col, hover) => {
            app.state
                .set_hover_for_idx_and_col(side, idx, file_col, hover);

            (Task::none(), false)
        }
        Message::SortRecords(side, sorting_options) => {
            app.state.sort_records(side, sorting_options);
            (Task::none(), true)
        }
        Message::KeyArrowDown => {
            let side = app.state.get_selected_side();
            app.state.select_next_idx(side);

            (Task::none(), true)
        }
        Message::KeyArrowUp => {
            let side = app.state.get_selected_side();
            app.state.select_prev_idx(side);

            (Task::none(), true)
        }
        Message::KeyEnter => (Task::done(Message::RecordDoubleClick), false),
        Message::PathButtonClick(side, path_to_open) => {
            app.loading = true;
            (
                Task::perform(
                    AppState::read_directory(path_to_open),
                    move |result| Message::DirectoryOpened(side, result),
                ),
                false,
            )
        }

        Message::RecordDoubleClick => {
            let side = app.state.get_selected_side();
            let file_name = app.state.get_selected_file_name(side);
            app.loading = true;

            let path = app.state.get_path(side);

            (
                Task::perform(
                    AppState::read_dir_or_open_file(
                        path.to_path_buf(),
                        file_name.clone(),
                    ),
                    move |result| match result {
                        Ok(option) => match option {
                            Some(result) => {
                                Message::DirectoryOpened(side, Ok(result))
                            }
                            None => Message::FileOpened,
                        },
                        Err(e) => {
                            log::error!("failed to opend record {:#?}", e);

                            Message::AppExit
                        }
                    },
                ),
                false,
            )
        }
        Message::DirectoryOpened(side, result) => {
            app.loading = false;

            match result {
                Ok(ReadDirData { path, records }) => {
                    app.state.save_read_directory_data(side, path, records);

                    (Task::none(), true)
                }
                Err(error) => {
                    log::error!("failed to open directory: {error}");

                    (iced::exit(), false)
                }
            }
        }
        Message::FileOpened => {
            app.loading = false;

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
            Message::AppSaved,
        )
    } else {
        Task::none()
    };

    Task::batch([task, save])
}
