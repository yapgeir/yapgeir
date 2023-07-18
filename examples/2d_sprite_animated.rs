use std::ops::Deref;

use hecs::World;
use nalgebra::{Isometry2, Matrix3, Vector2};
use yapgeir_assets::{
    animations::{Animation, AnimationKind, AnimationSequence},
    png::decode_png,
};
use yapgeir_core::WindowSize;
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
use yapgeir_physics_2d::simple::KinematicBody;
use yapgeir_realm::{Realm, Res, ResMut};
use yapgeir_renderer_2d::{
    quad_index_buffer::QuadIndexBuffer,
    sprite_renderer::{DrawRegion, SpriteRenderer, TextureRegion},
    NdcProjection,
};
use yapgeir_sdl::SdlSettings;
use yapgeir_sdl_graphics::SdlWindowBackend;
use yapgeir_world_2d::{DrawQuad, Drawable, SpriteSheet, Transform};
use yapgeir_world_2d_sprites::animation::{AnimationSequenceKey, AnimationStorage, Animator};

pub type GraphicsAdapter = Gles<SdlWindowBackend>;

const BATCH: usize = 5_000;

fn main() {
    let mut realm = Realm::default();

    realm
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
        // Game logic system
        .add_system(flip_direction)
        .add_system(spawn_entities_on_left_click)
        .add_system(despawn_entities_on_right_click)
        // Manages animation frame changes
        .add_plugin(yapgeir_world_2d_sprites::animation::plugin)
        // Update drawable data for rendering
        .add_plugin(yapgeir_world_2d_sprites::sprites::plugin)
        // Manage translation changes according to velocity
        .add_plugin(yapgeir_physics_2d::simple::plugin)
        // Sets up resources for rendering pipeline, and a system that will do actual rendering
        .add_plugin(initialize_rendering::<GraphicsAdapter>)
        .add_plugin(initialize_animations)
        // Initializes entities in ECS
        .run_system(|mut world: ResMut<World>, animations: Res<Animations>| {
            for _ in 0..BATCH {
                spawn_entity(
                    &mut world,
                    &animations,
                    Transform::new(
                        Isometry2::translation(
                            rand::random::<f32>() * 600. - 300.,
                            rand::random::<f32>() * 600. - 300.,
                        ),
                        None,
                    ),
                );
            }
        });

    realm.run();
}

fn spawn_entity(world: &mut World, animations: &Animations, transform: Transform) {
    world.spawn((
        transform,
        KinematicBody::new(
            Vector2::new(
                rand::random::<f32>() * 600. - 300.,
                rand::random::<f32>() * 600. - 300.,
            ),
            Vector2::default(),
        ),
        Animator::new(animations.player),
    ));
}

fn flip_direction(mut world: ResMut<World>, window_size: Res<WindowSize>) {
    for (_, (transform, kinematic_body)) in
        world.query_mut::<(&mut Transform, &mut KinematicBody)>()
    {
        let translation = &mut transform.isometry.translation;
        let velocity = &mut kinematic_body.velocity;

        if translation.x > window_size.w as f32 / 2. {
            velocity.x = -1. * velocity.x.abs();
        } else if translation.x < -(window_size.w as f32 / 2.) {
            velocity.x = velocity.x.abs();
        }

        if translation.y > window_size.h as f32 / 2. {
            velocity.y = -1. * velocity.y.abs();
        } else if translation.y < -(window_size.h as f32 / 2.) {
            velocity.y = velocity.y.abs();
        }
    }
}

fn window_to_world(position: Axial<i32>, window_size: WindowSize) -> Vector2<f32> {
    let x = position.x as f32 - (window_size.w as f32 / 2.);
    let y = -(position.y as f32 - (window_size.h as f32 / 2.));

    Vector2::new(x, y)
}

fn spawn_entities_on_left_click(
    mut world: ResMut<World>,
    mouse_button_events: Res<Events<MouseButtonEvent>>,
    window_size: Res<WindowSize>,
    animations: Res<Animations>,
) {
    let left_button_mouse_down_events = mouse_button_events
        .iter()
        .filter(|e| e.action == ButtonAction::Down && e.button == MouseButton::Left);

    for e in left_button_mouse_down_events {
        let position = window_to_world(e.coordinate, *window_size);

        for _ in 0..BATCH {
            spawn_entity(
                &mut world,
                &animations,
                Transform::new(Isometry2::translation(position.x, position.y), None),
            );
        }
    }
}

fn despawn_entities_on_right_click(
    mut world: ResMut<World>,
    mouse_button_events: Res<Events<MouseButtonEvent>>,
) {
    let right_button_mouse_down_events = mouse_button_events
        .iter()
        .filter(|e| e.action == ButtonAction::Down && e.button == MouseButton::Right)
        .last();

    if right_button_mouse_down_events.is_none() {
        return;
    }

    let mut entities = world
        .query::<&Drawable>()
        .iter()
        .map(|(entity, _)| entity)
        .take(BATCH)
        .collect::<Vec<_>>();

    for entity in entities.drain(0..) {
        world.despawn(entity).expect("Unable to despawn entity");
    }
}

fn initialize_rendering<G: Graphics>(realm: &mut Realm) {
    realm
        .add_plugin(yapgeir_renderer_2d::plugin::<G>)
        .initialize_resource_with(|graphics: Res<G>| -> G::Texture {
            let (image, size) = decode_png(include_bytes!("assets/sheet.png")).unwrap();
            let texture = graphics.new_texture(PixelFormat::Rgba, size, Some(&image));

            texture
        })
        .initialize_resource_with(
            |graphics: Res<G>, quad_index_buffer: Res<QuadIndexBuffer<G>>| -> SpriteRenderer<G> {
                SpriteRenderer::new(graphics.deref(), quad_index_buffer.clone())
            },
        )
        .add_system(render::<G>);
}

/// A resource that keeps ids of loaded animations
pub struct Animations {
    player: AnimationSequenceKey,
}

fn initialize_animations(realm: &mut Realm) {
    realm.initialize_resource_with(|mut animation_storage: ResMut<AnimationStorage>| {
        let atlas = SpriteSheet::new([64 * 3, 64], [64, 64]);

        let player = animation_storage.insert(
            "player",
            AnimationSequence(vec![Animation {
                frames: (0..3).map(|i| atlas.drawable(i, 0)).collect(),
                kind: AnimationKind::Loop,
                frame_time: 0.16,
            }]),
        );

        Animations { player }
    });
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

    sprite_renderer.batch(
        &fb,
        Matrix3::identity().into(),
        NdcProjection::Center,
        Sampler::nearest(&texture),
        |batch| {
            for (_, (draw_quad, drawable)) in world.query::<(&DrawQuad, &Drawable)>().iter() {
                batch.draw_sprite(
                    DrawRegion::Quad(**draw_quad),
                    TextureRegion::TexelsBox2D(drawable.sprite.sub_texture),
                    0,
                );
            }
        },
    );

    graphics.swap_buffers();
}
