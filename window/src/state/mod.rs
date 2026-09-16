use font::{ FontManager };
use render::render::render_layout_tree;
use render::{ GlyphAtlas, Material, Vertex, bind_group_layout, pipeline };
pub mod tab;
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
    glyph_atlas: GlyphAtlas,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    glyph_bind_group_layout: wgpu::BindGroupLayout,
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

        let chosen_color_space = if supported_color_spaces.contains(wgpu::SurfaceColorSpaces::SRGB) {
            wgpu::SurfaceColorSpace::Srgb
        } else {
            wgpu::SurfaceColorSpace::Auto
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

        // Create glyph bind group layout
        let glyph_bind_group_layout: wgpu::BindGroupLayout;
        {
            let mut builder = bind_group_layout::Builder::new(&device);
            builder.add_glyph_texture_and_sampler();
            glyph_bind_group_layout = builder.build("Glyph Bind Group Layout");
        }

        let default_material = Material::default(
            &device,
            &queue,
            "Default Material",
            &glyph_bind_group_layout,
            [255, 255, 255, 255]
        );

        let render_pipeline: wgpu::RenderPipeline;
        {
            let mut builder = pipeline::Builder::new(&device);
            builder.set_shader_module("vs_main", "fs_main");
            builder.set_pixel_format(config.format);
            builder.add_vertex_buffer_layout(Vertex::get_layout());
            builder.add_bind_group_layout(&glyph_bind_group_layout);
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
            glyph_atlas: GlyphAtlas::new(),
            glyph_bind_group_layout,
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

    pub fn render(&mut self, font_manager: &FontManager) {
        let tab = match self.cur_tab {
            Some(id) =>
                match self.tabs.iter().find(|tab| tab.tab_id == id && tab.document.is_some()) {
                    Some(tab) => tab,
                    None => {
                        return;
                    }
                }
            None => {
                return;
            }
        };

        let document = match &tab.document {
            Some(document) => document,
            None => {
                return;
            }
        };

        let root = if let Some(root) = &document.layout_tree.root {
            root
        } else {
            return;
        };

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

        let mut command_encoder = self.device.create_command_encoder(&command_encoder_descriptor);

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
            //cannot borrow `*self` as mutable because it is also borrowed as immutable
            render_layout_tree(
                &root,
                &mut render_pass,
                &mut self.glyph_atlas,
                0,
                size,
                &document.computed_styles,
                font_manager,
                &self.device,
                &self.default_material,
                &self.queue,
                &self.glyph_bind_group_layout
            );
        }

        let event: wgpu::SubmissionIndex = self.queue.submit(
            std::iter::once(command_encoder.finish())
        );
        let maintain = wgpu::PollType::Wait { submission_index: Some(event), timeout: None };
        self.device.poll(maintain).unwrap();
        self.queue.present(drawable);
    }
}
