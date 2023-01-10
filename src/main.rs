use std::{borrow::Cow, path::Path};

use label::Label;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

type Size2D = winit::dpi::PhysicalSize<u32>;

mod label;

/// A combined surface and rendering context which
/// manages all the wgpu handles requires for rendering
struct Canvas {
    label: Label,
    instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,

    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
}
impl Canvas {
    async fn new(label: &Label, window: &Window) -> Canvas {
        let backends = wgpu::Backends::all();
        let instance = wgpu::Instance::new(backends);

        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Context: failed to find appropriate GPU adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some(label.sublabel("device").as_str()),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_webgl2_defaults(),
                },
                None,
            )
            .await
            .expect("Context: failed to find appropriate GPU device");

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };

        Self {
            label: label.to_owned(),
            instance,
            device,
            queue,
            surface,
            config,
        }
    }

    fn load_shader<P: AsRef<Path>>(&self, path: P) -> wgpu::ShaderModule {
        let source = std::fs::read_to_string(path.as_ref())
            .expect(format!("Canvas: unable to load shader at [{:?}]", path.as_ref()).as_str());

        let label = path.as_ref().to_string_lossy();

        self.device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(self.label.sublabel(label).as_str()),
                source: wgpu::ShaderSource::Wgsl(Cow::from(source)),
            })
    }

    fn get_size(&self) -> Size2D {
        Size2D::new(self.config.width, self.config.height)
    }

    fn resize(&mut self, size: Size2D) {
        if size.width > 0 && size.height > 0 {
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config)
        }
    }
}


struct Engine {
    canvas: Canvas,
    pipeline: wgpu::RenderPipeline,
}
impl Engine {
    async fn new(label: &Label, window: &Window) -> Self {
        let canvas = Canvas::new(label, window).await;

        let layout = canvas
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label.sublabel("pipeline").sublabel("layout").as_str()),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let module = canvas.load_shader("shaders/render.wgsl");

        let pipeline = canvas
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label.sublabel("pipeline").as_str()),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: "fs_main",
                    targets: &[
                        Some(wgpu::ColorTargetState {
                            format: canvas.config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })
                    ],
                }),
                multiview: None,
            });

        Self { canvas, pipeline }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("hello winit")
        .build(&event_loop)
        .expect("winit: unable to create window");

    let mut canvas = pollster::block_on(Canvas::new(&Label::from("Canvas"), &window));

    event_loop.run(move |event, _, flow| match event {
        Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
            WindowEvent::Resized(new_inner_size) => canvas.resize(new_inner_size),
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                canvas.resize(*new_inner_size)
            }
            WindowEvent::CloseRequested => *flow = ControlFlow::Exit,
            _ => {}
        },
        Event::RedrawRequested(window_id) if window_id == window.id() => {}
        Event::RedrawEventsCleared => window.request_redraw(),
        _ => {}
    });
}
