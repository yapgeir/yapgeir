use std::cell::{Cell, RefCell};

use glow::HasContext;
use yapgeir_graphics_hal::{
    buffer::{BufferKind, BufferUsage},
    frame_buffer::RenderBufferFormat,
    index_buffer::PrimitiveMode,
    shader::TextShaderSource,
    ImageSize, Rgba,
};

use crate::{constants::GlConstant, context::GlesContextRef, shader::compile_program};
#[cfg(not(target_os = "vita"))]
const SHADER: TextShaderSource = TextShaderSource {
    vertex: r#"
        #version 120

        attribute vec2 position;
        varying vec2 v_tex_position;

        void main() {
            v_tex_position = (position + 1) * 0.5;
            v_tex_position.y = 1 - v_tex_position.y;
            gl_Position = vec4(position, 1, 1);
        }
    "#,
    fragment: r#"
        #version 120

        uniform sampler2D tex;

        varying vec2 v_tex_position;
        void main() {
            gl_FragColor = texture2D(tex, v_tex_position);
        }
    "#,
};

#[cfg(target_os = "vita")]
const SHADER: TextShaderSource = TextShaderSource {
    vertex: r#"
        void main(
            float2 position,
            float2 out v_tex_position: TEXCOORD0,
            float4 out gl_Position : POSITION
        ) {
            v_tex_position = (position + 1) * 0.5;
            gl_Position = float4(position, 1, 1);
        }
    "#,
    fragment: r#"
        uniform sampler2D tex: TEXUNIT0;

        float4 main(float2 v_tex_position: TEXCOORD0) {
            return tex2D(tex, v_tex_position);
        }
    "#,
};

pub struct FakeFramebuffer {
    pub framebuffer: glow::Framebuffer,
    pub draw_texture: glow::Texture,
    pub depth_stencil: glow::Renderbuffer,
}

impl FakeFramebuffer {
    pub unsafe fn new(ctx: &mut GlesContextRef, size: ImageSize<u32>) -> Self {
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
            framebuffer,
            draw_texture,
            depth_stencil,
        }
    }

    // To prevent recursive objects, cleanup is manual
    pub unsafe fn destroy(&self, gl: &glow::Context) {
        gl.delete_framebuffer(self.framebuffer);
        gl.delete_renderbuffer(self.depth_stencil);
        gl.delete_texture(self.draw_texture);
    }
}

pub struct ScreenFlipper {
    pub size: Cell<ImageSize<u32>>,

    pub framebuffer: RefCell<FakeFramebuffer>,
    pub vertex_buffer: glow::Buffer,
    pub program: glow::Program,

    vertex_attrib_location: u32,
}

impl ScreenFlipper {
    pub unsafe fn new(ctx: &mut GlesContextRef, size: ImageSize<u32>) -> Self {
        let framebuffer = FakeFramebuffer::new(ctx, size);

        let vertex_buffer = ctx.gl.create_buffer().expect("Unable to create buffer.");
        ctx.bind_buffer(BufferKind::Vertex, Some(vertex_buffer));
        ctx.gl.buffer_data_u8_slice(
            BufferKind::Vertex.gl_const(),
            bytemuck::cast_slice(&[[-1f32, -1f32], [-1., 1.], [1., 1.], [1., -1.]]),
            BufferUsage::Static.gl_const(),
        );

        let program = compile_program(&ctx.gl, &SHADER);
        let uniform_location = ctx
            .gl
            .get_uniform_location(program, "tex")
            .expect("Uniform tex not found!");

        ctx.use_program(Some(program));
        ctx.gl
            .uniform_1_i32(Some(&uniform_location), ctx.state.texture_unit_limit as i32);

        let vertex_attrib_location = ctx
            .gl
            .get_attrib_location(program, "position")
            .expect("attribute location not found");

        Self {
            size: Cell::new(size),
            framebuffer: RefCell::new(framebuffer),
            vertex_buffer,
            program,
            vertex_attrib_location,
        }
    }

    pub unsafe fn framebuffer(
        &self,
        ctx: &mut GlesContextRef,
        size: ImageSize<u32>,
    ) -> glow::Framebuffer {
        let mut fb = self.framebuffer.borrow_mut();
        if size != self.size.get() {
            self.size.set(size);
            fb.destroy(&ctx.gl);
            *fb = FakeFramebuffer::new(ctx, size);
        }

        fb.framebuffer
    }

    pub unsafe fn blit(&self, ctx: &mut GlesContextRef) {
        ctx.bind_frame_buffer(None);
        ctx.use_program(Some(self.program));

        if ctx.extensions.vertex_array_objects {
            ctx.bind_vertex_array(None);
        }

        ctx.state.draw_descriptor_cache.current = 0;
        ctx.bind_buffer(BufferKind::Vertex, Some(self.vertex_buffer));

        ctx.gl
            .enable_vertex_attrib_array(self.vertex_attrib_location);
        ctx.gl
            .vertex_attrib_pointer_f32(self.vertex_attrib_location, 2, glow::FLOAT, false, 8, 0);

        ctx.set_blend(None);
        ctx.set_color_mask(Rgba::all(true));
        ctx.set_cull_face(None);
        ctx.set_depth(None);
        ctx.set_stencil(None);
        ctx.set_scissor(None);
        ctx.set_viewport(self.size.get().into());
        ctx.set_dithering(false);

        ctx.gl
            .draw_arrays(PrimitiveMode::TriangleFan.gl_const(), 0, 4);
    }

    pub unsafe fn drop_all(&self, gl: &glow::Context) {
        self.framebuffer.borrow().destroy(gl);

        gl.use_program(None);
        gl.bind_buffer(glow::ARRAY_BUFFER, None);

        gl.delete_buffer(self.vertex_buffer);
        gl.delete_program(self.program);
    }
}
