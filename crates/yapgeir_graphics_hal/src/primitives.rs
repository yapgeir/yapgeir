use std::ops::Add;

use bytemuck::Zeroable;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T> From<(T, T)> for Point<T> {
    fn from((x, y): (T, T)) -> Self {
        Self::new(x, y)
    }
}

impl<T> From<[T; 2]> for Point<T> {
    fn from([x, y]: [T; 2]) -> Self {
        Self::new(x, y)
    }
}

impl<T> From<Point<T>> for [T; 2] {
    fn from(value: Point<T>) -> Self {
        [value.x, value.y]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageSize<T> {
    pub w: T,
    pub h: T,
}

impl<T> ImageSize<T> {
    pub const fn new(w: T, h: T) -> Self {
        Self { w, h }
    }
}

impl<T> From<(T, T)> for ImageSize<T> {
    fn from((w, h): (T, T)) -> Self {
        Self::new(w, h)
    }
}

impl<T> From<[T; 2]> for ImageSize<T> {
    fn from([w, h]: [T; 2]) -> Self {
        Self::new(w, h)
    }
}

impl<T> From<ImageSize<T>> for [T; 2] {
    fn from(value: ImageSize<T>) -> Self {
        [value.w, value.h]
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Zeroable)]
pub struct Rgba<T> {
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T,
}

impl<T> Rgba<T> {
    pub const fn new(r: T, g: T, b: T, a: T) -> Self {
        Self { r, g, b, a }
    }

    pub fn all(value: T) -> Self
    where
        T: Copy,
    {
        Self::new(value, value, value, value)
    }
}

impl<T> From<(T, T, T, T)> for Rgba<T> {
    fn from((r, g, b, a): (T, T, T, T)) -> Self {
        Self::new(r, g, b, a)
    }
}

impl<T> From<[T; 4]> for Rgba<T> {
    fn from([r, g, b, a]: [T; 4]) -> Self {
        Self::new(r, g, b, a)
    }
}

impl<T> From<Rgba<T>> for [T; 4] {
    fn from(value: Rgba<T>) -> Self {
        [value.r, value.g, value.b, value.a]
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Rect<T> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
}

impl<T> Rect<T> {
    pub fn new(x: T, y: T, w: T, h: T) -> Self {
        Self { x, y, w, h }
    }

    pub fn points(&self) -> [Point<T>; 4]
    where
        T: Add<Output = T> + Copy,
    {
        self.into()
    }
}

impl<T> From<&Rect<T>> for [Point<T>; 4]
where
    T: Add<Output = T> + Copy,
{
    fn from(value: &Rect<T>) -> Self {
        [
            Point::new(value.x, value.y),
            Point::new(value.x + value.w, value.y),
            Point::new(value.x + value.w, value.y + value.h),
            Point::new(value.x, value.y + value.h),
        ]
    }
}

impl<T> From<(T, T, T, T)> for Rect<T> {
    fn from((x, y, w, h): (T, T, T, T)) -> Self {
        Self::new(x, y, w, h)
    }
}

impl<T: Default> From<ImageSize<T>> for Rect<T> {
    fn from(value: ImageSize<T>) -> Self {
        Self::new(T::default(), T::default(), value.w, value.h)
    }
}

impl From<&Rect<u32>> for Rect<i32> {
    fn from(value: &Rect<u32>) -> Self {
        Self::new(
            value.x as i32,
            value.y as i32,
            value.w as i32,
            value.h as i32,
        )
    }
}
