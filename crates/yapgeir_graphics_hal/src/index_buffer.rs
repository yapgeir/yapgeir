use bytemuck::Pod;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexKind {
    U8,
    U16,
    U32,
}

impl IndexKind {
    pub fn size(self) -> usize {
        match self {
            IndexKind::U8 => 1,
            IndexKind::U16 => 2,
            IndexKind::U32 => 4,
        }
    }
}

pub trait Index: Pod {
    const KIND: IndexKind;
}

impl Index for u8 {
    const KIND: IndexKind = IndexKind::U8;
}

impl Index for u16 {
    const KIND: IndexKind = IndexKind::U16;
}

impl Index for u32 {
    const KIND: IndexKind = IndexKind::U32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveMode {
    Points,
    Lines,
    LineStrip,
    LineLoop,
    Triangles,
    TriangleStrip,
    TriangleFan,
}
