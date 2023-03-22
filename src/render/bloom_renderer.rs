use crate::render::context::Context;

use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferUsage,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::image::view::ImageView;
use vulkano::image::ImageAccess;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::{ComputePipeline, PipelineBindPoint};
use vulkano::sync::GpuFuture;

pub struct BloomRenderer {
    downsample_pipeline: Arc<ComputePipeline>,
    upsample_pipeline: Arc<ComputePipeline>,

    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl BloomRenderer {
    pub fn new(
        context: &Context,
        input_images: Vec<Arc<ImageView<impl ImageAccess>>>,
        output_images: Vec<Arc<ImageView<impl ImageAccess>>>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    ) -> BloomRenderer {
        let downsample_pipeline = {
            let shader = cs::downsample::load(context.device()).unwrap();

            ComputePipeline::new(
                context.device(),
                shader.entry_point("main").unwrap(),
                &(),
                None,
                |_| {},
            )
            .unwrap()
        };

        let upsample_pipeline = {
            let shader = cs::upsample::load(context.device()).unwrap();

            ComputePipeline::new(
                context.device(),
                shader.entry_point("main").unwrap(),
                &(),
                None,
                |_| {},
            )
            .unwrap()
        };

        BloomRenderer {
            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,
            downsample_pipeline,
            upsample_pipeline,
        }
    }

    pub fn resize(&mut self, images: &Vec<Arc<ImageView<impl ImageAccess>>>) {}

    pub fn render<F>(&self, context: &Context, future: F) -> CommandBufferExecFuture<F>
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

        builder.bind_pipeline_compute(self.downsample_pipeline.clone());
        // .bind_descriptor_sets(PipelineBindPoint::Compute, pipeline_layout.clone(), 0, set)
        // .dispatch(image_dimensions[0], image_dimensions[1], 1)
        // .unwrap();

        let command_buffer = builder.build().unwrap();

        future
            .then_execute(context.queue(), command_buffer)
            .unwrap()
    }
}

mod cs {
    pub mod downsample {
        vulkano_shaders::shader! {
            ty: "compute",
            path: "assets/shaders/bloom/downsample.comp",
        }
    }

    pub mod upsample {
        vulkano_shaders::shader! {
            ty: "compute",
            path: "assets/shaders/bloom/upsample.comp",
        }
    }
}
