use derive_more::{Constructor, Deref};
use smart_default::SmartDefault;

#[cfg(feature = "reflection")]
use yapgeir_reflection::bevy_reflect::{self, Reflect};

pub mod frame_stats;

/// A resource that holds time that passed since the previous frame in seconds.
#[derive(Default, Clone, Copy, Deref, Debug, PartialEq)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
pub struct Delta(pub f32);

/// Current frame number.
#[derive(Default, Clone, Copy, Deref, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
pub struct Frame(pub u64);

/// Pixels per point returned by the window manager.
/// Can be used for HiDPI scaling of user interfaces.
#[derive(SmartDefault, Clone, Copy, Deref, Debug, PartialEq)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
pub struct ScreenPpt(#[default(1.)] pub f32);

/// Current window size in pixels. Refreshed automatically by the window manager on each frame.
#[derive(Constructor, Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
pub struct WindowSize {
    pub w: u32,
    pub h: u32,
}
