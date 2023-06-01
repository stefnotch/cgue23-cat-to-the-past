use app::entity_event::EntityEvent;
use bevy_ecs::prelude::Entity;
use bevy_ecs::system::Query;
use rapier3d::geometry::CollisionEvent as RapierCollisionEvent;
use rapier3d::prelude::{ColliderHandle, ColliderSet};

pub use rapier3d::prelude::CollisionEventFlags;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CollisionEvent {
    /// other entity, and the flags
    Started(Entity, CollisionEventFlags),
    /// other entity, and the flags
    Stopped(Entity, CollisionEventFlags),
}

pub fn handle_collision_event(
    colliders: &ColliderSet,
    event: RapierCollisionEvent,
    query: &mut Query<&mut EntityEvent<CollisionEvent>>,
) {
    let colliders2entities = |handle1, handle2| {
        let entity1 = collider2entity(colliders, handle1);
        let entity2 = collider2entity(colliders, handle2);
        (entity1, entity2)
    };

    match event {
        RapierCollisionEvent::Started(handle1, handle2, flags) => {
            let (entity1, entity2) = colliders2entities(handle1, handle2);
            if let Ok(mut e1) = query.get_mut(entity1) {
                e1.add(CollisionEvent::Started(entity2, flags));
            }
            if let Ok(mut e2) = query.get_mut(entity2) {
                e2.add(CollisionEvent::Started(entity1, flags));
            }
        }
        RapierCollisionEvent::Stopped(handle1, handle2, flags) => {
            let (entity1, entity2) = colliders2entities(handle1, handle2);
            if let Ok(mut e1) = query.get_mut(entity1) {
                e1.add(CollisionEvent::Stopped(entity2, flags));
            }
            if let Ok(mut e2) = query.get_mut(entity2) {
                e2.add(CollisionEvent::Stopped(entity1, flags));
            }
        }
    };
}

pub fn collider2entity(colliders: &ColliderSet, handle: ColliderHandle) -> Entity {
    colliders
        .get(handle)
        .map(|collider| Entity::from_bits(collider.user_data as u64))
        .expect("entity not found for collision event")
}
