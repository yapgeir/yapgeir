use quad_index_buffer::QuadIndexBuffer;
use yapgeir_graphics_hal::Graphics;
use yapgeir_realm::{Realm, Res};

pub mod batch_renderer;
pub mod primitive_renderer;
pub mod quad_index_buffer;
pub mod sprite_renderer;
pub mod texture_renderer;

pub fn plugin<G: Graphics>(realm: &mut Realm) {
    realm.initialize_resource_with(|ctx: Res<G>| QuadIndexBuffer::<G>::new(&ctx, 65532u16));
}
