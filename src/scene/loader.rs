use crate::context::Context;
use crate::physics_context::BoxCollider;
use crate::scene::light::{Light, PointLight};
use crate::scene::material::NewMaterial;
use crate::scene::mesh::{BoundingBox, MeshVertex};
use crate::scene::model::{Model, Primitive};
use crate::scene::transform::Transform;
use bevy_ecs::prelude::*;
use gltf::khr_lights_punctual::Kind;
use gltf::mesh::util::ReadTexCoords::F32;
use gltf::texture::{MagFilter, MinFilter};
use gltf::{import, khr_lights_punctual, Node, Semantic};
use nalgebra::{Quaternion, Translation3, UnitQuaternion, Vector3};
use std::hash::{Hash, Hasher};
use std::iter::repeat;
use std::sync::Arc;
use std::time::Instant;
use std::{collections::HashMap, path::Path};
use uuid::Uuid;
use vulkano::memory::allocator::{MemoryAllocator, StandardMemoryAllocator};
use vulkano::sampler::{Filter, Sampler, SamplerCreateInfo};

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
        // TODO: open issue on gltf repository (working with buffers and images is unintuitive and not very good documented)
        let (doc, buffers, images) = import(path)?;

        if doc.scenes().len() > 1 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "We shouldn't have more than one scene",
            )));
        }

        let mut scene_loading_data = SceneLoadingData::new(memory_allocator, buffers, images);
        let mut scene_loading_result = SceneLoadingResult::new();

        let scene = doc.default_scene().ok_or("Default scene is not set")?;

        for node in scene.nodes() {
            Self::read_node(
                &node,
                &mut scene_loading_data,
                &mut scene_loading_result,
                &Transform::default(),
            );
        }

        let sphere = Mesh::sphere(10, 16, 0.1, memory_allocator);

        for (transform, light) in scene_loading_result.lights {
            commands.spawn((
                light,
                Model {
                    primitives: vec![Primitive {
                        mesh: sphere.clone(),
                        material: Arc::new(Material {
                            color: Vector3::new(1.0, 1.0, 1.0),
                            ka: 1.0,
                            kd: 0.0,
                            ks: 0.0,
                            alpha: 1.0,
                        }),
                    }],
                },
                transform,
            ));
        }

        println!(
            "{}",
            &scene_loading_result
                .models
                .iter()
                .map(|(_, model)| &model.primitives)
                .flatten()
                .map(|primitive| primitive.mesh.vertices.len())
                .sum::<usize>()
        );

        let before = Instant::now();
        for (transform, model) in scene_loading_result.models {
            let bounding_box = model
                .primitives
                .iter()
                .map(|primitive| &primitive.mesh.bounding_box)
                .fold(BoundingBox::empty(), |a, b| (a.combine(b)));

            let box_collider = BoxCollider {
                bounds: bounding_box,
            };

            commands.spawn((model, transform, box_collider));
        }
        println!(
            "Spawning entities took {}sec",
            before.elapsed().as_secs_f64()
        );

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
                .push((global_transform.clone(), Self::load_light(light)));
        }

        if let Some(mesh) = node.mesh() {
            scene_loading_result.models.push((
                global_transform.clone(),
                Self::load_model(mesh, scene_loading_data),
            ));
        }
    }

    fn load_light(light: khr_lights_punctual::Light) -> Light {
        match light.kind() {
            Kind::Directional => {
                todo!("directional lights are not supported yet")
            }
            Kind::Point => Light::Point(PointLight {
                color: light.color().into(),
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
            let material = scene_loading_data.get_material(&primitive);
            let mesh = scene_loading_data.get_mesh(&primitive);

            model.primitives.push(Primitive { mesh, material })
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
    gltf_images: HashMap<usize, gltf::image::Data>,
    meshes: HashMap<MeshKey, Arc<Mesh>>,
    materials: HashMap<usize, Arc<Material>>,
    missing_material: Arc<Material>,
    samplers: HashMap<SamplerKey, Arc<Sampler>>,
    allocator: &'a dyn MemoryAllocator,
}

struct SceneLoadingResult {
    lights: Vec<(Transform, Light)>,
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
        let material = Arc::new(Material {
            color: Vector3::new(1.0, 0.0, 1.0),
            ka: 0.0,
            kd: 1.0,
            ks: 0.0,
            alpha: 1.0,
        });

        let images = images.into_iter().enumerate().collect();

        Self {
            gltf_buffers: buffers,
            gltf_images: images,
            meshes: HashMap::new(),
            materials: HashMap::new(),
            missing_material: material,
            samplers: HashMap::new(),
            allocator: memory_allocator,
        }
    }

    fn get_mesh(&mut self, primitive: &gltf::Primitive) -> Arc<Mesh> {
        assert_eq!(primitive.mode(), gltf::mesh::Mode::Triangles);

        let mesh_key = MeshKey {
            index_buffer_id: primitive.indices().unwrap().index(),
            vertex_buffer_positions_id: primitive.get(&Semantic::Positions).unwrap().index(),
            vertex_buffer_normals_id: primitive.get(&Semantic::Normals).unwrap().index(),
            vertex_buffer_uvs_id: primitive.get(&Semantic::TexCoords(0)).map(|a| a.index()),
        };

        self.meshes
            .entry(mesh_key)
            .or_insert_with(|| {
                let reader = primitive.reader(|buffer| Some(&self.gltf_buffers[buffer.index()]));
                let positions = reader.read_positions().unwrap();
                let normals = reader.read_normals().unwrap();
                let uvs: Box<dyn Iterator<Item = _>> =
                    if let Some(read_tex_coords) = reader.read_tex_coords(0) {
                        Box::new(read_tex_coords.into_f32())
                    } else {
                        Box::new(repeat([0.0f32, 0.0f32]))
                    };

                let mut vertices = vec![];

                // zippy zip https://stackoverflow.com/a/71494478/3492994
                for (position, (normal, uv)) in positions.zip(normals.zip(uvs)) {
                    vertices.push(MeshVertex {
                        position,
                        normal,
                        uv,
                    });
                }

                let indices = reader
                    .read_indices()
                    .map(|indices| indices.into_u32().collect())
                    .unwrap_or_else(|| (0..vertices.len()).map(|index| index as u32).collect());

                let gltf_bounding_box = primitive.bounding_box();
                let bounding_box = BoundingBox::<Vector3<f32>>::new(
                    gltf_bounding_box.min.into(),
                    gltf_bounding_box.max.into(),
                );

                Mesh::new(vertices, indices, bounding_box, self.allocator)
            })
            .clone()
    }

    fn get_material(&mut self, primitive: &gltf::Primitive) -> Arc<Material> {
        let gltf_material = primitive.material();

        if let Some(material_index) = gltf_material.index() {
            self.materials
                .entry(material_index)
                .or_insert_with(|| {
                    let gltf_material_pbr = gltf_material.pbr_metallic_roughness();
                    let material = Material {
                        color: Vector3::from_row_slice(
                            &gltf_material_pbr.base_color_factor()[0..3],
                        ),
                        ka: 0.1,
                        kd: 0.4,
                        ks: 0.0,
                        alpha: 1.0,
                    };
                    // let material = NewMaterial {
                    //     base_color: Vector3::from_row_slice(&gltf_material_pbr.base_color_factor()[0..2]),
                    //     base_color_texture: None,
                    //     normal_texture: None,
                    //     emissivity: gltf_material.emissive_factor().into(),
                    //     metallic_factor: gltf_material_pbr.metallic_factor(),
                    //     roughness_factor: gltf_material_pbr.roughness_factor(),
                    // };
                    Arc::new(material)
                })
                .clone()
        } else {
            self.missing_material.clone()
        }
    }

    fn get_sampler(&self, gltf_texture: gltf::texture::Texture, context: &Context) -> Arc<Sampler> {
        let sampler = gltf_texture.sampler();

        let min_filter = sampler.min_filter().unwrap_or(MinFilter::Linear);
        let mag_filter = sampler.mag_filter().unwrap_or(MagFilter::Linear);

        let sampler_key = SamplerKey {
            min_filter,
            mag_filter,
        };

        if let Some(sampler) = self.samplers.get(&sampler_key) {
            return sampler.clone();
        } else {
            Sampler::new(
                context.device(),
                SamplerCreateInfo {
                    // TODO: use right filter
                    mag_filter: Filter::Linear,
                    min_filter: Filter::Linear,
                    ..SamplerCreateInfo::default()
                },
            )
            .unwrap()
        }
    }

    fn get_texture(
        &mut self,
        gltf_texture: &gltf::texture::Texture,
        sampler: Arc<Sampler>,
        context: &Context,
    ) -> Arc<Texture> {
        Texture::from_gltf_image(
            self.gltf_images
                .remove(&(gltf_texture.source().index() as usize))
                .unwrap(),
            sampler,
            context,
        )
    }
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct MeshKey {
    index_buffer_id: usize,
    vertex_buffer_positions_id: usize,
    vertex_buffer_normals_id: usize,
    vertex_buffer_uvs_id: Option<usize>,
}

#[derive(Eq, PartialEq, Debug)]
struct SamplerKey {
    min_filter: MinFilter,
    mag_filter: MagFilter,
}

impl Hash for SamplerKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // TODO: validate implementation
        self.min_filter.as_gl_enum().hash(state);
        self.mag_filter.as_gl_enum().hash(state);
    }
}
