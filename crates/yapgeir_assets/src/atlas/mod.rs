use derive_more::Constructor;
use std::{collections::HashMap, ops::RangeInclusive};
use yapgeir_world_2d::SubTexture;

pub mod ase;

#[derive(Debug, Clone)]
pub struct Sprite {
    /// Sprite size in pixels
    pub size: (u32, u32),
    /// Contains clip rect and coordinates on an atlas
    pub sub_texture: SubTexture,
}

#[derive(Debug, Constructor)]
pub struct Atlas {
    pub sprites: HashMap<String, Sprite>,
    pub frame_tags: HashMap<String, RangeInclusive<usize>>,
}
