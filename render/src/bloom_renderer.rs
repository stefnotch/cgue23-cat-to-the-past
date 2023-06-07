use crate::context::Context;

use crate::custom_storage_image::CustomStorageImage;
use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferUsage, CopyImageInfo,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::image::view::{ImageView, ImageViewCreateInfo, ImageViewCreationError};
use vulkano::image::{
    AttachmentImage, ImageAccess, ImageCreateFlags, ImageDimensions, ImageLayout,
    ImageSubresourceRange, ImageUsage, ImageViewAbstract,
};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::{ComputePipeline, Pipeline, PipelineBindPoint};
use vulkano::sampler::{Filter, Sampler, SamplerCreateInfo, SamplerMipmapMode};
use vulkano::sync::GpuFuture;

pub struct BloomRenderer {
    downsample_pipeline: Arc<ComputePipeline>,
    upsample_pipeline: Arc<ComputePipeline>,

    input_images: Vec<Arc<ImageView<AttachmentImage>>>,
    output_images: Vec<ImageWithMipViews>,

    sampler: Arc<Sampler>,

    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl BloomRenderer {
    pub fn new(
        context: &Context,
        input_images: Vec<Arc<ImageView<AttachmentImage>>>,
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

        let output_images = input_images
            .iter()
            .map(|input_image| {
                ImageWithMipViews::new(input_image.clone(), memory_allocator.clone())
            })
            .collect();

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

        BloomRenderer {
            downsample_pipeline,
            upsample_pipeline,
            sampler,

            input_images,
            output_images,

            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,
        }
    }

    pub fn resize(&mut self, input_images: Vec<Arc<ImageView<AttachmentImage>>>) {
        self.input_images = input_images.clone();
        self.output_images = input_images
            .iter()
            .map(|input_image| {
                ImageWithMipViews::new(input_image.clone(), self.memory_allocator.clone())
            })
            .collect();
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
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let scene_image = self.input_images[image_index as usize].image().clone();
        let work_image = &self.output_images[image_index as usize];

        // copy scene image to work image
        builder
            .copy_image(CopyImageInfo::images(
                scene_image.clone(),
                work_image.get_image(),
            ))
            .unwrap();

        // downsample passes
        builder.bind_pipeline_compute(self.downsample_pipeline.clone());

        let downsample_set_layout = self
            .downsample_pipeline
            .layout()
            .set_layouts()
            .get(0)
            .unwrap();

        for i in 0..(work_image.get_image().mip_levels() - 1) {
            let input_miplevel = i;
            let output_miplevel = i + 1;

            let output_image_view = work_image.get_mip_view(output_miplevel);
            let input_image_view = work_image.get_mip_view(input_miplevel);

            let downsample_descriptor_set = PersistentDescriptorSet::new(
                &self.descriptor_set_allocator,
                downsample_set_layout.clone(),
                [
                    WriteDescriptorSet::image_view_sampler(
                        0,
                        input_image_view.clone(),
                        self.sampler.clone(),
                    ),
                    WriteDescriptorSet::image_view(1, output_image_view.clone()),
                ],
            )
            .unwrap();

            let input_size = work_image
                .image
                .dimensions()
                .mip_level_dimensions(input_miplevel)
                .unwrap();
            let output_size = work_image
                .image
                .dimensions()
                .mip_level_dimensions(output_miplevel)
                .unwrap();

            let downsample_pass = cs::downsample::Pass {
                inputTexelSize: input_size.width_height().map(|v| 1.0 / (v as f32)),
                isFirstPass: (input_miplevel == 0) as u32,
                threshold: 1.0, // TODO: make this configurable
                knee: 0.1,
            };

            builder
                .push_constants(
                    self.downsample_pipeline.layout().clone(),
                    0,
                    downsample_pass,
                )
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    self.downsample_pipeline.layout().clone(),
                    0,
                    downsample_descriptor_set.clone(),
                )
                .dispatch(output_size.width_height_depth())
                .unwrap();
        }

        // upsample passes

        builder.bind_pipeline_compute(self.upsample_pipeline.clone());

        let upsample_set_layout = self
            .upsample_pipeline
            .layout()
            .set_layouts()
            .get(0)
            .unwrap();

        for i in (0..(work_image.get_image().mip_levels() - 1)).rev() {
            let input_miplevel = i + 1;
            let output_miplevel = i;

            let output_image_view = work_image.get_mip_view(output_miplevel);
            let input_image_view = work_image.get_mip_view(input_miplevel);

            let upsample_descriptor_set = PersistentDescriptorSet::new(
                &self.descriptor_set_allocator,
                upsample_set_layout.clone(),
                [
                    WriteDescriptorSet::image_view_sampler(
                        0,
                        input_image_view.clone(),
                        self.sampler.clone(),
                    ),
                    WriteDescriptorSet::image_view(1, output_image_view.clone()),
                ],
            )
            .unwrap();

            let input_size = work_image
                .image
                .dimensions()
                .mip_level_dimensions(input_miplevel)
                .unwrap();
            let output_size = work_image
                .image
                .dimensions()
                .mip_level_dimensions(output_miplevel)
                .unwrap();

            let upsample_pass = cs::upsample::Pass {
                inputTexelSize: input_size.width_height().map(|v| 1.0 / (v as f32)),
                intensity: 1.0, // TODO: make this configurable
            };

            builder
                .push_constants(self.upsample_pipeline.layout().clone(), 0, upsample_pass)
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    self.upsample_pipeline.layout().clone(),
                    0,
                    upsample_descriptor_set.clone(),
                )
                .dispatch(output_size.width_height_depth())
                .unwrap();
        }
        let command_buffer = builder.build().unwrap();

        future
            .then_execute(context.queue(), command_buffer)
            .unwrap()
    }

    pub fn output_images(&self) -> Vec<Arc<ImageView<CustomStorageImage>>> {
        self.output_images
            .iter()
            .map(|image| image.image.clone())
            .collect()
    }
}

fn single_miplevel_imageview<I>(
    image: Arc<I>,
    mip_level: u32,
) -> Result<Arc<ImageView<I>>, ImageViewCreationError>
where
    I: ImageAccess + ?Sized,
{
    ImageView::new(
        image.clone(),
        ImageViewCreateInfo {
            format: Some(image.format()),
            subresource_range: ImageSubresourceRange {
                mip_levels: mip_level..(mip_level + 1),
                ..image.subresource_range()
            },
            ..ImageViewCreateInfo::default()
        },
    )
}

struct ImageWithMipViews {
    image: Arc<ImageView<CustomStorageImage>>,
    mip_views: Vec<Arc<ImageView<CustomStorageImage>>>,
}

impl ImageWithMipViews {
    fn new(
        input_image: Arc<ImageView<AttachmentImage>>,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Self {
        let image = Self::create_output_image(input_image, memory_allocator);
        let mip_views = Self::create_mip_image_views(image.image().clone());
        Self { image, mip_views }
    }

    fn create_output_image(
        input_image: Arc<ImageView<AttachmentImage>>,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Arc<ImageView<CustomStorageImage>> {
        let [width, height] = input_image.dimensions().width_height();
        let storage_image = CustomStorageImage::uninitialized(
            &memory_allocator,
            ImageDimensions::Dim2d {
                width,
                height,
                array_layers: 1,
            },
            input_image.image().format(),
            6,
            ImageUsage::TRANSFER_DST | ImageUsage::STORAGE | ImageUsage::SAMPLED,
            ImageCreateFlags::empty(),
            ImageLayout::General,
        )
        .unwrap();

        let view = ImageView::new(
            storage_image.clone(),
            ImageViewCreateInfo {
                format: Some(storage_image.format()),
                subresource_range: ImageSubresourceRange {
                    mip_levels: 0..1,
                    ..storage_image.subresource_range()
                },
                ..ImageViewCreateInfo::default()
            },
        )
        .unwrap();
        view
    }

    fn get_image(&self) -> Arc<CustomStorageImage> {
        self.image.image().clone()
    }

    fn create_mip_image_views(
        image: Arc<CustomStorageImage>,
    ) -> Vec<Arc<ImageView<CustomStorageImage>>> {
        (0..image.mip_levels())
            .map(|i| single_miplevel_imageview(image.clone(), i).unwrap())
            .collect()
    }

    fn get_mip_view(&self, output_miplevel: u32) -> Arc<ImageView<CustomStorageImage>> {
        self.mip_views[output_miplevel as usize].clone()
    }
}

mod cs {
    pub mod downsample {
        vulkano_shaders::shader! {
            ty: "compute",
            path: "../assets/shaders/bloom/downsample.comp",
        }
    }

    pub mod upsample {
        vulkano_shaders::shader! {
            ty: "compute",
            path: "../assets/shaders/bloom/upsample.comp",
        }
    }
}
