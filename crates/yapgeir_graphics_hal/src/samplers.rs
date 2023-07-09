use std::borrow::Borrow;

use derive_more::Constructor;

use crate::{sampler::Sampler, Graphics};

/// An array of SamplerAttributes is passed to a draw call.
/// The texture samplers are bound to free texture units (slots)
/// and bound to shader uniforms by the implementation.
#[derive(Constructor, Clone)]
pub struct SamplerAttribute<G: Graphics, T: Borrow<G::Texture>> {
    pub name: &'static str,
    pub location: u8,
    pub sampler: Sampler<G, T>,
}

impl<'a, G: Graphics + 'a> SamplerAttribute<G, &'a G::Texture> {
    pub fn named<const N: usize, T: Borrow<G::Texture> + 'a>(
        attributes: [(&'static str, &'a Sampler<G, T>); N],
    ) -> [Self; N] {
        let mut location = 0;
        attributes.map(|(name, sampler)| {
            let attribute = Self::new(name, location, sampler.as_borrowed());
            location += 1;
            attribute
        })
    }
}

pub trait Samplers<G: Graphics, const N: usize> {
    fn attributes<'a>(&'a self) -> [SamplerAttribute<G, &'a G::Texture>; N];
}

impl<G: Graphics> Samplers<G, 0> for () {
    fn attributes<'a>(&'a self) -> [SamplerAttribute<G, &'a G::Texture>; 0] {
        []
    }
}

#[macro_export]
macro_rules! samplers {
    (@step $_idx:expr,) => {};

    (@step $idx:expr, $name:ident => $val:expr $(, $($tail:tt)*)?) => {
        yapgeir_graphics_hal::samplers::SamplerAttribute {
            name: stringify!($name),
            location: $idx,
            sampler: $val,
        }

        $(samplers!(@step $idx + 1u8, $($tail)*))?
    };

    ($($n:tt)*) => {
        (&[samplers!(@step 0u8, $($n)*)])
    };
}
