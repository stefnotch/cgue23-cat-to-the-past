use crate::application::{Application, GameState, Run};
use cgmath::Rad;

mod application;
mod camera;
mod context;
mod input;
mod render;
mod scene;

struct Game {}

impl Run for Game {
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
    let game = Game::new();
    let application = Application::new();
    application.run(game);
}
