use app::plugin::{Plugin, PluginAppAccess};
use bevy_ecs::prelude::{not, Commands, Entity, EventReader, Query, Res, With};
use bevy_ecs::schedule::IntoSystemConfig;
use bevy_ecs::system::{ResMut, Resource};
use input::events::{ElementState, MouseButton, MouseInput};
use physics::physics_context::{PhysicsContext, RapierRigidBodyHandle, Ray};
use physics::pickup_physics::PickedUp;
use scene::camera::Camera;
use scene::pickup::Pickupable;
use time::time_manager::is_rewinding;

use crate::player::Player;

#[derive(Resource)]
pub struct PickupInfo {
    pub can_pickup: bool,
}

impl PickupInfo {
    pub fn new() -> Self {
        Self { can_pickup: false }
    }
}

fn ray_cast(
    mut commands: Commands,
    mut event_reader: EventReader<MouseInput>,
    physics_context: Res<PhysicsContext>,
    camera: Res<Camera>,
    mut pickup_info: ResMut<PickupInfo>,
    query: Query<Entity, With<PickedUp>>,
    query_pickupable: Query<&Pickupable>,
    exclude_query: Query<&RapierRigidBodyHandle, With<Player>>,
) {
    let ray = Ray::new(
        camera.position,
        camera.orientation * Camera::forward().into_inner(),
    );
    let hit = physics_context.cast_ray(&ray, 5.0, true, exclude_query.iter().collect());
    let entity = hit
        .map(|(entity, _toi)| entity)
        .filter(|entity| query_pickupable.contains(*entity));

    pickup_info.can_pickup = entity.is_some();

    for event in event_reader.iter() {
        if event.button != MouseButton::Left {
            continue;
        }

        match event.state {
            ElementState::Pressed => {
                if let Some(entity) = entity {
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
            .with_resource(PickupInfo::new())
            .with_system(drop_when_rewinding.run_if(is_rewinding))
            .with_system(ray_cast.run_if(not(is_rewinding)));
    }
}
