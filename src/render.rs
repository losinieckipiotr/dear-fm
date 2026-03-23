use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};

use crate::{AppState, AppWindow, Side, files};

use imgui::*;

struct RenderTableResult {
    table_clicked: bool,
    to_open_idx: Option<usize>,
}

pub fn render_frame(app_window: &mut AppWindow) {
    let now = Instant::now();

    let imgui = app_window.imgui.as_mut().unwrap();

    {
        let mut test_state = app_window.state.borrow_mut();

        let delta_time = now - test_state.last_frame;
        imgui.context.io_mut().update_delta_time(delta_time);
        test_state.last_frame = now;
    }

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
    let scale = window.scale_factor();
    let width = ((inner_size.width as f64) / scale) as f32;
    let height = (inner_size.height as f64 / scale) as f32;

    let mut demo_open: bool;
    {
        let mut test_state = app_window.state.borrow_mut();
        let dt = now - test_state.last_frame_measure_time;

        if dt > Duration::from_secs(1) {
            // TODO: cleanup frame rate
            // let frame_rate2 = ui.io().framerate;
            let frame_rate = frame_count - test_state.last_measure_frame_count;

            test_state.frame_rate = frame_rate;
            test_state.last_frame_measure_time = now;
            test_state.last_measure_frame_count = frame_count;
        }

        demo_open = test_state.demo_open;
    }

    // TODO: render two windows instead main and child windows

    if demo_open {
        ui.show_demo_window(&mut demo_open);
    } else {
        ui.window("main_window")
            .size([width, height], Condition::Always)
            .always_auto_resize(true)
            .position([0.0, 0.0], Condition::Appearing)
            .collapsible(false)
            .resizable(false)
            .movable(false)
            .title_bar(false)
            .scrollable(false)
            .scroll_bar(false)
            .build(|| {
                {
                    let mut test_state = app_window.state.borrow_mut();

                    if ui.is_key_pressed(imgui::Key::Tab) {
                        test_state.focused_window_left =
                            !test_state.focused_window_left;
                    }
                }
                {
                    let test_state = app_window.state.borrow();
                    let frame_rate = test_state.frame_rate;

                    ui.text(format!("Frame rate: {frame_rate} FPS",));
                    ui.text(format!("Frame count: {frame_count}"));
                }

                let content_region_avail = ui.content_region_avail();
                let half_screen = content_region_avail[0] / 2.0;
                let main_window_h = content_region_avail[1];

                let mut path_to_open_option: Option<PathBuf>;

                path_to_open_option = render_files_window(
                    ui,
                    app_window.state.clone(),
                    half_screen,
                    main_window_h,
                    Side::Left,
                );

                // TODO: refacotr left and right side
                if let Some(path_to_open) = path_to_open_option {
                    log::info!(
                        "left window path_to_open: {}",
                        path_to_open.display()
                    );

                    if files::is_dir(&path_to_open) {
                        let mut test_state = app_window.state.borrow_mut();

                        let path_str = path_to_open.as_path().to_str().unwrap();
                        let files = files::read_directory(path_str);

                        test_state.app_files.left_path = path_str.to_string();
                        test_state.app_files.left_files = files;
                        // TODO: handle case if directory is empty?
                        test_state.set_selected_idx(Side::Left, 0);

                        app_window.window.request_redraw();
                    }
                }

                ui.same_line();

                path_to_open_option = render_files_window(
                    ui,
                    app_window.state.clone(),
                    0.0,
                    main_window_h,
                    Side::Right,
                );

                if let Some(path_to_open) = path_to_open_option {
                    log::info!(
                        "right window path_to_open: {}",
                        path_to_open.display()
                    );

                    if files::is_dir(&path_to_open) {
                        let mut test_state = app_window.state.borrow_mut();

                        let path_str = path_to_open.as_path().to_str().unwrap();
                        let files = files::read_directory(path_str);

                        test_state.app_files.right_path = path_str.to_string();
                        test_state.app_files.right_files = files;
                        test_state.set_selected_idx(Side::Right, 0);

                        app_window.window.request_redraw();
                    }
                }
            });
    }

    let mut encoder: wgpu::CommandEncoder =
        app_window.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: None },
        );

    {
        let mut test_state = app_window.state.borrow_mut();

        if test_state.last_cursor != ui.mouse_cursor() {
            test_state.last_cursor = ui.mouse_cursor();
            imgui.platform.prepare_render(&ui, &app_window.window);
        }
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

/// Renders files window (left or right).
fn render_files_window(
    ui: &Ui,
    state: Rc<RefCell<AppState>>,
    width: f32,
    height: f32,
    side: Side,
) -> Option<PathBuf> {
    let window_name: String = format!("{} window", side.as_str());
    let is_window_focused = state.borrow().is_window_focused(side);

    let mut path_to_open_option: Option<PathBuf> = None;

    ui.child_window(window_name)
        .size([width, height])
        .border(true)
        .focused(is_window_focused)
        .build(|| {
            {
                let mut state = state.borrow_mut();
                let files_len = state.get_window_files(side).len();

                let has_window_focus = ui.is_window_focused_with_flags(
                    imgui::WindowFocusedFlags::CHILD_WINDOWS,
                );

                ui.text(format!("Has focus: {has_window_focus}"));

                let current_item = state.get_selected_idx(side);

                if has_window_focus {
                    if ui.is_key_pressed(imgui::Key::DownArrow) {
                        let next_item = current_item + 1;
                        if next_item < files_len as i32 {
                            state.set_selected_idx(side, next_item);
                        }
                    } else if ui.is_key_pressed(imgui::Key::UpArrow) {
                        let prev_item = current_item - 1;
                        if prev_item >= 0 {
                            state.set_selected_idx(side, prev_item);
                        }
                    } else if ui.is_key_pressed(imgui::Key::Enter) {
                        // TODO: refactor, with below code when tab element is clicked
                        log::info!("{} table enter pressed", side.as_str());

                        let files = state.get_window_files(side);
                        let path = state.get_path(side);
                        let element_to_open = &files[current_item as usize];
                        let path_to_open: PathBuf =
                            [path, element_to_open].iter().collect();

                        path_to_open_option = Some(path_to_open);
                        // TODO: should i return early?
                    }
                }

                let path = state.get_path(side);
                ui.text(format!("Path: {path}"));
            }

            let render_table_result = render_table(ui, state.clone(), side);

            {
                let mut state = state.borrow_mut();

                if render_table_result.table_clicked {
                    log::debug!("{} table clicked", side.as_str());
                    state.focused_window_left = side.is_left();
                }

                if let Some(idx) = render_table_result.to_open_idx {
                    let files = state.get_window_files(side);
                    let path = state.get_path(side);
                    let element_to_open = &files[idx];
                    let path_to_open: PathBuf =
                        [path, element_to_open].iter().collect();

                    path_to_open_option = Some(path_to_open);
                }
            }
        });

    if ui.is_item_clicked() {
        log::debug!("{} window clicked", side.as_str());

        let mut state = state.borrow_mut();
        state.focused_window_left = side.is_left();
    }

    path_to_open_option
}

/// Renders table and some debug info about it.
fn render_table(
    ui: &Ui,
    state: Rc<RefCell<AppState>>,
    side: Side,
) -> RenderTableResult {
    let table_token = ui
        .begin_table_with_flags(
            "table",
            2,
            TableFlags::SORTABLE | TableFlags::RESIZABLE | TableFlags::ROW_BG,
        )
        .unwrap();

    let mut state = state.borrow_mut();
    let files = state.get_window_files(side);
    let mut current_item = state.get_selected_idx(side);

    let mut double_clicked_idx: Option<usize> = None;
    let mut any_row_clicked = false;

    for (idx, file) in files.iter().enumerate() {
        ui.table_next_row();
        ui.table_next_column();

        let clicked = ui
            .selectable_config(format!("{idx}"))
            .selected(idx == current_item as usize)
            .flags(
                SelectableFlags::SPAN_ALL_COLUMNS
                    | SelectableFlags::ALLOW_DOUBLE_CLICK,
            )
            .build();

        if clicked {
            log::info!("clicked idx: {idx}");
            if ui.is_mouse_double_clicked(MouseButton::Left) {
                double_clicked_idx = Some(idx);
            }

            current_item = idx as i32;
            any_row_clicked = true;
        }

        ui.table_next_column();
        ui.text(file);
    }

    table_token.end();

    state.set_selected_idx(side, current_item);

    ui.text(format!("current_item: {current_item}"));

    RenderTableResult {
        table_clicked: any_row_clicked,
        to_open_idx: double_clicked_idx,
    }
}
