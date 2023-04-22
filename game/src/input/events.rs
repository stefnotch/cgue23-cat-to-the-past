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

pub struct WindowResize {
    pub width: u32,
    pub height: u32,
}

pub struct WindowFocusChanged {
    pub has_focus: bool,
}
