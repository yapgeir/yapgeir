use crate::Graphics;
use bytemuck::Pod;
use derive_more::Constructor;

pub use yapgeir_graphics_hal_macro::Uniforms;

#[derive(Constructor, Clone, PartialEq)]
pub struct UniformAttribute {
    pub name: &'static str,
    pub offset: usize,
    pub size: usize,
}

pub trait Uniforms {
    const FORMAT: &'static [UniformAttribute];
}

pub trait UniformBuffer<G: Graphics, T: Pod> {
    fn new(g: G, initial: &T) -> Self;
    fn write(&self, value: &T);
}

impl Uniforms for () {
    const FORMAT: &'static [UniformAttribute] = &[];
}
