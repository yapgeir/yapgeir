use derive_more::Constructor;
pub use yapgeir_graphics_hal_macro::Vertex;

pub trait Vertex {
    // Format is used to attach attributes during a draw call
    // or to a VAO. VAO is always a sized thing,
    // so there is no need for visitors, it can directly be converted
    // to a byte slice with bytemuck, and sent to a GPU.
    const FORMAT: &'static [VertexAttribute];
}

#[derive(Constructor, Clone, PartialEq, Eq)]
pub struct VertexAttribute {
    pub name: &'static str,
    pub offset: usize,
    pub kind: AttributeKind,
    pub size: VectorSize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AttributeKind {
    I8,
    U8,
    I16,
    U16,
    F32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VectorSize {
    N1,
    N2,
    N3,
    N4,
}

impl VectorSize {
    pub fn size(self) -> usize {
        match self {
            VectorSize::N1 => 1,
            VectorSize::N2 => 2,
            VectorSize::N3 => 3,
            VectorSize::N4 => 4,
        }
    }
}

pub trait AsAttributeKind {
    const KIND: AttributeKind;
    const SIZE: VectorSize;
}

macro_rules! impl_as_attribute_kind_single {
    ($(($t:ty, $cons:ident)),*) => {
        $(impl AsAttributeKind for $t {
            const KIND: AttributeKind = AttributeKind::$cons;
            const SIZE: VectorSize = VectorSize::N1;
        })*
    };
}

macro_rules! impl_as_attribute_kind_array {
    ($sizes:tt, $(($t:ty, $cons:ident)),*) => {
        $(impl_as_attribute_kind_array_inner!($sizes, $t, $cons);)*
    };
}

macro_rules! impl_as_attribute_kind_array_inner {
    ([$(($n:literal, $nn:ident)),*], $t:ty, $cons:ident) => {
        $(impl AsAttributeKind for [$t; $n] {
            const KIND: AttributeKind = AttributeKind::$cons;
            const SIZE: VectorSize = VectorSize::$nn;
        })*
    };
}

impl_as_attribute_kind_single!((i8, I8), (u8, U8), (i16, I16), (u16, U16), (f32, F32));

impl_as_attribute_kind_array!(
    [(1, N1), (2, N2), (3, N3), (4, N4)],
    (i8, I8),
    (u8, U8),
    (i16, I16),
    (u16, U16),
    (f32, F32)
);
