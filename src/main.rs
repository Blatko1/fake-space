/// For the record:
/// I have tried adding FXAA in the fragment shader, which ended up in a weird
/// output, have tried MSAA but it doesn't work on textures, have tried applying
/// bilinear texture filtering but unnoticeable.
pub mod backend;
mod player;
mod render;
mod state;
mod voxel;
mod world;
mod dbg;

use std::fs;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::world::World;
use backend::Canvas;
use pollster::block_on;
use wgpu_text::glyph_brush::ab_glyph::FontVec;
use state::State;
use winit::{
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder as WinitWindowBuilder,
};
use winit::event::KeyEvent;
use winit::keyboard::{Key, NamedKey};
use crate::dbg::Dbg;

const FPS: u32 = 60;

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error");
    }
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let winit_window = Arc::new(WinitWindowBuilder::new().build(&event_loop).unwrap());
    winit_window.set_title("False Space");

    let world = World::from_path("maps/world.txt").unwrap();

    let mut canvas = block_on(Canvas::init(winit_window.clone(), 240 * 2, 135 * 2));
    // TODO change/fix this
    let font_data = fs::read("res/DejaVuSans.ttf").unwrap();
    let font = FontVec::try_from_vec(font_data).unwrap();
    let mut dbg = Dbg::new(canvas.gfx(), font);

    let mut state = State::new(&canvas, world);

    let framerate_delta = Duration::from_secs_f64(1.0 / FPS as f64);
    let mut time_delta = Instant::now();
    let mut fps_update_delta = Instant::now();
    let mut framerate = 0;
    let mut fps_avg = 0;
    let mut last_fps = 0;
    let mut last_fps_avg_time_ms = 0.0;

    event_loop.run(move |event, elwt| {
        match event {
            Event::NewEvents(_) => {
                let elapsed = time_delta.elapsed();

                if framerate_delta <= elapsed {
                    winit_window.request_redraw();
                    time_delta = Instant::now();
                    framerate += 1;
                    fps_avg += elapsed.as_micros();
                    if fps_update_delta.elapsed().as_millis() >= 1000 {
                        fps_update_delta = Instant::now();
                        last_fps = framerate;
                        last_fps_avg_time_ms = (fps_avg / framerate as u128) as f32 / 1000.0;
                        framerate = 0;
                        fps_avg = 0;
                    }
                } else {
                    elwt.set_control_flow(ControlFlow::WaitUntil(
                        Instant::now() + framerate_delta - elapsed,
                    ));
                }
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                    elwt.exit();
                }
                WindowEvent::KeyboardInput {
                    event: KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        ..
                    },
                    ..
                } => elwt.exit(),
                WindowEvent::KeyboardInput { event, .. } => {
                    state.process_keyboard_input(event)
                }
                WindowEvent::Resized(new_size) => {
                    canvas.resize(new_size);
                    dbg.resize(&canvas);
                },
                WindowEvent::RedrawRequested => {
                    state.update();
                    dbg.update(&state, last_fps_avg_time_ms, last_fps);

                    // TODO check result instead of unwrap
                    dbg.queue_data(canvas.gfx()).unwrap();
                    canvas.clear_buffer();
                    state.draw(canvas.mut_column_iterator());

                    match canvas.render(&dbg) {
                        Ok(_) => (),
                        Err(wgpu::SurfaceError::Lost) => canvas.on_surface_lost(),
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => {println!("Out of memory!"); elwt.exit()},
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                _ => (),
            },
            Event::DeviceEvent { event, ..} => state.process_mouse_input(event),
            Event::LoopExiting => println!("Exited!"),
            _ => (),
        }
    }).unwrap();
}
