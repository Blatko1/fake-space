/// For the record:
/// I have tried adding FXAA in the fragment shader, which ended up in a weird
/// output, have tried MSAA but it doesn't work on textures, have tried applying
/// bilinear texture filtering but unnoticeable.
mod canvas;
mod draw;
mod map;
mod state;
mod textures;
mod voxel;

use std::time::{Duration, Instant};

use canvas::Canvas;
use map::Map;
use pollster::block_on;
use state::State;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder as WinitWindowBuilder,
};

const FPS: u32 = 60;

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error");
    }
    env_logger::init();

    let event_loop = EventLoop::new();
    let winit_window = WinitWindowBuilder::new().build(&event_loop).unwrap();
    winit_window.set_title("False Space");

    let (map, textures) =
        Map::from_file_str(include_str!("../maps/map1.txt")).unwrap();

    let canvas = block_on(Canvas::init(&winit_window, 240, 180));

    let mut state = State::new(canvas, map, textures);

    let framerate_delta = Duration::from_secs_f64(1.0 / FPS as f64);
    let mut time_delta = Instant::now();
    let mut fps_update_delta = Instant::now();
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
                    state.process_keyboard_input(input)
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.resize(*new_inner_size);
                }
                WindowEvent::Resized(new_size) => state.resize(new_size),
                _ => (),
            },
            Event::DeviceEvent { event, .. } => {
                state.process_mouse_input(event)
            }
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
                    if fps_update_delta.elapsed().as_millis() >= 1000 {
                        println!(
                            "avg frame time: {} ms, FPS: {}",
                            (fps_avg / framerate as u128) as f32 / 1000.0,
                            framerate
                        );
                        fps_update_delta = Instant::now();
                        framerate = 0;
                        fps_avg = 0;
                    }
                } else {
                    *control = ControlFlow::WaitUntil(
                        Instant::now() + framerate_delta - elapsed,
                    );
                }
                //if fps_update_delta.elapsed().as_millis() >= 1000 {
                //    println!("avg frame time: {}, FPS: {}", fps_avg as f64 / framerate as f64, framerate);
                //    fps_update_delta = Instant::now();
                //    framerate = 0;
                //    fps_avg = 0;
                //}
                //framerate += 1;
                //fps_avg += time_delta.elapsed().as_micros();
                //time_delta = Instant::now();
            }
            Event::LoopDestroyed => println!("Exited!"),
            _ => (),
        }
    });
}
