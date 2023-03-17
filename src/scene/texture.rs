use super::loader::Asset;
use crate::context::Context;
use std::sync::Arc;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{ImageDimensions, StorageImage};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::sync;
use vulkano::sync::GpuFuture;

pub struct Texture {}

impl Texture {
    pub fn from_gltf_image(image_data: gltf::image::Data, context: &Context) -> Arc<Texture> {
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
                width: image_data.width,
                height: image_data.height,
                array_layers: 1,
            };

            let image = StorageImage::new(
                &memory_allocator,
                dimensions,
                gltf_image_format_to_vulkan_format(&image_data.format),
                [context.queue_family_index()],
            )
            .unwrap();

            let buffer = CpuAccessibleBuffer::from_iter(
                &memory_allocator,
                BufferUsage {
                    transfer_src: true,
                    ..BufferUsage::empty()
                },
                false,
                image_data.pixels,
            )
            .unwrap();
        };

        let command_buffer = command_buffer_builder.build().unwrap();

        let future = future
            .then_execute(context.queue(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();

        Arc::new(Texture {})
    }
}

fn gltf_image_format_to_vulkan_format(format: &gltf::image::Format) -> Format {
    match format {
        gltf::image::Format::R8 => Format::R8_UINT,
        gltf::image::Format::R8G8 => Format::R8G8_UINT,
        gltf::image::Format::R8G8B8 => Format::R8G8B8_UINT,
        gltf::image::Format::R8G8B8A8 => Format::R8G8B8A8_UINT,
        gltf::image::Format::R16 => Format::R16_UINT,
        gltf::image::Format::R16G16 => Format::R16G16_UINT,
        gltf::image::Format::R16G16B16 => Format::R16G16B16_UINT,
        gltf::image::Format::R16G16B16A16 => Format::R16G16B16A16_UINT,
        gltf::image::Format::R32G32B32FLOAT => Format::R32G32B32_SFLOAT,
        gltf::image::Format::R32G32B32A32FLOAT => Format::R32G32B32A32_SFLOAT,
    }
}

impl Asset for Texture {}
