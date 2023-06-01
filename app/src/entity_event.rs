use bevy_ecs::{
    prelude::{Component, Event},
    system::Query,
};

/// An event that is tied to an entity.
#[derive(Component, Debug)]
pub struct EntityEvent<T>
where
    T: Event,
{
    events: Vec<T>,
}

impl<T> Default for EntityEvent<T>
where
    T: Event,
{
    fn default() -> Self {
        Self { events: Vec::new() }
    }
}

impl<T> EntityEvent<T>
where
    T: Event,
{
    pub fn update(mut query: Query<&mut EntityEvent<T>>) {
        for mut event_holder in query.iter_mut() {
            event_holder.events.clear();
        }
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn add(&mut self, event: T) {
        self.events.push(event);
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.events.iter()
    }

    pub fn into_iter(&mut self) -> impl Iterator<Item = T> {
        let events = std::mem::take(&mut self.events);
        events.into_iter()
    }
}
