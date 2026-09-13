use glm::{ Vec3, Vec4 };
use layout::Rect;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

pub struct TriangleMesh(pub Mesh);
pub struct QuadMesh(pub Mesh);

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: Vec3,
    pub color: Vec4,
}

pub struct Mesh {
    pub buffer: wgpu::Buffer,
    pub offset: u64,
}

impl Vertex {
    pub fn get_layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [
            wgpu::VertexAttribute;
            2
        ] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

/// Convert a layout-space rectangle (viewport pixels) to NDC for the render pipeline.
pub fn physical_rect_to_ndc(rect: &Rect, viewport: PhysicalSize<u32>) -> (f32, f32, f32, f32) {
    let w = viewport.width as f32;
    let h = viewport.height as f32;
    let x_ndc = (2.0 * rect.x) / w - 1.0;
    let y_ndc = 1.0 - (2.0 * rect.y) / h;
    let width_ndc = (2.0 * rect.width) / w;
    let height_ndc = (-2.0 * rect.height) / h;
    (x_ndc, y_ndc, width_ndc, height_ndc)
}

pub fn make_rect_mesh_from_physical_rect(
    device: &wgpu::Device,
    rect: &Rect,
    color: Vec4,
    viewport: PhysicalSize<u32>
) -> Mesh {
    let (x, y, width, height) = physical_rect_to_ndc(rect, viewport);
    make_rect_mesh(device, &(x, y, width, height, color))
}

/// Expects coordinates and sizes normalized to NDC (-1.0 to 1.0).
pub fn make_rect_mesh(device: &wgpu::Device, rectangle: &(f32, f32, f32, f32, Vec4)) -> Mesh {
    let (x, y, width, height, color) = *rectangle;

    let vertices = vec![
        Vertex { position: Vec3::new(x, y, 0.0), color },
        Vertex { position: Vec3::new(x + width, y, 0.0), color },
        Vertex { position: Vec3::new(x + width, y + height, 0.0), color },
        Vertex { position: Vec3::new(x, y + height, 0.0), color }
    ];

    let indices: Vec<u16> = vec![0, 2, 1, 2, 0, 3];
    let bytes1 = unsafe {
        std::slice::from_raw_parts(
            vertices.as_ptr() as *const u8,
            vertices.len() * std::mem::size_of::<Vertex>()
        )
    };

    let bytes2 = unsafe {
        std::slice::from_raw_parts(
            indices.as_ptr() as *const u8,
            indices.len() * std::mem::size_of::<u16>()
        )
    };

    let contents = &[bytes1, bytes2].concat();

    let buffer_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Quad buffer"),
        contents,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDEX,
    };

    let buffer = device.create_buffer_init(&buffer_descriptor);

    Mesh { buffer, offset: bytes1.len() as u64 }
}
