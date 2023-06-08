use crate::context::Context;
use crate::custom_storage_image::CustomStorageImage;
use crate::quad::{create_geometry_buffers, QuadVertex};
use crate::scene::ui_component::GpuUIComponent;
use nalgebra::Matrix4;
use scene::ui_component::UIComponent;
use std::sync::Arc;
use vulkano::buffer::Subbuffer;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferInheritanceInfo,
    CommandBufferUsage, RenderPassBeginInfo, SecondaryAutoCommandBuffer, SubpassContents,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::format::Format;
use vulkano::image::view::{ImageView, ImageViewCreateInfo};
use vulkano::image::{
    AttachmentImage, ImageAccess, ImageLayout, ImageUsage, ImageViewAbstract, SampleCount,
    SwapchainImage,
};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::graphics::depth_stencil::DepthStencilState;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::Vertex;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint};
use vulkano::render_pass;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode};
use vulkano::sync::GpuFuture;

pub struct OverlayRenderer {
    render_pass: Arc<RenderPass>,

    quad_pipeline: Arc<GraphicsPipeline>,
    ui_pipeline: Arc<GraphicsPipeline>,

    index_buffer: Subbuffer<[u32]>,
    vertex_buffer: Subbuffer<[QuadVertex]>,
    sampler: Arc<Sampler>,

    framebuffers: Vec<Arc<Framebuffer>>,
    quad_command_buffers: Vec<Arc<SecondaryAutoCommandBuffer>>,
    input_descriptor_sets: Vec<Arc<PersistentDescriptorSet>>,

    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl OverlayRenderer {
    pub fn new(
        context: &Context,
        output_images: &[Arc<ImageView<SwapchainImage>>],
        final_output_format: Format,
        memory_allocator: Arc<StandardMemoryAllocator>,
        command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
        brightness: f32,
    ) -> OverlayRenderer {
        let (vertex_buffer, index_buffer) = create_geometry_buffers(memory_allocator.clone());

        let render_pass = Self::create_renderpass(context, final_output_format);

        let quad_pipeline = {
            let vertex_shader = quad::vs::load(context.device()).unwrap();
            let fragment_shader = quad::fs::load(context.device()).unwrap();

            let spec_consts = quad::fs::SpecializationConstants { brightness };

            GraphicsPipeline::start()
                .vertex_input_state(QuadVertex::per_vertex())
                .vertex_shader(vertex_shader.entry_point("main").unwrap(), ())
                .input_assembly_state(InputAssemblyState::new())
                .fragment_shader(fragment_shader.entry_point("main").unwrap(), spec_consts)
                .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(context.device())
                .unwrap()
        };

        let ui_pipeline = {
            let vs = ui::vs::load(context.device()).unwrap();
            let fs = ui::fs::load(context.device()).unwrap();

            GraphicsPipeline::start()
                .vertex_input_state(QuadVertex::per_vertex())
                .vertex_shader(vs.entry_point("main").unwrap(), ())
                .input_assembly_state(InputAssemblyState::new())
                .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
                .fragment_shader(fs.entry_point("main").unwrap(), ())
                .depth_stencil_state(DepthStencilState::simple_depth_test())
                .render_pass(Subpass::from(render_pass.clone(), 1).unwrap())
                .build(context.device())
                .unwrap()
        };

        let sampler = Sampler::new(
            context.device(),
            SamplerCreateInfo {
                mag_filter: Filter::Nearest,
                min_filter: Filter::Nearest,
                address_mode: [SamplerAddressMode::Repeat; 3],
                mipmap_mode: SamplerMipmapMode::Nearest,
                ..SamplerCreateInfo::default()
            },
        )
        .unwrap();

        let framebuffers =
            Self::create_framebuffers(memory_allocator.clone(), output_images, render_pass.clone());

        let input_descriptor_sets = Self::create_input_descriptor_sets(
            ui_pipeline.clone(),
            output_images,
            descriptor_set_allocator.clone(),
        );

        OverlayRenderer {
            render_pass,
            quad_pipeline,
            ui_pipeline,

            index_buffer,
            vertex_buffer,
            sampler,

            quad_command_buffers: vec![],
            framebuffers,
            input_descriptor_sets,

            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,
        }
    }

    pub fn resize(&mut self, output_images: &[Arc<ImageView<SwapchainImage>>]) {
        self.framebuffers = Self::create_framebuffers(
            self.memory_allocator.clone(),
            output_images,
            self.render_pass.clone(),
        );

        self.input_descriptor_sets = Self::create_input_descriptor_sets(
            self.ui_pipeline.clone(),
            output_images,
            self.descriptor_set_allocator.clone(),
        );

        // remember to call pre_record_command_buffer_quad again
        self.quad_command_buffers.clear();
    }

    pub fn render<F>(
        &self,
        context: &Context,
        ui_components: Vec<(&GpuUIComponent, &UIComponent)>,
        future: F,
        swapchain_frame_index: u32,
        viewport: &Viewport,
    ) -> CommandBufferExecFuture<F>
    where
        F: GpuFuture + 'static,
    {
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            context.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .set_viewport(0, [viewport.clone()])
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into()), Some(1.0f32.into())],
                    ..RenderPassBeginInfo::framebuffer(
                        self.framebuffers[swapchain_frame_index as usize].clone(),
                    )
                },
                SubpassContents::SecondaryCommandBuffers,
            )
            .unwrap();

        let quad_command_buffer = self.draw_quad(swapchain_frame_index);

        builder
            .execute_commands(quad_command_buffer)
            .unwrap()
            .next_subpass(SubpassContents::SecondaryCommandBuffers)
            .unwrap();

        let ui_command_buffer =
            self.draw_ui(context, ui_components, swapchain_frame_index, viewport);

        builder
            .execute_commands(ui_command_buffer)
            .unwrap()
            .end_render_pass()
            .unwrap();

        let command_buffer = builder.build().unwrap();

        future
            .then_execute(context.queue(), command_buffer)
            .unwrap()
    }

    pub fn pre_record_command_buffer_quad(
        &mut self,
        context: &Context,
        input_images: &[Arc<ImageView<CustomStorageImage>>],
        viewport: &Viewport,
    ) {
        self.quad_command_buffers =
            self.record_quad_command_buffers(context, input_images, viewport);
    }

    fn draw_quad(&self, swapchain_frame_index: u32) -> Arc<SecondaryAutoCommandBuffer> {
        assert_ne!(self.quad_command_buffers.len(), 0);

        self.quad_command_buffers[swapchain_frame_index as usize].clone()
    }

    fn draw_ui(
        &self,
        context: &Context,
        ui_components: Vec<(&GpuUIComponent, &UIComponent)>,
        swapchain_frame_index: u32,
        viewport: &Viewport,
    ) -> SecondaryAutoCommandBuffer {
        let mut builder = AutoCommandBufferBuilder::secondary(
            &self.command_buffer_allocator,
            context.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
            CommandBufferInheritanceInfo {
                render_pass: Some(Subpass::from(self.render_pass.clone(), 1).unwrap().into()),
                ..CommandBufferInheritanceInfo::default()
            },
        )
        .unwrap();

        builder
            .set_viewport(0, [viewport.clone()])
            .bind_pipeline_graphics(self.ui_pipeline.clone());

        let set_layout = self.ui_pipeline.layout().set_layouts().get(1).unwrap();

        for (gpu_component, cpu_component) in ui_components {
            let component_push_constant = ui::vs::UIComponent {
                MVP: Matrix4::identity().into(),
            };

            let descriptor_set = PersistentDescriptorSet::new(
                &self.descriptor_set_allocator,
                set_layout.clone(),
                [WriteDescriptorSet::image_view_sampler(
                    0,
                    gpu_component.texture.image_view.clone(),
                    gpu_component.texture.sampler.clone(),
                )],
            )
            .unwrap();

            builder
                .push_constants(
                    self.ui_pipeline.layout().clone(),
                    0,
                    component_push_constant,
                )
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    self.ui_pipeline.layout().clone(),
                    0,
                    self.input_descriptor_sets[swapchain_frame_index as usize].clone(),
                )
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    self.ui_pipeline.layout().clone(),
                    1,
                    descriptor_set.clone(),
                )
                .bind_index_buffer(self.index_buffer.clone())
                .bind_vertex_buffers(0, self.vertex_buffer.clone())
                .draw_indexed(6, 1, 0, 0, 0)
                .unwrap(); // TODO: remove magic number 6
        }

        builder.build().unwrap()
    }

    fn record_quad_command_buffers(
        &self,
        context: &Context,
        images: &[Arc<ImageView<CustomStorageImage>>],
        viewport: &Viewport,
    ) -> Vec<Arc<SecondaryAutoCommandBuffer>> {
        let layout = self.quad_pipeline.layout().set_layouts().get(0).unwrap();

        images
            .iter()
            .map(|image| {
                let mut builder = AutoCommandBufferBuilder::secondary(
                    &self.command_buffer_allocator,
                    context.queue_family_index(),
                    CommandBufferUsage::MultipleSubmit,
                    CommandBufferInheritanceInfo {
                        render_pass: Some(
                            Subpass::from(self.render_pass.clone(), 0).unwrap().into(),
                        ),
                        ..CommandBufferInheritanceInfo::default()
                    },
                )
                .unwrap();

                let descriptor_set = PersistentDescriptorSet::new(
                    &self.descriptor_set_allocator,
                    layout.clone(),
                    [WriteDescriptorSet::image_view_sampler(
                        0,
                        image.clone(),
                        self.sampler.clone(),
                    )],
                )
                .unwrap();

                builder
                    .set_viewport(0, [viewport.clone()])
                    .bind_pipeline_graphics(self.quad_pipeline.clone())
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        self.quad_pipeline.layout().clone(),
                        0,
                        descriptor_set.clone(),
                    )
                    .bind_index_buffer(self.index_buffer.clone())
                    .bind_vertex_buffers(0, self.vertex_buffer.clone())
                    .draw_indexed(6, 1, 0, 0, 0)
                    .unwrap(); // TODO: remove magic number 6

                Arc::new(builder.build().unwrap())
            })
            .collect()
    }

    fn create_framebuffers(
        memory_allocator: Arc<StandardMemoryAllocator>,
        output_images: &[Arc<ImageView<SwapchainImage>>],
        render_pass: Arc<RenderPass>,
    ) -> Vec<Arc<Framebuffer>> {
        let dimensions = output_images[0].dimensions().width_height();

        output_images
            .iter()
            .map(|output_image| {
                let depth_buffer = ImageView::new_default(
                    AttachmentImage::transient(&memory_allocator, dimensions, Format::D16_UNORM)
                        .unwrap(),
                )
                .unwrap();

                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![output_image.clone(), depth_buffer.clone()],
                        ..FramebufferCreateInfo::default()
                    },
                )
                .expect("failed to create framebuffer")
            })
            .collect()
    }

    fn create_input_descriptor_sets(
        pipeline: Arc<GraphicsPipeline>,
        input_images: &[Arc<ImageView<SwapchainImage>>],
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    ) -> Vec<Arc<PersistentDescriptorSet>> {
        let layout = pipeline.layout().set_layouts().get(0).unwrap();

        input_images
            .iter()
            .map(|image| {
                PersistentDescriptorSet::new(
                    &descriptor_set_allocator,
                    layout.clone(),
                    [WriteDescriptorSet::image_view(0, image.clone())],
                )
                .unwrap()
            })
            .collect()
    }

    fn create_renderpass(context: &Context, final_output_format: Format) -> Arc<RenderPass> {
        let render_pass = {
            let create_info = {
                let mut attachment_num = 0;
                let color = attachment_num;
                attachment_num += 1;
                let depth_index = attachment_num;
                attachment_num += 1;
                let mut layouts: Vec<(Option<ImageLayout>, Option<ImageLayout>)> =
                    vec![(None, None); 2];
                let subpasses = vec![
                    {
                        let desc = render_pass::SubpassDescription {
                            color_attachments: (<[_]>::into_vec(Box::new([{
                                let layout = &mut layouts[color as usize];
                                layout.0 = layout.0.or(Some(ImageLayout::ColorAttachmentOptimal));
                                layout.1 = Some(ImageLayout::ColorAttachmentOptimal);
                                Some(render_pass::AttachmentReference {
                                    attachment: color,
                                    layout: ImageLayout::ColorAttachmentOptimal,
                                    ..Default::default()
                                })
                            }]))),
                            depth_stencil_attachment: {
                                let layout = &mut layouts[depth_index as usize];
                                layout.1 = Some(ImageLayout::DepthStencilAttachmentOptimal);
                                layout.0 = layout.0.or(layout.1);
                                let depth = Some(render_pass::AttachmentReference {
                                    attachment: depth_index,
                                    layout: ImageLayout::DepthStencilAttachmentOptimal,
                                    ..Default::default()
                                });
                                depth
                            },
                            input_attachments: (Vec::new()),
                            resolve_attachments: (Vec::new()),
                            preserve_attachments: (0..attachment_num)
                                .filter(|&a| {
                                    if a == color {
                                        return false;
                                    }
                                    if a == depth_index {
                                        return false;
                                    }
                                    true
                                })
                                .collect(),
                            ..Default::default()
                        };
                        {
                            if !(desc.resolve_attachments.is_empty()
                                || desc.resolve_attachments.len() == desc.color_attachments.len())
                            {
                                panic!("explicit panic");
                            }
                        };
                        desc
                    },
                    {
                        let desc = render_pass::SubpassDescription {
                            color_attachments: (<[_]>::into_vec(Box::new([{
                                let layout = &mut layouts[color as usize];
                                layout.0 = layout.0.or(Some(ImageLayout::ColorAttachmentOptimal));
                                layout.1 = Some(ImageLayout::ColorAttachmentOptimal);
                                Some(render_pass::AttachmentReference {
                                    attachment: color,
                                    layout: ImageLayout::General,
                                    ..Default::default()
                                })
                            }]))),
                            depth_stencil_attachment: {
                                let layout = &mut layouts[depth_index as usize];
                                layout.1 = Some(ImageLayout::DepthStencilAttachmentOptimal);
                                layout.0 = layout.0.or(layout.1);
                                let depth = Some(render_pass::AttachmentReference {
                                    attachment: depth_index,
                                    layout: ImageLayout::DepthStencilAttachmentOptimal,
                                    ..Default::default()
                                });
                                depth
                            },
                            input_attachments: (<[_]>::into_vec(Box::new([{
                                let layout = &mut layouts[color as usize];
                                layout.1 = Some(ImageLayout::General);
                                layout.0 = layout.0.or(layout.1);
                                Some(render_pass::AttachmentReference {
                                    attachment: color,
                                    layout: ImageLayout::General,
                                    ..Default::default()
                                })
                            }]))),
                            resolve_attachments: (Vec::new()),
                            preserve_attachments: (0..attachment_num)
                                .filter(|&a| {
                                    if a == color {
                                        return false;
                                    }
                                    if a == depth_index {
                                        return false;
                                    }
                                    if a == color {
                                        return false;
                                    }
                                    true
                                })
                                .collect(),
                            ..Default::default()
                        };
                        {
                            if !(desc.resolve_attachments.is_empty()
                                || desc.resolve_attachments.len() == desc.color_attachments.len())
                            {
                                panic!("explicit panic");
                            }
                        };
                        desc
                    },
                ];
                let dependencies: Vec<_> = (0..subpasses.len().saturating_sub(1) as u32)
                    .map(|id| {
                        let src_stages = vulkano::sync::PipelineStages::ALL_GRAPHICS;
                        let dst_stages = vulkano::sync::PipelineStages::ALL_GRAPHICS;
                        let src_access = vulkano::sync::AccessFlags::MEMORY_READ
                            | vulkano::sync::AccessFlags::MEMORY_WRITE;
                        let dst_access = vulkano::sync::AccessFlags::MEMORY_READ
                            | vulkano::sync::AccessFlags::MEMORY_WRITE;
                        render_pass::SubpassDependency {
                            src_subpass: id.into(),
                            dst_subpass: (id + 1).into(),
                            src_stages,
                            dst_stages,
                            src_access,
                            dst_access,
                            dependency_flags: vulkano::sync::DependencyFlags::BY_REGION,
                            ..Default::default()
                        }
                    })
                    .collect();
                let attachments = vec![
                    {
                        let layout = &mut layouts[color as usize];
                        render_pass::AttachmentDescription {
                            format: Some(final_output_format),
                            samples: SampleCount::try_from(1).unwrap(),
                            load_op: render_pass::LoadOp::Clear,
                            store_op: render_pass::StoreOp::Store,
                            stencil_load_op: render_pass::LoadOp::Clear,
                            stencil_store_op: render_pass::StoreOp::Store,
                            initial_layout: layout.0.expect("ee"),
                            final_layout: layout.1.expect("ee"),
                            ..Default::default()
                        }
                    },
                    {
                        let layout = &mut layouts[depth_index as usize];
                        render_pass::AttachmentDescription {
                            format: Some(Format::D16_UNORM),
                            samples: SampleCount::try_from(1).unwrap(),
                            load_op: render_pass::LoadOp::Clear,
                            store_op: render_pass::StoreOp::DontCare,
                            stencil_load_op: render_pass::LoadOp::Clear,
                            stencil_store_op: render_pass::StoreOp::DontCare,
                            initial_layout: layout.0.expect("ee"),
                            final_layout: layout.1.expect("ee"),
                            ..Default::default()
                        }
                    },
                ];
                render_pass::RenderPassCreateInfo {
                    attachments,
                    subpasses,
                    dependencies,
                    ..Default::default()
                }
            };
            RenderPass::new(context.device(), create_info)
        }
        .unwrap();

        render_pass
    }
}

mod quad {
    pub(crate) mod vs {
        vulkano_shaders::shader! {
            ty: "vertex",
            path: "../assets/shaders/quad/quad.vert",
        }
    }

    pub(crate) mod fs {
        vulkano_shaders::shader! {
            ty: "fragment",
            path: "../assets/shaders/quad/quad.frag",
        }
    }
}

mod ui {
    pub(crate) mod vs {
        vulkano_shaders::shader! {
            ty: "vertex",
            path: "../assets/shaders/ui/ui.vert",
        }
    }

    pub(crate) mod fs {
        vulkano_shaders::shader! {
            ty: "fragment",
            path: "../assets/shaders/ui/ui.frag",
        }
    }
}
