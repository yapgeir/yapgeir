use derive_more::Constructor;
use nalgebra::{point, Point2};

/// Defines a rectangle by two dots.
/// Since we are not storing position and size here, by switching points
/// we can change rectangles orientation
///  ---     ---
/// | / |   | \ |
///  ---     ---
#[derive(Constructor, Debug, Clone, Default)]
pub struct Rect {
    a: Point2<f32>,
    b: Point2<f32>,
}

impl Rect {
    pub fn flip_y(self) -> Self {
        Self::new(point![self.a.x, -self.a.y], point![self.b.x, -self.b.y])
    }

    #[inline]
    pub fn points(&self) -> [Point2<f32>; 4] {
        [
            point![self.a.x, self.b.y],
            self.b,
            point![self.b.x, self.a.y],
            self.a,
        ]
    }

    #[inline]
    pub fn raw_points(&self) -> [[f32; 2]; 4] {
        [
            [self.a.x, self.a.y],
            [self.a.x, self.b.y],
            [self.b.x, self.b.y],
            [self.b.x, self.a.y],
        ]
    }
}
