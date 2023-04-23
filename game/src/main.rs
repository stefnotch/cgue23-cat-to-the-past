use bevy_ecs::prelude::{Component, Query, With};
use game::scene::loader::AssetServer;
use game_core::time_manager::TimeManager;
use game_core::{asset::AssetId, time::Time};
use scene::{
    material::CpuMaterial,
    mesh::CpuMesh,
    model::{CpuPrimitive, Model},
};
use std::f32::consts::PI;
use std::sync::Arc;
use std::time::Instant;

use bevy_ecs::system::{Commands, Res};
use nalgebra::{Point3, Translation3, Vector3};

use game::core::application::{AppConfig, ApplicationBuilder};

use debug::tracing::start_tracing;

use game::player::{PlayerControllerSettings, PlayerSpawnSettings};
use physics::physics_context::{BoxCollider, MoveBodyPosition, RigidBody, RigidBodyType};
use scene::transform::{Transform, TransformBuilder};

fn _spawn_pbr_demo(mut commands: Commands) {
    let sphere = CpuMesh::sphere(64, 32, 1.0);

    // TODO: add missing lights

    let spacing: f32 = 1.25;

    let n = 7;

    for row in 0..n {
        let metallic: f32 = row as f32 / (n as f32 - 1.0);
        for col in 0..n {
            let roughness: f32 = col as f32 / (n as f32 - 1.0);

            commands.spawn((
                Model {
                    primitives: vec![CpuPrimitive {
                        mesh: sphere.clone(),
                        material: Arc::new(CpuMaterial {
                            id: AssetId::new_v4(),
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

fn spawn_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    let before = Instant::now();
    asset_server
        .load_default_scene(
            "./assets/scene/testing/prototype/prototype.glb",
            &mut commands,
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

pub fn spawn_moving_cube(mut commands: Commands) {
    let cube = CpuMesh::cube(1.0, 1.0, 1.0);

    commands.spawn((
        Transform::default(),
        Model {
            primitives: vec![CpuPrimitive {
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
            1.0,
            4.0 * (time.level_time_seconds() * PI / 2.0 * 0.5).sin(),
        );
        move_body_position.new_position = shift.transform_point(&origin);
    }
}

fn main() {
    let _guard = start_tracing();

    // TODO: remove this
    std::env::set_var("RUST_BACKTRACE", "1");

    // TODO: read from file
    let config = AppConfig::default();

    let player_controller_settings = PlayerControllerSettings::new(5.0, 1.0, 9.81);

    let player_spawn_settings = PlayerSpawnSettings {
        initial_transform: TransformBuilder::new()
            .position([0.0, 5.0, 0.0].into())
            .build(),
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
