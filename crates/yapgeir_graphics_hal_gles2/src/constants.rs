use yapgeir_graphics_hal::{
    buffer::{BufferKind, BufferUsage},
    draw_params::{
        BlendingEquation, BlendingFactor, CullFaceMode, DepthStencilTest, StencilActionMode,
    },
    frame_buffer::RenderBufferFormat,
    index_buffer::{IndexKind, PrimitiveMode},
    sampler::{Filter, MinFilter, WrapFunction},
    vertex_buffer::AttributeKind,
};

use crate::texture::{RgbLayout, RgbaLayout};

pub trait GlConstant {
    fn gl_const(self) -> u32;
}

impl GlConstant for AttributeKind {
    fn gl_const(self) -> u32 {
        match self {
            AttributeKind::I8 => glow::BYTE,
            AttributeKind::U8 => glow::UNSIGNED_BYTE,
            AttributeKind::I16 => glow::SHORT,
            AttributeKind::U16 => glow::UNSIGNED_SHORT,
            AttributeKind::F32 => glow::FLOAT,
        }
    }
}

impl GlConstant for BufferKind {
    fn gl_const(self) -> u32 {
        match self {
            BufferKind::Index => glow::ELEMENT_ARRAY_BUFFER,
            BufferKind::Vertex => glow::ARRAY_BUFFER,
        }
    }
}

impl GlConstant for BufferUsage {
    fn gl_const(self) -> u32 {
        match self {
            BufferUsage::Static => glow::STATIC_DRAW,
            BufferUsage::Dynamic => glow::DYNAMIC_DRAW,
            BufferUsage::Stream => glow::STREAM_DRAW,
        }
    }
}

impl GlConstant for BlendingFactor {
    fn gl_const(self) -> u32 {
        match self {
            BlendingFactor::Zero => glow::ZERO,
            BlendingFactor::One => glow::ONE,
            BlendingFactor::SourceColor => glow::SRC_COLOR,
            BlendingFactor::OneMinusSourceColor => glow::ONE_MINUS_SRC_COLOR,
            BlendingFactor::DestinationColor => glow::DST_COLOR,
            BlendingFactor::OneMinusDestinationColor => glow::ONE_MINUS_DST_COLOR,
            BlendingFactor::SourceAlpha => glow::SRC_ALPHA,
            BlendingFactor::OneMinusSourceAlpha => glow::ONE_MINUS_SRC_ALPHA,
            BlendingFactor::DestinationAlpha => glow::DST_ALPHA,
            BlendingFactor::OneMinusDestinationAlpha => glow::ONE_MINUS_DST_ALPHA,
            BlendingFactor::ConstantColor => glow::CONSTANT_COLOR,
            BlendingFactor::OneMinusConstantColor => glow::ONE_MINUS_CONSTANT_COLOR,
            BlendingFactor::ConstantAlpha => glow::CONSTANT_ALPHA,
            BlendingFactor::OneMinusConstantAlpha => glow::ONE_MINUS_CONSTANT_ALPHA,
            BlendingFactor::SourceAlphaSaturate => glow::SRC_ALPHA_SATURATE,
        }
    }
}

impl GlConstant for DepthStencilTest {
    fn gl_const(self) -> u32 {
        match self {
            DepthStencilTest::Always => glow::ALWAYS,
            DepthStencilTest::Never => glow::NEVER,
            DepthStencilTest::Less => glow::LESS,
            DepthStencilTest::Equal => glow::EQUAL,
            DepthStencilTest::NotEqual => glow::NOTEQUAL,
            DepthStencilTest::LessOrEqual => glow::LEQUAL,
            DepthStencilTest::Greater => glow::GREATER,
            DepthStencilTest::GreaterOrEqual => glow::GEQUAL,
        }
    }
}

impl GlConstant for BlendingEquation {
    fn gl_const(self) -> u32 {
        match self {
            BlendingEquation::Add => glow::FUNC_ADD,
            BlendingEquation::Subtract => glow::FUNC_SUBTRACT,
            BlendingEquation::ReverseSubtract => glow::FUNC_REVERSE_SUBTRACT,
        }
    }
}

impl GlConstant for StencilActionMode {
    fn gl_const(self) -> u32 {
        match self {
            StencilActionMode::Keep => glow::KEEP,
            StencilActionMode::Zero => glow::ZERO,
            StencilActionMode::Replace => glow::REPLACE,
            StencilActionMode::Increment => glow::INCR,
            StencilActionMode::IncrementWrap => glow::INCR_WRAP,
            StencilActionMode::Decrement => glow::DECR,
            StencilActionMode::DecrementWrap => glow::DECR_WRAP,
            StencilActionMode::Invert => glow::INVERT,
        }
    }
}

impl GlConstant for CullFaceMode {
    fn gl_const(self) -> u32 {
        match self {
            CullFaceMode::Front => glow::FRONT,
            CullFaceMode::Back => glow::BACK,
            CullFaceMode::All => glow::FRONT_AND_BACK,
        }
    }
}

impl GlConstant for RenderBufferFormat {
    fn gl_const(self) -> u32 {
        match self {
            RenderBufferFormat::Depth => glow::DEPTH_COMPONENT24,
            RenderBufferFormat::Stencil => glow::STENCIL_INDEX8,
            RenderBufferFormat::DepthStencil => glow::DEPTH24_STENCIL8,
        }
    }
}

impl GlConstant for PrimitiveMode {
    fn gl_const(self) -> u32 {
        match self {
            PrimitiveMode::Points => glow::POINTS,
            PrimitiveMode::Lines => glow::LINES,
            PrimitiveMode::LineStrip => glow::LINE_STRIP,
            PrimitiveMode::LineLoop => glow::LINE_LOOP,
            PrimitiveMode::Triangles => glow::TRIANGLES,
            PrimitiveMode::TriangleStrip => glow::TRIANGLE_STRIP,
            PrimitiveMode::TriangleFan => glow::TRIANGLE_FAN,
        }
    }
}

impl GlConstant for IndexKind {
    fn gl_const(self) -> u32 {
        match self {
            IndexKind::U8 => glow::UNSIGNED_BYTE,
            IndexKind::U16 => glow::UNSIGNED_SHORT,
            IndexKind::U32 => glow::UNSIGNED_INT,
        }
    }
}

impl GlConstant for WrapFunction {
    fn gl_const(self) -> u32 {
        match self {
            WrapFunction::Clamp => glow::CLAMP_TO_EDGE,
            WrapFunction::Repeat => glow::REPEAT,
            WrapFunction::MirrorClamp => glow::MIRROR_CLAMP_TO_EDGE,
            WrapFunction::MirrorRepeat => glow::MIRRORED_REPEAT,
        }
    }
}

impl GlConstant for Filter {
    fn gl_const(self) -> u32 {
        match self {
            Filter::Linear => glow::LINEAR,
            Filter::Nearest => glow::NEAREST,
        }
    }
}

impl GlConstant for MinFilter {
    fn gl_const(self) -> u32 {
        match self {
            MinFilter::Origin(filter) => filter.gl_const(),
            MinFilter::Mipmap {
                mipmap: Filter::Linear,
                texel: Filter::Nearest,
            } => glow::LINEAR_MIPMAP_NEAREST,
            MinFilter::Mipmap {
                mipmap: Filter::Nearest,
                texel: Filter::Linear,
            } => glow::NEAREST_MIPMAP_LINEAR,
            MinFilter::Mipmap {
                mipmap: Filter::Linear,
                texel: Filter::Linear,
            } => glow::LINEAR_MIPMAP_LINEAR,
            MinFilter::Mipmap {
                mipmap: Filter::Nearest,
                texel: Filter::Nearest,
            } => glow::NEAREST_MIPMAP_NEAREST,
        }
    }
}

impl GlConstant for RgbLayout {
    fn gl_const(self) -> u32 {
        match self {
            RgbLayout::U8 => glow::UNSIGNED_BYTE,
            RgbLayout::U16_5_6_5 => glow::UNSIGNED_SHORT_5_6_5,
        }
    }
}

impl GlConstant for RgbaLayout {
    fn gl_const(self) -> u32 {
        match self {
            RgbaLayout::U8 => glow::UNSIGNED_BYTE,
            RgbaLayout::U16_4_4_4_4 => glow::UNSIGNED_SHORT_4_4_4_4,
            RgbaLayout::U16_5_5_5_1 => glow::UNSIGNED_SHORT_5_5_5_1,
        }
    }
}
