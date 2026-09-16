pub mod utils;
pub mod batch;

use glm::{ Vec3, Vec4 };
use layout::Rect;
use wgpu::{ Buffer, util::DeviceExt };
use winit::dpi::PhysicalSize;

use crate::{ Vertex, utils::{ physical_rect_to_ndc, physical_triangle_to_ndc } };

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn upload(&self, device: &wgpu::Device, label: &str) -> (Buffer, u64) {
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
        (buffer, bytes_vertices.len() as u64)
    }

    pub fn make_rect_mesh_from_physical_rect(
        rect: &Rect,
        color: Vec4,
        viewport: PhysicalSize<u32>
    ) -> Self {
        let (x, y, width, height) = physical_rect_to_ndc(rect, viewport);
        Self::make_rect_mesh(&(x, y, width, height, color))
    }

    pub fn make_triangle_mesh_from_physical_points(
        points: &[(f32, f32); 3],
        color: Vec4,
        viewport: PhysicalSize<u32>
    ) -> Self {
        let ndc_points = physical_triangle_to_ndc(points, viewport);
        Self::make_triangle_mesh(&ndc_points, color)
    }

    pub fn make_rect_mesh(rectangle: &(f32, f32, f32, f32, Vec4)) -> Self {
        let (x, y, width, height, color) = *rectangle;

        let vertices = vec![
            Vertex { position: Vec3::new(x, y, 0.0), color, uv: glm::vec2(0.0, 0.0) }, // Top-left
            Vertex { position: Vec3::new(x + width, y, 0.0), color, uv: glm::vec2(1.0, 0.0) }, // Top-right
            Vertex {
                position: Vec3::new(x + width, y + height, 0.0),
                color,
                uv: glm::vec2(1.0, 1.0),
            }, // Bottom-right
            Vertex { position: Vec3::new(x, y + height, 0.0), color, uv: glm::vec2(0.0, 1.0) } // Bottom-left
        ];

        let indices: Vec<u32> = vec![0, 2, 1, 2, 0, 3];

        Mesh { vertices, indices }
    }

    pub fn make_triangle_mesh(points: &[(f32, f32); 3], color: Vec4) -> Mesh {
        let vertices = vec![
            Vertex {
                position: Vec3::new(points[0].0, points[0].1, 0.0),
                color,
                uv: glm::vec2(0.0, 0.0),
            },
            Vertex {
                position: Vec3::new(points[1].0, points[1].1, 0.0),
                color,
                uv: glm::vec2(0.0, 0.0),
            },
            Vertex {
                position: Vec3::new(points[2].0, points[2].1, 0.0),
                color,
                uv: glm::vec2(0.0, 0.0),
            }
        ];

        let indices: Vec<u32> = vec![0, 2, 1];

        Mesh { vertices, indices }
    }
}
