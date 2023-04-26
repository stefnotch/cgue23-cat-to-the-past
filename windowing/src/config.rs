#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub resolution: (u32, u32),
    pub fullscreen: bool,
    /// The desired refresh rate of the game in fullscreen mode.
    /// Maps to "GLFW_REFRESH_RATE" in an OpenGL application, which only applies to fullscreen mode.
    /// We should query all video modes https://docs.rs/winit/latest/winit/monitor/struct.MonitorHandle.html#method.video_modes
    /// and pick the closest one to the desired refresh rate. https://docs.rs/winit/latest/winit/monitor/struct.VideoMode.html#method.refresh_rate_millihertz
    /// Then, we use that video mode to create the window in fullscreen mode.
    pub refresh_rate: u32,
}
