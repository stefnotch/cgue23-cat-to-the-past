use crate::context::Context;
use crate::render::scene_renderer::SceneRenderer;
use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{ImageAccess, ImageUsage, SwapchainImage};
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::swapchain::{
    acquire_next_image, AcquireError, ColorSpace, Surface, SurfaceInfo, Swapchain,
    SwapchainCreateInfo, SwapchainCreationError, SwapchainPresentInfo,
};
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};
use winit::window::Window;

/// Responsible for keeping the swapchain up-to-date and calling the sub-rendersystems
pub struct Renderer {
    recreate_swapchain: bool,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    swapchain: Arc<Swapchain>,
    images: Vec<Arc<ImageView<SwapchainImage>>>,
    scene_renderer: SceneRenderer,
    viewport: Viewport,
}

impl Renderer {
    pub fn new(context: &Context) -> Renderer {
        let previous_frame_end = Some(sync::now(context.device()).boxed());

        let (swapchain, images) = create_swapchain(context.device(), context.surface());

        let dimensions = images[0].dimensions().width_height().map(|i| i as f32);

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions,
            depth_range: 0.0..1.0,
        };

        let images = images
            .into_iter()
            .map(|image| ImageView::new_default(image.clone()).unwrap())
            .collect::<Vec<_>>();

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            context.device(),
            Default::default(),
        ));

        let scene_renderer = SceneRenderer::new(
            context,
            &images,
            swapchain.image_format(),
            command_buffer_allocator,
        );

        Renderer {
            recreate_swapchain: false,
            previous_frame_end,
            swapchain,
            images,
            scene_renderer,
            viewport,
        }
    }

    pub fn recreate_swapchain(&mut self) {
        self.recreate_swapchain = true;
    }

    pub fn render(&mut self, context: &Context) {
        // On Windows, this can occur from minimizing the application.
        let surface = context.surface();
        let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();
        let dimensions = window.inner_size();
        if dimensions.width == 0 || dimensions.height == 0 {
            return;
        }

        // It is important to call this function from time to time, otherwise resources will keep
        // accumulating and you will eventually reach an out of memory error.
        // Calling this function polls various fences in order to determine what the GPU has
        // already processed, and frees the resources that are no longer needed.
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        // Whenever the window resizes we need to recreate everything dependent on the window size.
        // In this example that includes the swapchain, the framebuffers and the dynamic state viewport.
        if self.recreate_swapchain {
            // Use the new dimensions of the window.

            let (new_swapchain, new_images) = match self.swapchain.recreate(SwapchainCreateInfo {
                image_extent: dimensions.into(),
                ..self.swapchain.create_info()
            }) {
                Ok(r) => r,
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                Err(e) => panic!("Failed to recreate swapchain: {e:?}"),
            };

            self.swapchain = new_swapchain;

            let dimensions = new_images[0].dimensions().width_height().map(|i| i as f32);
            self.viewport.dimensions = dimensions;

            self.images = new_images
                .into_iter()
                .map(|image| ImageView::new_default(image.clone()).unwrap())
                .collect::<Vec<_>>();

            // TODO: delegate task to fetch new framebuffers to subrendersystems

            self.scene_renderer.resize(&self.images);

            self.recreate_swapchain = false;
        }

        // Before we can draw on the output, we have to *acquire* an image from the swapchain. If
        // no image is available (which happens if you submit draw commands too quickly), then the
        // function will block.
        // This operation returns the index of the image that we are allowed to draw upon.
        //
        // This function can block if no image is available. The parameter is an optional timeout
        // after which the function call will return an error.
        let (image_index, suboptimal, acquire_future) =
            match acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return;
                }
                Err(e) => panic!("Failed to acquire next image: {e:?}"),
            };

        // acquire_next_image can be successful, but suboptimal. This means that the swapchain image
        // will still work, but it may not display correctly. With some drivers this can be when
        // the window resizes, but it may not cause the swapchain to become out of date.
        if suboptimal {
            self.recreate_swapchain = true;
        }

        let future = self.previous_frame_end.take().unwrap().join(acquire_future);

        let future = self
            .scene_renderer
            .render(&context, future, image_index, &self.viewport);
        // TODO: record render things

        let future = future
            .then_swapchain_present(
                context.queue(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
            }
            Err(FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end = Some(sync::now(context.device().clone()).boxed());
            }
            Err(e) => {
                panic!("Failed to flush future: {e:?}");
                // previous_frame_end = Some(sync::now(device.clone()).boxed());
            }
        }
    }
}

pub fn create_swapchain(
    device: Arc<Device>,
    surface: Arc<Surface>,
) -> (Arc<Swapchain>, Vec<Arc<SwapchainImage>>) {
    let (swapchain, images) = {
        let surface_capabilities = device
            .physical_device()
            .surface_capabilities(&surface, SurfaceInfo::default())
            .expect("could not fetch surface capabilities");

        let image_format = Some(
            device
                .physical_device()
                .surface_formats(&surface, SurfaceInfo::default())
                .expect("could not fetch surface formats")
                .iter()
                .min_by_key(|(format, color)| {
                    // Prefer a RGB8/sRGB format
                    match (format, color) {
                        (Format::B8G8R8A8_SRGB, _) => 1,
                        (Format::R8G8B8A8_SRGB, ColorSpace::SrgbNonLinear) => 2,
                        (_, _) => 3,
                    }
                })
                .expect("could not fetch image format")
                .0, // just the format
        );

        let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();

        Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: surface_capabilities.min_image_count + 1,
                image_format,
                image_extent: window.inner_size().into(),
                image_usage: ImageUsage {
                    color_attachment: true,
                    // TODO: depth attachment needed?
                    ..Default::default()
                },
                composite_alpha: surface_capabilities
                    .supported_composite_alpha
                    .iter()
                    .next()
                    .expect("could not fetch supported composite alpha"),
                ..Default::default()
            },
        )
        .expect("failed to create swapchain")
    };
    (swapchain, images)
}
