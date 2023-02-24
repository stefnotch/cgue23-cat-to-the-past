use crate::application::{AppConfig, Application, GameState, Run};
use crate::camera::PlayerController;
use cgmath::Rad;

mod application;
mod camera;
mod context;
mod input;
mod physics;
mod render;
mod scene;

struct Game {
    player_controller: PlayerController,
}

impl Run for Game {
    fn init(&self, state: &mut GameState) {
        // setup scene

        /*let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(context.device()));
        let cube = Mesh::cube(0.5, 0.5, 0.5, &memory_allocator);

        state.scene_graph.add(cube);*/
    }

    fn input(&self, _state: &mut GameState) {}

    fn update(&mut self, state: &mut GameState, delta_time: f64) {
        self.player_controller
            .update_camera(&mut state.camera, &state.input_map, delta_time);
    }
}

impl Game {
    pub fn new() -> Game {
        let controller = PlayerController::new(5.0, 0.5);
        Game {
            player_controller: controller,
        }
    }
}

fn main() {
    // TODO: read from file
    let config = AppConfig {
        resolution: (800, 800),
        fullscreen: false,
        brightness: 1.0,
        refresh_rate: 60,
    };

    let application = Application::new(&config);

    let game = Game::new();
    application.run(game);
}
