use crate::bloom_renderer::BloomRenderer;
use crate::context::Context;
use crate::create_gpu_models;
use crate::model_uploader::{
    create_ui_component, update_gpu_models, ModelUploaderAllocator, SamplerInfoMap,
};
use crate::quad_renderer::QuadRenderer;
use crate::scene::material::Material;
use crate::scene::mesh::Mesh;
use crate::scene::model::GpuModel;
use crate::scene::texture::Texture;
use crate::scene_renderer::SceneRenderer;
use crate::shadow_renderer::ShadowRenderer;
use app::plugin::{Plugin, PluginAppAccess};
use bevy_ecs::prelude::{Local, Resource};
use bevy_ecs::query::With;
use bevy_ecs::schedule::{IntoSystemConfig, SystemSet};
use bevy_ecs::system::{NonSend, NonSendMut, Query, Res};
use levels::current_level::CurrentLevel;
use levels::level_id::LevelId;
use scene::asset::Assets;
use scene::camera::Camera;
use scene::light::{CastsShadow, Light, LightCastShadow};
use scene::transform::Transform;
use scene::ui_component::UIComponent;
use std::sync::Arc;
use time::time_manager::TimeManager;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{ImageAccess, ImageUsage, SwapchainImage};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::swapchain::{
    acquire_next_image, AcquireError, ColorSpace, PresentMode, Surface, SurfaceInfo, Swapchain,
    SwapchainCreateInfo, SwapchainCreationError, SwapchainPresentInfo,
};
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};
use windowing::window::WindowManager;

use crate::scene::ui_component::GpuUIComponent;
use crate::ui_renderer::UIRenderer;
use windowing::window::Window;

#[derive(Resource)]
pub struct ViewFrustumCullingMode {
    pub enabled: bool,
}

/// Responsible for keeping the swapchain up-to-date and calling the sub-rendersystems
pub struct Renderer {
    recreate_swapchain: bool,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    swapchain: SwapchainContainer,
    shadow_renderer: ShadowRenderer,
    scene_renderer: SceneRenderer,
    bloom_renderer: BloomRenderer,
    quad_renderer: QuadRenderer,
    ui_renderer: UIRenderer,
    viewport: Viewport,
}

struct SwapchainContainer {
    swapchain: Arc<Swapchain>,
    images: Vec<Arc<ImageView<SwapchainImage>>>,
    dimensions: [u32; 2],
}

impl Renderer {
    pub fn new(context: &Context, brightness: f32) -> Renderer {
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

        let shadow_renderer = ShadowRenderer::new(
            context,
            swapchain_image_count,
            memory_allocator.clone(),
            command_buffer_allocator.clone(),
            descriptor_set_allocator.clone(),
        );

        let scene_renderer = SceneRenderer::new(
            context,
            shadow_renderer.get_shadow_cube_maps(),
            dimensions,
            swapchain_image_count,
            memory_allocator.clone(),
            command_buffer_allocator.clone(),
            descriptor_set_allocator.clone(),
        );

        let bloom_renderer = BloomRenderer::new(
            context,
            scene_renderer.output_images().clone(),
            memory_allocator.clone(),
            command_buffer_allocator.clone(),
            descriptor_set_allocator.clone(),
        );

        let quad_renderer = QuadRenderer::new(
            context,
            &bloom_renderer.output_images(),
            &swapchain.images,
            swapchain.swapchain.image_format(),
            memory_allocator.clone(),
            command_buffer_allocator.clone(),
            descriptor_set_allocator.clone(),
            brightness,
        );

        let ui_renderer = UIRenderer::new(
            context,
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
            shadow_renderer,
            scene_renderer,
            bloom_renderer,
            quad_renderer,
            ui_renderer,
            viewport,
        }
    }

    pub fn recreate_swapchain(&mut self) {
        self.recreate_swapchain = true;
    }
}

#[derive(SystemSet, Clone, PartialEq, Eq, Hash, Debug)]
pub enum RendererPluginSets {
    Render,
}

pub struct RendererPlugin {
    brightness: f32,
}

impl RendererPlugin {
    pub fn new(brightness: f32) -> Self {
        Self { brightness }
    }
}

impl Plugin for RendererPlugin {
    fn build(&mut self, app: &mut PluginAppAccess) {
        let context = Context::new(
            app.world_hack_access()
                .get_resource::<WindowManager>()
                .unwrap()
                .window
                .clone(),
        );
        let renderer = Renderer::new(&context, self.brightness);
        let model_uploading_allocator = ModelUploaderAllocator::new(context.device());
        let sampler_info_map = SamplerInfoMap::new();

        app //
            .with_non_send_resource(context)
            .with_non_send_resource(renderer)
            .with_system(
                create_gpu_models
                    .in_set(RendererPluginSets::Render)
                    .before(render),
            )
            .with_system(
                update_gpu_models
                    .in_set(RendererPluginSets::Render)
                    .after(create_gpu_models)
                    .before(render),
            )
            .with_system(
                create_ui_component
                    .in_set(RendererPluginSets::Render)
                    .after(update_gpu_models)
                    .before(render),
            )
            .with_system(render.in_set(RendererPluginSets::Render))
            .with_resource(ViewFrustumCullingMode { enabled: true })
            .with_resource(model_uploading_allocator)
            .with_resource(sampler_info_map)
            .with_resource(Assets::<Mesh>::default())
            .with_resource(Assets::<Material>::default())
            .with_resource(Assets::<Texture>::default());
    }
}

pub fn render(
    mut renderer: NonSendMut<Renderer>,
    context: NonSend<Context>,
    camera: Res<Camera>,
    time_manager: Res<TimeManager>,
    current_level: Res<CurrentLevel>,
    query_models: Query<(&Transform, &GpuModel)>,
    query_lights: Query<(&Transform, &Light, &LevelId)>,
    query_shadow_light: Query<(&Transform, &LevelId), (With<LightCastShadow>, With<Light>)>,
    query_shadow_casting_models: Query<(&Transform, &GpuModel, &LevelId), With<CastsShadow>>,
    mut frame_counter: Local<u64>,
    query_ui_components: Query<(&GpuUIComponent, &UIComponent)>,
    view_frustum_culling_mode: Res<ViewFrustumCullingMode>,
    mut rewind_start_time: Local<f32>,
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

        let _dimensions = renderer.swapchain.swapchain.image_extent();
        let _swapchain_image_count = renderer.swapchain.swapchain.image_count();

        // https://doc.rust-lang.org/nomicon/borrow-splitting.html
        let renderer = renderer.as_mut();
        renderer
            .shadow_renderer
            .resize(renderer.swapchain.images.len() as u32);
        renderer.scene_renderer.resize(
            &renderer.swapchain.images,
            renderer.shadow_renderer.get_shadow_cube_maps(),
        );
        renderer
            .bloom_renderer
            .resize(renderer.scene_renderer.output_images().clone());

        renderer.quad_renderer.resize(
            &renderer.swapchain.images,
            &renderer.bloom_renderer.output_images(),
        );

        renderer.ui_renderer.resize(&renderer.swapchain.images);

        renderer.recreate_swapchain = false;

        *frame_counter = 0;
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

    let current_level_id = current_level.level_id;
    let models = query_models.iter().collect();
    let lights = query_lights
        .iter()
        .filter(|(_, _, level_id)| level_id == &&current_level_id)
        .map(|(transform, light, _)| (transform, light))
        .collect();
    let ui_components = query_ui_components.iter().collect();
    let shadow_cast_models = query_shadow_casting_models
        .iter()
        .filter(|(_, _, level_id)| level_id == &&current_level_id)
        .map(|(transform, gpu_model, _)| (transform, gpu_model))
        .collect();

    let nearest_shadow_light = query_shadow_light
        .iter()
        .filter(|(_, level_id)| level_id == &&current_level_id)
        .map(|(transform, _)| transform)
        .min_by(|transform_a, transform_b| {
            let distance_a = (camera.position - transform_a.position).norm_squared();
            let distance_b = (camera.position - transform_b.position).norm_squared();
            distance_a.total_cmp(&distance_b)
        });

    let rewind_time = if time_manager.is_rewinding() {
        if time_manager.level_delta_time().duration().is_zero() {
            // if we cannot rewind anymore
            0.0
        } else {
            *rewind_start_time - time_manager.level_time().as_secs_f32()
        }
    } else {
        *rewind_start_time = time_manager.level_time().as_secs_f32();
        0.0
    };

    let future = if let (Some(nearest_shadow_light), true) = (
        nearest_shadow_light,
        *frame_counter > renderer.swapchain.images.len() as u64,
    ) {
        renderer
            .shadow_renderer
            .render(
                &context,
                rewind_time,
                &shadow_cast_models,
                nearest_shadow_light,
                camera.as_ref(),
                future,
                image_index,
            )
            .boxed()
    } else {
        future.boxed()
    };

    let future = renderer.scene_renderer.render(
        &context,
        camera.as_ref(),
        rewind_time,
        models,
        lights,
        future,
        nearest_shadow_light,
        view_frustum_culling_mode.as_ref(),
        image_index,
        *frame_counter,
        &renderer.viewport,
    );

    let future = renderer
        .bloom_renderer
        .render(&context, future, image_index);

    let future = renderer
        .quad_renderer
        .render(&context, future, image_index, &renderer.viewport);

    let future = if *frame_counter > renderer.swapchain.images.len() as u64 {
        renderer
            .ui_renderer
            .render(
                &context,
                ui_components,
                future,
                image_index,
                &renderer.viewport,
            )
            .boxed()
    } else {
        future.boxed()
    };

    let future = future
        .then_swapchain_present(
            context.queue(),
            SwapchainPresentInfo::swapchain_image_index(
                renderer.swapchain.swapchain.clone(),
                image_index,
            ),
        )
        .then_signal_fence_and_flush();

    *frame_counter += 1;
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
                    present_mode: PresentMode::Immediate,
                    min_image_count: surface_capabilities.min_image_count,
                    image_format,
                    image_extent: window.inner_size().into(),
                    image_usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::INPUT_ATTACHMENT,
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
            image_extent: dimensions,
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
