use crate::context::Context;
use crate::quad::{create_geometry_buffers, QuadVertex};
use crate::scene::ui_component::GpuUIComponent;
use nalgebra::{Matrix4, Vector2};
use scene::ui_component::UIComponent;
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
use vulkano::image::{AttachmentImage, ImageUsage, ImageViewAbstract, SwapchainImage};
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

    index_buffer: Subbuffer<[u32]>,
    vertex_buffer: Subbuffer<[QuadVertex]>,

    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl UIRenderer {
    pub fn new(
        context: &Context,
        images: &[Arc<ImageView<SwapchainImage>>],
        final_output_format: Format,
        memory_allocator: Arc<StandardMemoryAllocator>,
        command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    ) -> Self {
        let (vertex_buffer, index_buffer) = create_geometry_buffers(memory_allocator.clone());

        let render_pass = vulkano::single_pass_renderpass!(
            context.device(),
            attachments: {
                color: {
                    load: Load,
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

        let framebuffers =
            Self::create_framebuffers(memory_allocator.clone(), images, render_pass.clone());

        UIRenderer {
            render_pass,
            pipeline,

            framebuffers,

            index_buffer,
            vertex_buffer,

            command_buffer_allocator,
            memory_allocator,
            descriptor_set_allocator,
        }
    }

    pub fn resize(&mut self, images: &[Arc<ImageView<SwapchainImage>>]) {
        self.framebuffers = Self::create_framebuffers(
            self.memory_allocator.clone(),
            images,
            self.render_pass.clone(),
        );
    }

    pub fn render<F>(
        &self,
        context: &Context,
        ui_components: Vec<(&GpuUIComponent, &UIComponent)>,
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
                    clear_values: vec![None, Some(1.0f32.into())],
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

        for (gpu_component, cpu_component) in ui_components {
            let screen_size = Vector2::from(viewport.dimensions);
            let pixel_center = cpu_component
                .position
                .xy()
                .coords
                .component_mul(&screen_size);
            let texture_size = cpu_component.texture.data.dimensions();
            let texture_size = Vector2::new(texture_size[0] as f32, texture_size[1] as f32);
            let pixel_size = texture_size.component_mul(&screen_size);
            let pixel_top_left = pixel_center - pixel_size / 2.0;
            /*
            let mvp = calculate_orthographic(screen size)
            * Matrix4::new_translation(pixel_xy, depth)
            * Matrix4::new_rotation(Vector3::unit_z, angle)
            * Matrix4::new_scale(pixel_size, 1.0) */
            let component_push_constant = vs::UIComponent {
                MVP: Matrix4::identity().into(),
            };

            let descriptor_set = PersistentDescriptorSet::new(
                &self.descriptor_set_allocator,
                set_layout.clone(),
                [WriteDescriptorSet::image_view_sampler(
                    0,
                    gpu_component.texture.image_view.clone(),
                    gpu_component.texture.sampler.clone(),
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

    fn create_framebuffers(
        memory_allocator: Arc<StandardMemoryAllocator>,
        images: &[Arc<ImageView<SwapchainImage>>],
        render_pass: Arc<RenderPass>,
    ) -> Vec<Arc<Framebuffer>> {
        images
            .into_iter()
            .map(|image| {
                let dimensions = image.dimensions().width_height();

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
