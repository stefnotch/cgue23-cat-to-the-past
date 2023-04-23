use game_core::time::{update_time, Time};

use crate::input::events::{WindowFocusChanged, WindowResize};
use crate::input::input_map::{handle_keyboard_input, handle_mouse_input, InputMap};
use crate::scene::loader::AssetServer;
use angle::Deg;
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ExecutorKind;
use game_core::application::AppStage;
use game_core::camera::{update_camera, Camera};
use input::events::{KeyboardInput, MouseInput, MouseMovement};
use nalgebra::{Point3, UnitQuaternion};
use physics::physics_context::PhysicsContext;
use render::context::Context;
use render::{render, Renderer};
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
use game_core::time_manager::game_change::GameChangeHistory;
use game_core::time_manager::TimeManager;

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

pub struct ApplicationBuilder {
    config: AppConfig,
    startup_schedule: Schedule,
    schedule: Schedule,
    world: World,
}

impl ApplicationBuilder {
    pub fn new(config: AppConfig) -> Self {
        let mut startup_schedule = Schedule::default();
        startup_schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        let mut schedule = Schedule::default();
        schedule.configure_sets(
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

        schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        let world = World::new();

        ApplicationBuilder {
            config,
            startup_schedule,
            schedule,
            world,
        }
    }

    /// call this with system.in_set(AppStage::...)
    pub fn with_system<Params>(mut self, system: impl IntoSystemConfig<Params>) -> Self {
        self.schedule.add_system(system);
        self
    }

    /// call this with system.in_set(AppStartupStage::...)
    pub fn with_startup_system<Params>(mut self, system: impl IntoSystemConfig<Params>) -> Self {
        self.startup_schedule.add_system(system);
        self
    }

    pub fn with_resource<R: Resource>(mut self, res: R) -> Self {
        self.world.insert_resource(res);
        self
    }

    pub fn build(self) -> Application {
        Application::new(
            self.config,
            self.startup_schedule,
            self.schedule,
            self.world,
        )
    }
}

pub struct Application {
    event_loop: EventLoop<()>,

    config: AppConfig,
    world: World,
    schedule: Schedule,
    startup_schedule: Schedule,
}

impl Application {
    fn new(
        config: AppConfig,
        startup_schedule: Schedule,
        schedule: Schedule,
        world: World,
    ) -> Application {
        let event_loop = EventLoop::new();

        Application {
            event_loop,

            config,
            world,
            schedule,
            startup_schedule,
        }
    }

    pub fn run(mut self)
    where
        Self: 'static,
    {
        let config = &self.config;
        let window_builder = self.create_window(&config.window);

        let mut world = &mut self.world;
        let schedule = &mut self.schedule;
        let startup_schedule = &mut self.startup_schedule;

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

        let physics_context = PhysicsContext::new();
        physics_context.setup_systems(world, schedule);

        let time = Time::new();
        world.insert_resource(time);
        schedule.add_system(update_time.in_set(AppStage::StartFrame));

        let time_manager = TimeManager::new();
        time_manager.setup_systems(world, schedule);

        let transform_history = GameChangeHistory::<TransformChange>::new();
        transform_history.setup_systems(
            world,
            schedule,
            time_manager_track_transform,
            time_manager_rewind_transform,
        );

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

        startup_schedule.run(&mut world);

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
                    self.world.send_event(WindowResize { width, height });

                    self.world
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

                        self.world.send_event(KeyboardInput { key_code, state });
                    }
                    WindowEvent::MouseInput { button, state, .. } => {
                        self.world.send_event(MouseInput { button, state });
                    }
                    WindowEvent::Focused(focused) => {
                        self.world
                            .send_event(WindowFocusChanged { has_focus: focused });
                    }
                    _ => (),
                },

                Event::DeviceEvent { event, .. } => match event {
                    DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                        self.world.send_event(MouseMovement(dx, dy))
                    }
                    _ => (),
                },

                Event::RedrawEventsCleared => {
                    self.schedule.run(&mut self.world);
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
