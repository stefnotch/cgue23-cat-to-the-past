use bevy_ecs::prelude::{Entity, EventWriter};
use rapier3d::geometry::CollisionEvent as RapierCollisionEvent;
use rapier3d::prelude::{ColliderHandle, ColliderSet};

pub use rapier3d::prelude::CollisionEventFlags;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CollisionEvent {
    Started(Entity, Entity, CollisionEventFlags),
    Stopped(Entity, Entity, CollisionEventFlags),
}

pub fn handle_collision_event(
    colliders: &ColliderSet,
    event: RapierCollisionEvent,
    collision_events: &mut EventWriter<CollisionEvent>,
) {
    let colliders2entities = |handle1, handle2| {
        let entity1 = collider2entity(colliders, handle1);
        let entity2 = collider2entity(colliders, handle2);
        (entity1, entity2)
    };

    let event = match event {
        RapierCollisionEvent::Started(handle1, handle2, flags) => {
            let (entity1, entity2) = colliders2entities(handle1, handle2);
            CollisionEvent::Started(entity1, entity2, flags)
        }
        RapierCollisionEvent::Stopped(handle1, handle2, flags) => {
            let (entity1, entity2) = colliders2entities(handle1, handle2);
            CollisionEvent::Stopped(entity1, entity2, flags)
        }
    };

    collision_events.send(event);
}

fn collider2entity(colliders: &ColliderSet, handle: ColliderHandle) -> Entity {
    colliders
        .get(handle)
        .map(|collider| Entity::from_bits(collider.user_data as u64))
        .expect("entity not found for collision event")
}
