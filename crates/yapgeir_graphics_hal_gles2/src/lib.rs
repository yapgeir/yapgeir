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
    buffer::BufferUsage, frame_buffer::RenderBufferFormat, Backend, Graphics,
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
pub struct Gles<B: Backend>(pub(crate) Rc<GlesContext<B>>);

impl<B: Backend> Clone for Gles<B> {
    fn clone(&self) -> Self {
        Gles(self.0.clone())
    }
}

impl<B: Backend> Gles<B> {
    pub fn new(backend: B) -> Self {
        unsafe { Self(Rc::new(GlesContext::new(backend))) }
    }
}

impl<B: Backend + 'static> Graphics for Gles<B> {
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

    fn swap_buffers(&self) {
        self.backend.swap_buffers();
        self.get_ref().state.default_frame_buffer_size = None;
    }
}
