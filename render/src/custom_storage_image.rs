// Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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

/// Similar to vulkano's StorageImage but with multi mip-level support
#[derive(Debug)]
pub struct CustomStorageImage {
    inner: Arc<Image>,

    // If true, then the image is in the layout `target_layout`. If false, then it
    // is still `Undefined`.
    layout_initialized: AtomicBool,

    target_layout: ImageLayout,
}

impl CustomStorageImage {
    /// Builds an uninitialized storage image.
    ///
    /// Returns the uninitialized image
    pub fn uninitialized(
        allocator: &(impl MemoryAllocator + ?Sized),
        dimensions: ImageDimensions,
        format: Format,
        num_mip_levels: u32,
        usage: ImageUsage,
        flags: ImageCreateFlags,
        target_layout: ImageLayout,
    ) -> Result<Arc<CustomStorageImage>, ImageError> {
        assert!(!flags.intersects(ImageCreateFlags::DISJOINT));

        let raw_image = RawImage::new(
            allocator.device().clone(),
            ImageCreateInfo {
                flags,
                dimensions,
                format: Some(format),
                mip_levels: num_mip_levels,
                usage,
                sharing: Sharing::Exclusive, // Note: assuming exclusive sharing
                ..Default::default()
            },
        )?;
        let requirements = raw_image.memory_requirements()[0];
        let res = unsafe {
            allocator.allocate_unchecked(
                requirements,
                AllocationType::NonLinear,
                AllocationCreateInfo {
                    usage: MemoryUsage::DeviceOnly,
                    allocate_preference: MemoryAllocatePreference::Unknown,
                    ..Default::default()
                },
                Some(DedicatedAllocation::Image(&raw_image)),
            )
        };

        match res {
            Ok(alloc) => {
                debug_assert!(
                    alloc.offset() & (requirements.layout.alignment().as_devicesize() - 1) == 0
                );
                debug_assert!(alloc.size() == requirements.layout.size());

                let inner = Arc::new(
                    unsafe { raw_image.bind_memory_unchecked([alloc]) }
                        .map_err(|(err, _, _)| err)?,
                );

                let image = Arc::new(CustomStorageImage {
                    inner,
                    layout_initialized: AtomicBool::new(false),
                    target_layout,
                });

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
    unsafe fn layout_initialized(&self) {
        self.layout_initialized.store(true, Ordering::Relaxed);
    }

    #[inline]
    fn is_layout_initialized(&self) -> bool {
        self.layout_initialized.load(Ordering::Relaxed)
    }

    #[inline]
    fn initial_layout_requirement(&self) -> ImageLayout {
        self.target_layout
        //ImageLayout::General
    }

    #[inline]
    fn final_layout_requirement(&self) -> ImageLayout {
        self.target_layout
        //ImageLayout::General
    }

    #[inline]
    fn descriptor_layouts(&self) -> Option<ImageDescriptorLayouts> {
        Some(ImageDescriptorLayouts {
            storage_image: self.target_layout,
            combined_image_sampler: ImageLayout::ShaderReadOnlyOptimal,
            sampled_image: ImageLayout::ShaderReadOnlyOptimal,
            input_attachment: self.target_layout,
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
