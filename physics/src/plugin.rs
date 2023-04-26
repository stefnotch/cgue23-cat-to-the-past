use app::plugin::{Plugin, PluginAppAccess};
use bevy_ecs::schedule::{IntoSystemConfig, IntoSystemSetConfig, SystemSet};

use crate::{
    physics_context::{
        apply_collider_changes, apply_collider_sensor_change, apply_rigid_body_added,
        apply_rigid_body_type_change, step_physics_simulation, update_move_body_position_system,
        update_transform_system, PhysicsContext,
    },
    player_physics::{apply_player_character_controller_changes, step_character_controllers},
};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PhysicsPluginSets {
    BeforePhysics,
    /// Physics
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
        .with_system(apply_player_character_controller_changes.in_set(PhysicsPluginSets::BeforePhysics))
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
         .with_resource(Events::<CollisionEvent>::default()).
        with_system(
            Events::<CollisionEvent>::update_system
                .in_set(PhysicsPluginSets::Physics) 
                .after(step_physics_simulation),
        )
         .with_set(PhysicsPluginSets::BeforePhysics.before(PhysicsPluginSets::Physics))
         // Time rewinding
        let velocity_history = GameChangeHistory::<VelocityChange>::new();
        velocity_history.setup_systems(
            world,
            schedule,
            time_manager_track_rigid_body_velocity,
            time_manager_rewind_rigid_body_velocity,
        );

        let body_type_history = GameChangeHistory::<RigidBodyTypeChange>::new();
        body_type_history.setup_systems(
            world,
            schedule,
            time_manager_track_rigid_body_type,
            time_manager_rewind_rigid_body_type,
        );

        // Special logic for time rewinding with a Transform component
        schedule.add_system(
            time_rewinding_move_body_transform
                .in_set(AppStage::Update)
                .run_if(is_rewinding),
        );

        
            //e
            ;
    }
}
