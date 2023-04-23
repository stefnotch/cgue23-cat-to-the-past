use std::sync::Arc;

use bevy_ecs::system::Commands;
use game::core::application::{AppConfig, ApplicationBuilder};
use game::player::{PlayerControllerSettings, PlayerSpawnSettings};
use game_core::asset::AssetId;
use nalgebra::{Point3, Vector3};
use scene::light::{Light, PointLight};
use scene::material::CpuMaterial;
use scene::mesh::CpuMesh;
use scene::model::{CpuPrimitive, Model};
use scene::transform::TransformBuilder;

fn spawn_pbr_demo(mut commands: Commands) {
    let sphere = CpuMesh::sphere(64, 32, 1.0);

    let mut spawn_light = |position: Point3<f32>| {
        commands.spawn((
            Light::Point(PointLight {
                color: Vector3::new(1.0, 1.0, 1.0),
                range: 1000.0,
                intensity: 300.0,
            }),
            TransformBuilder::new().position(position).build(),
        ));
    };

    spawn_light([-10.0, 10.0, 10.0].into());
    spawn_light([10.0, 10.0, 10.0].into());
    spawn_light([-10.0, -10.0, 10.0].into());
    spawn_light([10.0, -10.0, 10.0].into());

    let spacing: f32 = 1.25;

    let n = 7;

    for row in 0..n {
        let metallic: f32 = row as f32 / (n as f32 - 1.0);
        for col in 0..n {
            let roughness: f32 = (col as f32 / (n as f32 - 1.0)).max(0.05);

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

fn main() {
    let config = AppConfig::default();

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
