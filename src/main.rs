use std::default::Default;
use std::sync::Arc;
use cgmath::{Matrix4, Point3, Rad, SquareMatrix, Vector3};
use vulkano::device::{physical::PhysicalDeviceType, DeviceExtensions, Device, DeviceCreateInfo, QueueCreateInfo};
use vulkano::format::Format;
use vulkano::image::{ImageAccess, ImageUsage, SwapchainImage};
use vulkano::{sync, VulkanLibrary};
use vulkano::instance::{Instance, InstanceCreateInfo, InstanceExtensions};
use vulkano::memory::allocator::{MemoryUsage, StandardMemoryAllocator};
use vulkano::swapchain::{acquire_next_image, AcquireError, ColorSpace, Surface, SurfaceInfo, Swapchain, SwapchainCreateInfo, SwapchainCreationError, SwapchainPresentInfo};
use vulkano_win::VkSurfaceBuild;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, CpuBufferPool, TypedBufferAccess};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassContents};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::physical::PhysicalDevice;
use vulkano::image::view::ImageView;
use vulkano::instance::debug::{DebugUtilsMessageSeverity, DebugUtilsMessageType, DebugUtilsMessenger, DebugUtilsMessengerCreateInfo};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::sync::{FlushError, GpuFuture};
use crate::mesh::{Mesh, MeshVertex};

mod mesh;
mod camera;

fn main() {
    let library = VulkanLibrary::new()
        .expect("no local Vulkan library/DLL");

    let instance_extensions = InstanceExtensions {
        ext_debug_report: true,
        ..vulkano_win::required_extensions(&library)
    };

    // NOTE: To simplify the example code we won't verify these layer(s) are actually in the layers list:
    // see: https://github.com/vulkano-rs/vulkano/blob/85e9d1c24ec612023dbc5b13e6164706ea52e963/examples/src/bin/debug.rs#L64
    let instance = Instance::new(library, InstanceCreateInfo {
        enabled_extensions: instance_extensions,
        enabled_layers: vec![String::from("VK_LAYER_KHRONOS_validation")],
        ..Default::default()
    }).expect("failed to create instance");


    let _debug_callback = unsafe {
        DebugUtilsMessenger::new(
            instance.clone(),
            DebugUtilsMessengerCreateInfo {
                message_severity: DebugUtilsMessageSeverity {
                    error: true,
                    warning: true,
                    information: false,
                    verbose: false,
                    ..Default::default()
                },
                message_type: DebugUtilsMessageType {
                    general: true,
                    validation: true,
                    performance: true,
                    ..Default::default()
                },
                ..DebugUtilsMessengerCreateInfo::user_callback(Arc::new(|msg| {
                    let severity = if msg.severity.error {
                        "Error"
                    } else if msg.severity.warning {
                        "Warning"
                    } else if msg.severity.information {
                        "Information"
                    } else if msg.severity.verbose {
                        "Verbose"
                    } else {
                        "Unknown"
                    };

                    let ty = if msg.ty.general {
                        "General"
                    } else if msg.ty.validation {
                        "Validation"
                    } else if msg.ty.performance {
                        "Performance"
                    } else {
                        "Unknown"
                    };

                    println!(
                        "[VkDebug][Layer: {}][Severity: {}][Type: {}]: {}",
                        msg.layer_prefix.unwrap_or("Unknown"),
                        severity,
                        ty,
                        msg.description
                    );
                }))
            },
        )
            .ok()
    };


    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&event_loop, instance.clone())
        .expect("could not create window");

    let device_extensions = DeviceExtensions {
        khr_swapchain:true,
        ..DeviceExtensions::empty()
    };

    let (physical_device, queue_family_index) = find_physical_device(
        instance.clone(),
        surface.clone(),
        &device_extensions
    );

    println!(
        "Using device: {} (type: {:?})",
        physical_device.properties().device_name,
        physical_device.properties().device_type,
    );

    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            enabled_extensions: device_extensions,
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    ).expect("could not create logical device");

    let queue = queues.next().expect("could not fetch queue");

    let (mut swapchain, images) = {
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
                        (_, _) => 3
                    }
                }).expect("could not fetch image format")
                .0 // just the format
        );

        let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();

        Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: surface_capabilities.min_image_count +1,
                image_format,
                image_extent: window.inner_size().into(),
                image_usage: ImageUsage {
                    color_attachment: true,
                    ..Default::default()
                },
                composite_alpha: surface_capabilities.
                    supported_composite_alpha
                    .iter().next()
                    .expect("could not fetch supported composite alpha"),
                ..Default::default()
            }

        ).expect("failed to create swapchain")
    };

    mod vs {
        vulkano_shaders::shader! {
            ty: "vertex",
            path: "assets/shaders/vert.glsl",
            types_meta: {
                 use bytemuck::{Pod, Zeroable};
                #[derive(Clone, Copy, Zeroable, Pod)]
            }
        }
    }

    mod fs {
        vulkano_shaders::shader! {
            ty: "fragment",
            path: "assets/shaders/frag.glsl"
        }
    }

    let vs = vs::load(device.clone()).unwrap();
    let fs = fs::load(device.clone()).unwrap();

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

    let cube = Mesh::cube(0.5, 0.5, 0.5);



    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        &memory_allocator,
        BufferUsage {
            vertex_buffer: true,
            ..Default::default()
        },
        false,
        cube.vertices.iter().cloned()
    ).expect("could not upload vertex data to GPU");

    let index_buffer = CpuAccessibleBuffer::from_iter(
        &memory_allocator,
        BufferUsage {
            index_buffer: true,
            ..Default::default()
        },
        false,
        cube.indices.iter().cloned()
    ).expect("could not upload indices data to GPU");

    let uniform_buffer = CpuBufferPool::<vs::ty::Data>::new(
        memory_allocator.clone(),
        BufferUsage {
            uniform_buffer: true,
            ..Default::default()
        },
        MemoryUsage::Upload,
    );

    let render_pass = vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            // `color` is a custom name we give to the first and only attachment.
            color: {
                load: Clear,
                store: Store,
                format: swapchain.image_format(),
                samples: 1,
            }
        },
        pass: {
            // We use the attachment named `color` as the one and only color attachment.
            color: [color],
            // No depth-stencil attachment is indicated with empty brackets.
            depth_stencil: {}
        }
    )
        .unwrap();

    let pipeline = GraphicsPipeline::start()
        // We have to indicate which subpass of which render pass this pipeline is going to be used
        // in. The pipeline will only be usable from this particular subpass.
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        // We need to indicate the layout of the vertices.
        .vertex_input_state(BuffersDefinition::new().vertex::<MeshVertex>())
        // The content of the vertex buffer describes a list of triangles.
        .input_assembly_state(InputAssemblyState::new())
        // A Vulkan shader can in theory contain multiple entry points, so we have to specify
        // which one.
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        // Use a resizable viewport set to draw over the entire window
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        // See `vertex_shader`.
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        // Now that our builder is filled, we call `build()` to obtain an actual pipeline.
        .build(device.clone())
        .expect("could not create pipeline");

    let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone());

    // Dynamic viewports allow us to recreate just the viewport when the window is resized
    // Otherwise we would have to recreate the whole pipeline.
    let mut viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [0.0, 0.0],
        depth_range: 0.0..1.0,
    };

    // The render pass we created above only describes the layout of our framebuffers. Before we
    // can draw we also need to create the actual framebuffers.
    //
    // Since we need to draw to multiple images, we are going to create a different framebuffer for
    // each image.
    let mut framebuffers = window_size_dependent_setup(&images, render_pass.clone(), &mut viewport);

    // Before we can start creating and recording command buffers, we need a way of allocating
    // them. Vulkano provides a command buffer allocator, which manages raw Vulkan command pools
    // underneath and provides a safe interface for them.
    let command_buffer_allocator =
        StandardCommandBufferAllocator::new(device.clone(), Default::default());


    // In some situations, the swapchain will become invalid by itself. This includes for example
    // when the window is resized (as the images of the swapchain will no longer match the
    // window's) or, on Android, when the application went to the background and goes back to the
    // foreground.
    //
    // In this situation, acquiring a swapchain image or presenting it will return an error.
    // Rendering to an image of that swapchain will not produce any error, but may or may not work.
    // To continue rendering, we need to recreate the swapchain by creating a new swapchain.
    // Here, we remember that we need to do this for the next loop iteration.
    let mut recreate_swapchain = false;

    // In the loop below we are going to submit commands to the GPU. Submitting a command produces
    // an object that implements the `GpuFuture` trait, which holds the resources for as long as
    // they are in use by the GPU.
    //
    // Destroying the `GpuFuture` blocks until the GPU is finished executing it. In order to avoid
    // that, we store the submission of the previous frame here.
    let mut previous_frame_end = Some(sync::now(device.clone()).boxed());

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                recreate_swapchain = true;
            }
            Event::RedrawEventsCleared => {
                // Do not draw frame when screen dimensions are zero.
                // On Windows, this can occur from minimizing the application.
                let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();
                let dimensions = window.inner_size();
                if dimensions.width == 0 || dimensions.height == 0 {
                    return;
                }

                // It is important to call this function from time to time, otherwise resources will keep
                // accumulating and you will eventually reach an out of memory error.
                // Calling this function polls various fences in order to determine what the GPU has
                // already processed, and frees the resources that are no longer needed.
                previous_frame_end.as_mut().unwrap().cleanup_finished();

                // Whenever the window resizes we need to recreate everything dependent on the window size.
                // In this example that includes the swapchain, the framebuffers and the dynamic state viewport.
                if recreate_swapchain {
                    // Use the new dimensions of the window.

                    let (new_swapchain, new_images) =
                        match swapchain.recreate(SwapchainCreateInfo {
                            image_extent: dimensions.into(),
                            ..swapchain.create_info()
                        }) {
                            Ok(r) => r,
                            // This error tends to happen when the user is manually resizing the window.
                            // Simply restarting the loop is the easiest way to fix this issue.
                            Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                            Err(e) => panic!("Failed to recreate swapchain: {e:?}"),
                        };

                    swapchain = new_swapchain;
                    // Because framebuffers contains an Arc on the old swapchain, we need to
                    // recreate framebuffers as well.
                    framebuffers = window_size_dependent_setup(
                        &new_images,
                        render_pass.clone(),
                        &mut viewport,
                    );
                    recreate_swapchain = false;
                }

                let aspect_ratio = swapchain.image_extent()[0] as f32 / swapchain.image_extent()[1] as f32;

                let proj = cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2), aspect_ratio, 0.01, 100.0);

                let view: Matrix4<f32> = Matrix4::look_at_rh(
                    Point3::new(0.5,0.5,-1.0),
                    Point3::new(0.0,0.0,0.0),
                    Vector3::new(0.0,-1.0,0.0),
                );

                let world: Matrix4<f32> = Matrix4::identity();

                let uniform_data = vs::ty::Data {
                    world: world.into(),
                    view: view.into(),
                    proj: proj.into()
                };

                let uniform_buffer_subbuffer = uniform_buffer.from_data(uniform_data).unwrap();

                let set = PersistentDescriptorSet::new(
                    &descriptor_set_allocator,
                    pipeline.layout().set_layouts().get(0).unwrap().clone(),
                    [WriteDescriptorSet::buffer(0, uniform_buffer_subbuffer)],
                ).expect("could not create descriptor set");

                // Before we can draw on the output, we have to *acquire* an image from the swapchain. If
                // no image is available (which happens if you submit draw commands too quickly), then the
                // function will block.
                // This operation returns the index of the image that we are allowed to draw upon.
                //
                // This function can block if no image is available. The parameter is an optional timeout
                // after which the function call will return an error.
                let (image_index, suboptimal, acquire_future) =
                    match acquire_next_image(swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("Failed to acquire next image: {e:?}"),
                    };

                // acquire_next_image can be successful, but suboptimal. This means that the swapchain image
                // will still work, but it may not display correctly. With some drivers this can be when
                // the window resizes, but it may not cause the swapchain to become out of date.
                if suboptimal {
                    recreate_swapchain = true;
                }

                // In order to draw, we have to build a *command buffer*. The command buffer object holds
                // the list of commands that are going to be executed.
                //
                // Building a command buffer is an expensive operation (usually a few hundred
                // microseconds), but it is known to be a hot path in the driver and is expected to be
                // optimized.
                //
                // Note that we have to pass a queue family when we create the command buffer. The command
                // buffer will only be executable on that given queue family.
                let mut builder = AutoCommandBufferBuilder::primary(
                    &command_buffer_allocator,
                    queue.queue_family_index(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                    .unwrap();

                builder
                    // Before we can draw, we have to *enter a render pass*.
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            // A list of values to clear the attachments with. This list contains
                            // one item for each attachment in the render pass. In this case,
                            // there is only one attachment, and we clear it with a blue color.
                            //
                            // Only attachments that have `LoadOp::Clear` are provided with clear
                            // values, any others should use `ClearValue::None` as the clear value.
                            clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
                            ..RenderPassBeginInfo::framebuffer(
                                framebuffers[image_index as usize].clone(),
                            )
                        },
                        // The contents of the first (and only) subpass. This can be either
                        // `Inline` or `SecondaryCommandBuffers`. The latter is a bit more advanced
                        // and is not covered here.
                        SubpassContents::Inline,
                    )
                    .unwrap()
                    // We are now inside the first subpass of the render pass. We add a draw command.
                    //
                    // The last two parameters contain the list of resources to pass to the shaders.
                    // Since we used an `EmptyPipeline` object, the objects have to be `()`.
                    .set_viewport(0, [viewport.clone()])
                    .bind_pipeline_graphics(pipeline.clone())
                    .bind_index_buffer(index_buffer.clone())
                    .bind_vertex_buffers(0, vertex_buffer.clone())
                    .bind_descriptor_sets(PipelineBindPoint::Graphics, pipeline.layout().clone(), 0, set.clone())
                    .draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0)
                    .unwrap()
                    // We leave the render pass. Note that if we had multiple
                    // subpasses we could have called `next_subpass` to jump to the next subpass.
                    .end_render_pass()
                    .unwrap();

                // Finish building the command buffer by calling `build`.
                let command_buffer = builder.build().unwrap();

                let future = previous_frame_end
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(queue.clone(), command_buffer)
                    .unwrap()
                    // The color output is now expected to contain our triangle. But in order to show it on
                    // the screen, we have to *present* the image by calling `present`.
                    //
                    // This function does not actually present the image immediately. Instead it submits a
                    // present command at the end of the queue. This means that it will only be presented once
                    // the GPU has finished executing the command buffer that draws the triangle.
                    .then_swapchain_present(
                        queue.clone(),
                        SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_index),
                    )
                    .then_signal_fence_and_flush();

                match future {
                    Ok(future) => {
                        previous_frame_end = Some(future.boxed());
                    }
                    Err(FlushError::OutOfDate) => {
                        recreate_swapchain = true;
                        previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }
                    Err(e) => {
                        panic!("Failed to flush future: {e:?}");
                        // previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }
                }
            }
            _ => (),
        }
    });
}

fn find_physical_device(instance: Arc<Instance>, surface: Arc<Surface>, device_extensions: &DeviceExtensions) -> (Arc<PhysicalDevice>, u32) {
    instance
        .enumerate_physical_devices()
        .expect("could not enumerate physical devices")
        .filter(|p| {
            // check if device extensions are supported
            p.supported_extensions().contains(&device_extensions)
        })
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    // check for graphics flag in queue family
                    q.queue_flags.graphics &&
                        p.surface_support(i as u32, &surface).unwrap_or(false)
                })
                .map(|i| (p, i as u32))
        })
        .min_by_key(|(p, _)| {
            // prefer discrete gpus
            match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5
            }
        })
        .expect("No suitable physical device found")
}

/// This method is called once during initialization, then again whenever the window is resized
fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage>],
    render_pass: Arc<RenderPass>,
    viewport: &mut Viewport,
) -> Vec<Arc<Framebuffer>> {
    let dimensions = images[0].dimensions().width_height();
    viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];

    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            )
                .unwrap()
        })
        .collect::<Vec<_>>()
}