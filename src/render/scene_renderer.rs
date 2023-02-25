use crate::application::GameState;
use crate::context::Context;
use crate::render::SubRenderer;
use crate::scene::mesh::{Mesh, MeshVertex};
use crate::scene::scene_graph::{Model, SceneNode};
use cgmath::{Matrix4, Point3, SquareMatrix, Vector3};
use std::default::Default;
use std::sync::Arc;
use vulkano::buffer::{BufferUsage, CpuBufferPool, TypedBufferAccess};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferUsage, RenderPassBeginInfo,
    SubpassContents,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{DescriptorSet, PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{AttachmentImage, ImageViewAbstract, SwapchainImage};
use vulkano::memory::allocator::{MemoryUsage, StandardMemoryAllocator};
use vulkano::pipeline::graphics::depth_stencil::DepthStencilState;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::rasterization::{CullMode, RasterizationState};
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::sync::GpuFuture;

pub struct SceneRenderer {
    render_pass: Arc<RenderPass>,
    pipeline: Arc<GraphicsPipeline>,
    framebuffers: Vec<Arc<Framebuffer>>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,

    uniform_buffer: CpuBufferPool<vs::ty::Data>,
    // maybe move that to the main renderer?
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl SceneRenderer {
    pub fn new(
        context: &Context,
        images: &[Arc<ImageView<SwapchainImage>>],
        final_output_format: Format,
        memory_allocator: Arc<StandardMemoryAllocator>,
        command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    ) -> Self {
        let vs = vs::load(context.device()).unwrap();
        let fs = fs::load(context.device()).unwrap();

        // a pool of buffers, giving us more buffers as needed
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
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .depth_stencil_state(DepthStencilState::simple_depth_test())
            .rasterization_state(RasterizationState::new().cull_mode(CullMode::Back))
            .vertex_input_state(BuffersDefinition::new().vertex::<MeshVertex>())
            .input_assembly_state(InputAssemblyState::new())
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .fragment_shader(fs.entry_point("main").unwrap(), ())
            .build(context.device())
            .expect("could not create pipeline");

        // TODO: let the main_renderer manage those swapchain related framebuffers?

        let dimensions = images[0].dimensions().width_height();
        let depth_buffer = ImageView::new_default(
            AttachmentImage::transient(&memory_allocator, dimensions, Format::D32_SFLOAT).unwrap(),
        )
        .unwrap();

        let framebuffers = images
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
            .collect();

        let descriptor_set_allocator =
            Arc::new(StandardDescriptorSetAllocator::new(context.device()));

        SceneRenderer {
            render_pass,
            pipeline,
            framebuffers,
            memory_allocator,
            command_buffer_allocator,
            uniform_buffer,
            descriptor_set_allocator,
        }
    }
}

impl SubRenderer for SceneRenderer {
    fn resize(&mut self, images: &[Arc<ImageView<SwapchainImage>>]) {
        let dimensions = images[0].dimensions().width_height();
        let depth_buffer = ImageView::new_default(
            AttachmentImage::transient(&self.memory_allocator, dimensions, Format::D32_SFLOAT)
                .unwrap(),
        )
        .unwrap();

        self.framebuffers = images
            .into_iter()
            .map(|image| {
                Framebuffer::new(
                    self.render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![image.clone(), depth_buffer.clone()],
                        ..FramebufferCreateInfo::default()
                    },
                )
                .expect("failed to create framebuffer")
            })
            .collect();
    }

    fn render<F>(
        &self,
        context: &Context,
        game_state: &GameState,
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

        // read from scene
        let camera = &game_state.camera;
        let models = game_state
            .scene_graph
            .get_data_recursive::<Model, &SceneNode>(|node| node);

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
        let layout = self.pipeline.layout().set_layouts().get(0).unwrap();
        for (model, scene_node) in models {
            // descriptor set
            let uniform_buffer_subbuffer = {
                let proj = game_state.camera.proj();
                let view = game_state.camera.view();

                let uniform_data = vs::ty::Data {
                    world: scene_node.world_matrix().into(),
                    view: view.clone().into(),
                    proj: proj.clone().into(),
                };

                self.uniform_buffer.from_data(uniform_data).unwrap()
            };

            // TODO: Don't create a new descriptor set every frame
            /*
                let e = WriteDescriptorSet::buffer(0, uniform_buffer_subbuffer);
            set.resources().update(&e);
             */
            let set = PersistentDescriptorSet::new(
                &self.descriptor_set_allocator,
                layout.clone(),
                [WriteDescriptorSet::buffer(0, uniform_buffer_subbuffer)],
            )
            .unwrap();

            builder
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    self.pipeline.layout().clone(),
                    0,
                    set.clone(),
                )
                .bind_index_buffer(model.mesh.index_buffer.clone())
                .bind_vertex_buffers(0, model.mesh.vertex_buffer.clone())
                .draw_indexed(model.mesh.index_buffer.len() as u32, 1, 0, 0, 0)
                .unwrap();
        }

        builder.end_render_pass().unwrap();

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
