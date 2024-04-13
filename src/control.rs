use hashbrown::{HashMap, HashSet};
use strum::{EnumIter, IntoEnumIterator};
use winit::keyboard::KeyCode;

pub struct ControllerSettings {
    keybindings: HashMap<GameInput, KeyCode>,
    inverse_keybindings: HashMap<KeyCode, HashSet<GameInput>>,
}

impl ControllerSettings {
    pub fn init() -> Self {
        Self::default()
    }

    pub fn get_input_binding(&self, input_key: &KeyCode) -> Option<&HashSet<GameInput>> {
        self.inverse_keybindings.get(input_key)
    }

    fn default_binding(input: GameInput) -> KeyCode {
        match input {
            GameInput::MoveForward => KeyCode::KeyW,
            GameInput::MoveBackward => KeyCode::KeyS,
            GameInput::StrafeLeft => KeyCode::KeyA,
            GameInput::StrafeRight => KeyCode::KeyD,
            GameInput::Jump => KeyCode::Space,
            GameInput::FlyUp => KeyCode::Space,
            GameInput::FlyDown => KeyCode::ShiftLeft,
            GameInput::IncreaseFOV => KeyCode::ArrowDown,
            GameInput::DecreaseFOV => KeyCode::ArrowUp,
            GameInput::PhysicsSwitch => KeyCode::Equal,
        }
    }
}

impl Default for ControllerSettings {
    fn default() -> Self {
        let mut new = Self {
            keybindings: HashMap::new(),
            inverse_keybindings: HashMap::new(),
        };
        for input in GameInput::iter() {
            let default_binding = Self::default_binding(input);
            new.keybindings.insert(input, default_binding);
            new.inverse_keybindings
                .entry(default_binding)
                .or_default()
                .insert(input);
        }

        new
    }
}

#[derive(Debug, Clone, Copy, EnumIter, PartialEq, Eq, Hash)]
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
    PhysicsSwitch,
}
