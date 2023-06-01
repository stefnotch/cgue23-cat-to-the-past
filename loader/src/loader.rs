use animations::animation::{Animation, PlayingAnimation};
use bevy_ecs::prelude::*;
use game_core::time_manager::level_time::LevelTime;
use game_core::time_manager::TimeTracked;
use gltf::khr_lights_punctual::Kind;
use gltf::texture::{MagFilter, MinFilter, WrappingMode};
use gltf::{import, khr_lights_punctual, Node, Semantic};
use math::bounding_box::BoundingBox;
use nalgebra::{Point3, Quaternion, UnitQuaternion, Vector3};
use physics::physics_context::{BoxCollider, RigidBody, Sensor};
use scene::asset::AssetId;
use scene::debug_name::DebugName;
use scene::light::{CastShadow, Light, PointLight};
use scene::material::CpuMaterial;
use scene::mesh::{CpuMesh, CpuMeshVertex};
use scene::model::{CpuPrimitive, Model};
use scene::texture::{
    AddressMode, BytesTextureData, CpuTexture, Filter, SamplerInfo, TextureFormat,
};
use scene::transform::Transform;
use std::hash::Hash;
use std::iter::repeat;
use std::ops::Add;
use std::sync::Arc;
use std::time::Duration;
use std::{collections::HashMap, path::Path};

use game_core::pickup::Pickupable;
use physics::physics_context::RigidBodyType::{Dynamic, KinematicPositionBased};
use serde::Deserialize;

// scene.json -> assets

// list of assets in code

#[derive(Component)]
pub struct Door {
    /// true if the door is open *or if it is opening*
    pub is_open: bool,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct AnimationProperty {
    pub translation: [f32; 3],
    pub duration: f32,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
struct GLTFExtras {
    pub sensor: Option<u32>,
    pub box_collider: Option<bool>,
    pub rigid_body: Option<String>,
    pub animation: Option<AnimationProperty>,
    pub door: Option<bool>,
    pub pickupable: Option<bool>,
}

#[derive(Resource)]
pub struct SceneLoader {}

impl SceneLoader {
    /// loads one .gltf file
    pub fn load_default_scene<P>(
        &self,
        path: P,
        commands: &mut Commands,
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

        let mut scene_loading_data = SceneLoadingData::new(buffers, images);
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

        let sphere = CpuMesh::sphere(10, 16, 0.1);

        for (transform, light, name) in scene_loading_result.lights {
            commands.spawn((
                name,
                light,
                CastShadow,
                Model {
                    primitives: vec![CpuPrimitive {
                        mesh: sphere.clone(),
                        material: Arc::new(CpuMaterial::default()),
                    }],
                },
                transform,
            ));
        }

        for (transform, model, extras, name) in scene_loading_result.models {
            let box_collider = BoxCollider {
                bounds: model.bounding_box(),
            };

            let mut entity = commands.spawn((name, transform.clone()));

            if let Some(_) = extras.sensor {
                // and sensor component
                entity.insert(Sensor);
            } else {
                // add model component
                entity.insert(model);
            }

            // add box collider component
            if let Some(true) = extras.box_collider {
                entity.insert(box_collider);
            }

            if let Some(str) = extras.rigid_body {
                if str == "kinematic" {
                    entity.insert((RigidBody(KinematicPositionBased), TimeTracked::new()));
                } else if str == "dynamic" {
                    entity.insert((RigidBody(Dynamic), TimeTracked::new()));
                } else {
                    panic!("Unknown rigid_body type: {}", str);
                }
            }

            if let Some(is_open) = extras.door {
                entity.insert(Door { is_open });
            }

            if let Some(animation) = extras.animation {
                let start_transform = transform.clone();
                let mut end_transform = transform.clone();
                let test: Vector3<f32> = animation.translation.into();
                end_transform.position = end_transform.position.add(test);

                let animation = Animation {
                    start_transform,
                    end_transform,
                    duration: Duration::from_secs_f32(animation.duration),
                };

                let playing_animation = PlayingAnimation {
                    animation,
                    end_time: LevelTime::zero(),
                    reverse: true,
                };

                entity.insert(playing_animation);

                // May not have a time tracked if it's animated
                entity.remove::<TimeTracked>();
            }

            if let Some(true) = extras.pickupable {
                entity.insert(Pickupable);
            }
        }

        Ok(())
    }

    pub fn new() -> Self {
        SceneLoader {}
    }

    fn read_node(
        node: &Node,
        scene_loading_data: &mut SceneLoadingData,
        scene_loading_result: &mut SceneLoadingResult,
        parent_transform: &Transform,
    ) {
        let local_transform: Transform = from_gltf_transform(node.transform());
        let global_transform = parent_transform * local_transform;

        for child in node.children() {
            SceneLoader::read_node(
                &child,
                scene_loading_data,
                scene_loading_result,
                &global_transform,
            );
        }

        // skip loading camera (hardcoded)

        if let Some(light) = node.light() {
            scene_loading_result.lights.push((
                global_transform.clone(),
                Self::load_light(light),
                DebugName(node.name().unwrap_or_default().to_string()),
            ));
        }

        let extras = node
            .extras()
            .as_ref()
            .map(|extra| {
                let str = extra.get();

                let result: GLTFExtras = serde_json::from_str(str).expect(str);

                result
            })
            .unwrap_or_default();

        if let Some(mesh) = node.mesh() {
            scene_loading_result.models.push((
                global_transform.clone(),
                Self::load_model(mesh, scene_loading_data),
                extras,
                DebugName(node.name().unwrap_or_default().to_string()),
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
                // TODO: What if a point light doesn't have a range?
                range: light.range().unwrap_or(20.0f32),
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

            model.primitives.push(CpuPrimitive { mesh, material })
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

struct SceneLoadingData {
    gltf_buffers: Vec<gltf::buffer::Data>,
    gltf_images: HashMap<usize, gltf::image::Data>,
    meshes: HashMap<MeshKey, Arc<CpuMesh>>,
    materials: HashMap<usize, Arc<CpuMaterial>>,
    missing_material: Arc<CpuMaterial>,
}

struct SceneLoadingResult {
    lights: Vec<(Transform, Light, DebugName)>,
    models: Vec<(Transform, Model, GLTFExtras, DebugName)>,
}
impl SceneLoadingResult {
    fn new() -> Self {
        Self {
            lights: vec![],
            models: vec![],
        }
    }
}

impl SceneLoadingData {
    fn new(buffers: Vec<gltf::buffer::Data>, images: Vec<gltf::image::Data>) -> Self {
        let images = images.into_iter().enumerate().collect();

        Self {
            gltf_buffers: buffers,
            gltf_images: images,
            meshes: HashMap::new(),
            materials: HashMap::new(),
            missing_material: Arc::new(CpuMaterial::default()),
        }
    }

    fn get_mesh(&mut self, primitive: &gltf::Primitive) -> Arc<CpuMesh> {
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
                    vertices.push(CpuMeshVertex {
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

                CpuMesh::new(vertices, indices, bounding_box)
            })
            .clone()
    }

    fn get_material(&mut self, primitive: &gltf::Primitive) -> Arc<CpuMaterial> {
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
                let material = Arc::new(CpuMaterial {
                    id: AssetId::new_v4(),
                    base_color: Vector3::from_row_slice(
                        &gltf_material_pbr.base_color_factor()[0..3],
                    ),
                    base_color_texture: gltf_material_pbr.base_color_texture().map(|info| {
                        let sampler = self.get_sampler(&info.texture());
                        self.get_texture(&info.texture(), sampler)
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

    fn get_sampler(&mut self, gltf_texture: &gltf::texture::Texture) -> SamplerInfo {
        let sampler = gltf_texture.sampler();

        let min_filter = from_gltf_min_filter(sampler.min_filter().unwrap_or(MinFilter::Linear));
        let mag_filter = from_gltf_max_filter(sampler.mag_filter().unwrap_or(MagFilter::Linear));

        let address_mode: [AddressMode; 3] = [
            from_gltf_wrapping_mode(sampler.wrap_s()),
            from_gltf_wrapping_mode(sampler.wrap_s()),
            AddressMode::ClampToEdge,
        ];
        SamplerInfo {
            min_filter,
            mag_filter,
            address_mode,
        }
    }

    fn get_texture(
        &mut self,
        gltf_texture: &gltf::texture::Texture,
        sampler: SamplerInfo,
    ) -> Arc<CpuTexture> {
        gltf_texture_to_cpu_texture(
            self.gltf_images
                .remove(&(gltf_texture.source().index()))
                .unwrap(),
            sampler,
        )
    }
}

fn gltf_texture_to_cpu_texture(
    image_data: gltf::image::Data,
    sampler: SamplerInfo,
) -> Arc<CpuTexture> {
    // Widely supported formats https://vulkan.gpuinfo.org/listlineartilingformats.php

    let width = image_data.width;
    let height = image_data.height;
    let (image, format) = gltf_image_format_to_vulkan_format(image_data.pixels, &image_data.format);
    Arc::new(CpuTexture {
        id: AssetId::new_v4(),
        data: Box::new(BytesTextureData::new((width, height), format, image)),
        sampler_info: sampler,
    })
}

fn from_gltf_wrapping_mode(wrapping_mode: WrappingMode) -> AddressMode {
    match wrapping_mode {
        WrappingMode::ClampToEdge => AddressMode::ClampToEdge,
        WrappingMode::MirroredRepeat => AddressMode::MirroredRepeat,
        WrappingMode::Repeat => AddressMode::Repeat,
    }
}

fn from_gltf_max_filter(linear: MagFilter) -> Filter {
    match linear {
        MagFilter::Nearest => Filter::Nearest,
        MagFilter::Linear => Filter::Linear,
    }
}

fn from_gltf_min_filter(gltf_min_filter: MinFilter) -> Filter {
    match gltf_min_filter {
        MinFilter::Nearest => Filter::Nearest,
        MinFilter::Linear => Filter::Linear,
        MinFilter::NearestMipmapNearest => Filter::Nearest,
        MinFilter::LinearMipmapNearest => Filter::Linear,
        MinFilter::NearestMipmapLinear => Filter::Linear,
        MinFilter::LinearMipmapLinear => Filter::Linear,
    }
}

fn gltf_image_format_to_vulkan_format(
    image: Vec<u8>,
    format: &gltf::image::Format,
) -> (Vec<u8>, TextureFormat) {
    match format {
        gltf::image::Format::R8 => (image, TextureFormat::R8_UNORM),
        gltf::image::Format::R8G8 => (image, TextureFormat::R8G8_UNORM),
        gltf::image::Format::R8G8B8 => {
            // rarely supported format
            let mut image_with_alpha = Vec::new();
            for i in 0..image.len() / 3 {
                image_with_alpha.push(image[i * 3]);
                image_with_alpha.push(image[i * 3 + 1]);
                image_with_alpha.push(image[i * 3 + 2]);
                image_with_alpha.push(255);
            }
            (image_with_alpha, TextureFormat::R8G8B8A8_UNORM)
        }
        gltf::image::Format::R8G8B8A8 => (image, TextureFormat::R8G8B8A8_UNORM),
        gltf::image::Format::R16 => (image, TextureFormat::R16_UNORM),
        gltf::image::Format::R16G16 => (image, TextureFormat::R16G16_UNORM),
        gltf::image::Format::R16G16B16 => {
            // rarely supported format
            todo!()
        }
        gltf::image::Format::R16G16B16A16 => (image, TextureFormat::R16G16B16A16_UNORM),
        gltf::image::Format::R32G32B32FLOAT => {
            // rarely supported format
            todo!()
        }
        gltf::image::Format::R32G32B32A32FLOAT => (image, TextureFormat::R32G32B32A32_SFLOAT),
    }
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct MeshKey {
    index_buffer_id: usize,
    vertex_buffer_positions_id: usize,
    vertex_buffer_normals_id: usize,
    vertex_buffer_uvs_id: Option<usize>,
}
