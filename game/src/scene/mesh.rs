use math::bounding_box::BoundingBox;
use nalgebra::Vector3;
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryUsage};
use vulkano::pipeline::graphics::vertex_input::Vertex;

#[repr(C)]
#[derive(BufferContents, Vertex, Clone)]
pub struct MeshVertex {
    #[format(R32G32B32_SFLOAT)]
    pub(super) position: [f32; 3],

    #[format(R32G32B32_SFLOAT)]
    pub(super) normal: [f32; 3],

    #[format(R32G32_SFLOAT)]
    pub(super) uv: [f32; 2],
}

pub struct Mesh {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,

    pub vertex_buffer: Subbuffer<[MeshVertex]>,
    pub index_buffer: Subbuffer<[u32]>,

    pub bounding_box: BoundingBox<Vector3<f32>>,
}

impl Mesh {
    pub fn new(
        vertices: Vec<MeshVertex>,
        indices: Vec<u32>,
        bounding_box: BoundingBox<Vector3<f32>>,
        allocator: &(impl MemoryAllocator + ?Sized),
    ) -> Arc<Self> {
        let (vertex_buffer, index_buffer) = Mesh::setup_buffers(&vertices, &indices, allocator);

        Arc::new(Self {
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
            bounding_box,
        })
    }

    fn setup_buffers(
        vertices: &[MeshVertex],
        indices: &[u32],
        allocator: &(impl MemoryAllocator + ?Sized),
    ) -> (Subbuffer<[MeshVertex]>, Subbuffer<[u32]>) {
        let vertex_buffer = Buffer::from_iter(
            allocator,
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            vertices.iter().cloned(),
        )
        .expect("could not upload vertex data to GPU");

        let index_buffer = Buffer::from_iter(
            allocator,
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            indices.iter().cloned(),
        )
        .expect("could not upload indices data to GPU");

        (vertex_buffer, index_buffer)
    }
}
