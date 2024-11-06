mod backend;
mod control;
mod player;
mod map;
mod textures;
mod models;
mod map_parser;
mod raycaster;
mod state;

use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::{Duration, Instant};

use backend::ctx::Ctx;
use backend::Canvas;
use control::ControllerSettings;
use glam::Vec2;
use map::room::RoomID;
use map::Map;
use map_parser::{cleanup_input, MapParser};
use models::ModelArray;
use nom::error::convert_error;
use nom::Finish;
use player::{Player};
use pollster::block_on;
use raycaster::camera::Camera;
use state::{GameState};
use textures::TextureArray;
use wgpu_text::glyph_brush::ab_glyph::FontVec;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, KeyEvent, StartCause};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey, PhysicalKey};
use winit::window::WindowId;
use winit::{
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
};

const FPS_CAP: u32 = 60;
const CANVAS_WIDTH: u32 = 240 * 2;
const CANVAS_HEIGHT: u32 = 135 * 2;
const SLEEP_BETWEEN_FRAMES: bool = false;

pub struct App {
    canvas: Option<Canvas>,
    controls: ControllerSettings,

    state: GameState,

    time_per_frame: Duration,
    now: Instant,
}

impl App {
    pub fn new() -> Self {
        Self {
            canvas: None,
            controls: ControllerSettings::init(),

            state: GameState::new("maps/map.txt"),

            time_per_frame: Duration::from_secs_f64(1.0 / FPS_CAP as f64),
            now: Instant::now(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.canvas = Some(Canvas::new(event_loop, CANVAS_WIDTH, CANVAS_HEIGHT));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        ..
                    },
                ..
            } => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    if let Some(game_input) = self.controls.get_input_binding(&key) {
                        let is_pressed = event.state.is_pressed();
                        for &input in game_input.iter() {
                            self.state.handle_game_input(input, is_pressed);
                        }
                    }
                }
            }
            WindowEvent::Resized(new_size) => {
                let canvas = self.canvas.as_mut().unwrap();
                canvas.resize(new_size);
                canvas.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let canvas = self.canvas.as_mut().unwrap();

                // Clearing the buffer isn't needed since everything is being overwritten
                // canvas.clear_buffer();

                match canvas.render() {
                    Ok(_) => (),
                    Err(wgpu::SurfaceError::Lost) => canvas.on_surface_lost(),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        println!("Out of memory!");
                        event_loop.exit()
                    }
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            _ => (),
        }
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, _: StartCause) {
        let elapsed = self.now.elapsed();

        if elapsed >= self.time_per_frame {
            self.now = Instant::now();
            // Update game
            self.state.update(elapsed.as_secs_f32());

            if let Some(canvas) = self.canvas.as_mut() {
                // First render game by pixel manipulation
                self.state.render(canvas.mut_column_iterator());
                // Then request the screen redraw
                canvas.request_redraw();
            }

        } else if SLEEP_BETWEEN_FRAMES {
            event_loop.set_control_flow(ControlFlow::WaitUntil(
                Instant::now()
                    .checked_add(self.time_per_frame - elapsed)
                    .unwrap(),
            ))
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        self.state.handle_device_event(event);
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        println!("Exited!")
    }
}

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error");
    }
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut state = App::new();
    event_loop.run_app(&mut state).unwrap();
}
