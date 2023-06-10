//#![windows_subsystem = "windows"]

mod levels;

use app::entity_event::EntityEvent;
use app::plugin::{Plugin, PluginAppAccess};
use bevy_ecs::prelude::{not, Query};
use bevy_ecs::schedule::IntoSystemConfig;
use bevy_ecs::schedule::IntoSystemSetConfig;
use debug::setup_debugging;
use game::level_flags::{FlagChange, LevelFlags};
use game::pickup_system::PickupPlugin;
use game::rewind_power::RewindPowerPlugin;
use levels::level1::Level1Plugin;
use loader::config_loader::LoadableConfig;
use loader::loader::SceneLoader;
use scene::flag_trigger::FlagTrigger;
use scene::level::LevelId;

use std::time::Instant;
use time::time::Time;
use time::time_manager::{game_change, is_rewinding};

use bevy_ecs::system::{Commands, Res, ResMut};

use game::core::application::{AppConfig, AppStage, Application};
use game::game_ui::UIPlugin;
use game::player::{PlayerControllerSettings, PlayerPlugin, PlayerSpawnSettings};

use physics::physics_events::{CollisionEvent, CollisionEventFlags};

use crate::levels::level0::Level0Plugin;
use crate::levels::level2::Level2Plugin;
use scene::transform::TransformBuilder;

fn spawn_world(mut commands: Commands, scene_loader: Res<SceneLoader>) {
    let before = Instant::now();
    scene_loader
        .load_default_scene("./assets/scene/levels/levels.gltf", &mut commands)
        .unwrap();
    println!(
        "Loading the scene took {}sec",
        before.elapsed().as_secs_f64()
    );
}

fn setup_levels(
    mut level_flags: ResMut<LevelFlags>,
    mut game_changes: ResMut<game_change::GameChangeHistory<FlagChange>>,
) {
    level_flags.set_count(LevelId::new(0), 1, &mut game_changes);
    level_flags.set_count(LevelId::new(1), 2, &mut game_changes);
    level_flags.set_count(LevelId::new(2), 2, &mut game_changes);
}

fn _print_fps(time: Res<Time>) {
    println!(
        "{} FPS - {} ms",
        1.0 / time.delta_seconds(),
        time.delta_seconds() * 1000.0
    );
}

fn flag_system(
    mut level_flags: ResMut<LevelFlags>,
    mut game_changes: ResMut<game_change::GameChangeHistory<FlagChange>>,
    flag_triggers: Query<(&FlagTrigger, &EntityEvent<CollisionEvent>)>,
) {
    for (flag_trigger, collision_events) in flag_triggers.iter() {
        for collision_event in collision_events.iter() {
            match collision_event {
                CollisionEvent::Started(_e2, CollisionEventFlags::SENSOR) => {
                    level_flags.set_and_record(
                        flag_trigger.level_id,
                        flag_trigger.flag_id,
                        true,
                        &mut game_changes,
                    );
                }
                CollisionEvent::Stopped(_e2, CollisionEventFlags::SENSOR) => {
                    level_flags.set_and_record(
                        flag_trigger.level_id,
                        flag_trigger.flag_id,
                        false,
                        &mut game_changes,
                    );
                }
                _ => {}
            }
        }
    }
}

struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&mut self, app: &mut PluginAppAccess) {
        app.with_startup_system(spawn_world)
            .with_startup_system(setup_levels)
            .with_plugin(UIPlugin)
            .with_set(UIPlugin::system_set().in_set(AppStage::Update))
            .with_plugin(PickupPlugin)
            .with_plugin(RewindPowerPlugin)
            .with_plugin(Level0Plugin)
            .with_set(Level0Plugin::system_set().in_set(AppStage::UpdateLevel))
            .with_plugin(Level1Plugin)
            .with_set(
                Level1Plugin::system_set()
                    .in_set(AppStage::UpdateLevel)
                    .after(Level0Plugin::system_set()),
            )
            .with_plugin(Level2Plugin)
            .with_set(
                Level2Plugin::system_set()
                    .in_set(AppStage::UpdateLevel)
                    .after(Level1Plugin::system_set()),
            )
            .with_set(
                RewindPowerPlugin::system_set()
                    .in_set(AppStage::Update)
                    .before(UIPlugin::system_set()),
            )
            .with_system(
                flag_system
                    .in_set(AppStage::Update)
                    .run_if(not(is_rewinding)),
            );
    }
}

fn main() {
    let _guard = setup_debugging();

    // Only the main project actually loads the config from the file
    let config: AppConfig = LoadableConfig::load("./assets/config.json").into();

    let player_spawn_settings = PlayerSpawnSettings {
        initial_transform: TransformBuilder::new()
            .position([0.0, 1.0, 3.0].into())
            .build(),
        controller_settings: PlayerControllerSettings::default()
            .with_sensitivity(config.mouse_sensitivity),
        free_cam_activated: false,
    };

    let mut application = Application::new(config);
    application
        .app
        .with_plugin(GamePlugin)
        .with_plugin(PlayerPlugin::new(player_spawn_settings));

    application.run();
}
