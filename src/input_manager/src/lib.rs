use std::collections::HashMap;

use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode},
};

mod game;

#[derive(serde::Serialize, serde::Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
enum UserAction {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down,

    Escape,

    // Mouse
    LeftClick,
    RightClick,
    MiddleClick,
}

enum KeyState {
    Pressed,
    Released,
    /// Analog input, such as mouse movement or gamepad axis
    Analog(f32),
}

impl KeyState {
    pub fn to_value(&self) -> f32 {
        match self {
            Self::Pressed => 1.0,
            Self::Released => 0.0,
            Self::Analog(value) => *value,
        }
    }
}

impl From<ElementState> for KeyState {
    fn from(state: ElementState) -> Self {
        match state {
            ElementState::Pressed => Self::Pressed,
            ElementState::Released => Self::Released,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InputManager {
    key_settings: HashMap<VirtualKeyCode, UserAction>,
    mouse_settings: HashMap<MouseButton, UserAction>,

    mouse_sensitivity: (f64, f64),

    // Used for faster lookup
    // #[serde(skip)]
    // key_settings_reversed: HashMap<UserAction, Vec<VirtualKeyCode>>,

    // State
    #[serde(skip)]
    pub is_focused: bool,
    #[serde(skip)]
    previous_mouse_position: PhysicalPosition<f64>,
    #[serde(skip)]
    mouse_delta: (f64, f64),
    #[serde(skip)]
    key_state: HashMap<UserAction, KeyState>,
    #[serde(skip)]
    mouse_state: HashMap<UserAction, KeyState>,
}

impl InputManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_key(&mut self, input: &KeyboardInput) {
        #[rustfmt::skip]
        let KeyboardInput { virtual_keycode, state, .. } = *input;

        if virtual_keycode.is_none() {
            #[cfg(feature = "debug_input")]
            log::info!("Unknown keyboard event, skipping ({})", input.scancode);
            return;
        }

        #[cfg(feature = "debug_input")]
        log::trace!("Keyboard event: {:?} {:?}", virtual_keycode, state);

        let virtual_keycode = virtual_keycode.unwrap();

        if let Some(&key_action) = self.key_settings.get(&virtual_keycode) {
            self.key_state.insert(key_action, state.into());
        }
    }

    pub fn update_mouse_position(&mut self, position: &PhysicalPosition<f64>) {
        let delta = (
            position.x - self.previous_mouse_position.x,
            position.y - self.previous_mouse_position.y,
        );
        self.previous_mouse_position = *position;

        self.update_mouse_delta(&delta);
    }

    pub fn update_mouse_delta(&mut self, delta: &(f64, f64)) {
        self.mouse_delta.0 += delta.0 * self.mouse_sensitivity.0;
        self.mouse_delta.1 += delta.1 * self.mouse_sensitivity.1;
    }

    pub fn update_mouse_button(&mut self, button: &MouseButton, state: &ElementState) {
        #[cfg(feature = "debug_input")]
        log::info!("Mouse button event: {} {:?}", button, state);

        if let Some(&key_action) = self.mouse_settings.get(button) {
            self.mouse_state.insert(key_action, (*state).into());
        }
    }

    pub fn update_focus(&mut self, is_focused: bool) {
        self.is_focused = is_focused;
    }

    pub fn clear_state(&mut self) {
        self.key_state.clear();
        self.mouse_state.clear();

        self.mouse_delta = (0f64, 0f64);
    }
}

/// Save and load the input manager to/from a file
impl InputManager {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self) {
        #[cfg(feature = "debug_input")]
        log::info!("Saving input settings to file");

        let mut file = std::fs::File::create("input_manager.ron").unwrap();
        ron::ser::to_writer(&mut file, &self).unwrap();
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) {
        #[cfg(feature = "debug_input")]
        log::info!("Saving input settings to local storage");

        let window = web_sys::window().unwrap();
        let storage = window.local_storage().unwrap().unwrap();

        let settings = ron::ser::to_string(&self).unwrap();
        storage.set_item("input_manager", &settings).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_file() -> Self {
        #[cfg(feature = "debug_input")]
        log::info!("Loading input settings from file");

        let file = std::fs::File::open("input_manager.ron").unwrap();
        ron::de::from_reader(&file).unwrap()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn from_file() -> Self {
        #[cfg(feature = "debug_input")]
        log::info!("Loading input settings from local storage");

        let window = web_sys::window().unwrap();
        let storage = window.local_storage().unwrap().unwrap();

        let settings = storage.get_item("input_manager").unwrap().unwrap();
        ron::de::from_str(&settings).unwrap()
    }
}

impl Default for InputManager {
    fn default() -> Self {
        #[cfg(feature = "debug_input")]
        log::info!("Loading default input settings");

        let mut key_settings: HashMap<VirtualKeyCode, UserAction> = HashMap::new();

        key_settings.insert(VirtualKeyCode::Z, UserAction::Forward);
        key_settings.insert(VirtualKeyCode::S, UserAction::Backward);
        key_settings.insert(VirtualKeyCode::Q, UserAction::Left);
        key_settings.insert(VirtualKeyCode::D, UserAction::Right);
        key_settings.insert(VirtualKeyCode::Space, UserAction::Up);
        key_settings.insert(VirtualKeyCode::LShift, UserAction::Down);
        key_settings.insert(VirtualKeyCode::Escape, UserAction::Escape);

        let mut mouse_settings: HashMap<MouseButton, UserAction> = HashMap::new();

        mouse_settings.insert(MouseButton::Left, UserAction::LeftClick);
        mouse_settings.insert(MouseButton::Right, UserAction::RightClick);
        mouse_settings.insert(MouseButton::Middle, UserAction::MiddleClick);

        Self {
            key_settings,
            mouse_settings,
            mouse_sensitivity: (0.001f64, 0.001f64),
            // key_settings_reversed,
            is_focused: false,
            previous_mouse_position: PhysicalPosition::new(0.0, 0.0),
            mouse_delta: (0.0, 0.0),
            key_state: HashMap::new(),
            mouse_state: HashMap::new(),
        }
    }
}
