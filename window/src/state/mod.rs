use css::{ ComputedStyle, Edges };
use font::FontManager;
use glm::Vec4;
use layout::text::inline_fragment::InlineFragment;
use render::{
    GlyphAtlas,
    Material,
    Mesh,
    Vertex,
    bind_group_layout,
    make_rect_mesh_from_physical_rect,
    make_triangle_mesh_from_physical_points,
    mesh_builder,
    pipeline,
};

use layout::{ LayoutNode, Rect };
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
            [255, 255, 255, 255]
        );

        // Create glyph bind group layout
        let glyph_bind_group_layout: wgpu::BindGroupLayout;
        {
            let mut builder = bind_group_layout::Builder::new(&device);
            builder.add_glyph_texture_and_sampler();
            glyph_bind_group_layout = builder.build("Glyph Bind Group Layout");
        }

        let render_pipeline: wgpu::RenderPipeline;
        {
            let mut builder = pipeline::Builder::new(&device);
            builder.set_shader_module("vs_main", "fs_main");
            builder.set_pixel_format(config.format);
            builder.add_vertex_buffer_layout(Vertex::get_layout());
            builder.add_bind_group_layout(&material_bind_group_layout);
            // builder.add_bind_group_layout(&glyph_bind_group_layout);
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

    pub fn render(&mut self, font_manager: &FontManager) {
        // Get the current tab and its document
        let (_, document) = match self.get_current_tab() {
            Some(tab) =>
                match &tab.document {
                    Some(document) => (tab, document.clone()), // TODO: FIX CLONE - OWNERSHIP PROBLEM
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
                self.render_layout_tree(&root, &mut render_pass, 0, size, &document, font_manager);
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

    /// Paints each element's background over its border box (padding + border, not margin).
    pub fn render_layout_tree(
        &mut self,
        node: &LayoutNode,
        render_pass: &mut wgpu::RenderPass,
        depth: usize,
        size: PhysicalSize<u32>,
        document: &Document,
        font_manager: &FontManager
    ) {
        println!("---------------------------------------------------------------------------");
        // println!("{node:?}");
        let node_id = node.node_id();
        match node_id {
            Some(id) => {
                let computed_style = document.computed_styles[id].as_ref();

                self.render_node(
                    node,
                    render_pass,
                    depth,
                    size,
                    document,
                    computed_style,
                    font_manager
                );
            }
            None => {
                self.render_node(node, render_pass, depth, size, document, None, font_manager);
            }
        }
    }
    fn render_node(
        &mut self,
        node: &LayoutNode,
        render_pass: &mut wgpu::RenderPass,
        depth: usize,
        size: PhysicalSize<u32>,
        document: &Document,
        computed_style: Option<&ComputedStyle>,
        font_manager: &FontManager
    ) {
        if let Some(background) = node.background_rect() {
            let color = match computed_style {
                Some(style) => {
                    let bg_color = style.bg_color();
                    glm::Vector4 {
                        x: bg_color[0],
                        y: bg_color[1],
                        z: bg_color[2],
                        w: bg_color[3],
                    }
                }
                _ => { glm::vec4(0.0, 0.0, 0.0, 0.0) }
            };
            let mesh = make_rect_mesh_from_physical_rect(&self.device, &background, color, size);
            self.draw_mesh_indexed(render_pass, &mesh, &self.default_material);
            self.apply_border(node, computed_style, render_pass, size);
        } else if let Some(fragments) = node.text_fragments() {
            for fragment in fragments {
                match fragment {
                    InlineFragment::Text(text_frag) => {
                        let mut pen_x = text_frag.x;
                        let baseline_y = text_frag.y + text_frag.baseline_offset;
                        if let Some(font) = font_manager.cache.get(&text_frag.font_key) {
                            for glyph in &text_frag.glyph_run.glyphs {
                                if
                                    let Some(image) = self.glyph_atlas.get_glpyh_image(
                                        text_frag.font_size,
                                        &font,
                                        glyph.x_offset,
                                        glyph.y_offset,
                                        glyph.glyph_id
                                    )
                                {
                                    // Calculate glyph position
                                    let glyph_x =
                                        pen_x + glyph.x_offset + (image.placement.left as f32);
                                    let glyph_y =
                                        baseline_y - glyph.y_offset - (image.placement.top as f32);

                                    let rect = Rect {
                                        x: glyph_x,
                                        y: glyph_y,
                                        width: image.placement.width as f32,
                                        height: image.placement.height as f32,
                                    };

                                    // Build mesh for this glyph
                                    let mesh = make_rect_mesh_from_physical_rect(
                                        &self.device,
                                        &rect,
                                        // Use the desired text color, e.g. style.color(), or fallback
                                        glm::vec4(0.0, 0.0, 0.0, 1.0),
                                        size
                                    );

                                    self.draw_mesh_indexed(
                                        render_pass,
                                        &mesh,
                                        &self.default_material
                                    );
                                    pen_x += glyph.x_advance;
                                }
                            }
                        }
                    }
                }
            }
        }

        for child in node.children() {
            self.render_layout_tree(child, render_pass, depth + 1, size, document, font_manager);
        }
    }

    fn apply_border(
        &self,
        node: &LayoutNode,
        computed_style: Option<&ComputedStyle>,
        render_pass: &mut wgpu::RenderPass,
        size: PhysicalSize<u32>
    ) {
        if let Some(dimensions) = node.dimensions() {
            if let Some(border) = node.background_rect() {
                if let Some(style) = computed_style {
                    let border_color: Edges<[f32; 4]> = style.border_color();

                    let top_rect = Rect {
                        x: border.x + dimensions.border.left,
                        y: border.y,
                        width: border.width - dimensions.border.left - dimensions.border.right,
                        height: dimensions.border.top,
                    };
                    let right_rect = Rect {
                        x: border.x + border.width - dimensions.border.right,
                        y: border.y + dimensions.border.top,
                        width: dimensions.border.right,
                        height: border.height - dimensions.border.top - dimensions.border.bottom,
                    };
                    let bottom_rect = Rect {
                        x: border.x + dimensions.border.left,
                        y: border.y + border.height - dimensions.border.bottom,
                        width: border.width - dimensions.border.left - dimensions.border.right,
                        height: dimensions.border.bottom,
                    };
                    let left_rect = Rect {
                        x: border.x,
                        y: border.y + dimensions.border.top,
                        width: dimensions.border.left,
                        height: border.height - dimensions.border.top - dimensions.border.bottom,
                    };

                    let rects_colors: [(Rect, Vec4); 4] = [
                        (
                            top_rect,
                            glm::vec4(
                                border_color.top[0],
                                border_color.top[1],
                                border_color.top[2],
                                border_color.top[3]
                            ),
                        ),
                        (
                            right_rect,
                            glm::vec4(
                                border_color.right[0],
                                border_color.right[1],
                                border_color.right[2],
                                border_color.right[3]
                            ),
                        ),
                        (
                            bottom_rect,
                            glm::vec4(
                                border_color.bottom[0],
                                border_color.bottom[1],
                                border_color.bottom[2],
                                border_color.bottom[3]
                            ),
                        ),
                        (
                            left_rect,
                            glm::vec4(
                                border_color.left[0],
                                border_color.left[1],
                                border_color.left[2],
                                border_color.left[3]
                            ),
                        ),
                    ];

                    for (rect, color) in rects_colors {
                        let mesh = make_rect_mesh_from_physical_rect(
                            &self.device,
                            &rect,
                            color,
                            size
                        );
                        self.draw_mesh_indexed(render_pass, &mesh, &self.default_material);
                    }

                    let triangle_corners_and_colors: [([(f32, f32); 3], Vec4); 8] = [
                        (
                            [
                                (border.x, border.y),
                                (border.x + dimensions.border.left, border.y),
                                (
                                    border.x + dimensions.border.left,
                                    border.y + dimensions.border.top,
                                ),
                            ],
                            glm::vec4(
                                border_color.top[0],
                                border_color.top[1],
                                border_color.top[2],
                                border_color.top[3]
                            ),
                        ),
                        (
                            [
                                (border.x + border.width - dimensions.border.right, border.y),
                                (border.x + border.width, border.y),
                                (
                                    border.x + border.width - dimensions.border.right,
                                    border.y + dimensions.border.top,
                                ),
                            ],
                            glm::vec4(
                                border_color.top[0],
                                border_color.top[1],
                                border_color.top[2],
                                border_color.top[3]
                            ),
                        ),
                        (
                            [
                                (border.x, border.y),
                                (
                                    border.x + dimensions.border.left,
                                    border.y + dimensions.border.top,
                                ),
                                (border.x, border.y + dimensions.border.top),
                            ],
                            glm::vec4(
                                border_color.left[0],
                                border_color.left[1],
                                border_color.left[2],
                                border_color.left[3]
                            ),
                        ),
                        (
                            [
                                (border.x, border.y + border.height - dimensions.border.bottom),
                                (
                                    border.x + dimensions.border.left,
                                    border.y + border.height - dimensions.border.bottom,
                                ),
                                (border.x, border.y + border.height),
                            ],
                            glm::vec4(
                                border_color.left[0],
                                border_color.left[1],
                                border_color.left[2],
                                border_color.left[3]
                            ),
                        ),
                        (
                            [
                                (
                                    border.x + dimensions.border.left,
                                    border.y + border.height - dimensions.border.bottom,
                                ),
                                (border.x + dimensions.border.left, border.y + border.height),
                                (border.x, border.y + border.height),
                            ],
                            glm::vec4(
                                border_color.bottom[0],
                                border_color.bottom[1],
                                border_color.bottom[2],
                                border_color.bottom[3]
                            ),
                        ),
                        (
                            [
                                (border.x + border.width, border.y + border.height),

                                (
                                    border.x + border.width - dimensions.border.right,
                                    border.y + border.height,
                                ),

                                (
                                    border.x + border.width - dimensions.border.right,
                                    border.y + border.height - dimensions.border.bottom,
                                ),
                            ],
                            glm::vec4(
                                border_color.bottom[0],
                                border_color.bottom[1],
                                border_color.bottom[2],
                                border_color.bottom[3]
                            ),
                        ),
                        (
                            [
                                (
                                    border.x + border.width - dimensions.border.right,
                                    border.y + border.height - dimensions.border.bottom,
                                ),

                                (
                                    border.x + border.width,
                                    border.y + border.height - dimensions.border.bottom,
                                ),

                                (border.x + border.width, border.y + border.height),
                            ],
                            glm::vec4(
                                border_color.right[0],
                                border_color.right[1],
                                border_color.right[2],
                                border_color.right[3]
                            ),
                        ),
                        (
                            [
                                (border.x + border.width, border.y),

                                (border.x + border.width, border.y + dimensions.border.top),

                                (
                                    border.x + border.width - dimensions.border.left,
                                    border.y + dimensions.border.top,
                                ),
                            ],
                            glm::vec4(
                                border_color.right[0],
                                border_color.right[1],
                                border_color.right[2],
                                border_color.right[3]
                            ),
                        ),
                    ];

                    for (corners, color) in triangle_corners_and_colors.iter() {
                        let tmesh = make_triangle_mesh_from_physical_points(
                            &self.device,
                            corners,
                            *color,
                            size
                        );
                        self.draw_mesh_indexed(render_pass, &tmesh, &self.default_material);
                    }
                }
            }
        }
    }
}
