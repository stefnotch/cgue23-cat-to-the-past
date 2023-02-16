use std::sync::Arc;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo};
use vulkano::instance::debug::{
    DebugUtilsMessageSeverity, DebugUtilsMessageType, DebugUtilsMessenger,
    DebugUtilsMessengerCreateInfo,
};
use vulkano::instance::{Instance, InstanceCreateInfo, InstanceExtensions};
use vulkano::swapchain::Surface;
use vulkano::VulkanLibrary;
use vulkano_win::VkSurfaceBuild;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

///
/// see also https://gpuopen.com/learn/understanding-vulkan-objects/
pub struct Context {
    instance: Arc<Instance>,
    debug_callback: Option<DebugUtilsMessenger>,
    surface: Arc<Surface>,
    physical_device: Arc<PhysicalDevice>,
    device: Arc<Device>,
    queue_family_index: u32,
    graphics_queue: Arc<Queue>,
}

impl Context {
    pub fn new(window_builder: WindowBuilder, event_loop: &EventLoop<()>) -> Context {
        let (instance, debug_callback) = create_instance();

        let surface = window_builder
            .build_vk_surface(&event_loop, instance.clone())
            .expect("could not create window");

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) =
            find_physical_device(instance.clone(), surface.clone(), &device_extensions);

        let (device, graphics_queue) = create_logical_device(
            physical_device.clone(),
            queue_family_index,
            &device_extensions,
        );

        Context {
            instance,
            debug_callback,
            surface,
            physical_device,
            queue_family_index,
            device,
            graphics_queue,
        }
    }

    pub fn surface(&self) -> Arc<Surface> {
        self.surface.clone()
    }

    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
    }

    pub fn queue(&self) -> Arc<Queue> {
        self.graphics_queue.clone()
    }

    pub fn queue_family_index(&self) -> u32 {
        self.queue_family_index
    }
}

fn create_instance() -> (Arc<Instance>, Option<DebugUtilsMessenger>) {
    let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");

    // calls vkEnumerateInstanceExtensionProperties under the hood https://docs.rs/vulkano/0.32.3/src/vulkano/library.rs.html#155
    let supported_extensions = library.supported_extensions();
    let suported_layers: Vec<_> = library
        .layer_properties()
        .expect("could not enumerate layers")
        .collect();

    // enable debugging if available
    let debug_extension_name = String::from("VK_LAYER_KHRONOS_validation");
    let debug_enabled = supported_extensions.ext_debug_utils
        && suported_layers
            .iter()
            .any(|l| l.name() == debug_extension_name);

    let instance_extensions = InstanceExtensions {
        ext_debug_utils: debug_enabled,
        ..vulkano_win::required_extensions(&library)
    };

    let mut layers = vec![];
    if debug_enabled {
        layers.push(debug_extension_name);
    }

    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: instance_extensions,
            enabled_layers: layers,
            ..Default::default()
        },
    )
    .expect("failed to create instance");

    // the debug callback should stay alive as long as the instance
    // otherwise the callback will be dropped and no longer print any messages
    let debug_callback = if debug_enabled {
        create_debug_callback(instance.clone())
    } else {
        None
    };
    (instance, debug_callback)
}

fn create_debug_callback(instance: Arc<Instance>) -> Option<DebugUtilsMessenger> {
    let debug_callback = unsafe {
        DebugUtilsMessenger::new(
            instance.clone(),
            DebugUtilsMessengerCreateInfo {
                message_severity: DebugUtilsMessageSeverity {
                    error: true,
                    warning: true,
                    information: true,
                    verbose: true,
                    ..DebugUtilsMessageSeverity::empty()
                },
                message_type: DebugUtilsMessageType {
                    general: true,
                    validation: true,
                    performance: true,
                    ..DebugUtilsMessageType::empty()
                },
                ..DebugUtilsMessengerCreateInfo::user_callback(Arc::new(|msg| {
                    let severity = if msg.severity.error {
                        "error"
                    } else if msg.severity.warning {
                        "warning"
                    } else if msg.severity.information {
                        "information"
                    } else if msg.severity.verbose {
                        "verbose"
                    } else {
                        panic!("no-impl");
                    };

                    let ty = if msg.ty.general {
                        "general"
                    } else if msg.ty.validation {
                        "validation"
                    } else if msg.ty.performance {
                        "performance"
                    } else {
                        panic!("no-impl");
                    };

                    println!(
                        "{} {} {}: {}",
                        msg.layer_prefix.unwrap_or("unknown"),
                        ty,
                        severity,
                        msg.description
                    );
                }))
            },
        )
        .ok()
    };

    debug_callback
}

fn find_physical_device(
    instance: Arc<Instance>,
    surface: Arc<Surface>,
    device_extensions: &DeviceExtensions,
) -> (Arc<PhysicalDevice>, u32) {
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
                    q.queue_flags.graphics && p.surface_support(i as u32, &surface).unwrap_or(false)
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
                _ => 5,
            }
        })
        .expect("No suitable physical device found")
}

fn create_logical_device(
    physical_device: Arc<PhysicalDevice>,
    queue_family_index: u32,
    device_extensions: &DeviceExtensions,
) -> (Arc<Device>, Arc<Queue>) {
    let (device, mut queues) = Device::new(
        physical_device.clone(),
        DeviceCreateInfo {
            enabled_extensions: *device_extensions,
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .expect("could not create logical device");

    let graphics_queue = queues.next().expect("could not fetch queue");

    (device, graphics_queue)
}
