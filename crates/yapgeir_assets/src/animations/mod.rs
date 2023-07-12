use derive_more::Deref;
use serde::Deserialize;
use yapgeir_world_2d::Drawable;

pub mod file;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum AnimationKind {
    Loop,
    PingPong,
    Single,
}

/// This defines a single animation as a relationship between
/// sprites in the atlas.
#[derive(Debug, Clone)]
pub struct Animation {
    pub frames: Vec<Drawable>,
    pub kind: AnimationKind,
    pub frame_time: f32,
}

#[derive(Debug, Clone, Deref)]
pub struct AnimationSequence(pub Vec<Animation>);

impl Animation {
    #[inline]
    pub fn is_last_frame(&self, frame: u8) -> bool {
        self.frames.len() - 1 <= frame as usize
    }

    #[inline]
    pub fn is_end(&self, frame: u8) -> bool {
        self.is_last_frame(frame) && self.kind == AnimationKind::Single
    }
}
