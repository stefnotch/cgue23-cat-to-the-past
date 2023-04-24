use std::{collections::HashMap, sync::Arc};

use bevy_ecs::{
    prelude::Entity,
    query::Without,
    system::{Commands, NonSendMut, Query, Res, ResMut, Resource},
};
use game_core::asset::{Asset, Assets};
use scene::{
    material::CpuMaterial,
    mesh::CpuMesh,
    model::Model,
    texture::{CpuTexture, SamplerInfo},
};
use vulkano::{
    device::Device,
    memory::allocator::StandardMemoryAllocator,
    sampler::{Sampler, SamplerCreateInfo},
};

use crate::{
    context::Context,
    scene::{
        material::Material,
        mesh::Mesh,
        model::{GpuModel, Primitive},
        texture::Texture,
    },
    Renderer,
};

#[derive(Resource)]
pub struct ModelUploaderAllocator {
    allocator: Arc<StandardMemoryAllocator>,
}
impl ModelUploaderAllocator {
    pub fn new(device: Arc<Device>) -> Self {
        Self {
            allocator: Arc::new(StandardMemoryAllocator::new_default(device)),
        }
    }
}

#[derive(Resource)]
pub struct SamplerInfoMap {
    samplers: HashMap<SamplerInfo, Arc<Sampler>>,
}
impl SamplerInfoMap {
    pub fn new() -> Self {
        Self {
            samplers: HashMap::new(),
        }
    }
}

pub fn create_gpu_models(
    context: Res<Context>,
    allocator: Res<ModelUploaderAllocator>,
    mut commands: Commands,
    query_models: Query<(Entity, &Model), Without<GpuModel>>,

    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<Material>>,
    mut texture_assets: ResMut<Assets<Texture>>,
    mut samplers: ResMut<SamplerInfoMap>,
) {
    for (entity, model) in query_models.iter() {
        let primitives = model
            .primitives
            .iter()
            .map(|primitive| {
                let mesh = create_gpu_mesh(&mut mesh_assets, &primitive.mesh, &allocator);

                let material = create_gpu_material(
                    &mut material_assets,
                    &mut texture_assets,
                    &mut samplers,
                    &primitive.material,
                    &context,
                );
                Primitive { mesh, material }
            })
            .collect();

        let gpu_model = GpuModel { primitives };
        commands.entity(entity).insert(gpu_model);
    }
}

fn create_gpu_mesh(
    mesh_assets: &mut Assets<Mesh>,
    mesh: &CpuMesh,
    allocator: &ModelUploaderAllocator,
) -> Arc<Mesh> {
    mesh_assets
        .assets
        .entry(mesh.id())
        .or_insert_with(|| {
            Mesh::new(
                mesh.id(),
                mesh.vertices.iter().map(|vertex| vertex.into()).collect(),
                mesh.indices.clone(),
                mesh.bounding_box.clone(),
                &allocator.allocator,
            )
        })
        .to_owned()
}

fn create_gpu_material(
    material_assets: &mut Assets<Material>,
    texture_assets: &mut Assets<Texture>,
    samplers: &mut SamplerInfoMap,
    material: &CpuMaterial,
    context: &Context,
) -> Arc<Material> {
    material_assets
        .assets
        .entry(material.id())
        .or_insert_with(|| {
            Arc::new(Material {
                id: material.id(),
                base_color: material.base_color,
                base_color_texture: material
                    .base_color_texture
                    .as_ref()
                    .map(|texture| create_gpu_texture(texture_assets, samplers, texture, context)),
                roughness_factor: material.roughness_factor,
                metallic_factor: material.metallic_factor,
                emissivity: material.emissivity,
            })
        })
        .to_owned()
}

fn create_gpu_texture(
    texture_assets: &mut Assets<Texture>,
    samplers: &mut SamplerInfoMap,
    texture: &CpuTexture,
    context: &Context,
) -> Arc<Texture> {
    texture_assets
        .assets
        .entry(texture.id())
        .or_insert_with(|| {
            let (width, height) = texture.data.dimensions();
            let a: Vec<u8> = texture.data.bytes().into();
            Texture::new(
                texture.id(),
                a,
                width,
                height,
                to_vulkano_format(texture.data.format()),
                create_gpu_sampler(samplers, &texture.sampler_info, context),
                context,
            )
        })
        .to_owned()
}

fn to_vulkano_format(format: &scene::texture::TextureFormat) -> vulkano::format::Format {
    match format {
        scene::texture::TextureFormat::R8_UNORM => vulkano::format::Format::R8_UNORM,
        scene::texture::TextureFormat::R8G8_UNORM => vulkano::format::Format::R8G8_UNORM,
        scene::texture::TextureFormat::R8G8B8A8_UNORM => vulkano::format::Format::R8G8B8A8_UNORM,
        scene::texture::TextureFormat::R16_UNORM => vulkano::format::Format::R16_UNORM,
        scene::texture::TextureFormat::R16G16_UNORM => vulkano::format::Format::R16G16_UNORM,
        scene::texture::TextureFormat::R16G16B16A16_UNORM => {
            vulkano::format::Format::R16G16B16A16_UNORM
        }
        scene::texture::TextureFormat::R32G32B32A32_SFLOAT => {
            vulkano::format::Format::R32G32B32A32_SFLOAT
        }
    }
}

fn create_gpu_sampler(
    samplers: &mut SamplerInfoMap,
    sampler_info: &SamplerInfo,
    context: &Context,
) -> Arc<Sampler> {
    samplers
        .samplers
        .entry(sampler_info.clone())
        .or_insert_with(|| {
            Sampler::new(
                context.device(),
                SamplerCreateInfo {
                    mag_filter: to_vulkano_filter(sampler_info.mag_filter),
                    min_filter: to_vulkano_filter(sampler_info.min_filter),
                    address_mode: to_vulkano_address_mode(sampler_info.address_mode),
                    ..SamplerCreateInfo::default()
                },
            )
            .unwrap()
        })
        .to_owned()
}

fn to_vulkano_address_mode(
    address_mode: [scene::texture::AddressMode; 3],
) -> [vulkano::sampler::SamplerAddressMode; 3] {
    [
        to_vulkano_address_mode_single(address_mode[0]),
        to_vulkano_address_mode_single(address_mode[1]),
        to_vulkano_address_mode_single(address_mode[2]),
    ]
}

fn to_vulkano_address_mode_single(
    address_mode: scene::texture::AddressMode,
) -> vulkano::sampler::SamplerAddressMode {
    match address_mode {
        scene::texture::AddressMode::ClampToEdge => {
            vulkano::sampler::SamplerAddressMode::ClampToEdge
        }
        scene::texture::AddressMode::Repeat => vulkano::sampler::SamplerAddressMode::Repeat,
        scene::texture::AddressMode::MirroredRepeat => {
            vulkano::sampler::SamplerAddressMode::MirroredRepeat
        }
        scene::texture::AddressMode::ClampToBorder => {
            vulkano::sampler::SamplerAddressMode::ClampToBorder
        }
    }
}

fn to_vulkano_filter(mag_filter: scene::texture::Filter) -> vulkano::sampler::Filter {
    match mag_filter {
        scene::texture::Filter::Nearest => vulkano::sampler::Filter::Nearest,
        scene::texture::Filter::Linear => vulkano::sampler::Filter::Linear,
    }
}