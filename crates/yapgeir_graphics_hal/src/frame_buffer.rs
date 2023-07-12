use std::{borrow::Borrow, rc::Rc};

use bytemuck::Pod;
use derive_more::Constructor;

use crate::{
    draw_params::DrawParameters,
    index_buffer::PrimitiveMode,
    primitives::{Rect, Rgba},
    samplers::SamplerAttribute,
    uniforms::Uniforms,
    Graphics, ImageSize,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderBufferFormat {
    Depth,
    Stencil,
    DepthStencil,
}

pub trait RenderBuffer<G: Graphics> {
    type Format;

    fn new(renderer: G, size: ImageSize<u32>, format: Self::Format) -> Self;
}

pub enum Attachment<G: Graphics> {
    Texture(Rc<G::Texture>),
    RenderBuffer(Rc<G::RenderBuffer>),
}

pub enum DepthStencilAttachment<R: Graphics> {
    None,
    Depth(Attachment<R>),
    Stencil(Attachment<R>),
    DepthStencil(Attachment<R>),
    DepthAndStencil {
        depth: Attachment<R>,
        stencil: Attachment<R>,
    },
}

#[derive(Default)]
pub enum IndicesSource<B> {
    #[default]
    Default,
    Buffer(B),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadFormat {
    Alpha,
    Rgb,
    Rgba,
}

#[derive(Constructor, Debug, Clone)]
pub struct Indices {
    pub mode: PrimitiveMode,
    pub offset: usize,
    pub len: usize,
}

pub trait FrameBuffer<G: Graphics> {
    /// A FrameBuffer implementation can support extended read formats,
    /// but all of the implementations are guaranteed to support at least
    /// Alpha, Rgb and Rgba with 8 bits per component.
    type ReadFormat;

    /// Return the default framebuffer, which is bound to a screen.
    fn default(renderer: G) -> Self;

    /// Create a new framebuffer.
    ///
    /// A framebuffer uses a Texture for a depth component,
    /// and can optionally have depth and/or stencil components.
    ///
    /// Depth and stencil components can be a texture or a renderbuffer.
    fn new(renderer: G, draw: Rc<G::Texture>, depth_stencil: DepthStencilAttachment<G>) -> Self;

    /// Returns the size of the framebuffer in pixels.
    fn size(&self) -> ImageSize<u32>;

    /// Reset values in the underlying draw depth and stencil buffers
    /// that are covered by a scissor rectangle to a constant value.
    ///
    /// If `scissor` is `None`, uses the clear the whole framebuffer.
    /// Note that `scissor` is in Y-down coordinate space regardless of
    /// the implementation, meaning that a (0, 0) point is in the left
    /// top corner.
    ///
    /// This function only clears components, which are present in the
    /// arguments.
    fn clear(
        &self,
        scissor: Option<Rect<u32>>,
        color: Option<Rgba<f32>>,
        depth: Option<f32>,
        stencil: Option<u8>,
    );

    /// Draws the vertices on the framebuffer.
    ///
    /// # Arguments
    ///
    /// * `draw_descriptor` - a set of vertices and optionally and index buffer.
    /// Think of it as of a vertex array object in OpenGL terms.
    /// * `draw_parameters` - a set of parameters that control how the vertices are drawn.
    /// Note that a `viewport` and `scissor` are in Y-down coordinate space regardless of
    /// the implementation, meaning that a (0, 0) point is in the left
    /// top corner.
    /// * `textures` - a set of samplers that will be used in a shader. Describes
    /// which textures are used, how they are sampled, and where are they bound to.
    /// * `uniforms` - a set of uniforms that will be used in a shader. Only a
    /// single uniform buffer binding is supported.
    /// * `indices` - describes how to interpret the indices. Uses an index buffer
    /// that was bound to a `draw_descriptor`. If no index buffer was bound to
    /// a `draw_descriptor`, then indices are sequential.
    fn draw<U: Uniforms + Pod>(
        &self,
        draw_descriptor: &G::DrawDescriptor,
        draw_parameters: &DrawParameters,

        samplers: &[SamplerAttribute<G, impl Borrow<G::Texture>>],
        uniforms: Option<&G::UniformBuffer<U>>,
        indices: &Indices,
    );

    /// Reads the data from the framebuffers draw texture to the provided
    /// byte slice.
    fn read(&self, rect: Rect<u32>, read_format: Self::ReadFormat, target: &mut [u8]);
}
