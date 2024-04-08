use hashbrown::{HashMap, HashSet};
use winit::keyboard::KeyCode;



pub struct ControllSettings {
    keybindings: HashMap<GameInput, Option<KeyCode>>,
    inverse_keybindings: HashMap<KeyCode, HashSet<GameInput>>
}

impl ControllSettings {
    pub fn init() -> Self {
        
    }
}

pub enum GameInput {
    MoveForward,
    MoveBackward,
    StrafeLeft,
    StrafeRight,
    Jump,
    FlyUp,
    FlyDown,
    IncreaseFOV,
    DecreaseFOV,
}