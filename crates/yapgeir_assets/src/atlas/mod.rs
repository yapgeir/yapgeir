use derive_more::Constructor;
use std::{collections::HashMap, ops::RangeInclusive};
use yapgeir_world_2d::Drawable;

pub mod ase;

#[derive(Debug, Constructor)]
pub struct Atlas {
    pub drawables: HashMap<String, Drawable>,
    pub frame_tags: HashMap<String, RangeInclusive<usize>>,
}
