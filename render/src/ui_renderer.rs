use crate::context::Context;
use crate::quad::{create_geometry_buffers, QuadVertex};
use crate::scene::ui_component::GpuUIComponent;
use nalgebra::Matrix4;
use std::sync::Arc;
use vulkano::buffer::allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo};
use vulkano::buffer::{BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferUsage, RenderPassBeginInfo,
    SubpassContents,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{AttachmentImage, ImageUsage};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::graphics::depth_stencil::DepthStencilState;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::Vertex;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::sync::GpuFuture;

pub struct UIRenderer {
    render_pass: Arc<RenderPass>,
    pipeline: Arc<GraphicsPipeline>,
    framebuffers: Vec<Arc<Framebuffer>>,
    output_images: Vec<Arc<ImageView<AttachmentImage>>>,

    index_buffer: Subbuffer<[u32]>,
    vertex_buffer: Subbuffer<[QuadVertex]>,

    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl UIRenderer {
    pub fn new(
        context: &Context,
        image_count: u32,
        dimensions: [u32; 2],
        memory_allocator: Arc<StandardMemoryAllocator>,
        command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    ) -> Self {
        let (vertex_buffer, index_buffer) = create_geometry_buffers(memory_allocator.clone());

        let render_pass = vulkano::single_pass_renderpass!(
            context.device(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: Format::B8G8R8A8_SRGB,
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D16_UNORM,
                    samples: 1,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {depth}
            }
        )
        .unwrap();

        let pipeline = {
            let vs = vs::load(context.device()).unwrap();
            let fs = fs::load(context.device()).unwrap();

            GraphicsPipeline::start()
                .vertex_input_state(QuadVertex::per_vertex())
                .vertex_shader(vs.entry_point("main").unwrap(), ())
                .input_assembly_state(InputAssemblyState::new())
                .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
                .fragment_shader(fs.entry_point("main").unwrap(), ())
                .depth_stencil_state(DepthStencilState::simple_depth_test())
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(context.device())
                .unwrap()
        };

        let images = Self::create_images(memory_allocator.clone(), image_count, dimensions);
        let framebuffers = Self::create_framebuffers(
            memory_allocator.clone(),
            dimensions,
            images.clone(),
            render_pass.clone(),
        );

        UIRenderer {
            render_pass,
            pipeline,

            framebuffers,
            output_images: images,

            index_buffer,
            vertex_buffer,

            command_buffer_allocator,
            memory_allocator,
            descriptor_set_allocator,
        }
    }

    pub fn resize(&mut self, dimensions: [u32; 2], image_count: u32) {
        self.output_images =
            Self::create_images(self.memory_allocator.clone(), image_count, dimensions);

        self.framebuffers = Self::create_framebuffers(
            self.memory_allocator.clone(),
            dimensions,
            self.output_images.clone(),
            self.render_pass.clone(),
        );
    }

    pub fn render<F>(
        &self,
        context: &Context,
        ui_components: Vec<&GpuUIComponent>,
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
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0; 4].into()), Some(1.0f32.into())],
                    ..RenderPassBeginInfo::framebuffer(
                        self.framebuffers[swapchain_frame_index as usize].clone(),
                    )
                },
                SubpassContents::Inline,
            )
            .unwrap()
            .set_viewport(0, [viewport.clone()])
            .bind_pipeline_graphics(self.pipeline.clone());

        let set_layout = self.pipeline.layout().set_layouts().get(0).unwrap();

        for component in ui_components {
            let component_push_constant = vs::UIComponent {
                MVP: Matrix4::identity().into(),
            };

            let descriptor_set = PersistentDescriptorSet::new(
                &self.descriptor_set_allocator,
                set_layout.clone(),
                [WriteDescriptorSet::image_view_sampler(
                    0,
                    component.texture.image_view.clone(),
                    component.texture.sampler.clone(),
                )],
            )
            .unwrap();

            builder
                .push_constants(self.pipeline.layout().clone(), 0, component_push_constant)
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    self.pipeline.layout().clone(),
                    0,
                    descriptor_set.clone(),
                )
                .bind_index_buffer(self.index_buffer.clone())
                .bind_vertex_buffers(0, self.vertex_buffer.clone())
                .draw_indexed(6, 1, 0, 0, 0)
                .unwrap(); // TODO: remove magic number 6
        }

        builder.end_render_pass().unwrap();

        let command_buffer = builder.build().unwrap();

        future
            .then_execute(context.queue(), command_buffer)
            .unwrap()
    }

    fn create_images(
        memory_allocator: Arc<StandardMemoryAllocator>,
        image_count: u32,
        dimensions: [u32; 2],
    ) -> Vec<Arc<ImageView<AttachmentImage>>> {
        (0..image_count)
            .map(|_| {
                ImageView::new_default(
                    AttachmentImage::with_usage(
                        &memory_allocator,
                        dimensions,
                        Format::B8G8R8A8_SRGB,
                        ImageUsage::SAMPLED | ImageUsage::COLOR_ATTACHMENT,
                    )
                    .unwrap(),
                )
                .unwrap()
            })
            .collect()
    }

    fn create_framebuffers(
        memory_allocator: Arc<StandardMemoryAllocator>,
        dimensions: [u32; 2],
        images: Vec<Arc<ImageView<AttachmentImage>>>,
        render_pass: Arc<RenderPass>,
    ) -> Vec<Arc<Framebuffer>> {
        images
            .into_iter()
            .map(|image| {
                let depth_buffer = ImageView::new_default(
                    AttachmentImage::transient(&memory_allocator, dimensions, Format::D16_UNORM)
                        .unwrap(),
                )
                .unwrap();

                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![image.clone(), depth_buffer.clone()],
                        ..FramebufferCreateInfo::default()
                    },
                )
                .unwrap()
            })
            .collect()
    }

    pub fn get_ui_images(&self) -> &Vec<Arc<ImageView<AttachmentImage>>> {
        &self.output_images
    }
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "../assets/shaders/ui/ui.vert",
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "../assets/shaders/ui/ui.frag",
    }
}
