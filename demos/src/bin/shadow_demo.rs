use bevy_ecs::prelude::{Component, Res, With};

use bevy_ecs::schedule::IntoSystemConfig;
use loader::config_loader::LoadableConfig;

use std::sync::Arc;
use std::time::Instant;

use app::plugin::{Plugin, PluginAppAccess};
use bevy_ecs::system::{Commands, Query};
use nalgebra::Point3;

use game::core::application::{AppConfig, AppStage, Application};
use game::player::{PlayerPlugin, PlayerSpawnSettings};
use levels::level_id::LevelId;
use time::time_manager::TimeManager;

use loader::loader::SceneLoader;
use scene::light::CastsShadow;
use scene::mesh::CpuMesh;
use scene::model::{CpuPrimitive, Model};
use scene::transform::Transform;

fn spawn_world(mut commands: Commands, scene_loader: Res<SceneLoader>) {
    let before = Instant::now();
    scene_loader
        .load_default_scene(
            "./assets/scene/demo/shadow_demo/shadow_test.glb",
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
        CastsShadow,
        MovingBox,
        LevelId::new(0),
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
    let config: AppConfig = LoadableConfig::default().into();

    let player_spawn_settings = PlayerSpawnSettings {
        initial_transform: Default::default(),
        controller_settings: Default::default(),
        free_cam_activated: true,
    };

    let mut application = Application::new(config);
    application
        .app
        .with_plugin(ShadowDemoPlugin)
        .with_plugin(PlayerPlugin::new(player_spawn_settings));

    application.run();
}
