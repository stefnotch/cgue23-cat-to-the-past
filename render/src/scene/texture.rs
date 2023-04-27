use crate::context::Context;
use scene::asset::{Asset, AssetId};
use std::sync::Arc;
use vulkano::buffer::BufferContents;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{ImageDimensions, ImmutableImage, MipmapsCount};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::sampler::Sampler;
use vulkano::sync;
use vulkano::sync::GpuFuture;

#[derive(Debug, PartialEq)]
pub struct Texture {
    pub id: AssetId,
    pub image_view: Arc<ImageView<ImmutableImage>>,
    pub sampler: Arc<Sampler>,
}

impl Texture {
    pub fn new_one_by_one(sampler: Arc<Sampler>, context: &Context) -> Arc<Texture> {
        Self::new(
            AssetId::new_v4(),
            vec![255u8, 255u8, 255u8, 255u8],
            1,
            1,
            Format::R8G8B8A8_UNORM,
            sampler,
            context,
        )
    }

    pub fn new<I, Px>(
        id: AssetId,
        data_iterator: I,
        width: u32,
        height: u32,
        format: Format,
        sampler: Arc<Sampler>,
        context: &Context,
    ) -> Arc<Texture>
    where
        Px: BufferContents,
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
            id,
            image_view: texture,
            sampler,
        })
    }
}

impl Asset for Texture {
    fn id(&self) -> AssetId {
        self.id
    }
}
