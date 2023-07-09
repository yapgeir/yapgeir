use std::{mem::size_of, ops::Deref, rc::Rc};

use bytemuck::Pod;

use crate::{
    buffer::Buffer,
    index_buffer::{Index, IndexKind},
    vertex_buffer::{Vertex, VertexAttribute},
    Graphics,
};

/// A DrawDescriptor is a structure which defines a relationship between
/// vertex buffers, index buffer and a shader program, and keeps references to
/// all of these GPU objects.
///
/// This is essentially a Vertex Array Object in terms of OpenGL.
pub trait DrawDescriptor<G: Graphics> {
    fn new(
        renderer: G,
        shader: Rc<G::Shader>,
        indices: IndexBinding<G>,
        vertices: &[VertexBindings<G>],
    ) -> Self;
}

/// IndexBinding defines zero or one index buffers with erased type information.
/// This is used to reduce the amount of generics and unnecessarily duplicated generated code.
pub enum IndexBinding<G: Graphics> {
    /// Index buffer will not be used for this draw descriptor.
    None,

    /// An index buffer of a specific index kind will be used (u8, u16 or u32)
    Some {
        buffer: Rc<G::ByteBuffer>,
        kind: IndexKind,
    },
}

/// IndexBinding is Clone. This may be useful when you want to share an index buffer
/// between different draw descriptors, for example for quad indices.
/// With Clone you can keep your index buffer resource (e.g. in ECS) with erased type
/// and decide which type to use for the buffer in runtime.
impl<G: Graphics> Clone for IndexBinding<G> {
    fn clone(&self) -> Self {
        match self {
            IndexBinding::None => IndexBinding::None,
            IndexBinding::Some { buffer, kind } => IndexBinding::Some {
                buffer: buffer.clone(),
                kind: kind.clone(),
            },
        }
    }
}

impl<G: Graphics, T, I: Index> From<Option<T>> for IndexBinding<G>
where
    T: Deref<Target = Buffer<G, I>>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            None => Self::None,
            Some(value) => Self::Some {
                buffer: value.bytes.clone(),
                kind: I::KIND,
            },
        }
    }
}

/// VertexBindings describes how attributes are bound to a shader.
/// This is essentially a buffer with an erased data type and with some
/// data type data. Type erasure is necessary to allow binding multiple
/// vertex buffers to a single draw descriptor in a slice.
///
/// In modern graphics implementations attributes most likely will be unused,
/// they would normally define your vertices as structs in a shader.
#[derive(Clone)]
pub struct VertexBindings<'a, G: Graphics> {
    /// A vertex buffer that will be bound during a draw call.
    pub buffer: Rc<G::ByteBuffer>,

    /// A slice attributes for the given vertex. Essential for GLES2 implementation.
    /// Defines each field of the vertex that will be bound.
    pub attributes: &'a [VertexAttribute],

    /// A size of the data type T of the buffer.
    pub stride: usize,
}

/// Converts vertex data to bindings.
pub trait AsVertexBindings<G: Graphics> {
    fn bindings<'a>(&'a self) -> VertexBindings<'a, G>;
}

impl<T: Vertex + Pod, G: Graphics> AsVertexBindings<G> for Buffer<G, T> {
    fn bindings<'a>(&'a self) -> VertexBindings<'a, G> {
        VertexBindings {
            buffer: self.bytes.clone(),
            attributes: T::FORMAT,
            stride: size_of::<T>(),
        }
    }
}
