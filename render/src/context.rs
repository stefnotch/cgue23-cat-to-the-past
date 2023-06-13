use std::sync::Arc;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo, QueueFlags,
};
use vulkano::instance::debug::{
    DebugUtilsMessageSeverity, DebugUtilsMessageType, DebugUtilsMessenger,
    DebugUtilsMessengerCreateInfo,
};
use vulkano::instance::{Instance, InstanceCreateInfo, InstanceExtensions};
use vulkano::swapchain::Surface;
use vulkano::{Version, VulkanLibrary};
use vulkano_win::create_surface_from_handle;

use windowing::window::Window;

///
/// see also https://gpuopen.com/learn/understanding-vulkan-objects/
pub struct Context {
    _instance: Arc<Instance>,
    /// we need to keep a reference to the debug callback, otherwise it will be dropped
    _debug_callback: Option<DebugUtilsMessenger>,
    surface: Arc<Surface>,
    _physical_device: Arc<PhysicalDevice>,
    device: Arc<Device>,
    queue_family_index: u32,
    graphics_queue: Arc<Queue>,
}

impl Context {
    pub fn new(window: Arc<Window>) -> Context {
        let (instance, debug_callback) = create_instance();

        // Consume the WindowBuilder, build it, and get the surface
        let surface =
            create_surface_from_handle(window, instance.clone()).expect("could not create window");

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
            _instance: instance,
            _debug_callback: debug_callback,
            surface,
            _physical_device: physical_device,
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

    pub fn window(&self) -> Arc<Window> {
        self.surface
            .object()
            .unwrap()
            .clone()
            .downcast::<Window>()
            .unwrap()
    }
}

fn create_instance() -> (Arc<Instance>, Option<DebugUtilsMessenger>) {
    let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");

    // calls vkEnumerateInstanceExtensionProperties under the hood https://docs.rs/vulkano/0.32.3/src/vulkano/library.rs.html#155
    // see also https://www.lunarg.com/wp-content/uploads/2018/05/Vulkan-Debug-Utils_05_18_v1.pdf
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
        ..required_extensions(&library)
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
            max_api_version: Some(Version::major_minor(1, 3)),
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
    unsafe {
        DebugUtilsMessenger::new(
            instance.clone(),
            DebugUtilsMessengerCreateInfo {
                message_severity: DebugUtilsMessageSeverity::ERROR
                    | DebugUtilsMessageSeverity::WARNING
                    | DebugUtilsMessageSeverity::INFO
                    | DebugUtilsMessageSeverity::VERBOSE,
                message_type: DebugUtilsMessageType::GENERAL
                    | DebugUtilsMessageType::VALIDATION
                    | DebugUtilsMessageType::PERFORMANCE,
                ..DebugUtilsMessengerCreateInfo::user_callback(Arc::new(|msg| {
                    let severity = if msg.severity.intersects(DebugUtilsMessageSeverity::ERROR) {
                        "error"
                    } else if msg.severity.intersects(DebugUtilsMessageSeverity::WARNING) {
                        "warning"
                    } else if msg.severity.intersects(DebugUtilsMessageSeverity::INFO) {
                        "information"
                    } else if msg.severity.intersects(DebugUtilsMessageSeverity::VERBOSE) {
                        "verbose"
                    } else {
                        panic!("no-impl");
                    };

                    let ty = if msg.ty.intersects(DebugUtilsMessageType::GENERAL) {
                        "general"
                    } else if msg.ty.intersects(DebugUtilsMessageType::VALIDATION) {
                        "validation"
                    } else if msg.ty.intersects(DebugUtilsMessageType::PERFORMANCE) {
                        "performance"
                    } else {
                        panic!("no-impl");
                    };

                    if msg.severity.intersects(DebugUtilsMessageSeverity::VERBOSE)
                        || msg.severity.intersects(DebugUtilsMessageSeverity::INFO)
                    {
                        return;
                    }
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
    }
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
            p.supported_extensions().contains(device_extensions)
        })
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    // check for graphics flag in queue family
                    q.queue_flags.intersects(QueueFlags::GRAPHICS)
                        && p.surface_support(i as u32, &surface).unwrap_or(false)
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
            enabled_features: Features {
                fill_mode_non_solid: true,
                ..Default::default()
            },
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

/// Source https://github.com/vulkano-rs/vulkano/blob/bb7990fd491bed13746c8b85408097b5f0799c50/vulkano-win/src/winit.rs#L17
pub fn required_extensions(library: &VulkanLibrary) -> InstanceExtensions {
    let ideal = InstanceExtensions {
        khr_surface: true,
        khr_xlib_surface: true,
        khr_xcb_surface: true,
        khr_wayland_surface: true,
        khr_android_surface: true,
        khr_win32_surface: true,
        mvk_ios_surface: true,
        mvk_macos_surface: true,
        khr_get_physical_device_properties2: true,
        khr_get_surface_capabilities2: true,
        ..InstanceExtensions::empty()
    };

    library.supported_extensions().intersection(&ideal)
}
