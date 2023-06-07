use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryUsage, StandardMemoryAllocator};
use vulkano::pipeline::graphics::vertex_input::Vertex;

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct QuadVertex {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],

    #[format(R32G32_SFLOAT)]
    pub uv: [f32; 2],
}

pub const fn quad_mesh() -> ([QuadVertex; 4], [u32; 6]) {
    let vertices = [
        QuadVertex {
            position: [-1.0, -1.0],
            uv: [0.0, 0.0],
        },
        QuadVertex {
            position: [1.0, -1.0],
            uv: [1.0, 0.0],
        },
        QuadVertex {
            position: [1.0, 1.0],
            uv: [1.0, 1.0],
        },
        QuadVertex {
            position: [-1.0, 1.0],
            uv: [0.0, 1.0],
        },
    ];

    let indices = [0, 1, 2, 2, 3, 0];

    (vertices, indices)
}

pub fn create_geometry_buffers(
    memory_allocator: Arc<StandardMemoryAllocator>,
) -> (Subbuffer<[QuadVertex]>, Subbuffer<[u32]>) {
    let (vertices, indices) = quad_mesh();
    let vertex_buffer = Buffer::from_iter(
        &memory_allocator,
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Upload,
            ..Default::default()
        },
        vertices,
    )
    .unwrap();

    let index_buffer = Buffer::from_iter(
        &memory_allocator,
        BufferCreateInfo {
            usage: BufferUsage::INDEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Upload,
            ..Default::default()
        },
        indices,
    )
    .unwrap();

    (vertex_buffer, index_buffer)
}
