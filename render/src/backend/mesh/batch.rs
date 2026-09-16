use glm::{ Vec3, Vec4 };
use layout::Rect;
use wgpu::{ Buffer, util::DeviceExt };
use winit::dpi::PhysicalSize;

use crate::{ Mesh, Vertex, utils::{ physical_rect_to_ndc, physical_triangle_to_ndc } };

pub struct MeshBatch {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl MeshBatch {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn upload(&self, device: &wgpu::Device, label: &'static str) -> (Buffer, u64, u32) {
        let bytes_vertices = unsafe {
            std::slice::from_raw_parts(
                self.vertices.as_ptr() as *const u8,
                self.vertices.len() * std::mem::size_of::<Vertex>()
            )
        };

        let bytes_indices = unsafe {
            std::slice::from_raw_parts(
                self.indices.as_ptr() as *const u8,
                self.indices.len() * std::mem::size_of::<u32>()
            )
        };

        let contents = &[bytes_vertices, bytes_indices].concat();

        let buffer_descriptor = wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDEX,
        };

        let buffer = device.create_buffer_init(&buffer_descriptor);
        let vertex_bytes = bytes_vertices.len() as u64;
        let index_count = self.indices.len() as u32;

        (buffer, vertex_bytes, index_count)
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    // pub fn push_quad_mesh(&mut self, mesh: Mesh) {
    // let vertex_offset = self.vertices.len() as u32;
    //
    // self.vertices.extend(mesh.vertices);
    //
    // self.indices.extend(mesh.indices.into_iter().map(|index| index + vertex_offset));
    // }

    pub fn push_quad_mesh(&mut self, rect: &Rect, color: Vec4, viewport: PhysicalSize<u32>) {
        let vertex_offset = self.vertices.len() as u32;
        let (x, y, width, height) = physical_rect_to_ndc(rect, viewport);

        self.vertices.push(Vertex {
            position: Vec3::new(x, y, 0.0),
            color,
            uv: glm::vec2(0.0, 0.0),
        }); // Top-left
        self.vertices.push(Vertex {
            position: Vec3::new(x + width, y, 0.0),
            color,
            uv: glm::vec2(1.0, 0.0),
        }); // Top-right
        self.vertices.push(Vertex {
            position: Vec3::new(x + width, y + height, 0.0),
            color,
            uv: glm::vec2(1.0, 1.0),
        }); // Bottom-right
        self.vertices.push(Vertex {
            position: Vec3::new(x, y + height, 0.0),
            color,
            uv: glm::vec2(0.0, 1.0),
        }); // Bottom-left
        self.indices.push(vertex_offset + 0);
        self.indices.push(vertex_offset + 2);
        self.indices.push(vertex_offset + 1);
        self.indices.push(vertex_offset + 2);
        self.indices.push(vertex_offset + 0);
        self.indices.push(vertex_offset + 3);
    }

    pub fn push_tri_mesh(
        &mut self,
        points: &[(f32, f32); 3],
        color: Vec4,
        viewport: PhysicalSize<u32>
    ) {
        let vertex_offset = self.vertices.len() as u32;
        let ndc_points = physical_triangle_to_ndc(points, viewport);

        self.vertices.push(Vertex {
            position: Vec3::new(ndc_points[0].0, ndc_points[0].1, 0.0),
            color,
            uv: glm::vec2(0.0, 0.0),
        });
        self.vertices.push(Vertex {
            position: Vec3::new(ndc_points[1].0, ndc_points[1].1, 0.0),
            color,
            uv: glm::vec2(0.0, 0.0),
        });
        self.vertices.push(Vertex {
            position: Vec3::new(ndc_points[2].0, ndc_points[2].1, 0.0),
            color,
            uv: glm::vec2(0.0, 0.0),
        });

        self.indices.push(vertex_offset + 0);
        self.indices.push(vertex_offset + 2);
        self.indices.push(vertex_offset + 1);
    }
}
