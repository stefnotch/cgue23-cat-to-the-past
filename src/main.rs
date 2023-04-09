use bevy_ecs::prelude::{Component, Query, With};
use cat_to_the_past::core::time_manager::TimeManager;
use cat_to_the_past::scene::loader::AssetServer;
use std::f32::consts::PI;
use std::sync::Arc;
use std::time::Instant;

use bevy_ecs::system::{Commands, Res};
use cat_to_the_past::render::context::Context;
use nalgebra::{Point3, Translation3};
use rapier3d::dynamics::RigidBodyType;
use rapier3d::na::Vector3;

use cat_to_the_past::core::application::{AppConfig, ApplicationBuilder};

#[cfg(feature = "trace")]
use cat_to_the_past::debug::tracing::start_tracing;

use cat_to_the_past::core::time::Time;
use cat_to_the_past::physics::physics_context::{BoxCollider, MoveBodyPosition, RigidBody};
use cat_to_the_past::player::{PlayerControllerSettings, PlayerSpawnSettings};
use cat_to_the_past::scene::material::Material;
use cat_to_the_past::scene::mesh::Mesh;
use cat_to_the_past::scene::model::{Model, Primitive};
use cat_to_the_past::scene::transform::{Transform, TransformBuilder};

fn _spawn_pbr_demo(mut commands: Commands, context: Res<Context>) {
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

fn spawn_world(mut commands: Commands, context: Res<Context>, asset_server: Res<AssetServer>) {
    let memory_allocator = Arc::new(
        vulkano::memory::allocator::StandardMemoryAllocator::new_default(context.device()),
    );

    let before = Instant::now();
    asset_server
        .load_default_scene(
            "./assets/scene/testing/bloom test/bloom.glb",
            &mut commands,
            &memory_allocator,
            &context,
        )
        .unwrap();
    println!(
        "Loading the scene took {}sec",
        before.elapsed().as_secs_f64()
    );
}

fn _print_fps(time: Res<Time>) {
    println!("{}", 1.0 / time.delta_seconds())
}

#[derive(Component)]
pub struct MovingBox;

pub fn spawn_moving_cube(mut commands: Commands, context: Res<Context>) {
    let memory_allocator = Arc::new(
        vulkano::memory::allocator::StandardMemoryAllocator::new_default(context.device()),
    );

    let cube = Mesh::cube(1.0, 1.0, 1.0, &memory_allocator);

    commands.spawn((
        Transform::default(),
        Model {
            primitives: vec![Primitive {
                mesh: cube.clone(),
                material: Arc::new(Default::default()),
            }],
        },
        BoxCollider {
            bounds: cube.bounding_box.clone(),
        },
        RigidBody(RigidBodyType::KinematicPositionBased),
        MoveBodyPosition {
            new_position: Default::default(),
        },
        MovingBox,
    ));
}

pub fn move_cubes(
    mut query: Query<&mut MoveBodyPosition, With<MovingBox>>,
    time: Res<TimeManager>,
) {
    let origin = Point3::origin();
    for mut move_body_position in query.iter_mut() {
        let shift = Translation3::new(
            0.0,
            0.0,
            4.0 * (time.level_time_seconds() * PI / 2.0 * 0.5).sin(),
        );
        move_body_position.new_position = shift.transform_point(&origin);
    }
}

fn main() {
    #[cfg(feature = "trace")]
    let _guard = start_tracing();

    // TODO: remove this
    std::env::set_var("RUST_BACKTRACE", "1");

    // TODO: read from file
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
        free_cam_activated: false,
    };

    let application = ApplicationBuilder::new(config)
        .with_startup_system(spawn_world)
        .with_startup_system(spawn_moving_cube)
        .with_player_controller(player_spawn_settings)
        .with_system(move_cubes)
        // .with_system(print_fps)
        // .with_system(rotate_entites)
        .build();

    application.run();
}
