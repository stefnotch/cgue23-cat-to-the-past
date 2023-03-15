use crate::scene::light::{Light, PointLight};
use crate::scene::mesh::MeshVertex;
use crate::scene::model::Model;
use crate::scene::transform::{Transform, TransformBuilder};
use bevy_ecs::prelude::*;
use gltf::khr_lights_punctual::Kind;
use gltf::{import, khr_lights_punctual, Node, Semantic};
use nalgebra::{Point3, Quaternion, Translation3, UnitQuaternion, Vector3};
use std::sync::Arc;
use std::{collections::HashMap, path::Path};
use uuid::Uuid;
use vulkano::memory::allocator::{MemoryAllocator, StandardMemoryAllocator};

use super::material::Material;
use super::mesh::Mesh;
use super::texture::Texture;

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
    pub fn load_default_scene<P>(
        &self,
        path: P,
        commands: &mut Commands,
        memory_allocator: &Arc<StandardMemoryAllocator>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        P: AsRef<Path>,
    {
        let (doc, buffers, images) = import(path)?;
        let mut scene_loading_data = SceneLoadingData::new(memory_allocator, buffers, images);
        let mut scene_loading_result = SceneLoadingResult::new();

        if doc.scenes().len() > 1 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "We shouldn't have more than one scene",
            )));
        }

        let scene = doc.default_scene().ok_or("Default scene is not set")?;

        for node in scene.nodes() {
            Self::read_node(
                &node,
                &mut scene_loading_data,
                &mut scene_loading_result,
                &Transform::default(),
            );
        }

        println!("{:?}", scene_loading_result.lights);

        for light in scene_loading_result.lights {
            commands.spawn(light);
        }

        for (transform, model) in scene_loading_result.models {
            // TODO: Add colliders
            commands.spawn((model, transform));
        }

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
        scene_loading_data: &mut SceneLoadingData,
        scene_loading_result: &mut SceneLoadingResult,
        parent_transform: &Transform,
    ) {
        let local_transform: Transform = node.transform().into();
        let global_transform = parent_transform * local_transform;

        for child in node.children() {
            AssetServer::read_node(
                &child,
                scene_loading_data,
                scene_loading_result,
                &global_transform,
            );
        }

        // skip loading camera (hardcoded)

        if let Some(light) = node.light() {
            scene_loading_result
                .lights
                .push(Self::load_light(light, &global_transform));
        }

        if let Some(mesh) = node.mesh() {
            scene_loading_result.models.push((
                global_transform.clone(),
                Self::load_model(mesh, scene_loading_data),
            ));
        }
    }

    fn load_light(light: khr_lights_punctual::Light, global_transform: &Transform) -> Light {
        match light.kind() {
            Kind::Directional => {
                todo!("directional lights are not supported yet")
            }
            Kind::Point => Light::Point(PointLight {
                position: Point3::from(global_transform.translation.vector),
                // TODO: validate implementation (might have mistaken column and row)
                color: Vector3::from_column_slice(&light.color()),
                range: light.range().unwrap_or_else(|| 20.0f32),
                intensity: light.intensity(),
            }),
            Kind::Spot { .. } => {
                todo!("spot lights are not supported yet")
            }
        }
    }

    fn load_model(mesh: gltf::Mesh, scene_loading_data: &mut SceneLoadingData) -> Model {
        let mut model = Model {
            primitives: Vec::new(),
        };
        for primitive in mesh.primitives() {
            let mesh = scene_loading_data.get_mesh(primitive);

            model.primitives.push(crate::scene::model::Primitive {
                mesh,
                // TODO: actually load a material
                material: Arc::new(Material {
                    color: Vector3::new(1.0, 0.0, 1.0),
                    ka: 0.0,
                    kd: 1.0,
                    ks: 0.0,
                    alpha: 1.0,
                }),
            })
        }

        model
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

struct SceneLoadingData<'a> {
    gltf_buffers: Vec<gltf::buffer::Data>,
    gltf_images: Vec<gltf::image::Data>,
    meshes: HashMap<MeshKey, Arc<Mesh>>,
    textures: Vec<Option<Texture>>,
    allocator: &'a dyn MemoryAllocator,
}

struct SceneLoadingResult {
    lights: Vec<Light>,
    models: Vec<(Transform, Model)>,
}
impl SceneLoadingResult {
    fn new() -> Self {
        Self {
            lights: vec![],
            models: vec![],
        }
    }
}

impl<'a> SceneLoadingData<'a> {
    fn new(
        memory_allocator: &'a dyn MemoryAllocator,
        buffers: Vec<gltf::buffer::Data>,
        images: Vec<gltf::image::Data>,
    ) -> Self {
        let mut textures = vec![];
        for _ in 0..images.len() {
            textures.push(None);
        }

        Self {
            gltf_buffers: buffers,
            gltf_images: images,
            meshes: HashMap::new(),
            textures,
            allocator: memory_allocator,
        }
    }

    fn get_mesh(&mut self, primitive: gltf::Primitive) -> Arc<Mesh> {
        assert!(primitive.mode() == gltf::mesh::Mode::Triangles);

        let mesh_key = MeshKey {
            index_buffer_id: primitive.indices().unwrap().index(),
            vertex_buffer_positions_id: primitive.get(&Semantic::Positions).unwrap().index(),
            vertex_buffer_normals_id: primitive.get(&Semantic::Normals).unwrap().index(),
            // TODO: Fallback for missing UVs
            vertex_buffer_uvs_id: primitive.get(&Semantic::TexCoords(0)).unwrap().index(),
        };

        if let Some(mesh) = self.meshes.get(&mesh_key) {
            return mesh.clone();
        } else {
            let reader = primitive.reader(|buffer| Some(&self.gltf_buffers[buffer.index()]));
            let positions = reader.read_positions().unwrap();
            let normals = reader.read_normals().unwrap();
            // let uvs = reader.read_tex_coords(0).unwrap();
            let mut vertices = vec![];

            // zippy zip https://stackoverflow.com/a/71494478/3492994
            for (mut position, mut normal) in positions.zip(normals) {
                position[2] *= -1.0;
                normal[2] *= -1.0;

                vertices.push(MeshVertex {
                    position,
                    normal,
                    // uv: Vector2::from(uv),
                });
            }

            let indices = reader
                .read_indices()
                .map(|indices| indices.into_u32().collect())
                .unwrap_or_else(|| (0..vertices.len()).map(|index| index as u32).collect());
            Mesh::new(vertices, indices, self.allocator)
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct MeshKey {
    index_buffer_id: usize,
    vertex_buffer_positions_id: usize,
    vertex_buffer_normals_id: usize,
    vertex_buffer_uvs_id: usize,
}
