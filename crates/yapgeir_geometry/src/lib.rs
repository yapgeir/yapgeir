use std::ops::{Add, Sub};

use derive_more::Constructor;
use yapgeir_reflection::bevy_reflect::{self, Reflect};

/// A rectangle defined as an origin point and a size.
#[derive(Constructor, Default, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
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
#[derive(Constructor, Default, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
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

impl<T> From<Box2D<T>> for Rect<T>
where
    T: Copy + Sub<T, Output = T>,
{
    fn from(box2d: Box2D<T>) -> Self {
        Self {
            x: box2d.min[0],
            y: box2d.min[1],
            w: box2d.max[0] - box2d.min[0],
            h: box2d.max[1] - box2d.min[1],
        }
    }
}
