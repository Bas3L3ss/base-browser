use glm::Vec3;
use wgpu::util::DeviceExt;

pub struct TriangleMesh(pub Mesh);
pub struct QuadMesh(pub Mesh);

#[repr(C)] // C-style data layout
pub struct Vertex {
    position: Vec3,
    color: Vec3,
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
        ] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

pub fn make_triangle(device: &wgpu::Device) -> Mesh {
    // Vertices for a single triangle
    let vertices: Vec<Vertex> = vec![
        Vertex { position: Vec3::new(-0.75, -0.75, 0.0), color: Vec3::new(0.0, 1.0, 0.0) },
        Vertex { position: Vec3::new(0.75, -0.75, 0.0), color: Vec3::new(0.0, 1.0, 0.0) },
        Vertex { position: Vec3::new(0.0, 0.75, 0.0), color: Vec3::new(0.0, 1.0, 0.0) }
    ];

    let bytes1 = unsafe {
        std::slice::from_raw_parts(
            vertices.as_ptr() as *const u8,
            vertices.len() * std::mem::size_of::<Vertex>()
        )
    };

    // Indices for the triangle (just one triangle)
    let indices: Vec<u16> = vec![0, 1, 2];

    let bytes2 = unsafe {
        std::slice::from_raw_parts(
            indices.as_ptr() as *const u8,
            indices.len() * std::mem::size_of::<u16>()
        )
    };

    let contents = &[bytes1, bytes2].concat();

    let buffer_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Triangle buffer"),
        contents,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDEX,
    };

    let buffer = device.create_buffer_init(&buffer_descriptor);

    Mesh { buffer, offset: bytes1.len() as u64 }
}

pub fn make_quad(device: &wgpu::Device) -> Mesh {
    // Vertices for two triangles forming a square
    let vertices: Vec<Vertex> = vec![
        Vertex { position: Vec3::new(-0.75, -0.75, 0.0), color: Vec3::new(1.0, 0.0, 0.0) },
        Vertex { position: Vec3::new(0.75, -0.75, 0.0), color: Vec3::new(1.0, 0.0, 0.0) },
        Vertex { position: Vec3::new(0.75, 0.75, 0.0), color: Vec3::new(1.0, 0.0, 0.0) },
        Vertex { position: Vec3::new(-0.75, 0.75, 0.0), color: Vec3::new(1.0, 0.0, 0.0) }
    ];
    // vertex_count = buffer.size() as usize / std::mem::size_of::<Vertex>();
    let bytes1 = unsafe {
        std::slice::from_raw_parts(
            vertices.as_ptr() as *const u8,
            vertices.len() * std::mem::size_of::<Vertex>()
        )
    };

    let indices: Vec<u16> = vec![0, 1, 2, 2, 3, 0];

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
