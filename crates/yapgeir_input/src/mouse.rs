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

    /// X and Y amounts scrolled horizontally and vertically between the frames
    pub wheel: Axial<i32>,

    /// Current cursor position in pixels relative to window
    pub cursor_position: Axial<i32>,

    /// The coordinate difference between current and previous frame
    pub motion: Axial<i32>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct MouseButtonEvent {
    pub coordinate: Axial<i32>,
    pub button: MouseButton,
    pub action: ButtonAction,
}
