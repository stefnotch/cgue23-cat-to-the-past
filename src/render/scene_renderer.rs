use std::sync::Arc;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, CpuBufferPool};
use vulkano::command_buffer::{RenderPassBeginInfo, SubpassContents};
use vulkano::format::Format;
use vulkano::image::ImageViewAbstract;
use vulkano::memory::allocator::{MemoryUsage, StandardMemoryAllocator};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::pipeline::{GraphicsPipeline, PipelineBindPoint};
use vulkano::render_pass::Subpass;
use vulkano::sync::GpuFuture;
use crate::context::Context;
use crate::mesh::{Mesh, MeshVertex};

pub struct SceneRenderer {}

impl SceneRenderer {
    pub fn new(
        context: &Context,
        final_output_format: Format,
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

        SceneRenderer {
            render_pass,
            pipeline,
            mesh: cube
        }
    }

    pub fn resize(&self) {}

    pub fn render<F>(
        &self,
        future: F,
        final_image: Arc<dyn ImageViewAbstract<Handle=()> + 'static>,
    )
        where F: GpuFuture + 'static
    {
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
                    ..RenderPassBeginInfo::framebuffer(final_image)
                },
                SubpassContents::Inline,
            )
            .unwrap()
            // We are now inside the first subpass of the render pass. We add a draw command.
            //
            // The last two parameters contain the list of resources to pass to the shaders.
            // Since we used an `EmptyPipeline` object, the objects have to be `()`.
            .set_viewport(0, [viewport.clone()])
            .bind_pipeline_graphics(pipeline.clone())
            .bind_descriptor_sets(PipelineBindPoint::Graphics, pipeline.layout().clone(), 0, set.clone())
            .bind_index_buffer(index_buffer.clone())
            .bind_vertex_buffers(0, vertex_buffer.clone())
            .draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0)
            .unwrap()
            .end_render_pass()
            .unwrap();

        // Finish building the command buffer by calling `build`.
        let command_buffer = builder.build().unwrap();
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