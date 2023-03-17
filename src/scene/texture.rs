use super::loader::Asset;
use crate::context::Context;
use std::sync::Arc;
use vulkano::buffer::BufferContents;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBufferAbstract,
};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{ImageDimensions, ImmutableImage, MipmapsCount};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::sampler::Sampler;
use vulkano::sync;
use vulkano::sync::GpuFuture;

pub struct Texture {
    pub image_view: Arc<ImageView<ImmutableImage>>,
    pub sampler: Arc<Sampler>,
}

impl Texture {
    pub fn from_gltf_image(
        image_data: gltf::image::Data,
        sampler: Arc<Sampler>,
        context: &Context,
    ) -> Arc<Texture> {
        // Widely supported formats https://vulkan.gpuinfo.org/listlineartilingformats.php

        let width = image_data.width;
        let height = image_data.height;
        let (image, format) =
            gltf_image_format_to_vulkan_format(image_data.pixels, &image_data.format);
        Self::new(image, width, height, format, sampler, context)
    }

    pub fn new_one_by_one(sampler: Arc<Sampler>, context: &Context) -> Arc<Texture> {
        Self::new(
            vec![255u8, 255u8, 255u8, 255u8],
            1,
            1,
            Format::R8G8B8A8_UNORM,
            sampler,
            context,
        )
    }

    pub fn new<I, Px>(
        data_iterator: I,
        width: u32,
        height: u32,
        format: Format,
        sampler: Arc<Sampler>,
        context: &Context,
    ) -> Arc<Texture>
    where
        [Px]: BufferContents,
        I: IntoIterator<Item = Px>,
        I::IntoIter: ExactSizeIterator,
    {
        let future = sync::now(context.device()).boxed();

        let command_buffer_allocator =
            StandardCommandBufferAllocator::new(context.device(), Default::default());

        let memory_allocator = StandardMemoryAllocator::new_default(context.device());

        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            &command_buffer_allocator,
            context.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let texture = {
            let dimensions = ImageDimensions::Dim2d {
                width,
                height,
                array_layers: 1,
            };

            let image = ImmutableImage::from_iter(
                &memory_allocator,
                data_iterator,
                dimensions,
                MipmapsCount::One,
                format,
                &mut command_buffer_builder,
            )
            .unwrap();

            ImageView::new_default(image).unwrap()
        };

        let command_buffer = command_buffer_builder.build().unwrap();

        let future = future
            .then_execute(context.queue(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();

        Arc::new(Texture {
            image_view: texture,
            sampler,
        })
    }
}

fn gltf_image_format_to_vulkan_format(
    image: Vec<u8>,
    format: &gltf::image::Format,
) -> (Vec<u8>, Format) {
    match format {
        gltf::image::Format::R8 => (image, Format::R8_UNORM),
        gltf::image::Format::R8G8 => (image, Format::R8G8_UNORM),
        gltf::image::Format::R8G8B8 => {
            // rarely supported format
            let mut image_with_alpha = Vec::new();
            for i in 0..image.len() / 3 {
                image_with_alpha.push(image[i * 3]);
                image_with_alpha.push(image[i * 3 + 1]);
                image_with_alpha.push(image[i * 3 + 2]);
                image_with_alpha.push(255);
            }
            (image_with_alpha, Format::R8G8B8A8_UNORM)
        }
        gltf::image::Format::R8G8B8A8 => (image, Format::R8G8B8A8_UNORM),
        gltf::image::Format::R16 => (image, Format::R16_UNORM),
        gltf::image::Format::R16G16 => (image, Format::R16G16_UNORM),
        gltf::image::Format::R16G16B16 => {
            // rarely supported format
            todo!()
        }
        gltf::image::Format::R16G16B16A16 => (image, Format::R16G16B16A16_UNORM),
        gltf::image::Format::R32G32B32FLOAT => {
            // rarely supported format
            todo!()
        }
        gltf::image::Format::R32G32B32A32FLOAT => (image, Format::R32G32B32A32_SFLOAT),
    }
}

impl Asset for Texture {}
