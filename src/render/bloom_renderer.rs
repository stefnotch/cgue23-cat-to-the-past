use crate::render::context::Context;

use crate::render::custom_storage_image::StorageImage;
use std::sync::Arc;
use vulkano::buffer::{BufferUsage, CpuBufferPool};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferUsage, CopyImageInfo, ImageCopy,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::format::Format;
use vulkano::image::view::{ImageView, ImageViewCreateInfo};
use vulkano::image::{
    AttachmentImage, ImageAccess, ImageSubresourceLayers, ImageSubresourceRange, ImageUsage,
    ImageViewAbstract, ImageViewType,
};
use vulkano::memory::allocator::{MemoryUsage, StandardMemoryAllocator};
use vulkano::pipeline::{ComputePipeline, Pipeline, PipelineBindPoint};
use vulkano::sampler::{Filter, Sampler, SamplerCreateInfo, SamplerMipmapMode};
use vulkano::shader::spirv::Dim::Buffer;
use vulkano::sync::GpuFuture;

pub struct BloomRenderer {
    downsample_pipeline: Arc<ComputePipeline>,
    upsample_pipeline: Arc<ComputePipeline>,

    images: Vec<Arc<ImageView<AttachmentImage>>>,
    image_objects: Vec<Arc<ImageView<StorageImage>>>,

    uniform_buffer_pool_downsample_pass: CpuBufferPool<cs::downsample::ty::Pass>,
    sampler: Arc<Sampler>,

    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl BloomRenderer {
    pub fn new(
        context: &Context,
        dimensions: [u32; 2],
        swapchain_image_count: u32,
        images: Vec<Arc<ImageView<AttachmentImage>>>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    ) -> BloomRenderer {
        let image_objects =
            Self::create_images(memory_allocator.clone(), swapchain_image_count, dimensions);

        let uniform_buffer_pool_downsample_pass = CpuBufferPool::new(
            memory_allocator.clone(),
            BufferUsage {
                uniform_buffer: true,
                ..Default::default()
            },
            MemoryUsage::Upload,
        );

        let sampler = Sampler::new(
            context.device(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                mipmap_mode: SamplerMipmapMode::Nearest,
                ..Default::default()
            },
        )
        .unwrap();

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

            uniform_buffer_pool_downsample_pass,
            sampler,

            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,
        }
    }

    pub fn resize(&mut self, images: &Vec<Arc<ImageView<AttachmentImage>>>) {
        let dimensions = images[0].dimensions().width_height();
        let swapchain_image_count = images.len() as u32;

        self.image_objects = Self::create_images(
            self.memory_allocator.clone(),
            swapchain_image_count,
            dimensions,
        );
    }

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

        let scene_image = self.images[image_index as usize].image().clone();

        let work_image = self.image_objects[image_index as usize].clone();

        let region = ImageCopy {
            src_subresource: ImageSubresourceLayers {
                aspects: scene_image.format().aspects(),
                mip_level: 0,
                array_layers: 0..1,
            },
            dst_subresource: ImageSubresourceLayers {
                aspects: work_image.image().format().aspects(),
                mip_level: 0,
                array_layers: 0..1,
            },
            src_offset: [0; 3],
            dst_offset: [0; 3],
            extent: scene_image.dimensions().width_height_depth(),
            ..ImageCopy::default()
        };

        let downsample_pass_set_layout = self
            .downsample_pipeline
            .layout()
            .set_layouts()
            .get(0)
            .unwrap();

        // descriptor set
        let uniform_subbuffer_downsample_pass = {
            let uniform_data = cs::downsample::ty::Pass {
                mipLevel: 0,
                _dummy0: Default::default(),
                texelSize: work_image
                    .dimensions()
                    .width_height()
                    .map(|v| 1.0 / v as f32)
                    .into(),
            };

            self.uniform_buffer_pool_downsample_pass
                .from_data(uniform_data)
                .unwrap()
        };

        let output_image = ImageView::new(
            work_image.image().clone(),
            ImageViewCreateInfo {
                view_type: ImageViewType::Dim2d,
                format: work_image.format(),
                component_mapping: work_image.component_mapping(),
                subresource_range: ImageSubresourceRange {
                    aspects: work_image.subresource_range().aspects.clone(),
                    mip_levels: 1..2,
                    array_layers: work_image.subresource_range().array_layers.clone(),
                },
                usage: work_image.usage().clone(),
                ..ImageViewCreateInfo::default()
            },
        )
        .unwrap();

        let pass_descriptor_set = PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            downsample_pass_set_layout.clone(),
            [
                WriteDescriptorSet::buffer(0, uniform_subbuffer_downsample_pass),
                WriteDescriptorSet::image_view_sampler(1, work_image.clone(), self.sampler.clone()),
                WriteDescriptorSet::image_view(2, output_image),
            ],
        )
        .unwrap();

        let [width, height, _] = work_image.dimensions().width_height_depth();

        builder
            .copy_image(CopyImageInfo {
                regions: [region].into(),
                ..CopyImageInfo::images(scene_image, work_image.image().clone())
            })
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                self.downsample_pipeline.layout().clone(),
                0,
                pass_descriptor_set,
            )
            .bind_pipeline_compute(self.downsample_pipeline.clone())
            .dispatch([width / 2, height / 2, 1])
            .unwrap();

        let command_buffer = builder.build().unwrap();

        future
            .then_execute(context.queue(), command_buffer)
            .unwrap()
    }
    fn create_images(
        memory_allocator: Arc<StandardMemoryAllocator>,
        swapchain_image_count: u32,
        dimensions: [u32; 2],
    ) -> Vec<Arc<ImageView<StorageImage>>> {
        (0..swapchain_image_count)
            .map(|_| {
                StorageImage::general_purpose_image_view(
                    &memory_allocator,
                    dimensions,
                    Format::R16G16B16A16_SFLOAT,
                    ImageUsage {
                        sampled: true,
                        storage: true,
                        color_attachment: true,
                        transfer_dst: true,
                        ..ImageUsage::empty()
                    },
                )
                .unwrap()
            })
            .collect()
    }
}

mod cs {
    pub mod downsample {
        vulkano_shaders::shader! {
            ty: "compute",
            path: "assets/shaders/bloom/downsample.comp",
            types_meta: {
                use bytemuck::{Pod, Zeroable};
                #[derive(Clone, Copy, Zeroable, Pod, Debug)]
            }
        }
    }

    pub mod upsample {
        vulkano_shaders::shader! {
            ty: "compute",
            path: "assets/shaders/bloom/upsample.comp",
            types_meta: {
                use bytemuck::{Pod, Zeroable};
                #[derive(Clone, Copy, Zeroable, Pod, Debug)]
            }
        }
    }
}
