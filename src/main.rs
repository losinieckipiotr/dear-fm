use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use pollster::block_on;
use std::{sync::Arc, time::Instant};
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

struct ImguiState {
    context: imgui::Context,
    platform: WinitPlatform,
    renderer: Renderer,
    clear_color: wgpu::Color,
    demo_open: bool,
    last_frame: Instant,
    last_cursor: Option<MouseCursor>,
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

#[derive(Default)]
struct App {
    window: Option<AppWindow>,
    frame_count: u64,
    last_frame_rate: f32,
}

// impl App {
//     fn new(width: f64, height: f64) -> Self {
//         Self {
//             width,
//             height,
//             window: None,
//         }
//     }
// }

fn main() {
    // env_logger::init();

    env_logger::init_from_env(Env::new().default_filter_or(log::Level::Info.as_str()));

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    //    if let handle = event_loop.owned_display_handle().display_handle().unwrap() {
    //         let size = winit::monitor::MonitorHandle::size(handle);

    //         event_loop.run_app(&mut App::new(size.width, size.height)).unwrap();
    //    }

    event_loop.run_app(&mut App::default()).unwrap();
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(AppWindow::new(event_loop));
        self.frame_count = 0;
        self.last_frame_rate = 0.0;
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let window = self.window.as_mut().unwrap();
        let imgui = window.imgui.as_mut().unwrap();

        match &event {
            WindowEvent::Resized(size) => {
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
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyF) => {
                        if event.state.is_pressed() {
                            // let monitor = window.window.current_monitor().unwrap();
                            // let size = monitor.size();
                            // debug!("WIDTH: {}", size.width);
                            // debug!("HEIGHT: {}", size.height);

                            log::info!("Toggling fullscreen");

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

            WindowEvent::RedrawRequested => {
                self.frame_count += 1;

                let delta_s = imgui.last_frame.elapsed();
                let now = Instant::now();
                imgui
                    .context
                    .io_mut()
                    .update_delta_time(now - imgui.last_frame);
                imgui.last_frame = now;

                let frame = match window.surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(e) => {
                        eprintln!("dropped frame: {e:?}");
                        return;
                    }
                };
                imgui
                    .platform
                    .prepare_frame(imgui.context.io_mut(), &window.window)
                    .expect("Failed to prepare frame");
                let ui = imgui.context.frame();

                let app_window = &window.window;

                {
                    // let ava_size = ui.content_region_avail();
                    // ui.io().config_flags

                    let inner_size = app_window.inner_size();
                    // log::info!("{:?}", inner_size);

                    let scale = app_window.scale_factor();

                    // log::info!("{:?}", scale);

                    let width = ((inner_size.width as f64) / scale) as f32;
                    let height = (inner_size.height as f64 / scale) as f32;

                    // log::info!("{:?}", width);

                    let window = ui.window("Hello too");
                    window
                        .size([width, height], Condition::Always)
                        .position([0.0, 0.0], Condition::Always)
                        .focus_on_appearing(true)
                        .always_vertical_scrollbar(true)
                        .collapsible(false)
                        .resizable(false)
                        .movable(false)
                        .build(|| {
                            // ui.text(format!("Frametime: {delta_s:?}"));

                            let refresh_rate: u64 = 60;

                            // TODO: save time to avrage FPS
                            if (self.frame_count % refresh_rate) == 0 {
                                self.last_frame_rate = 1.0 / delta_s.as_secs_f32();
                            }

                            let last_frame_rate: u32 = self.last_frame_rate.round() as u32;

                            ui.text(format!("Frame rate: {last_frame_rate} FPS"));

                            let window_child_1 = ui.child_window("Left");

                            // let [w, _] = ui.content_region_avail();

                            window_child_1
                                .size([width / 2.0, height])
                                .border(true)
                                // .flags(WindowFlags::NO_COLLAPSE | WindowFlags::NO_DECORATION)
                                .build(|| {
                                    ui.text("left child");
                                });

                            ui.same_line();

                            let window_child_2 = ui.child_window("Right");
                            window_child_2
                                .size([width / 2., height])
                                .border(true)
                                .build(|| {
                                    ui.text("right child");
                                });
                        });
                }

                ui.show_demo_window(&mut imgui.demo_open);

                let mut encoder: wgpu::CommandEncoder = window
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                if imgui.last_cursor != ui.mouse_cursor() {
                    imgui.last_cursor = ui.mouse_cursor();
                    imgui.platform.prepare_render(ui, &window.window);
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
            _ => (),
        }

        imgui.platform.handle_event::<()>(
            imgui.context.io_mut(),
            &window.window,
            &Event::WindowEvent { window_id, event },
        );
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: ()) {
        let window = self.window.as_mut().unwrap();
        let imgui = window.imgui.as_mut().unwrap();
        imgui.platform.handle_event::<()>(
            imgui.context.io_mut(),
            &window.window,
            &Event::UserEvent(event),
        );
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let window = self.window.as_mut().unwrap();
        let imgui = window.imgui.as_mut().unwrap();

        imgui.platform.handle_event::<()>(
            imgui.context.io_mut(),
            &window.window,
            &Event::DeviceEvent { device_id, event },
        );
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let window = self.window.as_mut().unwrap();
        let imgui = window.imgui.as_mut().unwrap();
        window.window.request_redraw();
        imgui.platform.handle_event::<()>(
            imgui.context.io_mut(),
            &window.window,
            &Event::AboutToWait,
        );
    }
}

impl AppWindow {
    fn new(event_loop: &ActiveEventLoop) -> Self {
        let mut window = Self::setup_gpu(event_loop);
        window.setup_imgui();

        window
    }

    // TODO: refactor for better readability
    // initialization funcions should be at the top of the file
    fn setup_gpu(event_loop: &ActiveEventLoop) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let size = event_loop.primary_monitor().unwrap().size();

        let window = {
            let version = env!("CARGO_PKG_VERSION");

            let attributes = Window::default_attributes()
                .with_inner_size(LogicalSize::new(size.width, size.height))
                .with_title(format!("KURWAAAAA {version}"));
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

        // Set up swap chain
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

        // winit::monitor::MonitorHandle::size(&self)

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

        //
        // Set up dear imgui wgpu renderer
        //
        let clear_color = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };

        let renderer_config = RendererConfig {
            texture_format: self.surface_desc.format,
            ..Default::default()
        };

        let renderer = Renderer::new(&mut context, &self.device, &self.queue, renderer_config);
        let last_frame = Instant::now();
        let last_cursor = None;
        let demo_open = true;

        self.imgui = Some(ImguiState {
            context,
            platform,
            renderer,
            clear_color,
            demo_open,
            last_frame,
            last_cursor,
        })
    }
}
