use animations::animation::AnimationPlugin;
use app::plugin::Plugin;
use app::App;
use input::plugin::InputPlugin;
use levels::LevelsPlugin;
use loader::config_loader::LoadableConfig;
use physics::plugin::PhysicsPlugin;
use time::time::{Time, TimePlugin, TimePluginSet};
use windowing::window::{EventLoopContainer, WindowPlugin};

use crate::pickup_system::PickupPlugin;
use crate::player::{PlayerPlugin, PlayerPluginSets};
use angle::Deg;
use bevy_ecs::prelude::*;
use input::events::{KeyboardInput, MouseInput, MouseMovement};
use loader::loader::SceneLoader;
use nalgebra::{Point3, UnitQuaternion};
use render::context::Context;
use render::{Renderer, RendererPlugin, RendererPluginSets, ViewFrustumCullingMode};
use scene::camera::{update_camera, Camera};
use windowing::config::WindowConfig;
use windowing::dpi::PhysicalSize;
use windowing::event::{
    DeviceEvent, Event, KeyboardInput as KeyboardInputWinit, VirtualKeyCode, WindowEvent,
};
use windowing::event_loop::ControlFlow;
use windowing::events::{WindowFocusChanged, WindowResize};

use windowing::window::CursorGrabMode;

use crate::core::transform_change::{
    time_manager_rewind_transform, time_manager_track_transform, TransformChange,
};
use time::time_manager::game_change::GameChangeHistoryPlugin;
use time::time_manager::{TimeManagerPlugin, TimeManagerPluginSet};
use windowing::event::ElementState::Released;
use windowing::event::VirtualKeyCode::F8;

use super::transform_change::time_manager_start_track_transform;

pub struct AppConfig {
    pub window: WindowConfig,
    /// Projectors are usually very dark, this parameter should control how bright your total
    /// scene is, e.g., an illumination multiplier
    pub brightness: f32,
    pub mouse_sensitivity: f32,
}

impl From<LoadableConfig> for AppConfig {
    fn from(config: LoadableConfig) -> Self {
        Self {
            window: WindowConfig {
                resolution: config.resolution,
                fullscreen: config.fullscreen,
                refresh_rate: config.refresh_rate,
            },
            brightness: config.brightness,
            mouse_sensitivity: config.mouse_sensitivity,
        }
    }
}

#[derive(SystemSet, Clone, PartialEq, Eq, Hash, Debug)]
pub enum AppStage {
    StartFrame,
    EventUpdate,
    /// for engine logic that depends on events
    BeforeUpdate,
    /// for game logic
    Update,
    /// for level specific game logic
    UpdateLevel,
    UpdatePhysics,
    /// after physics
    BeforeRender,
    Render,
    EndFrame,
}

pub struct Application {
    config: AppConfig,
    pub app: App,
}

impl Application {
    pub fn new(config: AppConfig) -> Self {
        let mut app = App::new();
        app.schedule.configure_sets(
            (
                AppStage::StartFrame,
                AppStage::EventUpdate,
                AppStage::BeforeUpdate,
                AppStage::Update,
                AppStage::UpdateLevel,
                AppStage::UpdatePhysics,
                AppStage::BeforeRender,
                AppStage::Render,
                AppStage::EndFrame,
            )
                .chain(),
        );

        Self::add_default_plugins(&mut app, &config);

        Self { config, app }
    }

    fn add_default_plugins(app: &mut App, config: &AppConfig) {
        app //
            .with_plugin(TimePlugin)
            .with_set(TimePluginSet::UpdateTime.in_set(AppStage::StartFrame))
            .with_plugin(LevelsPlugin)
            .with_set(LevelsPlugin::system_set().in_set(AppStage::StartFrame))
            .with_plugin(TimeManagerPlugin)
            .with_set(
                TimeManagerPluginSet::StartFrame
                    .in_set(AppStage::StartFrame)
                    .after(TimePluginSet::UpdateTime)
                    .after(LevelsPlugin::system_set()),
            )
            .with_plugin(InputPlugin)
            .with_set(InputPlugin::system_set().in_set(AppStage::EventUpdate))
            .with_plugin(AnimationPlugin)
            .with_set(
                AnimationPlugin::system_set()
                    .after(AppStage::UpdateLevel)
                    .before(AppStage::UpdatePhysics),
            )
            .with_plugin(PhysicsPlugin)
            .with_set(PhysicsPlugin::system_set().in_set(AppStage::UpdatePhysics))
            // Transform tracking
            .with_plugin(
                GameChangeHistoryPlugin::<TransformChange>::new()
                    .with_tracker(time_manager_start_track_transform)
                    .with_tracker(
                        time_manager_track_transform.after(time_manager_start_track_transform),
                    )
                    .with_rewinder(time_manager_rewind_transform),
            )
            .with_set(
                GameChangeHistoryPlugin::<TransformChange>::system_set()
                    .after(AppStage::UpdateLevel)
                    .after(AnimationPlugin::system_set())
                    .before(AppStage::UpdatePhysics),
            )
            .with_plugin(WindowPlugin::new(config.window.clone()))
            .with_plugin(RendererPlugin::new(config.brightness))
            .with_set(RendererPluginSets::Render.in_set(AppStage::Render))
            // Configuring the player plugin (but not adding it)
            .with_set(PlayerPluginSets::UpdateInput.in_set(AppStage::BeforeUpdate))
            .with_set(PlayerPluginSets::Update.in_set(AppStage::BeforeUpdate))
            .with_set(PlayerPluginSets::UpdateCamera.in_set(AppStage::BeforeRender))
            .with_set(
                PickupPlugin::system_set()
                    .in_set(AppStage::BeforeUpdate)
                    .after(PlayerPluginSets::Update),
            );
    }

    pub fn run(mut self)
    where
        Self: 'static,
    {
        self.app.build_plugins();

        let config: &AppConfig = &self.config;
        let world = &mut self.app.world;
        let schedule = &mut self.app.schedule;

        let aspect_ratio = config.window.resolution.0 as f32 / config.window.resolution.1 as f32;

        let scene_loader = SceneLoader::new();
        world.insert_resource(scene_loader);

        let camera = Camera::new(
            Point3::origin(), // Note: The player updates this
            UnitQuaternion::identity(),
            aspect_ratio,
            Deg(60.0),
            0.01,
            100.0,
        );
        schedule.add_system(
            update_camera_aspect_ratio
                .after(AppStage::EventUpdate)
                .before(AppStage::BeforeUpdate),
        );
        schedule.add_system(
            update_camera
                .in_set(AppStage::BeforeRender)
                .after(PlayerPlugin::system_set()),
        );
        world.insert_resource(camera);

        world.insert_resource(Events::<WindowResize>::default());
        schedule.add_system(Events::<WindowResize>::update_system.in_set(AppStage::EventUpdate));

        world.insert_resource(Events::<WindowFocusChanged>::default());
        schedule
            .add_system(Events::<WindowFocusChanged>::update_system.in_set(AppStage::EventUpdate));

        schedule.add_system(lock_mouse.in_set(AppStage::BeforeUpdate));

        schedule.add_system(update_view_frustum_culling_enabled.in_set(AppStage::BeforeUpdate));

        self.app.run_startup();
        // Reset time after startup
        self.app.world.get_resource_mut::<Time>().unwrap().update();

        self.app
            .world
            .remove_non_send_resource::<EventLoopContainer>()
            .unwrap()
            .event_loop
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
                    self.app.world.clear_trackers(); // Needs to be called for "RemovedComponents" to work properly
                }

                _ => (),
            });
    }
}

fn lock_mouse(context: NonSend<Context>, mut event: EventReader<WindowFocusChanged>) {
    for WindowFocusChanged { has_focus } in event.into_iter() {
        let window = context.window();

        // TODO: Don't aggressively grab the cursor, instead only grab it when the user actually clicked on the window

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

fn update_view_frustum_culling_enabled(
    mut view_frustum_culling_mode: ResMut<ViewFrustumCullingMode>,
    mut event_reader: EventReader<KeyboardInput>,
) {
    for event in event_reader.iter() {
        if event.key_code == F8 && event.state == Released {
            view_frustum_culling_mode.enabled = !view_frustum_culling_mode.enabled;
        }
    }
}

fn update_camera_aspect_ratio(mut camera: ResMut<Camera>, mut reader: EventReader<WindowResize>) {
    for event in reader.iter() {
        camera.update_aspect_ratio(event.width as f32 / event.height as f32);
    }
}
