use std::ops::Deref;

use hecs::World;
use nalgebra::{Matrix3, Vector2};
use yapgeir_assets::png::decode_png;
use yapgeir_core::{Delta, WindowSize};
use yapgeir_events::Events;
use yapgeir_graphics_hal::{
    frame_buffer::FrameBuffer, sampler::Sampler, texture::PixelFormat, Graphics,
};
use yapgeir_graphics_hal_gles2::Gles;
use yapgeir_input::{
    buttons::ButtonAction,
    mouse::{MouseButton, MouseButtonEvent},
    Axial,
};
use yapgeir_realm::{Realm, Res, ResMut};
use yapgeir_renderer_2d::{
    quad_index_buffer::QuadIndexBuffer,
    sprite_renderer::{DrawRegion, SpriteRenderer, TextureRegion},
};
use yapgeir_sdl::SdlSettings;
use yapgeir_sdl_graphics::SdlWindowBackend;

pub type GraphicsAdapter = Gles<SdlWindowBackend>;

fn main() {
    yapgeir_realm::Realm::default()
        // Creates SDL window, initializes input, Delta and Frame.
        .add_plugin(yapgeir_sdl::plugin(SdlSettings {
            window_size: WindowSize::new(600, 400),
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
            for _ in 0..1000 {
                world.spawn((
                    Position(Vector2::new(
                        rand::random::<f32>() * 200. - 100.,
                        rand::random::<f32>() * 200. - 100.,
                    )),
                    Velocity(Vector2::new(
                        rand::random::<f32>() * 600. - 300.,
                        rand::random::<f32>() * 600. - 300.,
                    )),
                ));
            }
        })
        // Game logic system
        .add_system(move_tile)
        .add_system(spawn_tile_on_left_click)
        .add_system(despawn_tile_on_right_click)
        // Sets up resources for rendering pipeline, and a system that will do actual rendering
        .add_plugin(initialize_rendering::<GraphicsAdapter>)
        .run();
}

#[derive(Debug, Clone, Copy)]
struct Position(Vector2<f32>);

#[derive(Debug)]
struct Velocity(Vector2<f32>);

fn move_tile(mut world: ResMut<World>, delta: Res<Delta>, window_size: Res<WindowSize>) {
    for (_, (position, velocity)) in world.query_mut::<(&mut Position, &mut Velocity)>() {
        position.0 += velocity.0 * delta.0;

        if position.0.x > window_size.w as f32 / 2. {
            velocity.0.x = -1. * velocity.0.x.abs();
        } else if position.0.x < -(window_size.w as f32 / 2.) {
            velocity.0.x = velocity.0.x.abs();
        }

        if position.0.y > window_size.h as f32 / 2. {
            velocity.0 = -1. * velocity.0.abs();
        } else if position.0.y < -(window_size.h as f32 / 2.) {
            velocity.0 = velocity.0.abs();
        }
    }
}

fn window_to_world(position: Axial<i32>, window_size: WindowSize) -> Position {
    let x = position.x as f32 - (window_size.w as f32 / 2.);
    let y = -(position.y as f32 - (window_size.h as f32 / 2.));

    Position(Vector2::new(x, y))
}

fn spawn_tile_on_left_click(
    mut world: ResMut<World>,
    mouse_button_events: Res<Events<MouseButtonEvent>>,
    window_size: Res<WindowSize>,
) {
    let left_button_mouse_down_events = mouse_button_events
        .iter()
        .filter(|e| e.action == ButtonAction::Down && e.button == MouseButton::Left);

    for e in left_button_mouse_down_events {
        let position = window_to_world(e.coordinate, *window_size);
        world.spawn((
            position,
            Velocity(Vector2::new(
                rand::random::<f32>() * 200. - 100.,
                rand::random::<f32>() * 600. - 300.,
            )),
        ));
    }
}

fn is_in_rectangle(center: Position, point: Position, side_length: f32) -> bool {
    let half_side = side_length / 2.0;
    if point.0.x < center.0.x - half_side || point.0.x > center.0.x + half_side {
        return false;
    }
    if point.0.y < center.0.y - half_side || point.0.y > center.0.y + half_side {
        return false;
    }
    true
}

fn despawn_tile_on_right_click(
    mut world: ResMut<World>,
    mouse_button_events: Res<Events<MouseButtonEvent>>,
    window_size: Res<WindowSize>,
) {
    let right_button_mouse_down_events = mouse_button_events
        .iter()
        .filter(|e| e.action == ButtonAction::Down && e.button == MouseButton::Right);

    for e in right_button_mouse_down_events {
        let position = window_to_world(e.coordinate, *window_size);
        let entity = world
            .query::<&Position>()
            .iter()
            .find(|(_, center)| is_in_rectangle(**center, position, 50.))
            .map(|(e, _)| e);

        if let Some(entity) = entity {
            world.despawn(entity).expect("Unable to despawn entity");
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
    let fb = graphics.default_frame_buffer();
    fb.clear(
        None,
        Some(yapgeir_graphics_hal::Rgba::new(0., 0., 0., 1.)),
        Some(0.),
        None,
    );

    {
        let mut batch = sprite_renderer.start_batch(
            &fb,
            Matrix3::identity().into(),
            Sampler::nearest(&texture),
        );

        for (_, tile) in world.query::<&Position>().iter() {
            batch.draw_sprite(
                DrawRegion::Point(tile.0.cast().into()),
                TextureRegion::Full,
                0,
            );
        }
    }

    graphics.swap_buffers();
}
