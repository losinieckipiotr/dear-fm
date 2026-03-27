use crate::{
    AppState, AppWindow, Side,
    files::{self},
};
use chrono::{DateTime, Local};
use humansize::{DECIMAL, format_size};
use imgui::*;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use winit::dpi::PhysicalSize;

struct RenderTableResult {
    table_clicked: bool,
    to_open_idx: Option<usize>,
}

pub fn render_frame(app_window: &mut AppWindow) {
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

    imgui
        .platform
        .prepare_frame(imgui.context.io_mut(), &app_window.window)
        .expect("Failed to prepare frame");

    let ui = imgui.context.frame();
    let window = &app_window.window;
    let inner_size = window.inner_size();

    ui.dockspace_over_main_viewport();
    let dt = now - state.last_frame_measure_time;

    if dt > Duration::from_secs(1) {
        let frame_rate = frame_count - state.last_measure_frame_count;

        state.frame_rate = frame_rate;
        state.last_frame_measure_time = now;
        state.last_measure_frame_count = frame_count;
    }

    let mut demo_open = state.demo_open;

    // TODO: render two windows instead main and child windows

    if demo_open {
        ui.show_demo_window(&mut demo_open);
    } else {
        if ui.is_key_pressed(imgui::Key::Tab) {
            state.toggle_window_focus();
        }

        render_side(Side::Left, ui, state, inner_size, frame_count);
        render_side(Side::Right, ui, state, inner_size, frame_count);
    }

    let mut encoder: wgpu::CommandEncoder =
        app_window.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: None },
        );

    if state.last_cursor != ui.mouse_cursor() {
        state.last_cursor = ui.mouse_cursor();
        imgui.platform.prepare_render(&ui, &app_window.window);
    }

    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
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

fn render_side(
    side: Side,
    ui: &Ui,
    mut state: &mut AppState,
    inner_size: PhysicalSize<u32>,
    frame_count: i32,
) {
    let path_to_open_option = render_files_window(
        ui,
        &mut state,
        inner_size.width as f32 / 2.0,
        inner_size.height as f32,
        side,
        frame_count,
    );

    if let Some(path_to_open) = path_to_open_option {
        log::debug!("{} window path_to_open: {}", side, path_to_open.display());

        if files::is_dir(&path_to_open) {
            state.go_to_directory(side, path_to_open);
        }
    }
}

fn render_files_window(
    ui: &Ui,
    mut state: &mut AppState,
    width: f32,
    height: f32,
    side: Side,
    frame_count: i32,
) -> Option<PathBuf> {
    let window_name: String = format!("{} window", side);
    let is_window_focused = state.is_window_focused(side);

    let mut path_to_open_option: Option<PathBuf> = None;

    /*
    - [ ] automatic windows resize, can be calculated based in ini file
    - [ ] unable to hide title bar
    */
    ui.window(window_name)
        .position([0.0, 0.0], Condition::FirstUseEver)
        .size([width, height], Condition::FirstUseEver)
        .collapsible(false)
        .focused(is_window_focused)
        .movable(false)
        .menu_bar(false)
        .title_bar(false)
        .no_decoration()
        .build(|| {
            {
                if !ui.is_window_focused() {
                    if ui.is_window_hovered() {
                        if ui.is_mouse_clicked(MouseButton::Left) {
                            log::debug!("no focus but click with hover");
                            state.focus_window(side);
                        }
                    }
                }

                let has_window_focus = ui.is_window_focused_with_flags(
                    imgui::WindowFocusedFlags::CHILD_WINDOWS,
                );

                if has_window_focus {
                    if ui.is_key_pressed(imgui::Key::DownArrow) {
                        state.select_next_idx(side);
                    } else if ui.is_key_pressed(imgui::Key::UpArrow) {
                        state.select_prev_idx(side);
                    } else if ui.is_key_pressed(imgui::Key::Enter) {
                        log::debug!("{} table enter pressed", side);

                        if let Some(idx) = state.get_selected_idx(side) {
                            path_to_open_option =
                                Some(state.get_path_to_open_at(side, idx));
                        }
                    }
                }

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

                    log::debug!(
                        "{} window, go back to {}",
                        side,
                        path_to_open.display()
                    );

                    state.go_to_directory(side, path_to_open);
                }
            }

            let render_table_result = render_table(ui, &mut state, side);

            if render_table_result.table_clicked {
                log::debug!("{} table clicked", side);
                state.focus_window(side);
            }

            if let Some(idx) = render_table_result.to_open_idx {
                path_to_open_option =
                    Some(state.get_path_to_open_at(side, idx));
            }

            let frame_rate = state.frame_rate;

            ui.text(format!("Frame rate: {frame_rate} FPS",));
            ui.text(format!("Frame count: {frame_count}"));
        });

    path_to_open_option
}

fn render_table(
    ui: &Ui,
    state: &mut AppState,
    side: Side,
) -> RenderTableResult {
    log::debug!("render_table");

    let table_token = ui
        .begin_table_with_flags(
            "table",
            3,
            TableFlags::SORTABLE
                | TableFlags::RESIZABLE
                | TableFlags::ROW_BG
                | TableFlags::SIZING_FIXED_FIT,
        )
        .unwrap();

    let files = state.get_window_files(side);
    let mut current_item = state.get_selected_idx(side);

    let mut double_clicked_idx: Option<usize> = None;
    let mut any_row_clicked = false;

    for (idx, file) in files.iter().enumerate() {
        ui.table_next_row();
        ui.table_next_column();

        let is_selected = match current_item {
            Some(current_item_idx) => idx == current_item_idx,
            None => false,
        };

        log::debug!("idx: {}, is_selected: {}", idx, is_selected);

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
                double_clicked_idx = Some(idx);
            }

            current_item = Some(idx);
            any_row_clicked = true;
        }

        ui.table_next_column();

        if file.is_go_back_record {
            ui.text("");
            ui.table_next_column();
            ui.text("");
        } else {
            let size = file.size;
            let is_file = file.is_file;
            let modified = file.modified;

            let formatted_size: String =
                format_size(size, DECIMAL.decimal_places(1));
            if is_file {
                ui.text(formatted_size);
            } else {
                ui.text("--");
            }
            ui.table_next_column();

            let datetime: DateTime<Local> = modified.into();

            ui.text(format!("{}", datetime.format("%d %b %Y at %H:%M")));
        }
    }

    table_token.end();

    if let Some(current_item_idx) = current_item {
        state.set_selected_idx(side, current_item_idx);
        ui.text(format!("current_item_idx: {current_item_idx}"));
    }

    RenderTableResult {
        table_clicked: any_row_clicked,
        to_open_idx: double_clicked_idx,
    }
}
