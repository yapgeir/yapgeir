use derive_more::{Constructor, From};
use nalgebra::{Isometry2, Matrix3, Vector2};
use yapgeir_reflection::bevy_reflect::Reflect;
use yapgeir_reflection::bevy_reflect::{self};

/// A view+projection matrix passed to a shader
#[derive(Default, Clone, From)]
pub struct WorldCamera {
    /// Transforms a world coordinate into a view coordinate with center at [0; 0] in pixels.
    /// This is intentionally not normalized, because this allows rounding pixels in a shader
    pub view: Matrix3<f32>,
    /// Transforms the view*point into normalized space (from -1 to +1)
    pub screen_space: Vector2<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum Flip {
    X,
    Y,
}

/// Transformation matrix of an entity
#[derive(Default, Debug, Clone, Constructor, Reflect)]
pub struct Transform {
    #[reflect(ignore)]
    pub isometry: Isometry2<f32>,
    pub flip: Option<Flip>,
}

/// Depth used for depth buffer to define render order
#[derive(Debug, Clone, Copy, Reflect)]
pub struct Depth(pub u16);
