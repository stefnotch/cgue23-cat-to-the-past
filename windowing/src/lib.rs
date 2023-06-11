pub mod config;
pub mod events;
mod icon;
pub mod window;

pub mod dpi {
    pub use winit::dpi::*;
}

pub mod event {
    pub use winit::event::{
        DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent,
    };
}

pub mod event_loop {
    pub use winit::event_loop::ControlFlow;
}
