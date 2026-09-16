use css::{ ComputedStyle, Edges };
use font::{ FontManager, GlyphPosition };
use glm::Vec4;
use layout::{ LayoutNode, Rect, text::inline_fragment::InlineFragment };
use swash::scale::image::Image;
use winit::dpi::PhysicalSize;

use crate::{ GlyphAtlas, Material, Mesh, batch::MeshBatch };

// pub fn draw_indexed_batch_mesh(
//     render_pass: &mut wgpu::RenderPass,
//     buffer: Buffer,
//     offset: u64,
//     material: &Material
// ) {
//     render_pass.set_bind_group(0, &material.bind_group, &[]);
//     render_pass.set_vertex_buffer(0, buffer.slice(0..offset));
//     render_pass.set_index_buffer(buffer.slice(offset..), wgpu::IndexFormat::Uint32);
//     render_pass.draw_indexed(
//         0..(buffer.slice(offset..).size() as u32) / (std::mem::size_of::<u32>() as u32),
//         0,
//         0..1
//     );
// }

pub fn draw_indexed_batch_mesh(
    render_pass: &mut wgpu::RenderPass,
    buffer: &wgpu::Buffer,
    vertex_bytes: u64,
    index_count: u32,
    material: &Material
) {
    render_pass.set_bind_group(0, &material.bind_group, &[]);

    render_pass.set_vertex_buffer(0, buffer.slice(0..vertex_bytes));

    render_pass.set_index_buffer(buffer.slice(vertex_bytes..), wgpu::IndexFormat::Uint32);

    render_pass.draw_indexed(0..index_count, 0, 0..1);
}

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
    glyph_bind_group_layout: &wgpu::BindGroupLayout,
    mesh_batch: &mut MeshBatch
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
        glyph_bind_group_layout,
        mesh_batch
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
    glyph_bind_group_layout: &wgpu::BindGroupLayout,
    mesh_batch: &mut MeshBatch
) {
    if let Some(background) = node.background_rect() {
        let color = match computed_style {
            Some(style) => {
                let bg_color = style.bg_color();
                glm::Vector4::new(bg_color[0], bg_color[1], bg_color[2], bg_color[3])
            }
            None => glm::vec4(0.0, 0.0, 0.0, 0.0),
        };
        mesh_batch.push_quad_mesh(&background, color, size);
        apply_border(node, computed_style, size, mesh_batch);
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
            glyph_bind_group_layout,
            mesh_batch
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

    // let mesh = Mesh::make_rect_mesh_from_physical_rect(&rect, glm::vec4(0.0, 0.0, 0.0, 1.0), size);

    // let material = Material::new(&image, device, queue, "Glyph text", glyph_bind_group_layout);

    // let (buffer, offset) = mesh.upload(&device, "Glyph Mesh");

    // draw_mesh_indexed(render_pass, &buffer, offset, &material);
}

fn apply_border(
    node: &LayoutNode,
    computed_style: Option<&ComputedStyle>,
    size: PhysicalSize<u32>,
    mesh_batch: &mut MeshBatch
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
                    mesh_batch.push_quad_mesh(rect, *color, size);
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
                    mesh_batch.push_tri_mesh(corners, *color, size);
                }
            }
        }
    }
}

pub fn draw_mesh_indexed(
    render_pass: &mut wgpu::RenderPass,
    buffer: &wgpu::Buffer,
    offset: u64,
    material: &Material
) {
    render_pass.set_bind_group(0, &material.bind_group, &[]);
    render_pass.set_vertex_buffer(0, buffer.slice(0..offset));
    render_pass.set_index_buffer(buffer.slice(offset..), wgpu::IndexFormat::Uint32);
    render_pass.draw_indexed(
        0..(buffer.slice(offset..).size() as u32) / (std::mem::size_of::<u32>() as u32),
        0,
        0..1
    );
}
