use winit::event::VirtualKeyCode;

const NUM_KEYS: usize = VirtualKeyCode::Cut as usize + 1;

pub struct InputMap {
    state: [bool; NUM_KEYS],
}

impl InputMap {
    pub fn new() -> Self {
        InputMap {
            state: [false; NUM_KEYS],
        }
    }

    pub fn key_press(&mut self, key: VirtualKeyCode) {
        self.state[key as usize] = true;
    }

    pub fn key_release(&mut self, key: VirtualKeyCode) {
        self.state[key as usize] = false;
    }

    pub fn is_pressed(&self, key: VirtualKeyCode) -> bool {
        self.state[key as usize]
    }
}
