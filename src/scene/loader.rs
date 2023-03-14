use bevy_ecs::prelude::*;
use gltf::Gltf;
use std::{collections::HashMap, path::Path};
use uuid::Uuid;

// textures
// meshes
// materials
// lights
pub trait Asset {}

#[derive(Resource)]
pub struct Assets<T: Asset> {
    assets: HashMap<Uuid, T>,
}

impl<T: Asset> Default for Assets<T> {
    fn default() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }
}

// scene.json -> assets

// list of assets in code

#[derive(Resource)]
pub struct AssetServer {}

impl AssetServer {
    pub fn insert_asset_types(world: &mut World) {
        world.insert_resource(Assets::<super::texture::Texture>::default());
        world.insert_resource(Assets::<super::mesh::Mesh>::default());
        world.insert_resource(Assets::<super::material::Material>::default());
        // world.insert_resource(Assets::<Light>::default());
    }

    /// loads one .gltf file
    pub fn load_scene<P>(&self, path: P) -> Result<(), Box<dyn std::error::Error>>
    where
        P: AsRef<Path>,
    {
        let scene = Gltf::open(path)?;
        println!("scene: {:?}", scene);

        Ok(())
    }

    pub fn new() -> Self {
        AssetServer {}
    }
}
