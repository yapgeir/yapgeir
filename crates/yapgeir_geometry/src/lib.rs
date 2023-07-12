use std::ops::Add;

use derive_more::Constructor;
use yapgeir_reflection::bevy_reflect::{self, Reflect};

/// A rectangle defined as an origin point and a size.
#[derive(Constructor, Default, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct Rect<T> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
}

impl<T> Rect<T> {
    pub fn points(&self) -> [[T; 2]; 4]
    where
        T: Add<Output = T> + Copy,
    {
        [
            [self.x, self.y],
            [self.x, self.y + self.h],
            [self.x + self.w, self.y + self.h],
            [self.x + self.w, self.y],
        ]
    }
}

/// A representation of a rectangle by two points of a diagonal.
#[derive(Constructor, Default, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct Box2D<T> {
    pub min: [T; 2],
    pub max: [T; 2],
}

impl<T> Box2D<T> {
    #[inline]
    pub fn points(&self) -> [[T; 2]; 4]
    where
        T: Copy,
    {
        [
            [self.min[0], self.min[1]],
            [self.min[0], self.max[1]],
            [self.max[0], self.max[1]],
            [self.max[0], self.min[1]],
        ]
    }
}
