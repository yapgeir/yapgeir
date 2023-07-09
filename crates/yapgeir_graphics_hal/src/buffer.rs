use std::{marker::PhantomData, mem::size_of, rc::Rc};

use bytemuck::Pod;
use enum_map::Enum;

use crate::Graphics;

/// BufferKind defines a type of a buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Enum)]
pub enum BufferKind {
    /// A buffer that will be used for indexes.
    /// Zero or one vertex buffer can be bound to a draw descriptor, and used for a draw call.
    Index,
    /// A buffer that will be used for vertex data.
    /// Multiple vertex buffers can be bound to a draw descriptor, and used for a draw call.
    Vertex,
}

/// BufferUsage is a hint for GPU describing how the buffer is going to be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferUsage {
    /// Buffer data will not be changed once it's uploaded.
    Static,
    /// Data will be updated occasionally.
    Dynamic,
    /// Data will be written after (almost) every use.
    /// Use this when you update your buffer every frame from every draw call.
    Stream,
}

/// BufferData is used to initialize the buffer when it is created.
pub enum BufferData<'a, T> {
    /// Buffer will be initialized with the given data.
    Data(&'a [T]),
    /// Buffer will be initialized with zeroes with the given size in T.
    Empty(usize),
}

impl<'a, T> From<&'a [T]> for BufferData<'a, T> {
    fn from(value: &'a [T]) -> Self {
        BufferData::Data(value)
    }
}

impl<'a, T, const N: usize> From<&'a [T; N]> for BufferData<'a, T> {
    fn from(value: &'a [T; N]) -> Self {
        BufferData::Data(value)
    }
}

impl<'a, T> From<&'a Vec<T>> for BufferData<'a, T> {
    fn from(value: &'a Vec<T>) -> Self {
        BufferData::Data(value)
    }
}

impl<T> From<usize> for BufferData<'static, T> {
    fn from(value: usize) -> Self {
        BufferData::Empty(value)
    }
}

impl<'a, T> BufferData<'a, T> {
    /// Returns the length of the buffer data in T.
    pub fn len(&self) -> usize {
        match self {
            BufferData::Data(data) => data.len(),
            BufferData::Empty(len) => *len,
        }
    }
}

impl<'a, T: Pod> BufferData<'a, T> {
    pub(crate) fn bytes(&self) -> BufferData<'a, u8> {
        match self {
            BufferData::Data(data) => BufferData::Data(bytemuck::cast_slice(*data)),
            BufferData::Empty(len) => BufferData::Empty(len * size_of::<T>()),
        }
    }
}

/// ByteBuffer trait defines the API for buffers allocated on a GPU.
///
/// It is parameterized with Usage, which is a hint telling GPU how a buffer
/// will be used (is it immutable, or does user code write to the buffer frequently or infrequently).
///
/// The supported buffer kinds are Vertex buffers and Index buffers.
pub trait ByteBuffer<G: Graphics> {
    /// Hint for a GPU telling how the buffer will be used.
    type Usage;

    /// Creates a new buffer on a GPU with a given kind and usage.
    /// If BufferData is Empty, zero allocates the buffer to a given size.
    /// If BufferData is Data, allocates buffer to a size of the data slice, and writes it.
    fn new<'a>(renderer: G, kind: BufferKind, usage: Self::Usage, data: BufferData<'a, u8>)
        -> Self;

    /// Returns the length of the buffer in bytes.
    fn len(&self) -> usize;

    /// Writes the data to a buffer at a given offset.
    /// Panics if the data stretches beyond the buffer boundaries.
    fn write(&self, offset: usize, data: &[u8]);
}

/// Buffer is a type retaining proxy for a ByteBuffer.
/// It allows creating, getting length and writing to a ByteBuffer
/// by converting slices of underlying type T to a byte slice with bytemuck.
#[derive(Clone)]
pub struct Buffer<G: Graphics, T: Pod> {
    pub bytes: Rc<G::ByteBuffer>,
    _t: PhantomData<T>,
}

impl<G: Graphics, T: Pod> Buffer<G, T> {
    /// Create a new Buffer.
    ///
    /// This method creates the underlying ByteBuffer on a GPU with given parameters.
    pub(crate) fn new<'a>(
        renderer: G,
        kind: BufferKind,
        usage: G::BufferUsage,
        data: BufferData<'a, T>,
    ) -> Self {
        Self {
            bytes: Rc::new(G::ByteBuffer::new(renderer, kind, usage, data.bytes())),
            _t: PhantomData,
        }
    }

    /// Returns the size of the buffer in T.
    pub fn len(&self) -> usize {
        self.bytes.len() / size_of::<T>()
    }

    /// Writes the data to a buffer at a given offset.
    /// Panics if the data stretches beyond the buffer boundaries.
    pub fn write(&self, offset: usize, data: &[T]) {
        let data = bytemuck::cast_slice(data);
        self.bytes.write(offset * size_of::<T>(), data);
    }
}
