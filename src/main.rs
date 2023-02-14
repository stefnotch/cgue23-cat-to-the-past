use crate::application::{Application, Run};

mod camera;
mod input;
pub mod mesh;
mod context;
mod application;
mod render;

struct Game {

}

impl Run for Game {

}

impl Game {
    pub fn new() -> Game {
        Game {

        }
    }
}

fn main() {
    let game = Game::new();
    let application = Application::new();
    application.run(game);
}
