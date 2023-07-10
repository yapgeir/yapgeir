use std::collections::HashMap;

use anyhow::Result;
use nalgebra::{point, vector, Point2, Scale2, Vector2};
use serde::Deserialize;
use yapgeir_geometry::Rect as GRect;

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
    fn to_sprite(&self, texture_scale: Scale2<f32>, pixels_per_meter: f32) -> super::Sprite {
        let half_size = self.source_size.vector() * 0.5;

        let sub_texture = super::SubTexture {
            clip: GRect::new(
                (&self.sprite_source_size.a() - half_size) / pixels_per_meter,
                (&self.sprite_source_size.b() - half_size) / pixels_per_meter,
            )
            .flip_y(),
            sprite: GRect::new(
                texture_scale.transform_point(&self.frame.b()),
                texture_scale.transform_point(&self.frame.a()),
            ),
        };

        super::Sprite {
            size: self.source_size.vector() / pixels_per_meter,
            sub_texture,
        }
    }
}

impl AsepriteAtlas {
    pub fn decode(json: &str) -> Result<AsepriteAtlas> {
        Ok(serde_json::from_str(json)?)
    }

    pub fn to_atlas(&self, pixels_per_meter: f32) -> Atlas {
        let size = vector![self.meta.size.w, self.meta.size.h];
        let texture_scale = Scale2::from(size).cast::<f32>().pseudo_inverse();

        Atlas {
            sprites: self
                .frames
                .iter()
                .map(|(k, v)| (k.clone(), v.to_sprite(texture_scale, pixels_per_meter)))
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
