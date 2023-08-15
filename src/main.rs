/// For the record:
/// I have tried adding FXAA in the fragment shader, which ended up in a weird
/// output, have tried MSAA but it doesn't work on textures, have tried applying
/// bilinear texture filtering but unnoticeable.
mod canvas;
mod map;
mod render;
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

const FPS: u32 = 60;

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error");
    }
    env_logger::init();

    let event_loop = EventLoop::new();
    let winit_window = WinitWindowBuilder::new().build(&event_loop).unwrap();
    winit_window.set_title("False Space");

    let window = block_on(Window::init(&winit_window)).unwrap();

    let mut state = State::new(window, 640, 480);

    let framerate_delta = Duration::from_secs_f64(1.0 / FPS as f64);
    let mut time_delta = Instant::now();
    let mut framerate = 0;
    let mut fps_avg = 0;

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
                    Err(wgpu::SurfaceError::Lost) => state.on_surface_lost(),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        *control = ControlFlow::Exit
                    }
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                let elapsed = time_delta.elapsed();

                if framerate_delta <= elapsed {
                    winit_window.request_redraw();
                    time_delta = Instant::now();
                    framerate += 1;
                    fps_avg += elapsed.as_micros();
                    // println!() every second instead.
                    if framerate >= FPS {
                        println!("avg frame time: {}", fps_avg / 60);
                        framerate = 0;
                        fps_avg = 0;
                    }
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
