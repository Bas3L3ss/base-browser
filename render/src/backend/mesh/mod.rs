pub mod utils;
use glm::{ Vec3, Vec4 };
use layout::Rect;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

use crate::{ Vertex, utils::{ physical_rect_to_ndc, physical_triangle_to_ndc } };

pub struct Mesh {
    pub buffer: wgpu::Buffer,
    pub offset: u64,
}

impl Mesh {
    // NOTE: In this code, each call creates a new GPU buffer for every mesh (rectangle or triangle) generated.
    // This is simple and works fine for very small numbers of dynamic meshes, but can be wasteful for
    // performance-critical or high-frequency scenarios. Allocating buffers every frame or for every primitive
    // leads to overhead and GPU memory churn.
    //
    // # Performance/Caching tips:
    // - **GPU Buffer Caching:** Reuse buffers: allocate/upsize big enough vertex/index buffers up front,
    //   and update regions in-place. Use buffer.map/write rather than recreating buffers constantly.
    // - **Dynamic Buffer Mapping:** For top efficiency with many dynamic meshes, one would typically use
    //   a ring buffer or upload heap pattern. See wgpu's MAP_WRITE/BYTES and queue.write_buffer for partial updates.
    // - **Batching:** You can batch many small meshes into big ones and draw with a single draw call
    //   (changing offsets as needed).
    // - **Uniform Buffer Objects (UBO):** "UBO" refers to Uniform Buffer Object, very common in OpenGL/WebGPU.
    //   Use for passing small, per-draw (or per-frame) data (constants, transforms, colors) to the GPU, rather than
    //   baking them into every vertex. This reduces vertex memory pressure—though may not be relevant if your shapes
    //   are totally dynamic.
    // - **Instancing:** For many identical shapes at different positions/sizes/colors, instance data in a UBO/storage buffer.
    //   Draw many with a single mesh definition + instance buffer.
    //
    // # When is the current approach OK?
    // - For simple UI demos, small numbers of unique items, educational code, or where code clarity > speed.
    //
    // # When should you optimize as above?
    // - If you profile and see lots of wgpu buffer creation, extra allocations, or have choppy framerates when many rectangles/triangles.
    // - If you have repetitive geometry that could use instancing or batching.

    fn buffer_with_vertices_indices(
        device: &wgpu::Device,
        vertices: &[Vertex],
        indices: &[u16],
        label: &'static str
    ) -> Mesh {
        let bytes_vertices = unsafe {
            std::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                vertices.len() * std::mem::size_of::<Vertex>()
            )
        };

        let bytes_indices = unsafe {
            std::slice::from_raw_parts(
                indices.as_ptr() as *const u8,
                indices.len() * std::mem::size_of::<u16>()
            )
        };

        let contents = &[bytes_vertices, bytes_indices].concat();

        let buffer_descriptor = wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDEX,
        };

        let buffer = device.create_buffer_init(&buffer_descriptor);

        Mesh { buffer, offset: bytes_vertices.len() as u64 }
    }

    pub fn make_rect_mesh_from_physical_rect(
        device: &wgpu::Device,
        rect: &Rect,
        color: Vec4,
        viewport: PhysicalSize<u32>
    ) -> Self {
        let (x, y, width, height) = physical_rect_to_ndc(rect, viewport);
        Self::make_rect_mesh(device, &(x, y, width, height, color))
    }

    pub fn make_triangle_mesh_from_physical_points(
        device: &wgpu::Device,
        points: &[(f32, f32); 3],
        color: Vec4,
        viewport: PhysicalSize<u32>
    ) -> Self {
        let ndc_points = physical_triangle_to_ndc(points, viewport);
        Self::make_triangle_mesh(device, &ndc_points, color)
    }

    pub fn make_rect_mesh(device: &wgpu::Device, rectangle: &(f32, f32, f32, f32, Vec4)) -> Self {
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

        let indices: Vec<u16> = vec![0, 2, 1, 2, 0, 3];

        Self::buffer_with_vertices_indices(device, &vertices, &indices, "Quad buffer")
    }

    pub fn make_triangle_mesh(
        device: &wgpu::Device,
        points: &[(f32, f32); 3],
        color: Vec4
    ) -> Mesh {
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

        let indices: Vec<u16> = vec![0, 2, 1];

        Self::buffer_with_vertices_indices(device, &vertices, &indices, "Triangle buffer")
    }
}
