use crate::camera::Camera;
use crate::context::Context;
use crate::input::InputMap;
use crate::render::Renderer;
use crate::scene::scene_graph::SceneGraph;
use std::time::Instant;
use winit::dpi;
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, Event, KeyboardInput, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Fullscreen::Exclusive;
use winit::window::WindowBuilder;

pub struct AppConfig {
    pub resolution: (u32, u32),
    pub fullscreen: bool,
    /// Projectors are usually very dark, this parameter should control how bright your total
    /// scene is, e.g., an illumination multiplier
    pub brightness: f32,
    // TODO: not really sure how to implement this
    pub refresh_rate: u32,
}

pub struct Application {
    context: Context,
    event_loop: EventLoop<()>,
    game_state: GameState,
    renderer: Renderer,
}

// these are the application thingys that the game actually needs
pub struct GameState {
    pub input_map: InputMap,
    pub camera: Camera,
    pub scene_graph: SceneGraph,
}

impl Application {
    pub fn new(config: &AppConfig) -> Application {
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
            if let Some(video_mode) = monitor.video_modes().next() {
                window_builder = window_builder.with_fullscreen(Some(Exclusive(video_mode)))
            }
        }

        let context = Context::new(window_builder, &event_loop);

        let renderer = Renderer::new(&context);

        let aspect_ratio = config.resolution.0 as f32 / config.resolution.1 as f32;

        let game_state = GameState {
            input_map: InputMap::new(),
            camera: Camera::new(60.0, aspect_ratio, 0.01, 100.0),
            scene_graph: SceneGraph::new(),
        };

        Application {
            context,
            event_loop,
            game_state,
            renderer,
        }
    }

    pub fn run<T>(mut self, runner: T)
    where
        T: Run + 'static,
        Self: 'static,
    {
        runner.init(&mut self.game_state);

        let mut last_frame = Instant::now();

        // Dummy code to test the renderer
        let memory_allocator = std::sync::Arc::new(
            vulkano::memory::allocator::StandardMemoryAllocator::new_default(self.context.device()),
        );
        let cube = crate::scene::mesh::Mesh::cube(0.5, 0.5, 0.5, &memory_allocator);
        self.game_state
            .scene_graph
            .add(crate::scene::scene_graph::Model {
                mesh: cube,
                material: std::sync::Arc::new(crate::scene::material::Material {}),
            });

        self.event_loop.run(move |event, _, control_flow| {
            match event {
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

                    self.game_state.camera.update_aspect_ratio(new_aspect_ratio);
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
                    } => {}
                    WindowEvent::MouseInput { button, state, .. } => {}
                    _ => (),
                },

                Event::DeviceEvent { event, .. } => match event {
                    DeviceEvent::MouseMotion { delta } => {
                        self.game_state.input_map.update_mouse_move(delta);
                    }
                    _ => (),
                },

                Event::RedrawEventsCleared => {
                    let delta_time = last_frame.elapsed().as_secs_f64();
                    last_frame = Instant::now();

                    // println!("Deltatime: {dt}");

                    self.game_state.camera.update();
                    runner.update(&mut self.game_state, delta_time);
                    self.renderer.render(&self.context, &self.game_state);
                    self.game_state.input_map.new_frame();
                }

                _ => (),
            }
            // self.input_map.key_release(VirtualKeyCode::A);
        });
    }
}

pub trait Run {
    fn init(&self, state: &mut GameState);

    fn input(&self, state: &mut GameState);

    fn update(&self, state: &mut GameState, delta_time: f64);
}
