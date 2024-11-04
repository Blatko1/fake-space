pub mod backend;
mod control;
mod dbg;
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

use crate::dbg::Dbg;
use backend::ctx::Ctx;
use backend::Canvas;
use control::ControllerSettings;
use dbg::DebugData;
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
use state::{GameState, State};
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
const CANVAS_WIDTH: u32 = 240 * 1;
const CANVAS_HEIGHT: u32 = 135 * 1;
const PHYSICS_TIMESTEP: f32 = 0.01;
const SLEEP_BETWEEN_FRAMES: bool = false;

pub struct App {
    canvas: Option<Canvas>,
    controls: ControllerSettings,
    dbg: Option<Dbg>,

    state: GameState,

    delta_accumulator: f32,
    time_per_frame: Duration,
    now: Instant,
}

impl App {
    pub fn new() -> Self {
        Self {
            canvas: None,
            controls: ControllerSettings::init(),
            dbg: None,

            state: GameState::new("maps/map.txt"),

            delta_accumulator: 0.0,
            time_per_frame: Duration::from_secs_f64(1.0 / FPS_CAP as f64),
            now: Instant::now(),
        }
    }

    fn update(&mut self, delta: f32) {
        // Update world and player
        self.delta_accumulator += delta;
        while self.delta_accumulator >= PHYSICS_TIMESTEP {
            self.player.update(&self.map, PHYSICS_TIMESTEP);
            self.delta_accumulator -= PHYSICS_TIMESTEP;
        }
        //self.world.update(&mut self.player);

        let dbg_data = self.collect_dbg_data();

        if let Some(dbg) = self.dbg.as_mut() {
            dbg.update(dbg_data)
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let ctx = block_on(Ctx::new(event_loop)).unwrap();
        // TODO change/fix this
        let font_data = fs::read("res/Minecraft.ttf").unwrap();
        let font = FontVec::try_from_vec(font_data).unwrap();
        self.dbg = Some(Dbg::new(&ctx, font));

        self.canvas = Some(Canvas::new(ctx, CANVAS_WIDTH, CANVAS_HEIGHT));
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
                        for input in game_input.iter() {
                            self.player.process_input(*input, is_pressed)
                        }
                    }
                }
            }
            WindowEvent::Resized(new_size) => {
                let canvas = self.canvas.as_mut().unwrap();
                let dbg = self.dbg.as_mut().unwrap();
                canvas.resize(new_size);
                dbg.resize(canvas);
                canvas.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let canvas = self.canvas.as_mut().unwrap();
                let dbg = self.dbg.as_mut().unwrap();

                // TODO check result instead of unwrap
                dbg.queue_data(canvas.ctx()).unwrap();
                // Clearing the buffer isn't needed since everything is being overwritten
                // canvas.clear_buffer();
                raycaster::cast_and_draw(&self.player, &self.map, &self.textures, &self.models, canvas.mut_column_iterator());

                match canvas.render(dbg) {
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

            if let Some(canvas) = self.canvas.as_ref() {
                canvas.request_redraw();
                self.dbg.as_mut().unwrap().update_frame_timings();
            }
            self.update(elapsed.as_secs_f32());
        } else if SLEEP_BETWEEN_FRAMES {
            event_loop.set_control_flow(ControlFlow::WaitUntil(
                Instant::now()
                    .checked_add(self.time_per_frame - elapsed)
                    .unwrap(),
            ))
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            let delta = Vec2::new(delta.0 as f32, delta.1 as f32);
            self.player.on_mouse_move(delta);
        }
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
