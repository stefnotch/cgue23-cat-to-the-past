use crate::scene::bounding_box::BoundingBox;
use bytemuck::{Pod, Zeroable};
use nalgebra::{Vector2, Vector3};
use std::f32::consts::PI;
use std::sync::Arc;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::memory::allocator::MemoryAllocator;

use super::loader::Asset;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct MeshVertex {
    pub(super) position: [f32; 3],
    pub(super) normal: [f32; 3],
    pub(super) uv: [f32; 2],
}

vulkano::impl_vertex!(MeshVertex, position, normal, uv);

pub struct Mesh {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,

    pub vertex_buffer: Arc<CpuAccessibleBuffer<[MeshVertex]>>,
    pub index_buffer: Arc<CpuAccessibleBuffer<[u32]>>,

    pub bounding_box: BoundingBox<Vector3<f32>>,
}
impl Asset for Mesh {}

impl Mesh {
    pub fn new(
        vertices: Vec<MeshVertex>,
        indices: Vec<u32>,
        bounding_box: BoundingBox<Vector3<f32>>,
        allocator: &(impl MemoryAllocator + ?Sized),
    ) -> Arc<Self> {
        let (vertex_buffer, index_buffer) = Mesh::setup_buffer(&vertices, &indices, allocator);

        Arc::new(Self {
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
            bounding_box,
        })
    }

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

        let uvs_face: [Vector2<f32>; 4] = [
            Vector2::new(0.0, 1.0),
            Vector2::new(1.0, 1.0),
            Vector2::new(1.0, 0.0),
            Vector2::new(0.0, 0.0),
        ];

        let vertices: Vec<MeshVertex> = faces
            .iter()
            .flat_map(|face| {
                face.position_indices
                    .iter()
                    .enumerate()
                    .map(|(i, pos_index)| MeshVertex {
                        position: positions[*pos_index].into(),
                        normal: face.normal.into(),
                        uv: uvs_face[i].into(),
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
            bounding_box: BoundingBox::new(
                Vector3::new(-width / 2.0, -height / 2.0, -depth / 2.0),
                Vector3::new(width / 2.0, height / 2.0, depth / 2.0),
            ),
        })
    }

    pub fn sphere(
        longitude_segments: u32,
        latitude_segments: u32,
        radius: f32,
        allocator: &impl MemoryAllocator,
    ) -> Arc<Self> {
        let mut vertices: Vec<MeshVertex> = vec![];

        let num_latitude_vertices = latitude_segments + 1;
        let num_longitude_vertices = longitude_segments + 2;

        // vertices
        for i in 0..num_latitude_vertices {
            let vertical_angle: f32 = i as f32 * PI / latitude_segments as f32;
            for j in 0..num_longitude_vertices {
                let horizontal_angle: f32 = j as f32 * (2.0 * PI) / longitude_segments as f32;

                let position = Vector3::new(
                    radius * vertical_angle.sin() * horizontal_angle.cos(),
                    radius * vertical_angle.sin() * horizontal_angle.sin(),
                    radius * vertical_angle.cos(),
                );

                let normal: Vector3<f32> = position.normalize();
                let uv = Vector2::new(
                    1.0 - j as f32 / (num_longitude_vertices - 1) as f32,
                    i as f32 / (num_latitude_vertices - 1) as f32,
                );
                vertices.push(MeshVertex {
                    position: position.into(),
                    normal: normal.into(),
                    uv: uv.into(),
                });
            }
        }

        let mut indices: Vec<u32> = vec![];

        let calc_index = |i: u32, j: u32| i * num_longitude_vertices + j;

        for i in 0..latitude_segments {
            for j in 0..longitude_segments {
                if i != 0 {
                    indices.push(calc_index(i + 1, j));
                    indices.push(calc_index(i, j + 1));
                    indices.push(calc_index(i, j));
                }

                if i != latitude_segments - 1 {
                    indices.push(calc_index(i, j + 1));
                    indices.push(calc_index(i + 1, j));
                    indices.push(calc_index(i + 1, j + 1));
                }
            }
        }

        let (vertex_buffer, index_buffer) = Mesh::setup_buffer(&vertices, &indices, allocator);

        Arc::new(Mesh {
            vertices,
            indices,

            vertex_buffer,
            index_buffer,
            bounding_box: BoundingBox::new(
                Vector3::new(-radius, -radius, -radius),
                Vector3::new(radius, radius, radius),
            ),
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
