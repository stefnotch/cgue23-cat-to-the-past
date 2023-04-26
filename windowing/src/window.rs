use std::sync::Arc;

use app::plugin::Plugin;
use bevy_ecs::system::Resource;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event_loop::EventLoop,
    window::{Fullscreen, Window, WindowBuilder},
};

use crate::{config::WindowConfig, icon::get_icon};

#[derive(Resource)]
pub struct WindowManager {
    pub window: Arc<Window>,
}

pub struct EventLoopContainer {
    pub event_loop: EventLoop<()>,
}

pub struct WindowPlugin {
    config: WindowConfig,
}

impl WindowPlugin {
    pub fn new(config: WindowConfig) -> Self {
        Self { config }
    }
}

impl Plugin for WindowPlugin {
    fn build(&mut self, app: &mut app::plugin::PluginAppAccess) {
        let event_loop = EventLoop::new();
        let window_builder = create_window_builder(self.config.clone(), &event_loop);
        let window = Arc::new(window_builder.build(&event_loop).unwrap());

        app.with_resource(WindowManager { window });
        app.with_non_send_resource(EventLoopContainer { event_loop });
    }
}

fn create_window_builder(config: WindowConfig, event_loop: &EventLoop<()>) -> WindowBuilder {
    let monitor = event_loop
        .available_monitors()
        .next()
        .expect("no monitor found!");

    let mut window_builder = WindowBuilder::new()
        .with_inner_size(LogicalSize {
            width: config.resolution.0,
            height: config.resolution.1,
        })
        .with_title("Cat to the past");

    if let Ok(icon) = get_icon() {
        //.with_taskbar_icon(taskbar_icon)
        window_builder = window_builder.with_window_icon(Some(icon));
    }

    if config.fullscreen {
        if let Some(video_mode) = monitor
            .video_modes()
            .filter(|v| {
                let PhysicalSize { width, height } = v.size();

                v.refresh_rate_millihertz() == config.refresh_rate * 1000
                    && (width, height) == config.resolution
            })
            .next()
        {
            window_builder = window_builder.with_fullscreen(Some(Fullscreen::Exclusive(video_mode)))
        }
    }
    window_builder
}
