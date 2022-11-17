use std::{fs::File, io::Read, path::Path};

use wgpu::{
    BindGroupLayout, Device, RenderPipeline, ShaderModuleDescriptor, ShaderSource,
    SurfaceConfiguration,
};

use crate::geo::Vertex;

pub struct CreateRenderPipelineArgs<'a> {
    pub device: &'a Device,
    pub bind_group_layout: &'a BindGroupLayout,
    pub surface_config: &'a SurfaceConfiguration,
}

pub fn create_render_pipeline_for_shader_at_path(
    path: impl AsRef<Path>,
    args: CreateRenderPipelineArgs<'_>,
) -> RenderPipeline {
    let mut file = File::open(path).unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).unwrap();

    let shader = args.device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Shader Module"),
        source: ShaderSource::Wgsl(buffer.into()),
    });

    let pipeline_layout = args
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[args.bind_group_layout],
            push_constant_ranges: &[],
        });

    let pipeline = args
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: args.surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

    pipeline
}
