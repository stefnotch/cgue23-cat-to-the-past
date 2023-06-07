use crate::context::Context;
use crate::custom_storage_image::CustomStorageImage;
use crate::quad::{create_geometry_buffers, QuadVertex};
use crate::scene::ui_component::UIComponent;
use nalgebra::Matrix4;
use std::sync::Arc;
use vulkano::buffer::Subbuffer;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferInheritanceInfo,
    CommandBufferUsage, RenderPassBeginInfo, SecondaryAutoCommandBuffer, SubpassContents,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::format::Format;
use vulkano::image::view::{ImageView, ImageViewCreateInfo};
use vulkano::image::{AttachmentImage, ImageAccess, ImageUsage, ImageViewAbstract, SwapchainImage};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::graphics::depth_stencil::DepthStencilState;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::Vertex;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode};
use vulkano::sync::GpuFuture;

pub struct OverlayRenderer {
    render_pass: Arc<RenderPass>,

    quad_pipeline: Arc<GraphicsPipeline>,
    ui_pipeline: Arc<GraphicsPipeline>,

    index_buffer: Subbuffer<[u32]>,
    vertex_buffer: Subbuffer<[QuadVertex]>,
    sampler: Arc<Sampler>,

    framebuffers: Vec<Arc<Framebuffer>>,
    quad_command_buffers: Vec<Arc<SecondaryAutoCommandBuffer>>,
    input_descriptor_sets: Vec<Arc<PersistentDescriptorSet>>,

    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl OverlayRenderer {
    pub fn new(
        context: &Context,
        output_images: &[Arc<ImageView<SwapchainImage>>],
        final_output_format: Format,
        memory_allocator: Arc<StandardMemoryAllocator>,
        command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
        brightness: f32,
    ) -> OverlayRenderer {
        let (vertex_buffer, index_buffer) = create_geometry_buffers(memory_allocator.clone());

        let render_pass = vulkano::ordered_passes_renderpass!(
            context.device(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: final_output_format,
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D16_UNORM,
                    samples: 1,
                },
            },
            passes: [
                {
                    color: [color],
                    depth_stencil: {depth},
                    input: []
                },
                {
                    color: [color],
                    depth_stencil: {depth},
                    input: [color]
                }
            ]
        )
        .unwrap();

        let quad_pipeline = {
            let vertex_shader = quad::vs::load(context.device()).unwrap();
            let fragment_shader = quad::fs::load(context.device()).unwrap();

            let spec_consts = quad::fs::SpecializationConstants { brightness };

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

        let ui_pipeline = {
            let vs = ui::vs::load(context.device()).unwrap();
            let fs = ui::fs::load(context.device()).unwrap();

            GraphicsPipeline::start()
                .vertex_input_state(QuadVertex::per_vertex())
                .vertex_shader(vs.entry_point("main").unwrap(), ())
                .input_assembly_state(InputAssemblyState::new())
                .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
                .fragment_shader(fs.entry_point("main").unwrap(), ())
                .depth_stencil_state(DepthStencilState::simple_depth_test())
                .render_pass(Subpass::from(render_pass.clone(), 1).unwrap())
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

        let framebuffers =
            Self::create_framebuffers(memory_allocator.clone(), output_images, render_pass.clone());

        let input_descriptor_sets = Self::create_input_descriptor_sets(
            ui_pipeline.clone(),
            sampler.clone(),
            output_images,
            descriptor_set_allocator.clone(),
        );

        OverlayRenderer {
            render_pass,
            quad_pipeline,
            ui_pipeline,

            index_buffer,
            vertex_buffer,
            sampler,

            quad_command_buffers: vec![],
            framebuffers,
            input_descriptor_sets,

            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,
        }
    }

    pub fn resize(&mut self, output_images: &[Arc<ImageView<SwapchainImage>>]) {
        self.framebuffers = Self::create_framebuffers(
            self.memory_allocator.clone(),
            output_images,
            self.render_pass.clone(),
        );

        // remember to call pre_record_command_buffer_quad again
        self.quad_command_buffers.clear();
    }

    pub fn render<F>(
        &self,
        context: &Context,
        ui_components: Vec<&UIComponent>,
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
            .set_viewport(0, [viewport.clone()])
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into()), Some(1.0f32.into())],
                    ..RenderPassBeginInfo::framebuffer(
                        self.framebuffers[swapchain_frame_index as usize].clone(),
                    )
                },
                SubpassContents::SecondaryCommandBuffers,
            )
            .unwrap();

        let quad_command_buffer = self.draw_quad(swapchain_frame_index);

        builder
            .execute_commands(quad_command_buffer)
            .unwrap()
            .next_subpass(SubpassContents::SecondaryCommandBuffers)
            .unwrap();

        let ui_command_buffer =
            self.draw_ui(context, ui_components, swapchain_frame_index, viewport);

        builder
            .execute_commands(ui_command_buffer)
            .unwrap()
            .end_render_pass()
            .unwrap();

        let command_buffer = builder.build().unwrap();

        future
            .then_execute(context.queue(), command_buffer)
            .unwrap()
    }

    pub fn pre_record_command_buffer_quad(
        &mut self,
        context: &Context,
        input_images: &[Arc<ImageView<CustomStorageImage>>],
        viewport: &Viewport,
    ) {
        self.quad_command_buffers =
            self.record_quad_command_buffers(context, input_images, viewport);
    }

    fn draw_quad(&self, swapchain_frame_index: u32) -> Arc<SecondaryAutoCommandBuffer> {
        assert_ne!(self.quad_command_buffers.len(), 0);

        self.quad_command_buffers[swapchain_frame_index as usize].clone()
    }

    fn draw_ui(
        &self,
        context: &Context,
        ui_components: Vec<&UIComponent>,
        swapchain_frame_index: u32,
        viewport: &Viewport,
    ) -> SecondaryAutoCommandBuffer {
        let mut builder = AutoCommandBufferBuilder::secondary(
            &self.command_buffer_allocator,
            context.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
            CommandBufferInheritanceInfo {
                render_pass: Some(Subpass::from(self.render_pass.clone(), 1).unwrap().into()),
                ..CommandBufferInheritanceInfo::default()
            },
        )
        .unwrap();

        builder
            .set_viewport(0, [viewport.clone()])
            .bind_pipeline_graphics(self.ui_pipeline.clone());

        let set_layout = self.ui_pipeline.layout().set_layouts().get(0).unwrap();

        for component in ui_components {
            let component_push_constant = ui::vs::UIComponent {
                MVP: Matrix4::identity().into(),
            };

            let descriptor_set = PersistentDescriptorSet::new(
                &self.descriptor_set_allocator,
                set_layout.clone(),
                [WriteDescriptorSet::image_view_sampler(
                    1,
                    component.texture.image_view.clone(),
                    component.texture.sampler.clone(),
                )],
            )
            .unwrap();

            builder
                .push_constants(
                    self.ui_pipeline.layout().clone(),
                    0,
                    component_push_constant,
                )
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    self.ui_pipeline.layout().clone(),
                    0,
                    self.input_descriptor_sets[swapchain_frame_index as usize].clone(),
                )
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    self.ui_pipeline.layout().clone(),
                    1,
                    descriptor_set.clone(),
                )
                .bind_index_buffer(self.index_buffer.clone())
                .bind_vertex_buffers(0, self.vertex_buffer.clone())
                .draw_indexed(6, 1, 0, 0, 0)
                .unwrap(); // TODO: remove magic number 6
        }

        builder.build().unwrap()
    }

    fn record_quad_command_buffers(
        &self,
        context: &Context,
        images: &[Arc<ImageView<CustomStorageImage>>],
        viewport: &Viewport,
    ) -> Vec<Arc<SecondaryAutoCommandBuffer>> {
        let layout = self.quad_pipeline.layout().set_layouts().get(0).unwrap();

        images
            .iter()
            .map(|image| {
                let mut builder = AutoCommandBufferBuilder::secondary(
                    &self.command_buffer_allocator,
                    context.queue_family_index(),
                    CommandBufferUsage::MultipleSubmit,
                    CommandBufferInheritanceInfo {
                        render_pass: Some(
                            Subpass::from(self.render_pass.clone(), 0).unwrap().into(),
                        ),
                        ..CommandBufferInheritanceInfo::default()
                    },
                )
                .unwrap();

                let descriptor_set = PersistentDescriptorSet::new(
                    &self.descriptor_set_allocator,
                    layout.clone(),
                    [WriteDescriptorSet::image_view_sampler(
                        0,
                        image.clone(),
                        self.sampler.clone(),
                    )],
                )
                .unwrap();

                builder
                    .set_viewport(0, [viewport.clone()])
                    .bind_pipeline_graphics(self.quad_pipeline.clone())
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        self.quad_pipeline.layout().clone(),
                        0,
                        descriptor_set.clone(),
                    )
                    .bind_index_buffer(self.index_buffer.clone())
                    .bind_vertex_buffers(0, self.vertex_buffer.clone())
                    .draw_indexed(6, 1, 0, 0, 0)
                    .unwrap(); // TODO: remove magic number 6

                Arc::new(builder.build().unwrap())
            })
            .collect()
    }

    fn create_framebuffers(
        memory_allocator: Arc<StandardMemoryAllocator>,
        output_images: &[Arc<ImageView<SwapchainImage>>],
        render_pass: Arc<RenderPass>,
    ) -> Vec<Arc<Framebuffer>> {
        let dimensions = output_images[0].dimensions().width_height();

        output_images
            .iter()
            .map(|output_image| {
                let depth_buffer = ImageView::new_default(
                    AttachmentImage::transient(&memory_allocator, dimensions, Format::D16_UNORM)
                        .unwrap(),
                )
                .unwrap();

                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![output_image.clone(), depth_buffer.clone()],
                        ..FramebufferCreateInfo::default()
                    },
                )
                .expect("failed to create framebuffer")
            })
            .collect()
    }

    fn create_input_descriptor_sets(
        pipeline: Arc<GraphicsPipeline>,
        sampler: Arc<Sampler>,
        input_images: &[Arc<ImageView<SwapchainImage>>],
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    ) -> Vec<Arc<PersistentDescriptorSet>> {
        let layout = pipeline.layout().set_layouts().get(0).unwrap();

        input_images
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

    fn create_images(
        memory_allocator: Arc<StandardMemoryAllocator>,
        final_output_format: Format,
        image_count: u32,
        dimensions: [u32; 2],
    ) -> Vec<Arc<ImageView<AttachmentImage>>> {
        (0..image_count)
            .map(|_| {
                ImageView::new_default(
                    AttachmentImage::with_usage(
                        &memory_allocator,
                        dimensions,
                        final_output_format,
                        ImageUsage::TRANSIENT_ATTACHMENT | ImageUsage::INPUT_ATTACHMENT,
                    )
                    .unwrap(),
                )
                .unwrap()
            })
            .collect()
    }
}

mod quad {
    pub(crate) mod vs {
        vulkano_shaders::shader! {
            ty: "vertex",
            path: "../assets/shaders/quad/quad.vert",
        }
    }

    pub(crate) mod fs {
        vulkano_shaders::shader! {
            ty: "fragment",
            path: "../assets/shaders/quad/quad.frag",
        }
    }
}

mod ui {
    pub(crate) mod vs {
        vulkano_shaders::shader! {
            ty: "vertex",
            path: "../assets/shaders/ui/ui.vert",
        }
    }

    pub(crate) mod fs {
        vulkano_shaders::shader! {
            ty: "fragment",
            path: "../assets/shaders/ui/ui.frag",
        }
    }
}
