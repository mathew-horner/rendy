mod bind_group;

use std::io::Read;
use std::path::Path;

use wgpu::util::DeviceExt;
use wgpu::{
    BindGroup, Buffer, Device, Instance, Queue, RenderPipeline, Surface, SurfaceConfiguration,
};
use winit::dpi::PhysicalSize;
use winit::window::Window;
use crate::camera::{Camera, CameraUniform};

use self::bind_group::{create_bind_group_for_image_texture_at_path, CreateBindGroupArgs};
use crate::geo::Vertex;
use crate::renderer::bind_group::create_bind_group_for_uniform_buffer;

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
    camera: Camera,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    camera_uniform: CameraUniform,
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

        let background_color = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };

        let camera = Camera {
            // position the camera one unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 1.0, 2.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.set_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = create_bind_group_for_uniform_buffer(&camera_buffer, CreateBindGroupArgs { device: &device, queue: &queue });

        let texture_bind_group = create_bind_group_for_image_texture_at_path(
            texture_path,
            CreateBindGroupArgs {
                device: &device,
                queue: &queue,
            },
        );

        let mut file = std::fs::File::open(shader_path).unwrap();
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader Module"),
            source: wgpu::ShaderSource::Wgsl(buffer.into()),
        });

        let pipeline_layout = device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group.layout, &camera_bind_group.layout],
                push_constant_ranges: &[],
            });

        let pipeline = device
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
                        format: config.format,
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
            diffuse_bind_group: texture_bind_group.actual,
            background_color,
            camera,
            camera_buffer,
            camera_bind_group: camera_bind_group.actual,
            camera_uniform,
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

    pub fn move_camera(&mut self, vector: cgmath::Vector3<f32>) {
        self.camera.eye.x += 1.0;
        // self.camera.eye += vector;
        self.camera_uniform.set_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
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
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.indices, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
