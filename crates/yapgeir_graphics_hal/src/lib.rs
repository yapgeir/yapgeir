use std::{ffi::c_void, rc::Rc};

use buffer::{Buffer, BufferData, BufferKind, BufferUsage, ByteBuffer};
use bytemuck::Pod;
use draw_descriptor::{DrawDescriptor, IndexBinding, VertexBindings};
use frame_buffer::{
    DepthStencilAttachment, FrameBuffer, ReadFormat, RenderBuffer, RenderBufferFormat,
};
use shader::{Shader, TextShaderSource};
use texture::{PixelFormat, Texture};
use uniforms::{UniformBuffer, Uniforms};

pub use primitives::*;

pub mod buffer;
pub mod draw_descriptor;
pub mod draw_params;
pub mod frame_buffer;
pub mod index_buffer;
mod primitives;
pub mod sampler;
pub mod samplers;
pub mod shader;
pub mod texture;
pub mod uniforms;
pub mod vertex_buffer;

pub trait WindowBackend
where
    Self: 'static,
{
    fn swap_buffers(&self);
    fn get_proc_address(&self, symbol: &str) -> *const c_void;
    fn default_framebuffer_size(&self) -> ImageSize<u32>;
}

pub trait Graphics
where
    Self: Sized + Clone + 'static,
{
    type Backend: WindowBackend;
    type Shader: Shader<Self>;
    type PixelFormat: From<PixelFormat>;
    type Texture: Texture<Self, PixelFormat = Self::PixelFormat>;
    type RenderBufferFormat: From<RenderBufferFormat>;
    type RenderBuffer: RenderBuffer<Self, Format = Self::RenderBufferFormat>;
    type ReadFormat: From<ReadFormat>;
    type DrawDescriptor: DrawDescriptor<Self>;
    type FrameBuffer: FrameBuffer<Self, ReadFormat = Self::ReadFormat>;
    type BufferUsage: From<BufferUsage>;
    type ByteBuffer: ByteBuffer<Self, Usage = Self::BufferUsage>;
    type UniformBuffer<T: Pod>: UniformBuffer<Self, T>;

    fn new(backend: Self::Backend) -> Self;

    fn default_framebuffer(&self) -> Self::FrameBuffer {
        Self::FrameBuffer::default(self.clone())
    }

    fn new_shader<'a>(&self, source: &TextShaderSource) -> Self::Shader
    where
        Self: 'a,
    {
        Self::Shader::new(self.clone(), source)
    }

    fn new_buffer<'a, T: Pod>(
        &self,
        kind: BufferKind,
        usage: impl Into<Self::BufferUsage>,
        data: impl Into<BufferData<'a, T>>,
    ) -> Buffer<Self, T> {
        Buffer::new(self.clone(), kind, usage.into(), data.into())
    }

    fn new_draw_descriptor<'a>(
        &self,
        shader: Rc<Self::Shader>,
        indices: impl Into<IndexBinding<Self>>,
        vertices: impl AsRef<[VertexBindings<'a, Self>]>,
    ) -> Self::DrawDescriptor
    where
        Self: 'a,
    {
        Self::DrawDescriptor::new(self.clone(), shader, indices.into(), vertices.as_ref())
    }

    fn new_texture<'a>(
        &self,
        format: impl Into<Self::PixelFormat>,
        size: impl Into<ImageSize<u32>>,
        bytes: Option<&'a [u8]>,
    ) -> Self::Texture {
        Self::Texture::new(self.clone(), format.into(), size.into(), bytes.into())
    }

    fn new_render_buffer(
        &self,
        size: impl Into<ImageSize<u32>>,
        format: impl Into<Self::RenderBufferFormat>,
    ) -> Self::RenderBuffer {
        Self::RenderBuffer::new(self.clone(), size.into(), format.into())
    }

    fn new_frame_buffer(
        &self,
        draw: Rc<Self::Texture>,
        depth_stencil: impl Into<DepthStencilAttachment<Self>>,
    ) -> Self::FrameBuffer {
        Self::FrameBuffer::new(self.clone(), draw, depth_stencil.into())
    }

    fn new_uniform_buffer<'a, T: Uniforms + Pod>(&self, initial: &T) -> Self::UniformBuffer<T> {
        Self::UniformBuffer::new(self.clone(), initial)
    }

    fn swap_buffers(&self);
}
