use yapgeir_geometry::Size;

use crate::Graphics;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderBufferFormat {
    Depth,
    Stencil,
    DepthStencil,
}

pub trait RenderBuffer<G: Graphics> {
    type Format;

    fn new(renderer: G, size: Size<u32>, format: Self::Format) -> Self;
}
