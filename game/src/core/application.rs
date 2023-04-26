use app::plugin::Plugin;
use app::App;
use game_core::level::level_flags::LevelFlags;
use game_core::time::{TimePlugin, TimePluginSet};
use physics::plugin::PhysicsPlugin;

use crate::input::events::{WindowFocusChanged, WindowResize};
use crate::input::input_map::{handle_keyboard_input, handle_mouse_input, InputMap};
use angle::Deg;
use bevy_ecs::prelude::*;
use game_core::application::AppStage;
use game_core::camera::{update_camera, Camera};
use input::events::{KeyboardInput, MouseInput, MouseMovement};
use nalgebra::{Point3, UnitQuaternion};
use render::context::Context;
use render::Renderer;
use scene_loader::loader::AssetServer;
use windowing::config::WindowConfig;
use windowing::icon::get_icon;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{
    DeviceEvent, Event, KeyboardInput as KeyboardInputWinit, MouseButton, VirtualKeyCode,
    WindowEvent,
};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Fullscreen::Exclusive;
use winit::window::{CursorGrabMode, WindowBuilder};

use crate::core::transform_change::{
    time_manager_rewind_transform, time_manager_track_transform, TransformChange,
};
use game_core::time_manager::game_change::{GameChangeHistoryPlugin};
use game_core::time_manager::{is_rewinding, TimeManager, TimeManagerPlugin, TimeManagerPluginSet};

pub struct AppConfig {
    pub window: WindowConfig,
    /// Projectors are usually very dark, this parameter should control how bright your total
    /// scene is, e.g., an illumination multiplier
    pub brightness: f32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window: WindowConfig {
                resolution: (1280, 720),
                fullscreen: false,
                refresh_rate: 60,
            },
            brightness: 1.0,
        }
    }
}
pub struct Application {
    event_loop: EventLoop<()>,

    config: AppConfig,
    pub app: App,
}

impl Application {
    pub fn new(config: AppConfig) -> Self {
        let event_loop = EventLoop::new();

        let mut app = App::new();
        app.schedule.configure_sets(
            (
                AppStage::StartFrame,
                AppStage::EventUpdate,
                AppStage::Update,
                AppStage::UpdatePhysics,
                AppStage::BeforeRender,
                AppStage::Render,
                AppStage::EndFrame,
            )
                .chain(),
        );

        Self::add_default_plugins(&mut app);

        Self {
            event_loop,

            config,
            app,
        }
    }

    fn add_default_plugins(app: &mut App) {
        app.with_plugin(TimePlugin)
            .with_set(TimePluginSet::UpdateTime.in_set(AppStage::StartFrame))
            .with_plugin(TimeManagerPlugin)
            .with_set(
                TimeManagerPluginSet::StartFrame
                    .in_set(AppStage::StartFrame)
                    .after(TimePluginSet::UpdateTime),
            )
            .with_plugin(PhysicsPlugin)
            .with_set(PhysicsPlugin::system_set().in_set(AppStage::UpdatePhysics))
            // Transform tracking
            .with_plugin(GameChangeHistoryPlugin::<TransformChange>::new())
            .with_system(
                time_manager_track_transform
                    .after(AppStage::Update)
                    .before(AppStage::UpdatePhysics)
                    .run_if(not(is_rewinding)),
            )
            .with_system(
                time_manager_rewind_transform
                    .after(AppStage::Update)
                    .before(AppStage::UpdatePhysics)
                    .run_if(is_rewinding),
            );
    }

    pub fn run(mut self)
    where
        Self: 'static,
    {
        let config = &self.config;
        let window_builder = self.create_window(&config.window);

        let mut world = &mut self.app.world;
        let schedule = &mut self.app.schedule;

        let aspect_ratio = config.window.resolution.0 as f32 / config.window.resolution.1 as f32;

        let asset_server = AssetServer::new();
        world.insert_resource(asset_server);
        AssetServer::insert_asset_types(&mut world);

        let camera = Camera::new(
            Point3::origin(), // Note: The player updates this
            UnitQuaternion::identity(),
            aspect_ratio,
            Deg(60.0),
            0.01,
            100.0,
        );
        schedule.add_system(update_camera_aspect_ratio.in_set(AppStage::EventUpdate));
        schedule.add_system(update_camera.in_set(AppStage::BeforeRender));
        world.insert_resource(camera);

        let context = Context::new(window_builder, &self.event_loop);
        let renderer = Renderer::new(&context);
        renderer.setup_systems(&context, world, schedule);
        world.insert_resource(context);

        world.insert_resource(LevelFlags::new());

        // TODO: Move that code to the input.rs file?
        let input_map = InputMap::new();
        world.insert_resource(input_map);
        world.insert_resource(Events::<MouseMovement>::default());
        schedule.add_system(Events::<MouseMovement>::update_system.in_set(AppStage::EventUpdate));

        world.insert_resource(Events::<MouseInput>::default());
        schedule.add_system(Events::<MouseInput>::update_system.in_set(AppStage::EventUpdate));

        world.insert_resource(Events::<KeyboardInput>::default());
        schedule.add_system(Events::<KeyboardInput>::update_system.in_set(AppStage::EventUpdate));

        world.insert_resource(Events::<WindowResize>::default());
        schedule.add_system(Events::<WindowResize>::update_system.in_set(AppStage::EventUpdate));

        world.insert_resource(Events::<WindowFocusChanged>::default());
        schedule
            .add_system(Events::<WindowFocusChanged>::update_system.in_set(AppStage::EventUpdate));

        schedule.add_system(handle_keyboard_input.in_set(AppStage::EventUpdate));
        schedule.add_system(handle_mouse_input.in_set(AppStage::EventUpdate));

        schedule.add_system(read_input.in_set(AppStage::Update));

        schedule.add_system(lock_mouse);

        self.app.run_startup();

        self.event_loop
            .run(move |event, _, control_flow| match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }

                Event::WindowEvent {
                    event: WindowEvent::Resized(PhysicalSize { width, height }),
                    ..
                } => {
                    self.app.world.send_event(WindowResize { width, height });

                    self.app
                        .world
                        .get_non_send_resource_mut::<Renderer>()
                        .unwrap()
                        .recreate_swapchain();
                }

                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInputWinit {
                                virtual_keycode: Some(key_code),
                                state,
                                ..
                            },
                        ..
                    } => {
                        if key_code == VirtualKeyCode::Escape {
                            *control_flow = ControlFlow::Exit;
                        }

                        self.app.world.send_event(KeyboardInput { key_code, state });
                    }
                    WindowEvent::MouseInput { button, state, .. } => {
                        self.app.world.send_event(MouseInput { button, state });
                    }
                    WindowEvent::Focused(focused) => {
                        self.app
                            .world
                            .send_event(WindowFocusChanged { has_focus: focused });
                    }
                    _ => (),
                },

                Event::DeviceEvent { event, .. } => match event {
                    DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                        self.app.world.send_event(MouseMovement(dx, dy))
                    }
                    _ => (),
                },

                Event::RedrawEventsCleared => {
                    self.app.schedule.run(&mut self.app.world);
                }

                _ => (),
            });
    }

    fn create_window(&self, config: &WindowConfig) -> WindowBuilder {
        let monitor = self
            .event_loop
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
                window_builder = window_builder.with_fullscreen(Some(Exclusive(video_mode)))
            }
        }
        window_builder
    }
}

fn lock_mouse(context: Res<Context>, mut event: EventReader<WindowFocusChanged>) {
    for WindowFocusChanged { has_focus } in event.into_iter() {
        let window = context.window();

        if *has_focus {
            window
                .set_cursor_grab(CursorGrabMode::Confined)
                .or_else(|_e| window.set_cursor_grab(CursorGrabMode::Locked))
                .unwrap();
            window.set_cursor_visible(false);
        } else {
            window.set_cursor_grab(CursorGrabMode::None).unwrap();
            window.set_cursor_visible(true);
        }
    }
}

fn update_camera_aspect_ratio(mut camera: ResMut<Camera>, mut reader: EventReader<WindowResize>) {
    for event in reader.iter() {
        camera.update_aspect_ratio(event.width as f32 / event.height as f32);
    }
}

fn read_input(mut time_manager: ResMut<TimeManager>, mouse_input: Res<InputMap>) {
    time_manager.will_rewind_next_frame = mouse_input.is_mouse_pressed(MouseButton::Right);
}
