use std::{ffi::c_void, ops::Deref, rc::Rc};

use hecs::World;
use nalgebra::{vector, Matrix3};
use yapgeir_assets::png::decode_png;
use yapgeir_graphics_hal::{
    frame_buffer::FrameBuffer, sampler::Sampler, texture::PixelFormat, Backend, Graphics, ImageSize,
};
use yapgeir_graphics_hal_gles2::Gles;
use yapgeir_realm::{Realm, Res, ResMut};
use yapgeir_renderer_2d::{
    quad_index_buffer::QuadIndexBuffer,
    sprite_renderer::{DrawRegion, SpriteRenderer, SpriteUniforms, TextureRegion},
};
use yapgeir_sdl::{
    sdl2::{self, video::SwapInterval},
    SdlSettings,
};

const WIDTH: u32 = 600;
const HEIGHT: u32 = 400;

fn main() {
    yapgeir_realm::Realm::default()
        .add_plugin(yapgeir_sdl::plugin(SdlSettings {
            screen_size: vector![WIDTH, HEIGHT],
            ..SdlSettings::default()
        }))
        .add_plugin(yapgeir_core::frame_stats::plugin)
        .add_plugin(graphics_adapter_plugin)
        .add_plugin(rendering_plugin::<GraphicsAdapter>)
        .add_plugin(setup_plugin)
        .add_system(move_tile)
        .run();
}

#[derive(Debug)]
struct Tile {
    pub x: u32,
    pub y: u32,
}

fn setup_plugin(realm: &mut Realm) {
    realm
        .initialize_resource::<World>()
        .run_system(|mut world: ResMut<World>| {
            world.spawn((Tile { x: 0, y: 0 },));
        });
}

fn move_tile(mut world: ResMut<World>) {
    for (_, tile) in world.query_mut::<&mut Tile>() {
        tile.x += 1;
    }
}

pub struct SdlBackend(Rc<sdl2::video::Window>);

impl Backend for SdlBackend {
    fn swap_buffers(&self) {
        self.0.gl_swap_window();
    }

    fn get_proc_address(&self, symbol: &str) -> *const c_void {
        self.0.subsystem().gl_get_proc_address(symbol) as *const c_void
    }

    fn default_framebuffer_size(&self) -> ImageSize<u32> {
        self.0.drawable_size().into()
    }
}

pub type GraphicsAdapter = Gles<SdlBackend>;

pub fn graphics_adapter_plugin(realm: &mut Realm) {
    realm.initialize_resource_with(|window: Res<Rc<sdl2::video::Window>>| {
        let graphics_adapter = GraphicsAdapter::new(SdlBackend(window.clone()));

        window
            .gl_set_context_to_current()
            .expect("unable to set current gl context");

        window
            .subsystem()
            .gl_set_swap_interval(SwapInterval::VSync)
            .expect("Unable to set swap interval");

        graphics_adapter
    });
}

struct Textures<G: Graphics> {
    pub tile: G::Texture,
}

fn rendering_plugin<G: Graphics>(realm: &mut Realm) {
    realm
        .add_plugin(yapgeir_renderer_2d::plugin::<G>)
        .initialize_resource_with(|graphics: Res<G>| -> Textures<G> {
            let (tile_image, tile_size) = decode_png(include_bytes!("assets/tile.png")).unwrap();
            let tile_texture =
                graphics.new_texture(PixelFormat::Rgba, tile_size, Some(&tile_image));

            Textures { tile: tile_texture }
        })
        .initialize_resource_with(
            |graphics: Res<G>, quad_index_buffer: Res<QuadIndexBuffer<G>>| -> SpriteRenderer<G> {
                SpriteRenderer::new(graphics.deref(), quad_index_buffer.clone())
            },
        )
        .add_system(render::<G>);
}

fn render<G: Graphics>(
    mut sprite_renderer: ResMut<SpriteRenderer<G>>,
    graphics: Res<G>,
    textures: Res<Textures<G>>,
    world: Res<World>,
) {
    let fb = graphics.default_framebuffer();
    fb.clear(
        None,
        Some(yapgeir_graphics_hal::Rgba::new(0., 0., 0., 1.)),
        Some(0.),
        None,
    );

    {
        let mut batch = sprite_renderer.start_batch(
            &fb,
            &SpriteUniforms {
                view: Matrix3::identity().into(),
                scale: [1. / WIDTH as f32, 1. / HEIGHT as f32],
            },
            Sampler::nearest(&textures.tile),
        );

        for (_, tile) in world.query::<&Tile>().iter() {
            batch.draw_sprite(DrawRegion::Point((tile.x as f32, tile.y as f32).into()), TextureRegion::Full, 0);
        }


    }

    graphics.swap_buffers();
}
