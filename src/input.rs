use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::{ResMut, Resource};
use winit::event::{ElementState, VirtualKeyCode};

use crate::application::ApplicationBuilder;
use crate::player::PlayerSettings;

pub struct MouseMovement(pub f64, pub f64);
pub struct KeyboardInput {
    pub key_code: VirtualKeyCode,
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

#[derive(Resource)]
pub struct InputMap {
    state: [bool; NUM_KEYS],
}

impl InputMap {
    pub fn new() -> Self {
        InputMap {
            state: [false; NUM_KEYS],
        }
    }

    fn update_key_press(&mut self, key: VirtualKeyCode) {
        self.state[key as usize] = true;
    }

    fn update_key_release(&mut self, key: VirtualKeyCode) {
        self.state[key as usize] = false;
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
