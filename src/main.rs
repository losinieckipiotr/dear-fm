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
    dpi::{LogicalSize, PhysicalSize},
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
    last_frame: Instant,
    last_cursor: Option<MouseCursor>,
    last_frame_measure_time: Instant,
    last_measure_frame_count: i32,
    frame_rate: i32,
    left_item_selected_idx: i32,
    right_item_selected_idx: i32,
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

fn main() {
    // env_logger::init();
    env_logger::init_from_env(Env::new().default_filter_or(log::Level::Info.as_str()));

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut App::new()).unwrap();
}

impl AppWindow {
    fn new(event_loop: &ActiveEventLoop) -> Self {
        let mut window = Self::setup_gpu(event_loop);
        window.setup_imgui();

        window
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

        let size = window.inner_size();
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

        let surface_desc = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![wgpu::TextureFormat::Bgra8Unorm],
        };

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

        let font_size = (13.0 * self.hidpi_factor) as f32;
        context.io_mut().font_global_scale = (1.0 / self.hidpi_factor) as f32;

        context.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

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
            last_frame,
            last_cursor: None,
            last_frame_measure_time: last_frame,
            last_measure_frame_count: 0,
            frame_rate: 0,
            left_item_selected_idx: 0,
            right_item_selected_idx: 0,
        })
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::debug!("resumed");

        self.reset_state();
        self.window = Some(AppWindow::new(event_loop));

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
                log::debug!("WindowEvent::Resized");

                self.on_window_resized(size);
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

                        if event.state.is_pressed() {
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

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        log::debug!("about_to_wait");

        let window = self.window.as_mut().unwrap();
        let imgui = window.imgui.as_mut().unwrap();

        window.window.request_redraw();

        imgui.platform.handle_event::<()>(
            imgui.context.io_mut(),
            &window.window,
            &Event::AboutToWait,
        );
    }

    // fn device_event(
    //     &mut self,
    //     _event_loop: &ActiveEventLoop,
    //     device_id: winit::event::DeviceId,
    //     event: winit::event::DeviceEvent,
    // ) {
    //     let window = self.window.as_mut().unwrap();
    //     let imgui = window.imgui.as_mut().unwrap();

    //     match event {
    //         winit::event::DeviceEvent::MouseWheel { delta } => {
    //             log::info!("scroll event, {:#?}", delta);

    //             // event_loop.listen_device_events();
    //         }

    //         _ => (),
    //     }

    //     imgui.platform.handle_event::<()>(
    //         imgui.context.io_mut(),
    //         &window.window,
    //         &Event::DeviceEvent { device_id, event },
    //     );
    // }
}

impl App {
    fn on_window_resized(&mut self, size: &PhysicalSize<u32>) {
        log::debug!("on_window_resized");

        let window = self.window.as_mut().unwrap();

        window.surface_desc = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![wgpu::TextureFormat::Bgra8Unorm],
        };

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

        imgui
            .context
            .io_mut()
            .update_delta_time(now - imgui.last_frame);
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
            let frame_rate = frame_count - imgui.last_measure_frame_count;

            imgui.frame_rate = frame_rate;
            imgui.last_frame_measure_time = now;
            imgui.last_measure_frame_count = frame_count;
        }

        ui.window("Main window")
            .size([width, height], Condition::Always)
            .position([0.0, 0.0], Condition::Always)
            .focus_on_appearing(true)
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

                unsafe {
                    render::render_left(
                        ui_ptr,
                        imgui_ptr,
                        half_screen,
                        main_window_h,
                        &self.left_files,
                    );
                }

                ui.same_line();

                unsafe {
                    render::render_right(ui_ptr, imgui_ptr, main_window_h, &self.right_files);
                }
            });

        if imgui.demo_open {
            ui.show_demo_window(&mut imgui.demo_open);
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
