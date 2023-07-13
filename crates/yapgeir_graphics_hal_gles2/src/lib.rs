use std::rc::Rc;

use buffer::GlesBuffer;
use bytemuck::Pod;
use context::GlesContext;
use derive_more::Deref;
use draw_descriptor::GlesDrawDescriptor;
use fake_default_framebuffer::ScreenFlipper;
use frame_buffer::{GlesFrameBuffer, GlesRenderBuffer};
use shader::GlesShader;
use smart_default::SmartDefault;
use texture::GlesTexture;
use uniforms::GlesUniformBuffer;
use yapgeir_graphics_hal::{
    buffer::BufferUsage, frame_buffer::RenderBufferFormat, Graphics, WindowBackend,
};

/// Re-export extended variants of the default enums
pub use texture::GlesPixelFormat;
pub use frame_buffer::GlesReadFormat;

mod buffer;
mod constants;
mod context;
mod draw_descriptor;
mod fake_default_framebuffer;
mod frame_buffer;
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
    pub flip_default_framebuffer: bool,
}

impl<B: WindowBackend> Gles<B> {
    pub fn new_with_settings(backend: B, settings: GlesSettings) -> Self {
        let flip_default_framebuffer = settings.flip_default_framebuffer;
        let mut ctx = unsafe { GlesContext::new(backend, settings) };

        if flip_default_framebuffer {
            let screen_flipper = unsafe { ScreenFlipper::new(&mut ctx.get_ref(), ctx.default_framebuffer_size()) };
            ctx.screen_flipper = Some(screen_flipper);
        }

        Self(Rc::new(ctx))
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

        if let Some(screen_flipper) = &self.screen_flipper {
            unsafe { screen_flipper.blit(&mut ctx) };
        }

        ctx.bind_frame_buffer(None);
        self.default_framebuffer_size.take();
        self.backend.swap_buffers();
    }
}
