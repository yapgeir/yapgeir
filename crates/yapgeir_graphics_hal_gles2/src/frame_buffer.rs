use std::{borrow::Borrow, ops::Deref, rc::Rc};

use bitvec::prelude::BitArray;
use bm::Pod;
use bytemuck as bm;
use glow::HasContext;
use yapgeir_graphics_hal::{
    draw_params::DrawParameters,
    frame_buffer::{
        Attachment, DepthStencilAttachment, FrameBuffer, Indices, ReadFormat, RenderBuffer,
        RenderBufferFormat,
    },
    sampler::SamplerState,
    samplers::SamplerAttribute,
    uniforms::{UniformAttribute, Uniforms},
    Backend, ImageSize, Rect, Rgba,
};

use crate::{
    constants::GlConstant,
    context::GlesContextRef,
    draw_descriptor::GlesDrawDescriptor,
    shader::{GlesShader, ShaderState, UniformKind},
    texture::{GlesTexture, RgbLayout, RgbaLayout},
    uniforms::GlesUniformBuffer,
    Gles,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlesReadFormat {
    Alpha,
    Rgb(RgbLayout),
    Rgba(RgbaLayout),
}

impl From<ReadFormat> for GlesReadFormat {
    fn from(value: ReadFormat) -> Self {
        match value {
            ReadFormat::Alpha => GlesReadFormat::Alpha,
            ReadFormat::Rgb => GlesReadFormat::Rgb(RgbLayout::U8),
            ReadFormat::Rgba => GlesReadFormat::Rgba(RgbaLayout::U8),
        }
    }
}

impl GlesReadFormat {
    fn gl(self) -> (u32, u32) {
        match self {
            GlesReadFormat::Alpha => (glow::ALPHA, glow::UNSIGNED_BYTE),
            GlesReadFormat::Rgb(f) => (glow::RGB, f.gl_const()),
            GlesReadFormat::Rgba(f) => (glow::RGBA, f.gl_const()),
        }
    }
}

pub struct GlesRenderBuffer<B: Backend> {
    ctx: Gles<B>,
    renderbuffer: glow::Renderbuffer,
}

impl<B: Backend> RenderBuffer<Gles<B>> for GlesRenderBuffer<B> {
    type Format = RenderBufferFormat;

    fn new(ctx: Gles<B>, size: ImageSize<u32>, format: RenderBufferFormat) -> Self {
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

impl<B: Backend> Drop for GlesRenderBuffer<B> {
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

unsafe fn attach_texture<B: Backend>(
    gl: &glow::Context,
    texture: &GlesTexture<B>,
    attachment: u32,
) {
    gl.framebuffer_texture_2d(
        glow::FRAMEBUFFER,
        attachment,
        glow::TEXTURE_2D,
        Some(texture.texture),
        0,
    )
}

unsafe fn attach_render_buffer<B: Backend>(
    gl: &glow::Context,
    render_buffer: &GlesRenderBuffer<B>,
    attachment: u32,
) {
    gl.framebuffer_renderbuffer(
        glow::FRAMEBUFFER,
        attachment,
        glow::RENDERBUFFER,
        Some(render_buffer.renderbuffer),
    )
}

unsafe fn attach<B: Backend>(gl: &glow::Context, attachment: &Attachment<Gles<B>>, kind: u32) {
    match attachment {
        Attachment::Texture(texture) => attach_texture(gl, &texture, kind),
        Attachment::RenderBuffer(renderbuffer) => attach_render_buffer(gl, &renderbuffer, kind),
    }
}

enum Resources<B: Backend> {
    Default,
    Managed {
        size: ImageSize<u32>,
        framebuffer: glow::Framebuffer,
        _draw_texture: Rc<GlesTexture<B>>,
        _depth_stencil: DepthStencilAttachment<Gles<B>>,
    },
}

impl<B: Backend> Resources<B> {
    fn framebuffer(&self) -> Option<glow::Framebuffer> {
        match self {
            Resources::Default => None,
            Resources::Managed { framebuffer, .. } => Some(*framebuffer),
        }
    }
}

pub struct GlesFrameBuffer<B: Backend> {
    ctx: Gles<B>,
    res: Resources<B>,
}

impl<B: Backend + 'static> FrameBuffer<Gles<B>> for GlesFrameBuffer<B> {
    type ReadFormat = GlesReadFormat;

    fn default(ctx: Gles<B>) -> Self {
        Self {
            ctx,
            res: Resources::Default,
        }
    }

    fn new(
        ctx: Gles<B>,
        draw_texture: Rc<GlesTexture<B>>,
        depth_stencil: DepthStencilAttachment<Gles<B>>,
    ) -> Self {
        let gl = &ctx.gl;
        let framebuffer = unsafe {
            let fb = gl
                .create_framebuffer()
                .expect("unable to create a framebuffer");
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fb));

            attach_texture(gl, &draw_texture, glow::COLOR_ATTACHMENT0);

            match &depth_stencil {
                DepthStencilAttachment::None => {}
                DepthStencilAttachment::Depth(depth) => {
                    attach(gl, depth, glow::DEPTH_ATTACHMENT);
                }
                DepthStencilAttachment::Stencil(stencil) => {
                    attach(gl, stencil, glow::STENCIL_ATTACHMENT);
                }
                DepthStencilAttachment::DepthStencil(depth_stencil) => {
                    attach(gl, depth_stencil, glow::DEPTH_STENCIL_ATTACHMENT);
                }
                DepthStencilAttachment::DepthAndStencil { depth, stencil } => {
                    attach(gl, depth, glow::DEPTH_ATTACHMENT);
                    attach(gl, stencil, glow::STENCIL_ATTACHMENT);
                }
            }

            fb
        };

        Self {
            ctx,
            res: Resources::Managed {
                size: draw_texture.size,
                framebuffer,
                _draw_texture: draw_texture,
                _depth_stencil: depth_stencil,
            },
        }
    }

    fn size(&self) -> ImageSize<u32> {
        let mut ctx = self.ctx.get_ref();
        match self.res {
            Resources::Default => match ctx.state.default_frame_buffer_size {
                Some(size) => size,
                None => {
                    let size = self.ctx.backend.default_framebuffer_size();
                    ctx.state.default_frame_buffer_size = Some(size);
                    size
                }
            },
            Resources::Managed { size, .. } => size,
        }
    }

    fn clear(
        &self,
        scissor: Option<Rect<u32>>,
        color: Option<Rgba<f32>>,
        depth: Option<f32>,
        stencil: Option<u8>,
    ) {
        let mut ctx = self.ctx.get_ref();
        ctx.bind_frame_buffer(self.res.framebuffer());
        ctx.clear(scissor, color, depth, stencil);
    }

    fn draw<U: Uniforms + Pod>(
        &self,
        draw_descriptor: &GlesDrawDescriptor<B>,
        draw_parameters: &DrawParameters,
        textures: &[SamplerAttribute<Gles<B>, impl Borrow<GlesTexture<B>>>],
        uniforms: Option<&GlesUniformBuffer<U>>,
        indices: &Indices,
    ) {
        let size = self.size();
        let mut ctx = self.ctx.get_ref();
        ctx.use_program(Some(draw_descriptor.shader.program));
        bind_textures(&mut ctx, &draw_descriptor.shader, textures);

        if let Some(uniforms) = uniforms {
            let uniforms = uniforms.value.borrow();
            let uniforms = bm::bytes_of(uniforms.deref());
            bind_uniforms(&mut ctx, &draw_descriptor.shader, uniforms, U::FORMAT);
        }

        // To reduce code duplication, the remaining code without generics is
        // extracted as a function
        draw_impl(
            &mut ctx,
            self.res.framebuffer(),
            draw_descriptor,
            draw_parameters,
            size,
            indices,
        );
    }

    fn read(&self, rect: Rect<u32>, format: GlesReadFormat, target: &mut [u8]) {
        let mut ctx = self.ctx.get_ref();
        ctx.bind_frame_buffer(self.res.framebuffer());

        let (format, ty) = format.gl();

        unsafe {
            ctx.gl.read_pixels(
                rect.x as i32,
                rect.y as i32,
                rect.w as i32,
                rect.h as i32,
                format,
                ty,
                glow::PixelPackData::Slice(target),
            );
        }
    }
}

fn draw_impl<'a, B: Backend>(
    ctx: &mut GlesContextRef<'_>,
    frame_buffer: Option<glow::Framebuffer>,
    draw_descriptor: &GlesDrawDescriptor<B>,
    draw_parameters: &DrawParameters,
    size: ImageSize<u32>,
    indices: &Indices,
) {
    ctx.bind_frame_buffer(frame_buffer);
    draw_descriptor.bind(ctx);
    set_draw_parameters(ctx, draw_parameters, size);

    unsafe {
        match &draw_descriptor.index_kind {
            None => {
                ctx.gl.draw_arrays(
                    indices.mode.gl_const(),
                    indices.offset as i32,
                    indices.len as i32,
                );
            }
            Some(kind) => {
                ctx.gl.draw_elements(
                    indices.mode.gl_const(),
                    indices.len as i32,
                    kind.gl_const(),
                    (indices.offset * kind.size()) as i32,
                );
            }
        }
    }
}

impl<B: Backend> Drop for GlesFrameBuffer<B> {
    fn drop(&mut self) {
        unsafe {
            let mut ctx = self.ctx.get_ref();

            if let Resources::Managed { framebuffer, .. } = self.res {
                if ctx.state.bound_frame_buffer == Some(framebuffer) {
                    ctx.bind_frame_buffer(None);
                }
                ctx.gl.delete_framebuffer(framebuffer);
            }
        }
    }
}

fn set_draw_parameters<'a>(
    ctx: &mut GlesContextRef<'a>,
    draw_parameters: &DrawParameters,
    framebuffer_size: ImageSize<u32>,
) {
    let viewport = draw_parameters
        .viewport
        .clone()
        .unwrap_or_else(|| framebuffer_size.into());

    ctx.set_blend(draw_parameters.blend.clone());
    ctx.set_color_mask(draw_parameters.color_mask.clone());
    ctx.set_cull_face(draw_parameters.cull_face);
    ctx.set_depth(draw_parameters.depth.clone());
    ctx.set_stencil(draw_parameters.stencil.clone());
    ctx.set_scissor(draw_parameters.scissor.clone());
    ctx.set_viewport(viewport);
    ctx.set_line_width(draw_parameters.line_width);
    ctx.set_polygon_offset(draw_parameters.polygon_offset.clone());
    ctx.set_dithering(draw_parameters.dithering);
}

fn bind_textures<'a, B: Backend + 'a>(
    ctx: &mut GlesContextRef<'a>,
    shader: &GlesShader<B>,
    textures: &[SamplerAttribute<Gles<B>, impl Borrow<GlesTexture<B>>>],
) {
    let mut used_units = BitArray::<u32>::new(0u32);
    let mut shader_state = shader.state.borrow_mut();
    let mut no_free_units = false;

    for binding in textures {
        bind_texture(
            ctx,
            &mut used_units,
            &mut shader_state,
            &mut no_free_units,
            binding.name,
            binding.sampler.state,
            binding.sampler.texture.borrow(),
        );
    }
}

fn bind_texture<B: Backend>(
    ctx: &mut GlesContextRef,
    used_units: &mut BitArray<u32>,
    shader_state: &mut ShaderState,
    no_free_units: &mut bool,

    name: &str,
    sampler: SamplerState,
    texture: &GlesTexture<B>,
) {
    // Skip if shader has no texture binding with this name
    let (location, cached_unit) = match shader_state.sampler_attributes.get_mut(name) {
        Some(l) => l,
        None => return,
    };

    // Check if no binding is necessary
    {
        let unit: usize = *cached_unit as usize;
        let unit_data = &ctx.state.texture_units[unit];
        if unit_data.texture == Some(texture.texture) {
            if unit_data.sampler == sampler {
                used_units.set(unit, true);
                return;
            }

            // A sampler can be changed only if its location is not used by another binding for this draw call.
            // This allows binding same texture with different samplers, if sampler objects are supported.

            let can_reuse = !ctx.features.sampler_objects
                || !used_units.get(unit).as_deref().cloned().unwrap_or(false);

            if can_reuse {
                used_units.set(unit, true);
                ctx.bind_sampler(unit as u32, sampler);
                return;
            }
        }
    }

    let unit = if *no_free_units {
        None
    } else {
        let free_unit = (0..ctx.state.texture_unit_limit)
            .find(|&i| ctx.state.texture_units[i].texture.is_none());
        if free_unit == None {
            *no_free_units = true;
        }

        free_unit
    };

    // Find an unused slot and bind texture and sampler there.
    let unit = unit
        .or_else(|| {
            (0..ctx.state.texture_unit_limit)
                .find(|&i| !used_units.get(i).as_deref().cloned().unwrap_or(false))
        })
        .expect("Trying to bind more textures than there are slots");

    unsafe {
        ctx.activate_texture_unit(unit as u32);
        ctx.gl.bind_texture(glow::TEXTURE_2D, Some(texture.texture));
        ctx.state.texture_units[unit].texture = Some(texture.texture);
        ctx.bind_sampler(unit as u32, sampler);

        ctx.gl.uniform_1_i32(Some(&location), unit as i32);
    }

    // Update shader's unit cache
    *cached_unit = unit as u8;
    // Mark the new slot as used
    used_units.set(unit, true);
}

fn bind_uniforms<'a, B: Backend>(
    ctx: &mut GlesContextRef<'a>,
    shader: &GlesShader<B>,
    uniforms: &[u8],
    format: &'static [UniformAttribute],
) {
    let mut shader_state = shader.state.borrow_mut();

    let same_type = std::ptr::eq(shader_state.uniforms_cache.0, format);

    for attribute in format.iter() {
        let (location, kind, size) = match shader.uniform_attributes.get(attribute.name) {
            Some(location) => location,
            None => {
                // Uniform not defined in our shader, skipping binding
                continue;
            }
        };

        let range = attribute.offset..(attribute.offset + attribute.size);
        let new = &uniforms[range.clone()];
        if same_type {
            if &shader_state.uniforms_cache.1[range] == new {
                continue;
            }
        }

        assert!(
            *size <= attribute.size,
            "Shader expects larger value than provided by uniform for uniform {}",
            attribute.name
        );

        let l = Some(location);
        unsafe {
            match kind {
                UniformKind::Int => ctx.gl.uniform_1_i32_slice(l, bm::cast_slice(new)),
                UniformKind::IntVec2 => ctx.gl.uniform_2_i32_slice(l, bm::cast_slice(new)),
                UniformKind::IntVec3 => ctx.gl.uniform_3_i32_slice(l, bm::cast_slice(new)),
                UniformKind::IntVec4 => ctx.gl.uniform_4_i32_slice(l, bm::cast_slice(new)),
                UniformKind::Float => ctx.gl.uniform_1_f32_slice(l, bm::cast_slice(new)),
                UniformKind::FloatVec2 => ctx.gl.uniform_2_f32_slice(l, bm::cast_slice(new)),
                UniformKind::FloatVec3 => ctx.gl.uniform_3_f32_slice(l, bm::cast_slice(new)),
                UniformKind::FloatVec4 => ctx.gl.uniform_4_f32_slice(l, bm::cast_slice(new)),
                UniformKind::Mat2 => {
                    ctx.gl
                        .uniform_matrix_2_f32_slice(l, false, bm::cast_slice(new))
                }
                UniformKind::Mat3 => {
                    ctx.gl
                        .uniform_matrix_3_f32_slice(l, false, bm::cast_slice(new))
                }
                UniformKind::Mat4 => {
                    ctx.gl
                        .uniform_matrix_4_f32_slice(l, false, bm::cast_slice(new))
                }
            }
        }
    }

    shader_state.uniforms_cache.0 = format;
    shader_state.uniforms_cache.1.clear();
    shader_state.uniforms_cache.1.extend_from_slice(uniforms);
}
