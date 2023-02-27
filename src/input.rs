use bevy_ecs::prelude::Resource;
use winit::event::VirtualKeyCode;

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

    pub fn update_key_press(&mut self, key: VirtualKeyCode) {
        self.state[key as usize] = true;
    }

    pub fn update_key_release(&mut self, key: VirtualKeyCode) {
        self.state[key as usize] = false;
    }

    pub fn is_pressed(&self, key: VirtualKeyCode) -> bool {
        self.state[key as usize]
    }
}
