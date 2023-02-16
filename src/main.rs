use crate::application::{Application, Run};

mod application;
mod camera;
mod context;
mod input;
mod render;
mod scene;

struct Game {}

impl Run for Game {}

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
