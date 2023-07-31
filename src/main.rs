mod state;
mod window;

use pollster::block_on;
use state::{State, WIDTH, HEIGHT};
use window::Window;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder as WinitWindowBuilder, dpi::PhysicalSize,
};

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error");
    }
    env_logger::init();

    let event_loop = EventLoop::new();
    let winit_window = WinitWindowBuilder::new().build(&event_loop).unwrap();
    winit_window.set_title("False Space");
    winit_window.set_inner_size(PhysicalSize::new(WIDTH as u32, HEIGHT as u32));

    let window = block_on(Window::init(winit_window)).unwrap();

    let mut state = State::new(window);

    event_loop.run(move |event, _, control| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                    *control = ControlFlow::Exit
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    state.process_keyboard_input(input)
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.window().resize(*new_inner_size);
                }
                WindowEvent::Resized(new_size) => {
                    state.window().resize(new_size)
                }
                _ => (),
            },
            Event::DeviceEvent { device_id, event } => (),
            Event::MainEventsCleared => {
                if state.should_exit() {
                    *control = ControlFlow::Exit;
                }
                match state.render() {
                    Ok(_) => (),
                    Err(wgpu::SurfaceError::Lost) => {
                        state.window().recreate_sc()
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        *control = ControlFlow::Exit
                    }
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::RedrawRequested(_) => (),
            Event::LoopDestroyed => println!("Exited!"),
            _ => (),
        }
    });
}
