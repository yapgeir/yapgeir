use std::borrow::Cow;

use derive_more::{Constructor, Deref};
use smart_default::SmartDefault;

#[cfg(feature = "reflection")]
use yapgeir_reflection::bevy_reflect::{self, Reflect};

/// An internal hack used to put reflection under a feature toggle.
/// Use these stubs when you need to provide trait bounds of Reflect trait,
/// but want to make reflection conditional under a feature flag, disabling yapgeir_reflection
/// dependency all together.
pub mod __reflection_stubs;


pub mod frame_stats;

/// A resource that holds time that passed since the previous frame in seconds.
#[derive(Default, Clone, Copy, Deref, Debug, PartialEq)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
pub struct Delta(pub f32);

/// Current frame number. Will be reset to 0 on overflow.
#[derive(Default, Clone, Copy, Deref, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
pub struct Frame(pub u64);

/// Pixels per point returned by the window manager. Can be used for HiDPI scaling of user interfaces.
#[derive(SmartDefault, Clone, Copy, Deref, Debug, PartialEq)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
pub struct ScreenPpt(#[default(1.)] pub f32);

/// Current window size in pixels. Refreshed automatically by the window manager plugin on each frame.
#[derive(Constructor, Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
pub struct WindowSize {
    pub w: u32,
    pub h: u32,
}

/// A component that can be used for debugging your entities. A name is either an owned string,
/// or a reference with a 'static lifetime.
#[derive(Debug, Clone, PartialEq, Eq, Deref)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
#[deref(forward)]
pub struct Named(pub Cow<'static, str>);

impl Named {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self(name.into())
    }
}
