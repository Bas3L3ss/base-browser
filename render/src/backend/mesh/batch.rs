use glm::{ Vec3, Vec4 };
use layout::Rect;
use winit::dpi::PhysicalSize;

use crate::{ Vertex, utils::{ physical_rect_to_ndc, physical_triangle_to_ndc } };

const INITIAL_VERTEX_CAPACITY: usize = 4096;
const INITIAL_INDEX_CAPACITY: usize = 8192;

pub struct MeshBatch {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    // Number of ELEMENTS, not bytes.
    vertex_capacity: usize,
    index_capacity: usize,
}

impl MeshBatch {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertex_capacity = INITIAL_VERTEX_CAPACITY;
        let index_capacity = INITIAL_INDEX_CAPACITY;

        let vertex_buffer = Self::create_vertex_buffer(device, vertex_capacity);

        let index_buffer = Self::create_index_buffer(device, index_capacity);

        Self {
            // Reserve CPU memory too.
            vertices: Vec::with_capacity(vertex_capacity),
            indices: Vec::with_capacity(index_capacity),

            vertex_buffer,
            index_buffer,

            vertex_capacity,
            index_capacity,
        }
    }

    fn create_vertex_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
        device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Mesh Batch Vertex Buffer"),

                size: (capacity * std::mem::size_of::<Vertex>()) as u64,

                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,

                mapped_at_creation: false,
            })
        )
    }

    fn create_index_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
        device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Mesh Batch Index Buffer"),

                size: (capacity * std::mem::size_of::<u32>()) as u64,

                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,

                mapped_at_creation: false,
            })
        )
    }

    fn ensure_gpu_capacity(&mut self, device: &wgpu::Device) {
        if self.vertices.len() > self.vertex_capacity {
            self.vertex_capacity = self.vertices.len().next_power_of_two();

            self.vertex_buffer = Self::create_vertex_buffer(device, self.vertex_capacity);
        }

        if self.indices.len() > self.index_capacity {
            self.index_capacity = self.indices.len().next_power_of_two();

            self.index_buffer = Self::create_index_buffer(device, self.index_capacity);
        }
    }

    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.ensure_gpu_capacity(device);

        if !self.vertices.is_empty() {
            let vertex_bytes = unsafe {
                std::slice::from_raw_parts(
                    self.vertices.as_ptr() as *const u8,
                    self.vertices.len() * std::mem::size_of::<Vertex>()
                )
            };

            queue.write_buffer(&self.vertex_buffer, 0, vertex_bytes);
        }

        if !self.indices.is_empty() {
            let index_bytes = unsafe {
                std::slice::from_raw_parts(
                    self.indices.as_ptr() as *const u8,
                    self.indices.len() * std::mem::size_of::<u32>()
                )
            };

            queue.write_buffer(&self.index_buffer, 0, index_bytes);
        }
    }

    pub fn clear(&mut self) {
        // Keeps the Vec allocations/capacity.
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn index_count(&self) -> u32 {
        self.indices.len() as u32
    }

    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    pub fn push_quad_mesh(&mut self, rect: &Rect, color: Vec4, viewport: PhysicalSize<u32>) {
        let base = self.vertices.len() as u32;

        let (x, y, width, height) = physical_rect_to_ndc(rect, viewport);

        self.vertices.extend_from_slice(
            &[
                Vertex {
                    position: Vec3::new(x, y, 0.0),
                    color,
                    uv: glm::vec2(0.0, 0.0),
                },

                Vertex {
                    position: Vec3::new(x + width, y, 0.0),
                    color,
                    uv: glm::vec2(1.0, 0.0),
                },

                Vertex {
                    position: Vec3::new(x + width, y + height, 0.0),
                    color,
                    uv: glm::vec2(1.0, 1.0),
                },

                Vertex {
                    position: Vec3::new(x, y + height, 0.0),
                    color,
                    uv: glm::vec2(0.0, 1.0),
                },
            ]
        );

        self.indices.extend_from_slice(&[base, base + 2, base + 1, base + 2, base, base + 3]);
    }

    pub fn push_tri_mesh(
        &mut self,
        points: &[(f32, f32); 3],
        color: Vec4,
        viewport: PhysicalSize<u32>
    ) {
        let base = self.vertices.len() as u32;

        let points = physical_triangle_to_ndc(points, viewport);

        self.vertices.extend_from_slice(
            &[
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
                },
            ]
        );

        self.indices.extend_from_slice(&[base, base + 2, base + 1]);
    }
}
