use glow::HasContext;
use yapgeir_graphics_hal::{render_buffer::RenderBufferFormat, sampler::Filter, Size};

use crate::{
    constants::GlConstant,
    context::GlesContextRef,
    frame_buffer_blitter::{BlitSourceRect, FrameBufferBlitter, ReadSource},
};

pub struct FakeDefaultFrameBuffer {
    pub size: Size<u32>,
    pub framebuffer: glow::Framebuffer,
    pub draw_texture: glow::Texture,
    pub depth_stencil: glow::Renderbuffer,
}

impl FakeDefaultFrameBuffer {
    pub unsafe fn new(ctx: &mut GlesContextRef, size: Size<u32>) -> Self {
        // Create a new draw texture
        let draw_texture = ctx.gl.create_texture().expect("unable to create a texture");
        ctx.activate_texture_unit(ctx.state.texture_unit_limit as u32);
        ctx.gl.bind_texture(glow::TEXTURE_2D, Some(draw_texture));
        ctx.gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA as i32,
            size.w as i32,
            size.h as i32,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            None,
        );
        ctx.gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_S,
            glow::CLAMP_TO_EDGE as i32,
        );
        ctx.gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_T,
            glow::CLAMP_TO_EDGE as i32,
        );
        ctx.gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::NEAREST as i32,
        );
        ctx.gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::NEAREST as i32,
        );

        // Create a new renderbuffer for depth/stencil
        let depth_stencil = ctx
            .gl
            .create_renderbuffer()
            .expect("unable to create a renderbuffer");
        ctx.bind_render_buffer(Some(depth_stencil));

        ctx.gl.renderbuffer_storage(
            glow::RENDERBUFFER,
            RenderBufferFormat::DepthStencil.gl_const(),
            size.w as i32,
            size.h as i32,
        );

        // Create a new framebuffer and bind both attachments to it
        let framebuffer = ctx
            .gl
            .create_framebuffer()
            .expect("unable to create a framebuffer");
        ctx.bind_frame_buffer(Some(framebuffer));

        ctx.gl.framebuffer_texture_2d(
            glow::FRAMEBUFFER,
            glow::COLOR_ATTACHMENT0,
            glow::TEXTURE_2D,
            Some(draw_texture),
            0,
        );
        ctx.gl.framebuffer_renderbuffer(
            glow::FRAMEBUFFER,
            glow::DEPTH_STENCIL_ATTACHMENT,
            glow::RENDERBUFFER,
            Some(depth_stencil),
        );

        Self {
            size,
            framebuffer,
            draw_texture,
            depth_stencil,
        }
    }

    pub unsafe fn framebuffer(
        &mut self,
        ctx: &mut GlesContextRef,
        size: Size<u32>,
    ) -> glow::Framebuffer {
        if size != self.size {
            self.size = size;
            self.destroy(&ctx.gl);
            *self = FakeDefaultFrameBuffer::new(ctx, size);
        }

        self.framebuffer
    }

    pub unsafe fn blit(&self, ctx: &mut GlesContextRef, blitter: &FrameBufferBlitter) {
        blitter.blit(
            ctx,
            None,
            (
                self.size,
                self.framebuffer,
                ReadSource::Unit(ctx.state.texture_unit_limit),
            ),
            BlitSourceRect::FullFlipY,
            self.size.into(),
            Filter::Nearest,
        );
    }

    // To prevent recursive objects, cleanup is manual
    pub unsafe fn destroy(&self, gl: &glow::Context) {
        gl.delete_framebuffer(self.framebuffer);
        gl.delete_renderbuffer(self.depth_stencil);
        gl.delete_texture(self.draw_texture);
    }
}
