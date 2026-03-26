use crate::state::*;
use env_logger::Env;
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

mod files;
mod render;
mod state;

pub struct ImguiState {
    context: imgui::Context,
    platform: WinitPlatform,
    renderer: Renderer,
    clear_color: wgpu::Color,
}

struct AppWindow {
    device: wgpu::Device,
    queue: wgpu::Queue,
    window: Arc<Window>,
    surface_desc: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'static>,
    hidpi_factor: f64,
    imgui: Option<ImguiState>,
    state: AppState,
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

        Self {
            device,
            queue,
            window,
            surface_desc,
            surface,
            hidpi_factor,
            imgui: None,
            state: AppState::new(),
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
        })
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::debug!("resumed");

        let mut app_window = AppWindow::new(event_loop);
        let state = &mut app_window.state;

        state.go_to_directory(
            Side::Left,
            PathBuf::from("/Users/piotrlosiniecki"),
        );
        state.go_to_directory(
            Side::Right,
            PathBuf::from("/Users/piotrlosiniecki/Projects"),
        );

        let imgui = app_window.imgui.as_mut().unwrap();
        imgui.platform.handle_event::<()>(
            imgui.context.io_mut(),
            &app_window.window,
            &Event::Resumed,
        );

        self.app_window = Some(app_window);
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
                let inner_size = app_window.window.inner_size();

                log::debug!(
                    "inner_size.width: {}, inner_size.height: {}",
                    inner_size.width,
                    inner_size.height
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

                let app_window = self.app_window.as_mut().unwrap();

                render::render_frame(app_window);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                log::debug!("WindowEvent::KeyboardInput");

                let app_window = self.app_window.as_ref().unwrap();

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
        let state = &app_window.state;

        // TODO: use this mechanism limit fps in idle application
        if state.limit_fps {
            let now = Instant::now();
            let last_frame = state.last_frame;
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
