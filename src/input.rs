use winit::event::VirtualKeyCode;

const NUM_KEYS: usize = VirtualKeyCode::Cut as usize + 1;

pub struct InputMap {
    state: [bool; NUM_KEYS],
    last_state: [bool; NUM_KEYS],
    /// delta since last frame
    mouse_delta: (f64, f64),
}

impl InputMap {
    pub fn new() -> Self {
        InputMap {
            state: [false; NUM_KEYS],
            last_state: [false; NUM_KEYS],
            mouse_delta: (0.0, 0.0),
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

    pub fn is_pressed_this_frame(&self, key: VirtualKeyCode) -> bool {
        self.state[key as usize] && !self.last_state[key as usize]
    }

    pub fn is_released_this_frame(&self, key: VirtualKeyCode) -> bool {
        !self.state[key as usize] && self.last_state[key as usize]
    }

    pub fn mouse_move(&mut self, delta: (f64, f64)) {
        self.mouse_delta.0 += delta.0;
        self.mouse_delta.1 += delta.1;
    }

    pub fn mouse_delta(&self) -> (f64, f64) {
        self.mouse_delta
    }

    pub fn new_frame(&mut self) {
        self.mouse_delta = (0.0, 0.0);
        self.last_state.copy_from_slice(&self.state[..]);
    }
}
