use crate::Graphics;

#[derive(Debug, Clone)]
pub struct TextShaderSource<'a> {
    pub vertex: &'a str,
    pub fragment: &'a str,
}

pub trait Shader<G: Graphics> {
    type Source;

    fn new(renderer: G, source: &Self::Source) -> Self;
}
