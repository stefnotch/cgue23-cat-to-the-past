use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::{ResMut, Resource};
use winit::event::{ElementState, MouseButton, VirtualKeyCode};

pub struct MouseMovement(pub f64, pub f64);
pub struct KeyboardInput {
    pub key_code: VirtualKeyCode,
    pub state: ElementState,
}
pub struct MouseInput {
    pub button: MouseButton,
    pub state: ElementState,
}
// pub struct KeyboardUp;

// pub enum InputEvent {
//     KeyboardUp(),
//     KeyboardDown(),
//     MouseDown(),
//     MouseUp(),
//     MouseMovement((f64, f64)),
// }

const NUM_KEYS: usize = VirtualKeyCode::Cut as usize + 1;
const NUM_MOUSE_BUTTONS: usize = 2;

#[derive(Resource)]
pub struct InputMap {
    state: [bool; NUM_KEYS],
    mouse_state: [bool; NUM_MOUSE_BUTTONS],
}

impl InputMap {
    pub fn new() -> Self {
        InputMap {
            state: [false; NUM_KEYS],
            mouse_state: [false; NUM_MOUSE_BUTTONS],
        }
    }

    fn update_key_press(&mut self, key: VirtualKeyCode) {
        self.state[key as usize] = true;
    }

    fn update_key_release(&mut self, key: VirtualKeyCode) {
        self.state[key as usize] = false;
    }

    fn update_mouse_press(&mut self, button: MouseButton) {
        match button {
            MouseButton::Left => self.mouse_state[0] = true,
            MouseButton::Right => self.mouse_state[1] = true,
            _ => {}
        }
    }

    fn update_mouse_release(&mut self, button: MouseButton) {
        match button {
            MouseButton::Left => self.mouse_state[0] = false,
            MouseButton::Right => self.mouse_state[1] = false,
            _ => {}
        }
    }

    pub fn is_pressed(&self, key: VirtualKeyCode) -> bool {
        self.state[key as usize]
    }
}

pub fn handle_keyboard_input(
    mut input: ResMut<InputMap>,
    mut event_reader: EventReader<KeyboardInput>,
) {
    for event in event_reader.iter() {
        match event.state {
            ElementState::Pressed => {
                input.update_key_press(event.key_code);
            }
            ElementState::Released => {
                input.update_key_release(event.key_code);
            }
        }
    }
}

pub fn handle_mouse_input(mut input: ResMut<InputMap>, mut event_reader: EventReader<MouseInput>) {
    for event in event_reader.iter() {
        match event.state {
            ElementState::Pressed => {
                input.update_mouse_press(event.button);
            }
            ElementState::Released => {
                input.update_mouse_release(event.button);
            }
        }
    }
}
