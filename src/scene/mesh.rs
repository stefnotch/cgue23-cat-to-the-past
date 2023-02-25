use bytemuck::{Pod, Zeroable};
use cgmath::Vector3;
use std::sync::Arc;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::memory::allocator::MemoryAllocator;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct MeshVertex {
    position: [f32; 3],
    normal: [f32; 3],
}

vulkano::impl_vertex!(MeshVertex, position, normal);

pub struct Mesh {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,

    pub vertex_buffer: Arc<CpuAccessibleBuffer<[MeshVertex]>>,
    pub index_buffer: Arc<CpuAccessibleBuffer<[u32]>>,
}

impl Mesh {
    pub fn cube(
        width: f32,
        height: f32,
        depth: f32,
        allocator: &(impl MemoryAllocator + ?Sized),
    ) -> Arc<Self> {
        struct CubeFace {
            position_indices: [usize; 4],
            normal: Vector3<f32>,
        }

        let positions: [Vector3<f32>; 8] = [
            // front
            Vector3::new(-0.5, -0.5, 0.5),
            Vector3::new(0.5, -0.5, 0.5),
            Vector3::new(0.5, 0.5, 0.5),
            Vector3::new(-0.5, 0.5, 0.5),
            // back
            Vector3::new(-0.5, -0.5, -0.5),
            Vector3::new(0.5, -0.5, -0.5),
            Vector3::new(0.5, 0.5, -0.5),
            Vector3::new(-0.5, 0.5, -0.5),
        ];

        let faces: [CubeFace; 6] = [
            // front
            CubeFace {
                position_indices: [0, 1, 2, 3],
                normal: Vector3::new(0.0, 0.0, 1.0),
            },
            // back
            CubeFace {
                position_indices: [5, 4, 7, 6],
                normal: Vector3::new(0.0, 0.0, -1.0),
            },
            // right
            CubeFace {
                position_indices: [1, 5, 6, 2],
                normal: Vector3::new(1.0, 0.0, 0.0),
            },
            // left
            CubeFace {
                position_indices: [4, 0, 3, 7],
                normal: Vector3::new(-1.0, 0.0, 0.0),
            },
            // up
            CubeFace {
                position_indices: [3, 2, 6, 7],
                normal: Vector3::new(0.0, 1.0, 0.0),
            },
            // down
            CubeFace {
                position_indices: [1, 0, 4, 5],
                normal: Vector3::new(0.0, -1.0, 0.0),
            },
        ];

        let vertices: Vec<MeshVertex> = faces
            .iter()
            .flat_map(|face| {
                face.position_indices.map(|i| MeshVertex {
                    position: positions[i].into(),
                    normal: face.normal.into(),
                })
            })
            .collect();

        let vertices: Vec<MeshVertex> = vertices
            .into_iter()
            .map(|mut vertex| {
                vertex.position[0] *= width;
                vertex.position[1] *= height;
                vertex.position[2] *= depth;

                vertex
            })
            .collect();

        let face_indices_schema = [
            0, 1, 2, // bottom right
            2, 3, 0, // top left
        ];

        let indices: Vec<u32> = faces
            .iter()
            .enumerate()
            .flat_map(|(face_index, _)| {
                let offset = 4 * face_index as u32;
                face_indices_schema.map(|i| offset + i)
            })
            .collect();

        let (vertex_buffer, index_buffer) = Mesh::setup_buffer(&vertices, &indices, allocator);

        Arc::new(Mesh {
            vertices,
            indices,

            vertex_buffer,
            index_buffer,
        })
    }

    pub fn plane_horizontal(
        width: f32,
        height: f32,
        allocator: &(impl MemoryAllocator + ?Sized),
    ) -> Arc<Self> {
        let vertices = vec![
            MeshVertex {
                position: Vector3::new(-0.5 * width, 0.0, -0.5 * height).into(),
                normal: Vector3::unit_y().into(),
            },
            MeshVertex {
                position: Vector3::new(0.5 * width, 0.0, -0.5 * height).into(),
                normal: Vector3::unit_y().into(),
            },
            MeshVertex {
                position: Vector3::new(0.5 * width, 0.0, 0.5 * height).into(),
                normal: Vector3::unit_y().into(),
            },
            MeshVertex {
                position: Vector3::new(-0.5 * width, 0.0, 0.5 * height).into(),
                normal: Vector3::unit_y().into(),
            },
        ];

        let indices: Vec<u32> = vec![0, 1, 2, 2, 3, 0];

        let (vertex_buffer, index_buffer) = Mesh::setup_buffer(&vertices, &indices, allocator);

        Arc::new(Mesh {
            vertices,
            indices,

            vertex_buffer,
            index_buffer,
        })
    }

    fn setup_buffer(
        vertices: &[MeshVertex],
        indices: &[u32],
        allocator: &(impl MemoryAllocator + ?Sized),
    ) -> (
        Arc<CpuAccessibleBuffer<[MeshVertex]>>,
        Arc<CpuAccessibleBuffer<[u32]>>,
    ) {
        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            allocator,
            BufferUsage {
                vertex_buffer: true,
                ..Default::default()
            },
            false,
            vertices.iter().cloned(),
        )
        .expect("could not upload vertex data to GPU");

        let index_buffer = CpuAccessibleBuffer::from_iter(
            allocator,
            BufferUsage {
                index_buffer: true,
                ..Default::default()
            },
            false,
            indices.iter().cloned(),
        )
        .expect("could not upload indices data to GPU");

        (vertex_buffer, index_buffer)
    }
}
