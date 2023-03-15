use crate::scene::light::Light::PointLight;
use crate::scene::light::{Attenuation, Light};
use crate::scene::model::Model;
use crate::scene::transform::Transform;
use bevy_ecs::prelude::*;
use gltf::khr_lights_punctual::Kind;
use gltf::{import, khr_lights_punctual, Gltf, Node, Primitive};
use nalgebra::{Point3, Quaternion, Translation3, UnitQuaternion, Vector3};
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
    pub fn load_default_scene<P>(&self, path: P) -> Result<(), Box<dyn std::error::Error>>
    where
        P: AsRef<Path>,
    {
        let (doc, buffers, images) = import(path)?;

        if doc.scenes().len() > 1 {
            todo!("We shouldn't have more than one scene; return Err result later");
        }

        let scene = doc.default_scene().ok_or("Default scene is not set")?;

        let mut lights = vec![];
        let mut meshes = vec![];

        for node in scene.nodes() {
            Self::read_node(&node, &mut lights, &mut meshes, &Transform::default());
        }

        println!("{:?}", lights);

        // https://github.com/flomonster/easy-gltf/blob/master/src/utils/gltf_data.rs

        // TODO: https://github.com/bevyengine/bevy/blob/main/crates/bevy_gltf/src/loader.rs#L1027

        // create our meshes and upload them to the GPU

        // load textures

        // load materials

        // load primitives (reference stuff above) and create models

        Ok(())
    }

    pub fn new() -> Self {
        AssetServer {}
    }

    fn read_node(
        node: &Node,
        lights: &mut Vec<Light>,
        models: &mut Vec<Model>,
        parent_transform: &Transform,
    ) {
        let local_transform: Transform = node.transform().into();
        let global_transform = parent_transform * local_transform;

        for child in node.children() {
            AssetServer::read_node(&child, lights, models, &global_transform);
        }

        // skip loading camera (hardcoded)

        if let Some(light) = node.light() {
            lights.push(Self::load_light(light, &global_transform));
        }

        if let Some(mesh) = node.mesh() {
            for primitive in mesh.primitives() {
                models.push(Self::load_model(primitive, &global_transform))
            }
        }
    }

    fn load_light(light: khr_lights_punctual::Light, global_transform: &Transform) -> Light {
        match light.kind() {
            Kind::Directional => {
                todo!("directional lights are not supported yet")
            }
            Kind::Point => Light::PointLight {
                position: Point3::from(global_transform.translation.vector),
                // TODO: validate implementation (might have mistaken column and row)
                color: Vector3::from_column_slice(&light.color()),
                range: light.range().unwrap_or_else(|| 20.0f32),
                intensity: light.intensity(),
            },
            Kind::Spot { .. } => {
                todo!("spot lights are not supported yet")
            }
        }
    }

    fn load_model(primitive: Primitive, global_transform: &Transform) -> Model {
        Model { primitives: vec![] }
    }
}

impl From<gltf::scene::Transform> for Transform {
    fn from(value: gltf::scene::Transform) -> Self {
        // rotation is a quaternion
        let (translation, rotation, scale) = value.decomposed();

        let translation: Translation3<f32> = Translation3::from(translation);
        let rotation: UnitQuaternion<f32> =
            UnitQuaternion::new_normalize(Quaternion::from(rotation));
        let scale: Vector3<f32> = Vector3::from_row_slice(&scale);

        Self {
            translation,
            rotation,
            scale,
        }
    }
}
