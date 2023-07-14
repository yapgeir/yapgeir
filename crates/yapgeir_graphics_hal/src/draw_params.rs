use smart_default::SmartDefault;

use crate::primitives::{Rect, Rgba};

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendingEquation {
    #[default]
    Add,
    Subtract,
    ReverseSubtract,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendingFactor {
    #[default]
    Zero,
    One,
    SourceColor,
    OneMinusSourceColor,
    DestinationColor,
    OneMinusDestinationColor,
    SourceAlpha,
    OneMinusSourceAlpha,
    DestinationAlpha,
    OneMinusDestinationAlpha,
    ConstantColor,
    OneMinusConstantColor,
    ConstantAlpha,
    OneMinusConstantAlpha,
    SourceAlphaSaturate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlendingFunction {
    pub source: BlendingFactor,
    pub destination: BlendingFactor,
}

impl Default for BlendingFunction {
    fn default() -> Self {
        Self {
            source: BlendingFactor::One,
            destination: BlendingFactor::Zero,
        }
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlendingMethod {
    pub equation: BlendingEquation,
    pub function: BlendingFunction,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeparateBlending<T> {
    pub rgb: T,
    pub alpha: T,
}

impl<T: Copy> SeparateBlending<T> {
    pub const fn all(t: T) -> Self {
        Self { rgb: t, alpha: t }
    }
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Blend {
    pub equation: SeparateBlending<BlendingEquation>,
    pub function: SeparateBlending<BlendingFunction>,
    pub color: Rgba<f32>,
}

impl Blend {
    pub fn alpha() -> Self {
        Self {
            function: SeparateBlending {
                rgb: BlendingFunction {
                    source: BlendingFactor::One,
                    destination: BlendingFactor::OneMinusSourceAlpha,
                },
                alpha: BlendingFunction {
                    source: BlendingFactor::OneMinusDestinationAlpha,
                    destination: BlendingFactor::One,
                },
            },
            ..Default::default()
        }
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CullFaceMode {
    #[default]
    Front,
    Back,
    // Will not draw polygons at all
    All,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct PolygonOffset {
    pub factor: f32,
    pub units: f32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepthStencilTest {
    #[default]
    Always,
    Never,
    Less,
    Equal,
    NotEqual,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}

#[derive(SmartDefault, Clone, Debug, PartialEq)]
pub struct Depth {
    pub test: DepthStencilTest,
    pub write: bool,
    #[default((0., 1.))]
    pub range: (f32, f32),
}

#[derive(SmartDefault, Clone, Debug, PartialEq, Eq)]
pub struct StencilFunction {
    pub test: DepthStencilTest,
    pub reference_value: u8,
    #[default(0xff)]
    pub mask: u8,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum StencilActionMode {
    #[default]
    Keep,
    Zero,
    Replace,
    Increment,
    IncrementWrap,
    Decrement,
    DecrementWrap,
    Invert,
}

#[derive(SmartDefault, Clone, Debug, PartialEq, Eq)]
pub struct StencilAction {
    pub stencil_fail: StencilActionMode,
    pub depth_fail: StencilActionMode,
    pub pass: StencilActionMode,
}

#[derive(SmartDefault, Clone, Debug, PartialEq)]
pub struct StencilCheck {
    pub function: StencilFunction,
    pub action: StencilAction,
    #[default(0xff)]
    pub action_mask: u8,
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Stencil {
    pub front: StencilCheck,
    pub back: StencilCheck,
}

#[derive(SmartDefault, Clone, Debug)]
pub struct DrawParameters {
    pub blend: Option<Blend>,
    #[default(Rgba::all(true))]
    pub color_mask: Rgba<bool>,
    pub cull_face: Option<CullFaceMode>,
    pub depth: Option<Depth>,
    pub stencil: Option<Stencil>,
    pub scissor: Option<Rect<u32>>,
    pub viewport: Option<Rect<u32>>,
    #[default(1.)]
    pub line_width: f32,
    pub polygon_offset: Option<PolygonOffset>,
    pub dithering: bool,
}
