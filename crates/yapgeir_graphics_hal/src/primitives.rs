use bytemuck::Zeroable;
use derive_more::Constructor;

#[derive(Constructor, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageSize<T> {
    pub w: T,
    pub h: T,
}

impl<T> From<(T, T)> for ImageSize<T> {
    fn from((w, h): (T, T)) -> Self {
        Self::new(w, h)
    }
}

#[derive(Clone, Copy, Default, Constructor, Debug, PartialEq, Eq, Zeroable)]
pub struct Rgba<T> {
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T,
}

impl<T: Copy> Rgba<T> {
    pub fn all(value: T) -> Self {
        Self::new(value, value, value, value)
    }
}

impl<T> From<(T, T, T, T)> for Rgba<T> {
    fn from((r, g, b, a): (T, T, T, T)) -> Self {
        Self::new(r, g, b, a)
    }
}

#[derive(Default, Constructor, Clone, Debug, PartialEq, Eq)]
pub struct Rect<T> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
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
