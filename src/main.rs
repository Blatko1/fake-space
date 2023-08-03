mod canvas;
mod state;
mod window;

use pollster::block_on;
use state::State;
use window::Window;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder as WinitWindowBuilder,
};

const WIDTH: u32 = 80;
const HEIGHT: u32 = 60;

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error");
    }
    env_logger::init();

    let event_loop = EventLoop::new();
    let winit_window = WinitWindowBuilder::new().build(&event_loop).unwrap();
    winit_window.set_title("False Space");

    let window = block_on(Window::init(winit_window)).unwrap();

    let mut state = State::new(window, WIDTH, HEIGHT);

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
                    state.resize(*new_inner_size);
                }
                WindowEvent::Resized(new_size) => state.resize(new_size),
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
