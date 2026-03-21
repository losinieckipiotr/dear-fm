use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use pollster::block_on;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
    platform::macos::WindowExtMacOS,
    window::Window,
};

use env_logger::Env;

mod files;
mod render;

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
    left_item_selected_idx: i32,
    right_item_selected_idx: i32,
    focused_window_left: bool,
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
    window: Option<AppWindow>,
    left_files: Vec<String>,
    right_files: Vec<String>,
}

impl App {
    fn new() -> Self {
        Self {
            window: Option::None,
            left_files: Vec::new(),
            right_files: Vec::new(),
        }
    }

    fn reset_state(&mut self) {
        let new_state = App::new();

        self.window = new_state.window;
    }
}

const LEFT_PATH: &str = "/";
const RIGHT_PATH: &str = "/Users/piotrlosiniecki";
// crashes with bigger value?

fn main() {
    // env_logger::init();
    env_logger::init_from_env(Env::new().default_filter_or(log::Level::Info.as_str()));

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

        let monitor = event_loop.primary_monitor().unwrap();
        let size = monitor.size();

        let window = {
            let version = env!("CARGO_PKG_VERSION");

            let attributes = Window::default_attributes()
                .with_inner_size(LogicalSize::new(size.width, size.height))
                .with_title(format!("Dear File Manager {version}"));
            Arc::new(event_loop.create_window(attributes).unwrap())
        };

        let hidpi_factor = window.scale_factor();
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) =
            block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();

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

        platform.attach_window(context.io_mut(), &self.window, HiDpiMode::Default);
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

        let renderer = Renderer::new(&mut context, &self.device, &self.queue, renderer_config);

        let last_frame = Instant::now();

        self.imgui = Some(ImguiState {
            context,
            platform,
            renderer,
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
            demo_open: false,
            limit_fps: false,
            last_frame,
            last_cursor: None,
            last_frame_measure_time: last_frame,
            last_measure_frame_count: 0,
            frame_rate: 0,
            left_item_selected_idx: 0,
            right_item_selected_idx: 0,
            focused_window_left: true,
        })
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::debug!("resumed");

        self.reset_state();

        let app_window = AppWindow::new(event_loop);
        self.window = Some(app_window);

        match files::read_directory(LEFT_PATH) {
            Ok(files) => self.left_files = files,
            Err(error) => {
                log::error!("error during left directory read: {:#?}", error);
                self.left_files = vec![];
            }
        }

        match files::read_directory(RIGHT_PATH) {
            Ok(files) => self.right_files = files,
            Err(error) => {
                log::error!("error during right directory read: {:#?}", error);
                self.right_files = vec![];
            }
        }
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

                self.on_window_resized();
            }
            WindowEvent::RedrawRequested => {
                log::debug!("WindowEvent::RedrawRequested");

                self.on_redraw_requested(event_loop);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                log::debug!("WindowEvent::KeyboardInput");

                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyF) => {
                        let window = self.window.as_mut().unwrap();

                        if event.state.is_pressed() && !event.repeat {
                            log::debug!("Toggling fullscreen");

                            window
                                .window
                                .set_simple_fullscreen(!window.window.simple_fullscreen());
                        }
                    }
                    _ => {}
                }
                if let Key::Named(NamedKey::Escape) = event.logical_key {
                    if event.state.is_pressed() {
                        event_loop.exit();
                    }
                }
            }
            // WindowEvent::MouseWheel {
            //     device_id: _,
            //     delta,
            //     phase,
            // } => {
            //     // log::info!("delta: {:#?}", delta);
            //     // log::info!("phase: {:#?}", phase);

            //     // TODO: scroll is too fast i probably need to throttle
            //     // based on refresh rate?
            // }
            WindowEvent::CloseRequested => {
                log::debug!("WindowEvent::CloseRequested");

                event_loop.exit()
            }
            _ => (),
        }

        let window = self.window.as_mut().unwrap();
        let imgui = window.imgui.as_mut().unwrap();

        imgui.platform.handle_event::<()>(
            imgui.context.io_mut(),
            &window.window,
            &Event::WindowEvent { window_id, event },
        );
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        log::debug!("about_to_wait");

        let window = self.window.as_mut().unwrap();
        let imgui = window.imgui.as_mut().unwrap();

        let limit_fps = imgui.limit_fps;

        // TODO: use this mechanism limit fps in idle application
        if limit_fps {
            let now = Instant::now();
            let last_frame = imgui.last_frame;
            let delta_time = now - last_frame;
            let target_fps = 60.0;
            let target_fps_s = 1.0 / target_fps;
            let fps_100_duration = Duration::from_secs_f64(target_fps_s);

            log::debug!("delta_time: {} micro s", delta_time.as_micros());

            if delta_time > fps_100_duration {
                log::debug!("request_redraw");
                window.window.request_redraw();
            } else {
                let wait_for = fps_100_duration - delta_time;
                // TODO: do not wait if wait_for is small
                log::debug!("wait_for: {} mikro s", wait_for.as_micros());
                event_loop.set_control_flow(ControlFlow::WaitUntil(now + wait_for));
            }
        } else {
            log::debug!("request_redraw");
            event_loop.set_control_flow(ControlFlow::Poll);
            window.window.request_redraw();
        }

        imgui.platform.handle_event::<()>(
            imgui.context.io_mut(),
            &window.window,
            &Event::AboutToWait,
        );
    }
}

impl App {
    fn on_window_resized(&mut self) {
        log::debug!("on_window_resized");

        let window = self.window.as_mut().unwrap();
        let size = window.window.inner_size();

        log::debug!("size.width: {}, size.height: {}", size.width, size.height);

        window.surface_desc = AppWindow::get_surface_desc(&window.window);

        window
            .surface
            .configure(&window.device, &window.surface_desc);
    }

    fn on_redraw_requested(&mut self, _event_loop: &ActiveEventLoop) {
        log::debug!("on_redraw_requested");

        let now = Instant::now();

        let window = self.window.as_mut().unwrap();
        let imgui = window.imgui.as_mut().unwrap();
        let imgui_ptr = imgui as *mut ImguiState;

        let delta_time = now - imgui.last_frame;
        imgui.context.io_mut().update_delta_time(delta_time);
        imgui.last_frame = now;

        let frame = match window.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => {
                log::warn!("dropped frame: {e:?}");
                return;
            }
        };
        let frame_count = imgui.context.frame_count();

        imgui
            .platform
            .prepare_frame(imgui.context.io_mut(), &window.window)
            .expect("Failed to prepare frame");
        let ui = imgui.context.frame();
        let ui_ptr = ui as *mut Ui;

        let app_window = &window.window;
        let inner_size = app_window.inner_size();
        let scale = app_window.scale_factor();
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

        if imgui.demo_open {
            ui.show_demo_window(&mut imgui.demo_open);
        } else {
            ui.window("Main window")
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
                    let content_region_avail = ui.content_region_avail();

                    let half_screen = content_region_avail[0] / 2.0;
                    let main_window_h = content_region_avail[1];

                    if ui.is_key_pressed(imgui::Key::Tab) {
                        imgui.focused_window_left = !imgui.focused_window_left;
                    }

                    unsafe {
                        render::render_left(
                            ui_ptr,
                            imgui_ptr,
                            half_screen,
                            main_window_h,
                            LEFT_PATH,
                            &self.left_files,
                        );
                    }

                    ui.same_line();

                    unsafe {
                        render::render_right(ui_ptr, imgui_ptr, main_window_h, &self.right_files);
                    }
                });
        }

        let mut encoder: wgpu::CommandEncoder = window
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        if imgui.last_cursor != ui.mouse_cursor() {
            imgui.last_cursor = ui.mouse_cursor();
            imgui.platform.prepare_render(&ui, &window.window);
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
                &window.queue,
                &window.device,
                &mut rpass,
            )
            .expect("Rendering failed");

        drop(rpass);

        window.queue.submit(Some(encoder.finish()));

        frame.present();
    }
}
