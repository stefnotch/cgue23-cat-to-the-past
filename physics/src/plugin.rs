use app::plugin::{Plugin, PluginAppAccess};
use bevy_ecs::schedule::{apply_system_buffers, IntoSystemConfig, IntoSystemSetConfig, SystemSet};
use game_core::time_manager::game_change::GameChangeHistoryPlugin;

use crate::{
    physics_change::{
        time_manager_rewind_rigid_body_type, time_manager_rewind_rigid_body_velocity,
        time_manager_track_rigid_body_type, time_manager_track_rigid_body_velocity,
        RigidBodyTypeChange, RigidBodyTypes, VelocityChange,
    },
    physics_context::{
        apply_collider_changes, apply_collider_sensor_change, apply_rigid_body_added,
        apply_rigid_body_type_change, apply_transform_changes, step_physics_simulation,
        write_transform_back, PhysicsContext,
    },
    pickup_physics::{
        start_pickup, stop_pickup, update_pickup_target_position, update_pickup_transform,
    },
    player_physics::{apply_player_character_controller_changes, step_character_controllers},
};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum PhysicsPluginSets {
    PickupUpdate,
    TimeRewinding,
    BeforePhysics,
    Physics,
    AfterPhysics,
}

// We could improve the physics timestep stuff using https://gafferongames.com/post/fix_your_timestep/
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&mut self, app: &mut PluginAppAccess) {
        app.with_resource(PhysicsContext::new())
            .with_set(PhysicsPluginSets::PickupUpdate.before(PhysicsPluginSets::TimeRewinding))
            .with_set(PhysicsPluginSets::TimeRewinding.before(PhysicsPluginSets::BeforePhysics))
            .with_set(PhysicsPluginSets::BeforePhysics.before(PhysicsPluginSets::Physics))
            .with_set(PhysicsPluginSets::Physics.before(PhysicsPluginSets::AfterPhysics));

        // Make sure that all command buffers have been applied
        app.with_system(apply_system_buffers.before(PhysicsPluginSets::PickupUpdate));

        // Time rewinding happens before all physics (we recreate a snapshot of what the physics world looked like before we step it)
        app //
            .with_resource(RigidBodyTypes::default())
            .with_plugin(
                GameChangeHistoryPlugin::<RigidBodyTypeChange>::new()
                    .with_tracker(time_manager_track_rigid_body_type)
                    .with_rewinder(time_manager_rewind_rigid_body_type),
            )
            .with_set(
                GameChangeHistoryPlugin::<RigidBodyTypeChange>::system_set()
                    .in_set(PhysicsPluginSets::TimeRewinding),
            )
            .with_system(
                apply_system_buffers
                    .after(PhysicsPluginSets::TimeRewinding)
                    .before(PhysicsPluginSets::BeforePhysics),
            );

        // Keep ECS and physics world in sync, do note that we should probably do this after update and before physics.
        app //
            .with_system(apply_collider_changes.in_set(PhysicsPluginSets::BeforePhysics))
            .with_system(
                apply_rigid_body_added
                    .in_set(PhysicsPluginSets::BeforePhysics)
                    .after(apply_collider_changes),
            )
            .with_system(
                apply_rigid_body_type_change
                    .in_set(PhysicsPluginSets::BeforePhysics)
                    .after(apply_rigid_body_added),
            )
            .with_system(
                apply_collider_sensor_change
                    .in_set(PhysicsPluginSets::BeforePhysics)
                    .after(apply_rigid_body_type_change),
            )
            .with_system(
                apply_player_character_controller_changes
                    .in_set(PhysicsPluginSets::BeforePhysics)
                    .after(apply_collider_sensor_change),
            )
            .with_system(
                apply_transform_changes
                    .in_set(PhysicsPluginSets::BeforePhysics)
                    .after(apply_player_character_controller_changes),
            )
            // The velocity change direcly modifies the physics world, so we need to do it after we have applied the rigid body type change
            .with_plugin(
                GameChangeHistoryPlugin::<VelocityChange>::new()
                    .with_tracker(time_manager_track_rigid_body_velocity)
                    .with_rewinder(time_manager_rewind_rigid_body_velocity),
            )
            .with_set(
                GameChangeHistoryPlugin::<VelocityChange>::system_set()
                    .in_set(PhysicsPluginSets::BeforePhysics)
                    .after(apply_transform_changes),
            );

        // Physics step
        app //
            .with_system(step_physics_simulation.in_set(PhysicsPluginSets::Physics))
            .with_system(
                step_character_controllers
                    .in_set(PhysicsPluginSets::Physics)
                    .after(step_physics_simulation),
            );

        // Write back
        app //
            .with_system(write_transform_back.in_set(PhysicsPluginSets::AfterPhysics));

        // Pick up logic, most of it is pretty much independent of the physics and simply happens before it
        app //
            .with_system(start_pickup.in_set(PhysicsPluginSets::PickupUpdate))
            .with_system(
                stop_pickup
                    .in_set(PhysicsPluginSets::PickupUpdate)
                    .after(start_pickup),
            )
            .with_system(
                update_pickup_target_position
                    .in_set(PhysicsPluginSets::PickupUpdate)
                    .after(start_pickup),
            )
            .with_system(
                update_pickup_transform
                    .in_set(PhysicsPluginSets::Physics)
                    .after(step_physics_simulation)
                    .after(step_character_controllers),
            );
    }
}
