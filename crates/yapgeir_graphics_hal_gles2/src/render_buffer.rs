use glow::HasContext;
use yapgeir_graphics_hal::{
    render_buffer::{RenderBuffer, RenderBufferFormat},
    Size, WindowBackend,
};

use crate::{constants::GlConstant, Gles};

pub struct GlesRenderBuffer<B: WindowBackend> {
    pub ctx: Gles<B>,
    pub renderbuffer: glow::Renderbuffer,
}

impl<B: WindowBackend> RenderBuffer<Gles<B>> for GlesRenderBuffer<B> {
    type Format = RenderBufferFormat;

    fn new(ctx: Gles<B>, size: Size<u32>, format: RenderBufferFormat) -> Self {
        let format = format.gl_const();

        let renderbuffer = unsafe {
            let mut ctx = ctx.get_ref();
            let rb = ctx
                .gl
                .create_renderbuffer()
                .expect("unable to create a renderbuffer");
            ctx.bind_render_buffer(Some(rb));
            ctx.gl
                .renderbuffer_storage(glow::RENDERBUFFER, format, size.w as i32, size.h as i32);

            rb
        };

        Self { ctx, renderbuffer }
    }
}

impl<B: WindowBackend> Drop for GlesRenderBuffer<B> {
    fn drop(&mut self) {
        unsafe {
            let mut ctx = self.ctx.get_ref();
            if ctx.state.bound_render_buffer == Some(self.renderbuffer) {
                ctx.bind_render_buffer(None);
            }
            ctx.gl.delete_renderbuffer(self.renderbuffer);
        }
    }
}
