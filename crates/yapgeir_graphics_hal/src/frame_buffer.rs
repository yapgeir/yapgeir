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
    type ReadFormat;

    fn default(renderer: G) -> Self;

    fn new(renderer: G, draw: Rc<G::Texture>, depth_stencil: DepthStencilAttachment<G>) -> Self;

    fn size(&self) -> ImageSize<u32>;

    fn clear(
        &self,
        scissor: Option<Rect<u32>>,
        color: Option<Rgba<f32>>,
        depth: Option<f32>,
        stencil: Option<u8>,
    );

    fn draw<U: Uniforms + Pod>(
        &self,
        draw_descriptor: &G::DrawDescriptor,
        draw_parameters: &DrawParameters,

        textures: &[SamplerAttribute<G, impl Borrow<G::Texture>>],
        uniforms: Option<&G::UniformBuffer<U>>,
        indices: &Indices,
    );

    fn read(&self, rect: Rect<u32>, read_format: Self::ReadFormat, target: &mut [u8]);
}
