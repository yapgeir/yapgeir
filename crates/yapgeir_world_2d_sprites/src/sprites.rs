use std::ops::Div;

use derive_more::{Deref, DerefMut};
use hecs::{Entity, With, Without, World};
use nalgebra::Point;
use yapgeir_realm::{Realm, Res, ResMut};
use yapgeir_world_2d::{
    Dirty, DrawQuad, Drawable, Flip, Static, Transform, TransformPpt, WorldCamera,
};

#[cfg(feature = "reflection")]
use yapgeir_reflection::RealmExtensions;

/// Applies transformation matrix to a Drawable, updating a DrawQuad.
fn update_model(
    ppt: &TransformPpt,
    (_, (transform, drawable, model)): (Entity, (&Transform, &Drawable, &mut DrawQuad)),
) {
    *model = drawable
        .sprite
        .boundaries
        .points()
        .map(|mut p| {
            match transform.flip {
                Some(Flip::X) => p[0] = -p[0],
                Some(Flip::Y) => p[1] = -p[1],
                None => (),
            };

            transform
                .isometry
                .transform_point(&Point::from(p).div(**ppt))
                .into()
        })
        .into();
}

/// This resource is used to reduce allocations.
/// Essentially this is a cached-forever Vec, that is used
/// to update/remove components in systems.
#[derive(Default, Deref, DerefMut)]
struct SpritesEntityCache(Vec<Entity>);

fn add_draw_quads(mut world: ResMut<World>, mut cache: ResMut<SpritesEntityCache>) {
    // Add a DrawQuad to all non-static entities
    world
        .query::<Without<Without<(&Transform, &Drawable), &DrawQuad>, &Static>>()
        .iter()
        .for_each(|(e, _)| {
            cache.push(e);
        });

    for e in cache.drain(0..) {
        world
            .insert_one(e.clone(), DrawQuad::default())
            .expect("Unable to insert Drawable for entity");
    }

    // Add a DrawQuad and Dirty to all static entities. Since we've already iterated
    // all non-static ones, the only remaining ones are static.
    world
        .query::<Without<(&Transform, &Drawable), &DrawQuad>>()
        .iter()
        .for_each(|(e, _)| {
            cache.push(e);
        });

    for e in cache.drain(0..) {
        world
            .insert(e.clone(), (DrawQuad::default(), Dirty))
            .expect("Unable to insert Drawable and Dirty for entity");
    }
}

fn update_quads(
    mut world: ResMut<World>,
    mut cache: ResMut<SpritesEntityCache>,
    ppt: Option<Res<TransformPpt>>,
) {
    let ppt = ppt.as_deref().cloned().unwrap_or_default();

    // Update non-static entities
    world
        .query::<Without<(&Transform, &Drawable, &mut DrawQuad), &Static>>()
        .iter()
        .for_each(|entity| update_model(&ppt, entity));

    // Update dirty static entities
    world
        .query::<With<With<(&Transform, &Drawable, &mut DrawQuad), &Static>, &Dirty>>()
        .iter()
        .for_each(|entity| {
            cache.push(entity.0);
            update_model(&ppt, entity);
        });

    // Remove dirty flag
    for e in cache.drain(0..) {
        world
            .remove_one::<Dirty>(e.clone())
            .expect("Unable to insert Drawable for entity");
    }
}

pub fn plugin(realm: &mut Realm) {
    #[cfg(feature = "reflection")]
    realm
        .register_type::<yapgeir_world_2d::DrawQuad>()
        .register_type::<yapgeir_world_2d::Drawable>()
        .register_type::<yapgeir_world_2d::Static>()
        .register_type::<yapgeir_world_2d::Dirty>()
        .register_type::<yapgeir_world_2d::Flip>()
        .register_type::<yapgeir_world_2d::Transform>()
        .register_type::<yapgeir_world_2d::Sprite>();

    realm
        .initialize_resource::<WorldCamera>()
        .initialize_resource::<SpritesEntityCache>()
        .add_system(add_draw_quads)
        .add_system(update_quads);
}
