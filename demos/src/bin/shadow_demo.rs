use bevy_ecs::prelude::{Component, Res, With};

use bevy_ecs::schedule::{IntoSystemConfig, IntoSystemSetConfig};

use std::sync::Arc;
use std::time::Instant;

use app::plugin::{Plugin, PluginAppAccess};
use bevy_ecs::system::{Commands, Query};
use nalgebra::Point3;

use game::core::application::{AppConfig, AppStage, Application};
use game::player::{PlayerControllerSettings, PlayerPlugin, PlayerSpawnSettings};
use game_core::time_manager::TimeManager;

use scene::mesh::CpuMesh;
use scene::model::{CpuPrimitive, Model};
use scene::transform::Transform;
use scene_loader::loader::AssetServer;

fn spawn_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    let before = Instant::now();
    asset_server
        .load_default_scene(
            "./assets/scene/testing/shadow_test/shadow_test.glb",
            &mut commands,
        )
        .unwrap();
    println!(
        "Loading the scene took {}sec",
        before.elapsed().as_secs_f64()
    );
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
        MovingBox,
    ));
}

pub fn move_cubes(mut query: Query<&mut Transform, With<MovingBox>>, time: Res<TimeManager>) {
    for mut transform in query.iter_mut() {
        let new_position = Point3::new(
            time.level_time_seconds().sin() * 2.0,
            time.level_time_seconds().cos() * 2.0 + 2.0,
            time.level_time_seconds().cos() * 2.0,
        );
        transform.position = new_position;
    }
}

struct ShadowDemoPlugin;
impl Plugin for ShadowDemoPlugin {
    fn build(&mut self, app: &mut PluginAppAccess) {
        app.with_startup_system(spawn_world)
            .with_startup_system(spawn_moving_cube)
            .with_system(move_cubes.in_set(AppStage::Update));
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

    let mut application = Application::new(config);
    application
        .app
        .with_plugin(ShadowDemoPlugin)
        .with_plugin(PlayerPlugin::new(player_spawn_settings))
        .with_set(PlayerPlugin::system_set().in_set(AppStage::Update));

    application.run();
}
