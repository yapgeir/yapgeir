use strum::EnumCount;

use crate::{
    buttons::{u32_blocks, ButtonAction, Buttons, CastToUsize},
    Axial,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, EnumCount)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    X1,
    X2,
}

const BLOCKS: usize = u32_blocks(256);

impl CastToUsize for MouseButton {
    fn as_usize(self) -> usize {
        self as usize
    }
}

#[derive(Default)]
pub struct Mouse {
    /// Button states
    pub buttons: Buttons<BLOCKS, MouseButton>,

    /// X and Y pixel amounts scrolled horizontally and vertically between the frames.
    pub wheel: Axial<i32>,

    /// Current cursor position in pixels relative to window
    pub cursor_position: Axial<i32>,

    /// The coordinate difference between current and previous frame in pixels
    pub motion: Axial<i32>,
}

/// Using just Mouse structure may not be enough, since
/// 1. Between the frames multiple mouse events of the same type may have happened
/// 2. The cursor could have landed far away from the place where the mouse button event happened
///
/// To account for that, input system will also emit MouseButtonEvents, that
/// keep mouse coordinate of the place where the event took place.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct MouseButtonEvent {
    /// The coordinate
    pub coordinate: Axial<i32>,
    pub button: MouseButton,
    pub action: ButtonAction,
}
