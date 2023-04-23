use crate::render::context::Context;
use crate::scene::mesh::MeshVertex;
use crate::scene::model::{Model, Primitive};
use bevy_ecs::prelude::*;
use game_core::time_manager::TimeTracked;
use gltf::khr_lights_punctual::Kind;
use gltf::texture::{MagFilter, MinFilter, WrappingMode};
use gltf::{import, khr_lights_punctual, Node, Semantic};
use math::bounding_box::BoundingBox;
use nalgebra::{Point3, Quaternion, UnitQuaternion, Vector3};
use physics::physics_context::{BoxCollider, RigidBody, RigidBodyType};
use scene::light::{Light, PointLight};
use scene::transform::Transform;
use std::hash::Hash;
use std::iter::repeat;
use std::sync::Arc;
use std::{collections::HashMap, path::Path};
use uuid::Uuid;
use vulkano::memory::allocator::{MemoryAllocator, StandardMemoryAllocator};
use vulkano::sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo};

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
        context: &Context,
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
                context,
            );
        }

        let sphere = Mesh::sphere(10, 16, 0.1, memory_allocator);

        for (transform, light) in scene_loading_result.lights {
            commands.spawn((
                light,
                Model {
                    primitives: vec![Primitive {
                        mesh: sphere.clone(),
                        material: Arc::new(Material::default()),
                    }],
                },
                transform,
            ));
        }

        for (transform, model, has_rigidbody) in scene_loading_result.models {
            let box_collider = BoxCollider {
                bounds: model.bounding_box(),
            };

            if has_rigidbody {
                commands.spawn((
                    model,
                    transform,
                    box_collider,
                    RigidBody(RigidBodyType::Dynamic),
                    TimeTracked::new(),
                ));
            } else {
                commands.spawn((model, transform, box_collider));
            }
        }

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
        context: &Context,
    ) {
        let local_transform: Transform = from_gltf_transform(node.transform());
        let global_transform = parent_transform * local_transform;

        for child in node.children() {
            AssetServer::read_node(
                &child,
                scene_loading_data,
                scene_loading_result,
                &global_transform,
                context,
            );
        }

        // skip loading camera (hardcoded)

        if let Some(light) = node.light() {
            scene_loading_result
                .lights
                .push((global_transform.clone(), Self::load_light(light)));
        }

        let mut rigidbody = false;
        // TODO: Read JSON using serde
        if let Some(extras) = node.extras() {
            rigidbody = extras.get().contains("Rigidbody");
        }

        if let Some(mesh) = node.mesh() {
            scene_loading_result.models.push((
                global_transform.clone(),
                Self::load_model(mesh, scene_loading_data, context),
                rigidbody,
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

    fn load_model(
        mesh: gltf::Mesh,
        scene_loading_data: &mut SceneLoadingData,
        context: &Context,
    ) -> Model {
        let mut model = Model {
            primitives: Vec::new(),
        };

        for primitive in mesh.primitives() {
            let material = scene_loading_data.get_material(&primitive, context);
            let mesh = scene_loading_data.get_mesh(&primitive);

            model.primitives.push(Primitive { mesh, material })
        }

        model
    }
}

fn from_gltf_transform(value: gltf::scene::Transform) -> Transform {
    // rotation is a quaternion
    let (translation, rotation, scale) = value.decomposed();

    let position: Point3<f32> = translation.into();
    let rotation: UnitQuaternion<f32> = UnitQuaternion::new_normalize(Quaternion::from(rotation));
    let scale: Vector3<f32> = Vector3::from_row_slice(&scale);

    Transform {
        position,
        rotation,
        scale,
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
    models: Vec<(Transform, Model, bool)>,
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
        let images = images.into_iter().enumerate().collect();

        Self {
            gltf_buffers: buffers,
            gltf_images: images,
            meshes: HashMap::new(),
            materials: HashMap::new(),
            missing_material: Arc::new(Material::default()),
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

    fn get_material(&mut self, primitive: &gltf::Primitive, context: &Context) -> Arc<Material> {
        let gltf_material = primitive.material();

        if let Some(material_index) = gltf_material.index() {
            if let Some(material) = self.materials.get(&material_index) {
                material.clone()
            } else {
                let gltf_material_pbr = gltf_material.pbr_metallic_roughness();
                let emissive_strength = gltf_material.emissive_strength().unwrap_or(1.0);
                let emissive_factor = gltf_material
                    .emissive_factor()
                    .map(|v| v * emissive_strength);
                let material = Arc::new(Material {
                    base_color: Vector3::from_row_slice(
                        &gltf_material_pbr.base_color_factor()[0..3],
                    ),
                    base_color_texture: gltf_material_pbr.base_color_texture().map(|info| {
                        let sampler = self.get_sampler(&info.texture(), context);
                        self.get_texture(&info.texture(), sampler, context)
                    }),
                    roughness_factor: gltf_material_pbr.roughness_factor(),
                    metallic_factor: gltf_material_pbr.metallic_factor(),
                    emissivity: emissive_factor.into(),
                });

                self.materials.insert(material_index, material.clone());
                material.clone()
            }
        } else {
            self.missing_material.clone()
        }
    }

    fn get_sampler(
        &mut self,
        gltf_texture: &gltf::texture::Texture,
        context: &Context,
    ) -> Arc<Sampler> {
        let sampler = gltf_texture.sampler();

        let min_filter =
            gltf_min_filter_to_vulkano(sampler.min_filter().unwrap_or(MinFilter::Linear));
        let mag_filter =
            gltf_max_filter_to_vulkano(sampler.mag_filter().unwrap_or(MagFilter::Linear));

        let address_mode: [SamplerAddressMode; 3] = [
            gltf_wrapping_mode_to_vulkano(sampler.wrap_s()),
            gltf_wrapping_mode_to_vulkano(sampler.wrap_s()),
            SamplerAddressMode::ClampToEdge,
        ];

        let sampler_key = SamplerKey {
            min_filter,
            mag_filter,
        };

        self.samplers
            .entry(sampler_key)
            .or_insert_with(|| {
                Sampler::new(
                    context.device(),
                    SamplerCreateInfo {
                        mag_filter,
                        min_filter,
                        address_mode,
                        ..SamplerCreateInfo::default()
                    },
                )
                .unwrap()
            })
            .clone()
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

fn gltf_wrapping_mode_to_vulkano(wrapping_mode: WrappingMode) -> SamplerAddressMode {
    match wrapping_mode {
        WrappingMode::ClampToEdge => SamplerAddressMode::ClampToEdge,
        WrappingMode::MirroredRepeat => SamplerAddressMode::MirroredRepeat,
        WrappingMode::Repeat => SamplerAddressMode::Repeat,
    }
}

fn gltf_max_filter_to_vulkano(linear: MagFilter) -> Filter {
    match linear {
        MagFilter::Nearest => Filter::Nearest,
        MagFilter::Linear => Filter::Linear,
    }
}

fn gltf_min_filter_to_vulkano(gltf_min_filter: MinFilter) -> Filter {
    match gltf_min_filter {
        MinFilter::Nearest => Filter::Nearest,
        MinFilter::Linear => Filter::Linear,
        MinFilter::NearestMipmapNearest => Filter::Nearest,
        MinFilter::LinearMipmapNearest => Filter::Linear,
        MinFilter::NearestMipmapLinear => Filter::Linear,
        MinFilter::LinearMipmapLinear => Filter::Linear,
    }
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct MeshKey {
    index_buffer_id: usize,
    vertex_buffer_positions_id: usize,
    vertex_buffer_normals_id: usize,
    vertex_buffer_uvs_id: Option<usize>,
}

#[derive(Eq, PartialEq, Debug, Hash)]
struct SamplerKey {
    min_filter: Filter,
    mag_filter: Filter,
}