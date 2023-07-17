use quad_index_buffer::QuadIndexBuffer;
use yapgeir_geometry::Size;
use yapgeir_graphics_hal::Graphics;
use yapgeir_realm::{Realm, Res};

pub mod batch_renderer;
pub mod primitive_renderer;
pub mod quad_index_buffer;
pub mod sprite_renderer;

pub enum NdcProjection {
    Center,
    TopLeft,
    Custom { offset: [f32; 2], scale: [f32; 2] },
}

impl NdcProjection {
    pub fn offset_and_scale(self, size: Size<u32>) -> ([f32; 2], [f32; 2]) {
        match self {
            Self::Center => (
                [0., 0.],
                [1. / (size.w / 2) as f32, 1. / (size.h / 2) as f32],
            ),
            Self::TopLeft => {
                let offset = [-((size.w / 2) as f32), ((size.h / 2) as f32)];
                let scale = [-1. / offset[0], 1. / offset[1]];
                (offset, scale)
            }
            Self::Custom { offset, scale } => (offset, scale),
        }
    }
}

pub fn plugin<G: Graphics>(realm: &mut Realm) {
    realm.initialize_resource_with(|ctx: Res<G>| QuadIndexBuffer::<G>::new(&ctx, 65532u16));
}
