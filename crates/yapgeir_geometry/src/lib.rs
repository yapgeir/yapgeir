use std::ops::{Add, Sub};

use derive_more::Constructor;

#[cfg(feature = "reflection")]
use yapgeir_reflection::bevy_reflect::{self, Reflect};

/// ImageSize<u32> is a structure that is generally used to denote image, texture and frame buffer sizes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Size<T> {
    pub w: T,
    pub h: T,
}

impl<T> Size<T> {
    #[inline]
    pub const fn new(w: T, h: T) -> Self {
        Self { w, h }
    }
}

impl<T> From<(T, T)> for Size<T> {
    #[inline]
    fn from((w, h): (T, T)) -> Self {
        Self::new(w, h)
    }
}

impl<T> From<[T; 2]> for Size<T> {
    #[inline]
    fn from([w, h]: [T; 2]) -> Self {
        Self::new(w, h)
    }
}

impl<T> From<Size<T>> for [T; 2] {
    #[inline]
    fn from(value: Size<T>) -> Self {
        [value.w, value.h]
    }
}

impl From<Size<u32>> for (u32, u32) {
    #[inline]
    fn from(value: Size<u32>) -> Self {
        (value.w, value.h)
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Rgba<T> {
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T,
}

impl<T> Rgba<T> {
    #[inline]
    pub const fn new(r: T, g: T, b: T, a: T) -> Self {
        Self { r, g, b, a }
    }

    #[inline]
    pub fn all(value: T) -> Self
    where
        T: Copy,
    {
        Self::new(value, value, value, value)
    }
}

impl<T> From<(T, T, T, T)> for Rgba<T> {
    #[inline]
    fn from((r, g, b, a): (T, T, T, T)) -> Self {
        Self::new(r, g, b, a)
    }
}

impl<T> From<[T; 4]> for Rgba<T> {
    #[inline]
    fn from([r, g, b, a]: [T; 4]) -> Self {
        Self::new(r, g, b, a)
    }
}

impl<T> From<Rgba<T>> for [T; 4] {
    #[inline]
    fn from(value: Rgba<T>) -> Self {
        [value.r, value.g, value.b, value.a]
    }
}

/// A rectangle defined as an origin point and a size.
#[derive(Constructor, Default, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
pub struct Rect<T> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
}

impl<T: Default> From<Size<T>> for Rect<T> {
    #[inline]
    fn from(value: Size<T>) -> Self {
        Self::new(T::default(), T::default(), value.w, value.h)
    }
}

impl<T> From<(T, T, T, T)> for Rect<T> {
    #[inline]
    fn from((x, y, w, h): (T, T, T, T)) -> Self {
        Self::new(x, y, w, h)
    }
}

impl From<&Rect<u32>> for Rect<i32> {
    #[inline]
    fn from(value: &Rect<u32>) -> Self {
        Self::new(
            value.x as i32,
            value.y as i32,
            value.w as i32,
            value.h as i32,
        )
    }
}

impl<T> From<Rect<T>> for Box2D<T>
where
    T: Copy + Add<T, Output = T>,
{
    #[inline]
    fn from(rect: Rect<T>) -> Self {
        Self {
            a: [rect.x, rect.y],
            b: [rect.x + rect.w, rect.y + rect.h],
        }
    }
}

impl<T> Rect<T> {
    #[inline]
    pub fn size(&self) -> Size<T>
    where
        T: Copy,
    {
        Size {
            w: self.w,
            h: self.h,
        }
    }

    #[inline]
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

/// A representation of a rectangle by two points of a rectangles diagonal.
#[derive(Constructor, Default, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
pub struct Box2D<T> {
    pub a: [T; 2],
    pub b: [T; 2],
}

impl<T> Box2D<T> {
    #[inline]
    pub fn points(&self) -> [[T; 2]; 4]
    where
        T: Copy,
    {
        [
            [self.a[0], self.a[1]],
            [self.a[0], self.b[1]],
            [self.b[0], self.b[1]],
            [self.b[0], self.a[1]],
        ]
    }

    pub fn size(&self) -> Size<T>
    where
        T: Copy + Sub<T, Output = T> + PartialOrd,
    {
        Size {
            w: match self.a[0] < self.b[0] {
                true => self.b[0] - self.a[0],
                false => self.a[0] - self.b[0],
            },
            h: match self.a[1] < self.b[1] {
                true => self.b[1] - self.a[1],
                false => self.a[1] - self.b[1],
            },
        }
    }
}

impl<T> From<Box2D<T>> for Rect<T>
where
    T: Copy + Sub<T, Output = T> + PartialOrd,
{
    fn from(value: Box2D<T>) -> Self {
        let (x, w) = match value.a[0] < value.b[0] {
            true => (value.a[0], value.b[0] - value.a[0]),
            false => (value.b[0], value.a[0] - value.b[0]),
        };
        let (y, h) = match value.a[1] < value.b[1] {
            true => (value.a[1], value.b[1] - value.a[1]),
            false => (value.b[1], value.a[1] - value.b[1]),
        };

        Self { x, y, w, h }
    }
}
