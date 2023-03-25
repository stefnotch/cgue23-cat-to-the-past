// Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use std::hash::{Hash, Hasher};
use std::sync::Arc;
use vulkano::device::{Device, DeviceOwned};
use vulkano::format::Format;
use vulkano::image::sys::{Image, ImageCreateInfo, RawImage};
use vulkano::image::traits::ImageContent;
use vulkano::image::view::ImageView;
use vulkano::image::{
    ImageAccess, ImageCreateFlags, ImageDescriptorLayouts, ImageDimensions, ImageError, ImageInner,
    ImageLayout, ImageUsage,
};
use vulkano::memory::allocator::{
    AllocationCreateInfo, AllocationType, MemoryAllocatePreference, MemoryAllocator, MemoryUsage,
};
use vulkano::memory::DedicatedAllocation;
use vulkano::sync::Sharing;

/// General-purpose image in device memory. Can be used for any usage, but will be slower than a
/// specialized image.
#[derive(Debug)]
pub struct StorageImage {
    inner: Arc<Image>,
}

impl StorageImage {
    /// Creates a new image with the given dimensions and format.
    pub fn new(
        allocator: &(impl MemoryAllocator + ?Sized),
        dimensions: ImageDimensions,
        format: Format,
    ) -> Result<Arc<StorageImage>, ImageError> {
        let aspects = format.aspects();
        let is_depth = aspects.depth || aspects.stencil;

        if format.compression().is_some() {
            panic!() // TODO: message?
        }

        let usage = ImageUsage {
            transfer_src: true,
            transfer_dst: true,
            sampled: true,
            storage: true,
            color_attachment: !is_depth,
            depth_stencil_attachment: is_depth,
            input_attachment: true,
            ..ImageUsage::empty()
        };
        let flags = ImageCreateFlags::empty();

        StorageImage::with_usage(allocator, dimensions, format, usage, flags)
    }

    /// Same as `new`, but allows specifying the usage.
    pub fn with_usage(
        allocator: &(impl MemoryAllocator + ?Sized),
        dimensions: ImageDimensions,
        format: Format,
        usage: ImageUsage,
        flags: ImageCreateFlags,
    ) -> Result<Arc<StorageImage>, ImageError> {
        assert!(!flags.disjoint); // TODO: adjust the code below to make this safe

        let raw_image = RawImage::new(
            allocator.device().clone(),
            ImageCreateInfo {
                flags,
                dimensions,
                format: Some(format),
                usage,
                sharing: Sharing::Exclusive,
                mip_levels: 2,
                ..Default::default()
            },
        )?;
        let requirements = raw_image.memory_requirements()[0];
        let create_info = AllocationCreateInfo {
            requirements,
            allocation_type: AllocationType::NonLinear,
            usage: MemoryUsage::GpuOnly,
            allocate_preference: MemoryAllocatePreference::Unknown,
            dedicated_allocation: Some(DedicatedAllocation::Image(&raw_image)),
            ..Default::default()
        };

        match unsafe { allocator.allocate_unchecked(create_info) } {
            Ok(alloc) => {
                debug_assert!(alloc.offset() % requirements.alignment == 0);
                debug_assert!(alloc.size() == requirements.size);
                let inner = Arc::new(unsafe {
                    raw_image
                        .bind_memory_unchecked([alloc])
                        .map_err(|(err, _, _)| err)?
                });

                Ok(Arc::new(StorageImage { inner }))
            }
            Err(err) => Err(err.into()),
        }
    }

    /// Allows the creation of a simple 2D general purpose image view from `StorageImage`.
    #[inline]
    pub fn general_purpose_image_view(
        allocator: &(impl MemoryAllocator + ?Sized),
        size: [u32; 2],
        format: Format,
        usage: ImageUsage,
    ) -> Result<Arc<ImageView<StorageImage>>, ImageError> {
        let dims = ImageDimensions::Dim2d {
            width: size[0],
            height: size[1],
            array_layers: 1,
        };
        let flags = ImageCreateFlags::empty();
        let image_result = StorageImage::with_usage(allocator, dims, format, usage, flags);

        match image_result {
            Ok(image) => {
                let image_view = ImageView::new_default(image);
                match image_view {
                    Ok(view) => Ok(view),
                    Err(e) => Err(ImageError::DirectImageViewCreationFailed(e)),
                }
            }
            Err(e) => Err(e),
        }
    }
}

unsafe impl DeviceOwned for StorageImage {
    #[inline]
    fn device(&self) -> &Arc<Device> {
        self.inner.device()
    }
}

unsafe impl ImageAccess for StorageImage {
    #[inline]
    fn inner(&self) -> ImageInner<'_> {
        ImageInner {
            image: &self.inner,
            first_layer: 0,
            num_layers: self.inner.dimensions().array_layers(),
            first_mipmap_level: 0,
            num_mipmap_levels: self.inner.mip_levels(),
        }
    }

    #[inline]
    fn initial_layout_requirement(&self) -> ImageLayout {
        ImageLayout::General
    }

    #[inline]
    fn final_layout_requirement(&self) -> ImageLayout {
        ImageLayout::General
    }

    #[inline]
    fn descriptor_layouts(&self) -> Option<ImageDescriptorLayouts> {
        Some(ImageDescriptorLayouts {
            storage_image: ImageLayout::General,
            combined_image_sampler: ImageLayout::General,
            sampled_image: ImageLayout::General,
            input_attachment: ImageLayout::General,
        })
    }
}

unsafe impl<P> ImageContent<P> for StorageImage {
    fn matches_format(&self) -> bool {
        true // FIXME:
    }
}

impl PartialEq for StorageImage {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner() == other.inner()
    }
}

impl Eq for StorageImage {}

impl Hash for StorageImage {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner().hash(state);
    }
}
