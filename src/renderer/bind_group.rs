use std::io::Read;
use std::{fs::File, path::Path};

use wgpu::{BindGroup, BindGroupLayout, Device, Queue};

use crate::texture::ImageTexture;

/// Arguments for calls to `create_bind_group_*` functions.
pub struct CreateBindGroupArgs<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,
}

/// Stores a [`BindGroup`] with its associated [`BindGroupLayout`] together.
pub struct BindGroupWithLayout {
    pub actual: BindGroup,
    pub layout: BindGroupLayout,
}

/// Creates a new [`BindGroupWithLayout`] based on the image texture at the given `path`.
pub fn create_bind_group_for_image_texture_at_path(
    path: impl AsRef<Path>,
    args: CreateBindGroupArgs<'_>,
) -> BindGroupWithLayout {
    let mut file = File::open(path).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let texture = ImageTexture::from_bytes(buffer.as_slice()).write(args.device, args.queue);
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let sampler = args.device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let layout = args
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

    let bind_group = args.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
        label: Some("diffuse_bind_group"),
    });

    BindGroupWithLayout {
        actual: bind_group,
        layout,
    }
}

/// Creates a new [`BindGroupWithLayout`] for a uniform buffer.
pub fn create_bind_group_for_uniform_buffer(uniform_buffer: &wgpu::Buffer, args: CreateBindGroupArgs) -> BindGroupWithLayout {
    let layout = args.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }
        ],
        label: Some("Uniform Buffer Bind Group Layout"),
    });

    let bind_group = args.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Uniform Buffer Bind Group"),
        layout: &layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }]
    });

    BindGroupWithLayout {
        actual: bind_group,
        layout,
    }
}