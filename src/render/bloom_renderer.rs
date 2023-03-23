use crate::render::context::Context;

use crate::render::custom_storage_image::CustomStorageImage;
use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferUsage, CopyImageInfo,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{
    AttachmentImage, ImageAccess, ImageCreateFlags, ImageDimensions, ImageLayout, ImageUsage,
};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::{ComputePipeline, PipelineBindPoint};
use vulkano::sync::GpuFuture;

pub struct BloomRenderer {
    downsample_pipeline: Arc<ComputePipeline>,
    upsample_pipeline: Arc<ComputePipeline>,

    images: Vec<Arc<ImageView<AttachmentImage>>>,
    image_objects: Vec<Arc<CustomStorageImage>>,

    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl BloomRenderer {
    pub fn new(
        context: &Context,
        images: Vec<Arc<ImageView<AttachmentImage>>>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    ) -> BloomRenderer {
        let image_objects = images
            .iter()
            .map(|_| {
                CustomStorageImage::uninitialized(
                    &memory_allocator,
                    ImageDimensions::Dim2d {
                        width: 1280,
                        height: 720,
                        array_layers: 1,
                    },
                    Format::R16G16B16A16_SFLOAT,
                    6,
                    ImageUsage {
                        sampled: true,
                        storage: true,
                        transfer_dst: true,
                        ..ImageUsage::empty()
                    },
                )
                .unwrap()
            })
            .collect();

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
            downsample_pipeline,
            upsample_pipeline,

            images,
            image_objects,

            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,
        }
    }

    pub fn resize(&mut self, images: &Vec<Arc<ImageView<impl ImageAccess>>>) {}

    pub fn render<F>(
        &self,
        context: &Context,
        future: F,
        image_index: u32,
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
            .copy_image(CopyImageInfo {
                // src_image_layout: ImageLayout::ColorAttachmentOptimal,
                dst_image_layout: ImageLayout::General,
                ..CopyImageInfo::images(
                    self.images[image_index as usize].image().clone(),
                    self.image_objects[image_index as usize].clone(),
                )
            })
            .unwrap();
        // .bind_pipeline_compute(self.downsample_pipeline.clone())
        // .bind_descriptor_sets(PipelineBindPoint::Compute, pipeline_layout.clone(), 0, set)
        // .dispatch([1280, 720, 1])
        // .unwrap()
        //     .;

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
