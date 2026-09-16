use css::{ ComputedStyle, Edges };
use font::{ FontManager, GlyphPosition };
use glm::Vec4;
use layout::{ LayoutNode, Rect, text::inline_fragment::InlineFragment };
use swash::scale::image::Image;
use winit::dpi::PhysicalSize;

use crate::{ GlyphAtlas, Material, Mesh };

pub fn render_layout_tree(
    node: &LayoutNode,
    render_pass: &mut wgpu::RenderPass,
    glyph_atlas: &mut GlyphAtlas,
    depth: usize,
    size: PhysicalSize<u32>,
    computed_styles: &[Option<ComputedStyle>],
    font_manager: &FontManager,
    device: &wgpu::Device,
    default_material: &Material,
    queue: &wgpu::Queue,
    glyph_bind_group_layout: &wgpu::BindGroupLayout
) {
    let node_id = node.node_id();
    let computed_style = node_id.and_then(|id| computed_styles[id].as_ref());

    render_node(
        node,
        render_pass,
        glyph_atlas,
        depth,
        size,
        computed_style,
        font_manager,
        default_material,
        device,
        computed_styles,
        queue,
        glyph_bind_group_layout
    );
}

fn render_node(
    node: &LayoutNode,
    render_pass: &mut wgpu::RenderPass,
    glyph_atlas: &mut GlyphAtlas,
    depth: usize,
    size: PhysicalSize<u32>,
    computed_style: Option<&ComputedStyle>,
    font_manager: &FontManager,
    default_material: &Material,
    device: &wgpu::Device,
    computed_styles: &[Option<ComputedStyle>],
    queue: &wgpu::Queue,
    glyph_bind_group_layout: &wgpu::BindGroupLayout
) {
    if let Some(background) = node.background_rect() {
        let color = match computed_style {
            Some(style) => {
                let bg_color = style.bg_color();
                glm::Vector4::new(bg_color[0], bg_color[1], bg_color[2], bg_color[3])
            }
            None => glm::vec4(0.0, 0.0, 0.0, 0.0),
        };
        let mesh = Mesh::make_rect_mesh_from_physical_rect(&device, &background, color, size);
        draw_mesh_indexed(render_pass, &mesh, &default_material);
        apply_border(node, computed_style, render_pass, size, device, default_material);
    } else if let Some(fragments) = node.text_fragments() {
        for fragment in fragments {
            match fragment {
                InlineFragment::Text(text_frag) => {
                    let mut pen_x = text_frag.x;
                    let baseline_y = text_frag.y + text_frag.baseline_offset;
                    if let Some(font) = font_manager.cache.get(&text_frag.font_key) {
                        for glyph_pos in &text_frag.glyph_run.glyphs {
                            if
                                let Some(image) = glyph_atlas.get_glpyh_image(
                                    text_frag.font_size,
                                    &font,
                                    glyph_pos.x_offset,
                                    glyph_pos.y_offset,
                                    glyph_pos.glyph_id
                                )
                            {
                                render_glyph_rect(
                                    glyph_pos,
                                    size,
                                    render_pass,
                                    pen_x,
                                    baseline_y,
                                    &image,
                                    device,
                                    queue,
                                    glyph_bind_group_layout
                                );
                                pen_x += glyph_pos.x_advance;
                            }
                        }
                    }
                }
            }
        }
    }

    for child in node.children() {
        render_layout_tree(
            child,
            render_pass,
            glyph_atlas,
            depth + 1,
            size,
            computed_styles,
            font_manager,
            device,
            default_material,
            queue,
            glyph_bind_group_layout
        );
    }
}

fn render_glyph_rect(
    glyph_pos: &GlyphPosition,
    size: winit::dpi::PhysicalSize<u32>,
    render_pass: &mut wgpu::RenderPass,
    pen_x: f32,
    baseline_y: f32,
    image: &Image,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    glyph_bind_group_layout: &wgpu::BindGroupLayout
) {
    if image.placement.width == 0 || image.placement.height == 0 {
        return;
    }

    let glyph_x = pen_x + glyph_pos.x_offset + (image.placement.left as f32);
    let glyph_y = baseline_y - glyph_pos.y_offset - (image.placement.top as f32);

    let rect = Rect {
        x: glyph_x,
        y: glyph_y,
        width: image.placement.width as f32,
        height: image.placement.height as f32,
    };

    let mesh = Mesh::make_rect_mesh_from_physical_rect(
        device,
        &rect,
        glm::vec4(0.0, 0.0, 0.0, 1.0),
        size
    );

    // let material = Material::new(&image, device, queue, "Glyph text", glyph_bind_group_layout);
    // draw_mesh_indexed(render_pass, &mesh, &material);
}

pub fn draw_mesh_indexed(render_pass: &mut wgpu::RenderPass, mesh: &Mesh, material: &Material) {
    render_pass.set_bind_group(0, &material.bind_group, &[]);
    render_pass.set_vertex_buffer(0, mesh.buffer.slice(0..mesh.offset));
    render_pass.set_index_buffer(mesh.buffer.slice(mesh.offset..), wgpu::IndexFormat::Uint16);
    render_pass.draw_indexed(
        0..(mesh.buffer.slice(mesh.offset..).size() as u32) / (std::mem::size_of::<u16>() as u32),
        0,
        0..1
    );
}

fn apply_border(
    node: &LayoutNode,
    computed_style: Option<&ComputedStyle>,
    render_pass: &mut wgpu::RenderPass,
    size: PhysicalSize<u32>,
    device: &wgpu::Device,
    default_material: &Material
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

                for (rect, color) in rects_colors.iter() {
                    let mesh = Mesh::make_rect_mesh_from_physical_rect(device, rect, *color, size);
                    draw_mesh_indexed(render_pass, &mesh, default_material);
                }

                let triangle_corners_and_colors: [([(f32, f32); 3], Vec4); 8] = [
                    (
                        [
                            (border.x, border.y),
                            (border.x + dimensions.border.left, border.y),
                            (border.x + dimensions.border.left, border.y + dimensions.border.top),
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
                            (border.x + dimensions.border.left, border.y + dimensions.border.top),
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
                    let tmesh = Mesh::make_triangle_mesh_from_physical_points(
                        device,
                        corners,
                        *color,
                        size
                    );
                    draw_mesh_indexed(render_pass, &tmesh, default_material);
                }
            }
        }
    }
}
