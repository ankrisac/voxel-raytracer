use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

type Size2D = winit::dpi::PhysicalSize<u32>;

mod label;
mod canvas;
mod pipeline;

use crate::canvas::Canvas;
use crate::label::Label;
use crate::pipeline::Pipeline;

struct Engine {
    canvas: Canvas,
    pipeline: Pipeline,
}
impl Engine {
    async fn new(label: &Label, window: &Window) -> Self {
        let canvas = Canvas::new(label, window).await;
        let pipeline = Pipeline::new(&canvas);

        Self {
            canvas,
            pipeline,
        }
    }

    fn resize(&mut self, size: Size2D) {
        self.canvas.resize(size);
        self.pipeline.resize(&self.canvas, size);
    }

    fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.canvas.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.pipeline.render(&self.canvas, &view);

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
