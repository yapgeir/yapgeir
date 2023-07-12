use derive_more::Constructor;
use yapgeir_reflection::bevy_reflect::{self, Reflect};

/// Defines a rectangle by two dots.
/// Since we are not storing position and size here, by switching points
/// we can change rectangles orientation
///  ---     ---
/// | / |   | \ |
///  ---     ---
#[derive(Constructor, Debug, Clone, Default, Reflect)]
pub struct Rect {
    pub a: [f32; 2],
    pub b: [f32; 2],
}

impl Rect {
    pub fn flip_y(self) -> Self {
        Self::new([self.a[0], -self.a[1]], [self.b[0], -self.b[1]])
    }

    #[inline]
    pub fn points(&self) -> [[f32; 2]; 4] {
        [
            [self.a[0], self.a[1]],
            [self.a[0], self.b[1]],
            [self.b[0], self.b[1]],
            [self.b[0], self.a[1]],
        ]
    }
}
