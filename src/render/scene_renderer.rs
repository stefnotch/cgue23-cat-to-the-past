use crate::context::Context;
use crate::mesh::{Mesh, MeshVertex};
use cgmath::{Matrix4, Point3};
use std::default::Default;
use std::sync::Arc;
use vulkano::buffer::{BufferUsage, CpuBufferPool, TypedBufferAccess};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferUsage, RenderPassBeginInfo,
    SubpassContents,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::SwapchainImage;
use vulkano::memory::allocator::{MemoryUsage, StandardMemoryAllocator};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::sync::GpuFuture;

pub struct SceneRenderer {
    pub render_pass: Arc<RenderPass>,
    pub pipeline: Arc<GraphicsPipeline>,
    pub mesh: Arc<Mesh>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    pub uniform_buffer: CpuBufferPool<vs::ty::Data>,
    // maybe move that to the main renderer?
    pub descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl SceneRenderer {
    pub fn new(
        context: &Context,
        images: &[Arc<ImageView<SwapchainImage>>],
        final_output_format: Format,
        command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    ) -> Self {
        let vs = vs::load(context.device()).unwrap();
        let fs = fs::load(context.device()).unwrap();

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(context.device()));

        let cube = Mesh::cube(0.5, 0.5, 0.5, &memory_allocator);

        let uniform_buffer = CpuBufferPool::<vs::ty::Data>::new(
            memory_allocator.clone(),
            BufferUsage {
                uniform_buffer: true,
                ..Default::default()
            },
            MemoryUsage::Upload,
        );

        let render_pass = vulkano::single_pass_renderpass!(
            context.device(),
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

        let pipeline = GraphicsPipeline::start()
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .vertex_input_state(BuffersDefinition::new().vertex::<MeshVertex>())
            .input_assembly_state(InputAssemblyState::new())
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .fragment_shader(fs.entry_point("main").unwrap(), ())
            .build(context.device())
            .expect("could not create pipeline");

        // TODO: let the main_renderer manage those swapchain related framebuffers?
        let framebuffers = images
            .into_iter()
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
            .collect();

        let descriptor_set_allocator =
            Arc::new(StandardDescriptorSetAllocator::new(context.device()));

        SceneRenderer {
            render_pass,
            pipeline,
            mesh: cube,
            framebuffers,
            command_buffer_allocator,
            uniform_buffer,
            descriptor_set_allocator,
        }
    }

    pub fn resize(&mut self, images: &[Arc<ImageView<SwapchainImage>>]) {
        self.framebuffers = images
            .into_iter()
            .map(|image| {
                Framebuffer::new(
                    self.render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![image.clone()],
                        ..FramebufferCreateInfo::default()
                    },
                )
                .expect("failed to create framebuffer")
            })
            .collect();
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
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        // descriptor set
        let uniform_buffer_subbuffer = {
            let aspect_ratio = 1.0;
            let proj = cgmath::perspective(
                cgmath::Rad(std::f32::consts::FRAC_PI_2),
                aspect_ratio,
                0.01,
                100.0,
            );
            let view = Matrix4::look_at_rh(
                Point3::new(0.3, 0.3, 1.0),
                Point3::new(0.0, 0.0, 0.0),
                cgmath::Vector3::new(0.0, -1.0, 0.0),
            );
            let scale = Matrix4::from_scale(1.0);

            let uniform_data = vs::ty::Data {
                world: Matrix4::from(cgmath::Matrix3::from_angle_y(cgmath::Rad(1.1))).into(),
                view: (view * scale).into(),
                proj: proj.into(),
            };

            self.uniform_buffer.from_data(uniform_data).unwrap()
        };

        let layout = self.pipeline.layout().set_layouts().get(0).unwrap();
        // TODO: Don't create a new descriptor set every frame
        let set = PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            layout.clone(),
            [WriteDescriptorSet::buffer(0, uniform_buffer_subbuffer)],
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
                    clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
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
            .bind_pipeline_graphics(self.pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                set.clone(),
            )
            .bind_index_buffer(self.mesh.index_buffer.clone())
            .bind_vertex_buffers(0, self.mesh.vertex_buffer.clone())
            .draw_indexed(self.mesh.index_buffer.len() as u32, 1, 0, 0, 0)
            .unwrap()
            .end_render_pass()
            .unwrap();

        // Finish building the command buffer by calling `build`.
        let command_buffer = builder.build().unwrap();

        future
            .then_execute(context.queue(), command_buffer)
            .unwrap()
    }
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "assets/shaders/vert.glsl",
        types_meta: {
            use bytemuck::{Pod, Zeroable};
            #[derive(Clone, Copy, Zeroable, Pod)]
        }
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "assets/shaders/frag.glsl"
    }
}
