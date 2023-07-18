use std::ops::Deref;

use derive_more::{Deref, DerefMut};
use hecs::World;
use nalgebra::Matrix3;
use yapgeir_assets::png::decode_png;
use yapgeir_core::{Delta, ScreenPpt, WindowSize};
use yapgeir_egui_sdl::{Egui, EguiRenderer};
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
use yapgeir_inspector_egui::draw_entity;
use yapgeir_realm::{Realm, Res, ResMut};
use yapgeir_reflection::{
    bevy_reflect::{self, Reflect},
    RealmExtensions, Reflection,
};
use yapgeir_renderer_2d::{
    quad_index_buffer::QuadIndexBuffer,
    sprite_renderer::{DrawRegion, SpriteRenderer, TextureRegion},
    NdcProjection,
};
use yapgeir_sdl::SdlSettings;
use yapgeir_sdl_graphics::SdlWindowBackend;

pub type GraphicsAdapter = Gles<SdlWindowBackend>;

fn main() {
    let mut realm = Realm::default();

    realm
        .add_plugin(yapgeir_inspector_egui::plugin)
        .register_type::<Position>()
        .register_type::<Velocity>()
        // Creates SDL window, initializes input, Delta and Frame.
        .add_plugin(yapgeir_sdl::plugin(SdlSettings {
            window_size: WindowSize::new(600, 400),
            ..SdlSettings::default()
        }))
        // Prints FPS stats to stdout
        .add_plugin(yapgeir_core::frame_stats::plugin)
        // Creates graphics context (in this case GLES2)
        .add_plugin(yapgeir_sdl_graphics::plugin::<GraphicsAdapter>)
        .add_plugin(yapgeir_egui_sdl::plugin::<GraphicsAdapter, _, _>(
            egui_update,
        ))
        .initialize_resource::<World>()
        // Initializes entities in ECS
        .run_system(|mut world: ResMut<World>| {
            for _ in 0..4 {
                world.spawn((
                    Position([
                        rand::random::<f32>() * 200. - 100.,
                        rand::random::<f32>() * 200. - 100.,
                    ]),
                    Velocity([
                        rand::random::<f32>() * 600. - 300.,
                        rand::random::<f32>() * 600. - 300.,
                    ]),
                ));
            }
        })
        // Game logic system
        .add_system(move_tile)
        .add_system(spawn_tile_on_left_click)
        .add_system(despawn_tile_on_right_click)
        // Sets up resources for rendering pipeline, and a system that will do actual rendering
        .add_plugin(initialize_rendering::<GraphicsAdapter>);

    realm.run();
}

#[derive(Debug, Default, Clone, Copy, Deref, DerefMut, Reflect)]
struct Position([f32; 2]);

#[derive(Debug, Default, Clone, Copy, Deref, DerefMut, Reflect)]
struct Velocity([f32; 2]);

fn move_tile(mut world: ResMut<World>, delta: Res<Delta>, window_size: Res<WindowSize>) {
    for (_, (position, velocity)) in world.query_mut::<(&mut Position, &mut Velocity)>() {
        position[0] += velocity[0] * delta.0;
        position[1] += velocity[1] * delta.0;

        if position[0] > window_size.w as f32 / 2. {
            velocity[0] = -1. * velocity[0].abs();
        } else if position.0[0] < -(window_size.w as f32 / 2.) {
            velocity[0] = velocity[0].abs();
        }

        if position[1] > window_size.h as f32 / 2. {
            velocity[1] = -1. * velocity[1].abs();
        } else if position[1] < -(window_size.h as f32 / 2.) {
            velocity[1] = velocity[1].abs();
        }
    }
}

fn window_to_world(position: Axial<i32>, window_size: WindowSize) -> Position {
    let x = position.x as f32 - (window_size.w as f32 / 2.);
    let y = -(position.y as f32 - (window_size.h as f32 / 2.));

    Position([x, y])
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
            Velocity([
                rand::random::<f32>() * 200. - 100.,
                rand::random::<f32>() * 600. - 300.,
            ]),
        ));
    }
}

fn is_in_rectangle(center: Position, point: Position, side_length: f32) -> bool {
    let half_side = side_length / 2.0;
    if point[0] < center[0] - half_side || point[0] > center[0] + half_side {
        return false;
    }
    if point[1] < center[1] - half_side || point[1] > center[1] + half_side {
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

fn egui_update(
    mut gui: ResMut<Egui>,
    mut mouse: ResMut<Events<MouseButtonEvent>>,
    reflection: Res<Reflection>,
    world: Res<World>,
) {
    let ctx = gui.context();

    if ctx.is_pointer_over_area() {
        mouse.clear();
    }

    egui::Window::new("Entities")
        .min_width(200.)
        .default_width(200.)
        .scroll2([false, true])
        .show(&ctx, |ui| {
            for entity in world.iter() {
                draw_entity(&reflection, ui, entity);
            }
            ui.allocate_space(ui.available_size());
        });
}

fn render<G: Graphics>(
    mut sprite_renderer: ResMut<SpriteRenderer<G>>,
    mut gui: Option<ResMut<EguiRenderer<G>>>,
    graphics: Res<G>,
    texture: Res<G::Texture>,
    world: Res<World>,
    screen_ppt: Res<ScreenPpt>,
) {
    let fb = graphics.default_frame_buffer();
    fb.clear(
        None,
        Some(yapgeir_graphics_hal::Rgba::new(0., 0., 0., 1.)),
        Some(0.),
        None,
    );

    // Draw sprites
    sprite_renderer.batch(
        &fb,
        Matrix3::identity().into(),
        NdcProjection::Center,
        Sampler::nearest(&texture),
        |batch| {
            for (_, tile) in world.query::<&Position>().iter() {
                batch.draw_sprite(DrawRegion::Point(tile.0), TextureRegion::Full, 0);
            }
        },
    );

    // Draw egui
    if let Some(gui) = gui.as_mut() {
        yapgeir_egui_sdl::render(gui, &fb, screen_ppt.to_owned());
    }

    graphics.swap_buffers();
}
