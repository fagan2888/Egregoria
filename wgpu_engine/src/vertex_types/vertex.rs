use crate::VBDesc;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ColoredVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

u8slice_impl!(ColoredVertex);

impl VBDesc for ColoredVertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<ColoredVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: Box::leak(Box::new(wgpu::vertex_attr_array![0 => Float3, 1 => Float4])),
        }
    }
}
