use vulkano::VulkanLibrary;
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano_win::VkSurfaceBuild;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

fn main() {
    let library = VulkanLibrary::new()
        .expect("no local Vulkan library/DLL");

    let instance_extensions = vulkano_win::required_extensions(&library);

    let instance = Instance::new(library, InstanceCreateInfo {
        enabled_extensions: instance_extensions,
        ..Default::default()
    }).expect("failed to create instance");

    let event_loop = EventLoop::new();
    let _surface = WindowBuilder::new()
        .build_vk_surface(&event_loop, instance.clone())
        .expect("could not create window");

    event_loop.run(|event, _, control_flow| {
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit
            }
            _ => ()
        }
    });

}