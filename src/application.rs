use crate::camera::{update_camera, update_camera_aspect_ratio, Camera};
use crate::context::Context;
use crate::input;
use crate::input::{InputMap, MouseMovement};
use crate::physics_context::{
    insert_collider_component, step_character_controller, step_physics_simulation,
    update_transform_system, PhysicsContext,
};
use crate::render::{render, Renderer};
use crate::scene::loader::AssetServer;
use crate::time::Time;
use angle::Deg;
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ExecutorKind;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{DeviceEvent, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Fullscreen::Exclusive;
use winit::window::{CursorGrabMode, Window, WindowBuilder};

pub struct AppConfig {
    pub resolution: (u32, u32),
    pub fullscreen: bool,
    /// Projectors are usually very dark, this parameter should control how bright your total
    /// scene is, e.g., an illumination multiplier
    pub brightness: f32,
    /// The desired refresh rate of the game in fullscreen mode.
    /// Maps to "GLFW_REFRESH_RATE" in an OpenGL application, which only applies to fullscreen mode.
    /// We should query all video modes https://docs.rs/winit/latest/winit/monitor/struct.MonitorHandle.html#method.video_modes
    /// and pick the closest one to the desired refresh rate. https://docs.rs/winit/latest/winit/monitor/struct.VideoMode.html#method.refresh_rate_millihertz
    /// Then, we use that video mode to create the window in fullscreen mode.
    pub refresh_rate: u32,
}

#[derive(SystemSet, Clone, PartialEq, Eq, Hash, Debug)]
pub enum AppStartupStage {
    Startup,
}

#[derive(SystemSet, Clone, PartialEq, Eq, Hash, Debug)]
pub enum AppStage {
    EventUpdate,
    Update,
    UpdatePhysics,
    PostUpdate,
    Render,
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
                AppStage::EventUpdate,
                AppStage::Update,
                AppStage::UpdatePhysics,
                AppStage::PostUpdate,
                AppStage::Render,
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
        let mut world = &mut self.world;
        let schedule = &mut self.schedule;
        let startup_schedule = &mut self.startup_schedule;
        let config = &self.config;

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

        let context = Context::new(window_builder, &self.event_loop);

        // TODO: move to a more appropriate place
        let surface = context.surface();
        let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();

        // TODO: Do that only when the user clicks on the window, and undo it when he hits the escape button
        window
            .set_cursor_grab(CursorGrabMode::Confined)
            .or_else(|_e| window.set_cursor_grab(CursorGrabMode::Locked))
            .unwrap();
        window.set_cursor_visible(false);

        let renderer = Renderer::new(&context);

        let aspect_ratio = config.resolution.0 as f32 / config.resolution.1 as f32;

        let asset_server = AssetServer::new();
        world.insert_resource(asset_server);
        AssetServer::insert_asset_types(&mut world);

        let camera = Camera::new(Deg(60.0), aspect_ratio, 0.01, 100.0);
        let input_map = InputMap::new();

        let physics_context = PhysicsContext::new();

        world.insert_resource(physics_context);
        schedule.add_system(insert_collider_component.in_set(AppStage::Update));
        schedule.add_system(step_physics_simulation.in_set(AppStage::UpdatePhysics));
        schedule.add_system(step_character_controller.in_set(AppStage::PostUpdate));
        schedule.add_system(update_transform_system.in_set(AppStage::PostUpdate));

        world.insert_resource(Events::<input::MouseMovement>::default());
        schedule.add_system(
            Events::<input::MouseMovement>::update_system.in_set(AppStage::EventUpdate),
        );

        world.insert_resource(Events::<input::MouseInput>::default());
        schedule
            .add_system(Events::<input::MouseInput>::update_system.in_set(AppStage::EventUpdate));

        world.insert_resource(Events::<input::KeyboardInput>::default());
        schedule.add_system(
            Events::<input::KeyboardInput>::update_system.in_set(AppStage::EventUpdate),
        );

        world.insert_resource(Events::<input::WindowResize>::default());
        schedule
            .add_system(Events::<input::WindowResize>::update_system.in_set(AppStage::EventUpdate));

        schedule.add_system(input::handle_keyboard_input.in_set(AppStage::EventUpdate));
        schedule.add_system(input::handle_mouse_input.in_set(AppStage::EventUpdate));

        schedule.add_system(update_camera_aspect_ratio.in_set(AppStage::EventUpdate));
        schedule.add_system(update_camera.in_set(AppStage::PostUpdate));

        world.insert_resource(camera);
        world.insert_resource(input_map);

        world.insert_resource(context);
        world.insert_non_send_resource(renderer);
        schedule.add_system(render.in_set(AppStage::Render));

        let time = Time::new();
        world.insert_resource(time);

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
                    self.world.send_event(input::WindowResize { width, height });

                    self.world
                        .get_non_send_resource_mut::<Renderer>()
                        .unwrap()
                        .recreate_swapchain();
                }

                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(key_code),
                                state,
                                ..
                            },
                        ..
                    } => {
                        if key_code == VirtualKeyCode::Escape {
                            *control_flow = ControlFlow::Exit;
                        }

                        self.world
                            .send_event(input::KeyboardInput { key_code, state });
                    }
                    WindowEvent::MouseInput { button, state, .. } => {
                        self.world.send_event(input::MouseInput { button, state })
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
                    let mut time = self.world.get_resource_mut::<Time>().unwrap();
                    time.update();

                    self.schedule.run(&mut self.world);
                }

                _ => (),
            });
    }
}
