use std::cell::{Cell, RefCell, RefMut};

use derive_more::Constructor;
use enum_map::EnumMap;
use glow::HasContext;
use smart_default::SmartDefault;
use yapgeir_graphics_hal::{
    buffer::BufferKind,
    draw_params::{Blend, CullFaceMode, Depth, PolygonOffset, Stencil, StencilCheck},
    sampler::SamplerState,
    Rect, Rgba, Size, WindowBackend,
};

use crate::{
    constants::GlConstant, fake_default_framebuffer::FakeDefaultFrameBuffer,
    frame_buffer_blitter::FrameBufferBlitter, samplers::Samplers, GlesSettings,
};

pub const MAX_TEXTURES: usize = 32;

fn set_parameter(gl: &glow::Context, parameter: u32, value: bool) {
    if value {
        unsafe { gl.enable(parameter) };
    } else {
        unsafe { gl.disable(parameter) };
    }
}

#[derive(Default, Constructor)]
pub struct Feature<T> {
    pub enabled: bool,
    pub value: T,
}

impl<T: PartialEq> Feature<T> {
    fn update(
        &mut self,
        gl: &glow::Context,
        parameter: u32,
        value: Option<T>,
        update: impl Fn(&glow::Context, &T, &T),
    ) {
        if value.is_some() != self.enabled {
            set_parameter(gl, parameter, value.is_some());
            self.enabled = value.is_some();
        }

        if let Some(new_value) = value {
            if new_value != self.value {
                update(gl, &self.value, &new_value);
                self.value = new_value;
            }
        }
    }
}

#[derive(Default, Clone)]
pub struct TextureUnit {
    pub texture: Option<glow::Texture>,
    pub sampler: SamplerState,
}

/// Keep current state for optimizations here, such as
/// currently bound objects.
#[derive(SmartDefault)]
pub struct GlesState {
    pub clear_color: Rgba<f32>,
    pub clear_depth: f32,
    pub clear_stencil: u8,

    pub blend: Feature<Blend>,
    pub cull_face: Feature<CullFaceMode>,
    pub depth: Feature<Depth>,
    pub stencil: Feature<Stencil>,
    pub scissor: Feature<Rect<u32>>,
    pub polygon_offset: Feature<PolygonOffset>,

    #[default(Rgba::all(true))]
    pub color_mask: Rgba<bool>,
    #[default(1.)]
    pub line_width: f32,
    pub dithering: bool,
    pub viewport: Rect<u32>,

    pub active_texture_unit: u32,
    pub texture_unit_limit: usize,
    pub texture_units: [TextureUnit; MAX_TEXTURES],

    pub bound_program: Option<glow::Program>,
    pub bound_buffers: EnumMap<BufferKind, Option<glow::Buffer>>,
    pub bound_frame_buffer: Option<glow::Framebuffer>,
    pub bound_render_buffer: Option<glow::Renderbuffer>,

    pub bound_vertex_array: Option<glow::VertexArray>,

    pub samplers: Samplers,

    // Only relevant when VAO are disabled
    pub draw_descriptor_cache: super::draw_descriptor::DrawDescriptorCache,
}

pub struct Extensions {
    pub vertex_array_objects: bool,
    pub sampler_objects: bool,
    pub blit_framebuffer: bool,
}

pub struct GlesContext<B: WindowBackend> {
    pub gl: glow::Context,
    pub backend: B,
    pub state: RefCell<GlesState>,
    pub default_framebuffer_size: Cell<Option<Size<u32>>>,
    pub extensions: Extensions,
    pub settings: GlesSettings,

    pub fake_default_frame_buffer: Option<RefCell<FakeDefaultFrameBuffer>>,
    pub frame_buffer_blitter: FrameBufferBlitter,
}

impl<B: WindowBackend> Drop for GlesContext<B> {
    fn drop(&mut self) {
        if self.extensions.sampler_objects {
            let mut state = self.state.borrow_mut();
            for i in 0..state.texture_unit_limit {
                unsafe { self.gl.bind_sampler(i as u32, None) };
            }

            state.samplers.drain(&self.gl);
        }

        if let Some(fake_default_frame_buffer) = &self.fake_default_frame_buffer {
            unsafe { fake_default_frame_buffer.borrow().destroy(&self.gl) };
        }

        unsafe { self.frame_buffer_blitter.destroy(&self.gl) };
    }
}

impl<B: WindowBackend> GlesContext<B> {
    pub unsafe fn new(backend: B, settings: GlesSettings) -> Self {
        let gl = glow::Context::from_loader_function(|s| backend.get_proc_address(s));

        gl.pixel_store_i32(glow::PACK_ALIGNMENT, 1);
        gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);

        let extensions = gl.supported_extensions();
        let extensions = Extensions {
            vertex_array_objects: extensions.contains("GL_OES_vertex_array_object"),
            sampler_objects: extensions.contains("GL_ARB_sampler_objects"),
            blit_framebuffer: extensions.contains("GL_EXT_framebuffer_blit"),
        };

        let default_framebuffer_size = backend.default_frame_buffer_size();

        let mut texture_unit_limit = gl.get_parameter_i32(glow::MAX_TEXTURE_IMAGE_UNITS) as usize;
        // Reserve a texture unit for the fake default framebuffer
        if settings.flip_default_frame_buffer && !extensions.blit_framebuffer {
            texture_unit_limit -= 1;
        }

        let state = RefCell::new(GlesState {
            texture_unit_limit,
            ..Default::default()
        });

        let (frame_buffer_blitter, fake_default_frame_buffer) = {
            let mut ctx = GlesContextRef {
                gl: &gl,
                state: state.borrow_mut(),
                extensions: &extensions,
            };

            let frame_buffer_blitter = FrameBufferBlitter::new(&mut ctx);

            let fake_default_frame_buffer = match settings.flip_default_frame_buffer {
                true => Some(RefCell::new(unsafe {
                    FakeDefaultFrameBuffer::new(&mut ctx, default_framebuffer_size)
                })),
                false => None,
            };

            (frame_buffer_blitter, fake_default_frame_buffer)
        };

        Self {
            gl,
            backend,
            state,
            extensions,
            settings,
            default_framebuffer_size: Cell::new(Some(default_framebuffer_size)),
            fake_default_frame_buffer,
            frame_buffer_blitter,
        }
    }

    pub fn default_framebuffer_size(&self) -> Size<u32> {
        match self.default_framebuffer_size.get() {
            Some(size) => size,
            None => {
                let size = self.backend.default_frame_buffer_size();
                self.default_framebuffer_size.set(Some(size));
                size
            }
        }
    }

    pub fn get_ref<'a>(&'a self) -> GlesContextRef<'a> {
        GlesContextRef {
            gl: &self.gl,
            state: self.state.borrow_mut(),
            extensions: &self.extensions,
        }
    }
}

pub struct GlesContextRef<'a> {
    pub gl: &'a glow::Context,
    pub state: RefMut<'a, GlesState>,

    pub extensions: &'a Extensions,
}

impl<'a> GlesContextRef<'a> {
    pub fn set_polygon_offset(&mut self, polygon_offset: Option<PolygonOffset>) {
        self.state.polygon_offset.update(
            &self.gl,
            glow::POLYGON_OFFSET_FILL,
            polygon_offset,
            |gl, _, new| unsafe {
                gl.polygon_offset(new.factor, new.units);
            },
        );
    }

    pub fn set_cull_face(&mut self, cull_face: Option<CullFaceMode>) {
        self.state
            .cull_face
            .update(&self.gl, glow::CULL_FACE, cull_face, |gl, _, new| unsafe {
                gl.cull_face(new.gl_const())
            });
    }

    pub fn set_scissor(&mut self, scissor: Option<Rect<u32>>) {
        self.state
            .scissor
            .update(&self.gl, glow::SCISSOR_TEST, scissor, |gl, _, new| unsafe {
                let rect: Rect<i32> = new.into();
                gl.scissor(rect.x, rect.y, rect.w, rect.h)
            });
    }

    pub fn set_blend(&mut self, blend: Option<Blend>) {
        self.state
            .blend
            .update(&self.gl, glow::BLEND, blend, |gl, old, new| unsafe {
                if old.color != new.color {
                    gl.blend_color(new.color.r, new.color.g, new.color.b, new.color.a);
                }

                if old.equation != new.equation {
                    gl.blend_equation_separate(
                        new.equation.rgb.gl_const(),
                        new.equation.alpha.gl_const(),
                    );
                }

                if old.function != new.function {
                    gl.blend_func_separate(
                        new.function.rgb.source.gl_const(),
                        new.function.rgb.destination.gl_const(),
                        new.function.alpha.source.gl_const(),
                        new.function.alpha.destination.gl_const(),
                    );
                }
            });
    }

    pub fn set_depth(&mut self, depth: Option<Depth>) {
        self.state
            .depth
            .update(&self.gl, glow::DEPTH_TEST, depth, |gl, old, new| unsafe {
                if old.test != new.test {
                    gl.depth_func(new.test.gl_const());
                }

                if old.write != new.write {
                    gl.depth_mask(new.write)
                }

                if old.range != new.range {
                    gl.depth_range_f32(new.range.0, new.range.1);
                }
            });
    }

    pub fn set_stencil(&mut self, stencil: Option<Stencil>) {
        self.state.stencil.update(
            &self.gl,
            glow::STENCIL_TEST,
            stencil,
            |gl, old, new| unsafe {
                unsafe fn update(
                    gl: &glow::Context,
                    face: u32,
                    old: &StencilCheck,
                    new: &StencilCheck,
                ) {
                    if old.action != new.action {
                        gl.stencil_op_separate(
                            face,
                            new.action.stencil_fail.gl_const(),
                            new.action.depth_fail.gl_const(),
                            new.action.pass.gl_const(),
                        )
                    }

                    if old.function != new.function {
                        gl.stencil_func_separate(
                            face,
                            new.function.test.gl_const(),
                            new.function.reference_value as i32,
                            new.function.mask as u32,
                        )
                    }

                    if old.action_mask != new.action_mask {
                        gl.stencil_mask_separate(face, new.action_mask as u32);
                    }
                }

                update(gl, glow::BACK, &old.back, &new.back);
                update(gl, glow::FRONT, &old.front, &new.front);
            },
        );
    }

    pub fn set_dithering(&mut self, dithering: bool) {
        if self.state.dithering != dithering {
            set_parameter(self.gl, glow::DITHER, dithering);
            self.state.dithering = dithering;
        }
    }

    pub fn set_color_mask(&mut self, mask: Rgba<bool>) {
        if self.state.color_mask != mask {
            unsafe { self.gl.color_mask(mask.r, mask.g, mask.b, mask.a) };
            self.state.color_mask = mask;
        }
    }

    pub fn set_line_width(&mut self, line_width: f32) {
        if self.state.line_width != line_width {
            unsafe { self.gl.line_width(line_width) };
            self.state.line_width = line_width;
        }
    }

    pub fn set_viewport(&mut self, viewport: Rect<u32>) {
        if viewport != self.state.viewport {
            unsafe {
                let rect: Rect<i32> = (&viewport).into();
                self.gl.viewport(rect.x, rect.y, rect.w, rect.h)
            };
            self.state.viewport = viewport;
        }
    }

    pub fn set_clear_color(&mut self, color: Rgba<f32>) {
        if self.state.clear_color != color {
            unsafe { self.gl.clear_color(color.r, color.g, color.b, color.a) };
            self.state.clear_color = color;
        }
    }

    pub fn set_clear_depth(&mut self, depth: f32) {
        if self.state.clear_depth != depth {
            unsafe { self.gl.clear_depth_f32(depth) };
            self.state.clear_depth = depth;
        }
    }

    pub fn set_clear_stencil(&mut self, stencil: u8) {
        if self.state.clear_stencil != stencil {
            unsafe { self.gl.clear_stencil(stencil as i32) };
            self.state.clear_stencil = stencil;
        }
    }

    pub fn clear(
        &mut self,
        scissor: Option<Rect<u32>>,
        color: Option<Rgba<f32>>,
        depth: Option<f32>,
        stencil: Option<u8>,
    ) {
        if color.is_none() && depth.is_none() && stencil.is_none() {
            return;
        }

        self.set_color_mask(Rgba::all(true));
        self.set_scissor(scissor);

        let mut mask = 0;
        if let Some(color) = color {
            mask |= glow::COLOR_BUFFER_BIT;
            self.set_clear_color(color);
        }
        if let Some(depth) = depth {
            mask |= glow::DEPTH_BUFFER_BIT;
            self.set_clear_depth(depth);
        }
        if let Some(stencil) = stencil {
            mask |= glow::STENCIL_BUFFER_BIT;
            self.set_clear_stencil(stencil);
        }

        unsafe {
            self.gl.clear(mask);
        }
    }

    pub fn use_program(&mut self, program: Option<glow::Program>) {
        if self.state.bound_program != program {
            unsafe { self.gl.use_program(program) };
            self.state.bound_program = program;
        }
    }

    pub fn activate_texture_unit(&mut self, unit: u32) {
        if self.state.active_texture_unit != unit {
            unsafe { self.gl.active_texture(glow::TEXTURE0 + unit) };
            self.state.active_texture_unit = unit;
        }
    }

    pub fn bind_texture(&mut self, unit: u32, texture: Option<glow::Texture>) {
        if self.state.texture_units[unit as usize].texture != texture {
            self.activate_texture_unit(unit);
            unsafe { self.gl.bind_texture(glow::TEXTURE_2D, texture) };
            self.state.texture_units[unit as usize].texture = texture;
        }
    }

    pub fn activate_texture(&mut self, texture: glow::Texture) {
        let mut empty_unit = None;
        let mut bound_unit = None;

        for i in 0..self.state.texture_unit_limit {
            let unit = &self.state.texture_units[i];

            if unit.texture == Some(texture) {
                bound_unit = Some(i);
                break;
            }

            if empty_unit == None && unit.texture == None {
                empty_unit = Some(i);
            }
        }

        if let Some(bound_unit) = bound_unit {
            self.activate_texture_unit(bound_unit as u32);
        } else {
            let empty_unit = empty_unit.unwrap_or(0) as u32;
            self.activate_texture_unit(empty_unit);
            unsafe { self.gl.bind_texture(glow::TEXTURE_2D, Some(texture)) };

            let unit = self.state.active_texture_unit as usize;
            self.state.texture_units[unit].texture = Some(texture);
        }
    }

    pub fn bind_render_buffer(&mut self, renderbuffer: Option<glow::Renderbuffer>) {
        if self.state.bound_render_buffer != renderbuffer {
            unsafe { self.gl.bind_renderbuffer(glow::RENDERBUFFER, renderbuffer) };
            self.state.bound_render_buffer = renderbuffer;
        }
    }

    pub fn bind_frame_buffer(&mut self, framebuffer: Option<glow::Framebuffer>) {
        if self.state.bound_frame_buffer != framebuffer {
            unsafe { self.gl.bind_framebuffer(glow::FRAMEBUFFER, framebuffer) };
            self.state.bound_frame_buffer = framebuffer;
        }
    }

    pub fn bind_buffer(&mut self, kind: BufferKind, buffer: Option<glow::Buffer>) {
        if self.state.bound_buffers[kind] != buffer {
            unsafe { self.gl.bind_buffer(kind.gl_const(), buffer) };
            self.state.bound_buffers[kind] = buffer;
        }
    }

    pub fn bind_vertex_array(&mut self, vertex_array: Option<glow::VertexArray>) {
        // Do not rely on bound buffers after switching VAO
        self.state.bound_buffers.clear();

        if self.state.bound_vertex_array != vertex_array {
            unsafe { self.gl.bind_vertex_array(vertex_array) };
            self.state.bound_vertex_array = vertex_array;
        }
    }
}
