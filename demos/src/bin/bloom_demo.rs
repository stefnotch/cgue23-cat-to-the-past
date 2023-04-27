//#![windows_subsystem = "windows"]
use std::f32::consts::PI;
use std::sync::Arc;

use app::plugin::{Plugin, PluginAppAccess};
use bevy_ecs::system::Commands;
use nalgebra::{Point3, UnitQuaternion, Vector3};
use scene::asset::AssetId;

use game::core::application::{AppConfig, Application};
use game::player::{PlayerPlugin, PlayerSpawnSettings};
use scene::light::{Light, PointLight};
use scene::material::CpuMaterial;
use scene::mesh::CpuMesh;
use scene::model::{CpuPrimitive, Model};
use scene::transform::TransformBuilder;

fn spawn_bloom_demo(mut commands: Commands) {
    let cube = CpuMesh::cube(1.0, 1.0, 1.0);

    let material = CpuMaterial {
        id: AssetId::new_v4(),
        base_color: [1.0, 1.0, 1.0].into(),
        base_color_texture: None,
        roughness_factor: 0.9,
        metallic_factor: 0.1,
        emissivity: Default::default(),
    };

    let model = Model {
        primitives: vec![CpuPrimitive {
            mesh: cube.clone(),
            material: Arc::from(material),
        }],
    };

    commands.spawn((
        model.clone(),
        TransformBuilder::new()
            .position([0.0, -1.0, 0.0].into())
            .scale([12.5, 0.5, 12.5].into())
            .build(),
    ));

    commands.spawn((
        model.clone(),
        TransformBuilder::new()
            .position([0.0, 1.5, 0.0].into())
            .scale([0.5, 0.5, 0.5].into())
            .build(),
    ));

    commands.spawn((
        model.clone(),
        TransformBuilder::new()
            .position([2.0, 0.0, 1.0].into())
            .scale([0.5, 0.5, 0.5].into())
            .build(),
    ));

    commands.spawn((
        model.clone(),
        TransformBuilder::new()
            .position([-1.0, -1.0, 2.0].into())
            .rotation(UnitQuaternion::from_euler_angles(PI / 3.0, 0.0, PI / 3.0))
            .build(),
    ));

    let mut spawn_light = |position: Point3<f32>, color: Vector3<f32>, intensity: f32| {
        commands.spawn((
            Model {
                primitives: vec![CpuPrimitive {
                    mesh: cube.clone(),
                    material: Arc::from(CpuMaterial {
                        id: AssetId::new_v4(),
                        base_color: Vector3::new(1.0, 1.0, 1.0),
                        base_color_texture: None,
                        roughness_factor: 0.9,
                        metallic_factor: 0.1,
                        emissivity: color * intensity,
                    }),
                }],
            },
            Light::Point(PointLight {
                color,
                range: 1000.0,
                intensity: 6.0,
            }),
            TransformBuilder::new()
                .position(position)
                .scale([0.25, 0.25, 0.25].into())
                .build(),
        ));
    };

    let lights: [(Point3<f32>, Vector3<f32>, f32); 4] = [
        ([0.0, 0.5, 1.5].into(), [1.0, 1.0, 1.0].into(), 2.0),
        ([-4.0, 0.5, -3.0].into(), [1.0, 0.0, 0.0].into(), 4.0),
        ([3.0, 0.5, 1.0].into(), [0.0, 0.0, 1.0].into(), 6.0),
        ([-0.8, 2.4, -1.0].into(), [0.0, 1.0, 0.0].into(), 2.5),
    ];

    for (position, color, intensity) in lights {
        spawn_light(position, color, intensity);
    }
}

struct BloomDemoPlugin;
impl Plugin for BloomDemoPlugin {
    fn build(&mut self, app: &mut PluginAppAccess) {
        app.with_startup_system(spawn_bloom_demo);
    }
}

fn main() {
    let config = AppConfig::default();

    let player_spawn_settings = PlayerSpawnSettings {
        initial_transform: Default::default(),
        controller_settings: Default::default(),
        free_cam_activated: true,
    };

    let mut application = Application::new(config);
    application
        .app
        .with_plugin(BloomDemoPlugin)
        .with_plugin(PlayerPlugin::new(player_spawn_settings));

    application.run();
}
