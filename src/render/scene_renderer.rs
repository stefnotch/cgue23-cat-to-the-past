use crate::core::camera::Camera;
use crate::render::context::Context;
use crate::scene::mesh::MeshVertex;
use crate::scene::model::Model;
use crate::scene::texture::Texture;
use crate::scene::transform::Transform;

use crate::scene::light::{Light, PointLight};
use crate::scene::material::Material;
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
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{AttachmentImage, ImageUsage, ImageViewAbstract, SwapchainImage};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::padded::Padded;
use vulkano::pipeline::graphics::depth_stencil::DepthStencilState;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::rasterization::{CullMode, PolygonMode, RasterizationState};
use vulkano::pipeline::graphics::vertex_input::Vertex;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::sampler::{Filter, Sampler, SamplerCreateInfo};
use vulkano::sync::GpuFuture;

pub struct SceneRenderer {
    render_pass: Arc<RenderPass>,
    pipeline: Arc<GraphicsPipeline>,
    framebuffers: Vec<Arc<Framebuffer>>,
    output_images: Vec<Arc<ImageView<AttachmentImage>>>,

    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,

    buffer_allocator: SubbufferAllocator,

    /// The 1x1 white texture used when a model is missing a texture
    missing_texture: Arc<Texture>,
}

impl SceneRenderer {
    pub fn new(
        context: &Context,
        dimensions: [u32; 2],
        swapchain_image_count: u32,
        memory_allocator: Arc<StandardMemoryAllocator>,
        command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    ) -> Self {
        let vs = vs::load(context.device()).unwrap();
        let fs = fs::load(context.device()).unwrap();

        // TODO: consider setting the initial size of the arena
        let buffer_allocator = SubbufferAllocator::new(
            memory_allocator.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
        );

        let render_pass = vulkano::single_pass_renderpass!(
            context.device(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: Format::R16G16B16A16_SFLOAT,
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D32_SFLOAT,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth}
            }
        )
        .unwrap();

        let pipeline = GraphicsPipeline::start()
            .rasterization_state(
                RasterizationState::new()
                    .cull_mode(CullMode::Back)
                    .polygon_mode(PolygonMode::Fill),
            )
            // .rasterization_state(RasterizationState::new().cull_mode(CullMode::Back))
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .depth_stencil_state(DepthStencilState::simple_depth_test())
            .vertex_input_state(MeshVertex::per_vertex())
            .input_assembly_state(InputAssemblyState::new())
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .fragment_shader(fs.entry_point("main").unwrap(), ())
            .build(context.device())
            .expect("could not create pipeline");

        // TODO: let the main_renderer manage those swapchain related framebuffers?

        let images: Vec<Arc<ImageView<AttachmentImage>>> =
            Self::create_images(memory_allocator.clone(), swapchain_image_count, dimensions);

        let framebuffers = Self::create_framebuffers(
            memory_allocator.clone(),
            dimensions,
            images.clone(),
            render_pass.clone(),
        );

        let missing_texture = Texture::new_one_by_one(
            Sampler::new(
                context.device(),
                SamplerCreateInfo {
                    mag_filter: Filter::Nearest,
                    min_filter: Filter::Nearest,
                    ..SamplerCreateInfo::default()
                },
            )
            .unwrap(),
            &context,
        );
        SceneRenderer {
            render_pass,
            pipeline,
            framebuffers,
            output_images: images,
            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,

            buffer_allocator,
            missing_texture,
        }
    }
}

impl SceneRenderer {
    pub fn resize(&mut self, images: &Vec<Arc<ImageView<SwapchainImage>>>) {
        let dimensions = images[0].dimensions().width_height();
        let swapchain_image_count = images.len() as u32;

        self.output_images = Self::create_images(
            self.memory_allocator.clone(),
            swapchain_image_count,
            dimensions,
        );

        self.framebuffers = Self::create_framebuffers(
            self.memory_allocator.clone(),
            dimensions,
            self.output_images.clone(),
            self.render_pass.clone(),
        );
    }

    fn create_framebuffers(
        memory_allocator: Arc<StandardMemoryAllocator>,
        dimensions: [u32; 2],
        images: Vec<Arc<ImageView<AttachmentImage>>>,
        render_pass: Arc<RenderPass>,
    ) -> Vec<Arc<Framebuffer>> {
        let depth_buffer = ImageView::new_default(
            AttachmentImage::transient(&memory_allocator, dimensions, Format::D32_SFLOAT).unwrap(),
        )
        .unwrap();

        images
            .into_iter()
            .map(|image| {
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![image.clone(), depth_buffer.clone()],
                        ..FramebufferCreateInfo::default()
                    },
                )
                .expect("failed to create framebuffer")
            })
            .collect()
    }

    fn create_images(
        memory_allocator: Arc<StandardMemoryAllocator>,
        swapchain_image_count: u32,
        dimensions: [u32; 2],
    ) -> Vec<Arc<ImageView<AttachmentImage>>> {
        (0..swapchain_image_count)
            .map(|_| {
                ImageView::new_default(
                    AttachmentImage::with_usage(
                        &memory_allocator,
                        dimensions,
                        Format::R16G16B16A16_SFLOAT,
                        ImageUsage::SAMPLED
                            .union(ImageUsage::COLOR_ATTACHMENT)
                            .union(ImageUsage::TRANSFER_SRC),
                    )
                    .unwrap(),
                )
                .unwrap()
            })
            .collect()
    }

    pub fn render<F>(
        &self,
        context: &Context,
        camera: &Camera,
        models: Vec<(&Transform, &Model)>,
        lights: Vec<(&Transform, &Light)>,
        future: F,
        swapchain_frame_index: u32,
        viewport: &Viewport,
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

        builder
            // Before we can draw, we have to *enter a render pass*.
            .begin_render_pass(
                RenderPassBeginInfo {
                    // A list of values to clear the attachments with. This list contains
                    // one item for each attachment in the render pass. In this case,
                    // there is only one attachment, and we clear it with a blue color.
                    //
                    // Only attachments that have `LoadOp::Clear` are provided with clear
                    // values, any others should use `ClearValue::None` as the clear value.
                    clear_values: vec![Some([0.2, 0.4, 0.8, 1.0].into()), Some(1f32.into())],
                    ..RenderPassBeginInfo::framebuffer(
                        self.framebuffers[swapchain_frame_index as usize].clone(),
                    )
                },
                SubpassContents::Inline,
            )
            .unwrap()
            // We are now inside the first subpass of the render pass. We add a draw command.
            //
            // The last two parameters contain the list of resources to pass to the shaders.
            // Since we used an `EmptyPipeline` object, the objects have to be `()`.
            .set_viewport(0, [viewport.clone()])
            .bind_pipeline_graphics(self.pipeline.clone());

        // TODO: models with different pipelines
        let scene_set_layout = self.pipeline.layout().set_layouts().get(0).unwrap();
        let camera_set_layout = self.pipeline.layout().set_layouts().get(1).unwrap();
        let material_set_layout = self.pipeline.layout().set_layouts().get(2).unwrap();
        let entity_set_layout = self.pipeline.layout().set_layouts().get(3).unwrap();

        let uniform_subbuffer_scene = {
            let src_point_lights: Vec<Padded<vs::PointLight, 12>> = lights
                .iter()
                .map(|(transform, light)| match light {
                    Light::Point(point_light) => {
                        Padded::from(make_shader_point_light(point_light, transform))
                    }
                })
                .collect();

            let num_lights = src_point_lights.len() as i32;

            let mut point_lights = [Padded::from(default_shader_point_light()); 32];

            point_lights[..src_point_lights.len()].copy_from_slice(src_point_lights.as_slice());

            let uniform_data = vs::Scene {
                pointLights: point_lights,
                numLights: num_lights,
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

        let uniform_subbuffer_camera = {
            let uniform_data = vs::Camera {
                view: camera.view().clone().into(),
                proj: camera.proj().clone().into(),
                position: camera.position.clone().into(),
            };

            let subbuffer = self.buffer_allocator.allocate_sized().unwrap();
            *subbuffer.write().unwrap() = uniform_data;

            subbuffer
        };

        let camera_descriptor_set = PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            camera_set_layout.clone(),
            [WriteDescriptorSet::buffer(0, uniform_subbuffer_camera)],
        )
        .unwrap();

        builder
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                scene_descriptor_set.clone(),
            )
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                1,
                camera_descriptor_set.clone(),
            );

        for (transform, model) in models {
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

            // TODO: Don't create a new descriptor set every frame
            /*
                let e = WriteDescriptorSet::buffer(0, uniform_buffer_subbuffer);
            set.resources().update(&e);
             */
            let entity_descriptor_set = PersistentDescriptorSet::new(
                &self.descriptor_set_allocator,
                entity_set_layout.clone(),
                [WriteDescriptorSet::buffer(0, uniform_subbuffer_entity)],
            )
            .unwrap();

            builder.bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                3,
                entity_descriptor_set.clone(),
            );

            for primitive in &model.primitives {
                // descriptor set
                let uniform_subbuffer_material = {
                    let uniform_data: vs::Material = primitive.material.as_ref().into();

                    let subbuffer = self.buffer_allocator.allocate_sized().unwrap();
                    *subbuffer.write().unwrap() = uniform_data;

                    subbuffer
                };

                let texture = primitive
                    .material
                    .base_color_texture
                    .clone()
                    .unwrap_or(self.missing_texture.clone());

                let material_descriptor_set = PersistentDescriptorSet::new(
                    &self.descriptor_set_allocator,
                    material_set_layout.clone(),
                    [
                        WriteDescriptorSet::buffer(0, uniform_subbuffer_material),
                        WriteDescriptorSet::image_view_sampler(
                            1,
                            texture.image_view.clone(),
                            texture.sampler.clone(),
                        ),
                    ],
                )
                .unwrap();

                builder
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        self.pipeline.layout().clone(),
                        2,
                        material_descriptor_set.clone(),
                    )
                    .bind_index_buffer(primitive.mesh.index_buffer.clone())
                    .bind_vertex_buffers(0, primitive.mesh.vertex_buffer.clone())
                    .draw_indexed(primitive.mesh.index_buffer.len() as u32, 1, 0, 0, 0)
                    .unwrap();
            }
        }

        builder.end_render_pass().unwrap();

        // Finish building the command buffer by calling `build`.
        let command_buffer = builder.build().unwrap();

        future
            .then_execute(context.queue(), command_buffer)
            .unwrap()
    }

    pub fn output_images(&self) -> &Vec<Arc<ImageView<AttachmentImage>>> {
        &self.output_images
    }
}

fn make_shader_point_light(point_light: &PointLight, transform: &Transform) -> vs::PointLight {
    vs::PointLight {
        position: Padded::from(<[f32; 3]>::from(transform.position)), // TODO: fix weird syntax
        color: point_light.color.into(),
        range: point_light.range,
        intensity: point_light.intensity,
    }
}

fn default_shader_point_light() -> vs::PointLight {
    vs::PointLight {
        position: Default::default(),
        color: [0.0, 0.0, 0.0],
        range: 0.0,
        intensity: 0.0,
    }
}

impl From<&Material> for vs::Material {
    fn from(value: &Material) -> Self {
        vs::Material {
            base_color: value.base_color.into(),
            roughness: value.roughness_factor,
            metallic: Padded::from(value.metallic_factor),
            emissivity: value.emissivity.into(),
        }
    }
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "assets/shaders/vert.glsl",
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "assets/shaders/frag.glsl",
    }
}
