use std::sync::Arc;

use app::plugin::{Plugin, PluginAppAccess};
use bevy_ecs::system::Commands;
use nalgebra::{Point3, Vector3};
use scene::asset::AssetId;

use game::core::application::{AppConfig, Application};
use game::player::{PlayerControllerSettings, PlayerPlugin, PlayerSpawnSettings};

use physics::physics_context::BoxCollider;
use scene::light::{CastShadow, Light, PointLight};
use scene::material::CpuMaterial;
use scene::mesh::CpuMesh;
use scene::model::{CpuPrimitive, Model};
use scene::transform::TransformBuilder;

fn spawn_world(mut commands: Commands) {
    let cube = CpuMesh::cube(1.0, 1.0, 1.0);

    let bounding_box = cube.bounding_box.clone();

    let white_material = Arc::new(CpuMaterial {
        id: AssetId::new_v4(),
        base_color: [1.0; 3].into(),
        base_color_texture: None,
        roughness_factor: 1.0,
        metallic_factor: 0.0,
        emissivity: Default::default(),
    });

    let mut spawn_light = |position: Point3<f32>| {
        commands.spawn((
            Light::Point(PointLight {
                color: Vector3::new(1.0, 1.0, 1.0),
                range: 1000.0,
                intensity: 60.0,
            }),
            TransformBuilder::new()
                .scale([0.1; 3].into())
                .position(position)
                .build(),
            CastShadow,
        ));
    };

    spawn_light([0.0, 0.0, 0.0].into());

    commands.spawn((
        Model {
            primitives: vec![CpuPrimitive {
                mesh: cube.clone(),
                material: white_material.clone(),
            }],
        },
        BoxCollider {
            bounds: bounding_box.clone(),
        },
        TransformBuilder::new()
            .position([0.0, -1.0, 0.0].into())
            .scale([30.0, 1.0, 20.0].into())
            .build(),
    ));

    commands.spawn((
        Model {
            primitives: vec![CpuPrimitive {
                mesh: cube.clone(),
                material: white_material.clone(),
            }],
        },
        BoxCollider {
            bounds: bounding_box.clone(),
        },
        TransformBuilder::new()
            .position([0.0, 1.5, -4.0].into())
            .build(),
    ));
}

struct ShadowDemoPlugin;
impl Plugin for ShadowDemoPlugin {
    fn build(&mut self, app: &mut PluginAppAccess) {
        app.with_startup_system(spawn_world);
    }
}

fn main() {
    let config = AppConfig::default();

    let player_controller_settings = PlayerControllerSettings::new(5.0, 1.0, 9.81);

    let player_spawn_settings = PlayerSpawnSettings {
        initial_transform: Default::default(),
        controller_settings: player_controller_settings,
        free_cam_activated: false,
    };

    let mut application = Application::new(config);
    application
        .app
        .with_plugin(ShadowDemoPlugin)
        .with_plugin(PlayerPlugin::new(player_spawn_settings));

    application.run();
}
