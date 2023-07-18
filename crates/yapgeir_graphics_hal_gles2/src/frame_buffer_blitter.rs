use std::cell::Cell;

use glow::HasContext;
use yapgeir_graphics_hal::{
    buffer::{BufferKind, BufferUsage},
    frame_buffer::FlipSource,
    index_buffer::PrimitiveMode,
    sampler::{Filter, SamplerState},
    shader::TextShaderSource,
    Box2D, Rect, Rgba, Size,
};

use crate::{constants::GlConstant, context::GlesContextRef, shader::compile_program};

unsafe fn bind_texture(
    ctx: &mut GlesContextRef,
    current_unit: usize,
    texture: glow::Texture,
    filter: Filter,
) -> usize {
    let sampler = match filter {
        Filter::Linear => SamplerState::linear(),
        Filter::Nearest => SamplerState::nearest(),
    };

    let current = &ctx.state.texture_units[current_unit];

    if current.texture == Some(texture) {
        if current.sampler == sampler {
            return current_unit;
        }

        if !ctx.extensions.sampler_objects {
            ctx.bind_sampler(current_unit as u32, sampler);
            return current_unit;
        }
    }

    // Perhaps the texture is already bound to another unit
    let mut empty_unit = None;
    for unit in 0..ctx.state.texture_unit_limit {
        let tex_unit = &ctx.state.texture_units[unit];
        if empty_unit == None && tex_unit.texture.is_none() {
            empty_unit = Some(unit);
            continue;
        }

        if tex_unit.texture == Some(texture) {
            if tex_unit.sampler == sampler {
                return unit;
            }
            if !ctx.extensions.sampler_objects {
                ctx.bind_sampler(unit as u32, sampler);
                return unit;
            }
        }
    }

    // Use first empty unit or a zero one.
    let unit = empty_unit.unwrap_or(0);
    ctx.bind_texture(unit as u32, Some(texture));
    ctx.bind_sampler(unit as u32, sampler);
    unit
}

#[cfg(not(target_os = "vita"))]
const SHADER: TextShaderSource = TextShaderSource {
    vertex: r#"
        // #version 100

        uniform vec2 uv[4];
        uniform vec2 tex_pos[4];

        attribute float f_index;
        varying vec2 v_tex_position;

        void main() {
            int index = int(f_index);
            v_tex_position = tex_pos[index];
            gl_Position = vec4(uv[index], 1, 1);
        }
    "#,
    fragment: r#"
        // #version 100
        precision highp float;

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
        uniform float2 uv[4];
        uniform float2 tex_pos[4];

        void main(
            float f_index,
            float2 out v_tex_position: TEXCOORD0,
            float4 out gl_Position : POSITION
        ) {
            int index = int(f_index);
            v_tex_position = tex_pos[index];
            gl_Position = vec4(uv[index], 1, 1);
        }
    "#,
    fragment: r#"
        uniform sampler2D tex: TEXUNIT0;

        float4 main(float2 v_tex_position: TEXCOORD0) {
            return tex2D(tex, v_tex_position);
        }
    "#,
};

pub struct FallbackFramebufferBlitter {
    program: glow::Program,
    vertex_buffer: glow::Buffer,

    vertex_attrib_location: u32,
    tex_location: glow::UniformLocation,
    tex_pos_location: glow::UniformLocation,

    current_texture_unit: Cell<usize>,
    current_tex_coords: Cell<[[f32; 2]; 4]>,
}

pub enum ReadSource {
    Texture(glow::Texture),
    Unit(usize),
}

pub enum BlitSourceRect {
    Pixel(Rect<u32>, FlipSource),
    FullFlipY,
}

impl FallbackFramebufferBlitter {
    pub unsafe fn new(ctx: &mut GlesContextRef) -> Self {
        let vertex_buffer = ctx.gl.create_buffer().expect("Unable to create buffer.");
        ctx.bind_buffer(BufferKind::Vertex, Some(vertex_buffer));
        ctx.gl.buffer_data_u8_slice(
            BufferKind::Vertex.gl_const(),
            bytemuck::cast_slice(&[0f32, 1f32, 2f32, 3f32]),
            BufferUsage::Static.gl_const(),
        );

        let program = compile_program(&ctx.gl, &SHADER);
        ctx.use_program(Some(program));

        let uv_location = ctx
            .gl
            .get_uniform_location(program, "uv")
            .expect("Uniform tex not found!");

        ctx.gl.uniform_2_f32_slice(
            Some(&uv_location),
            &[-1f32, -1f32, -1., 1., 1., 1., 1., -1.],
        );
        let tex_location = ctx
            .gl
            .get_uniform_location(program, "tex")
            .expect("Uniform tex not found!");

        ctx.gl
            .uniform_1_i32(Some(&tex_location), ctx.state.texture_unit_limit as i32);

        let tex_pos_location = ctx
            .gl
            .get_uniform_location(program, "tex_pos")
            .expect("Uniform tex not found!");

        let vertex_attrib_location = ctx
            .gl
            .get_attrib_location(program, "f_index")
            .expect("attribute location not found");

        Self {
            program,
            vertex_attrib_location,
            vertex_buffer,
            tex_location,
            tex_pos_location,
            current_texture_unit: Cell::new(ctx.state.texture_unit_limit),
            current_tex_coords: Default::default(),
        }
    }

    pub unsafe fn blit(
        &self,
        ctx: &mut GlesContextRef,

        frame_buffer: Option<glow::Framebuffer>,
        texture_unit: usize,

        tex_coords: [[f32; 2]; 4],
        viewport: Rect<u32>,
    ) {
        ctx.bind_frame_buffer(frame_buffer);
        ctx.use_program(Some(self.program));

        if self.current_texture_unit.get() != texture_unit {
            ctx.gl
                .uniform_1_i32(Some(&self.tex_location), texture_unit as i32);
            self.current_texture_unit.set(texture_unit);
        }

        if self.current_tex_coords.get() != tex_coords {
            ctx.gl.uniform_2_f32_slice(
                Some(&self.tex_pos_location),
                &bytemuck::cast_slice(&tex_coords),
            );

            self.current_tex_coords.set(tex_coords);
        }

        if ctx.extensions.vertex_array_objects {
            ctx.bind_vertex_array(None);
        }

        ctx.state.draw_descriptor_cache.current = 0;
        ctx.bind_buffer(BufferKind::Vertex, Some(self.vertex_buffer));

        ctx.gl
            .enable_vertex_attrib_array(self.vertex_attrib_location);
        ctx.gl
            .vertex_attrib_pointer_f32(self.vertex_attrib_location, 1, glow::FLOAT, false, 4, 0);

        ctx.set_blend(None);
        ctx.set_color_mask(Rgba::all(true));
        ctx.set_cull_face(None);
        ctx.set_depth(None);
        ctx.set_stencil(None);
        ctx.set_scissor(None);
        ctx.set_viewport(viewport);
        ctx.set_dithering(false);

        ctx.gl
            .draw_arrays(PrimitiveMode::TriangleFan.gl_const(), 0, 4);
    }

    pub unsafe fn destroy(&self, gl: &glow::Context) {
        gl.use_program(None);
        gl.bind_buffer(glow::ARRAY_BUFFER, None);
        gl.delete_buffer(self.vertex_buffer);
        gl.delete_program(self.program);
    }
}

pub struct FrameBufferBlitter {
    fallback: Option<FallbackFramebufferBlitter>,
}

impl FrameBufferBlitter {
    pub unsafe fn new(ctx: &mut GlesContextRef) -> Self {
        match ctx.extensions.blit_framebuffer {
            true => Self { fallback: None },
            false => Self {
                fallback: FallbackFramebufferBlitter::new(ctx).into(),
            },
        }
    }

    pub unsafe fn blit(
        &self,
        ctx: &mut GlesContextRef,
        fb_write: Option<glow::Framebuffer>,
        read: (Size<u32>, glow::Framebuffer, ReadSource),

        source: BlitSourceRect,
        destination: Rect<u32>,
        filter: Filter,
    ) {
        if let Some(fallback) = &self.fallback {
            let tex_coords = match source {
                BlitSourceRect::Pixel(source, flip) => {
                    let src = source.points().map(|pt| {
                        [
                            pt[0] as f32 / read.0.w as f32,
                            pt[1] as f32 / read.0.h as f32,
                        ]
                    });

                    match flip {
                        FlipSource::None => src,
                        FlipSource::X => [src[3], src[2], src[1], src[0]],
                        FlipSource::Y => [src[1], src[0], src[3], src[2]],
                        FlipSource::XY => [src[2], src[3], src[0], src[1]],
                    }
                }
                BlitSourceRect::FullFlipY => [[0., 1.], [0., 0.], [1., 0.], [1., 1.]],
            };

            let texture_unit = match read.2 {
                ReadSource::Texture(texture) => {
                    bind_texture(ctx, fallback.current_texture_unit.get(), texture, filter)
                }
                ReadSource::Unit(unit) => unit,
            };

            fallback.blit(ctx, fb_write, texture_unit, tex_coords, destination);
        } else {
            let fb_read = Some(read.1);

            let destination: Box2D<_> = destination.into();

            let (x0, y0, x1, y1) = match source {
                BlitSourceRect::Pixel(source, flip) => {
                    let src: Box2D<_> = source.into();
                    match flip {
                        FlipSource::None => (src.a[0], src.a[1], src.b[0], src.b[1]),
                        FlipSource::X => (src.b[0], src.a[1], src.a[0], src.b[1]),
                        FlipSource::Y => (src.a[0], src.b[1], src.b[0], src.a[1]),
                        FlipSource::XY => (src.b[0], src.b[1], src.a[0], src.a[1]),
                    }
                }
                BlitSourceRect::FullFlipY => (0, read.0.h, read.0.w, 0),
            };

            unsafe {
                ctx.set_scissor(None);
                ctx.bind_frame_buffer(fb_write);
                ctx.gl.bind_framebuffer(glow::READ_FRAMEBUFFER, fb_read);
                ctx.gl.blit_framebuffer(
                    x0 as i32,
                    y0 as i32,
                    x1 as i32,
                    y1 as i32,
                    destination.a[0] as i32,
                    destination.a[1] as i32,
                    destination.b[0] as i32,
                    destination.b[1] as i32,
                    glow::COLOR_BUFFER_BIT,
                    filter.gl_const(),
                )
            };
        }
    }

    pub unsafe fn destroy(&self, gl: &glow::Context) {
        if let Some(fallback) = &self.fallback {
            fallback.destroy(gl);
        }
    }
}
