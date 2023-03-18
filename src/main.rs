use bevy_ecs::query::Without;
use scene::loader::AssetServer;
use std::sync::Arc;
use std::time::Instant;

use bevy_ecs::system::{Commands, Query, Res};
use context::Context;
use nalgebra::{Translation3, UnitQuaternion};
use rapier3d::na::Vector3;

use crate::application::{AppConfig, ApplicationBuilder};
use crate::physics_context::BoxCollider;
use crate::player::PlayerSettings;
use crate::scene::light::{Light, PointLight};
use crate::scene::material::Material;
use crate::scene::mesh::Mesh;
use crate::scene::model::{Model, Primitive};
use crate::scene::transform::{Transform, TransformBuilder};
use crate::time::Time;

mod application;
mod camera;
mod context;
mod input;
mod physics_context;
mod player;
mod render;
mod scene;
mod time;
mod time_manager;

fn spawn_world(mut commands: Commands, context: Res<Context>, asset_server: Res<AssetServer>) {
    let memory_allocator = Arc::new(
        vulkano::memory::allocator::StandardMemoryAllocator::new_default(context.device()),
    );

    let sphere = Mesh::sphere(64, 32, 1.0, &memory_allocator);

    commands.spawn((
        Model {
            primitives: vec![Primitive {
                mesh: sphere.clone(),
                material: Arc::new(Material {
                    base_color: Vector3::new(1.0, 1.0, 1.0),
                    base_color_texture: None,
                    roughness_factor: 1.0,
                    metallic_factor: 0.0,
                    emissivity: Default::default(),
                }),
            }],
        },
        Light::Point(PointLight {
            color: Vector3::new(1.0, 1.0, 1.0),
            range: 0.0,
            intensity: 100.0,
        }),
        TransformBuilder::new()
            .translation(Translation3::new(0.0, 0.0, 10.0))
            .scale(Vector3::new(0.05, 0.05, 0.05))
            .build(),
    ));

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
                    .translation(Translation3::new(
                        (col - n / 2) as f32 * spacing,
                        (row - n / 2) as f32 * spacing,
                        0.0,
                    ))
                    .build(),
            ));
        }
    }

    // let before = Instant::now();
    // asset_server
    //     .load_default_scene(
    //         "./assets/scene/testing/sponza/sponza.glb",
    //         &mut commands,
    //         &memory_allocator,
    //         &context,
    //     )
    //     .unwrap();
    // println!(
    //     "Loading the scene took {}sec",
    //     before.elapsed().as_secs_f64()
    // );

    // commands.spawn((
    //     Model {
    //         primitives: vec![Primitive {
    //             mesh: sphere.clone(),
    //             material: Arc::new(Material {
    //                 base_color: Vector3::new(1.0, 0.0, 0.0),
    //                 base_color_texture: None,
    //                 roughness_factor: 0.8,
    //                 metallic_factor: 0.1,
    //                 emissivity: Default::default(),
    //             }),
    //         }],
    //     },
    //     Transform::default(),
    // ));

    // let cube = Mesh::cube(1.0, 1.0, 1.0, &memory_allocator);
    //
    // // let center = Vector3::new(4.07, 5.90, -1.01);
    // let center: Vector3<f32> = Vector3::zeros();
    //
    // for i in 0..32 {
    //     let angle = i as f32 * (2.0 * PI) / 32.0;
    //     let (sin, cos) = angle.sin_cos();
    //     commands.spawn((
    //         Model {
    //             primitives: vec![Primitive {
    //                 mesh: cube.clone(),
    //                 material: Arc::new(Material {
    //                     color: Vector3::new(1.0, 1.0, 1.0),
    //                     ka: 0.0,
    //                     kd: 1.0,
    //                     ks: 0.0,
    //                     alpha: 1.0,
    //                 }),
    //             }],
    //         },
    //         TransformBuilder::new()
    //             .translation(Translation3::new(
    //                 center.x + cos * 5.0,
    //                 center.y + 1.0,
    //                 center.z + sin * 5.0,
    //             ))
    //             .scale(Vector3::new(0.5, 0.5, 0.5))
    //             .build(),
    //     ));
    // }
    //
    // commands.spawn((
    //     Model {
    //         primitives: vec![Primitive {
    //             mesh: cube.clone(),
    //             material: Arc::new(Material {
    //                 color: Vector3::new(0.0, 1.0, 0.0),
    //                 ka: 0.0,
    //                 kd: 0.9,
    //                 ks: 0.3,
    //                 alpha: 10.0,
    //             }),
    //         }],
    //     },
    //     TransformBuilder::new()
    //         .translation(Translation3::new(0.0, 0.25, 0.0))
    //         .rotation(UnitQuaternion::from_axis_angle(
    //             &Vector3::y_axis(),
    //             PI / 2.0,
    //         ))
    //         .scale(Vector3::new(0.5, 0.5, 0.5))
    //         .build(),
    //     BoxCollider {
    //         size: Vector3::new(0.5, 0.5, 0.5),
    //     },
    //     RapierRigidBody { handle: None },
    // ));
    //
    // commands.spawn((
    //     Model {
    //         primitives: vec![Primitive {
    //             mesh: cube.clone(),
    //             material: Arc::new(Material {
    //                 color: Vector3::new(1.0, 0.0, 0.0),
    //                 ka: 0.0,
    //                 kd: 0.9,
    //                 ks: 0.0,
    //                 alpha: 10.0,
    //             }),
    //         }],
    //     },
    //     TransformBuilder::new()
    //         .translation(Translation3::new(0.0, -0.5, 0.0))
    //         .scale(Vector3::new(20.0, 0.1, 20.0))
    //         .build(),
    //     BoxCollider {
    //         size: Vector3::new(20.0, 0.1, 20.0),
    //     },
    // ));
}

fn _rotate_entites(mut query: Query<&mut Transform, Without<BoxCollider>>) {
    for mut transform in query.iter_mut() {
        transform.rotation *= UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.05);
    }
}

fn _print_fps(time: Res<Time>) {
    println!("{}", 1.0 / time.delta_seconds())
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

    let player_settings = PlayerSettings::new(5.0, 1.0, 9.81);

    let application = ApplicationBuilder::new(config)
        .with_startup_system(spawn_world)
        // .with_system(print_fps)
        // .with_system(rotate_entites)
        .with_player_controller(player_settings)
        .build();

    application.run();
}
