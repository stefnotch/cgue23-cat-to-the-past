use crate::application::{Application, GameState, Run};

mod application;
mod camera;
mod context;
mod input;
mod render;
mod scene;

struct Game {}

impl Run for Game {
    fn input(&self, _application: &mut GameState) {
        // todo!()
    }

    fn update(&self, _application: &mut GameState) {
        // todo!()
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
