use std::sync::Arc;

use bevy_ecs::system::{Commands, Res};
use context::Context;
use rapier3d::na::Vector3;
use scene::material::Material;
use scene::mesh::Mesh;
use scene::model::Model;
use scene::transform::Transform;

use crate::application::{AppConfig, ApplicationBuilder};
use crate::physics_context::{BoxCollider, RapierRigidBody};
use crate::player::PlayerSettings;

mod application;
mod camera;
mod context;
mod input;
mod physics_context;
mod player;
mod render;
mod scene;
mod time;

fn spawn_world(mut commands: Commands, context: Res<Context>) {
    let memory_allocator = Arc::new(
        vulkano::memory::allocator::StandardMemoryAllocator::new_default(context.device()),
    );

    let cube = Mesh::cube(0.5, 0.5, 0.5, &memory_allocator);
    let mut cube_transform = Transform::new();
    cube_transform.position = cgmath::Vector3::new(0.0, 3.5, 0.0);

    commands.spawn((
        Model {
            mesh: cube,
            material: Arc::new(Material {}),
        },
        cube_transform,
        BoxCollider {
            size: Vector3::new(0.5, 0.5, 0.5),
        },
        RapierRigidBody { handle: None },
    ));

    let platform = Mesh::cube(20.0, 0.1, 20.0, &memory_allocator);
    let mut plane_transform = Transform::new();
    plane_transform.position = cgmath::Vector3::new(0.0, -0.5, 0.0);

    commands.spawn((
        Model {
            mesh: platform,
            material: Arc::new(Material {}),
        },
        plane_transform,
        BoxCollider {
            size: Vector3::new(20.0, 0.1, 20.0),
        },
    ));
}

fn main() {
    // TODO: remove this
    std::env::set_var("RUST_BACKTRACE", "1");

    // TODO: read from file
    let config = AppConfig {
        resolution: (1280, 720),
        fullscreen: false,
        brightness: 1.0,
        refresh_rate: 60,
    };

    let player_settings = PlayerSettings::new(5.0, 0.5);

    let application = ApplicationBuilder::new(config)
        .with_startup_system(spawn_world)
        .with_player_controller(player_settings)
        .build();

    application.run();
}
