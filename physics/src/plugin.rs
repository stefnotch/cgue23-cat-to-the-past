use app::plugin::{Plugin, PluginAppAccess};
use bevy_ecs::{
    prelude::Events,
    schedule::{IntoSystemConfig, IntoSystemSetConfig, SystemSet},
};
use game_core::time_manager::{
    game_change::{GameChangeHistoryPlugin, GameChangeHistoryPluginSet},
    is_rewinding,
};

use crate::{
    physics_change::{
        time_manager_rewind_rigid_body_type, time_manager_rewind_rigid_body_velocity,
        time_manager_track_rigid_body_type, time_manager_track_rigid_body_velocity,
        RigidBodyTypeChange, VelocityChange,
    },
    physics_context::{
        apply_collider_changes, apply_collider_sensor_change, apply_rigid_body_added,
        apply_rigid_body_type_change, step_physics_simulation, time_rewinding_move_body_transform,
        update_move_body_position_system, update_transform_system, PhysicsContext,
    },
    physics_events::CollisionEvent,
    pickup_physics::{
        start_pickup, stop_pickup, update_pickup_target_position, update_pickup_transform,
    },
    player_physics::{apply_player_character_controller_changes, step_character_controllers},
};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PhysicsPluginSets {
    BeforePhysics,
    Physics,
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&mut self, app: &mut PluginAppAccess) {
        app.with_resource(PhysicsContext::new())
            // Keep ECS and physics world in sync, do note that we should probably do this after update and before physics.
            .with_system(apply_collider_changes.in_set(PhysicsPluginSets::BeforePhysics))
            .with_system(apply_rigid_body_added.in_set(PhysicsPluginSets::BeforePhysics))
            .with_system(apply_rigid_body_type_change.in_set(PhysicsPluginSets::BeforePhysics))
            .with_system(apply_collider_sensor_change.in_set(PhysicsPluginSets::BeforePhysics))
            .with_system(
                apply_player_character_controller_changes.in_set(PhysicsPluginSets::BeforePhysics),
            )
            .with_system(update_move_body_position_system.in_set(PhysicsPluginSets::BeforePhysics))
            // Update physics world and write back to ECS world
            .with_system(step_physics_simulation.in_set(PhysicsPluginSets::Physics))
            .with_system(
                step_character_controllers
                    .in_set(PhysicsPluginSets::Physics)
                    .after(step_physics_simulation),
            )
            .with_system(
                update_transform_system
                    .in_set(PhysicsPluginSets::Physics)
                    .after(step_physics_simulation),
            )
            .with_resource(Events::<CollisionEvent>::default())
            .with_system(
                Events::<CollisionEvent>::update_system
                    .in_set(PhysicsPluginSets::Physics)
                    .after(step_physics_simulation),
            )
            .with_set(PhysicsPluginSets::BeforePhysics.before(PhysicsPluginSets::Physics))
            // Time rewinding
            .with_plugin(
                GameChangeHistoryPlugin::<VelocityChange>::new()
                    .with_tracker(time_manager_track_rigid_body_velocity)
                    .with_rewinder(time_manager_rewind_rigid_body_velocity),
            )
            .with_set(
                GameChangeHistoryPluginSet::<VelocityChange>::Update
                    .in_set(PhysicsPluginSets::BeforePhysics),
                // TODO: Is BeforePhysics correct here?
            )
            .with_plugin(
                GameChangeHistoryPlugin::<RigidBodyTypeChange>::new()
                    .with_tracker(time_manager_track_rigid_body_type)
                    .with_rewinder(time_manager_rewind_rigid_body_type),
            )
            .with_set(
                GameChangeHistoryPluginSet::<RigidBodyTypeChange>::Update
                    .in_set(PhysicsPluginSets::BeforePhysics),
            )
            // Special logic for time rewinding with a Transform component
            .with_system(
                time_rewinding_move_body_transform
                    .in_set(PhysicsPluginSets::BeforePhysics)
                    .after(time_manager_rewind_rigid_body_type)
                    .run_if(is_rewinding),
            )
            // Pick up logic
            .with_system(start_pickup.in_set(PhysicsPluginSets::BeforePhysics))
            .with_system(stop_pickup.in_set(PhysicsPluginSets::BeforePhysics))
            .with_system(update_pickup_target_position.in_set(PhysicsPluginSets::BeforePhysics))
            .with_system(
                update_pickup_transform
                    .in_set(PhysicsPluginSets::Physics)
                    .after(step_physics_simulation),
            );
        //e
    }
}
