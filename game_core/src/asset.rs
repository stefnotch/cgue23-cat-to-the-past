use std::{collections::HashMap, sync::Arc};

use bevy_ecs::system::Resource;
use uuid::Uuid;

pub trait Asset {
    fn id(&self) -> AssetId;
}

pub type AssetId = uuid::Uuid;

#[derive(Resource)]
pub struct Assets<T: Asset> {
    pub assets: HashMap<Uuid, Arc<T>>,
}

impl<T: Asset> Default for Assets<T> {
    fn default() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }
}
