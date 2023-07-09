use derive_more::Constructor;
use nalgebra::Vector2;
use strum::EnumCount;

use crate::buttons::{u32_blocks, Buttons, CastToUsize};

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
    pub buttons: Buttons<BLOCKS, GamepadButton>,
    pub left_stick: Vector2<i16>,
    pub right_stick: Vector2<i16>,
    pub left_trigger: i16,
    pub right_trigger: i16,
}
