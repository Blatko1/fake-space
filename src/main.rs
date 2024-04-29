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
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::dbg::Dbg;
use crate::world::World;
use backend::Canvas;
use control::{ControllerSettings, GameInput};
use dbg::DebugData;
use glam::Vec2;
use hashbrown::HashSet;
use player::camera::Camera;
use player::Player;
use pollster::block_on;
use wgpu_text::glyph_brush::ab_glyph::FontVec;
use winit::event::{DeviceEvent, KeyEvent};
use winit::keyboard::{Key, NamedKey, PhysicalKey};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder as WinitWindowBuilder,
};
use world::RoomID;

const FPS_CAP: u32 = 60;
const CANVAS_WIDTH: u32 = 240 * 2;
const CANVAS_HEIGHT: u32 = 135 * 2;
const PHYSICS_TIMESTEP: f32 = 0.01;

pub struct State {
    world: World,
    player: Player,

    accumulator: f32,
}

impl State {
    pub fn new(canvas: &Canvas, world: World) -> Self {
        let camera = Camera::new(
            10.5,
            1.0,
            14.5,
            90f32.to_radians(),
            canvas.width(),
            canvas.height(),
        );

        Self {
            world,
            player: Player::new(camera, RoomID(0)),

            accumulator: 0.0,
        }
    }

    pub fn update(&mut self, delta: f32) {
        self.accumulator += delta;
        while self.accumulator >= PHYSICS_TIMESTEP {
            self.player.update(&self.world, PHYSICS_TIMESTEP);
            self.accumulator -= PHYSICS_TIMESTEP;
        }
        self.world.update(self.player.current_room_id());
    }

    pub fn draw<'a, C>(&self, canvas_column_iter: C)
    where
        C: Iterator<Item = &'a mut [u8]>,
    {
        self.player.cast_and_draw(&self.world, canvas_column_iter);
    }

    #[inline]
    pub fn process_game_input(
        &mut self,
        game_input: &HashSet<GameInput>,
        is_pressed: bool,
    ) {
        for input in game_input.iter() {
            self.player.process_input(*input, is_pressed)
        }
    }

    #[inline]
    pub fn on_mouse_move(&mut self, delta: Vec2) {
        self.player.on_mouse_move(delta);
    }

    pub fn collect_dbg_data(&self, avg_fps_time: f64, current_fps: i32) -> DebugData {
        let player_dbg_data = self.player.collect_dbg_data();
        let world_dbg_data = self.world.collect_dbg_data();

        DebugData {
            current_fps,
            avg_fps_time,

            player_data: player_dbg_data,
            world_data: world_dbg_data,
        }
    }
}

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error");
    }
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let winit_window = Arc::new(WinitWindowBuilder::new().build(&event_loop).unwrap());
    winit_window.set_title("False Space");

    let controls = ControllerSettings::init();
    let world = World::from_path("maps/world.txt").unwrap();

    let mut canvas = block_on(Canvas::init(
        winit_window.clone(),
        CANVAS_WIDTH,
        CANVAS_HEIGHT,
    ));
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
                        if let PhysicalKey::Code(key) = event.physical_key {
                            if let Some(game_input) = controls.get_input_binding(&key) {
                                let is_pressed = event.state.is_pressed();
                                state.process_game_input(game_input, is_pressed);
                            }
                        }
                    }
                    WindowEvent::Resized(new_size) => {
                        canvas.resize(new_size);
                        dbg.resize(&canvas);
                        winit_window.request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        state.update(frame_time);

                        let dbg_data = state
                            .collect_dbg_data(1000.0 / current_fps as f64, current_fps);
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
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta },
                    ..
                } => {
                    let delta = Vec2::new(delta.0 as f32, delta.1 as f32);
                    state.on_mouse_move(delta)
                }
                Event::LoopExiting => println!("Exited!"),
                _ => (),
            }
        })
        .unwrap();
}
