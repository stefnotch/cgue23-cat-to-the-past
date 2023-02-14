use std::sync::Arc;
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::device::physical::PhysicalDevice;
use vulkano::instance::Instance;
use vulkano::swapchain::Surface;
use vulkano_win::VkSurfaceBuild;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use crate::vulkan_graphics::{create_instance, create_logical_device, find_physical_device};

pub struct Context {
    instance: Arc<Instance>,
    surface: Arc<Surface>,
    physical_device: Arc<PhysicalDevice>,
    queue_family_index: u32,
    device: Arc<Device>,
    queue: Arc<Queue>,
}

impl Context {
    pub fn new(window_builder: WindowBuilder, event_loop: &EventLoop<()>) -> Context {
        let instance = create_instance();

        let surface = window_builder
            .build_vk_surface(&event_loop, instance.clone())
            .expect("could not create window");

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) = find_physical_device(
            instance.clone(),
            surface.clone(),
            &device_extensions,
        );

        let (device, queue) = create_logical_device(
            physical_device.clone(),
            queue_family_index,
            &device_extensions,
        );

        Context {
            instance,
            surface,
            physical_device,
            queue_family_index,
            device,
            queue,
        }
    }

    pub fn surface(&self) -> Arc<Surface> {
        self.surface.clone()
    }

    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
    }

    pub fn queue(&self) -> Arc<Queue> {
        self.queue.clone()
    }
}