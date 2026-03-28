use crate::{
    AppState, AppWindow, Side,
    files::{self, FileRecord, SortBy, SortDirection},
};
use chrono::{DateTime, Local};
use humansize::{DECIMAL, format_size};
use imgui::*;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

pub fn render_frame(app_window: &mut AppWindow) {
    // initialization //////////////////////////////////////////////////////////
    let now = Instant::now();

    let imgui = app_window.imgui.as_mut().unwrap();
    let state = &mut app_window.state;

    let delta_time = now - state.last_frame;
    let io = imgui.context.io_mut();
    io.update_delta_time(delta_time);
    state.last_frame = now;

    io.config_flags = io.config_flags | ConfigFlags::DOCKING_ENABLE;
    imgui
        .context
        .set_ini_filename(Some(PathBuf::from("./imgui.ini")));

    let frame = match app_window.surface.get_current_texture() {
        Ok(frame) => frame,
        Err(e) => {
            log::warn!("dropped frame: {e:?}");
            return;
        }
    };
    let frame_count = imgui.context.frame_count();
    state.frame_count = frame_count;

    imgui
        .platform
        .prepare_frame(imgui.context.io_mut(), &app_window.window)
        .expect("Failed to prepare frame");

    let ui = imgui.context.frame();
    let window = &app_window.window;
    let inner_size = window.inner_size();

    ui.dockspace_over_main_viewport();
    let dt = now - state.last_frame_measure_time;

    // prepare fps data ////////////////////////////////////////////////////////
    if dt > Duration::from_secs(1) {
        let frame_rate = frame_count - state.last_measure_frame_count;

        state.frame_rate = frame_rate;
        state.last_frame_measure_time = now;
        state.last_measure_frame_count = frame_count;
    }

    let mut demo_open = state.demo_open;

    // render //////////////////////////////////////////////////////////////////
    if demo_open {
        ui.show_demo_window(&mut demo_open);
    } else {
        handgle_global_keys(ui, state);

        let width = inner_size.width as f32 / 2.0;
        let height = inner_size.height as f32;

        render_files_window(ui, state, width, height, Side::Left);
        render_files_window(ui, state, width, height, Side::Right);
    }

    // render end, start backend rendering /////////////////////////////////////
    if state.last_cursor != ui.mouse_cursor() {
        state.last_cursor = ui.mouse_cursor();
        imgui.platform.prepare_render(&ui, &app_window.window);
    }

    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder: wgpu::CommandEncoder =
        app_window.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: None },
        );

    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(imgui.clear_color),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });

    imgui
        .renderer
        .render(
            imgui.context.render(),
            &app_window.queue,
            &app_window.device,
            &mut rpass,
        )
        .expect("Rendering failed");

    drop(rpass);

    app_window.queue.submit(Some(encoder.finish()));

    frame.present();
}

fn handgle_global_keys(ui: &Ui, state: &mut AppState) {
    if ui.is_key_pressed(imgui::Key::Tab) {
        state.toggle_window_focus();
    }
}

fn render_files_window(
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
            focus_file_window_on_click(ui, state, side);
            handle_keybord_in_files_window(ui, state, side);
            render_path_buttons(ui, state, side);
            render_frames_info(ui, state);
            render_table(ui, &mut state, side);
        });
}

fn focus_file_window_on_click(ui: &Ui, state: &mut AppState, side: Side) {
    if !ui.is_window_focused() {
        if ui.is_window_hovered() {
            if ui.is_mouse_clicked(MouseButton::Left) {
                log::debug!("no focus but click with hover");
                state.focus_window(side);
            }
        }
    }
}

fn handle_keybord_in_files_window(ui: &Ui, state: &mut AppState, side: Side) {
    if ui.is_window_focused_with_flags(imgui::WindowFocusedFlags::CHILD_WINDOWS)
    {
        if ui.is_key_pressed(imgui::Key::DownArrow) {
            state.select_next_idx(side);
        } else if ui.is_key_pressed(imgui::Key::UpArrow) {
            state.select_prev_idx(side);
        } else if ui.is_key_pressed(imgui::Key::Enter) {
            log::debug!("{} window enter pressed", side);

            if let Some(idx) = state.get_selected_idx(side) {
                open_if_directory(
                    state,
                    state.get_path_to_open_at(side, idx),
                    side,
                );
            }
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
    }
}

fn open_if_directory(state: &mut AppState, path_to_open: PathBuf, side: Side) {
    log::debug!(
        "open_if_directory path_to_open: {}, side: {}",
        path_to_open.display(),
        side
    );

    if files::is_dir(&path_to_open) {
        state.go_to_directory(side, path_to_open);
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

fn render_frames_info(ui: &Ui, state: &mut AppState) {
    let frame_rate = state.frame_rate;
    let frame_count = state.frame_count;

    ui.text(format!("Frame rate: {frame_rate} FPS",));
    ui.text(format!("Frame count: {frame_count}"));
}

fn render_table(ui: &Ui, state: &mut AppState, side: Side) {
    log::debug!("render_table");

    let table_token = ui
        .begin_table_header_with_flags(
            "table",
            [
                TableColumnSetup::new("Name"),
                TableColumnSetup::new("Size"),
                TableColumnSetup::new("Modified"),
            ],
            TableFlags::SORTABLE
                | TableFlags::RESIZABLE
                | TableFlags::ROW_BG
                | TableFlags::SIZING_FIXED_FIT,
        )
        .unwrap();

    handle_table_sorting(ui, state, side);

    let files = state.get_window_files(side);
    let mut selected_idx_option = state.get_selected_idx(side);
    let mut double_clicked_idx_option: Option<usize> = None;
    let mut any_row_clicked = false;

    for (idx, file) in files.iter().enumerate() {
        ui.table_next_row();

        render_name_column(
            ui,
            idx,
            file,
            &mut selected_idx_option,
            &mut double_clicked_idx_option,
            &mut any_row_clicked,
        );
        render_size_column(ui, file);
        render_modified_column(ui, file);
    }

    table_token.end();

    handle_table_interactions(
        state,
        side,
        selected_idx_option,
        any_row_clicked,
        double_clicked_idx_option,
    )
}

fn handle_table_sorting(ui: &Ui, state: &mut AppState, side: Side) {
    if let Some(sort_data) = ui.table_sort_specs_mut() {
        sort_data.conditional_sort(|specs| {
            let spec = specs.iter().next().unwrap();

            let get_sort_by = |column_idx: usize| -> SortBy {
                match column_idx {
                    0 => SortBy::Name,
                    1 => SortBy::Size,
                    2 => SortBy::Modified,
                    _ => unimplemented!(),
                }
            };

            if let Some(kind) = spec.sort_direction() {
                let (sort_by, direction) = match kind {
                    TableSortDirection::Ascending => (
                        get_sort_by(spec.column_idx()),
                        SortDirection::Ascending,
                    ),
                    TableSortDirection::Descending => (
                        get_sort_by(spec.column_idx()),
                        SortDirection::Descending,
                    ),
                };

                state.sort_window_files(side, sort_by, direction);
            }
        });
    }
}

fn render_name_column(
    ui: &Ui,
    idx: usize,
    file: &FileRecord,
    selected_idx_option: &mut Option<usize>,
    double_clicked_idx_option: &mut Option<usize>,
    any_row_clicked: &mut bool,
) {
    ui.table_next_column();

    let is_selected = match selected_idx_option {
        Some(selected_idx) => idx == *selected_idx,
        None => false,
    };

    let clicked = ui
        .selectable_config(&file.file_name)
        .selected(is_selected)
        .flags(
            SelectableFlags::SPAN_ALL_COLUMNS
                | SelectableFlags::ALLOW_DOUBLE_CLICK,
        )
        .build();

    if clicked {
        log::debug!("clicked idx: {idx}");

        if ui.is_mouse_double_clicked(MouseButton::Left) {
            *double_clicked_idx_option = Some(idx);
        }

        *selected_idx_option = Some(idx);
        *any_row_clicked = true;

        ui.table_next_column();
    }
}

fn render_size_column(ui: &Ui, file: &FileRecord) {
    ui.table_next_column();

    if file.is_go_back_record {
        ui.text("");
    } else {
        let size = file.size;
        let is_file = file.is_file;

        let formatted_size: String =
            format_size(size, DECIMAL.decimal_places(1));
        if is_file {
            ui.text(formatted_size);
        } else {
            ui.text("--");
        }
    }
}

fn render_modified_column(ui: &Ui, file: &FileRecord) {
    ui.table_next_column();

    if file.is_go_back_record {
        ui.text("");
    } else {
        let modified = file.modified;
        let datetime: DateTime<Local> = modified.into();

        ui.text(format!("{}", datetime.format("%d %b %Y at %H:%M")));
    }
}

fn handle_table_interactions(
    state: &mut AppState,
    side: Side,
    selected_idx_option: Option<usize>,
    any_row_clicked: bool,
    double_clicked_idx_option: Option<usize>,
) {
    if let Some(selected_idx) = selected_idx_option {
        state.set_selected_idx(side, selected_idx);
    }

    if any_row_clicked {
        log::debug!("{} table clicked", side);

        state.focus_window(side);
    }

    if let Some(idx) = double_clicked_idx_option {
        let path_to_open = state.get_path_to_open_at(side, idx);

        open_if_directory(state, path_to_open, side);
    }
}
