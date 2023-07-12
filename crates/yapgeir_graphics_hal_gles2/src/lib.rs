use std::{default, rc::Rc};

use blit_framebuffer::TextureRenderer;
use buffer::GlesBuffer;
use bytemuck::Pod;
use context::GlesContext;
use derive_more::Deref;
use draw_descriptor::GlesDrawDescriptor;
use frame_buffer::{real_default_framebuffer, GlesFrameBuffer, GlesReadFormat, GlesRenderBuffer};
use shader::GlesShader;
use smart_default::SmartDefault;
use texture::{GlesPixelFormat, GlesTexture};
use uniforms::GlesUniformBuffer;
use yapgeir_graphics_hal::{
    buffer::BufferUsage, frame_buffer::RenderBufferFormat, sampler::Sampler, Graphics,
    WindowBackend,
};

mod blit_framebuffer;
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

#[derive(SmartDefault, Clone)]
pub struct GlesSettings {
    /// If true, the default framebuffer returned by the API will actually be a fake one,
    /// and during swap_buffers a new draw call will be issued to blit the fake framebuffer
    /// onto the screen but inverting the Y axis.
    ///
    /// This is done to conform with the coordinate system of graphics-hal.
    #[default(true)]
    pub flip_default_framebuffer: bool,
}

impl<B: WindowBackend> Gles<B> {
    pub fn new_with_settings(backend: B, settings: GlesSettings) -> Self {
        let ctx = unsafe { Self(Rc::new(GlesContext::new(backend, settings))) };

        let texture_renderer = TextureRenderer::new(&ctx);
        {
            let mut tr = ctx.texture_renderer.borrow_mut();
            *tr = Some(texture_renderer);
        }

        ctx
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
        if let Some(texture_renderer) = self.texture_renderer.borrow().as_ref() {
            if let Some((tex, _)) = self.fake_framebuffer.borrow().as_ref() {
                texture_renderer.render(
                    &real_default_framebuffer(self.clone()),
                    Sampler::nearest(tex),
                    &default::Default::default(),
                );
            }
        }

        let mut ctx = self.get_ref();
        ctx.bind_frame_buffer(None);
        ctx.state.default_frame_buffer_size = None;

        self.backend.swap_buffers();
    }
}
