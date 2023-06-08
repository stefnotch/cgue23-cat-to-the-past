use crate::context::Context;
use crate::custom_storage_image::CustomStorageImage;
use crate::quad::{self, quad_mesh, QuadVertex};
use std::sync::Arc;
use vulkano::buffer::Subbuffer;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferUsage, RenderPassBeginInfo,
    SubpassContents,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::layout::DescriptorSetLayout;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::SwapchainImage;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::Vertex;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode};
use vulkano::sync::GpuFuture;

pub struct QuadRenderer {
    pipeline: Arc<GraphicsPipeline>,
    framebuffers: Vec<Arc<Framebuffer>>,
    render_pass: Arc<RenderPass>,

    sampler: Arc<Sampler>,
    descriptor_sets: Vec<Arc<PersistentDescriptorSet>>,
    index_buffer: Subbuffer<[u32]>,
    vertex_buffer: Subbuffer<[QuadVertex]>,

    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl QuadRenderer {
    pub fn new(
        context: &Context,
        input_images: &[Arc<ImageView<CustomStorageImage>>],
        output_images: &[Arc<ImageView<SwapchainImage>>],
        final_output_format: Format,
        memory_allocator: Arc<StandardMemoryAllocator>,
        command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
        brightness: f32,
    ) -> Self {
        let (vertex_buffer, index_buffer) =
            quad::create_geometry_buffers(quad_mesh(), memory_allocator.clone());

        let render_pass = vulkano::single_pass_renderpass!(context.device(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: final_output_format,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )
        .unwrap();

        let pipeline = {
            let vertex_shader = vs::load(context.device()).unwrap();
            let fragment_shader = fs::load(context.device()).unwrap();

            let spec_consts = fs::SpecializationConstants { brightness };

            GraphicsPipeline::start()
                .vertex_input_state(QuadVertex::per_vertex())
                .vertex_shader(vertex_shader.entry_point("main").unwrap(), ())
                .input_assembly_state(InputAssemblyState::new())
                .fragment_shader(fragment_shader.entry_point("main").unwrap(), spec_consts)
                .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(context.device())
                .unwrap()
        };

        let sampler = Sampler::new(
            context.device(),
            SamplerCreateInfo {
                mag_filter: Filter::Nearest,
                min_filter: Filter::Nearest,
                address_mode: [SamplerAddressMode::Repeat; 3],
                mipmap_mode: SamplerMipmapMode::Nearest,
                ..SamplerCreateInfo::default()
            },
        )
        .unwrap();

        let layout = pipeline.layout().set_layouts().get(0).unwrap();

        let descriptor_sets = Self::create_descriptor_sets(
            layout,
            input_images,
            sampler.clone(),
            descriptor_set_allocator.clone(),
        );

        let framebuffers = Self::create_framebuffers(render_pass.clone(), output_images);

        Self {
            pipeline,
            framebuffers,
            render_pass,

            sampler,
            descriptor_sets,
            index_buffer,
            vertex_buffer,

            command_buffer_allocator,
            descriptor_set_allocator,
        }
    }

    pub fn resize(
        &mut self,
        output_images: &[Arc<ImageView<SwapchainImage>>],
        input_images: &[Arc<ImageView<CustomStorageImage>>],
    ) {
        self.framebuffers = Self::create_framebuffers(self.render_pass.clone(), output_images);

        let layout = self.pipeline.layout().set_layouts().get(0).unwrap();
        self.descriptor_sets = Self::create_descriptor_sets(
            layout,
            input_images,
            self.sampler.clone(),
            self.descriptor_set_allocator.clone(),
        );
    }

    fn create_framebuffers(
        render_pass: Arc<RenderPass>,
        images: &[Arc<ImageView<SwapchainImage>>],
    ) -> Vec<Arc<Framebuffer>> {
        images
            .iter()
            .map(|image| {
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![image.clone()],
                        ..FramebufferCreateInfo::default()
                    },
                )
                .expect("failed to create framebuffer")
            })
            .collect()
    }

    fn create_descriptor_sets(
        layout: &Arc<DescriptorSetLayout>,
        images: &[Arc<ImageView<CustomStorageImage>>],
        sampler: Arc<Sampler>,
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    ) -> Vec<Arc<PersistentDescriptorSet>> {
        images
            .iter()
            .map(|image| {
                PersistentDescriptorSet::new(
                    &descriptor_set_allocator,
                    layout.clone(),
                    [WriteDescriptorSet::image_view_sampler(
                        0,
                        image.clone(),
                        sampler.clone(),
                    )],
                )
                .unwrap()
            })
            .collect()
    }

    pub fn render<F>(
        &self,
        context: &Context,
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
            // is it possible to record once and recycle the command buffer?
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .set_viewport(0, [viewport.clone()])
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(
                        self.framebuffers[swapchain_frame_index as usize].clone(),
                    )
                },
                SubpassContents::Inline,
            )
            .unwrap()
            .bind_pipeline_graphics(self.pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                self.descriptor_sets[swapchain_frame_index as usize].clone(),
            )
            .bind_index_buffer(self.index_buffer.clone())
            .bind_vertex_buffers(0, self.vertex_buffer.clone())
            .draw_indexed(6, 1, 0, 0, 0)
            .unwrap() // TODO: remove magic number 6
            .end_render_pass()
            .unwrap();

        let command_buffer = builder.build().unwrap();

        future
            .then_execute(context.queue(), command_buffer)
            .unwrap()
    }
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "../assets/shaders/quad/quad.vert",
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "../assets/shaders/quad/quad.frag",
    }
}
