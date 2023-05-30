//#![windows_subsystem = "windows"]
use app::plugin::{Plugin, PluginAppAccess};
use bevy_ecs::event::EventReader;
use bevy_ecs::schedule::IntoSystemSetConfig;
use std::sync::Arc;

use bevy_ecs::system::Commands;
use nalgebra::{Point3, Vector3};
use scene::asset::AssetId;

use game::core::application::{AppConfig, AppStage, Application};
use game::player::{PlayerPlugin, PlayerSpawnSettings};

use physics::physics_context::{BoxCollider, RigidBody, RigidBodyType, Sensor};
use physics::physics_events::CollisionEvent;
use scene::light::{Light, PointLight};
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
            TransformBuilder::new().position(position).build(),
        ));
    };

    spawn_light([0.0, 5.0, 0.0].into());

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
        Sensor,
        TransformBuilder::new()
            .position([0.0, 1.0, -3.0].into())
            .build(),
    ));

    // Stairs
    for i in 0..5 {
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
                .position([10.0 + i as f32, i as f32 * 0.25, -3.0].into())
                .build(),
        ));
    }

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
        RigidBody(RigidBodyType::Dynamic),
        TransformBuilder::new()
            .scale([0.5; 3].into())
            .position([0.0, 3.0, -3.0].into())
            .build(),
    ));
}

fn display_collision_events(mut collision_events: EventReader<CollisionEvent>) {
    for collision_event in collision_events.iter() {
        println!("Received collision event: {collision_event:?}");
    }
}

struct PhysicsDemoPlugin;
impl Plugin for PhysicsDemoPlugin {
    fn build(&mut self, app: &mut PluginAppAccess) {
        app.with_startup_system(spawn_world)
            .with_system(display_collision_events);
    }
}

fn main() {
    let config = AppConfig::default();

    let player_spawn_settings = PlayerSpawnSettings {
        initial_transform: Default::default(),
        controller_settings: Default::default(),
        free_cam_activated: false,
    };

    let mut application = Application::new(config);
    application
        .app
        .with_plugin(PhysicsDemoPlugin)
        .with_plugin(PlayerPlugin::new(player_spawn_settings))
        .with_set(PlayerPlugin::system_set().in_set(AppStage::Update));

    application.run();
}
