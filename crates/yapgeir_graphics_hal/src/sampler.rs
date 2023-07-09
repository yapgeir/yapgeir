use std::{borrow::Borrow, marker::PhantomData};

use crate::Graphics;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Filter {
    #[default]
    Linear,
    Nearest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MinFilter {
    Origin(Filter),
    Mipmap { mipmap: Filter, texel: Filter },
}

impl Default for MinFilter {
    fn default() -> Self {
        Self::Mipmap {
            mipmap: Filter::Linear,
            texel: Filter::Nearest,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WrapFunction {
    Clamp,
    #[default]
    Repeat,
    MirrorClamp,
    MirrorRepeat,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SamplerState {
    pub wrap: WrapFunction,
    pub min_filter: MinFilter,
    pub mag_filter: Filter,
}

impl SamplerState {
    pub fn linear() -> Self {
        SamplerState {
            wrap: WrapFunction::Clamp,
            min_filter: MinFilter::Origin(Filter::Linear),
            mag_filter: Filter::Linear,
        }
    }

    pub fn nearest() -> Self {
        SamplerState {
            wrap: WrapFunction::Clamp,
            min_filter: MinFilter::Origin(Filter::Nearest),
            mag_filter: Filter::Nearest,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sampler<G: Graphics, T: Borrow<G::Texture> = <G as Graphics>::Texture> {
    pub texture: T,
    pub state: SamplerState,

    _g: PhantomData<G>,
}

impl<G: Graphics, T: Borrow<G::Texture>> Sampler<G, T> {
    pub fn new(texture: T, state: SamplerState) -> Self {
        Self {
            texture,
            state,
            _g: PhantomData,
        }
    }

    pub fn linear(texture: T) -> Self {
        Self::new(texture, SamplerState::linear())
    }

    pub fn nearest(texture: T) -> Self {
        Self::new(texture, SamplerState::nearest())
    }

    pub fn as_borrowed<'a>(&'a self) -> Sampler<G, &'a G::Texture> {
        Sampler::new(self.texture.borrow(), self.state)
    }
}
