use std::sync::Arc;

use render::object::Object;
use render::{ Material, Mesh, UBOGroup, bind_group_layout, mesh_builder, pipeline };
use utils::profile_scope;
use wgpu::{ BackendOptions, CurrentSurfaceTexture, InstanceFlags };
use winit::dpi::{ PhysicalPosition, PhysicalSize };
use winit::window::{ Window };

pub struct State<'a> {
    // pub world: Option<World>,
    pub window: Arc<Window>,
    pub cursor_position: PhysicalPosition<f64>,
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    ubo: Option<UBOGroup>,
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

        let ubo_bind_group_layout: wgpu::BindGroupLayout;

        {
            let mut builder = bind_group_layout::Builder::new(&device);
            builder.add_ubo();
            ubo_bind_group_layout = builder.build("Material Bind Group Layout");
        }

        let render_pipeline: wgpu::RenderPipeline;
        {
            let mut builder = pipeline::Builder::new(&device);
            builder.set_shader_module("vs_main", "fs_main");
            builder.set_pixel_format(config.format);
            builder.add_vertex_buffer_layout(mesh_builder::Vertex::get_layout());
            builder.add_bind_group_layout(&material_bind_group_layout);
            builder.add_bind_group_layout(&ubo_bind_group_layout);
            render_pipeline = builder.build("Render Pipeline");
        }

        Self {
            config,
            device,
            queue,
            size,
            surface,
            window: Arc::clone(&window),
            render_pipeline,
            ubo: None,
            cursor_position: PhysicalPosition::new(400.0, 300.0),
        }
    }

    pub fn build_ubos_for_objects(&mut self, object_count: usize) {
        let ubo_bind_group_layout: wgpu::BindGroupLayout;

        {
            let mut builder = bind_group_layout::Builder::new(&self.device);
            builder.add_ubo();
            ubo_bind_group_layout = builder.build("Material Bind Group Layout");
        }

        self.ubo = Some(UBOGroup::new(&self.device, object_count, ubo_bind_group_layout));
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self) {
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
            label: Some("Render Encder"),
        };

        let mut command_encoder = self.device.create_command_encoder(&command_encoder_descriptor);

        let color_attachment = wgpu::RenderPassColorAttachment {
            view: &image_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.75, g: 0.5, b: 0.25, a: 1.0 }),
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

            // let mut offset: usize = 0;
            // Self::draw_mesh_indexed(
            //     &mut render_pass,
            //     &self.quad_material.0,
            //     &self.quad_mesh.0,
            //     &vec![],
            //     self.ubo.as_ref().unwrap(),
            //     offset
            // );

            // // offset = quads.len();

            // Self::draw_mesh_indexed(
            //     &mut render_pass,
            //     &self.triangle_material.0,
            //     &self.triangle_mesh.0,
            //     &vec![],
            //     self.ubo.as_ref().unwrap(),
            //     offset
            // );
        }

        let event = self.queue.submit(std::iter::once(command_encoder.finish()));
        let maintain = wgpu::PollType::Wait { submission_index: Some(event), timeout: None };
        self.device.poll(maintain).unwrap();
        self.queue.present(drawable);
    }

    fn draw_mesh_indexed(
        render_pass: &mut wgpu::RenderPass,
        material: &Material,
        mesh: &Mesh,
        objects: &[Object],
        ubo: &UBOGroup,
        offset: usize
    ) {
        render_pass.set_bind_group(0, &material.bind_group, &[]);

        render_pass.set_vertex_buffer(0, mesh.buffer.slice(0..mesh.offset));
        render_pass.set_index_buffer(mesh.buffer.slice(mesh.offset..), wgpu::IndexFormat::Uint16);

        for i in 0..objects.len() {
            render_pass.set_bind_group(1, &ubo.bind_groups[offset + i], &[]);
            render_pass.draw_indexed(
                0..(mesh.buffer.slice(mesh.offset..).size() as u32) /
                    (std::mem::size_of::<u16>() as u32),
                0,
                0..1
            );
        }
    }
}
