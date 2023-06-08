use crate::context::Context;
use crate::custom_storage_image::CustomStorageImage;
use crate::scene::mesh::MeshVertex;
use crate::scene::model::GpuModel;
use angle::Deg;
use nalgebra::{Matrix4, Translation3, Vector3};
use scene::camera::{calculate_projection, Camera};
use scene::transform::Transform;
use std::sync::Arc;
use vulkano::buffer::allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo};
use vulkano::buffer::BufferUsage;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferUsage, RenderPassBeginInfo,
    SubpassContents,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};

use vulkano::format::{ClearValue, Format};
use vulkano::image::view::{ImageView, ImageViewCreateInfo};
use vulkano::image::ImageDimensions::Dim2d;
use vulkano::image::{
    ImageAccess, ImageCreateFlags, ImageLayout, ImageSubresourceRange, ImageUsage, ImageViewType,
};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::padded::Padded;
use vulkano::pipeline::graphics::depth_stencil::DepthStencilState;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::rasterization::{CullMode, PolygonMode, RasterizationState};
use vulkano::pipeline::graphics::vertex_input::Vertex;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::sync::GpuFuture;

const CUBE_SIZE: u32 = 2048;

pub struct ShadowRenderer {
    render_pass: Arc<RenderPass>,
    pipeline: Arc<GraphicsPipeline>,
    framebuffers: Vec<[Arc<Framebuffer>; 6]>,
    shadow_maps: Vec<Arc<CustomStorageImage>>,
    shadow_maps_views: Vec<Arc<ImageView<CustomStorageImage>>>,

    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,

    buffer_allocator: SubbufferAllocator,

    face_view_matrices: [Matrix4<f32>; 6],
    perspective_matrix: Matrix4<f32>,
}

impl ShadowRenderer {
    pub fn new(
        context: &Context,
        image_count: u32,
        memory_allocator: Arc<StandardMemoryAllocator>,
        command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    ) -> Self {
        let render_pass = vulkano::single_pass_renderpass!(
            context.device(),
            attachments: {
                depth: {
                    load: Clear,
                    store: Store,
                    format: Format::D32_SFLOAT,
                    samples: 1
                },
            },
            pass: {
                color: [],
                depth_stencil: {depth}
            }
        )
        .unwrap();

        // TODO: consider setting the initial size of the arena
        let buffer_allocator = SubbufferAllocator::new(
            memory_allocator.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
        );

        let pipeline = {
            let vs = vs::load(context.device()).unwrap();
            let fs = fs::load(context.device()).unwrap();

            GraphicsPipeline::start()
                .vertex_input_state(MeshVertex::per_vertex())
                .rasterization_state(
                    RasterizationState::new()
                        .cull_mode(CullMode::Back)
                        .polygon_mode(PolygonMode::Fill),
                )
                .vertex_shader(vs.entry_point("main").unwrap(), ())
                .input_assembly_state(InputAssemblyState::new())
                .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
                .fragment_shader(fs.entry_point("main").unwrap(), ())
                .depth_stencil_state(DepthStencilState::simple_depth_test())
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(context.device())
                .unwrap()
        };

        let (shadow_maps, shadow_maps_views): (
            Vec<Arc<CustomStorageImage>>,
            Vec<Arc<ImageView<CustomStorageImage>>>,
        ) = Self::create_images(memory_allocator.clone(), image_count);

        let framebuffers: Vec<[Arc<Framebuffer>; 6]> =
            Self::create_framebuffers(shadow_maps.clone(), render_pass.clone());

        let face_view_matrices = [
            // POSITIVE_X
            Matrix4::from_row_slice(&[
                0.0, 0.0, 1.0, 0.0, //
                0.0, -1.0, 0.0, 0.0, //
                1.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 1.0, //
            ]),
            // NEGATIVE_X
            Matrix4::from_row_slice(&[
                0.0, 0.0, -1.0, 0.0, //
                0.0, -1.0, 0.0, 0.0, //
                -1.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 1.0, //
            ]),
            // POSITIVE_Y
            Matrix4::from_row_slice(&[
                -1.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 1.0, 0.0, //
                0.0, 1.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 1.0, //
            ]),
            // NEGATIVE_Y
            Matrix4::from_row_slice(&[
                -1.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, -1.0, 0.0, //
                0.0, -1.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 1.0, //
            ]),
            // POSITIVE_Z
            Matrix4::from_row_slice(&[
                -1.0, 0.0, 0.0, 0.0, //
                0.0, -1.0, 0.0, 0.0, //
                0.0, 0.0, 1.0, 0.0, //
                0.0, 0.0, 0.0, 1.0, //
            ]),
            // NEGATIVE_Z
            Matrix4::from_row_slice(&[
                1.0, 0.0, 0.0, 0.0, //
                0.0, -1.0, 0.0, 0.0, //
                0.0, 0.0, -1.0, 0.0, //
                0.0, 0.0, 0.0, 1.0, //
            ]),
        ];

        let far: f32 = 50.0;
        let near: f32 = 0.5;

        let perspective_matrix = calculate_projection(1.0, Deg(90.0).into(), near, far);

        ShadowRenderer {
            render_pass,
            pipeline,
            framebuffers,
            shadow_maps,
            shadow_maps_views,
            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,
            buffer_allocator,
            face_view_matrices,
            perspective_matrix,
        }
    }

    pub fn resize(&mut self, image_count: u32) {
        // resize is not necessary since the cubemap is always the same size
        // let (images, views) = Self::create_images(self.memory_allocator.clone(), image_count);
        //
        // self.shadow_maps = images;
        // self.shadow_maps_views = views;
        //
        // self.framebuffers =
        //     Self::create_framebuffers(self.shadow_maps.clone(), self.render_pass.clone());
        //
        // let aspect_ratio = 1.0;
        //
        // self.perspective_matrix = calculate_projection(aspect_ratio, Deg(45.0).into(), 0.01, 100.0);
    }

    pub fn render<F>(
        &self,
        context: &Context,
        rewind_time: f32,
        models: &Vec<(&Transform, &GpuModel)>,
        nearest_shadow_light: &Transform,
        camera: &Camera,
        future: F,
        swapchain_frame_index: u32,
    ) -> CommandBufferExecFuture<F>
    where
        F: GpuFuture + 'static,
    {
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            context.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [CUBE_SIZE as f32; 2],
            depth_range: 0.0..1.0,
        };

        let scene_set_layout = self.pipeline.layout().set_layouts().get(0).unwrap();
        let entity_set_layout = self.pipeline.layout().set_layouts().get(1).unwrap();

        for face_index in 0..6 {
            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![Some(ClearValue::Depth(1f32))],
                        ..RenderPassBeginInfo::framebuffer(
                            self.framebuffers[swapchain_frame_index as usize][face_index].clone(),
                        )
                    },
                    SubpassContents::Inline,
                )
                .unwrap()
                .set_viewport(0, [viewport.clone()])
                .bind_pipeline_graphics(self.pipeline.clone());

            let view_matrix = self.face_view_matrices[face_index];

            let light_position: Matrix4<f32> =
                Translation3::from(-nearest_shadow_light.position).to_homogeneous();
            let proj_view_matrix = self.perspective_matrix * view_matrix * light_position;

            let uniform_subbuffer_scene = {
                let uniform_data = vs::Scene {
                    projView: proj_view_matrix.into(),
                    lightPos: Padded::from(<[f32; 3]>::from(nearest_shadow_light.position)),
                    cameraPosition: camera.position.into(),
                    rewindTime: rewind_time.into(),
                };

                let subbuffer = self.buffer_allocator.allocate_sized().unwrap();
                *subbuffer.write().unwrap() = uniform_data;

                subbuffer
            };
            let scene_descriptor_set = PersistentDescriptorSet::new(
                &self.descriptor_set_allocator,
                scene_set_layout.clone(),
                [WriteDescriptorSet::buffer(0, uniform_subbuffer_scene)],
            )
            .unwrap();
            builder.bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                scene_descriptor_set.clone(),
            );

            for (transform, model) in models.iter() {
                for primitive in &model.primitives {
                    // descriptor set
                    let uniform_subbuffer_entity = {
                        let model_matrix = transform.to_matrix();
                        let normal_model_matrix = model_matrix.try_inverse().unwrap().transpose();

                        let uniform_data = vs::Entity {
                            model: model_matrix.into(),
                            normalMatrix: normal_model_matrix.into(),
                        };

                        let subbuffer = self.buffer_allocator.allocate_sized().unwrap();
                        *subbuffer.write().unwrap() = uniform_data;

                        subbuffer
                    };

                    let entity_descriptor_set = PersistentDescriptorSet::new(
                        &self.descriptor_set_allocator,
                        entity_set_layout.clone(),
                        [WriteDescriptorSet::buffer(0, uniform_subbuffer_entity)],
                    )
                    .unwrap();

                    builder
                        .bind_descriptor_sets(
                            PipelineBindPoint::Graphics,
                            self.pipeline.layout().clone(),
                            1,
                            entity_descriptor_set.clone(),
                        )
                        .bind_index_buffer(primitive.mesh.index_buffer.clone())
                        .bind_vertex_buffers(0, primitive.mesh.vertex_buffer.clone())
                        .draw_indexed(primitive.mesh.index_buffer.len() as u32, 1, 0, 0, 0)
                        .unwrap();
                }
            }

            builder.end_render_pass().unwrap();
        }

        let command_buffer = builder.build().unwrap();

        future
            .then_execute(context.queue(), command_buffer)
            .unwrap()
    }

    fn create_images(
        memory_allocator: Arc<StandardMemoryAllocator>,
        num_images: u32,
    ) -> (
        Vec<Arc<CustomStorageImage>>,
        Vec<Arc<ImageView<CustomStorageImage>>>,
    ) {
        let images: Vec<Arc<CustomStorageImage>> = (0..num_images)
            .map(|_| {
                CustomStorageImage::uninitialized(
                    &memory_allocator,
                    Dim2d {
                        width: CUBE_SIZE,
                        height: CUBE_SIZE,
                        array_layers: 6,
                    },
                    Format::D32_SFLOAT,
                    1,
                    ImageUsage::SAMPLED | ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                    ImageCreateFlags::CUBE_COMPATIBLE,
                    ImageLayout::DepthStencilAttachmentOptimal,
                )
                .unwrap()
            })
            .collect();

        let views: Vec<Arc<ImageView<CustomStorageImage>>> = images
            .iter()
            .map(|image| {
                ImageView::new(
                    image.clone(),
                    ImageViewCreateInfo {
                        view_type: ImageViewType::Cube,
                        format: Some(image.format()),
                        subresource_range: ImageSubresourceRange {
                            array_layers: 0..6,
                            ..image.subresource_range()
                        },
                        ..ImageViewCreateInfo::default()
                    },
                )
                .unwrap()
            })
            .collect();

        (images, views)
    }

    fn create_framebuffers(
        images: Vec<Arc<CustomStorageImage>>,
        renderpass: Arc<RenderPass>,
    ) -> Vec<[Arc<Framebuffer>; 6]> {
        images
            .into_iter()
            .map(|image| {
                (0..6)
                    .map(|face_index| {
                        let image_view = ImageView::new(
                            image.clone(),
                            ImageViewCreateInfo {
                                format: Some(image.format()),
                                subresource_range: ImageSubresourceRange {
                                    array_layers: face_index..(face_index + 1),
                                    ..image.subresource_range()
                                },
                                ..ImageViewCreateInfo::default()
                            },
                        )
                        .unwrap();

                        Framebuffer::new(
                            renderpass.clone(),
                            FramebufferCreateInfo {
                                attachments: vec![image_view.clone()],
                                ..FramebufferCreateInfo::default()
                            },
                        )
                        .unwrap()
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap()
            })
            .collect()
    }

    pub fn get_shadow_cube_maps(&self) -> Vec<Arc<ImageView<CustomStorageImage>>> {
        self.shadow_maps_views.clone()
    }
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "../assets/shaders/shadow/shadow.vert",
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "../assets/shaders/shadow/shadow.frag",
    }
}
