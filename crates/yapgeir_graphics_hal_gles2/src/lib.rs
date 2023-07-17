use std::rc::Rc;

use buffer::GlesBuffer;
use bytemuck::Pod;
use context::GlesContext;
use derive_more::Deref;
use draw_descriptor::GlesDrawDescriptor;
use frame_buffer::GlesFrameBuffer;
use render_buffer::GlesRenderBuffer;
use shader::GlesShader;
use smart_default::SmartDefault;
use texture::GlesTexture;
use uniforms::GlesUniformBuffer;
use yapgeir_graphics_hal::{
    buffer::BufferUsage, render_buffer::RenderBufferFormat, Graphics, WindowBackend,
};

pub use frame_buffer::GlesReadFormat;
/// Re-export extended variants of the default enums
pub use texture::GlesPixelFormat;

mod buffer;
mod constants;
mod context;
mod draw_descriptor;
mod fake_default_framebuffer;
mod frame_buffer;
mod frame_buffer_blitter;
mod render_buffer;
mod samplers;
mod shader;
mod texture;
mod uniforms;

#[derive(Deref)]
pub struct Gles<B: WindowBackend>(pub Rc<GlesContext<B>>);

#[derive(SmartDefault, Clone)]
pub struct GlesSettings {
    /// If true, the default framebuffer returned by the API will actually be a fake one,
    /// and during swap_buffers a new draw call will be issued to blit the fake framebuffer
    /// onto the screen but inverting the Y axis.
    ///
    /// This is done to conform to the coordinate system of graphics-hal, which is
    /// Y up for NDC, and Y down for frame buffers and textures.
    #[default(true)]
    pub flip_default_frame_buffer: bool,
}

impl<B: WindowBackend> Gles<B> {
    pub fn new_with_settings(backend: B, settings: GlesSettings) -> Self {
        Self(Rc::new(unsafe { GlesContext::new(backend, settings) }))
    }
}

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
        Self::new_with_settings(backend, Default::default())
    }

    fn swap_buffers(&self) {
        let mut ctx = self.get_ref();

        if let Some(fake_default_frame_buffer) = &self.fake_default_frame_buffer {
            unsafe {
                fake_default_frame_buffer
                    .borrow()
                    .blit(&mut ctx, &self.frame_buffer_blitter)
            };
        }

        ctx.bind_frame_buffer(None);
        self.default_framebuffer_size.take();
        self.backend.swap_buffers();
    }
}
