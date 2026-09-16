use swash::scale::image::Image;

use super::bind_group;

pub struct TriangleMaterial(pub Material);
pub struct QuadMaterial(pub Material);

pub struct Material {
    pub bind_group: wgpu::BindGroup,
}

impl Material {
    pub fn default(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: &str,
        layout: &wgpu::BindGroupLayout,
        rgba: [u8; 4]
    ) -> Self {
        // 1. Create a 1x1 pixel-sized texture with the specified color.
        let texture_size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };
        let texture_descriptor = wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some(label),
            view_formats: &[wgpu::TextureFormat::Bgra8UnormSrgb],
        };
        let texture = device.create_texture(&texture_descriptor);

        // 2. Upload the RGBA color data to the texture.
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba, // Single RGBA pixel
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4), // 4 bytes for 1 pixel per row
                rows_per_image: Some(1),
            },
            texture_size
        );

        // 3. Create a texture view for sampling.
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // 4. Create a simple sampler for the texture.
        let sampler_descriptor = wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        };
        let sampler = device.create_sampler(&sampler_descriptor);

        // 5. Create a bind group from the texture and sampler.
        let mut builder = bind_group::Builder::new(device);
        builder.set_layout(layout);
        builder.add_material(&view, &sampler);
        let bind_group = builder.build(label);

        // 6. Return the material.
        Material { bind_group }
    }

    pub fn new(
        image: &Image,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: &str,
        layout: &wgpu::BindGroupLayout
    ) -> Self {
        // Get absolute filepath from relative one
        let texture_size = wgpu::Extent3d {
            width: image.placement.width,
            height: image.placement.height,
            depth_or_array_layers: 1,
        };

        // Create the texture
        let texture_descriptor = wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some(label),
            view_formats: &[wgpu::TextureFormat::Bgra8UnormSrgb],
        };
        let texture = device.create_texture(&texture_descriptor);

        // Upload to it
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image.data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * image.placement.width),
                rows_per_image: Some(image.placement.height),
            },
            texture_size
        );

        // Get a view of the texture
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Make a sampler
        let sampler_descriptor = wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        };
        let sampler = device.create_sampler(&sampler_descriptor);

        // Make a bind group for everything
        let mut builder = bind_group::Builder::new(device);
        builder.set_layout(layout);
        builder.add_material(&view, &sampler);
        let bind_group = builder.build(label);

        Material { bind_group: bind_group }
    }
}
