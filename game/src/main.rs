//#![windows_subsystem = "windows"]

mod pickup_system;

use animations::animation::Animation;
use app::plugin::{Plugin, PluginAppAccess};
use bevy_ecs::prelude::{Component, EventReader, Query, With};
use game_core::level::LevelId;
use game_core::time_manager::TimeManager;
use game_core::{level::level_flags::LevelFlags, time::Time};
use scene::{
    mesh::CpuMesh,
    model::{CpuPrimitive, Model},
};
use scene_loader::loader::{AssetServer, Door};
use std::f32::consts::PI;
use std::sync::Arc;
use std::time::Instant;

use bevy_ecs::system::{Commands, Res, ResMut};
use nalgebra::{Point3, Translation3};

use game::core::application::{AppConfig, Application};

use debug::tracing::start_tracing;

use game::player::{PlayerPlugin, PlayerSpawnSettings};

use crate::pickup_system::ray_cast;
use physics::physics_context::{BoxCollider, MoveBodyPosition, RigidBody, RigidBodyType};
use physics::physics_events::{CollisionEvent, CollisionEventFlags};
use scene::transform::{Transform, TransformBuilder};

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

fn setup_levels(mut level_flags: ResMut<LevelFlags>) {
    level_flags.set_count(LevelId::new(0), 1);
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
            -4.0,
            1.0,
            5.0 * (time.level_time_seconds() * PI / 2.0 * 0.5).sin(),
        );
        move_body_position.new_position = Some(shift.transform_point(&origin));
    }
}

fn door_system(
    mut collision_events: EventReader<CollisionEvent>,
    mut query: Query<(&Animation, &mut MoveBodyPosition), With<Door>>,
) {
    for collision_event in collision_events.iter() {
        if let CollisionEvent::Started(_e1, _e2, CollisionEventFlags::SENSOR) = collision_event {
            let (animation, mut door_new_position) = query.single_mut();
            door_new_position.new_position = Some(animation.end_transform.position);
        }
    }
}

struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&mut self, app: &mut PluginAppAccess) {
        app.with_startup_system(spawn_world)
            .with_startup_system(setup_levels)
            .with_startup_system(spawn_moving_cube)
            .with_system(ray_cast)
            .with_system(door_system)
            .with_system(move_cubes);
    }
}

fn main() {
    let _guard = start_tracing();

    // TODO: remove this
    std::env::set_var("RUST_BACKTRACE", "1");

    // TODO: read from file
    let config = AppConfig::default();

    let player_spawn_settings = PlayerSpawnSettings {
        initial_transform: TransformBuilder::new()
            .position([0.0, 1.0, 3.0].into())
            .build(),
        controller_settings: Default::default(),
        free_cam_activated: false,
    };

    let mut application = Application::new(config);
    application
        .app
        .with_plugin(GamePlugin)
        .with_plugin(PlayerPlugin::new(player_spawn_settings));

    application.run();
}
