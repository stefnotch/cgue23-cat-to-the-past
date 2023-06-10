use app::plugin::{Plugin, PluginAppAccess};
use bevy_ecs::prelude::{not, Commands, Entity, EventReader, Query, Res, With};
use bevy_ecs::schedule::IntoSystemConfig;
use input::events::{ElementState, MouseButton, MouseInput};
use physics::physics_context::{PhysicsContext, RapierRigidBodyHandle, Ray};
use physics::pickup_physics::PickedUp;
use scene::camera::Camera;
use scene::pickup::Pickupable;
use time::time_manager::is_rewinding;

use crate::player::Player;

fn ray_cast(
    mut commands: Commands,
    mut event_reader: EventReader<MouseInput>,
    physics_context: Res<PhysicsContext>,
    camera: Res<Camera>,
    query: Query<Entity, With<PickedUp>>,
    query_pickupable: Query<&Pickupable>,
    exclude_query: Query<&RapierRigidBodyHandle, With<Player>>,
) {
    for event in event_reader.iter() {
        if event.button != MouseButton::Left {
            continue;
        }

        match event.state {
            ElementState::Pressed => {
                let ray = Ray::new(
                    camera.position,
                    camera.orientation * Camera::forward().into_inner(),
                );

                let hit = physics_context.cast_ray(&ray, 5.0, true, exclude_query.iter().collect());
                if let Some((entity, _toi)) = hit {
                    if !query_pickupable.contains(entity) {
                        return;
                    }

                    commands.entity(entity).insert(PickedUp {
                        position: camera.position,
                    });
                }
            }
            ElementState::Released => {
                for entity in query.iter() {
                    commands.entity(entity).remove::<PickedUp>();
                }
            }
        }
    }
}

fn drop_when_rewinding(mut commands: Commands, query: Query<Entity, With<PickedUp>>) {
    for entity in query.iter() {
        commands.entity(entity).remove::<PickedUp>();
    }
}

pub struct PickupPlugin;

impl Plugin for PickupPlugin {
    fn build(&mut self, app: &mut PluginAppAccess) {
        app //
            .with_system(drop_when_rewinding.run_if(is_rewinding))
            .with_system(ray_cast.run_if(not(is_rewinding)));
    }
}
