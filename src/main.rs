mod backend;
mod control;
mod map;
mod raycaster;
//mod map_parser;
mod models;
mod player;
//mod old_raycaster;
mod state;
mod textures;

use std::time::{Duration, Instant};

use backend::Canvas;
use control::{ControllerSettings, GameInput};
use state::GameState;
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
const CANVAS_WIDTH_FACTOR: u32 = 16;
const CANVAS_HEIGHT_FACTOR: u32 = 9;
const DEFAULT_CANVAS_WIDTH: u32 = 16 * 15;
const DEFAULT_CANVAS_HEIGHT: u32 = 9 * 15;

pub struct App {
    canvas: Option<Canvas>,
    controls: ControllerSettings,

    state: GameState,

    time_per_frame: Duration,
    now: Instant,
    sleep_between_frames: bool,

    acc_fps: u128,
    time: Instant,
}

impl App {
    pub fn new() -> Self {
        Self {
            canvas: None,
            controls: ControllerSettings::init(),

            state: GameState::new(
                "maps/map.txt",
                DEFAULT_CANVAS_WIDTH,
                DEFAULT_CANVAS_HEIGHT,
            ),

            time_per_frame: Duration::from_secs_f64(1.0 / FPS_CAP as f64),
            now: Instant::now(),
            sleep_between_frames: false,

            acc_fps: 0,
            time: Instant::now(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.canvas = Some(Canvas::new(
            event_loop,
            DEFAULT_CANVAS_WIDTH,
            DEFAULT_CANVAS_HEIGHT,
        ));
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
                            match input {
                                GameInput::ToggleSleepBetweenFrames if !is_pressed => {
                                    self.sleep_between_frames = !self.sleep_between_frames
                                }
                                GameInput::ToggleFullScreen if !is_pressed => {
                                    self.canvas.as_mut().unwrap().toggle_full_screen()
                                }
                                GameInput::IncreaseResolution if !is_pressed => {
                                    let canvas = self.canvas.as_mut().unwrap();
                                    canvas.increase_resolution();
                                    self.state.recreate_camera(
                                        canvas.view_width(),
                                        canvas.view_height(),
                                    );
                                    println!(
                                        "new dimensions: {}x{}",
                                        canvas.view_width(),
                                        canvas.view_height()
                                    )
                                }
                                GameInput::DecreaseResolution if !is_pressed => {
                                    let canvas = self.canvas.as_mut().unwrap();
                                    canvas.decrease_resolution();
                                    self.state.recreate_camera(
                                        canvas.view_width(),
                                        canvas.view_height(),
                                    );
                                    println!(
                                        "new dimensions: {}x{}",
                                        canvas.view_width(),
                                        canvas.view_height()
                                    )
                                }
                                _ => self.state.handle_game_input(input, is_pressed),
                            }
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
                if self.time.elapsed().as_micros() >= 1_000_000 {
                    println!("average: {} FPS", self.acc_fps);
                    self.acc_fps = 0;
                    self.time = Instant::now();
                }
                self.acc_fps += 1;
                // First render game by pixel manipulation, ...
                self.state.render(canvas.mut_column());
                // ... then request the screen redraw.
                canvas.request_redraw();
            }
        } else if self.sleep_between_frames {
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
