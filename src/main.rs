use crate::application::{AppConfig, Application, GameState, Run};
use cgmath::Rad;

mod application;
mod camera;
mod context;
mod input;
mod render;
mod scene;

struct Game {}

impl Run for Game {
    fn init(&self, state: &mut GameState) {
        /*let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(context.device()));
        let cube = Mesh::cube(0.5, 0.5, 0.5, &memory_allocator);

        state.scene_graph.add(cube);*/
    }

    fn input(&self, _state: &mut GameState) {}

    fn update(&self, state: &mut GameState, _delta_time: f64) {
        let (dx, dy) = state.input_map.mouse_delta();
        state.camera.yaw += Rad(dx as f32 * 0.005);
        state.camera.pitch += Rad(dy as f32 * 0.005);
    }
}

impl Game {
    pub fn new() -> Game {
        Game {}
    }
}

fn main() {
    // TODO: read from file
    let config = AppConfig {
        resolution: (800, 800),
        fullscreen: false,
        brightness: 1.0,
        refresh_rate: 60, // ignored for now
    };

    let game = Game::new();
    let application = Application::new(&config);
    application.run(game);
}
