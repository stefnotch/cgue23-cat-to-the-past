use crate::core::camera::Camera;
use crate::render::context::Context;
use crate::render::quad_renderer::QuadRenderer;
use crate::render::scene_renderer::SceneRenderer;
use crate::scene::light::Light;
use crate::scene::model::Model;
use crate::scene::transform::Transform;
use bevy_ecs::system::{NonSendMut, Query, Res};
use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{ImageAccess, ImageUsage, SwapchainImage};
use vulkano::memory::allocator::StandardMemoryAllocator;
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
    // TODO: Huh, this doesn't need to be an option?
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    swapchain: SwapchainContainer,
    scene_renderer: SceneRenderer,
    quad_renderer: QuadRenderer,
    viewport: Viewport,
}

struct SwapchainContainer {
    swapchain: Arc<Swapchain>,
    images: Vec<Arc<ImageView<SwapchainImage>>>,
    dimensions: [u32; 2],
}

impl Renderer {
    pub fn new(context: &Context) -> Renderer {
        let previous_frame_end = Some(sync::now(context.device()).boxed());

        let swapchain = SwapchainContainer::new(context.device(), context.surface());

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: swapchain.dimensions.map(|i| i as f32),
            depth_range: 0.0..1.0,
        };

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            context.device(),
            Default::default(),
        ));

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(context.device()));

        let descriptor_set_allocator =
            Arc::new(StandardDescriptorSetAllocator::new(context.device()));

        let dimensions = swapchain.swapchain.image_extent();
        let swapchain_image_count = swapchain.swapchain.image_count();

        let scene_renderer = SceneRenderer::new(
            context,
            dimensions,
            swapchain_image_count,
            memory_allocator.clone(),
            command_buffer_allocator.clone(),
            descriptor_set_allocator.clone(),
        );

        let quad_renderer = QuadRenderer::new(
            context,
            scene_renderer.output_images(),
            &swapchain.images,
            swapchain.swapchain.image_format(),
            memory_allocator.clone(),
            command_buffer_allocator.clone(),
            descriptor_set_allocator.clone(),
        );

        Renderer {
            recreate_swapchain: false,
            previous_frame_end,
            swapchain,
            scene_renderer,
            quad_renderer,
            viewport,
        }
    }

    pub fn recreate_swapchain(&mut self) {
        self.recreate_swapchain = true;
    }
}

pub fn render(
    mut renderer: NonSendMut<Renderer>,
    context: Res<Context>,
    camera: Res<Camera>,
    query_models: Query<(&Transform, &Model)>,
    query_lights: Query<(&Transform, &Light)>, // TODO: only query changed lights
) {
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
    renderer
        .previous_frame_end
        .as_mut()
        .unwrap()
        .cleanup_finished();

    // Whenever the window resizes we need to recreate everything dependent on the window size.
    // In this example that includes the swapchain, the framebuffers and the dynamic state viewport.
    if renderer.recreate_swapchain {
        // Use the new dimensions of the window.
        match renderer.swapchain.recreate(dimensions.into()) {
            Ok(r) => r,
            // This error tends to happen when the user is manually resizing the window.
            // Simply restarting the loop is the easiest way to fix this issue.
            Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => {
                println!("ImageExtentNotSupported");
                return;
            }
            Err(e) => panic!("Failed to recreate swapchain: {e:?}"),
        }

        renderer.viewport.dimensions = renderer.swapchain.dimensions.map(|i| i as f32);

        // https://doc.rust-lang.org/nomicon/borrow-splitting.html
        let renderer = renderer.as_mut();
        renderer.scene_renderer.resize(&renderer.swapchain.images);
        renderer.quad_renderer.resize(
            &renderer.swapchain.images,
            renderer.scene_renderer.output_images(),
        );

        renderer.recreate_swapchain = false;
    }

    // Before we can draw on the output, we have to *acquire* an image from the swapchain. If
    // no image is available (which happens if you submit draw commands too quickly), then the
    // function will block.
    // This operation returns the index of the image that we are allowed to draw upon.
    //
    // This function can block if no image is available. The parameter is an optional timeout
    // after which the function call will return an error.
    let (image_index, suboptimal, acquire_future) =
        match acquire_next_image(renderer.swapchain.swapchain.clone(), None) {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => {
                renderer.recreate_swapchain = true;
                return;
            }
            Err(e) => panic!("Failed to acquire next image: {e:?}"),
        };

    // acquire_next_image can be successful, but suboptimal. This means that the swapchain image
    // will still work, but it may not display correctly. With some drivers this can be when
    // the window resizes, but it may not cause the swapchain to become out of date.
    if suboptimal {
        renderer.recreate_swapchain = true;
    }

    let future = renderer
        .previous_frame_end
        .take()
        .unwrap()
        .join(acquire_future);

    let models = query_models.iter().collect();
    let lights = query_lights.iter().collect();

    let future = renderer.scene_renderer.render(
        &context,
        camera.as_ref(),
        models,
        lights,
        future,
        image_index,
        &renderer.viewport,
    );

    let future = renderer
        .quad_renderer
        .render(&context, future, image_index, &renderer.viewport);

    let future = future
        .then_swapchain_present(
            context.queue(),
            SwapchainPresentInfo::swapchain_image_index(
                renderer.swapchain.swapchain.clone(),
                image_index,
            ),
        )
        .then_signal_fence_and_flush();

    match future {
        Ok(future) => {
            // NOTE: one solution to remove the massive input delay with fullscreen-mode enabled
            future.wait(None).unwrap();

            renderer.previous_frame_end = Some(future.boxed());
        }
        Err(FlushError::OutOfDate) => {
            renderer.recreate_swapchain = true;
            renderer.previous_frame_end = Some(sync::now(context.device().clone()).boxed());
        }
        Err(e) => {
            println!("Failed to flush future: {e:?}");
            renderer.previous_frame_end = Some(sync::now(context.device()).boxed());
        }
    }
}

impl SwapchainContainer {
    pub fn new(device: Arc<Device>, surface: Arc<Surface>) -> SwapchainContainer {
        let (swapchain, images) = {
            let surface_capabilities = device
                .physical_device()
                .surface_capabilities(&surface, SurfaceInfo::default())
                .expect("could not fetch surface capabilities");

            println!(
                "{:#?}",
                device
                    .physical_device()
                    .surface_formats(&surface, SurfaceInfo::default())
                    .expect("could not fetch surface formats")
            );

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
                    min_image_count: surface_capabilities.min_image_count,
                    image_format,
                    image_extent: window.inner_size().into(),
                    image_usage: ImageUsage::COLOR_ATTACHMENT,
                    composite_alpha: surface_capabilities
                        .supported_composite_alpha
                        .into_iter()
                        .next()
                        .expect("could not fetch supported composite alpha"),
                    ..Default::default()
                },
            )
            .expect("failed to create swapchain")
        };

        let dimensions = images[0].dimensions().width_height();

        let images = images
            .into_iter()
            .map(|image| ImageView::new_default(image.clone()).unwrap())
            .collect::<Vec<_>>();

        SwapchainContainer {
            swapchain,
            images,
            dimensions,
        }
    }

    fn recreate(&mut self, dimensions: [u32; 2]) -> Result<(), SwapchainCreationError> {
        match self.swapchain.recreate(SwapchainCreateInfo {
            image_extent: dimensions.into(),
            ..self.swapchain.create_info()
        }) {
            Ok((new_swapchain, new_images)) => {
                self.swapchain = new_swapchain;
                self.dimensions = new_images[0].dimensions().width_height();
                self.images = new_images
                    .into_iter()
                    .map(|image| ImageView::new_default(image.clone()).unwrap())
                    .collect::<Vec<_>>();
                Ok(())
            }
            Err(v) => Err(v),
        }
    }
}
