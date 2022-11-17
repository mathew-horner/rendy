use image::{DynamicImage, GenericImageView};

pub struct ImageTexture {
    image: DynamicImage,
}

impl ImageTexture {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            image: image::load_from_memory(bytes).unwrap(),
        }
    }

    pub fn size(&self) -> wgpu::Extent3d {
        let dimensions = self.dimensions();
        wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.image.dimensions()
    }

    pub fn write(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        let dimensions = self.dimensions();
        let size = self.size();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.image.to_rgba8(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
                rows_per_image: std::num::NonZeroU32::new(dimensions.1),
            },
            size,
        );

        texture
    }
}
