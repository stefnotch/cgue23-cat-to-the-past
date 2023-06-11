//#![windows_subsystem = "windows"]

mod levels;

use ::levels::current_level::{CurrentLevel, NextLevel};
use ::levels::level_id::LevelId;
use app::entity_event::EntityEvent;
use app::plugin::{Plugin, PluginAppAccess};
use bevy_ecs::prelude::{not, Entity, EventReader, Query};
use bevy_ecs::query::{With, Without};
use bevy_ecs::schedule::IntoSystemConfig;
use bevy_ecs::schedule::IntoSystemSetConfig;
use debug::setup_debugging;
use game::level_flags::{FlagChange, LevelFlags};
use game::pickup_system::PickupPlugin;
use game::rewind_power::{RewindPower, RewindPowerPlugin};
use loader::config_loader::LoadableConfig;
use loader::loader::{PressurePlate, SceneLoader};
use scene::flag_trigger::FlagTrigger;
use scene::level::{NextLevelTrigger, Spawnpoint};

use std::time::Instant;
use time::time::Time;
use time::time_manager::{game_change, is_rewinding};

use bevy_ecs::system::{Commands, Res, ResMut};

use game::core::application::{AppConfig, AppStage, Application};
use game::game_ui::UIPlugin;
use game::player::{Player, PlayerControllerSettings, PlayerPlugin, PlayerSpawnSettings};

use physics::physics_events::CollisionEvent;
use scene::model::Model;

use crate::levels::level0::Level0Plugin;
use crate::levels::level1::Level1Plugin;
use crate::levels::level2::Level2Plugin;
use scene::transform::{Transform, TransformBuilder};

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
    level_flags.set_count(LevelId::new(3), 0, &mut game_changes);
}

fn setup_next_level(
    mut next_level_events: EventReader<NextLevel>,
    mut rewind_power: ResMut<RewindPower>,
) {
    for next_level in next_level_events.iter() {
        let rewind_power_per_level = match next_level.level_id.id() {
            0 => 60.0,
            1 => 60.0,
            2 => 60.0,
            3 => 60.0,
            _ => 0.0,
        };

        rewind_power.set_rewind_power(rewind_power_per_level);
    }
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
            let level_flag_value = match collision_event {
                CollisionEvent::Started(_e2) => true,
                CollisionEvent::Stopped(_e2) => false,
            };
            level_flags.set_and_record(
                flag_trigger.level_id,
                flag_trigger.flag_id,
                level_flag_value,
                &mut game_changes,
            );
        }
    }
}

fn pressure_plate_system(
    mut query: Query<(&mut Model, &PressurePlate, &FlagTrigger)>,
    level_flags: Res<LevelFlags>,
) {
    for (mut model, pressure_plate, flag_trigger) in query.iter_mut() {
        for primitive in model.primitives.iter_mut() {
            let active = level_flags.get(flag_trigger.level_id, flag_trigger.flag_id);
            primitive.material = if active {
                pressure_plate.active_material.clone()
            } else {
                pressure_plate.inactive_material.clone()
            };
        }
    }
}

fn next_level_trigger_system(
    level_triggers: Query<(&LevelId, &EntityEvent<CollisionEvent>), With<NextLevelTrigger>>,
    player_query: Query<Entity, With<Player>>,
    current_level: Res<CurrentLevel>,
) {
    for (level_id, collision_events) in level_triggers.iter() {
        for collision_event in collision_events.iter() {
            match collision_event {
                CollisionEvent::Started(entity) => {
                    if player_query.contains(*entity) {
                        current_level.start_next_level(*level_id);
                    }
                }
                _ => {}
            }
        }
    }
}

fn fall_out_of_world_system(
    current_level: Res<CurrentLevel>,
    mut players_query: Query<&mut Transform, With<Player>>,
    spawnpoints: Query<(&Transform, &LevelId), (With<Spawnpoint>, Without<Player>)>,
) {
    for mut transform in players_query.iter_mut() {
        let spawnpoint = spawnpoints
            .iter()
            .find(|(_, level_id)| level_id == &&current_level.level_id)
            .unwrap()
            .0;

        if transform.position.y < -10.0 {
            transform.position = spawnpoint.position;
        }
    }
}

struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&mut self, app: &mut PluginAppAccess) {
        app.with_startup_system(spawn_world)
            .with_startup_system(setup_levels)
            .with_plugin(PickupPlugin)
            .with_plugin(UIPlugin)
            .with_set(
                UIPlugin::system_set()
                    .in_set(AppStage::Update)
                    .after(PickupPlugin::system_set()),
            )
            .with_plugin(RewindPowerPlugin)
            .with_set(
                RewindPowerPlugin::system_set()
                    .in_set(AppStage::Update)
                    .before(UIPlugin::system_set()),
            )
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
            .with_system(next_level_trigger_system.in_set(AppStage::Update))
            .with_system(
                flag_system
                    .in_set(AppStage::Update)
                    .run_if(not(is_rewinding)),
            )
            .with_system(
                pressure_plate_system
                    .in_set(AppStage::Update)
                    .before(flag_system),
            )
            .with_system(fall_out_of_world_system.in_set(AppStage::Update))
            .with_system(
                setup_next_level
                    .in_set(AppStage::Update)
                    .before(RewindPowerPlugin::system_set()),
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
