use yapgeir_geometry::Box2D;

use crate::{Drawable, Sprite};

pub struct SpriteSheet {
    pub texture_size: [u32; 2],
    pub sprite_size: [u32; 2],

    half_size: [f32; 2],
    texel_size: [f32; 2],
}

impl SpriteSheet {
    pub fn new<TextureSize: Into<[u32; 2]>, SpriteSize: Into<[u32; 2]>>(
        texture_size: TextureSize,
        sprite_size: SpriteSize,
    ) -> Self {
        let texture_size = texture_size.into();
        let sprite_size = sprite_size.into();

        Self {
            texture_size,
            sprite_size,
            half_size: [sprite_size[0] as f32 / 2.0, sprite_size[1] as f32 / 2.0],
            texel_size: [
                sprite_size[0] as f32 / texture_size[0] as f32,
                sprite_size[1] as f32 / texture_size[1] as f32,
            ],
        }
    }

    pub fn drawable(&self, x: u32, y: u32) -> Drawable {
        Drawable {
            size: self.sprite_size,
            sprite: Sprite {
                boundaries: Box2D::new([-self.half_size[0], -self.half_size[1]], self.half_size),
                sub_texture: Box2D::new(
                    [x as f32 * self.texel_size[0], y as f32 * self.texel_size[1]],
                    [
                        (x + 1) as f32 * self.texel_size[0],
                        (y + 1) as f32 * self.texel_size[1],
                    ],
                ),
            },
        }
    }
}
