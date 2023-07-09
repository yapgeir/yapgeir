use std::{fmt::Debug, num::TryFromIntError};

use derive_more::Deref;
use yapgeir_graphics_hal::{
    buffer::{BufferKind, BufferUsage},
    draw_descriptor::IndexBinding,
    index_buffer::Index,
    Graphics,
};

#[derive(Clone, Deref)]
pub struct QuadIndexBuffer<G: Graphics>(IndexBinding<G>);

#[inline]
fn create_quad_indices<I>(quads: usize) -> Result<Vec<I>, TryFromIntError>
where
    I: TryFrom<usize, Error = TryFromIntError> + Index,
{
    let mut array = Vec::<I>::with_capacity(quads * 6);
    let mut quad = 0;
    while quad < quads {
        let j = quad * 4;
        array.push(j.try_into()?);
        array.push((j + 1).try_into()?);
        array.push((j + 2).try_into()?);
        array.push(j.try_into()?);
        array.push((j + 2).try_into()?);
        array.push((j + 3).try_into()?);
        quad += 1;
    }

    Ok(array)
}

impl<G: Graphics> QuadIndexBuffer<G> {
    pub fn new<I: Index + Into<usize> + TryFrom<usize, Error = TryFromIntError> + Debug>(
        ctx: &G,
        size: I,
    ) -> Self {
        let indices = create_quad_indices::<I>(size.into()).expect("Unable to create quad indices");
        let buffer = ctx.new_buffer(BufferKind::Index, BufferUsage::Static, &indices);
        Self(Some(&buffer).into())
    }
}
