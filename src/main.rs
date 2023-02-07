use std::default::Default;
use vulkano::device::{physical::PhysicalDeviceType, DeviceExtensions, Device, DeviceCreateInfo, QueueCreateInfo};
use vulkano::format::Format;
use vulkano::image::ImageUsage;
use vulkano::VulkanLibrary;
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::swapchain::{ColorSpace, SurfaceInfo, Swapchain, SwapchainCreateInfo};
use vulkano_win::VkSurfaceBuild;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

fn main() {
    let library = VulkanLibrary::new()
        .expect("no local Vulkan library/DLL");

    let instance_extensions = vulkano_win::required_extensions(&library);

    let instance = Instance::new(library, InstanceCreateInfo {
        enabled_extensions: instance_extensions,
        ..Default::default()
    }).expect("failed to create instance");

    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&event_loop, instance.clone())
        .expect("could not create window");

    let device_extensions = DeviceExtensions {
        khr_swapchain:true,
        ..DeviceExtensions::empty()
    };

    let (physical_device, queue_family_index) = instance
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
        .expect("No suitable physical device found");

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

    let _queue = queues.next().expect("could not fetch queue");

    let (mut _swapchain, _images) = {
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

        ).expect("failed to create _swapchain")
    };

    event_loop.run(|event, _, control_flow| {
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit
            }
            _ => ()
        }
    });
}