use std::path::PathBuf;

use imgui::{Condition, MouseButton, Ui};

use crate::{
    render_table::render_table,
    state::{AppState, Side},
};

pub fn render_files_window(
    ui: &Ui,
    mut state: &mut AppState,
    width: f32,
    height: f32,
    side: Side,
) {
    let is_window_focused = state.is_window_focused(side);

    /*
    - [ ] automatic windows resize, can be calculated based in ini file
    - [ ] unable to hide title bar
    */
    ui.window(format!("{} window", side))
        .position([0.0, 0.0], Condition::FirstUseEver)
        .size([width, height], Condition::FirstUseEver)
        .collapsible(false)
        .focused(is_window_focused)
        .movable(false)
        .menu_bar(false)
        .title_bar(false)
        .build(|| {
            focus_on_click(ui, state, side);
            handle_keybord(ui, state, side);
            render_path_buttons(ui, state, side);
            render_frames_info(ui, state);
            render_table(ui, &mut state, side);
        });
}

fn focus_on_click(ui: &Ui, state: &mut AppState, side: Side) {
    if !ui.is_window_focused() {
        if ui.is_window_hovered() {
            if ui.is_mouse_clicked(MouseButton::Left) {
                log::debug!("no focus but click with hover");
                state.focus_window(side);
            }
        }
    }
}

fn handle_keybord(ui: &Ui, state: &mut AppState, side: Side) {
    if ui.is_window_focused_with_flags(imgui::WindowFocusedFlags::CHILD_WINDOWS)
    {
        if ui.is_key_pressed(imgui::Key::DownArrow) {
            state.select_next_idx(side);
        } else if ui.is_key_pressed(imgui::Key::UpArrow) {
            state.select_prev_idx(side);
        } else if ui.is_key_pressed(imgui::Key::Enter) {
            log::debug!("{} window enter pressed", side);

            if let Some(idx) = state.get_selected_idx(side) {
                state.go_to_or_open(side, state.get_path_to_open_at(side, idx));
            }
        } else if ui.is_key_pressed(imgui::Key::Space) {
            // TODO: should I make my child window and my preview?
        } else if ui.is_key_pressed(imgui::Key::Backspace) {
            log::info!("{} window backspace pressed", side);

            let mut path_to_open = PathBuf::from(state.get_path(side));

            path_to_open.push("../");

            log::info!(
                "{} window, go back to {}",
                side,
                path_to_open.display()
            );

            state.go_to_directory(side, path_to_open);
        }
        // TODO: open in preview - space
    }
}

fn render_path_buttons(ui: &Ui, state: &mut AppState, side: Side) {
    let path = state.get_path(side);
    let buf = PathBuf::from(path);

    let mut clicked_index = -1;
    buf.iter().enumerate().for_each(|(i, p)| {
        if i > 0 {
            ui.same_line();
        }

        if ui.button(format!("{}", p.display())) {
            clicked_index = i as i32;
        }
    });

    if clicked_index >= 0 {
        let mut path_to_open = PathBuf::new();

        buf.iter().take(clicked_index as usize + 1).for_each(|i| {
            path_to_open.push(i);
        });

        log::debug!("{} window, go back to {}", side, path_to_open.display());

        state.go_to_directory(side, path_to_open);
    }
}

pub fn render_frames_info(ui: &Ui, state: &mut AppState) {
    let frame_rate = state.frame_rate;
    let frame_count = state.frame_count;

    ui.text(format!("Frame rate: {frame_rate} FPS",));
    ui.text(format!("Frame count: {frame_count}"));
}
