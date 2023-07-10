use derive_more::Constructor;
use strum::EnumCount;

use crate::{
    buttons::{u32_blocks, Buttons, CastToUsize},
    Axial,
};

#[derive(Constructor, PartialEq, Eq, Hash, Clone, Copy)]
pub struct GamepadId(pub u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, EnumCount)]
pub enum GamepadButton {
    A,
    B,
    X,
    Y,
    Back,
    Guide,
    Start,
    LeftStick,
    RightStick,
    LeftShoulder,
    RightShoulder,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    Misc1,
    Paddle1,
    Paddle2,
    Paddle3,
    Paddle4,
    TouchPad,
    Max,
}

const BLOCKS: usize = u32_blocks(GamepadButton::COUNT);

impl CastToUsize for GamepadButton {
    fn as_usize(self) -> usize {
        self as usize
    }
}

#[derive(Default)]
pub struct Gamepad {
    //// Current button states.
    pub buttons: Buttons<BLOCKS, GamepadButton>,

    /// Left stick coordinates. Each axis is normalized to [-1, 1]. Center is [0, 0].
    pub left_stick: Axial<f32>,

    /// Right stick coordinates. Each axis is normalized to [-1, 1]. Center is [0, 0].
    pub right_stick: Axial<f32>,

    /// Left trigger state. Normalized to [0, 1]. Depressed is 0.
    pub left_trigger: f32,

    /// Right trigger state. Normalized to [0, 1]. Depressed is 0.
    pub right_trigger: f32,
}
