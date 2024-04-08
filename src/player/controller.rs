use winit::{event::KeyEvent, keyboard::{KeyCode, PhysicalKey}};

pub struct Controller {
    pressed_forward: bool,
    pressed_backward: bool,
    pressed_strafe_left: bool,
    pressed_strafe_right: bool,
    pressed_jump: bool, 
    pressed_increase_fov: bool,
    pressed_decrease_fov: bool,

}

impl Controller {
    pub fn new() -> Self {
        
    }

    pub fn process_keyboard_input(&mut self, event: KeyEvent) {
        let is_pressed = event.state.is_pressed();
        if let PhysicalKey::Code(key) = event.physical_key {
            match key {
                KeyCode::KeyW => self.pressed_forward = is_pressed,
                KeyCode::KeyS => self.pressed_backward = is_pressed,
                KeyCode::KeyA => self.pressed_strafe_left = is_pressed,
                KeyCode::KeyD => self.pressed_strafe_right = is_pressed,
                KeyCode::Space => self.pressed_jump = is_pressed,
                KeyCode::ArrowUp => self.pressed_increase_fov = is_pressed,
                KeyCode::ArrowDown => self.pressed_decrease_fov = is_pressed,
                // Look more up (y_shearing):
                KeyCode::PageUp => self.camera.increase_y_shearing = value,
                // Look more down (y_shearing):
                KeyCode::PageDown => self.camera.decrease_y_shearing = value,
                // Reset look (y_shearing):
                KeyCode::Home => self.camera.y_shearing = 0.0,
                _ => (),
            }
        }
    }
}