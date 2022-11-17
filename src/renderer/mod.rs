mod bind_group;
mod pipeline;

use std::path::Path;

use wgpu::util::DeviceExt;
use wgpu::{
    BindGroup, Buffer, Device, Instance, Queue, RenderPipeline, Surface, SurfaceConfiguration,
};
use winit::dpi::PhysicalSize;
use winit::window::Window;

use self::bind_group::{create_bind_group_for_image_texture_at_path, CreateBindGroupArgs};
use self::pipeline::{create_render_pipeline_for_shader_at_path, CreateRenderPipelineArgs};
use crate::geo::Vertex;

const VERTICES: &[Vertex] = &[
    Vertex::new([0.5, 0.5, 0.0], [1.0, 1.0]),
    Vertex::new([-0.5, 0.5, 0.0], [0.0, 1.0]),
    Vertex::new([-0.5, -0.5, 0.0], [0.0, 0.0]),
    Vertex::new([0.5, -0.5, 0.0], [1.0, 0.0]),
];

const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

pub struct Renderer {
    surface: Surface,
    device: Device,
    queue: Queue,
    size: PhysicalSize<u32>,
    config: SurfaceConfiguration,
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    indices: u32,
    diffuse_bind_group: BindGroup,
    background_color: wgpu::Color,
}

impl Renderer {
    pub async fn new(
        window: &Window,
        texture_path: impl AsRef<Path>,
        shader_path: impl AsRef<Path>,
    ) -> Self {
        let instance = Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let size = window.inner_size();
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };

        surface.configure(&device, &config);

        let bind_group = create_bind_group_for_image_texture_at_path(
            texture_path,
            CreateBindGroupArgs {
                device: &device,
                queue: &queue,
            },
        );

        let pipeline = create_render_pipeline_for_shader_at_path(
            shader_path,
            CreateRenderPipelineArgs {
                device: &device,
                bind_group_layout: &bind_group.layout,
                surface_config: &config,
            },
        );

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let background_color = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };

        Self {
            surface,
            device,
            queue,
            size,
            config,
            pipeline,
            vertex_buffer,
            index_buffer,
            indices: INDICES.len() as u32,
            diffuse_bind_group: bind_group.actual,
            background_color,
        }
    }

    pub fn set_texture(&mut self, path: impl AsRef<Path>) {
        self.diffuse_bind_group = create_bind_group_for_image_texture_at_path(
            path,
            CreateBindGroupArgs {
                device: &self.device,
                queue: &self.queue,
            },
        )
        .actual;
    }

    pub fn set_background_color(&mut self, color: wgpu::Color) {
        self.background_color = color;
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.size = size;
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn draw(&self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.background_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.indices, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
