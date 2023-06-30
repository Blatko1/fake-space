mod state;
mod utils;

use futures::executor::block_on;
use state::State;
use utils::Window;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder as WinitWindowBuilder,
};

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error");
    }
    env_logger::init();

    let event_loop = EventLoop::new();
    let winit_window = WinitWindowBuilder::new().build(&event_loop).unwrap();
    winit_window.set_title("False Space");

    let window = block_on(Window::init(winit_window)).unwrap();

    let mut state = State::new(window);

    event_loop.run(move |event, _, control| {
        match event {
            Event::WindowEvent { window_id, event } => match event {
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
            Event::DeviceEvent { device_id, event } => todo!(),
            Event::MainEventsCleared => {
                if state.should_exit() {
                    *control = ControlFlow::Exit; 
                }
            },
            Event::RedrawRequested(_) => todo!(),
            Event::RedrawEventsCleared => todo!(),
            Event::LoopDestroyed => println!("Exited!"),
            _ => (),
        }
    });
}
