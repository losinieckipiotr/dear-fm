use crate::{
    AppState, AppWindow, Side, render_files_window::render_files_window,
};
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
