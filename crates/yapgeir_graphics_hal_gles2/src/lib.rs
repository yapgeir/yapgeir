use std::rc::Rc;

use buffer::GlesBuffer;
use bytemuck::Pod;
use context::GlesContext;
use derive_more::Deref;
use draw_descriptor::GlesDrawDescriptor;
use frame_buffer::{GlesFrameBuffer, GlesReadFormat, GlesRenderBuffer};
use shader::GlesShader;
use texture::{GlesPixelFormat, GlesTexture};
use uniforms::GlesUniformBuffer;
use yapgeir_graphics_hal::{
    buffer::BufferUsage, frame_buffer::RenderBufferFormat, Graphics, WindowBackend,
};

mod buffer;
mod constants;
pub mod context;
pub mod draw_descriptor;
pub mod frame_buffer;
mod samplers;
pub mod shader;
pub mod texture;
pub mod uniforms;

#[derive(Deref)]
pub struct Gles<B: WindowBackend>(pub(crate) Rc<GlesContext<B>>);

impl<B: WindowBackend> Clone for Gles<B> {
    fn clone(&self) -> Self {
        Gles(self.0.clone())
    }
}

impl<B: WindowBackend + 'static> Graphics for Gles<B> {
    type Backend = B;
    type Shader = GlesShader<B>;
    type PixelFormat = GlesPixelFormat;
    type Texture = GlesTexture<B>;
    type RenderBufferFormat = RenderBufferFormat;
    type RenderBuffer = GlesRenderBuffer<B>;
    type ReadFormat = GlesReadFormat;
    type DrawDescriptor = GlesDrawDescriptor<B>;
    type FrameBuffer = GlesFrameBuffer<B>;
    type UniformBuffer<T: Pod> = GlesUniformBuffer<T>;
    type BufferUsage = BufferUsage;
    type ByteBuffer = GlesBuffer<B>;

    fn new(backend: B) -> Self {
        unsafe { Self(Rc::new(GlesContext::new(backend))) }
    }

    fn swap_buffers(&self) {
        self.backend.swap_buffers();
        self.get_ref().state.default_frame_buffer_size = None;
    }
}
