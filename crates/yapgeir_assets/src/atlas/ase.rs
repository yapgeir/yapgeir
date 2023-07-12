use std::collections::HashMap;

use anyhow::Result;
use nalgebra::{point, vector, Point2, Scale2, Vector2};
use serde::Deserialize;
use yapgeir_geometry::Box2D as GRect;

use super::Atlas;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AsepriteAtlas {
    pub frames: HashMap<String, Sprite>,
    pub meta: Meta,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Sprite {
    pub frame: Rect,
    pub rotated: bool,
    pub trimmed: bool,
    pub sprite_source_size: Rect,
    pub source_size: Size,
    pub duration: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FrameTag {
    pub name: String,
    pub from: usize,
    pub to: usize,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub size: Size,
    pub frame_tags: Vec<FrameTag>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct Size {
    pub w: u32,
    pub h: u32,
}

#[derive(Default, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Size {
    fn vector(self) -> Vector2<f32> {
        vector![self.w, self.h].cast()
    }
}

impl Rect {
    fn a(&self) -> Point2<f32> {
        point![self.x, self.y].cast()
    }

    fn b(&self) -> Point2<f32> {
        self.a() + self.size()
    }

    fn size(&self) -> Vector2<f32> {
        vector![self.w, self.h].cast()
    }
}

impl Sprite {
    fn to_sprite(&self, texel_scale: Scale2<f32>) -> super::Sprite {
        let half_size = self.source_size.vector() * 0.5;

        // Clip is logically inverted, because it was calculated based on Y down,
        // but represents Y-up coordinates.
        // TODO: make this code comprehensible
        let mut a = self.sprite_source_size.a();
        let mut b = self.sprite_source_size.b();
        let ay = a.y;
        let by = b.y;
        a.y = self.source_size.h as f32 - by;
        b.y = self.source_size.h as f32 - ay;

        let sub_texture = super::SubTexture {
            boundaries: GRect::new((&a - half_size).into(), (&b - half_size).into()),
            sprite: GRect::new(
                texel_scale.transform_point(&self.frame.a()).into(),
                texel_scale.transform_point(&self.frame.b()).into(),
            ),
        };

        super::Sprite {
            size: (self.source_size.w, self.source_size.h),
            sub_texture,
        }
    }
}

impl AsepriteAtlas {
    pub fn decode(json: &str) -> Result<AsepriteAtlas> {
        Ok(serde_json::from_str(json)?)
    }

    pub fn to_atlas(&self) -> Atlas {
        let texel_space = Scale2::new(self.meta.size.w, self.meta.size.h)
            .cast::<f32>()
            .pseudo_inverse();

        Atlas {
            sprites: self
                .frames
                .iter()
                .map(|(k, v)| (k.clone(), v.to_sprite(texel_space)))
                .collect(),
            frame_tags: self
                .meta
                .frame_tags
                .iter()
                .map(|f| (f.name.clone(), f.from..=f.to))
                .collect(),
        }
    }
}
