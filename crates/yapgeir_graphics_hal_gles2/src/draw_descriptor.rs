use std::rc::Rc;

use crate::{
    buffer::GlesBuffer, constants::GlConstant, context::GlesContextRef, shader::GlesShader, Gles,
};
use glow::HasContext;
use yapgeir_graphics_hal::{
    buffer::BufferKind,
    draw_descriptor::{DrawDescriptor, IndexBinding, VertexBindings},
    index_buffer::IndexKind,
    vertex_buffer::VertexAttribute,
    Backend,
};

pub(crate) enum GlesDrawDescriptorImpl<B: Backend> {
    Vao(vao::GlesDrawDescriptor<B>),
    Fallback(fallback::GlesDrawDescriptor<B>),
}

pub struct GlesDrawDescriptor<B: Backend> {
    pub(crate) shader: Rc<GlesShader<B>>,
    pub(crate) index_kind: Option<IndexKind>,

    inner: GlesDrawDescriptorImpl<B>,
}

impl<B: Backend> DrawDescriptor<Gles<B>> for GlesDrawDescriptor<B> {
    fn new(
        ctx: Gles<B>,
        shader: Rc<GlesShader<B>>,
        indices: IndexBinding<Gles<B>>,
        vertices: &[VertexBindings<Gles<B>>],
    ) -> Self {
        Self {
            index_kind: match indices {
                IndexBinding::None => None,
                IndexBinding::Some { kind, .. } => Some(kind),
            },
            inner: if ctx.features.vertex_array_objects {
                GlesDrawDescriptorImpl::Vao(vao::GlesDrawDescriptor::new(
                    ctx,
                    shader.clone(),
                    indices,
                    vertices,
                ))
            } else {
                GlesDrawDescriptorImpl::Fallback(fallback::GlesDrawDescriptor::new(
                    ctx, indices, vertices,
                ))
            },
            shader,
        }
    }
}

impl<B: Backend> GlesDrawDescriptor<B> {
    pub(crate) fn bind(&self, ctx: &mut GlesContextRef) {
        match &self.inner {
            GlesDrawDescriptorImpl::Vao(vao) => vao.bind(ctx),
            GlesDrawDescriptorImpl::Fallback(fallback) => fallback.bind(ctx, &self.shader),
        }
    }
}

#[derive(Default)]
pub struct DrawDescriptorCache {
    counter: usize,
    current: usize,
}

struct Bindings<'a, B: Backend> {
    buffer: &'a GlesBuffer<B>,
    attributes: &'a [VertexAttribute],
    stride: usize,
}

unsafe fn bind_buffers<'a, B: Backend>(
    ctx: &mut GlesContextRef,
    shader: &GlesShader<B>,
    indices: &IndexBinding<Gles<B>>,
    vertices: impl Iterator<Item = Bindings<'a, B>>,
) {
    if let IndexBinding::Some { buffer, .. } = &indices {
        ctx.bind_buffer(BufferKind::Index, Some(buffer.buffer));
    } else {
        ctx.bind_buffer(BufferKind::Index, None);
    }

    for vertex in vertices {
        ctx.bind_buffer(BufferKind::Vertex, Some(vertex.buffer.buffer));
        let stride = vertex.stride as i32;

        for attribute in vertex.attributes {
            // Find attribute data from shader by name.
            let location = shader.attribute_data.get(attribute.name).cloned();

            // If attribute data is found, enable and set attribute pointer.
            if let Some(location) = location {
                ctx.gl.enable_vertex_attrib_array(location);
                ctx.gl.vertex_attrib_pointer_f32(
                    location,
                    attribute.size.size() as i32,
                    attribute.kind.gl_const(),
                    false,
                    stride,
                    attribute.offset as i32,
                );
            } else {
                continue;
            }
        }
    }
}

mod vao {
    use std::rc::Rc;

    use glow::HasContext;
    use yapgeir_graphics_hal::{
        draw_descriptor::{IndexBinding, VertexBindings},
        Backend,
    };

    use crate::{buffer::GlesBuffer, context::GlesContextRef, shader::GlesShader, Gles};

    use super::bind_buffers;

    pub struct GlesDrawDescriptor<B: Backend> {
        ctx: Gles<B>,
        vao: glow::VertexArray,
        _indices: IndexBinding<Gles<B>>,
        _vertices: Vec<Rc<GlesBuffer<B>>>,
    }

    impl<B: Backend> Drop for GlesDrawDescriptor<B> {
        fn drop(&mut self) {
            let mut ctx = self.ctx.get_ref();
            if ctx.state.bound_vertex_array == Some(self.vao) {
                ctx.bind_vertex_array(None);
            }

            unsafe { ctx.gl.delete_vertex_array(self.vao) };
        }
    }

    impl<B: Backend> GlesDrawDescriptor<B> {
        pub fn new(
            ctx: Gles<B>,
            shader: Rc<GlesShader<B>>,
            indices: IndexBinding<Gles<B>>,
            vertices: &[VertexBindings<Gles<B>>],
        ) -> Self {
            let vao: glow::NativeVertexArray = unsafe {
                let mut ctx = ctx.get_ref();
                let vao = ctx
                    .gl
                    .create_vertex_array()
                    .expect("Unable to generate vertex array.");

                ctx.bind_vertex_array(Some(vao));
                bind_buffers(
                    &mut ctx,
                    &shader,
                    &indices,
                    vertices.iter().map(|v| super::Bindings {
                        buffer: &v.buffer,
                        attributes: v.attributes,
                        stride: v.stride,
                    }),
                );

                vao
            };

            Self {
                ctx,
                vao,
                _indices: indices,
                _vertices: vertices.iter().map(|v| v.buffer.clone()).collect(),
            }
        }

        pub(crate) fn bind(&self, ctx: &mut GlesContextRef) {
            ctx.bind_vertex_array(Some(self.vao));
        }
    }
}

mod fallback {
    use std::rc::Rc;

    use yapgeir_graphics_hal::{
        draw_descriptor::{IndexBinding, VertexBindings},
        vertex_buffer::VertexAttribute,
        Backend,
    };

    use crate::{buffer::GlesBuffer, context::GlesContextRef, shader::GlesShader, Gles};

    use super::bind_buffers;

    struct OwnedBindings<B: Backend> {
        buffer: Rc<GlesBuffer<B>>,
        attributes: Vec<VertexAttribute>,
        stride: usize,
    }

    pub struct GlesDrawDescriptor<B: Backend> {
        id: usize,
        indices: IndexBinding<Gles<B>>,
        vertices: Vec<OwnedBindings<B>>,
    }

    impl<B: Backend> GlesDrawDescriptor<B> {
        pub fn new(
            ctx: Gles<B>,
            indices: IndexBinding<Gles<B>>,
            vertices: &[VertexBindings<Gles<B>>],
        ) -> Self {
            let mut ctx = ctx.get_ref();

            // We'll start id's with 1.
            ctx.state.draw_descriptor_cache.counter += 1;
            let id = ctx.state.draw_descriptor_cache.counter;

            Self {
                id,
                indices,
                vertices: vertices
                    .iter()
                    .map(|v| OwnedBindings {
                        buffer: v.buffer.clone(),
                        attributes: v.attributes.to_vec(),
                        stride: v.stride,
                    })
                    .collect(),
            }
        }

        pub(crate) fn bind(&self, ctx: &mut GlesContextRef, shader: &GlesShader<B>) {
            if ctx.state.draw_descriptor_cache.current == self.id {
                return;
            }
            ctx.state.draw_descriptor_cache.current = self.id;

            unsafe {
                bind_buffers(
                    ctx,
                    shader,
                    &self.indices,
                    self.vertices.iter().map(|v| super::Bindings {
                        buffer: &v.buffer,
                        attributes: &v.attributes,
                        stride: v.stride,
                    }),
                );
            }
        }
    }
}
