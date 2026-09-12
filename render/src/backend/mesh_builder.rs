use glm::{ Vec3, Vec4 };
use wgpu::util::DeviceExt;

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
        // Each vertex contains position (location 0) and color (location 1), both as 3 floats.
        const ATTRIBUTES: [
            wgpu::VertexAttribute;
            2
        ] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}
pub fn make_rect_mesh(device: &wgpu::Device, rectangles: &[(f32, f32, f32, f32, Vec4)]) -> Mesh {
    let (x, y, width, height, color) = rectangles[0];

    let vertices = vec![
        Vertex { position: Vec3::new(x, y, 0.0), color },
        Vertex { position: Vec3::new(x + width, y, 0.0), color },
        Vertex { position: Vec3::new(x + width, y + height, 0.0), color },
        Vertex { position: Vec3::new(x, y + height, 0.0), color }
    ];
    println!("{vertices:?}");

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
