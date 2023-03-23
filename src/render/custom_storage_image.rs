// Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};
use vulkano::device::{Device, DeviceOwned};
use vulkano::format::Format;
use vulkano::image::sys::{Image, ImageCreateInfo, RawImage};
use vulkano::image::traits::ImageContent;
use vulkano::image::{
    ImageAccess, ImageCreateFlags, ImageDescriptorLayouts, ImageDimensions, ImageError, ImageInner,
    ImageLayout, ImageUsage,
};
use vulkano::memory::allocator::{
    AllocationCreateInfo, AllocationType, MemoryAllocatePreference, MemoryAllocator, MemoryUsage,
};
use vulkano::memory::DedicatedAllocation;
use vulkano::sync::Sharing;

/// Similar to Vulkano's StorageImage but with multi mip-level support
#[derive(Debug)]
pub struct CustomStorageImage {
    inner: Arc<Image>,
}

impl CustomStorageImage {
    /// Builds an uninitialized image.
    ///
    /// Returns the image
    pub fn uninitialized(
        allocator: &(impl MemoryAllocator + ?Sized),
        dimensions: ImageDimensions,
        format: Format,
        num_mip_levels: u32,
        usage: ImageUsage,
    ) -> Result<Arc<CustomStorageImage>, ImageError> {
        let raw_image = RawImage::new(
            allocator.device().clone(),
            ImageCreateInfo {
                flags: ImageCreateFlags::empty(),
                dimensions,
                format: Some(format),
                mip_levels: num_mip_levels,
                usage,
                sharing: Sharing::Exclusive,
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

                let image = Arc::new(CustomStorageImage { inner });

                Ok(image)
            }
            Err(err) => Err(err.into()),
        }
    }
}

unsafe impl DeviceOwned for CustomStorageImage {
    #[inline]
    fn device(&self) -> &Arc<Device> {
        self.inner.device()
    }
}

unsafe impl ImageAccess for CustomStorageImage {
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
    fn is_layout_initialized(&self) -> bool {
        true
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

unsafe impl<P> ImageContent<P> for CustomStorageImage {
    fn matches_format(&self) -> bool {
        true // FIXME:
    }
}

impl PartialEq for CustomStorageImage {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner() == other.inner()
    }
}

impl Eq for CustomStorageImage {}

impl Hash for CustomStorageImage {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner().hash(state);
    }
}
