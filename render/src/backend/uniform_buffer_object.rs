use super::bind_group;

pub struct UBOGroup {
    pub buffer: wgpu::Buffer,
    pub bind_groups: Vec<Option<wgpu::BindGroup>>,
    allignment: u64,
}

impl UBOGroup {
    pub fn new(device: &wgpu::Device, object_count: usize, layout: wgpu::BindGroupLayout) -> Self {
        let allignment = glm::max(
            device.limits().min_storage_buffer_offset_alignment as u32,
            std::mem::size_of::<glm::Mat4>() as u32
        ) as u64;

        let buffer_descriptor = wgpu::BufferDescriptor {
            label: Some("UBO"),
            size: (object_count as u64) * allignment,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        let buffer = device.create_buffer(&buffer_descriptor);

        let mut bind_groups = Vec::new();
        for i in 0..object_count {
            let mut builder = bind_group::Builder::new(device);
            builder.set_layout(&layout);
            builder.add_buffer(&buffer, (i as u64) * allignment);
            bind_groups.push(Some(builder.build("Matrix")));
        }

        Self { buffer, bind_groups, allignment }
    }

    pub fn upload(&self, i: u64, matrix: &glm::Mat4, queue: &wgpu::Queue) {
        let offset = i * self.allignment;
        let data: &[u8] = unsafe {
            std::slice::from_raw_parts(
                matrix as *const glm::Mat4 as *const u8,
                std::mem::size_of::<glm::Mat4>()
            )
        };

        queue.write_buffer(&self.buffer, offset, data);
    }
}

pub struct UBOStandalone {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl UBOStandalone {
    pub fn new(device: &wgpu::Device, layout: wgpu::BindGroupLayout) -> Self {
        let buffer_descriptor = wgpu::BufferDescriptor {
            label: Some("UBO"),
            size: std::mem::size_of::<glm::Mat4>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        let buffer = device.create_buffer(&buffer_descriptor);

        let mut builder = bind_group::Builder::new(device);
        builder.set_layout(&layout);
        builder.add_buffer(&buffer, 0);
        let bind_group = builder.build("Matrix");

        Self { buffer, bind_group }
    }

    pub fn upload(&self, matrix: &glm::Mat4, queue: &wgpu::Queue) {
        let data: &[u8] = unsafe {
            std::slice::from_raw_parts(
                matrix as *const glm::Mat4 as *const u8,
                std::mem::size_of::<glm::Mat4>()
            )
        };

        queue.write_buffer(&self.buffer, 0, data);
    }
}
