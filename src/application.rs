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
use winit::window::{CursorGrabMode, Window, WindowBuilder};

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
    scene_graph: SceneGraph,
}

impl Application {
    pub fn new() -> Application {
        let event_loop = EventLoop::new();

        let window_builder = WindowBuilder::new()
            .with_inner_size(LogicalSize {
                width: 800,
                height: 800,
            })
            .with_title("CG Project");

        let context = Context::new(window_builder, &event_loop);

        let renderer = Renderer::new(&context);

        // TODO: calculate aspect ratio (assume 800x800 window for now)

        let game_state = GameState {
            input_map: InputMap::new(),
            camera: Camera::new(60.0, 1.0, 0.01, 100.0),
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
        let mut last_frame = Instant::now();

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
                        self.game_state.input_map.mouse_move(delta);
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
    fn input(&self, state: &mut GameState);

    fn update(&self, state: &mut GameState, delta_time: f64);
}
