pub use yapgeir_graphics_hal_macro::Samplers;

use crate::{
    primitives::{ImageSize, Rect},
    Graphics,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelFormat {
    Alpha,
    Lumi,
    Lumia,
    Rgb,
    Rgba,
}

pub trait Texture<G: Graphics> {
    type PixelFormat: From<PixelFormat>;

    fn new(renderer: G, format: G::PixelFormat, size: ImageSize<u32>, bytes: Option<&[u8]>)
        -> Self;

    fn size(&self) -> ImageSize<u32>;

    fn write(&self, mipmap_level: u32, format: G::PixelFormat, size: ImageSize<u32>, bytes: &[u8]);

    fn write_rect(&self, mipmap_level: u32, format: G::PixelFormat, rect: Rect<u32>, bytes: &[u8]);

    fn generate_mipmaps(&self);
}
