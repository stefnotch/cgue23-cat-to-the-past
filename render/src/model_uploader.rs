use std::sync::Arc;

use bevy_ecs::{
    prelude::Entity,
    query::Without,
    system::{Commands, NonSendMut, Query, Res, Resource},
};
use game_core::asset::Assets;
use scene::model::Model;
use vulkano::{device::Device, memory::allocator::StandardMemoryAllocator};

use crate::{
    context::Context,
    scene::{
        material::Material,
        mesh::Mesh,
        model::{GpuModel, Primitive},
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

pub fn create_gpu_models(
    mut renderer: NonSendMut<Renderer>,
    context: Res<Context>,
    allocator: Res<ModelUploaderAllocator>,
    mut commands: Commands,
    query_models: Query<(Entity, &Model), Without<GpuModel>>,

    mesh_assets: Res<Assets<Mesh>>,
    material_assets: Res<Assets<Material>>,
) {
    for (entity, model) in query_models.iter() {
        let gpu_model = GpuModel {
            primitives: model
                .primitives
                .iter()
                .map(|primitive| {
                    // TODO: Query from assets
                    let mesh = todo!();
                    let material = todo!();
                    Primitive { mesh, material }
                })
                .collect(),
        };
        commands.entity(entity).insert(gpu_model);
    }
}
