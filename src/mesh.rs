use std::sync::Arc;
use bytemuck::{Pod, Zeroable};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::memory::allocator::MemoryAllocator;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct MeshVertex {
    position: [f32; 3],
}

vulkano::impl_vertex!(MeshVertex, position);

pub struct Mesh {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,

    pub vertex_buffer: Arc<CpuAccessibleBuffer<[MeshVertex]>>,
    pub index_buffer: Arc<CpuAccessibleBuffer<[u32]>>,
}

impl Mesh {
    pub fn cube(width: f32, height: f32, depth: f32, allocator: &(impl MemoryAllocator + ?Sized)) -> Arc<Self> {
        let vertices = vec![
            // front
            MeshVertex { position: [-0.5, -0.5, 0.5] },
            MeshVertex { position: [0.5, -0.5, 0.5] },
            MeshVertex { position: [0.5, 0.5, 0.5] },
            MeshVertex { position: [-0.5, 0.5, 0.5] },

            // back
            MeshVertex { position: [-0.5, -0.5, -0.5] },
            MeshVertex { position: [0.5, -0.5, -0.5] },
            MeshVertex { position: [0.5, 0.5, -0.5] },
            MeshVertex { position: [-0.5, 0.5, -0.5] },
        ];

        let vertices: Vec<MeshVertex> = vertices
            .into_iter()
            .map(|mut vertex| {
                vertex.position[0] *= width;
                vertex.position[1] *= height;
                vertex.position[2] *= depth;

                vertex
            })
            .collect();

        let indices = vec![
            // front
            0, 1, 2, 0, 2, 3,
            // back
            5, 4, 7, 5, 7, 6,
            // right
            1, 5, 6, 1, 6, 2,
            // left
            4, 0, 3, 4, 3, 7,
            // up
            3, 2, 6, 3, 6, 7,
            // down
            1, 0, 4, 1, 4, 5,
        ];

        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            allocator,
            BufferUsage {
                vertex_buffer: true,
                ..Default::default()
            },
            false,
            vertices.iter().cloned(),
        ).expect("could not upload vertex data to GPU");

        let index_buffer = CpuAccessibleBuffer::from_iter(
            allocator,
            BufferUsage {
                index_buffer: true,
                ..Default::default()
            },
            false,
            indices.iter().cloned(),
        ).expect("could not upload indices data to GPU");

        Arc::new(Mesh {
            vertices,
            indices,

            vertex_buffer,
            index_buffer,
        })
    }
}