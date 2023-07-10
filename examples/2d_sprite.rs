use std::ops::Deref;

use hecs::World;
use nalgebra::Matrix3;
use yapgeir_assets::png::decode_png;
use yapgeir_core::Delta;
use yapgeir_graphics_hal::{
    frame_buffer::FrameBuffer, sampler::Sampler, texture::PixelFormat, Graphics,
};
use yapgeir_graphics_hal_gles2::Gles;
use yapgeir_realm::{Realm, Res, ResMut};
use yapgeir_renderer_2d::{
    quad_index_buffer::QuadIndexBuffer,
    sprite_renderer::{DrawRegion, SpriteRenderer, SpriteUniforms, TextureRegion},
};
use yapgeir_sdl::SdlSettings;
use yapgeir_sdl_graphics::SdlWindowBackend;

const WIDTH: u32 = 600;
const HEIGHT: u32 = 400;

pub type GraphicsAdapter = Gles<SdlWindowBackend>;

fn main() {
    yapgeir_realm::Realm::default()
        // Creates SDL window, initializes input, Delta and Frame.
        .add_plugin(yapgeir_sdl::plugin(SdlSettings {
            screen_size: (WIDTH, HEIGHT),
            ..SdlSettings::default()
        }))
        // Prints FPS stats to stdout
        .add_plugin(yapgeir_core::frame_stats::plugin)
        // Creates graphics context (in this case GLES2)
        .add_plugin(yapgeir_sdl_graphics::plugin::<GraphicsAdapter>)
        // Adds ECS as a resource
        .initialize_resource::<World>()
        // Initializes entities in ECS
        .run_system(|mut world: ResMut<World>| {
            world.spawn((Position { x: 0., y: 0. }, Speed(170.)));
            world.spawn((Position { x: -30., y: -50. }, Speed(100.2)));
            world.spawn((Position { x: 30., y: 50. }, Speed(-120.1)));
        })
        // Game logic system
        .add_system(move_tile)
        // Sets up resources for rendering pipeline, and a system that will do actual rendering
        .add_plugin(initialize_rendering::<GraphicsAdapter>)
        .run();
}

#[derive(Debug)]
struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
struct Speed(f32);

fn move_tile(mut world: ResMut<World>, delta: Res<Delta>) {
    for (_, (tile, speed)) in world.query_mut::<(&mut Position, &mut Speed)>() {
        tile.x = tile.x + speed.0 * delta.0;

        if tile.x > 200. || tile.x < -200. {
            speed.0 *= -1.;
        }
    }
}

fn initialize_rendering<G: Graphics>(realm: &mut Realm) {
    realm
        .add_plugin(yapgeir_renderer_2d::plugin::<G>)
        .initialize_resource_with(|graphics: Res<G>| -> G::Texture {
            let (tile_image, tile_size) = decode_png(include_bytes!("assets/tile.png")).unwrap();
            let texture = graphics.new_texture(PixelFormat::Rgba, tile_size, Some(&tile_image));

            texture
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
    texture: Res<G::Texture>,
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
            Sampler::nearest(&texture),
        );

        for (_, tile) in world.query::<&Position>().iter() {
            batch.draw_sprite(
                DrawRegion::Point((tile.x as f32, tile.y as f32).into()),
                TextureRegion::Full,
                0,
            );
        }
    }

    graphics.swap_buffers();
}
