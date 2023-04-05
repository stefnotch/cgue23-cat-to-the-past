use std::sync::Arc;

use bevy_ecs::system::{Commands, Res};
use cat_to_the_past::render::context::Context;
use nalgebra::Point3;
use rapier3d::na::Vector3;

use cat_to_the_past::core::application::{AppConfig, ApplicationBuilder};
use cat_to_the_past::player::{PlayerControllerSettings, PlayerSpawnSettings};
use cat_to_the_past::scene::material::Material;
use cat_to_the_past::scene::mesh::Mesh;
use cat_to_the_past::scene::model::{Model, Primitive};
use cat_to_the_past::scene::transform::TransformBuilder;

fn spawn_pbr_demo(mut commands: Commands, context: Res<Context>) {
    let memory_allocator = Arc::new(
        vulkano::memory::allocator::StandardMemoryAllocator::new_default(context.device()),
    );

    let sphere = Mesh::sphere(64, 32, 1.0, &memory_allocator);

    // TODO: add missing lights

    let spacing: f32 = 1.25;

    let n = 7;

    for row in 0..n {
        let metallic: f32 = row as f32 / (n as f32 - 1.0);
        for col in 0..n {
            let roughness: f32 = col as f32 / (n as f32 - 1.0);

            commands.spawn((
                Model {
                    primitives: vec![Primitive {
                        mesh: sphere.clone(),
                        material: Arc::new(Material {
                            base_color: Vector3::new(1.0, 0.0, 0.0),
                            base_color_texture: None,
                            roughness_factor: roughness,
                            metallic_factor: metallic,
                            emissivity: Default::default(),
                        }),
                    }],
                },
                TransformBuilder::new()
                    .scale(Vector3::new(0.5, 0.5, 0.5))
                    .position(Point3::new(
                        (col - n / 2) as f32 * spacing,
                        (row - n / 2) as f32 * spacing,
                        0.0,
                    ))
                    .build(),
            ));
        }
    }
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    let config = AppConfig {
        resolution: (1280, 720),
        fullscreen: false,
        brightness: 1.0,
        refresh_rate: 60,
    };

    let player_controller_settings = PlayerControllerSettings::new(5.0, 1.0, 9.81);

    let player_spawn_settings = PlayerSpawnSettings {
        initial_transform: Default::default(),
        controller_settings: player_controller_settings,
        free_cam_activated: true,
    };

    let application = ApplicationBuilder::new(config)
        .with_startup_system(spawn_pbr_demo)
        .with_player_controller(player_spawn_settings)
        .build();

    application.run();
}
