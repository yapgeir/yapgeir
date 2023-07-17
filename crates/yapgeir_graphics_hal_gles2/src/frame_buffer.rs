use core::panic;
use std::{borrow::Borrow, ops::Deref, rc::Rc};

use bitvec::prelude::BitArray;
use bm::Pod;
use bytemuck as bm;
use glow::HasContext;
use yapgeir_graphics_hal::{
    draw_params::DrawParameters,
    frame_buffer::{
        Attachment, DepthStencilAttachment, FlipSource, FrameBuffer, Indices, ReadFormat,
    },
    sampler::{Filter, SamplerState},
    samplers::SamplerAttribute,
    uniforms::{UniformAttribute, Uniforms},
    Rect, Rgba, Size, WindowBackend,
};

use crate::{
    constants::GlConstant,
    context::{GlesContext, GlesContextRef},
    draw_descriptor::GlesDrawDescriptor,
    frame_buffer_blitter::{BlitSourceRect, ReadSource},
    render_buffer::GlesRenderBuffer,
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

unsafe fn attach_texture<B: WindowBackend>(
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

unsafe fn attach_render_buffer<B: WindowBackend>(
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

unsafe fn attach<B: WindowBackend>(
    gl: &glow::Context,
    attachment: &Attachment<Gles<B>>,
    kind: u32,
) {
    match attachment {
        Attachment::Texture(texture) => attach_texture(gl, &texture, kind),
        Attachment::RenderBuffer(renderbuffer) => attach_render_buffer(gl, &renderbuffer, kind),
    }
}

// OpenGL uses Y-up coordinate system for everything.
// This function is used to convert scissor and viewport rectangles from
// y-down coordinates.
fn to_y_up(rect: &Rect<u32>, size: &Size<u32>) -> Rect<u32> {
    Rect::new(rect.x, size.h - rect.y - rect.h, rect.w, rect.h)
}

enum Resources<B: WindowBackend> {
    Default,
    Managed {
        size: Size<u32>,
        framebuffer: glow::Framebuffer,
        _draw_texture: Rc<GlesTexture<B>>,
        _depth_stencil: DepthStencilAttachment<Gles<B>>,
    },
}

impl<B: WindowBackend> Resources<B> {
    fn framebuffer(&self, ctx: &GlesContext<B>) -> Option<glow::Framebuffer> {
        match self {
            Resources::Default => match &ctx.fake_default_frame_buffer {
                Some(fake_default_frame_buffer) => Some(unsafe {
                    fake_default_frame_buffer
                        .borrow_mut()
                        .framebuffer(&mut ctx.get_ref(), self.size(ctx))
                }),
                None => None,
            },
            Resources::Managed { framebuffer, .. } => Some(*framebuffer),
        }
    }

    fn size<'a>(&self, ctx: &GlesContext<B>) -> Size<u32> {
        match self {
            Resources::Default => ctx.default_framebuffer_size(),
            Resources::Managed { size, .. } => *size,
        }
    }
}

pub struct GlesFrameBuffer<B: WindowBackend> {
    ctx: Gles<B>,
    res: Resources<B>,
}

impl<B: WindowBackend + 'static> FrameBuffer<Gles<B>> for GlesFrameBuffer<B> {
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
        let framebuffer = unsafe {
            let mut ctx = ctx.get_ref();
            let fb = ctx
                .gl
                .create_framebuffer()
                .expect("unable to create a framebuffer");
            ctx.bind_frame_buffer(Some(fb));

            attach_texture(ctx.gl, &draw_texture, glow::COLOR_ATTACHMENT0);

            match &depth_stencil {
                DepthStencilAttachment::None => {}
                DepthStencilAttachment::Depth(depth) => {
                    attach(ctx.gl, depth, glow::DEPTH_ATTACHMENT);
                }
                DepthStencilAttachment::Stencil(stencil) => {
                    attach(ctx.gl, stencil, glow::STENCIL_ATTACHMENT);
                }
                DepthStencilAttachment::DepthStencil(depth_stencil) => {
                    attach(ctx.gl, depth_stencil, glow::DEPTH_STENCIL_ATTACHMENT);
                }
                DepthStencilAttachment::DepthAndStencil { depth, stencil } => {
                    attach(ctx.gl, depth, glow::DEPTH_ATTACHMENT);
                    attach(ctx.gl, stencil, glow::STENCIL_ATTACHMENT);
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

    fn size(&self) -> Size<u32> {
        self.res.size(&self.ctx)
    }

    fn clear(
        &self,
        scissor: Option<Rect<u32>>,
        color: Option<Rgba<f32>>,
        depth: Option<f32>,
        stencil: Option<u8>,
    ) {
        // Flip scissor coordinates, unless we're conforming to a coordinate space
        let scissor = if self.ctx.settings.flip_default_frame_buffer {
            scissor
        } else {
            scissor.map(|scissor| {
                let size = self.size();
                to_y_up(&scissor, &size)
            })
        };

        let fb = self.res.framebuffer(&self.ctx);
        let mut ctx = self.ctx.get_ref();
        ctx.bind_frame_buffer(fb);
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
        let fb = self.res.framebuffer(&self.ctx);
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
            fb,
            draw_descriptor,
            draw_parameters,
            size,
            indices,
            self.ctx.settings.flip_default_frame_buffer,
        );
    }

    fn blit(
        &self,
        read_frame_buffer: &GlesFrameBuffer<B>,
        source: Rect<u32>,
        destination: Rect<u32>,
        flip_source: FlipSource,
        filter: Filter,
    ) {
        let read = match &read_frame_buffer.res {
            Resources::Default => {
                panic!("Reading from a default framebuffer is unsupported!");
            }
            Resources::Managed {
                size,
                framebuffer,
                _draw_texture: tex,
                _depth_stencil,
            } => (
                size.clone(),
                framebuffer.clone(),
                ReadSource::Texture(tex.texture),
            ),
        };

        let fb_write = self.res.framebuffer(&self.ctx);

        unsafe {
            self.ctx.frame_buffer_blitter.blit(
                &mut self.ctx.get_ref(),
                fb_write,
                read,
                BlitSourceRect::Pixel(source, flip_source),
                destination,
                filter,
            )
        };
    }

    fn read(&self, rect: Rect<u32>, format: GlesReadFormat, target: &mut [u8]) {
        let fb = self.res.framebuffer(&self.ctx);

        let mut ctx = self.ctx.get_ref();
        ctx.bind_frame_buffer(fb);

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

fn draw_impl<'a, B: WindowBackend>(
    ctx: &mut GlesContextRef<'_>,
    frame_buffer: Option<glow::Framebuffer>,
    draw_descriptor: &GlesDrawDescriptor<B>,
    draw_parameters: &DrawParameters,
    size: Size<u32>,
    indices: &Indices,
    flip_default_framebuffer: bool,
) {
    ctx.bind_frame_buffer(frame_buffer);
    draw_descriptor.bind(ctx);
    set_draw_parameters(ctx, draw_parameters, size, flip_default_framebuffer);

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

impl<B: WindowBackend> Drop for GlesFrameBuffer<B> {
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
    framebuffer_size: Size<u32>,
    flip_default_framebuffer: bool,
) {
    let (scissor, viewport) = if flip_default_framebuffer {
        (
            draw_parameters.scissor.clone(),
            draw_parameters.viewport.clone(),
        )
    } else {
        let scissor = draw_parameters
            .scissor
            .as_ref()
            .map(|scissor| to_y_up(&scissor, &framebuffer_size));

        let viewport = draw_parameters
            .viewport
            .clone()
            .map(|viewport| to_y_up(&viewport, &framebuffer_size));
        (scissor, viewport)
    };

    let viewport = viewport.unwrap_or_else(|| framebuffer_size.into());

    ctx.set_blend(draw_parameters.blend.clone());
    ctx.set_color_mask(draw_parameters.color_mask);
    ctx.set_cull_face(draw_parameters.cull_face);
    ctx.set_depth(draw_parameters.depth.clone());
    ctx.set_stencil(draw_parameters.stencil.clone());
    ctx.set_scissor(scissor);
    ctx.set_viewport(viewport);
    ctx.set_line_width(draw_parameters.line_width);
    ctx.set_polygon_offset(draw_parameters.polygon_offset);
    ctx.set_dithering(draw_parameters.dithering);
}

fn bind_textures<'a, B: WindowBackend + 'a>(
    ctx: &mut GlesContextRef<'a>,
    shader: &GlesShader<B>,
    textures: &[SamplerAttribute<Gles<B>, impl Borrow<GlesTexture<B>>>],
) {
    let mut used_units = BitArray::<u32>::new(0u32);
    let mut shader_state = shader.state.borrow_mut();

    for binding in textures {
        bind_texture(
            ctx,
            &mut used_units,
            &mut shader_state,
            binding.name,
            binding.sampler.state,
            binding.sampler.texture.borrow(),
        );
    }
}

fn reuse_texture_unit(
    ctx: &mut GlesContextRef,
    unit: usize,
    texture: glow::Texture,
    sampler: SamplerState,
    used_units: &mut BitArray<u32>,
) -> bool {
    let unit_data = &ctx.state.texture_units[unit];

    if unit_data.texture != Some(texture) {
        return false;
    }

    if unit_data.sampler == sampler {
        used_units.set(unit, true);
        return true;
    }

    // A sampler can be changed only if its location is not used by another binding for this draw call.
    // This allows binding same texture with different samplers, if sampler objects are supported.
    let can_reuse = !ctx.extensions.sampler_objects
        || !used_units.get(unit).as_deref().cloned().unwrap_or(false);

    if can_reuse {
        ctx.bind_sampler(unit as u32, sampler);
        used_units.set(unit, true);
        return true;
    }

    false
}

fn bind_texture<B: WindowBackend>(
    ctx: &mut GlesContextRef,
    used_units: &mut BitArray<u32>,
    shader_state: &mut ShaderState,

    name: &str,
    sampler: SamplerState,
    texture: &GlesTexture<B>,
) {
    // Skip if shader has no texture binding with this name
    let (location, cached_unit) = match shader_state.sampler_attributes.get_mut(name) {
        Some(l) => l,
        None => return,
    };

    // Check if no re-binding is necessary
    if reuse_texture_unit(ctx, *cached_unit, texture.texture, sampler, used_units) {
        return;
    }

    let mut empty_unit = None;
    let mut overridable_unit = None;

    for unit in 0..ctx.state.texture_unit_limit {
        if empty_unit == None && ctx.state.texture_units[unit].texture.is_none() {
            empty_unit = Some(unit);
            continue;
        }

        if overridable_unit == None && used_units.get(unit).as_deref().cloned().unwrap_or(false) {
            overridable_unit = Some(unit);
        }

        if reuse_texture_unit(ctx, unit, texture.texture, sampler, used_units) {
            unsafe { ctx.gl.uniform_1_i32(Some(&location), unit as i32) };
            *cached_unit = unit;
            return;
        }
    }

    // Prefer empty unit over overridable unit
    match empty_unit.or(overridable_unit) {
        Some(unit) => unsafe {
            ctx.bind_texture(unit as u32, Some(texture.texture));
            ctx.bind_sampler(unit as u32, sampler);
            used_units.set(unit, true);

            ctx.gl.uniform_1_i32(Some(&location), unit as i32);
            *cached_unit = unit;
        },
        None => {
            panic!("Trying to bind more textures than there are slots");
        }
    }
}

fn bind_uniforms<'a, B: WindowBackend>(
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
