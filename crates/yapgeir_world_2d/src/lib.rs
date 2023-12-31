use derive_more::{Constructor, Deref, DerefMut, From};
use nalgebra::{Isometry2, Matrix3};
use smart_default::SmartDefault;
use yapgeir_geometry::Box2D;

#[cfg(feature = "reflection")]
use yapgeir_reflection::bevy_reflect::{self, Reflect};

pub use sprite_sheet::*;

mod sprite_sheet;

/// A Drawable component represents a sprite.
///
/// Defines the logical size of the sprite,
/// the corresponding rectangle on a texture and a crop rectangle
/// for sprites that had transparent parts removed from the atlas.
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
pub struct Drawable {
    /// The logical size of the sprite.
    /// If you have drawn a 32x32 sprite, and an atlas has a cropped version of it,
    /// the size is [32, 32].
    ///
    /// This property is useful for debug drawing, otherwise it is unused.
    pub size: [u32; 2],

    /// An actual sprite used for rendering.
    pub sprite: Sprite,
}

/// The actual
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
pub struct Sprite {
    /// This is the clip rectangle in centered pixel coordinates.
    /// A rectangle from [-5, -5] to [5, 5] would map the sub-texture to a
    /// 10x10 pixel square with a center in your transformation translation.
    ///
    /// Why is this necessary at all? Imagine you have a sprite animation of a character
    /// where each frame is 32x32 px. Naturally some of the pixels will be transparent.
    /// These pixels can then be cropped out from an atlas. But this needs to be accounted
    /// for during rendering, so that you can draw the sub-texture, as if it's a 32x32 px
    /// with transparent parts.
    pub boundaries: Box2D<f32>,

    /// This is the location of a sprite on an atlas in texture space.
    /// A rectangle from [0, 0] to [1, 1] is a full texture.
    pub sub_texture: Box2D<f32>,
}

/// Sprite's quad in world space.
/// Defines an area where the sprite should be drawn.
/// `yapgeir_world_2d_sprites` plugin will automatically add this component
/// to all entities with `Drawable` and `Transform` components,
/// and will update it's value on every frame.
///
/// You don't need to add or edit this component to your entities,
/// it will be managed automatically.
#[derive(Debug, Clone, Deref, From, Default)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
pub struct DrawQuad([[f32; 2]; 4]);

/// This component denotes that the entity is static,
/// and it's `Drawable` should not be recalculated on every frame.
///
/// If you change the `Transform` component of an entity, it's `Drawable` position will not be changed.
/// To force update of the `Drawable` component, add `Dirty` marker to your `Static` entity.
#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
pub struct Static;

/// Marks a `Static` entity as `Dirty`, forcing it's `Drawable` component to be recalculated.
/// After recalculation `Dirty` component will be removed automatically.
///
/// Adding `Dirty` to a non-static entity will have no effect.
#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
pub struct Dirty;

/// A view+projection matrix passed to a shader.
/// A camera defines how world space is transformed into screen space.
#[derive(Default, Clone, From, Deref, DerefMut)]
pub struct WorldCamera(pub Matrix3<f32>);

/// Defines if a mesh should be flipped alongside it's X or Y axis.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
pub enum Flip {
    #[default]
    X,
    Y,
}

/// Transformation matrix of an entity. The unit of this matrix
/// is an abstract point. Unless `TransformPpt` resource
/// is registered, it is assumed that one point translates to one pixel.
#[derive(Constructor, Default, Debug, Clone)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
pub struct Transform {
    #[cfg_attr(feature = "reflection", reflect(ignore))]
    pub isometry: Isometry2<f32>,
    pub flip: Option<Flip>,
}

/// Depth used for depth buffer to define render order.
/// 0 is a near plane (foreground), and u16::MAX is a far plane (background).
#[derive(Default, Debug, Clone, Copy, Deref, DerefMut)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
pub struct Depth(pub u16);

/// Defines the conversion rate between transformation (and physics) units and pixels.
/// For a metric system this is pixels per meter.
#[derive(SmartDefault, Debug, Clone, Copy, Deref, DerefMut)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
pub struct TransformPpt(#[default(1.)] pub f32);
