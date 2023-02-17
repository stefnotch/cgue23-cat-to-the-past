use crate::camera::Camera;
use crate::context::Context;
use crate::input::InputMap;
use crate::render::Renderer;
use crate::scene::scene_graph::SceneGraph;
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, Event, KeyboardInput, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

pub struct Application {
    context: Context,
    event_loop: EventLoop<()>,
    game_state: GameState,
    renderer: Renderer,
}

// these are the application thingys that the game actually needs
pub struct GameState {
    input_map: InputMap,
    camera: Camera,
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
        self.event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }

                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                    self.game_state.camera.update_aspect_ratio();
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
                    DeviceEvent::MouseMotion { delta } => {}
                    _ => (),
                },

                Event::RedrawEventsCleared => {
                    runner.update(&mut self.game_state);
                    self.renderer.render(&self.context, &self.game_state);
                }

                _ => (),
            }
            // self.input_map.key_release(VirtualKeyCode::A);
        });
    }
}

pub trait Run {
    fn input(&self, _application: &mut GameState);

    fn update(&self, _application: &mut GameState);
}
