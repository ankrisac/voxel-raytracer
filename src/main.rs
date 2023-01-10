use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

type Size2D = winit::dpi::PhysicalSize<u32>;

mod label;

/// A combined surface and rendering context which
/// manages all the wgpu handles requires for rendering
mod canvas;
use crate::canvas::Canvas;
use crate::label::Label;

struct Engine {
    label: Label,
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
                    targets: &[Some(wgpu::ColorTargetState {
                        format: canvas.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        Self {
            label: label.to_owned(),
            canvas,
            pipeline,
        }
    }

    fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.canvas.surface.get_current_texture()?;

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .canvas
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(self.label.sublabel("render-encoder").as_str()),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(self.label.sublabel("render-pass").as_str()),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            pass.set_pipeline(&self.pipeline);
            pass.draw(0..3, 0..1);
        }
        
        self.canvas.queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        Ok(())
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("hello winit")
        .build(&event_loop)
        .expect("winit: unable to create window");

    let mut engine = pollster::block_on(Engine::new(&Label::from("Canvas"), &window));

    event_loop.run(move |event, _, flow| match event {
        Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
            WindowEvent::Resized(new_inner_size) => engine.canvas.resize(new_inner_size),
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                engine.canvas.resize(*new_inner_size)
            }
            WindowEvent::CloseRequested => *flow = ControlFlow::Exit,
            _ => {}
        },
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            if let Err(err) = engine.render() {
                match err {
                    wgpu::SurfaceError::Lost => {
                        engine.canvas.resize(engine.canvas.get_size());
                    }
                    wgpu::SurfaceError::OutOfMemory => {
                        eprintln!("wgpu: Out of memory");
                        *flow = ControlFlow::Exit;
                    }
                    other => eprintln!("RenderError: {other}"),
                }
            }
        }
        Event::RedrawEventsCleared => window.request_redraw(),
        _ => {}
    });
}
