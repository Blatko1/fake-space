/// For the record:
/// I have tried adding FXAA in the fragment shader, which ended up in a weird
/// output, have tried MSAA, but it doesn't work on textures, have tried applying
/// bilinear texture filtering but unnoticeable.
pub mod backend;
mod dbg;
mod player;
mod state;
mod voxel;
mod world;
mod control;

use std::fs;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::dbg::Dbg;
use crate::world::World;
use backend::Canvas;
use glam::Vec2;
use pollster::block_on;
use state::State;
use wgpu_text::glyph_brush::ab_glyph::FontVec;
use winit::event::{DeviceEvent, KeyEvent};
use winit::keyboard::{Key, NamedKey};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder as WinitWindowBuilder,
};

const FPS_CAP: u32 = 60;

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error");
    }
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let winit_window = Arc::new(WinitWindowBuilder::new().build(&event_loop).unwrap());
    winit_window.set_title("False Space");

    let world = World::from_path("maps/world.txt").unwrap();

    let mut canvas = block_on(Canvas::init(winit_window.clone(), 240 * 1, 135 * 1));
    // TODO change/fix this
    let font_data = fs::read("res/Minecraft.ttf").unwrap();
    let font = FontVec::try_from_vec(font_data).unwrap();
    let mut dbg = Dbg::new(canvas.gfx(), font);

    let mut state = State::new(&canvas, world);

    let framerate_delta = Duration::from_secs_f64(1.0 / FPS_CAP as f64);
    let mut time_delta = Instant::now();
    let mut fps_update_delta = Instant::now();
    let mut fps_counter = 0;
    let mut frame_time = 0.0;
    let mut current_fps = 0;

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);
            match event {
                Event::NewEvents(_) => {
                    let elapsed = time_delta.elapsed();

                    if elapsed >= framerate_delta {
                        time_delta = Instant::now();
                        winit_window.request_redraw();
                        fps_counter += 1;
                        frame_time = elapsed.as_secs_f32();
                        if fps_update_delta.elapsed().as_micros() >= 1000000 {
                            fps_update_delta = Instant::now();
                            current_fps = fps_counter;
                            fps_counter = 0;
                        }
                    }
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                        elwt.exit();
                    }
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
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
                        winit_window.request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        state.update(frame_time);

                        let dbg_data = state.collect_dbg_data(1000.0 / current_fps as f64, current_fps);
                        dbg.update(dbg_data);

                        // TODO check result instead of unwrap
                        dbg.queue_data(canvas.gfx()).unwrap();
                        // Clearing the buffer isn't needed since everything is being overwritten
                        //canvas.clear_buffer();
                        state.draw(canvas.mut_column_iterator());

                        match canvas.render(&dbg) {
                            Ok(_) => (),
                            Err(wgpu::SurfaceError::Lost) => canvas.on_surface_lost(),
                            // The system is out of memory, we should probably quit
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                println!("Out of memory!");
                                elwt.exit()
                            }
                            // All other errors (Outdated, Timeout) should be resolved by the next frame
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                    _ => (),
                },
                Event::DeviceEvent { event, .. } => {
                    match event {
                        DeviceEvent::MouseMotion { delta } => {
                            let delta = Vec2::new(delta.0 as f32, delta.1 as f32);
                            state.on_mouse_move(event)
                        }
                        _ => (),
                    }
                },
                Event::LoopExiting => println!("Exited!"),
                _ => (),
            }
        })
        .unwrap();
}
