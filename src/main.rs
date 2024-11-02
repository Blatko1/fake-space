/// For the record:
/// I have tried adding FXAA in the fragment shader, which ended up in a weird
/// output, have tried MSAA, but it doesn't work on textures, have tried applying
/// bilinear texture filtering but unnoticeable.
pub mod backend;
mod control;
mod dbg;
mod player;
mod world;

use std::fs;
use std::time::{Duration, Instant};

use crate::dbg::Dbg;
use crate::world::World;
use backend::ctx::Ctx;
use backend::Canvas;
use control::ControllerSettings;
use dbg::DebugData;
use glam::Vec2;
use player::camera::Camera;
use player::Player;
use pollster::block_on;
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
use world::RoomID;

const FPS_CAP: u32 = 60;
const CANVAS_WIDTH: u32 = 240 * 1;
const CANVAS_HEIGHT: u32 = 135 * 1;
const PHYSICS_TIMESTEP: f32 = 0.01;
const SLEEP_BETWEEN_FRAMES: bool = false;

pub struct State {
    canvas: Option<Canvas>,
    controls: ControllerSettings,
    dbg: Option<Dbg>,

    world: World,
    player: Player,

    delta_accumulator: f32,
    time_per_frame: Duration,
    now: Instant,
}

impl State {
    pub fn new() -> Self {
        let camera = Camera::new(
            10.5,
            1.0,
            14.5,
            90f32.to_radians(),
            CANVAS_WIDTH,
            CANVAS_HEIGHT,
        );

        Self {
            canvas: None,
            controls: ControllerSettings::init(),
            dbg: None,

            world: World::from_path("maps/world.txt").unwrap(),
            player: Player::new(camera, RoomID(0)),

            delta_accumulator: 0.0,
            time_per_frame: Duration::from_secs_f64(1.0 / FPS_CAP as f64),
            now: Instant::now(),
        }
    }

    fn update(&mut self, delta: f32) {
        // Update world and player
        self.delta_accumulator += delta;
        while self.delta_accumulator >= PHYSICS_TIMESTEP {
            self.player.update(&self.world, PHYSICS_TIMESTEP);
            self.delta_accumulator -= PHYSICS_TIMESTEP;
        }
        self.world.update(&mut self.player);

        let dbg_data = self.collect_dbg_data();

        if let Some(dbg) = self.dbg.as_mut() {
            dbg.update(dbg_data)
        }
    }

    pub fn collect_dbg_data(&self) -> DebugData {
        let player_dbg_data = self.player.collect_dbg_data();
        let world_dbg_data = self.world.collect_dbg_data();

        DebugData {
            player_data: player_dbg_data,
            world_data: world_dbg_data,
        }
    }
}

impl ApplicationHandler for State {
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
                self.player
                    .cast_and_draw(&self.world, canvas.mut_column_iterator());

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
    let mut state = State::new();
    event_loop.run_app(&mut state).unwrap();
}
