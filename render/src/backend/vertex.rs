use glm::{ Vec2, Vec3, Vec4 };

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: Vec3,
    pub color: Vec4,
    pub uv: Vec2,
}

impl Vertex {
    pub fn get_layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [
            wgpu::VertexAttribute;
            3
        ] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4, 2=> Float32x2];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}
