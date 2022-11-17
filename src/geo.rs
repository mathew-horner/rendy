pub type Vector<T, const N: usize> = [T; N];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: Vector<f32, 3>,
    pub tex_coords: Vector<f32, 2>,
}

impl Vertex {
    pub const fn new(position: Vector<f32, 3>, tex_coords: Vector<f32, 2>) -> Self {
        Self {
            position,
            tex_coords: [tex_coords[0], 1.0 - tex_coords[1]],
        }
    }

    pub fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
