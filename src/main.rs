use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use pollster::block_on;
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use wgpu::Color;
use winit::{
    application::ApplicationHandler,
    event::{Event, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
    platform::macos::WindowExtMacOS,
    window::Window,
};

use env_logger::Env;

use crate::files::read_directory;

mod files;
mod render;

struct AppFiles {
    left_path: String,
    right_path: String,
    left_files: Vec<String>,
    right_files: Vec<String>,
}

pub struct ImguiState {
    context: imgui::Context,
    platform: WinitPlatform,
    renderer: Renderer,
    clear_color: wgpu::Color,
    demo_open: bool,
    limit_fps: bool,
    last_frame: Instant,
    last_cursor: Option<MouseCursor>,
    last_frame_measure_time: Instant,
    last_measure_frame_count: i32,
    frame_rate: i32,
    // TODO: make selected idexes optional
    left_item_selected_idx: i32,
    right_item_selected_idx: i32,
    focused_window_left: bool,
    app_files: AppFiles,
}

struct AppWindow {
    device: wgpu::Device,
    queue: wgpu::Queue,
    window: Arc<Window>,
    surface_desc: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'static>,
    hidpi_factor: f64,
    imgui: Option<ImguiState>,
}

struct App {
    app_window: Option<AppWindow>,
}

impl App {
    fn new() -> Self {
        Self {
            app_window: Option::None,
        }
    }

    fn reset_state(&mut self) {
        let new_state = App::new();

        self.app_window = new_state.app_window;
    }
}

fn main() {
    // env_logger::init();
    env_logger::init_from_env(
        Env::new().default_filter_or(log::Level::Info.as_str()),
    );

    // TODO: event_loop proxy for dispatching actions

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);
    let result = event_loop.run_app(&mut App::new());

    if let Err(e) = result {
        log::error!("Application error: {:#?}", e);
    }
}

impl AppWindow {
    fn new(event_loop: &ActiveEventLoop) -> Self {
        let mut window = Self::setup_gpu(event_loop);
        window.setup_imgui();

        window
    }

    fn get_surface_desc(window: &Arc<Window>) -> wgpu::SurfaceConfiguration {
        let size = window.inner_size();

        log::debug!("AppWindow.configure_surface size: {:#?}", size);

        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![wgpu::TextureFormat::Bgra8Unorm],
        }
    }

    fn setup_gpu(event_loop: &ActiveEventLoop) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        // let monitor = event_loop.primary_monitor().unwrap();
        // let size = monitor.size();

        let window = {
            let version = env!("CARGO_PKG_VERSION");

            let attributes = Window::default_attributes()
                // .with_inner_size(LogicalSize::new(size.width, size.height))
                .with_title(format!("Dear File Manager {version}"));
            Arc::new(event_loop.create_window(attributes).unwrap())
        };

        let hidpi_factor = window.scale_factor();
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter =
            block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }))
            .unwrap();

        let (device, queue) = block_on(
            adapter.request_device(&wgpu::DeviceDescriptor::default()),
        )
        .unwrap();

        let surface_desc = Self::get_surface_desc(&window);
        surface.configure(&device, &surface_desc);

        let imgui = None;
        Self {
            device,
            queue,
            window,
            surface_desc,
            surface,
            hidpi_factor,
            imgui,
        }
    }

    fn setup_imgui(&mut self) {
        let mut context = imgui::Context::create();
        let mut platform = WinitPlatform::new(&mut context);

        platform.attach_window(
            context.io_mut(),
            &self.window,
            HiDpiMode::Default,
        );
        context.set_ini_filename(None);

        // may crash for too big font size and high oversampling, probably because of GPU memory limits?
        const FONT_SIZE: f64 = 18.0;
        const OVERSAMPLE_H: i32 = 2;
        const OVERSAMPLE_V: i32 = 2;
        const RASTERIZER_MULTIPLY: f32 = 1.0;

        let font_size = (FONT_SIZE * self.hidpi_factor) as f32;
        context.io_mut().font_global_scale = (1.0 / self.hidpi_factor) as f32;

        context.fonts().add_font(&[
            FontSource::TtfData {
                data: include_bytes!("../resources/Roboto-Regular.ttf"),
                size_pixels: font_size,
                config: Some(FontConfig {
                    // As imgui-glium-renderer isn't gamma-correct with
                    // it's font rendering, we apply an arbitrary
                    // multiplier to make the font a bit "heavier". With
                    // default imgui-glow-renderer this is unnecessary.
                    rasterizer_multiply: RASTERIZER_MULTIPLY,
                    // Oversampling font helps improve text rendering at
                    // expense of larger font atlas texture.
                    oversample_h: OVERSAMPLE_H,
                    oversample_v: OVERSAMPLE_V,
                    ..FontConfig::default()
                }),
            },
            FontSource::TtfData {
                data: include_bytes!("../resources/mplus-1p-regular.ttf"),
                size_pixels: font_size,
                config: Some(FontConfig {
                    // Oversampling font helps improve text rendering at
                    // expense of larger font atlas texture.
                    oversample_h: 4,
                    oversample_v: 4,
                    // Range of glyphs to rasterize
                    glyph_ranges: FontGlyphRanges::japanese(),
                    ..FontConfig::default()
                }),
            },
        ]);

        let renderer_config = RendererConfig {
            texture_format: self.surface_desc.format,
            ..Default::default()
        };

        let renderer = Renderer::new(
            &mut context,
            &self.device,
            &self.queue,
            renderer_config,
        );
        let last_frame = Instant::now();

        self.imgui = Some(ImguiState {
            context,
            platform,
            renderer,
            clear_color: Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
            demo_open: false,
            limit_fps: true,
            last_frame,
            last_cursor: None,
            last_frame_measure_time: last_frame,
            last_measure_frame_count: 0,
            frame_rate: 0,
            left_item_selected_idx: 0,
            right_item_selected_idx: 0,
            focused_window_left: true,
            app_files: AppFiles {
                left_path: String::from("/Users/piotrlosiniecki"),
                right_path: String::from("/Users/piotrlosiniecki/Projects"),
                left_files: Vec::new(),
                right_files: Vec::new(),
            },
        })
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::debug!("resumed");

        self.reset_state();

        let app_window = AppWindow::new(event_loop);
        self.app_window = Some(app_window);

        // TODO: move reading file system to another thread

        let window = self.app_window.as_mut().unwrap();
        let imgui = window.imgui.as_mut().unwrap();
        let app_files = &mut imgui.app_files;

        app_files.left_files = read_directory(&app_files.left_path);
        app_files.right_files = read_directory(&app_files.right_path);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        log::debug!("window_event");

        match &event {
            WindowEvent::Resized(size) => {
                log::debug!("WindowEvent::Resized size: {:#?}", size);

                let app_window = self.app_window.as_mut().unwrap();
                let size = app_window.window.inner_size();

                log::debug!(
                    "size.width: {}, size.height: {}",
                    size.width,
                    size.height
                );

                app_window.surface_desc =
                    AppWindow::get_surface_desc(&app_window.window);

                app_window
                    .surface
                    .configure(&app_window.device, &app_window.surface_desc);

                app_window.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                log::debug!("WindowEvent::RedrawRequested");

                self.on_redraw_requested(event_loop);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                log::debug!("WindowEvent::KeyboardInput");

                let app_window = self.app_window.as_mut().unwrap();

                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyF) => {
                        if event.state.is_pressed() && !event.repeat {
                            log::debug!("Toggling fullscreen");

                            app_window.window.set_simple_fullscreen(
                                !app_window.window.simple_fullscreen(),
                            );
                        }
                    }
                    _ => {}
                }
                if let Key::Named(NamedKey::Escape) = event.logical_key {
                    if event.state.is_pressed() {
                        event_loop.exit();
                    }
                }

                let has_focus = app_window.window.has_focus();

                if has_focus {
                    app_window.window.request_redraw();
                }
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                log::debug!(
                    "WindowEvent::CursorMoved, device_id: {:#?}, position: {:#?}",
                    device_id,
                    position
                );

                let app_window = self.app_window.as_ref().unwrap();
                let has_focus = app_window.window.has_focus();

                log::debug!(
                    "WindowEvent::CursorMoved - has_focus: {has_focus}"
                );

                if has_focus {
                    app_window.window.request_redraw();
                }
            }
            WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase,
            } => {
                log::debug!(
                    "WindowEvent::MouseWheel, delta: {:#?}, phase: {:#?}",
                    delta,
                    phase
                );

                // TODO: scroll is too fast i probably need to throttle
                // based on refresh rate?

                let app_window = self.app_window.as_ref().unwrap();
                let has_focus = app_window.window.has_focus();

                if has_focus {
                    app_window.window.request_redraw();
                }
            }
            WindowEvent::CloseRequested => {
                log::debug!("WindowEvent::CloseRequested");

                event_loop.exit()
            }
            _ => (),
        }

        let app_window = self.app_window.as_mut().unwrap();
        let imgui = app_window.imgui.as_mut().unwrap();

        imgui.platform.handle_event::<()>(
            imgui.context.io_mut(),
            &app_window.window,
            &Event::WindowEvent { window_id, event },
        );
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        log::debug!("about_to_wait");

        let app_window = self.app_window.as_mut().unwrap();
        let imgui = app_window.imgui.as_mut().unwrap();

        let limit_fps = imgui.limit_fps;

        // TODO: use this mechanism limit fps in idle application
        if limit_fps {
            let now = Instant::now();
            let last_frame = imgui.last_frame;
            let delta_time = now - last_frame;
            let target_fps = 30.0;
            let target_fps_s = 1.0 / target_fps;
            let fps_100_duration = Duration::from_secs_f64(target_fps_s);

            log::debug!("delta_time: {} micro s", delta_time.as_micros());

            if delta_time > fps_100_duration {
                log::debug!("request_redraw");
                app_window.window.request_redraw();
            } else {
                let wait_for = fps_100_duration - delta_time;
                // TODO: do not wait if wait_for is small
                log::debug!("wait_for: {} mikro s", wait_for.as_micros());
                event_loop
                    .set_control_flow(ControlFlow::WaitUntil(now + wait_for));
            }
        } else {
            log::debug!("request_redraw");
            event_loop.set_control_flow(ControlFlow::Poll);
            app_window.window.request_redraw();
        }

        imgui.platform.handle_event::<()>(
            imgui.context.io_mut(),
            &app_window.window,
            &Event::AboutToWait,
        );
    }
}

impl App {
    fn on_redraw_requested(&mut self, _event_loop: &ActiveEventLoop) {
        let now = Instant::now();

        let app_window = self.app_window.as_mut().unwrap();
        let imgui = app_window.imgui.as_mut().unwrap();
        let imgui_ptr = imgui as *mut ImguiState;

        let delta_time = now - imgui.last_frame;
        imgui.context.io_mut().update_delta_time(delta_time);
        imgui.last_frame = now;

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
        let ui_ptr = ui as *mut Ui;

        let window = &app_window.window;
        let inner_size = window.inner_size();
        let scale = window.scale_factor();
        let width = ((inner_size.width as f64) / scale) as f32;
        let height = (inner_size.height as f64 / scale) as f32;

        let dt = now - imgui.last_frame_measure_time;

        if dt > Duration::from_secs(1) {
            // TODO: cleanup frame rate
            // let frame_rate2 = ui.io().framerate;
            let frame_rate = frame_count - imgui.last_measure_frame_count;

            imgui.frame_rate = frame_rate;
            imgui.last_frame_measure_time = now;
            imgui.last_measure_frame_count = frame_count;
        }

        let frame_rate = imgui.frame_rate;

        if imgui.demo_open {
            ui.show_demo_window(&mut imgui.demo_open);
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
                    if ui.is_key_pressed(imgui::Key::Tab) {
                        imgui.focused_window_left = !imgui.focused_window_left;
                    }

                    ui.text(format!("Frame rate: {frame_rate} FPS"));
                    ui.text(format!("Frame count: {frame_count}"));

                    let content_region_avail = ui.content_region_avail();
                    let half_screen = content_region_avail[0] / 2.0;
                    let main_window_h = content_region_avail[1];

                    let mut path_to_open_option: Option<PathBuf>;

                    unsafe {
                        path_to_open_option = render::render_files_window(
                            ui_ptr,
                            imgui_ptr,
                            half_screen,
                            main_window_h,
                            true,
                            &imgui.app_files.left_path,
                            &imgui.app_files.left_files,
                        );
                    }

                    if let Some(path_to_open) = path_to_open_option {
                        log::info!(
                            "left window path_to_open: {}",
                            path_to_open.display()
                        );

                        if files::is_dir(&path_to_open) {
                            let path_str =
                                path_to_open.as_path().to_str().unwrap();
                            let files = files::read_directory(path_str);

                            imgui.app_files.left_path = path_str.to_string();
                            imgui.app_files.left_files = files;

                            app_window.window.request_redraw();
                        }
                    }

                    ui.same_line();

                    unsafe {
                        path_to_open_option = render::render_files_window(
                            ui_ptr,
                            imgui_ptr,
                            0.0,
                            main_window_h,
                            false,
                            &imgui.app_files.right_path,
                            &imgui.app_files.right_files,
                        );
                    }

                    if let Some(path_to_open) = path_to_open_option {
                        log::info!(
                            "right window path_to_open: {}",
                            path_to_open.display()
                        );

                        if files::is_dir(&path_to_open) {
                            let path_str =
                                path_to_open.as_path().to_str().unwrap();
                            let files = files::read_directory(path_str);

                            imgui.app_files.right_path = path_str.to_string();
                            imgui.app_files.right_files = files;

                            app_window.window.request_redraw();
                        }
                    }
                });
        }

        let mut encoder: wgpu::CommandEncoder =
            app_window.device.create_command_encoder(
                &wgpu::CommandEncoderDescriptor { label: None },
            );

        if imgui.last_cursor != ui.mouse_cursor() {
            imgui.last_cursor = ui.mouse_cursor();
            imgui.platform.prepare_render(&ui, &app_window.window);
        }

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut rpass =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
}
