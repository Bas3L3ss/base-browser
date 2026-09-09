use render::{ Material, Mesh, bind_group_layout, make_rect_mesh, mesh_builder, pipeline };

pub mod tab;
use layout::{ LayoutNode };
pub use tab::*;
pub mod document;
pub use document::*;

use std::sync::Arc;
use utils::profile_scope;
use wgpu::{ BackendOptions, CurrentSurfaceTexture, InstanceFlags };
use winit::dpi::{ PhysicalPosition, PhysicalSize };
use winit::window::{ Window };

pub struct State<'a> {
    pub window: Arc<Window>,
    pub cursor_position: PhysicalPosition<f64>,
    pub tabs: Vec<Tab>,
    pub cur_tab: Option<TabId>,
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    default_material: Material,
}

impl<'a> State<'a> {
    pub async fn new(window: Arc<Window>) -> Self {
        profile_scope!("State Creation");
        let size = window.inner_size();

        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            backend_options: BackendOptions { ..Default::default() },
            display: None,
            flags: InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        };

        let instance = wgpu::Instance::new(instance_descriptor);

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter_descriptor = wgpu::RequestAdapterOptionsBase {
            compatible_surface: Some(&surface),
            ..Default::default()
        };

        let adapter = instance.request_adapter(&adapter_descriptor).await.unwrap();

        let device_descriptor = wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::defaults(),
            label: Some("Device"),
            ..Default::default()
        };

        let (device, queue) = adapter.request_device(&device_descriptor).await.unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);

        // Prefer an sRGB format for proper color rendering, otherwise fall back to the first format.
        let surface_format = surface_capabilities.formats
            .iter()
            .copied()
            .find(|format| format.is_srgb())
            .unwrap_or_else(|| {
                log::warn!(
                    "No sRGB surface format found; using {:?}",
                    surface_capabilities.formats[0]
                );
                surface_capabilities.formats[0]
            });

        let supported_color_spaces = surface_capabilities.color_spaces(surface_format);

        // Reordered based on HDR/wide gamut priority, then SDR, then fallback.
        let chosen_color_space = match () {
            // Prioritize HDR color spaces if available
            _ if supported_color_spaces.contains(wgpu::SurfaceColorSpaces::BT2100_PQ) => {
                wgpu::SurfaceColorSpace::Bt2100Pq
            }
            _ if supported_color_spaces.contains(wgpu::SurfaceColorSpaces::BT2100_HLG) => {
                wgpu::SurfaceColorSpace::Bt2100Hlg
            }
            // Wide-gamut SDR
            _ if supported_color_spaces.contains(wgpu::SurfaceColorSpaces::EXTENDED_DISPLAY_P3) => {
                wgpu::SurfaceColorSpace::ExtendedDisplayP3
            }
            _ if supported_color_spaces.contains(wgpu::SurfaceColorSpaces::DISPLAY_P3) => {
                wgpu::SurfaceColorSpace::DisplayP3
            }
            // Extended range
            _ if supported_color_spaces.contains(wgpu::SurfaceColorSpaces::EXTENDED_SRGB) => {
                wgpu::SurfaceColorSpace::ExtendedSrgb
            }
            _ if supported_color_spaces.contains(wgpu::SurfaceColorSpaces::EXTENDED_SRGB_LINEAR) => {
                wgpu::SurfaceColorSpace::ExtendedSrgbLinear
            }
            // Default/fallback to standard sRGB if still not matched
            _ if supported_color_spaces.contains(wgpu::SurfaceColorSpaces::SRGB) => {
                wgpu::SurfaceColorSpace::Srgb
            }
            // Fallback to Auto
            _ => { wgpu::SurfaceColorSpace::Auto }
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: chosen_color_space,
        };

        surface.configure(&device, &config);

        let material_bind_group_layout: wgpu::BindGroupLayout;

        {
            let mut builder = bind_group_layout::Builder::new(&device);
            builder.add_material();
            material_bind_group_layout = builder.build("Material Bind Group Layout");
        }

        let default_material = Material::default(
            &device,
            &queue,
            "Default Material",
            &material_bind_group_layout,
            [232, 222, 90, 3]
        );

        let render_pipeline: wgpu::RenderPipeline;
        {
            let mut builder = pipeline::Builder::new(&device);
            builder.set_shader_module("vs_main", "fs_main");
            builder.set_pixel_format(config.format);
            builder.add_vertex_buffer_layout(mesh_builder::Vertex::get_layout());
            builder.add_bind_group_layout(&material_bind_group_layout);
            render_pipeline = builder.build("Render Pipeline");
        }

        Self {
            cur_tab: None,
            config,
            device,
            queue,
            render_pipeline,
            size,
            surface,
            window: Arc::clone(&window),
            tabs: Vec::new(),
            cursor_position: PhysicalPosition::new(400.0, 300.0),
            default_material,
        }
    }

    pub fn resize_physical(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn get_current_tab(&self) -> Option<&Tab> {
        if let Some(id) = self.cur_tab {
            if let Some(tab) = self.tabs.iter().find(|tab| tab.tab_id == id) {
                if tab.document.is_some() {
                    return Some(tab);
                }
            }
        }
        None
    }

    pub fn get_current_mut_tab(&mut self) -> Option<&mut Tab> {
        if let Some(id) = self.cur_tab {
            if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.tab_id == id) {
                if tab.document.is_some() {
                    return Some(tab);
                }
            }
        }
        None
    }

    pub fn render(&mut self) {
        // Get the current tab and its document
        let (_, document) = match self.get_current_tab() {
            Some(tab) =>
                match &tab.document {
                    Some(document) => (tab, document),
                    None => {
                        return;
                    }
                }
            None => {
                return;
            }
        };

        if let Some(root) = &document.layout_tree.root {
            let drawable = match self.surface.get_current_texture() {
                CurrentSurfaceTexture::Suboptimal(t) | CurrentSurfaceTexture::Success(t) => t,

                e => {
                    dbg!(e);
                    panic!()
                }
            };

            let image_view_descriptor = wgpu::TextureViewDescriptor::default();
            let image_view = drawable.texture.create_view(&image_view_descriptor);

            let command_encoder_descriptor = wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            };

            let mut command_encoder = self.device.create_command_encoder(
                &command_encoder_descriptor
            );

            let color_attachment = wgpu::RenderPassColorAttachment {
                view: &image_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }),

                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            };

            let render_pass_descriptor = wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(color_attachment)],
                ..Default::default()
            };

            {
                let mut render_pass = command_encoder.begin_render_pass(&render_pass_descriptor);
                render_pass.set_pipeline(&self.render_pipeline);
                let size = self.window.inner_size();
                self.render_layout_tree(&root, &mut render_pass, 0, size);
            }

            let event: wgpu::SubmissionIndex = self.queue.submit(
                std::iter::once(command_encoder.finish())
            );
            let maintain = wgpu::PollType::Wait { submission_index: Some(event), timeout: None };
            self.device.poll(maintain).unwrap();
            self.queue.present(drawable);
        }
    }

    pub fn draw_mesh_indexed(
        &self,
        render_pass: &mut wgpu::RenderPass,
        mesh: &Mesh,
        material: &Material
    ) {
        render_pass.set_bind_group(0, &material.bind_group, &[]);
        render_pass.set_vertex_buffer(0, mesh.buffer.slice(0..mesh.offset));
        render_pass.set_index_buffer(mesh.buffer.slice(mesh.offset..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(
            0..(mesh.buffer.slice(mesh.offset..).size() as u32) /
                (std::mem::size_of::<u16>() as u32),
            0,
            0..1
        );
    }

    /// `make_rect_mesh` expects the coordinates and sizes to be normalized to NDC (-1.0 to 1.0).
    pub fn render_layout_tree(
        &self,
        node: &LayoutNode,
        render_pass: &mut wgpu::RenderPass,
        depth: usize,
        size: PhysicalSize<u32>
    ) {
        println!("---------------------------------------------------------------------------");
        println!("{node:?}");

        if let Some(dim) = node.dimensions() {
            // Generate a random pastel color
            let color = glm::vec3(
                rand::random::<f32>() * 0.6 + 0.2,
                rand::random::<f32>() * 0.6 + 0.2,
                rand::random::<f32>() * 0.6 + 0.2
            );

            let x_ndc = (2.0 * dim.content.x) / (size.width as f32) - 1.0;
            let y_ndc = 1.0 - (2.0 * dim.content.y) / (size.height as f32);

            let width_ndc = (2.0 * dim.content.width) / (size.width as f32);
            let height_ndc = (-2.0 * dim.content.height) / (size.height as f32);
            println!("{x_ndc}, {y_ndc}, {width_ndc}, {height_ndc}");

            let rects = [(x_ndc, y_ndc, width_ndc, height_ndc, color)];

            let mesh = make_rect_mesh(&self.device, &rects);
            self.draw_mesh_indexed(render_pass, &mesh, &self.default_material);

            for child in node.children() {
                self.render_layout_tree(child, render_pass, depth + 1, size);
            }
        }
    }
}
