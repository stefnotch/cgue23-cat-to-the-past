use crate::application::{AppConfig, AppStage, ApplicationBuilder};

mod application;
mod camera;
mod context;
mod input;
mod physics;
mod player;
mod render;
mod scene;

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

    let application = ApplicationBuilder::new(config)
        .with_startup_system(hello_world)
        .with_system(AppStage::Update, hello_stefan)
        .build();

    application.run();
}
