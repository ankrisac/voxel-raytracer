use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

type Size2D = winit::dpi::PhysicalSize<u32>;

mod label;

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("hello winit")
        .build(&event_loop)
        .expect("winit: unable to create window");

    event_loop.run(move |event, _, flow| match event {
        Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
            WindowEvent::Resized(new_inner_size) => {}
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {}
            WindowEvent::CloseRequested => *flow = ControlFlow::Exit,
            _ => {}
        },
        Event::RedrawRequested(_) => {}
        Event::RedrawEventsCleared => window.request_redraw(),
        _ => {}
    });
}
