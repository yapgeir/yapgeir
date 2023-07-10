use derive_more::Constructor;
use std::{collections::HashMap, ops::RangeInclusive};

use nalgebra::Vector2;
use yapgeir_geometry::Rect;

pub mod ase;

#[derive(Debug, Clone)]
pub struct Sprite {
    /// Sprite size in meters
    pub size: Vector2<f32>,
    /// Contains clip rect and coordinates on an atlas
    pub sub_texture: SubTexture,
}

// Defines a single drawable object by it's coordinates on a texture
// and a crop factor
#[derive(Debug, Clone, Default)]
pub struct SubTexture {
    /// This is the clip rectangle in centered texture space
    pub clip: Rect,
    /// This is the location of a sprite on an atlas in texture space
    pub sprite: Rect,
}

#[derive(Debug, Constructor)]
pub struct Atlas {
    pub sprites: HashMap<String, Sprite>,
    pub frame_tags: HashMap<String, RangeInclusive<usize>>,
}
