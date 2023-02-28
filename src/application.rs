use crate::camera::Camera;
use crate::context::Context;
use crate::input;
use crate::input::{InputMap, MouseMovement};
use crate::render::Renderer;
use crate::time::Time;
use bevy_ecs::prelude::*;
use std::time::Instant;
use winit::dpi;
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, ElementState, Event, KeyboardInput, WindowEvent};
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

#[derive(StageLabel)]
pub enum AppStartupStage {
    Startup,
}

#[derive(StageLabel)]
pub enum AppStage {
    EventUpdate,
    Update,
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
        let startup_schedule = Schedule::default()
            .with_stage(AppStartupStage::Startup, SystemStage::single_threaded());

        let schedule = Schedule::default()
            .with_stage(AppStage::EventUpdate, SystemStage::single_threaded())
            .with_stage(AppStage::Update, SystemStage::single_threaded())
            .with_stage(AppStage::PostUpdate, SystemStage::single_threaded())
            .with_stage(AppStage::Render, SystemStage::single_threaded());

        let world = World::new();

        ApplicationBuilder {
            config,
            startup_schedule,
            schedule,
            world,
        }
    }

    pub fn with_system<Params>(
        mut self,
        stage: AppStage,
        system: impl IntoSystemDescriptor<Params>,
    ) -> Self {
        self.schedule.add_system_to_stage(stage, system);

        self
    }

    pub fn with_startup_system<Params>(
        mut self,
        system: impl IntoSystemDescriptor<Params>,
    ) -> Self {
        self.startup_schedule
            .add_system_to_stage(AppStartupStage::Startup, system);

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
    context: Context,
    event_loop: EventLoop<()>,
    renderer: Renderer,

    world: World,
    schedule: Schedule,
}

impl Application {
    fn new(
        config: AppConfig,
        mut startup_schedule: Schedule,
        mut schedule: Schedule,
        mut world: World,
    ) -> Application {
        let event_loop = EventLoop::new();

        let monitor = event_loop
            .available_monitors()
            .next()
            .expect("no monitor found!");

        let mut window_builder = WindowBuilder::new()
            .with_inner_size(LogicalSize {
                width: config.resolution.0,
                height: config.resolution.1,
            })
            .with_title("CG Project");

        if config.fullscreen {
            if let Some(video_mode) = monitor
                .video_modes()
                .filter(|v| v.refresh_rate_millihertz() == config.refresh_rate * 1000)
                .next()
            {
                window_builder = window_builder.with_fullscreen(Some(Exclusive(video_mode)))
            }
        }

        let context = Context::new(window_builder, &event_loop);

        // TODO: move to a more appropriate place
        let surface = context.surface();
        let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();

        window
            .set_cursor_grab(CursorGrabMode::Confined)
            .or_else(|_e| window.set_cursor_grab(CursorGrabMode::Locked))
            .unwrap();

        window.set_cursor_visible(false);

        let renderer = Renderer::new(&context);

        let aspect_ratio = config.resolution.0 as f32 / config.resolution.1 as f32;

        let camera = Camera::new(60.0, aspect_ratio, 0.01, 100.0);
        let input_map = InputMap::new();
        let time = Time::new();

        schedule.add_system_to_stage(
            AppStage::EventUpdate,
            Events::<MouseMovement>::update_system,
        );

        schedule.add_system_to_stage(
            AppStage::EventUpdate,
            Events::<input::KeyboardInput>::update_system,
        );

        schedule.add_system_to_stage(AppStage::EventUpdate, input::handle_keyboard_input);

        world.insert_resource(camera);
        world.insert_resource(input_map);
        world.insert_resource(time);

        startup_schedule.run(&mut world);

        Application {
            context,
            event_loop,
            renderer,

            world,
            schedule,
        }
    }

    pub fn run(mut self)
    where
        Self: 'static,
    {
        let mut last_frame = Instant::now();

        self.event_loop
            .run(move |event, _, control_flow| match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }

                Event::WindowEvent {
                    event: WindowEvent::Resized(dpi::PhysicalSize { width, height }),
                    ..
                } => {
                    let new_aspect_ratio = width as f32 / height as f32;

                    self.renderer.recreate_swapchain();
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
                        self.world
                            .send_event(input::KeyboardInput { key_code, state });

                        match state {
                            ElementState::Pressed => {
                                // self.game_state.input_map.update_key_press(key_code)
                            }
                            ElementState::Released => {
                                // self.game_state.input_map.update_key_release(key_code)
                            }
                        }
                    }
                    WindowEvent::MouseInput { button, state, .. } => {}
                    _ => (),
                },

                Event::DeviceEvent { event, .. } => match event {
                    DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                        self.world.send_event(MouseMovement(dx, dy))
                        // self.game_state.input_map.update_mouse_move(delta);
                    }
                    _ => (),
                },

                Event::RedrawEventsCleared => {
                    let delta_time = last_frame.elapsed().as_secs_f64();
                    last_frame = Instant::now();

                    let time = self.world.get_resource_mut::<Time>().unwrap();
                    time.delta_seconds = delta_time;

                    self.schedule.run(&mut self.world);
                }

                _ => (),
            });
    }
}
