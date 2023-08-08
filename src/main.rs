mod canvas;
mod map;
//mod player;
mod raycaster;
mod state;
mod textures;
mod window;

use std::time::{Duration, Instant};

use pollster::block_on;
use state::State;
use window::Window;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder as WinitWindowBuilder,
};
// TODO add portals like in Portal game
fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error");
    }
    env_logger::init();

    let event_loop = EventLoop::new();
    let winit_window = WinitWindowBuilder::new().build(&event_loop).unwrap();
    winit_window.set_title("False Space");

    let window = block_on(Window::init(&winit_window)).unwrap();

    let mut state = State::new(window, 960, 540);

    let framerate_delta = Duration::from_secs_f64(1.0 / 30.0);
    let mut time_delta = Instant::now();

    event_loop.run(move |event, _, control| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                    *control = ControlFlow::Exit
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => {
                    state.process_input(input)
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.resize(*new_inner_size);
                }
                WindowEvent::Resized(new_size) => state.resize(new_size),
                _ => (),
            },
            Event::RedrawRequested(..) => {
                state.update();

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
            Event::MainEventsCleared => {
                // Regulate to 60 FPS
                let elapsed = time_delta.elapsed();

                if framerate_delta <= elapsed {
                    winit_window.request_redraw();
                    time_delta = Instant::now();
                } else {
                    *control = ControlFlow::WaitUntil(
                        Instant::now() + framerate_delta - elapsed,
                    );
                }
            }
            Event::LoopDestroyed => println!("Exited!"),
            _ => (),
        }
    });
}
