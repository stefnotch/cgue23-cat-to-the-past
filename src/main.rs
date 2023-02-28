use crate::application::{AppConfig, AppStage, ApplicationBuilder};
use crate::player::PlayerSettings;

mod application;
mod camera;
mod context;
mod input;
mod physics;
mod player;
mod render;
mod scene;
mod time;

fn hello_world() {
    println!("Hello world");
}

fn hello_stefan() {
    println!("Hallo Stefan!");
}

fn main() {
    // TODO: read from file
    let config = AppConfig {
        resolution: (800, 800),
        fullscreen: false,
        brightness: 1.0,
        refresh_rate: 60,
    };

    let player_settings = PlayerSettings::new(5.0, 0.5);

    let application = ApplicationBuilder::new(config)
        .with_startup_system(hello_world)
        .with_system(AppStage::Update, hello_stefan)
        .with_player_controller(player_settings)
        .build();

    application.run();
}
