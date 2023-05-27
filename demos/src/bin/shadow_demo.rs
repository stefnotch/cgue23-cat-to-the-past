use bevy_ecs::prelude::{Entity, Res};
use bevy_ecs::query::Added;
use std::sync::Arc;
use std::time::Instant;

use app::plugin::{Plugin, PluginAppAccess};
use bevy_ecs::system::{Commands, Query};
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
//
// fn add_cast_shadow_comp(mut commands: Commands, query: Query<Entity, Added<Light>>) {
//     for entity in query.iter() {
//         commands.entity(entity).insert(CastShadow);
//     }
// }

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
        free_cam_activated: true,
    };

    let mut application = Application::new(config);
    application
        .app
        .with_plugin(ShadowDemoPlugin)
        .with_plugin(PlayerPlugin::new(player_spawn_settings));

    application.run();
}
