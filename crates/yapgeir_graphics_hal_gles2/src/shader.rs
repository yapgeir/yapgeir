use core::panic;
use std::{cell::RefCell, collections::HashMap};

use glow::HasContext;
use yapgeir_graphics_hal::{
    shader::{Shader, TextShaderSource},
    uniforms::{UniformAttribute, Uniforms},
    WindowBackend,
};

use crate::Gles;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UniformKind {
    Int,
    IntVec2,
    IntVec3,
    IntVec4,
    Float,
    FloatVec2,
    FloatVec3,
    FloatVec4,
    Mat2,
    Mat3,
    Mat4,
}

impl UniformKind {
    fn size(self) -> usize {
        match self {
            UniformKind::Int => 4 * 1,
            UniformKind::IntVec2 => 4 * 2,
            UniformKind::IntVec3 => 4 * 3,
            UniformKind::IntVec4 => 4 * 4,
            UniformKind::Float => 4 * 1,
            UniformKind::FloatVec2 => 4 * 2,
            UniformKind::FloatVec3 => 4 * 3,
            UniformKind::FloatVec4 => 4 * 4,
            UniformKind::Mat2 => 4 * 4,
            UniformKind::Mat3 => 4 * 9,
            UniformKind::Mat4 => 4 * 16,
        }
    }
}

pub struct ShaderState {
    pub sampler_attributes: HashMap<String, (glow::UniformLocation, usize)>,
    pub uniforms_cache: (&'static [UniformAttribute], Vec<u8>),
}

pub struct GlesShader<B: WindowBackend> {
    pub ctx: Gles<B>,
    pub program: glow::Program,
    pub attribute_data: HashMap<String, u32>,
    pub uniform_attributes: HashMap<String, (glow::UniformLocation, UniformKind, usize)>,

    pub state: RefCell<ShaderState>,
}

pub unsafe fn compile_program(gl: &glow::Context, source: &TextShaderSource) -> glow::Program {
    let program = gl.create_program().expect("Cannot create program");
    let shaders = [
        (glow::VERTEX_SHADER, source.vertex),
        (glow::FRAGMENT_SHADER, source.fragment),
    ]
    .map(|(kind, source)| {
        let shader = gl.create_shader(kind).expect("Cannot create shader");
        gl.shader_source(shader, source);
        gl.compile_shader(shader);
        if !gl.get_shader_compile_status(shader) {
            panic!(
                "Error compiling shader: {}. Shader: \n {source}",
                gl.get_shader_info_log(shader)
            );
        }
        gl.attach_shader(program, shader);
        shader
    });

    gl.link_program(program);
    if !gl.get_program_link_status(program) {
        panic!("{}", gl.get_program_info_log(program));
    }

    for shader in shaders {
        gl.delete_shader(shader);
    }

    program
}

unsafe fn get_uniforms(
    gl: &glow::Context,
    program: glow::Program,
) -> (
    HashMap<String, (glow::UniformLocation, UniformKind, usize)>,
    HashMap<String, (glow::UniformLocation, usize)>,
) {
    let uniform_count = gl.get_active_uniforms(program) as usize;
    let mut uniforms = HashMap::with_capacity(uniform_count);
    let mut samplers = HashMap::new();

    for i in 0..uniform_count {
        let uniform = gl
            .get_active_uniform(program, i as u32)
            .expect("uniform not found");
        let location = gl
            .get_uniform_location(program, &uniform.name)
            .expect("uniform location not found");

        enum SomeUniformKind {
            Sampler2d,
            Uniform(UniformKind),
            Unsupported,
        }

        let kind = match uniform.utype {
            glow::SAMPLER_2D => SomeUniformKind::Sampler2d,

            glow::FLOAT => SomeUniformKind::Uniform(UniformKind::Float),
            glow::FLOAT_VEC2 => SomeUniformKind::Uniform(UniformKind::FloatVec2),
            glow::FLOAT_VEC3 => SomeUniformKind::Uniform(UniformKind::FloatVec3),
            glow::FLOAT_VEC4 => SomeUniformKind::Uniform(UniformKind::FloatVec4),
            glow::INT | glow::UNSIGNED_INT | glow::BOOL => {
                SomeUniformKind::Uniform(UniformKind::Int)
            }
            glow::INT_VEC2 | glow::UNSIGNED_INT_VEC2 | glow::BOOL_VEC2 => {
                SomeUniformKind::Uniform(UniformKind::IntVec2)
            }
            glow::INT_VEC3 | glow::UNSIGNED_INT_VEC3 | glow::BOOL_VEC3 => {
                SomeUniformKind::Uniform(UniformKind::IntVec3)
            }
            glow::INT_VEC4 | glow::UNSIGNED_INT_VEC4 | glow::BOOL_VEC4 => {
                SomeUniformKind::Uniform(UniformKind::IntVec4)
            }
            glow::FLOAT_MAT2 => SomeUniformKind::Uniform(UniformKind::Mat2),

            glow::FLOAT_MAT3 => SomeUniformKind::Uniform(UniformKind::Mat3),
            glow::FLOAT_MAT4 => SomeUniformKind::Uniform(UniformKind::Mat4),
            _ => SomeUniformKind::Unsupported,
        };

        match kind {
            SomeUniformKind::Uniform(kind) => {
                uniforms.insert(
                    uniform.name.clone(),
                    (location, kind, kind.size() * uniform.size as usize),
                );
            }
            SomeUniformKind::Sampler2d => {
                samplers.insert(uniform.name.clone(), (location, 0));
            }
            SomeUniformKind::Unsupported => {
                panic!(
                    "Unsupported shader uniform type, name: {}, type: {}",
                    uniform.name, uniform.utype
                );
            }
        }
    }

    (uniforms, samplers)
}

unsafe fn get_vertex_attributes(
    gl: &glow::Context,
    program: glow::Program,
) -> HashMap<String, u32> {
    let attribute_count = gl.get_active_attributes(program) as usize;
    let mut attributes = HashMap::with_capacity(attribute_count);

    for i in 0..attribute_count {
        let attribute = gl
            .get_active_attribute(program, i as u32)
            .expect("attribute not found");
        let location = gl
            .get_attrib_location(program, &attribute.name)
            .expect("attribute location not found");

        attributes.insert(attribute.name, location);
    }

    attributes
}

impl<B: WindowBackend> Shader<Gles<B>> for GlesShader<B> {
    type Source = TextShaderSource<'static>;

    fn new(ctx: Gles<B>, source: &TextShaderSource) -> Self {
        let gl = &ctx.gl;

        unsafe {
            let program = compile_program(&gl, source);
            let (uniform_attributes, texture_attributes) = get_uniforms(&gl, program);
            let attribute_data = get_vertex_attributes(&gl, program);

            Self {
                ctx,
                program,
                uniform_attributes,
                attribute_data,
                state: RefCell::new(ShaderState {
                    sampler_attributes: texture_attributes,
                    uniforms_cache: (<()>::FORMAT, Vec::new()),
                }),
            }
        }
    }
}

impl<B: WindowBackend> Drop for GlesShader<B> {
    fn drop(&mut self) {
        unsafe {
            let mut ctx = self.ctx.get_ref();
            if ctx.state.bound_program == Some(self.program) {
                ctx.use_program(None);
            }

            self.ctx.gl.delete_program(self.program);
        }
    }
}
